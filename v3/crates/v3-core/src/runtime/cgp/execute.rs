use crate::config::RuntimeConfig;
use crate::contracts::InputReference;
use crate::creature::genome::cgp::{CgpGraphBackendDef, ComputeNodeKind};
use crate::creature::state::GraphRuntimeState;
use crate::runtime::cgp::effects::{apply_cgp_graph_effects, CgpEffectsTrace};
use crate::runtime::cgp::eval::{evaluate_compute_kind, sanitize_output};
use crate::runtime::cgp::sources::collect_cgp_weighted_inputs;
use crate::runtime::inputs::ResolveCtx;
use crate::runtime::plasticity::hebbian;
use crate::runtime::plasticity::traces;
use crate::runtime::routing::RouteDecision;
use crate::runtime::types::{MeshSideOutputs, NodeResult};
use crate::sensors::perception::SensorSnapshot;

/// Callback trait for instrumenting the graph relaxation loop.
#[allow(clippy::too_many_arguments)]
pub(crate) trait GraphTracer {
    fn on_pass_start(&mut self, pass: u32, pass_cost: f32, energy_after: f32);
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
    fn on_pass_end(&mut self, delta: f32);
    fn on_finish(&mut self, curr_outputs: &[f32], stable_passes: u32, converged: bool);
    fn on_effects(&mut self, effects: CgpEffectsTrace);
}

/// Zero-cost tracer used by the production path.
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
    #[inline]
    fn on_effects(&mut self, _: CgpEffectsTrace) {}
}

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
        return NodeResult::halted(
            *upstream_slots,
            RouteDecision::CgpNormalized { raw_value: 0.0 },
        );
    }

    if graph_runtime.node_state.len() <= node_idx {
        graph_runtime.node_state.resize(node_idx + 1, Vec::new());
    }

    let mut prev_outputs = std::mem::take(&mut graph_runtime.scratch_prev);
    let mut curr_outputs = std::mem::take(&mut graph_runtime.scratch_curr);
    let mut state_backup = std::mem::take(&mut graph_runtime.scratch_backup);

    state_backup.clear();
    state_backup.extend_from_slice(&graph_runtime.node_state[node_idx]);

    let state_vec = &mut graph_runtime.node_state[node_idx];
    if state_vec.len() < node_count {
        state_vec.resize(node_count, 0.0);
    }

    let use_plasticity = hebbian::has_any_hebbian(def);
    if use_plasticity {
        hebbian::ensure_hebbian_weights(def, node_idx, &mut graph_runtime.plasticity_weights);
    }

    let use_reward_modulated = traces::has_any_reward_modulated(def);
    if use_reward_modulated {
        traces::ensure_eligibility_traces(def, node_idx, &mut graph_runtime.eligibility_traces);
    }

    let max_passes = config.max_graph_relax_iters;
    let epsilon = config.graph_convergence_epsilon;
    let req_stable = config.graph_convergence_stable_passes;

    prev_outputs.clear();
    prev_outputs.resize(node_count, 0.0);
    curr_outputs.clear();
    curr_outputs.resize(node_count, 0.0);
    let mut stable_passes: u32 = 0;
    let mut w_inputs_buf = std::mem::take(&mut graph_runtime.scratch_w_inputs);

    for pass in 0..max_passes {
        let pass_cost = config.graph_node_base_cost * node_count as f32;
        *energy -= pass_cost;
        if *energy <= 0.0 {
            tracer.on_finish(&curr_outputs, stable_passes, false);
            graph_runtime.node_state[node_idx].clone_from(&state_backup);
            restore_scratch(
                graph_runtime,
                prev_outputs,
                curr_outputs,
                state_backup,
                w_inputs_buf,
            );
            return NodeResult::exhausted_with_route(RouteDecision::CgpNormalized {
                raw_value: 0.0,
            });
        }

        tracer.on_pass_start(pass, pass_cost, *energy);

        let resolve_ctx = ResolveCtx {
            sensors,
            upstream_slots,
            energy: *energy,
            energy_consumed,
            action_queue: &side_outputs.action_queue,
        };

        for current_idx in 0..node_count {
            let node = &def.compute_nodes[current_idx];

            if use_plasticity && node.plasticity.is_some() {
                let learned = &graph_runtime.plasticity_weights[node_idx][current_idx];
                hebbian::collect_weighted_inputs_hebbian(
                    node,
                    current_idx,
                    node_count,
                    &prev_outputs,
                    &curr_outputs,
                    input_refs,
                    &resolve_ctx,
                    shared_memory,
                    prev_shared_memory,
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

            let mut node_state = graph_runtime.node_state[node_idx][current_idx];
            let state_before = node_state;

            curr_outputs[current_idx] = sanitize_output(evaluate_compute_kind(
                &node.kind,
                &w_inputs_buf,
                wsum,
                &mut node_state,
            ));

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

    if use_plasticity {
        let queue_snapshot = side_outputs.action_queue.clone();
        let post_ctx = ResolveCtx {
            sensors,
            upstream_slots,
            energy: *energy,
            energy_consumed,
            action_queue: &queue_snapshot,
        };
        let plasticity_cost = hebbian::apply_hebbian_updates(
            def,
            node_idx,
            &mut graph_runtime.plasticity_weights,
            &curr_outputs,
            input_refs,
            &post_ctx,
            shared_memory,
            prev_shared_memory,
            config.plasticity_update_cost,
        );
        *energy -= plasticity_cost;
        if *energy <= 0.0 {
            restore_scratch(
                graph_runtime,
                prev_outputs,
                curr_outputs,
                state_backup,
                w_inputs_buf,
            );
            return NodeResult::exhausted_with_route(RouteDecision::CgpNormalized {
                raw_value: 0.0,
            });
        }
    }

    if use_reward_modulated {
        let queue_snapshot = side_outputs.action_queue.clone();
        let post_ctx = ResolveCtx {
            sensors,
            upstream_slots,
            energy: *energy,
            energy_consumed,
            action_queue: &queue_snapshot,
        };
        traces::update_eligibility_traces(
            def,
            node_idx,
            &mut graph_runtime.eligibility_traces,
            &graph_runtime.plasticity_weights,
            &curr_outputs,
            input_refs,
            &post_ctx,
            shared_memory,
            prev_shared_memory,
        );
    }

    let queue_snapshot = side_outputs.action_queue.clone();
    let effects_resolve_ctx = ResolveCtx {
        sensors,
        upstream_slots,
        energy: *energy,
        energy_consumed,
        action_queue: &queue_snapshot,
    };
    let (result, effects_trace) = apply_cgp_graph_effects(
        def,
        &curr_outputs,
        input_refs,
        &effects_resolve_ctx,
        upstream_slots,
        side_outputs,
        shared_memory,
        prev_shared_memory,
    );
    tracer.on_effects(effects_trace);

    restore_scratch(
        graph_runtime,
        prev_outputs,
        curr_outputs,
        state_backup,
        w_inputs_buf,
    );

    result
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn execute_graph_node(
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
