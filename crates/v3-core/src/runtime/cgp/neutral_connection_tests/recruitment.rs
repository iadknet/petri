//! F04 access through the actual preflight/append path and native evaluator.
use super::*;
use crate::config::{MutationConfig, NeutralInputRecruitment as Arm};
use crate::mutation::graph::operators::{alter_edge_weight_in_def, remove_edge};
use crate::mutation::graph::recruitment::{recruit_source, SourceFamily};
use crate::mutation::input_ref::{InputRefMutator, InputRefOperator};
use crate::mutation::reachability::TargetSelector;
use crate::mutation::topology::{TopologyMutator, TopologyOperator};
use rand::{rngs::SmallRng, SeedableRng};

fn examples() -> [SourceFamily; 5] {
    [
        SourceFamily::Input(InputReference::World(WorldInputKey::area_food_summary(
            OrdinaryFoodTypeId::new(0),
        ))),
        SourceFamily::Input(InputReference::World(WorldInputKey::food_here(
            OrdinaryFoodTypeId::new(1),
        ))),
        SourceFamily::Input(InputReference::World(WorldInputKey::neighbor_food_ring(
            OrdinaryFoodTypeId::new(0),
        ))),
        SourceFamily::Memory(false),
        SourceFamily::Memory(true),
    ]
}

#[test]
fn recruited_examples_are_exactly_neutral_then_independently_refinable_at_every_vote() {
    for arm in [Arm::SingleChannel, Arm::WholeFamily] {
        for family in examples() {
            for vote in VoteSink::all() {
                let mut node = graph_node(CgpGraphBackendDef::new_with_fixed_outputs(), vec![]);
                let before = effects(graph(&node), &node.input_refs, 0.5);
                recruit_source(&mut node, &family, arm, 0, vote).unwrap();
                let silent = effects(graph(&node), &node.input_refs, 0.5);
                // Exact numeric equality is the contract, including parameters,
                // carried memory and staged effect channels.
                assert_eq!(
                    silent.side.dispatch_effects(),
                    before.side.dispatch_effects()
                );
                assert_eq!(silent.result, before.result);
                assert_eq!(silent.memory, before.memory);
                let mut refined = node.clone();
                alter_edge_weight_in_def(graph_mut(&mut refined), &mut SmallRng::seed_from_u64(11))
                    .unwrap();
                let weights: Vec<_> = graph(&refined).edges().map(|edge| edge.weight).collect();
                assert_eq!(weights.iter().filter(|&&weight| weight != 0.0).count(), 1);
                assert!(graph(&node).edges().all(|edge| edge.weight == 0.0));
                let votes = effects(graph(&refined), &refined.input_refs, 0.5)
                    .side
                    .dispatch_effects()
                    .0;
                assert_ne!(votes[vote.index()], 0.0, "{family:?} {arm:?} {vote:?}");
                assert_eq!(votes.iter().filter(|&&vote| vote != 0.0).count(), 1);
            }
        }
    }
}

#[test]
fn recruitment_reuses_references_appends_duplicates_and_copies_then_prunes_without_regrowth() {
    let family = examples()[0].clone();
    let mut node = graph_node(CgpGraphBackendDef::new_with_fixed_outputs(), vec![]);
    for _ in 0..2 {
        recruit_source(&mut node, &family, Arm::SingleChannel, 0, VoteSink::Eat).unwrap();
    }
    assert_eq!(node.input_refs.len(), 1);
    assert_eq!(graph(&node).edges().count(), 2);
    node.node_id = NodeId::new(1);
    let mut entry = graph_node(CgpGraphBackendDef::new_with_fixed_outputs(), vec![]);
    entry.targets.push(RouteTarget {
        target_id: node.node_id,
        slot: 0,
        gate_bias: 1.0,
    });
    let mut genome = CreatureGenome {
        entry_node_id: entry.node_id,
        nodes: vec![entry, node],
    };
    TopologyMutator::apply(
        &mut genome,
        TopologyOperator::CopyNode,
        &mut TargetSelector::reachable_only(&[1], 1.0),
        &mut SmallRng::seed_from_u64(7),
        &MutationConfig::default(),
    )
    .unwrap();
    assert_eq!(genome.nodes[1].backend_def, genome.nodes[2].backend_def);
    let decoded: CreatureGenome =
        serde_json::from_str(&serde_json::to_string(&genome).unwrap()).unwrap();
    assert_eq!(genome, decoded);
    let mut rng = SmallRng::seed_from_u64(11);
    alter_edge_weight_in_def(graph_mut(&mut genome.nodes[2]), &mut rng).unwrap();
    assert!(graph(&genome.nodes[1])
        .edges()
        .all(|edge| edge.weight == 0.0));
    for _ in 0..2 {
        remove_edge(graph_mut(&mut genome.nodes[2]), &mut rng).unwrap();
    }
    InputRefMutator::apply_to_node(&mut genome, InputRefOperator::Prune, 2, &mut rng, 2).unwrap();
    let deleted = genome.nodes[2].clone();
    assert!(deleted.input_refs.is_empty());
    for _ in 0..3 {
        let run = effects(graph(&deleted), &deleted.input_refs, 0.5);
        assert_eq!(run.side.dispatch_effects().0, [0.0; VOTE_SINK_COUNT]);
        assert_eq!(graph(&deleted).edges().count(), 0);
    }
    // Fixture preparation: make the pruned donor the incumbent route. This
    // is not claimed to be a neutral evolutionary routing transition.
    genome.nodes[0].targets = vec![RouteTarget {
        target_id: deleted.node_id,
        slot: 0,
        gate_bias: 1.0,
    }];
    TopologyMutator::apply(
        &mut genome,
        TopologyOperator::CopyNode,
        &mut TargetSelector::reachable_only(&[2], 1.0),
        &mut rng,
        &MutationConfig::default(),
    )
    .unwrap();
    assert_eq!(
        genome.nodes.last().unwrap().backend_def,
        deleted.backend_def
    );
    assert!(genome.nodes.last().unwrap().input_refs.is_empty());
}

#[test]
fn neutral_recruitment_native_work_charges_activation_but_no_per_edge_price() {
    // Default rates and the exact F02 diagnostic rate are separate readings.
    for rate in [RuntimeConfig::default().graph_node_base_cost, BASE_COST] {
        let config = RuntimeConfig {
            graph_node_base_cost: rate,
            ..RuntimeConfig::default()
        };
        for active in [false, true] {
            for arm in [Arm::SingleChannel, Arm::WholeFamily] {
                for family in examples() {
                    let original = graph_node(
                        if active {
                            active_graph()
                        } else {
                            CgpGraphBackendDef::new_with_fixed_outputs()
                        },
                        vec![InputReference::World(WorldInputKey::NearbyCreatureCore)],
                    );
                    let mut recruited = original.clone();
                    recruit_source(&mut recruited, &family, arm, 0, VoteSink::Eat).unwrap();
                    let mut readings = Vec::new();
                    for node in [&original, &recruited] {
                        let mut energy = 10.0;
                        let mut state = GraphRuntimeState::new();
                        let mut side = side();
                        let mut memory = ramp(0.5);
                        let result = execute_graph_node(
                            graph(node),
                            &node.input_refs,
                            &ramp(0.25),
                            &mut energy,
                            0.0,
                            0,
                            &mut state,
                            &sensors(),
                            &config,
                            &mut side,
                            &mut memory,
                            &ramp(0.0),
                        );
                        assert!(!result.energy_exhausted);
                        readings.push((
                            side.work_counters,
                            side.energy_observation.graph_compute,
                            side.dispatch_effects(),
                            memory,
                        ));
                    }
                    assert_eq!(readings[0].2, readings[1].2);
                    assert_eq!(readings[0].3, readings[1].3);
                    if active {
                        assert_eq!(readings[0].0, readings[1].0);
                        assert_eq!(readings[0].1, readings[1].1);
                    } else {
                        assert_eq!(readings[0].0.graph_relax_iters, 0);
                        assert_eq!(readings[1].0.graph_relax_iters, 1);
                        assert_eq!(readings[0].1, 0.0);
                        assert_eq!(readings[1].1, f64::from(10.0f32 - (10.0f32 - rate)));
                    }
                    println!("RECRUIT_WORK rate={rate} initial_energy=10 active={active} {arm:?} {family:?}: iterations={}->{} graph_charge={:.9}->{:.9}", readings[0].0.graph_relax_iters, readings[1].0.graph_relax_iters, readings[0].1, readings[1].1);
                }
            }
        }
    }
}
