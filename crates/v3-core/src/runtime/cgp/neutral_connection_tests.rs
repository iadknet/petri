//! T20.F02: ordinary zero vote edges, qualified through the native evaluator.
//! Equality is exact numeric f32 equality; a tolerance would hide changed votes.
use super::effects::apply_cgp_graph_effects;
use super::execute::execute_graph_node;
use super::sources::resolve_source_post_convergence;
use crate::config::{OrdinaryFoodTypeId, RuntimeConfig};
use crate::contracts::{
    Direction, DynamicIntrospectionKey, InputReference, NodeId, RouteTarget,
    StaticIntrospectionKey, WorldAction, WorldInputKey,
};
use crate::creature::genome::cgp::{
    CgpGraphBackendDef, ComputeNode, ComputeNodeKind, GraphEdge, GraphSource, OutputSinkKind,
};
use crate::creature::genome::vote::{ActionParamField, VoteSink, VOTE_SINK_COUNT};
use crate::creature::genome::{
    BackendDef, CreatureGenome, HebbianRule, NodeGenome, OutcomeChannel, PlasticityConfig,
    RewardModulationConfig,
};
use crate::creature::state::GraphRuntimeState;
use crate::runtime::inputs::ResolveCtx;
use crate::runtime::mesh::execute_creature_mesh;
use crate::runtime::types::{MeshSideOutputs, NodeResult, OUTPUT_SLOT_COUNT};
use crate::sensors::perception::{PerceptionSnapshot, SensorSnapshot};
use crate::sensors::static_inputs::StaticInputs;
use crate::sensors::typed_food::TypedFoodLocalSnapshot;
use crate::simulation::tick::advance_shared_memory;
use proptest::prelude::*;

mod boundaries;
mod lifecycle;
mod recruitment;

const BASE_COST: f32 = 0.25;

fn ramp<const N: usize>(offset: f32) -> [f32; N] {
    std::array::from_fn(|i| offset + (i + 1) as f32 / 128.0)
}

fn sensors() -> SensorSnapshot {
    SensorSnapshot {
        local: StaticInputs {
            food_here: 0.25,
            neighbor_food: ramp(0.0),
            neighbor_barrier: ramp(0.125),
            neighbor_occupied: ramp(0.25),
            max_energy: 200.0,
            age_ticks: 10.0,
            previous_outcome: ramp(-0.5),
        },
        typed_local_food: TypedFoodLocalSnapshot {
            food_here_by_type: vec![0.25, 0.75],
            neighbor_food_by_type: vec![ramp(0.0), ramp(0.5)],
        },
        perception: PerceptionSnapshot {
            area_food: ramp(0.0),
            typed_area_food: vec![ramp(0.0), ramp(0.5)],
            area_barrier: ramp(0.125),
            area_occupancy: ramp(0.25),
            nearby_core: ramp(0.0),
            nearby_vitals: ramp(0.125),
            nearby_identity: ramp(0.25),
        },
    }
}

fn side() -> MeshSideOutputs {
    let mut side = MeshSideOutputs::new(8);
    for _ in 0..5 {
        side.action_queue.push(WorldAction::StealEnergy {
            direction: Direction::NW,
            amount: 0.75,
        });
    }
    side.votes = ramp(0.0);
    side.previous_pass_votes = ramp(0.5);
    side.commit_counts = [1, 2, 3, 4];
    side.action_params = [0.25, 0.5, 0.75];
    side.work_counters.mesh_hops = 6;
    side
}

fn edge(source: GraphSource, weight: f32) -> GraphEdge {
    GraphEdge { source, weight }
}

fn memory(slot: u8, previous: bool) -> GraphSource {
    GraphSource::SharedMemory { slot, previous }
}

fn leaf(ref_idx: u16, sub_idx: u16) -> GraphSource {
    GraphSource::InputLeaf { ref_idx, sub_idx }
}

fn wire(def: &mut CgpGraphBackendDef, kind: OutputSinkKind, edge: GraphEdge) {
    def.output_sinks
        .iter_mut()
        .find(|sink| sink.kind == kind)
        .expect("destination must already exist")
        .inputs
        .push(edge);
}

fn graph_node(def: CgpGraphBackendDef, refs: Vec<InputReference>) -> NodeGenome {
    NodeGenome {
        node_id: NodeId::new(0),
        input_refs: refs,
        backend_def: BackendDef::Graph(def),
        targets: vec![],
    }
}

fn graph(node: &NodeGenome) -> &CgpGraphBackendDef {
    let BackendDef::Graph(def) = &node.backend_def else {
        panic!("Graph fixture")
    };
    def
}

fn graph_mut(node: &mut NodeGenome) -> &mut CgpGraphBackendDef {
    let BackendDef::Graph(def) = &mut node.backend_def else {
        panic!("Graph fixture")
    };
    def
}

/// Explicit expected channels, independent of the resolver being qualified.
fn reference_cases() -> Vec<(InputReference, Vec<f32>)> {
    use InputReference as R;
    use WorldInputKey as W;
    let s = sensors();
    let mut cases = Vec::new();
    for t in 0..2 {
        let id = OrdinaryFoodTypeId::new(t);
        cases.extend([
            (
                R::World(W::food_here(id)),
                vec![s.typed_local_food.food_here_by_type[t as usize]],
            ),
            (
                R::World(W::neighbor_food_ring(id)),
                s.typed_local_food.neighbor_food_by_type[t as usize].to_vec(),
            ),
            (
                R::World(W::area_food_summary(id)),
                s.perception.typed_area_food[t as usize].to_vec(),
            ),
        ]);
    }
    cases.extend([
        (
            R::World(W::NeighborBarrierRing),
            s.local.neighbor_barrier.to_vec(),
        ),
        (
            R::World(W::NeighborOccupiedRing),
            s.local.neighbor_occupied.to_vec(),
        ),
        (
            R::World(W::AreaBarrierSummary),
            s.perception.area_barrier.to_vec(),
        ),
        (
            R::World(W::AreaOccupancySummary),
            s.perception.area_occupancy.to_vec(),
        ),
        (
            R::World(W::NearbyCreatureCore),
            s.perception.nearby_core.to_vec(),
        ),
        (
            R::World(W::NearbyCreatureVitals),
            s.perception.nearby_vitals.to_vec(),
        ),
        (
            R::World(W::NearbyCreatureIdentity),
            s.perception.nearby_identity.to_vec(),
        ),
        (
            R::StaticIntrospection(StaticIntrospectionKey::AgeTicks),
            vec![10.0],
        ),
        (
            R::DynamicIntrospection(DynamicIntrospectionKey::EnergyCurrent),
            vec![0.5],
        ),
        (
            R::DynamicIntrospection(DynamicIntrospectionKey::EnergyConsumedThisTick),
            vec![0.125],
        ),
        (
            R::DynamicIntrospection(DynamicIntrospectionKey::HopsThisTick),
            vec![6.0],
        ),
        (R::ActionQueue, [4.0, 7.0, 0.75].repeat(5)),
        (R::ActionVotes, side().votes.to_vec()),
        (R::PreviousPassVotes, side().previous_pass_votes.to_vec()),
        (R::CommitCounts, vec![1.0, 2.0, 3.0, 4.0]),
        (R::PreviousOutcome, s.local.previous_outcome.to_vec()),
    ]);
    cases.extend((0..OUTPUT_SLOT_COUNT).map(|slot| {
        (
            R::UpstreamSlot(slot),
            vec![ramp::<OUTPUT_SLOT_COUNT>(0.25)[slot]],
        )
    }));
    cases
}

struct Effects {
    result: NodeResult,
    side: MeshSideOutputs,
    memory: [f32; 16],
}

fn effects(def: &CgpGraphBackendDef, refs: &[InputReference], source_memory: f32) -> Effects {
    let sensors = sensors();
    let upstream = ramp(0.25);
    let input_side = side();
    let ctx = ResolveCtx {
        sensors: &sensors,
        upstream_slots: &upstream,
        energy: 100.0,
        energy_consumed: 25.0,
        action_queue: &input_side.action_queue,
        votes: &input_side.votes,
        previous_pass_votes: &input_side.previous_pass_votes,
        commit_counts: &input_side.commit_counts,
        mesh_hops: 6,
    };
    let mut memory = ramp(0.0);
    memory[15] = source_memory;
    let mut side = side();
    let (result, _) = apply_cgp_graph_effects(
        def,
        &[0.5; 3],
        refs,
        &ctx,
        &upstream,
        &mut side,
        &mut memory,
        &ramp(0.5),
    );
    Effects {
        result,
        side,
        memory,
    }
}

#[test]
fn every_vote_identity_accepts_every_excited_family_channel_and_independent_refinement() {
    let mut sources: Vec<_> = reference_cases()
        .into_iter()
        .flat_map(|(reference, values)| {
            values
                .into_iter()
                .enumerate()
                .map(move |(channel, expected)| {
                    (vec![reference.clone()], leaf(0, channel as u16), expected)
                })
        })
        .collect();
    for previous in [false, true] {
        for slot in 0..16 {
            let expected = ramp::<16>(if previous { 0.5 } else { 0.0 })[slot as usize];
            sources.push((vec![], memory(slot, previous), expected));
        }
    }
    assert_eq!(
        sources.len(),
        235,
        "203 declared channels and 32 memory slots"
    );
    let votes: Vec<_> = VoteSink::all().collect();
    assert_eq!(votes.len(), 27);
    for vote in votes {
        for wired in [false, true] {
            for (refs, source, expected) in &sources {
                assert_ne!(*expected, 0.0, "authored channels are excited");
                let mut base = CgpGraphBackendDef::new_with_fixed_outputs();
                // Rotate the catalog: eligibility comes from identity, never position.
                base.output_sinks.rotate_left(vote.index());
                if wired {
                    wire(
                        &mut base,
                        OutputSinkKind::ActionVote(vote),
                        edge(memory(0, true), 0.5),
                    );
                }
                let before = effects(&base, refs, ramp::<16>(0.0)[15]);
                let mut silent = base.clone();
                wire(
                    &mut silent,
                    OutputSinkKind::ActionVote(vote),
                    edge(*source, 0.0),
                );
                let after = effects(&silent, refs, ramp::<16>(0.0)[15]);
                assert_eq!(
                    before.side.dispatch_effects(),
                    after.side.dispatch_effects(),
                    "{vote:?}: {refs:?} {source:?}"
                );
                assert_eq!(before.result, after.result);
                assert_eq!(before.memory, after.memory);
                assert_eq!(before.side.work_counters, after.side.work_counters);
                let sink = silent
                    .output_sinks
                    .iter_mut()
                    .find(|s| s.kind == OutputSinkKind::ActionVote(vote))
                    .unwrap();
                assert_eq!(
                    &sink.inputs[..sink.inputs.len() - 1],
                    base.sink(sink.kind).unwrap().inputs
                );
                sink.inputs.last_mut().unwrap().weight = 0.5;
                let refined = effects(&silent, refs, ramp::<16>(0.0)[15]);
                let mut expected_votes = before.side.dispatch_effects().0;
                expected_votes[vote.index()] += expected * 0.5;
                assert_eq!(
                    refined.side.dispatch_effects().0,
                    expected_votes,
                    "{vote:?}: {refs:?} {source:?}"
                );
            }
        }
    }
}

proptest! {
    #[test]
    fn finite_zero_append_preserves_exact_vote_sum(
        source in any::<f32>().prop_filter("finite source", |v| v.is_finite()),
        incumbents in prop::collection::vec(-1.0e8f32..1.0e8, 0..20),
        index in 0usize..VOTE_SINK_COUNT,
        negative_zero in any::<bool>(),
    ) {
        let kind = OutputSinkKind::ActionVote(VoteSink::from_index(index).unwrap());
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
        for weight in incumbents {
            wire(&mut def, kind, edge(memory(0, true), weight));
        }
        let before = effects(&def, &[], source).side.dispatch_effects();
        wire(&mut def, kind, edge(memory(15, false), if negative_zero { -0.0 } else { 0.0 }));
        prop_assert_eq!(effects(&def, &[], source).side.dispatch_effects(), before);
    }
}

fn active_graph() -> CgpGraphBackendDef {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    for modulation in [
        None,
        Some(RewardModulationConfig {
            reward_source: OutcomeChannel::EnergyDelta,
            trace_decay: 0.5,
        }),
    ] {
        def.compute_nodes.push(ComputeNode {
            kind: ComputeNodeKind::DecayIntegrator(0.5),
            inputs: vec![edge(memory(0, false), 0.5)],
            plasticity: Some(PlasticityConfig {
                rule: HebbianRule::Classic,
                learning_rate: 0.125,
                weight_clamp: 2.0,
                lamarckian: false,
                modulation,
            }),
        });
    }
    for (kind, source, weight) in [
        (
            OutputSinkKind::CustomOutput(2),
            GraphSource::ComputeNode(0),
            1.0,
        ),
        (
            OutputSinkKind::WriteSlot(3),
            GraphSource::ComputeNode(1),
            1.0,
        ),
        (OutputSinkKind::RouterGate(0), memory(0, false), 1.0),
        (
            OutputSinkKind::ActionParam(ActionParamField::StealEnergyAmount),
            memory(0, false),
            1.0,
        ),
        (
            OutputSinkKind::ActionVote(VoteSink::StealEnergy(7)),
            memory(0, false),
            8.0,
        ),
    ] {
        wire(&mut def, kind, edge(source, weight));
    }
    def
}

#[test]
fn active_native_visits_and_ticks_preserve_actions_temporal_plasticity_and_costs() {
    let mut original = graph_node(
        active_graph(),
        vec![InputReference::World(WorldInputKey::NearbyCreatureCore)],
    );
    original.targets.push(RouteTarget {
        target_id: NodeId::new(0),
        slot: 0,
        gate_bias: 1.0,
    });
    let mut silent = original.clone();
    for vote in VoteSink::all() {
        use crate::config::NeutralInputRecruitment::WholeFamily;
        use crate::mutation::graph::recruitment::{recruit_source, SourceFamily};
        recruit_source(
            &mut silent,
            &SourceFamily::Input(InputReference::World(WorldInputKey::NearbyCreatureCore)),
            WholeFamily,
            0,
            vote,
        )
        .unwrap();
        recruit_source(
            &mut silent,
            &SourceFamily::Memory(true),
            WholeFamily,
            0,
            vote,
        )
        .unwrap();
    }
    let genomes = [original, silent].map(|node| CreatureGenome {
        entry_node_id: node.node_id,
        nodes: vec![node],
    });
    let config = RuntimeConfig {
        graph_node_base_cost: BASE_COST,
        plasticity_update_cost: 0.125,
        max_mesh_hops: 3,
        max_actions_per_turn: 2,
        ..RuntimeConfig::default()
    };
    let mut states = [GraphRuntimeState::new(), GraphRuntimeState::new()];
    for state in &mut states {
        state.node_state = vec![vec![0.25, 0.75]];
        state.node_outputs = vec![vec![0.125, 0.5]];
        state.plasticity_weights = vec![vec![
            vec![0.375].into_boxed_slice(),
            vec![0.625].into_boxed_slice(),
        ]];
        state.eligibility_traces = vec![vec![
            vec![0.0].into_boxed_slice(),
            vec![0.5].into_boxed_slice(),
        ]];
    }
    let mut energies = [1000.0; 2];
    let mut memories = [ramp(0.5); 2];
    let mut previous = [ramp(0.25); 2];
    for age in 1..=4 {
        let mut outputs = Vec::new();
        for i in 0..2 {
            advance_shared_memory(&mut memories[i], &mut previous[i], 0.125);
            states[i].begin_tick(&genomes[i].nodes, age);
            // Direct visit checks the staged vote before mesh pass bookkeeping.
            let mut side = side();
            let node = &genomes[i].nodes[0];
            let result = execute_graph_node(
                graph(node),
                &node.input_refs,
                &ramp(0.25),
                &mut energies[i],
                25.0,
                0,
                &mut states[i],
                &sensors(),
                &config,
                &mut side,
                &mut memories[i],
                &previous[i],
            );
            let mesh = execute_creature_mesh(
                &genomes[i],
                &sensors(),
                &mut energies[i],
                &mut memories[i],
                &previous[i],
                &mut states[i],
                &config,
            );
            outputs.push((result, side, mesh));
        }
        let (a, b) = (&outputs[0], &outputs[1]);
        assert_eq!(a.0, b.0);
        assert_eq!(a.1.dispatch_effects(), b.1.dispatch_effects());
        assert_eq!(a.1.work_counters, b.1.work_counters);
        assert_eq!(a.1.energy_observation, b.1.energy_observation);
        assert_eq!(a.2.actions, b.2.actions);
        assert!(a
            .2
            .actions
            .iter()
            .any(|action| matches!(action, WorldAction::StealEnergy { .. })));
        assert!(a.2.work_counters.graph_relax_iters > 1, "repeated visits");
        assert_eq!(a.2.work_counters, b.2.work_counters);
        assert_eq!(a.2.energy_observation, b.2.energy_observation);
        assert_eq!(a.2.commit_counts, b.2.commit_counts);
        assert_eq!(a.2.termination_reason, b.2.termination_reason);
        assert_eq!(a.2.priority_bid, b.2.priority_bid);
        assert_eq!(a.2.cost_report.graph_cost, b.2.cost_report.graph_cost);
        assert_eq!(a.2.cost_report.vm_cost, b.2.cost_report.vm_cost);
        assert_eq!(
            a.2.cost_report.mesh_ramp_cost,
            b.2.cost_report.mesh_ramp_cost
        );
        assert_eq!(energies[0], energies[1]);
        assert_eq!(memories[0], memories[1]);
        assert_eq!(previous[0], previous[1]);
        assert_eq!(states[0].node_state, states[1].node_state);
        assert_eq!(states[0].node_outputs, states[1].node_outputs);
        assert_eq!(states[0].plasticity_weights, states[1].plasticity_weights);
        assert_eq!(states[0].eligibility_traces, states[1].eligibility_traces);
        assert_eq!(states[0].dispatch_record, states[1].dispatch_record);
        assert_ne!(states[0].plasticity_weights[0][0][0], 0.375);
        assert_ne!(states[0].eligibility_traces[0][1][0], 0.5);
    }
}
