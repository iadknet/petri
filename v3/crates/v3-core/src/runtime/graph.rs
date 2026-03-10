use crate::config::RuntimeConfig;
use crate::contracts::InputReference;
use crate::creature::genome::cgp::{CgpGraphBackendDef, ComputeNodeKind};
use crate::creature::state::GraphRuntimeState;
use crate::runtime::cgp_graph::{
    collect_cgp_weighted_inputs, evaluate_compute_kind, sanitize_output,
};
use crate::runtime::cgp_graph_effects::apply_cgp_graph_effects;
use crate::runtime::inputs::ResolveCtx;
use crate::runtime::plasticity::hebbian;
use crate::runtime::plasticity::traces;
use crate::runtime::types::{MeshSideOutputs, NodeResult};
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
        kind: &ComputeNodeKind,
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
        _: &ComputeNodeKind,
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
    def: &CgpGraphBackendDef,
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
    let node_count = def.compute_nodes.len();

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

        // Rebuild resolve context each pass with live energy value.
        let resolve_ctx = ResolveCtx {
            sensors,
            upstream_slots,
            energy: *energy,
            energy_consumed,
            action_queue: &side_outputs.action_queue,
        };

        for current_idx in 0..node_count {
            let node = &def.compute_nodes[current_idx];

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
                collect_cgp_weighted_inputs(
                    &node.inputs,
                    current_idx,
                    node_count,
                    &prev_outputs,
                    &curr_outputs,
                    input_refs,
                    &resolve_ctx,
                    shared_memory,
                    prev_shared_memory,
                    &mut w_inputs_buf,
                );
            }
            let wsum: f32 = w_inputs_buf.iter().sum();

            // Load node-local state; write back after evaluate_compute_kind updates it.
            let mut node_state = graph_runtime.node_state[node_idx][current_idx];
            let state_before = node_state;

            curr_outputs[current_idx] = sanitize_output(evaluate_compute_kind(
                &node.kind,
                &w_inputs_buf,
                wsum,
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

    // Build NodeResult via the CGP 3-phase effect pass.
    // Use a snapshot of the action queue for the resolve context since the
    // effects pass will mutate the queue via side_outputs.
    let queue_snapshot = side_outputs.action_queue.clone();
    let effects_resolve_ctx = ResolveCtx {
        sensors,
        upstream_slots,
        energy: *energy,
        energy_consumed,
        action_queue: &queue_snapshot,
    };
    let result = apply_cgp_graph_effects(
        def,
        &curr_outputs,
        input_refs,
        &effects_resolve_ctx,
        upstream_slots,
        side_outputs,
        shared_memory,
        prev_shared_memory,
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

/// Execute a CGP graph-backend mesh node.
///
/// Runs the relaxation loop (Gauss-Seidel style) over compute nodes, handles
/// stateful operators, charges energy per pass, and applies the CGP 3-phase
/// effects pass (output sinks, action bank, execute gate) post-convergence.
///
/// # Energy semantics
/// Energy is charged **before** each pass. If energy drops to `<= 0` the
/// function restores the pre-call graph state snapshot and returns
/// [`NodeResult::exhausted`].
///
/// # Empty graph
/// If `def.compute_nodes` is empty, no energy is charged and the function
/// returns [`NodeResult::halted`] with the original `upstream_slots`.
#[allow(clippy::too_many_arguments)]
pub fn execute_graph_node(
    def: &CgpGraphBackendDef,
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

// Old graph tests removed — CGP graph tests live in runtime/cgp_graph.rs.
