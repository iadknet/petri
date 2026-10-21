use crate::config::RuntimeConfig;
use crate::contracts::InputReference;
use crate::creature::genome::cgp::{CgpGraphBackendDef, EdgeSurface, GraphEdge};
use crate::creature::state::GraphRuntimeState;
use crate::runtime::cgp::effects::apply_cgp_graph_effects;
use crate::runtime::cgp::eval::{evaluate_compute_kind, sanitize_output};
use crate::runtime::cgp::sources::{collect_cgp_weighted_inputs, resolve_source};
use crate::runtime::cgp::tracer::{GraphTracer, NoopTracer};
use crate::runtime::inputs::ResolveCtx;
use crate::runtime::plasticity::Learner;
use crate::runtime::routing::RouteGateMap;
use crate::runtime::types::{MeshSideOutputs, NodeResult, OUTPUT_SLOT_COUNT};
use crate::sensors::perception::SensorSnapshot;
use crate::simulation::energy_accounting::{applied_debit, observe_energy_change, DeathCause};

/// Collect a learning graph's weighted inputs of compute node `current_idx`
/// into `buf`: each edge's ordered read times its applied weight, the read
/// recorded as the pre sample of a plastic edge (T25.F06).
#[inline]
#[allow(clippy::too_many_arguments)]
fn collect_learning_inputs(
    edges: &[GraphEdge],
    current_idx: usize,
    node_count: usize,
    prev_outputs: &[f32],
    curr_outputs: &[f32],
    input_refs: &[InputReference],
    resolve_ctx: &ResolveCtx<'_>,
    shared_memory: &[f32; 16],
    prev_shared_memory: &[f32; 16],
    learner: &mut Learner<'_>,
    buf: &mut Vec<f32>,
) {
    buf.clear();
    learner.begin_row(EdgeSurface::ComputeInput(current_idx));
    buf.extend(edges.iter().map(|edge| {
        let value = resolve_source(
            &edge.source(),
            current_idx,
            node_count,
            prev_outputs,
            curr_outputs,
            input_refs,
            resolve_ctx,
            shared_memory,
            prev_shared_memory,
        );
        value * learner.weigh(value, edge.own_weight())
    }));
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
    let node_count = def.compute_nodes().len();

    // Neither compute nodes nor a wired effect surface: no work, no charge.
    if !def.enters_visit() {
        return NodeResult::halted(*upstream_slots, RouteGateMap::default());
    }

    let mut prev_outputs = std::mem::take(&mut graph_runtime.scratch_prev);
    let mut curr_outputs = std::mem::take(&mut graph_runtime.scratch_curr);
    // Live state (T19.F02): a visit starts from the last committed operator
    // state and outputs, whether that commit was this tick or earlier.
    let mut candidate_state = std::mem::take(&mut graph_runtime.scratch_backup);
    candidate_state.clear();
    if let Some(base) = graph_runtime.node_state.get(node_idx) {
        candidate_state.extend_from_slice(base);
    }
    candidate_state.resize(node_count, 0.0);
    prev_outputs.clear();
    if let Some(base) = graph_runtime.node_outputs.get(node_idx) {
        prev_outputs.extend_from_slice(base);
    }
    prev_outputs.resize(node_count, 0.0);
    curr_outputs.clear();
    curr_outputs.resize(node_count, 0.0);

    let mut w_inputs_buf = std::mem::take(&mut graph_runtime.scratch_w_inputs);

    // A learning graph (T25.F06): the backend counts as visited from its
    // entry, before the pass-cost check, and every plastic edge's samples
    // are recorded as the evaluation reads them.
    let mut learner = graph_runtime.learned.learner(node_idx);
    if let Some(learner) = &mut learner {
        learner.enter();
    }

    // The wire counter records entered graph visits, even when unaffordable.
    side_outputs.work_counters.graph_relax_iters += 1;
    // A wired zero-compute visit pays one node-equivalent: what the same graph
    // pays with a single dummy compute node, so removing the dummy is free of
    // work-accounting consequence.
    let pass_cost = config.graph_node_base_cost * node_count.max(1) as f32;
    let before = *energy;
    *energy -= pass_cost;
    side_outputs.energy_observation.graph_compute += applied_debit(before, *energy);
    observe_energy_change(
        &mut side_outputs.energy_observation.pending_cause,
        f64::from(before),
        f64::from(*energy),
        DeathCause::GraphCompute,
    );
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

    // The tick's decision state (T19.F05): nothing commits mid-dispatch, so
    // the evaluation and the effects read one context. Its borrows of the
    // side outputs are disjoint from the fields the visit writes.
    let resolve_ctx = ResolveCtx {
        sensors,
        upstream_slots,
        energy: *energy,
        energy_consumed,
        action_queue: &side_outputs.action_queue,
        votes: &side_outputs.votes,
        previous_pass_votes: &side_outputs.previous_pass_votes,
        commit_counts: &side_outputs.commit_counts,
        mesh_hops: side_outputs.work_counters.mesh_hops,
    };

    for current_idx in 0..node_count {
        let node = &def.compute_nodes()[current_idx];

        match &mut learner {
            Some(learner) => collect_learning_inputs(
                &node.inputs,
                current_idx,
                node_count,
                &prev_outputs,
                &curr_outputs,
                input_refs,
                &resolve_ctx,
                shared_memory,
                prev_shared_memory,
                learner,
                &mut w_inputs_buf,
            ),
            None => collect_cgp_weighted_inputs(
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
            ),
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
        // The node's committed output is the post sample of its plastic
        // input edges.
        if let Some(learner) = &mut learner {
            learner.end_row(curr_outputs[current_idx]);
        }

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

    let (result, contribution) = apply_cgp_graph_effects(
        tracer,
        def,
        &graph_runtime.sink_weights[node_idx],
        &curr_outputs,
        input_refs,
        &resolve_ctx,
        upstream_slots,
        learner.as_mut(),
        &mut w_inputs_buf,
        &mut side_outputs.work_counters.shared_memory_writes_changed,
        &mut side_outputs.action_params,
        shared_memory,
        prev_shared_memory,
    );
    tracer.on_effects();

    // Learning after the effects, from the samples the visit recorded and
    // the weights in force at its start (T25.F06 invariant 6).
    if let Some(learner) = learner {
        let counts = learner.commit();
        side_outputs.work_counters.plasticity_updates += counts.updates;
        side_outputs.work_counters.plasticity_changes += counts.changes;
    }
    // The visit reached its effects, so it commits (T19.F03): an exhausted
    // visit returns before them and stages nothing.
    side_outputs.stage_vote_contribution(&contribution);

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
        CgpGraphBackendDef, ComputeNode, ComputeNodeKind, GraphEdge, GraphSource,
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
            world_reads: Default::default(),
            local: StaticInputs {
                food_here: 0.0,
                neighbor_food: [0.0; 8],
                neighbor_barrier: [0.0; 8],
                neighbor_occupied: [0.0; 8],
                energy_scale: 200.0,
                age_ticks: 0.0,
                own_colour: [0.0; 3],
                previous_outcome: [0.0; 4],
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
                backend_def: BackendDef::Graph(
                    CgpGraphBackendDef::new_with_fixed_outputs()
                        .with_compute_nodes(vec![ComputeNode::new(
                            ComputeNodeKind::Constant(1.0),
                            vec![],
                        )])
                        .with_output_sinks(vec![]),
                ),
                targets: vec![],
            }],
        };

        let ss = empty_ss();
        let config = RuntimeConfig::default();
        let mut energy = 100.0f32;
        let mut smem = [0.0f32; 16];
        let prev_smem = [0.0f32; 16];
        let mut gr = GraphRuntimeState::for_genome(&genome, Vec::new());

        let output = execute_creature_mesh(
            &genome,
            &ss,
            &mut energy,
            &mut smem,
            &prev_smem,
            &mut gr,
            &config,
            crate::contracts::Direction::N,
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
                backend_def: BackendDef::Graph(
                    CgpGraphBackendDef::new_with_fixed_outputs()
                        .with_compute_nodes(vec![
                            crate::creature::genome::cgp::test_support::plastic_node(
                                ComputeNodeKind::Constant(1.0),
                                vec![GraphEdge::new(
                                    GraphSource::SharedMemory {
                                        slot: 0,
                                        previous: false,
                                    },
                                    1.0,
                                )],
                                Some(PlasticityConfig {
                                    rule: HebbianRule::Classic,
                                    learning_rate: 0.5,
                                    weight_clamp: 1.0,
                                    lamarckian: false,
                                    modulation: None,
                                    retention: 1.0,
                                }),
                            ),
                        ])
                        .with_output_sinks(vec![crate::creature::genome::cgp::OutputSink::new(
                            crate::creature::genome::cgp::OutputSinkKind::ActionVote(
                                crate::creature::genome::vote::VoteSink::Terminate,
                            ),
                            vec![GraphEdge::new(GraphSource::ComputeNode(0), 1.0)],
                        )]),
                ),
                targets: vec![],
            }],
        };

        let ss = empty_ss();
        let config = RuntimeConfig {
            graph_node_base_cost: 0.01,
            ..RuntimeConfig::default()
        };
        let mut energy = 100.0f32;
        let mut smem = [0.0f32; 16];
        let prev_smem = [0.0f32; 16];
        let mut gr = GraphRuntimeState::for_genome(&genome, Vec::new());

        let output = execute_creature_mesh(
            &genome,
            &ss,
            &mut energy,
            &mut smem,
            &prev_smem,
            &mut gr,
            &config,
            crate::contracts::Direction::N,
        );

        assert_eq!(output.work_counters.plasticity_updates, 1);
        assert_eq!(output.work_counters.graph_relax_iters, 1);
        // Learning is free (T25.F02): the visit pays its one pass cost only.
        assert_eq!(energy, 100.0f32 - 0.01);
    }
}

#[cfg(test)]
mod clock_tests {
    use super::*;
    use crate::creature::genome::cgp::{
        ComputeNode, ComputeNodeKind, GraphEdge, GraphSource, OutputSink, OutputSinkKind,
    };
    use crate::creature::genome::{HebbianRule, PlasticityConfig};
    use proptest::prelude::*;

    fn graph(kind: ComputeNodeKind, disconnected: usize) -> CgpGraphBackendDef {
        let mut nodes = vec![ComputeNode::new(
            kind,
            vec![GraphEdge::new(
                GraphSource::SharedMemory {
                    slot: 0,
                    previous: false,
                },
                1.0,
            )],
        )];
        nodes.extend(
            (0..disconnected).map(|_| ComputeNode::new(ComputeNodeKind::Oscillator(0.25), vec![])),
        );
        CgpGraphBackendDef::new_with_fixed_outputs()
            .with_compute_nodes(nodes)
            .with_output_sinks(vec![OutputSink::new(
                OutputSinkKind::WriteSlot(1),
                vec![GraphEdge::new(GraphSource::ComputeNode(0), 1.0)],
            )])
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

    /// Internal time is counted in visits (T19.F02): a second visit in the
    /// same tick starts from the first visit's commit, and ticks without a
    /// visit hold the last commit without catching up.
    #[test]
    fn repeated_visits_advance_from_the_last_commit_and_skipped_ticks_hold() {
        let def = graph(ComputeNodeKind::DecayIntegrator(0.5), 0);
        let mut state = crate::creature::genome::cgp::test_support::runtime_for(&def);
        let config = RuntimeConfig {
            graph_node_base_cost: 0.25,
            ..RuntimeConfig::default()
        };
        let mut energy = 100.0;
        state.begin_tick(0);
        assert_eq!(visit(&def, &mut state, 1.0, &mut energy, &config).0, 0.5);
        assert_eq!(visit(&def, &mut state, 2.0, &mut energy, &config).0, 1.25);
        assert_eq!(energy, 99.5);
        state.begin_tick(0);
        state.begin_tick(0); // no visit: sample and hold, no catch-up
        assert_eq!(state.node_outputs[0], [1.25]);
        assert_eq!(energy, 99.5);
        assert_eq!(visit(&def, &mut state, 1.0, &mut energy, &config).0, 1.125);
    }

    #[test]
    fn failed_revisits_preserve_temporal_commit_and_emit_no_effects() {
        let mut def = graph(ComputeNodeKind::DecayIntegrator(0.5), 0);
        crate::creature::genome::cgp::test_support::set_surface_plasticity(
            &mut def,
            crate::creature::genome::cgp::EdgeSurface::ComputeInput(0),
            Some(PlasticityConfig {
                rule: HebbianRule::Classic,
                learning_rate: 0.1,
                weight_clamp: 2.0,
                lamarckian: false,
                modulation: None,
                retention: 1.0,
            }),
        );
        let config = RuntimeConfig {
            graph_node_base_cost: 1.0,
            ..RuntimeConfig::default()
        };
        let mut state = crate::creature::genome::cgp::test_support::runtime_for(&def);
        state.begin_tick(0);
        visit(&def, &mut state, 1.0, &mut 100.0, &config);
        let committed_state = state.node_state.clone();
        let committed_outputs = state.node_outputs.clone();
        // The pass cost is the only exhaustion point: learning is free
        // (T25.F02), so a plastic node that affords its pass completes it.
        for initial_energy in [0.5, 1.0] {
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
        crate::creature::genome::cgp::test_support::set_surface_plasticity(
            &mut def,
            crate::creature::genome::cgp::EdgeSurface::ComputeInput(0),
            Some(PlasticityConfig {
                rule: HebbianRule::Classic,
                learning_rate: 0.1,
                weight_clamp: 2.0,
                lamarckian: false,
                modulation: None,
                retention: 1.0,
            }),
        );
        let config = RuntimeConfig {
            graph_node_base_cost: 1.0,
            ..RuntimeConfig::default()
        };
        let mut state = crate::creature::genome::cgp::test_support::runtime_for(&def);
        visit(&def, &mut state, 1.0, &mut 100.0, &config);
        // The pass cost is the only exhaustion point (T25.F02: learning is
        // free).
        let mut energy = 1.0;
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

    #[test]
    fn ordered_combinational_path_and_traced_temporal_state_match() {
        let mut def = graph(ComputeNodeKind::Add, 0);
        def.append_compute_node(ComputeNode::new(
            ComputeNodeKind::DecayIntegrator(0.5),
            vec![GraphEdge::new(GraphSource::ComputeNode(0), 2.0)],
        ));
        def.edge_mut(crate::creature::genome::cgp::EdgeOccurrence::new(
            crate::creature::genome::cgp::EdgeSurface::SinkInput(0),
            0,
        ))
        .set_source(GraphSource::ComputeNode(1));
        let mut ordinary = crate::creature::genome::cgp::test_support::runtime_for(&def);
        let mut traced = ordinary.clone();
        for expected_delta in [1.0, 0.5, 0.25] {
            ordinary.begin_tick(0);
            traced.begin_tick(0);
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
        /// One visit steps a stateful operator exactly once from its committed
        /// state whatever disconnected growth the module carries, and a
        /// second visit in the same tick steps it again from the first
        /// visit's commit (T19.F02).
        #[test]
        fn temporal_step_advances_once_per_visit_and_ignores_disconnected_nodes(
            extra in 0..10usize,
            input in -5.0f32..5.0, old in 0.1f32..0.8, family in 0..4u8,
        ) {
            let kind = match family {
                0 => ComputeNodeKind::DecayIntegrator(0.25),
                1 => ComputeNodeKind::Momentum(0.25),
                2 => ComputeNodeKind::Oscillator(0.125),
                _ => ComputeNodeKind::AdaptiveGain,
            };
            let step = |old: f32| match family {
                0 => 0.75 * old + 0.25 * input,
                1 => 0.25 * old + 0.75 * input,
                2 => (old + 0.125).fract(),
                _ => (old + 0.01 * input).clamp(0.1, 2.0),
            };
            let mut state = crate::creature::genome::cgp::test_support::runtime_for(&graph(kind, extra)); state.node_state = vec![vec![old]];
            state.begin_tick(0);
            let config = RuntimeConfig::default();
            let def = graph(kind, extra);
            let (_, side) = visit(&def, &mut state, input, &mut 100.0, &config);
            prop_assert!((state.node_state[0][0] - step(old)).abs() < 1e-6);
            prop_assert_eq!(side.work_counters.graph_relax_iters, 1);
            visit(&def, &mut state, input, &mut 100.0, &config);
            prop_assert!((state.node_state[0][0] - step(step(old))).abs() < 1e-6);
        }
    }
}
