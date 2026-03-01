use crate::config::RuntimeConfig;
use crate::contracts::InputReference;
use crate::creature::genome::{GraphBackendDef, GraphInternalNode, GraphNodeKind};
use crate::creature::state::GraphRuntimeState;
use crate::runtime::graph_effects::apply_graph_effects;
use crate::runtime::hebbian;
use crate::runtime::inputs::{resolve_input, ResolveCtx};
use crate::runtime::types::{sanitize_f32, MeshSideOutputs, NodeResult};
use crate::sensors::perception::SensorSnapshot;

/// Immutable context for resolving `InputRef` nodes during graph evaluation.
pub(crate) struct EvalCtx<'a> {
    pub(crate) input_refs: &'a [InputReference],
    pub(crate) resolve: ResolveCtx<'a>,
}

/// Evaluate one internal graph node's kind, returning the scalar output.
///
/// `w_inputs` holds the per-edge weighted values (source_value * weight).
/// `wsum` is `w_inputs.iter().sum()`.
/// `state` is the node's mutable persistent scalar state (for stateful operators).
#[inline]
pub(crate) fn evaluate_kind(
    kind: &GraphNodeKind,
    w_inputs: &[f32],
    wsum: f32,
    ctx: &EvalCtx<'_>,
    state: &mut f32,
) -> f32 {
    match kind {
        GraphNodeKind::InputRef { ref_idx, sub_idx } => {
            let resolved = if (*ref_idx as usize) < ctx.input_refs.len() {
                resolve_input(&ctx.input_refs[*ref_idx as usize], *sub_idx, &ctx.resolve)
            } else {
                0.0
            };
            resolved + wsum
        }
        GraphNodeKind::Constant(f) => *f,
        GraphNodeKind::Add => wsum,
        GraphNodeKind::Multiply => w_inputs.iter().copied().product::<f32>(),
        GraphNodeKind::Negate => -wsum,
        GraphNodeKind::Abs => wsum.abs(),
        GraphNodeKind::Min => w_inputs.iter().copied().reduce(f32::min).unwrap_or(0.0),
        GraphNodeKind::Max => w_inputs.iter().copied().reduce(f32::max).unwrap_or(0.0),
        GraphNodeKind::Threshold(t) => {
            if wsum > *t {
                1.0
            } else {
                0.0
            }
        }
        GraphNodeKind::GreaterThan => {
            let a = w_inputs.first().copied().unwrap_or(0.0);
            let b = w_inputs.get(1).copied().unwrap_or(0.0);
            if a > b {
                1.0
            } else {
                0.0
            }
        }
        GraphNodeKind::Sigmoid => 1.0 / (1.0 + (-wsum).exp()),
        GraphNodeKind::Tanh => wsum.tanh(),
        GraphNodeKind::Relu => wsum.max(0.0),
        GraphNodeKind::Select => {
            let cond = w_inputs.first().copied().unwrap_or(0.0);
            if cond >= 0.5 {
                w_inputs.get(1).copied().unwrap_or(0.0)
            } else {
                w_inputs.get(2).copied().unwrap_or(0.0)
            }
        }
        GraphNodeKind::Clamp01 => wsum.clamp(0.0, 1.0),
        GraphNodeKind::WeightedSum => wsum,
        GraphNodeKind::DecayIntegrator(a) => {
            let a_c = a.clamp(0.0, 1.0);
            *state = (1.0 - a_c) * *state + a_c * wsum;
            *state
        }
        GraphNodeKind::Momentum(b) => {
            let b_c = b.clamp(0.0, 1.0);
            *state = b_c * *state + (1.0 - b_c) * wsum;
            *state
        }
        GraphNodeKind::Oscillator(f) => {
            let f_c = f.clamp(0.0, 8.0);
            *state = (*state + f_c).fract();
            (2.0 * std::f32::consts::PI * *state).sin()
        }
        GraphNodeKind::AdaptiveGain => {
            *state = (*state + 0.01 * wsum).clamp(0.1, 2.0);
            *state * wsum
        }
        // Output writers: their deferred effect (on output_slots / route_target_idx) is
        // applied post-loop. During the loop their curr_outputs slot just holds wsum.
        GraphNodeKind::CustomOutput(_) => wsum,
        GraphNodeKind::RouterOutput => wsum,
        // Action-queue outputs: deferred effect applied post-convergence.
        // During relaxation they behave as passthrough (wsum).
        GraphNodeKind::WriteActionMeta(_) => wsum,
        GraphNodeKind::PushAction(_) => wsum,
        GraphNodeKind::PopAction => wsum,
        GraphNodeKind::ExecuteActionQueue => wsum,
    }
}

/// Collect the weighted input values for a single internal node into `buf`.
///
/// Implements Gauss-Seidel update order: sources already updated in this pass
/// (`source_idx < current_idx`) use `curr_outputs`; sources not yet updated
/// (or self-loops) use `prev_outputs`.
///
/// `buf` is cleared and filled with one entry per input edge. The caller
/// should allocate `buf` once and reuse it across nodes/passes.
#[inline]
pub(crate) fn collect_weighted_inputs(
    node: &GraphInternalNode,
    current_idx: usize,
    node_count: usize,
    prev_outputs: &[f32],
    curr_outputs: &[f32],
    buf: &mut Vec<f32>,
) {
    buf.clear();
    buf.extend(node.inputs.iter().map(|input| {
        let src = input.source_idx as usize;
        let source_value = if src >= node_count {
            0.0
        } else if src < current_idx {
            curr_outputs[src]
        } else {
            prev_outputs[src]
        };
        source_value * input.weight
    }));
}

/// Execute a graph-backend mesh node.
///
/// Runs the relaxation loop (Gauss-Seidel style), handles stateful operators,
/// charges energy per pass, and maps `CustomOutput`/`RouterOutput` nodes to the
/// returned [`NodeResult`].
///
/// # Energy semantics
/// Energy is charged **before** each pass. If energy drops to `<= 0` the
/// function restores the pre-call graph state snapshot and returns
/// [`NodeResult::exhausted`].
///
/// # Empty graph
/// If `def.internal_nodes` is empty, no energy is charged and the function
/// returns [`NodeResult::halted`] with the original `upstream_slots`.
// The signature is mandated by the v3 spec / mesh executor calling convention.
#[allow(clippy::too_many_arguments)]
pub fn execute_graph_node(
    def: &GraphBackendDef,
    input_refs: &[InputReference],
    upstream_slots: &[f32; 12],
    energy: &mut f32,
    energy_consumed: f32,
    node_idx: usize,
    graph_runtime: &mut GraphRuntimeState,
    sensors: &SensorSnapshot,
    config: &RuntimeConfig,
    side_outputs: &mut MeshSideOutputs,
) -> NodeResult {
    let node_count = def.internal_nodes.len();

    // Empty graph: no work, no energy charge.
    if node_count == 0 {
        return NodeResult::halted(*upstream_slots, 0.0);
    }

    // Ensure node_state has enough slots for this node index.
    if graph_runtime.node_state.len() <= node_idx {
        graph_runtime.node_state.resize(node_idx + 1, Vec::new());
    }

    // Extract scratch buffers (leaves empty Vecs in graph_runtime, avoids
    // borrow conflicts with graph_runtime.node_state).
    let mut prev_outputs = std::mem::take(&mut graph_runtime.scratch_prev);
    let mut curr_outputs = std::mem::take(&mut graph_runtime.scratch_curr);
    let mut state_backup = std::mem::take(&mut graph_runtime.scratch_backup);

    // Snapshot state for atomic rollback on energy exhaustion.
    state_backup.clear();
    state_backup.extend_from_slice(&graph_runtime.node_state[node_idx]);

    // Ensure the state Vec for this node is long enough.
    let state_vec = &mut graph_runtime.node_state[node_idx];
    if state_vec.len() < node_count {
        state_vec.resize(node_count, 0.0);
    }

    // Check if any node uses Hebbian learning and prepare weights if so.
    let use_hebbian = hebbian::has_any_hebbian(def);
    if use_hebbian {
        hebbian::ensure_hebbian_weights(def, node_idx, &mut graph_runtime.hebbian_weights);
    }

    let max_passes = config.max_graph_relax_iters;
    let epsilon = config.graph_convergence_epsilon;
    let req_stable = config.graph_convergence_stable_passes;

    // Prepare output buffers (reuses capacity from previous calls).
    prev_outputs.clear();
    prev_outputs.resize(node_count, 0.0);
    curr_outputs.clear();
    curr_outputs.resize(node_count, 0.0);
    let mut stable_passes: u32 = 0;
    let mut w_inputs_buf = std::mem::take(&mut graph_runtime.scratch_w_inputs);

    for _pass in 0..max_passes {
        // Charge energy BEFORE evaluating this pass.
        let pass_cost = config.graph_node_base_cost * node_count as f32;
        *energy -= pass_cost;
        if *energy <= 0.0 {
            // Restore state snapshot.
            graph_runtime.node_state[node_idx].clone_from(&state_backup);
            restore_scratch(
                graph_runtime,
                prev_outputs,
                curr_outputs,
                state_backup,
                w_inputs_buf,
            );
            return NodeResult::exhausted();
        }

        // Rebuild the eval context with the live energy value after the pass charge.
        let ctx = EvalCtx {
            input_refs,
            resolve: ResolveCtx {
                sensors,
                upstream_slots,
                energy: *energy,
                energy_consumed,
                action_queue: &side_outputs.action_queue,
            },
        };

        for current_idx in 0..node_count {
            let node = &def.internal_nodes[current_idx];

            // Use learned weights for Hebbian nodes, genome weights otherwise.
            if use_hebbian && node.hebbian.is_some() {
                let learned = &graph_runtime.hebbian_weights[node_idx][current_idx];
                hebbian::collect_weighted_inputs_hebbian(
                    node,
                    current_idx,
                    node_count,
                    &prev_outputs,
                    &curr_outputs,
                    learned,
                    &mut w_inputs_buf,
                );
            } else {
                collect_weighted_inputs(
                    node,
                    current_idx,
                    node_count,
                    &prev_outputs,
                    &curr_outputs,
                    &mut w_inputs_buf,
                );
            }
            let wsum: f32 = w_inputs_buf.iter().sum();

            // Load node-local state; write back after evaluate_kind updates it.
            let mut node_state = graph_runtime.node_state[node_idx][current_idx];

            curr_outputs[current_idx] = sanitize_f32(evaluate_kind(
                &node.kind,
                &w_inputs_buf,
                wsum,
                &ctx,
                &mut node_state,
            ));

            // Persist any state mutation from stateful operators.
            graph_runtime.node_state[node_idx][current_idx] = node_state;
        }

        // Convergence check: max absolute change across all outputs.
        let delta = prev_outputs
            .iter()
            .zip(curr_outputs.iter())
            .map(|(p, c)| (c - p).abs())
            .fold(0.0f32, f32::max);

        prev_outputs.copy_from_slice(&curr_outputs);

        if delta <= epsilon {
            stable_passes += 1;
        } else {
            stable_passes = 0;
        }
        if stable_passes >= req_stable {
            break;
        }
    }

    // Apply Hebbian weight updates after convergence.
    // NOTE: If energy goes negative here, we do not roll back weight changes or
    // return exhausted. Acceptable while hebbian_update_cost defaults to 0.0.
    // When a nonzero cost is introduced, add exhaustion handling here.
    if use_hebbian {
        let hebb_cost = hebbian::apply_hebbian_updates(
            def,
            node_idx,
            &mut graph_runtime.hebbian_weights,
            &curr_outputs,
            config.hebbian_update_cost,
        );
        *energy -= hebb_cost;
    }

    // Build NodeResult via the shared 3-phase effect pass.
    let result = apply_graph_effects(def, &curr_outputs, upstream_slots, side_outputs);

    // Restore scratch buffers before returning.
    restore_scratch(
        graph_runtime,
        prev_outputs,
        curr_outputs,
        state_backup,
        w_inputs_buf,
    );

    result
}

/// Restore scratch buffers to `GraphRuntimeState` (must be called on all return paths).
#[inline]
fn restore_scratch(
    graph_runtime: &mut GraphRuntimeState,
    prev: Vec<f32>,
    curr: Vec<f32>,
    backup: Vec<f32>,
    w_inputs: Vec<f32>,
) {
    graph_runtime.scratch_prev = prev;
    graph_runtime.scratch_curr = curr;
    graph_runtime.scratch_backup = backup;
    graph_runtime.scratch_w_inputs = w_inputs;
}

#[cfg(test)]
#[path = "tests/graph_tests.rs"]
mod tests;
