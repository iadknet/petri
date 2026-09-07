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

/// Callback trait for instrumenting the ordered graph evaluation.
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
    fn on_finish(&mut self, curr_outputs: &[f32], temporal_committed: bool);
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
    fn on_finish(&mut self, _: &[f32], _: bool) {}
    #[inline]
    fn on_effects(&mut self, _: CgpEffectsTrace) {}
}

#[allow(clippy::too_many_arguments)]
#[allow(
    clippy::too_many_lines,
    reason = "the ordered graph evaluation is a single hot path; extracting stages \
              would add borrow plumbing for the node and weight buffers"
)]
pub(crate) fn execute_graph_impl<T: GraphTracer>(
    tracer: &mut T,
    def: &CgpGraphBackendDef,
    input_refs: &[InputReference],
    upstream_slots: &[f32; OUTPUT_SLOT_COUNT],
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
        return NodeResult::halted(*upstream_slots, RouteGateMap::default());
    }

    let mut prev_outputs = std::mem::take(&mut graph_runtime.scratch_prev);
    let mut curr_outputs = std::mem::take(&mut graph_runtime.scratch_curr);
    let mut candidate_state = std::mem::take(&mut graph_runtime.scratch_backup);
    candidate_state.clear();
    if let Some(base) = graph_runtime.tick_start_state.get(node_idx) {
        candidate_state.extend_from_slice(base);
    }
    candidate_state.resize(node_count, 0.0);
    prev_outputs.clear();
    if let Some(base) = graph_runtime.tick_start_outputs.get(node_idx) {
        prev_outputs.extend_from_slice(base);
    }
    prev_outputs.resize(node_count, 0.0);
    curr_outputs.clear();
    curr_outputs.resize(node_count, 0.0);

    let use_plasticity = hebbian::has_any_hebbian(def);
    if use_plasticity {
        hebbian::ensure_hebbian_weights(def, node_idx, &mut graph_runtime.plasticity_weights);
    }

    let use_reward_modulated = traces::has_any_reward_modulated(def);
    if use_reward_modulated {
        traces::ensure_eligibility_traces(def, node_idx, &mut graph_runtime.eligibility_traces);
    }

    let mut w_inputs_buf = std::mem::take(&mut graph_runtime.scratch_w_inputs);

    // The wire counter records entered nonempty graph visits, even when unaffordable.
    side_outputs.work_counters.graph_relax_iters += 1;
    let pass_cost = config.graph_node_base_cost * node_count as f32;
    *energy -= pass_cost;
    tracer.on_pass_start(0, pass_cost, *energy);
    if *energy <= 0.0 {
        tracer.on_pass_end(0.0);
        tracer.on_finish(
            graph_runtime
                .node_outputs
                .get(node_idx)
                .map_or(&[], Vec::as_slice),
            false,
        );
        restore_scratch(
            graph_runtime,
            prev_outputs,
            curr_outputs,
            candidate_state,
            w_inputs_buf,
        );
        return NodeResult::exhausted();
    }

    let evaluation_energy = *energy;
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

        let mut node_state = candidate_state[current_idx];
        let state_before = node_state;

        curr_outputs[current_idx] = sanitize_output(evaluate_compute_kind(
            &node.kind,
            &w_inputs_buf,
            wsum,
            &mut node_state,
        ));

        candidate_state[current_idx] = node_state;

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

    tracer.on_pass_end(delta);

    if use_plasticity {
        let queue_snapshot = side_outputs.action_queue.clone();
        let post_ctx = ResolveCtx {
            sensors,
            upstream_slots,
            energy: *energy,
            energy_consumed,
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
            tracer.on_finish(
                graph_runtime
                    .node_outputs
                    .get(node_idx)
                    .map_or(&[], Vec::as_slice),
                false,
            );
            restore_scratch(
                graph_runtime,
                prev_outputs,
                curr_outputs,
                candidate_state,
                w_inputs_buf,
            );
            return NodeResult::exhausted();
        }
    }

    if graph_runtime.node_state.len() <= node_idx {
        graph_runtime.node_state.resize_with(node_idx + 1, Vec::new);
    }
    if graph_runtime.node_outputs.len() <= node_idx {
        graph_runtime
            .node_outputs
            .resize_with(node_idx + 1, Vec::new);
    }
    graph_runtime.node_state[node_idx].clone_from(&candidate_state);
    graph_runtime.node_outputs[node_idx].clone_from(&curr_outputs);
    tracer.on_finish(&curr_outputs, true);

    if use_reward_modulated {
        let queue_snapshot = side_outputs.action_queue.clone();
        let post_ctx = ResolveCtx {
            sensors,
            upstream_slots,
            energy: evaluation_energy,
            energy_consumed,
            action_queue: &queue_snapshot,
        };
        traces::update_eligibility_traces(
            def,
            node_idx,
            &mut graph_runtime.eligibility_traces,
            &graph_runtime.tick_start_eligibility_traces,
            &graph_runtime.plasticity_weights,
            &prev_outputs,
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
        candidate_state,
        w_inputs_buf,
    );

    result
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn execute_graph_node(
    def: &CgpGraphBackendDef,
    input_refs: &[InputReference],
    upstream_slots: &[f32; OUTPUT_SLOT_COUNT],
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

    pub(super) fn empty_ss() -> SensorSnapshot {
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

    /// A nonempty graph enters one evaluation regardless of convergence settings.
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
            &mut smem,
            &prev_smem,
            &mut gr,
            &config,
        );

        assert_eq!(output.work_counters.mesh_hops, 1);
        assert_eq!(output.work_counters.vm_steps, 0);
        assert_eq!(output.work_counters.graph_relax_iters, 1);
        assert_eq!(output.work_counters.plasticity_updates, 0);
    }

    /// A single compute node with one Hebbian-plastic edge applies exactly
    /// one weight update (one edge) after its ordered evaluation.
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
            &mut smem,
            &prev_smem,
            &mut gr,
            &config,
        );

        assert_eq!(output.work_counters.plasticity_updates, 1);
        assert_eq!(output.work_counters.graph_relax_iters, 1);
    }
}

#[cfg(test)]
mod clock_tests {
    use super::*;
    use crate::creature::genome::cgp::{
        ComputeNode, GraphEdge, GraphSource, OutputSink, OutputSinkKind,
    };
    use crate::creature::genome::{HebbianRule, PlasticityConfig};
    use proptest::prelude::*;

    fn graph(kind: ComputeNodeKind, disconnected: usize) -> CgpGraphBackendDef {
        let mut nodes = vec![ComputeNode {
            kind,
            inputs: vec![GraphEdge {
                source: GraphSource::SharedMemory {
                    slot: 0,
                    previous: false,
                },
                weight: 1.0,
            }],
            plasticity: None,
        }];
        nodes.extend((0..disconnected).map(|_| ComputeNode {
            kind: ComputeNodeKind::Oscillator(0.25),
            inputs: vec![],
            plasticity: None,
        }));
        CgpGraphBackendDef {
            compute_nodes: nodes,
            output_sinks: vec![OutputSink {
                kind: OutputSinkKind::WriteSlot(1),
                inputs: vec![GraphEdge {
                    source: GraphSource::ComputeNode(0),
                    weight: 1.0,
                }],
            }],
            action_bank: vec![],
            execute_gate: crate::creature::genome::cgp::ExecuteGate { inputs: vec![] },
        }
    }

    fn visit(
        def: &CgpGraphBackendDef,
        state: &mut GraphRuntimeState,
        input: f32,
        energy: &mut f32,
        config: &RuntimeConfig,
    ) -> (f32, MeshSideOutputs) {
        let mut memory = [0.0; 16];
        memory[0] = input;
        memory[1] = -99.0;
        let mut side = MeshSideOutputs::new(10);
        let _ = execute_graph_impl(
            &mut NoopTracer,
            def,
            &[],
            &[0.0; OUTPUT_SLOT_COUNT],
            energy,
            0.0,
            0,
            state,
            &super::work_counter_tests::empty_ss(),
            config,
            &mut side,
            &mut memory,
            &[0.0; 16],
        );
        (memory[1], side)
    }

    #[test]
    fn repeated_and_skipped_visits_use_frozen_base_and_last_success() {
        let def = graph(ComputeNodeKind::DecayIntegrator(0.5), 0);
        let mut state = GraphRuntimeState::new();
        let config = RuntimeConfig {
            graph_node_base_cost: 0.25,
            ..RuntimeConfig::default()
        };
        let mut energy = 100.0;
        state.begin_tick(&[]);
        assert_eq!(visit(&def, &mut state, 1.0, &mut energy, &config).0, 0.5);
        assert_eq!(visit(&def, &mut state, 2.0, &mut energy, &config).0, 1.0);
        assert_eq!(energy, 99.5);
        state.begin_tick(&[]);
        state.begin_tick(&[]); // no visit: sample and hold, no catch-up
        assert_eq!(state.node_outputs[0], [1.0]);
        assert_eq!(energy, 99.5);
        assert_eq!(visit(&def, &mut state, 1.0, &mut energy, &config).0, 1.0);
    }

    #[test]
    fn failed_revisits_preserve_temporal_commit_and_emit_no_effects() {
        let mut def = graph(ComputeNodeKind::DecayIntegrator(0.5), 0);
        def.compute_nodes[0].plasticity = Some(PlasticityConfig {
            rule: HebbianRule::Classic,
            learning_rate: 0.1,
            weight_clamp: 2.0,
            lamarckian: false,
            modulation: None,
        });
        let config = RuntimeConfig {
            graph_node_base_cost: 1.0,
            plasticity_update_cost: 1.0,
            ..RuntimeConfig::default()
        };
        let mut state = GraphRuntimeState::new();
        state.begin_tick(&[]);
        visit(&def, &mut state, 1.0, &mut 100.0, &config);
        let committed_state = state.node_state.clone();
        let committed_outputs = state.node_outputs.clone();
        for initial_energy in [0.5, 1.5] {
            let mut energy = initial_energy;
            let (memory, side) = visit(&def, &mut state, 8.0, &mut energy, &config);
            assert_eq!(memory, -99.0);
            assert!(side.action_queue.is_empty());
            assert_eq!(side.work_counters.graph_relax_iters, 1);
            assert_eq!(state.node_state, committed_state);
            assert_eq!(state.node_outputs, committed_outputs);
            assert!(energy <= 0.0);
        }
    }

    #[test]
    fn exhausted_trace_reports_committed_outputs_and_candidate_status() {
        let mut def = graph(ComputeNodeKind::DecayIntegrator(0.5), 0);
        def.compute_nodes[0].plasticity = Some(PlasticityConfig {
            rule: HebbianRule::Classic,
            learning_rate: 0.1,
            weight_clamp: 2.0,
            lamarckian: false,
            modulation: None,
        });
        let config = RuntimeConfig {
            graph_node_base_cost: 1.0,
            plasticity_update_cost: 1.0,
            ..RuntimeConfig::default()
        };
        let mut state = GraphRuntimeState::new();
        visit(&def, &mut state, 1.0, &mut 100.0, &config);
        for mut energy in [1.0, 2.0] {
            let mut memory = [0.0; 16];
            memory[0] = 8.0;
            let mut side = MeshSideOutputs::new(10);
            let (_, trace) = crate::runtime::cgp::traced::execute_graph_node_traced(
                &def,
                &[],
                &[0.0; OUTPUT_SLOT_COUNT],
                &mut energy,
                0.0,
                0,
                &mut state,
                &super::work_counter_tests::empty_ss(),
                &config,
                &mut side,
                &mut memory,
                &[0.0; 16],
            );
            assert!(!trace.temporal_committed);
            assert_eq!(trace.final_outputs, [0.5]);
            assert_eq!(trace.passes.len(), 1);
            assert_eq!(trace.passes[0].energy_cost, 1.0);
            assert!(trace.output_sinks.is_empty());
            assert_eq!(memory[1], 0.0);
            assert_eq!(energy, 0.0);
        }
    }

    #[test]
    fn ordered_combinational_path_and_traced_temporal_state_match() {
        let mut def = graph(ComputeNodeKind::Add, 0);
        def.compute_nodes.push(ComputeNode {
            kind: ComputeNodeKind::DecayIntegrator(0.5),
            inputs: vec![GraphEdge {
                source: GraphSource::ComputeNode(0),
                weight: 2.0,
            }],
            plasticity: None,
        });
        def.output_sinks[0].inputs[0].source = GraphSource::ComputeNode(1);
        let mut ordinary = GraphRuntimeState::new();
        let mut traced = ordinary.clone();
        for expected_delta in [1.0, 0.5, 0.25] {
            ordinary.begin_tick(&[]);
            traced.begin_tick(&[]);
            let mut energy = 100.0;
            let config = RuntimeConfig {
                graph_node_base_cost: 0.25,
                ..RuntimeConfig::default()
            };
            let (expected, side) = visit(&def, &mut ordinary, 1.0, &mut energy, &config);
            assert_eq!(energy, 99.5);
            let mut memory = [0.0; 16];
            memory[0] = 1.0;
            let mut traced_side = MeshSideOutputs::new(10);
            let (_, trace) = crate::runtime::cgp::traced::execute_graph_node_traced(
                &def,
                &[],
                &[0.0; OUTPUT_SLOT_COUNT],
                &mut 100.0,
                0.0,
                0,
                &mut traced,
                &super::work_counter_tests::empty_ss(),
                &config,
                &mut traced_side,
                &mut memory,
                &[0.0; 16],
            );
            assert_eq!(memory[1], expected);
            assert_eq!(traced.node_state, ordinary.node_state);
            assert_eq!(traced.node_outputs, ordinary.node_outputs);
            assert_eq!(trace.final_outputs, ordinary.node_outputs[0]);
            assert!(trace.temporal_committed);
            assert_eq!(trace.passes.len(), 1);
            assert_eq!(trace.passes[0].max_delta, expected_delta);
            assert_eq!(traced_side.work_counters, side.work_counters);
        }
        assert_eq!(ordinary.node_state[0][1], 1.75);
    }

    proptest! {
        #[test]
        fn temporal_step_ignores_pass_settings_and_disconnected_nodes(
            pass_cap in 0..50u32, stable in 0..20u32, extra in 0..10usize,
            input in -5.0f32..5.0, old in 0.1f32..0.8, family in 0..4u8,
        ) {
            let kind = match family {
                0 => ComputeNodeKind::DecayIntegrator(0.25),
                1 => ComputeNodeKind::Momentum(0.25),
                2 => ComputeNodeKind::Oscillator(0.125),
                _ => ComputeNodeKind::AdaptiveGain,
            };
            let expected_state = match family {
                0 => 0.75 * old + 0.25 * input,
                1 => 0.25 * old + 0.75 * input,
                2 => (old + 0.125).fract(),
                _ => (old + 0.01 * input).clamp(0.1, 2.0),
            };
            let mut state = GraphRuntimeState::new(); state.node_state = vec![vec![old]];
            state.begin_tick(&[]);
            let config = RuntimeConfig { max_graph_relax_iters: pass_cap, graph_convergence_stable_passes: stable, graph_convergence_epsilon: 99.0, ..RuntimeConfig::default() };
            let def = graph(kind, extra);
            let (_, side) = visit(&def, &mut state, input, &mut 100.0, &config);
            prop_assert!((state.node_state[0][0] - expected_state).abs() < 1e-6);
            prop_assert_eq!(side.work_counters.graph_relax_iters, 1);
            let outputs = state.node_outputs.clone();
            visit(&def, &mut state, input, &mut 100.0, &config);
            prop_assert_eq!(&state.node_outputs, &outputs);
        }
    }
}
