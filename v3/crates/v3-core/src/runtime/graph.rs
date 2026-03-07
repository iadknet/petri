use crate::config::RuntimeConfig;
use crate::contracts::InputReference;
use crate::creature::genome::{GraphBackendDef, GraphInternalNode, GraphNodeKind};
use crate::creature::state::GraphRuntimeState;
use crate::runtime::graph_effects::apply_graph_effects;
use crate::runtime::inputs::{resolve_input, ResolveCtx};
use crate::runtime::plasticity::hebbian;
use crate::runtime::plasticity::traces;
use crate::runtime::types::{sanitize_f32, MeshSideOutputs, NodeResult};
use crate::sensors::perception::SensorSnapshot;

// ─── Tracer trait ────────────────────────────────────────────────────────────

/// Callback trait for instrumenting the graph relaxation loop.
///
/// Two implementations exist:
/// - [`NoopTracer`] — zero-cost (all methods are `#[inline]` no-ops), used by
///   the production [`execute_graph_node`] path.
/// - [`super::traced_graph::RecordingTracer`] — records per-pass trace data for
///   the Execution Sampler.
///
/// Generic over `GraphTracer` so the compiler can monomorphize
/// `execute_graph_impl` separately for each tracer, eliminating all tracing
/// overhead on the hot path.
#[allow(clippy::too_many_arguments)]
pub(crate) trait GraphTracer {
    /// Called before each relaxation pass. Record energy cost.
    fn on_pass_start(&mut self, pass: u32, pass_cost: f32, energy_after: f32);
    /// Called after each node evaluation. Record inputs, state, output.
    fn on_node_eval(
        &mut self,
        node_index: usize,
        kind: &GraphNodeKind,
        weighted_inputs: &[f32],
        weighted_sum: f32,
        state_before: f32,
        state_after: f32,
        output: f32,
    );
    /// Called after each pass. Record convergence delta.
    fn on_pass_end(&mut self, delta: f32);
    /// Called once when the relaxation loop finishes (both normal and exhaustion paths).
    ///
    /// On the exhaustion path, `curr_outputs` reflects the state after the last
    /// *completed* pass (or zeros if exhaustion occurred before any pass completed).
    /// `converged` is always `false` on exhaustion.
    fn on_finish(&mut self, curr_outputs: &[f32], stable_passes: u32, converged: bool);
}

/// Zero-cost tracer used by the production path.
///
/// All methods are `#[inline]` no-ops — the compiler eliminates them entirely
/// when `execute_graph_impl` is monomorphized with `NoopTracer`.
pub(crate) struct NoopTracer;

impl GraphTracer for NoopTracer {
    #[inline]
    fn on_pass_start(&mut self, _: u32, _: f32, _: f32) {}
    #[inline]
    fn on_node_eval(
        &mut self,
        _: usize,
        _: &GraphNodeKind,
        _: &[f32],
        _: f32,
        _: f32,
        _: f32,
        _: f32,
    ) {
    }
    #[inline]
    fn on_pass_end(&mut self, _: f32) {}
    #[inline]
    fn on_finish(&mut self, _: &[f32], _: u32, _: bool) {}
}

// ─── Shared helpers ──────────────────────────────────────────────────────────

/// Immutable context for resolving `InputRef` nodes during graph evaluation.
pub(crate) struct EvalCtx<'a> {
    pub(crate) input_refs: &'a [InputReference],
    pub(crate) resolve: ResolveCtx<'a>,
    pub(crate) shared_memory: &'a [f32; 16],
    pub(crate) prev_shared_memory: &'a [f32; 16],
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
        // Shared memory slot nodes.
        GraphNodeKind::ReadSlot(slot_idx) => ctx.shared_memory[(*slot_idx as usize) % 16] + wsum,
        GraphNodeKind::ReadSlotPrev(slot_idx) => {
            ctx.prev_shared_memory[(*slot_idx as usize) % 16] + wsum
        }
        // WriteSlot/ClearSlot: deferred effect (post-convergence commit).
        // During relaxation they behave as passthrough.
        GraphNodeKind::WriteSlot(_) => wsum,
        GraphNodeKind::ClearSlot(_) => 0.0,
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

// ─── Unified graph execution ─────────────────────────────────────────────────

/// Core graph relaxation loop, generic over a [`GraphTracer`].
///
/// Both [`execute_graph_node`] and [`super::traced_graph::execute_graph_node_traced`]
/// delegate to this function. The tracer callbacks are monomorphized away for
/// [`NoopTracer`], making the production path zero-cost.
///
/// The tracer's [`on_finish`](GraphTracer::on_finish) callback receives
/// `curr_outputs`, `stable_passes`, and `converged` on all return paths
/// (both normal completion and energy exhaustion), so the caller can build
/// trace metadata without fragile reconstruction.
#[allow(clippy::too_many_arguments)]
pub(crate) fn execute_graph_impl<T: GraphTracer>(
    tracer: &mut T,
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
    shared_memory: &mut [f32; 16],
    prev_shared_memory: &[f32; 16],
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

    // Check if any node uses plasticity learning and prepare weights if so.
    let use_plasticity = hebbian::has_any_hebbian(def);
    if use_plasticity {
        hebbian::ensure_hebbian_weights(def, node_idx, &mut graph_runtime.plasticity_weights);
    }

    // Check if any node uses reward-modulated plasticity and prepare traces.
    let use_reward_modulated = traces::has_any_reward_modulated(def);
    if use_reward_modulated {
        traces::ensure_eligibility_traces(def, node_idx, &mut graph_runtime.eligibility_traces);
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

    for pass in 0..max_passes {
        // Charge energy BEFORE evaluating this pass.
        let pass_cost = config.graph_node_base_cost * node_count as f32;
        *energy -= pass_cost;
        if *energy <= 0.0 {
            tracer.on_finish(&curr_outputs, stable_passes, false);
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

        tracer.on_pass_start(pass, pass_cost, *energy);

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
            shared_memory,
            prev_shared_memory,
        };

        for current_idx in 0..node_count {
            let node = &def.internal_nodes[current_idx];

            // Use learned weights for plasticity nodes, genome weights otherwise.
            if use_plasticity && node.plasticity.is_some() {
                let learned = &graph_runtime.plasticity_weights[node_idx][current_idx];
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
            let state_before = node_state;

            curr_outputs[current_idx] = sanitize_f32(evaluate_kind(
                &node.kind,
                &w_inputs_buf,
                wsum,
                &ctx,
                &mut node_state,
            ));

            // Persist any state mutation from stateful operators.
            graph_runtime.node_state[node_idx][current_idx] = node_state;

            tracer.on_node_eval(
                current_idx,
                &node.kind,
                &w_inputs_buf,
                wsum,
                state_before,
                node_state,
                curr_outputs[current_idx],
            );
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

        tracer.on_pass_end(delta);

        if stable_passes >= req_stable {
            break;
        }
    }

    let converged = stable_passes >= req_stable;
    tracer.on_finish(&curr_outputs, stable_passes, converged);

    // Apply plasticity weight updates after convergence.
    // NOTE: If energy goes negative here, we do not roll back weight changes or
    // return exhausted. Acceptable while plasticity_update_cost defaults to 0.0.
    // When a nonzero cost is introduced, add exhaustion handling here.
    if use_plasticity {
        let plasticity_cost = hebbian::apply_hebbian_updates(
            def,
            node_idx,
            &mut graph_runtime.plasticity_weights,
            &curr_outputs,
            config.plasticity_update_cost,
        );
        *energy -= plasticity_cost;
    }

    // Update eligibility traces for reward-modulated nodes.
    // Traces accumulate Hebbian deltas with decay; actual weight updates happen
    // in Phase 2.5 (reward::apply_reward_modulated_updates).
    if use_reward_modulated {
        traces::update_eligibility_traces(
            def,
            node_idx,
            &mut graph_runtime.eligibility_traces,
            &graph_runtime.plasticity_weights,
            &curr_outputs,
        );
    }

    // Build NodeResult via the shared 3-phase effect pass.
    let result = apply_graph_effects(
        def,
        &curr_outputs,
        upstream_slots,
        side_outputs,
        shared_memory,
    );

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
    shared_memory: &mut [f32; 16],
    prev_shared_memory: &[f32; 16],
) -> NodeResult {
    execute_graph_impl(
        &mut NoopTracer,
        def,
        input_refs,
        upstream_slots,
        energy,
        energy_consumed,
        node_idx,
        graph_runtime,
        sensors,
        config,
        side_outputs,
        shared_memory,
        prev_shared_memory,
    )
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
