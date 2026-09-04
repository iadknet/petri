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
use crate::runtime::routing::RouteGateMap;
use crate::runtime::types::{MeshSideOutputs, NodeResult, OUTPUT_SLOT_COUNT};
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
    upstream_slots: &[f32; OUTPUT_SLOT_COUNT],
    energy: &mut f32,
    energy_consumed: f32,
    reproductive_reserve: f32,
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
        return NodeResult::halted(*upstream_slots, RouteGateMap::default());
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
        // One relaxation pass entered, counted regardless of whether it completes.
        side_outputs.work_counters.graph_relax_iters += 1;

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
            return NodeResult::exhausted();
        }

        tracer.on_pass_start(pass, pass_cost, *energy);

        let resolve_ctx = ResolveCtx {
            sensors,
            upstream_slots,
            energy: *energy,
            energy_consumed,
            reproductive_reserve,
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
            reproductive_reserve,
            action_queue: &queue_snapshot,
        };
        let (plasticity_cost, plasticity_update_count) = hebbian::apply_hebbian_updates(
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
        side_outputs.work_counters.plasticity_updates += plasticity_update_count;
        *energy -= plasticity_cost;
        if *energy <= 0.0 {
            restore_scratch(
                graph_runtime,
                prev_outputs,
                curr_outputs,
                state_backup,
                w_inputs_buf,
            );
            return NodeResult::exhausted();
        }
    }

    if use_reward_modulated {
        let queue_snapshot = side_outputs.action_queue.clone();
        let post_ctx = ResolveCtx {
            sensors,
            upstream_slots,
            energy: *energy,
            energy_consumed,
            reproductive_reserve,
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
        reproductive_reserve,
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
pub(crate) fn execute_graph_node_with_reserve(
    def: &CgpGraphBackendDef,
    input_refs: &[InputReference],
    upstream_slots: &[f32; OUTPUT_SLOT_COUNT],
    energy: &mut f32,
    energy_consumed: f32,
    reproductive_reserve: f32,
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
        reproductive_reserve,
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

#[cfg(test)]
mod work_counter_tests {
    use crate::config::RuntimeConfig;
    use crate::contracts::NodeId;
    use crate::creature::genome::cgp::{
        ActionSlot, ActionSlotBehavior, CgpGraphBackendDef, ComputeNode, ComputeNodeKind,
        ExecuteGate, GraphEdge, GraphSource, WorldActionKind,
    };
    use crate::creature::genome::{
        BackendDef, CreatureGenome, HebbianRule, NodeGenome, PlasticityConfig,
    };
    use crate::creature::state::GraphRuntimeState;
    use crate::runtime::mesh::execute_creature_mesh;
    use crate::sensors::perception::{PerceptionSnapshot, SensorSnapshot};
    use crate::sensors::static_inputs::StaticInputs;
    use crate::sensors::typed_food::TypedFoodLocalSnapshot;

    fn empty_ss() -> SensorSnapshot {
        SensorSnapshot {
            local: StaticInputs {
                food_here: 0.0,
                neighbor_food: [0.0; 8],
                neighbor_barrier: [0.0; 8],
                neighbor_occupied: [0.0; 8],
                generation: 0.0,
                age_ticks: 0.0,
            },
            typed_local_food: TypedFoodLocalSnapshot::zeroed(1),
            perception: PerceptionSnapshot::zeroed(1),
        }
    }

    /// A single-node graph outputting a non-zero constant takes one extra
    /// pass to converge beyond `graph_convergence_stable_passes` (default 2):
    /// pass 1 computes a delta of 1.0 against the zero-initialized previous
    /// outputs (not stable), pass 2 repeats the same output (first stable
    /// pass), pass 3 repeats it again (second stable pass, loop breaks).
    #[test]
    fn single_node_graph_reports_known_relax_iters_and_no_plasticity() {
        let id0 = NodeId::new(0);
        let genome = CreatureGenome {
            entry_node_id: id0,
            nodes: vec![NodeGenome {
                node_id: id0,
                input_refs: vec![],
                backend_def: BackendDef::Graph(CgpGraphBackendDef {
                    compute_nodes: vec![ComputeNode {
                        kind: ComputeNodeKind::Constant(1.0),
                        inputs: vec![],
                        plasticity: None,
                    }],
                    output_sinks: vec![],
                    action_bank: vec![],
                    execute_gate: ExecuteGate { inputs: vec![] },
                }),
                targets: vec![],
            }],
        };

        let ss = empty_ss();
        let config = RuntimeConfig::default();
        let mut energy = 100.0f32;
        let mut smem = [0.0f32; 16];
        let prev_smem = [0.0f32; 16];
        let mut gr = GraphRuntimeState::new();

        let output = execute_creature_mesh(
            &genome,
            &ss,
            &mut energy,
            0.0,
            &mut smem,
            &prev_smem,
            &mut gr,
            &config,
        );

        assert_eq!(output.work_counters.mesh_hops, 1);
        assert_eq!(output.work_counters.vm_steps, 0);
        assert_eq!(
            output.work_counters.graph_relax_iters,
            config.graph_convergence_stable_passes + 1
        );
        assert_eq!(output.work_counters.plasticity_updates, 0);
    }

    /// A single compute node with one Hebbian-plastic edge applies exactly
    /// one weight update (one edge) after the relaxation loop converges.
    #[test]
    fn hebbian_plastic_node_reports_one_plasticity_update() {
        let id0 = NodeId::new(0);
        let genome = CreatureGenome {
            entry_node_id: id0,
            nodes: vec![NodeGenome {
                node_id: id0,
                input_refs: vec![],
                backend_def: BackendDef::Graph(CgpGraphBackendDef {
                    compute_nodes: vec![ComputeNode {
                        kind: ComputeNodeKind::Constant(1.0),
                        inputs: vec![GraphEdge {
                            source: GraphSource::SharedMemory {
                                slot: 0,
                                previous: false,
                            },
                            weight: 1.0,
                        }],
                        plasticity: Some(PlasticityConfig {
                            rule: HebbianRule::Classic,
                            learning_rate: 0.5,
                            weight_clamp: 1.0,
                            lamarckian: false,
                            modulation: None,
                        }),
                    }],
                    output_sinks: vec![],
                    action_bank: vec![ActionSlot {
                        behavior: ActionSlotBehavior::Emit(WorldActionKind::Eat),
                        gate_inputs: vec![GraphEdge {
                            source: GraphSource::ComputeNode(0),
                            weight: 1.0,
                        }],
                        param_inputs: vec![],
                    }],
                    execute_gate: ExecuteGate {
                        inputs: vec![GraphEdge {
                            source: GraphSource::ComputeNode(0),
                            weight: 1.0,
                        }],
                    },
                }),
                targets: vec![],
            }],
        };

        let ss = empty_ss();
        let config = RuntimeConfig {
            graph_node_base_cost: 0.01,
            plasticity_update_cost: 0.01,
            ..RuntimeConfig::default()
        };
        let mut energy = 100.0f32;
        let mut smem = [0.0f32; 16];
        let prev_smem = [0.0f32; 16];
        let mut gr = GraphRuntimeState::new();

        let output = execute_creature_mesh(
            &genome,
            &ss,
            &mut energy,
            0.0,
            &mut smem,
            &prev_smem,
            &mut gr,
            &config,
        );

        assert_eq!(output.work_counters.plasticity_updates, 1);
        assert_eq!(
            output.work_counters.graph_relax_iters,
            config.graph_convergence_stable_passes + 1
        );
    }
}
