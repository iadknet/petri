//! Structured mutation uses the same inherited edges, input resolver and votes.
use super::*;
use crate::config::MutationConfig;
use crate::mutation::graph::operators::{alter_edge_weight_in_def, remove_edge};
use crate::mutation::graph::refinement::refine;
use crate::mutation::graph::{GraphMutator, GraphOperator};
use crate::mutation::input_ref::{InputRefMutator, InputRefOperator};
use crate::mutation::reachability::TargetSelector;
use crate::mutation::topology::{TopologyMutator, TopologyOperator};
use rand::{rngs::SmallRng, Rng, SeedableRng};
use std::time::Instant;

fn fixture(ring: bool) -> CreatureGenome {
    let key = if ring {
        WorldInputKey::neighbor_food_ring(OrdinaryFoodTypeId::new(0))
    } else {
        WorldInputKey::NearbyCreatureCore
    };
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    for i in 0..if ring { 8 } else { 4 } {
        let vote = if ring {
            VoteSink::Move(i)
        } else {
            VoteSink::Eat
        };
        let channel = if ring { u16::from(i) } else { u16::from(i) * 4 };
        wire(
            &mut def,
            OutputSinkKind::ActionVote(vote),
            edge(leaf(0, channel), if i == 0 { 0.025 } else { 0.0 }),
        );
    }
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![graph_node(def, vec![InputReference::World(key)])],
    }
}

fn weights(genome: &CreatureGenome) -> Vec<f32> {
    graph(&genome.nodes[0])
        .edges()
        .map(|edge| edge.weight)
        .collect()
}

fn native_run(
    genome: &CreatureGenome,
    ring: bool,
) -> (
    crate::creature::genome::vote::VoteVector,
    crate::runtime::types::WorkCounters,
    f64,
) {
    let node = &genome.nodes[0];
    let mut energy = 10.0;
    let mut state = GraphRuntimeState::new();
    let mut side = MeshSideOutputs::new(8);
    let mut memory = [0.0; 16];
    let snapshot = sensors();
    let result = execute_graph_node(
        graph(node),
        &node.input_refs,
        &[0.0; OUTPUT_SLOT_COUNT],
        &mut energy,
        0.0,
        0,
        &mut state,
        &snapshot,
        &RuntimeConfig::default(),
        &mut side,
        &mut memory,
        &[0.0; 16],
    );
    assert!(!result.energy_exhausted);
    let staged = side.dispatch_effects().0;
    side.commit_vote_contribution(0);
    assert_eq!(side.votes, staged);
    let mut expected = [0.0; VOTE_SINK_COUNT];
    for sink in &graph(node).output_sinks {
        let OutputSinkKind::ActionVote(vote) = sink.kind else {
            continue;
        };
        for edge in &sink.inputs {
            let GraphSource::InputLeaf { sub_idx, .. } = edge.source else {
                panic!("fixture leaf")
            };
            let input = if ring {
                snapshot.typed_local_food.neighbor_food_by_type[0][usize::from(sub_idx)]
            } else {
                snapshot.perception.nearby_core[usize::from(sub_idx)]
            };
            assert!(input > 0.0);
            expected[vote.index()] += edge.weight * input;
        }
    }
    assert_eq!(
        side.votes, expected,
        "native input resolution and applied vote contribution"
    );
    assert_eq!(side.work_counters.plasticity_updates, 0);
    (
        side.votes,
        side.work_counters,
        side.energy_observation.graph_compute,
    )
}

#[test]
fn structured_refinement_reaches_native_votes_without_adding_native_work_or_units() {
    for ring in [true, false] {
        let original = fixture(ring);
        let mut changed = original.clone();
        let before = native_run(&original, ring);
        refine(
            &mut changed.nodes[0],
            &mut SmallRng::seed_from_u64(29_005_000),
        )
        .unwrap();
        let after = native_run(&changed, ring);
        assert_ne!(before.0, after.0);
        assert_eq!(before.1, after.1);
        assert_eq!(before.2, after.2);
        assert_eq!(before.1.graph_relax_iters, 1);
        assert_eq!(changed.genome_size(), original.genome_size());
        let original_weights = weights(&original);
        let changed_weights = weights(&changed);
        for i in 1..original_weights.len() {
            assert!(
                ((changed_weights[0] - changed_weights[i])
                    - (original_weights[0] - original_weights[i]))
                    .abs()
                    < 1e-7
            );
        }
    }
}

#[test]
fn local_scalar_exceptions_sparse_deletion_copy_serde_and_pruning_persist() {
    for ring in [true, false] {
        let mut genome = fixture(ring);
        let mut rng = SmallRng::seed_from_u64(29_005_001);
        let original = weights(&genome);
        alter_edge_weight_in_def(graph_mut(&mut genome.nodes[0]), &mut rng).unwrap();
        let independent = weights(&genome);
        assert_eq!(
            independent
                .iter()
                .zip(&original)
                .filter(|(a, b)| a != b)
                .count(),
            1
        );
        refine(&mut genome.nodes[0], &mut rng).unwrap();
        let coordinated = weights(&genome);
        for i in 1..independent.len() {
            assert!(
                ((coordinated[0] - coordinated[i]) - (independent[0] - independent[i])).abs()
                    < 1e-7
            );
        }
        remove_edge(graph_mut(&mut genome.nodes[0]), &mut rng).unwrap();
        let sparse_count = graph(&genome.nodes[0]).edges().count();
        refine(&mut genome.nodes[0], &mut rng).unwrap();
        assert_eq!(graph(&genome.nodes[0]).edges().count(), sparse_count);
        let original_node = genome.nodes[0].clone();
        // Native copy needs a route to a non-entry donor.
        let mut entry = graph_node(CgpGraphBackendDef::new_with_fixed_outputs(), vec![]);
        entry.node_id = NodeId::new(1);
        entry.targets.push(RouteTarget {
            target_id: NodeId::new(0),
            slot: 0,
            gate_bias: 1.0,
        });
        genome.nodes.insert(0, entry);
        genome.entry_node_id = NodeId::new(1);
        TopologyMutator::apply(
            &mut genome,
            TopologyOperator::CopyNode,
            &mut TargetSelector::reachable_only(&[1], 1.0),
            &mut rng,
            &MutationConfig::default(),
        )
        .unwrap();
        assert_eq!(genome.nodes[2].backend_def, original_node.backend_def);
        assert_eq!(genome.nodes[2].input_refs, original_node.input_refs);
        let mut decoded: CreatureGenome =
            serde_json::from_str(&serde_json::to_string(&genome).unwrap()).unwrap();
        assert_eq!(decoded, genome);
        while graph(&decoded.nodes[2]).edges().count() > 1 {
            remove_edge(graph_mut(&mut decoded.nodes[2]), &mut rng).unwrap();
        }
        let lone = decoded.nodes[2].clone();
        assert_eq!(
            refine(&mut decoded.nodes[2], &mut rng),
            Err(crate::mutation::types::MutationSkipReason::NoApplicableTarget)
        );
        assert_eq!(decoded.nodes[2], lone);
        remove_edge(graph_mut(&mut decoded.nodes[2]), &mut rng).unwrap();
        InputRefMutator::apply_to_node(&mut decoded, InputRefOperator::Prune, 2, &mut rng, 1)
            .unwrap();
        assert!(decoded.nodes[2].input_refs.is_empty());
        let empty = decoded.nodes[2].clone();
        for _ in 0..3 {
            assert!(refine(&mut decoded.nodes[2], &mut rng).is_err());
            assert_eq!(
                effects(graph(&decoded.nodes[2]), &decoded.nodes[2].input_refs, 0.5)
                    .side
                    .dispatch_effects()
                    .0,
                [0.0; VOTE_SINK_COUNT]
            );
        }
        assert_eq!(decoded.nodes[2], empty);
        assert_eq!(genome.nodes[1], original_node);
    }
}

#[test]
fn structured_coefficients_and_deleted_positions_survive_native_offspring_construction() {
    use crate::simulation::{
        actions::{apply_reproduce, ReproductionActionResult},
        seed_simulation,
    };
    for ring in [true, false] {
        let mut genome = fixture(ring);
        let mut rng = SmallRng::seed_from_u64(29_005_002);
        refine(&mut genome.nodes[0], &mut rng).unwrap();
        remove_edge(graph_mut(&mut genome.nodes[0]), &mut rng).unwrap();
        let mut config = crate::config::SimulationConfig::default();
        config.population.initial_creatures = 1;
        config.mutation.per_unit_rate = 0.0;
        let mut sim = seed_simulation(config, 83);
        let parent_id = sim.creatures.keys().next().unwrap();
        let parent = &mut sim.creatures[parent_id];
        parent.genome = genome.clone();
        parent.cached_genome_size = genome.genome_size();
        parent.age = sim.config.energy.lifecycle.min_reproduce_age;
        parent.energy = 10000.0;
        let direction = Direction::ALL
            .into_iter()
            .find(|&d| {
                sim.world
                    .resolve_neighbor(sim.creatures[parent_id].position, d)
                    .is_some_and(|p| sim.world.is_valid_target_cell(p))
            })
            .unwrap();
        assert_eq!(
            apply_reproduce(parent_id, &mut sim, direction, 1.0, &mut rng),
            ReproductionActionResult::Spawned
        );
        let child = sim
            .creatures
            .iter()
            .find(|(id, _)| *id != parent_id)
            .unwrap()
            .1;
        assert_eq!(child.genome, genome);
        assert_eq!(child.cached_genome_size, genome.genome_size());
        assert_eq!(sim.creatures[parent_id].genome, genome);
        native_run(&child.genome, ring);
    }
}

#[derive(Debug, Clone, Copy)]
enum Arm {
    Scalar,
    MatchedAdditive,
    Coordinated,
}

/// Replay only to read the requested amplitude; the replay never guides mutation.
fn proposal(genome: &mut CreatureGenome, arm: Arm, seed: u64) -> (f64, Vec<usize>) {
    let mut rng = SmallRng::seed_from_u64(seed);
    let mut replay = rng.clone();
    TargetSelector::reachable_only(&[0], 0.0)
        .select(&[0], &mut replay)
        .unwrap();
    let config = MutationConfig {
        structured_heritable_refinement: true,
        ..MutationConfig::default()
    };
    if matches!(arm, Arm::Scalar) {
        let old = weights(genome);
        let selected = replay.gen_range(0..old.len());
        let requested = if old[selected].abs() > 0.01 {
            old[selected] * replay.gen_range(-0.2f32..=0.2)
        } else {
            replay.gen_range(-0.1f32..=0.1)
        };
        GraphMutator::apply(
            genome,
            GraphOperator::AlterGraphEdgeWeight,
            &mut TargetSelector::reachable_only(&[0], 0.0),
            &mut rng,
            &config,
        )
        .unwrap();
        assert_eq!(rng.gen::<u64>(), replay.gen::<u64>());
        return (f64::from(requested.abs()), vec![selected]);
    }
    let _group = replay.gen_range(0..1usize);
    let d = replay.gen_range(-0.1f32..=0.1);
    if matches!(arm, Arm::Coordinated) {
        GraphMutator::apply(
            genome,
            GraphOperator::RefineHeritableStructure,
            &mut TargetSelector::reachable_only(&[0], 0.0),
            &mut rng,
            &config,
        )
        .unwrap();
        assert_eq!(rng.gen::<u64>(), replay.gen::<u64>());
    } else {
        let increment = d.abs() / (weights(genome).len() as f32).sqrt();
        for edge in graph_mut(&mut genome.nodes[0])
            .output_sinks
            .iter_mut()
            .flat_map(|s| &mut s.inputs)
        {
            edge.weight += if replay.gen_bool(0.5) {
                increment
            } else {
                -increment
            };
        }
    }
    (f64::from(d.abs()), (0..weights(genome).len()).collect())
}

#[test]
#[ignore = "bounded T20.F05 release reading: 1,536 proposals, not inherited discovery"]
fn structured_refinement_engineering_panel() {
    let started = Instant::now();
    let mut rows = Vec::new();
    for ring in [true, false] {
        let original = fixture(ring);
        let old = weights(&original);
        let m = old.len();
        let before = native_run(&original, ring);
        for arm in [Arm::Scalar, Arm::MatchedAdditive, Arm::Coordinated] {
            let arm_started = Instant::now();
            let mut touches = 0;
            let mut changes = 0;
            let mut unchanged_proposals = 0;
            let mut preserved = 0;
            let mut norm_sum = 0.0;
            let mut norm_max = 0.0f64;
            let mut requested_max = 0.0f64;
            let mut vote_change_sum = 0;
            let mut vote_norm_sum = 0.0;
            let mut visits = 0;
            let mut energy = 0.0;
            let mut records = Vec::new();
            for i in 0..256 {
                assert!(
                    started.elapsed().as_secs_f64() < 60.0,
                    "release execution cap"
                );
                let seed = 29_005_000 + i;
                let mut child = original.clone();
                let (requested, selected_members) = proposal(&mut child, arm, seed);
                requested_max = requested_max.max(requested);
                let new = weights(&child);
                let changed = new.iter().zip(&old).filter(|(a, b)| a != b).count();
                let selected = if matches!(arm, Arm::Scalar) { 1 } else { m };
                touches += selected;
                changes += changed;
                unchanged_proposals += usize::from(changed == 0);
                let norm = new
                    .iter()
                    .zip(&old)
                    .map(|(a, b)| (f64::from(*a) - f64::from(*b)).powi(2))
                    .sum::<f64>()
                    .sqrt();
                assert!(norm <= 0.100001);
                norm_sum += norm;
                norm_max = norm_max.max(norm);
                let exception_preserved =
                    (1..m).all(|j| ((new[0] - new[j]) - (old[0] - old[j])).abs() < 1e-7);
                preserved += usize::from(exception_preserved);
                if matches!(arm, Arm::Coordinated) {
                    assert!(exception_preserved);
                }
                assert_eq!(child.genome_size(), original.genome_size());
                let after = native_run(&child, ring);
                assert_eq!(after.1, before.1);
                assert_eq!(after.2, before.2);
                let vote_changes = after
                    .0
                    .iter()
                    .zip(before.0)
                    .filter(|(a, b)| **a != *b)
                    .count();
                let vote_norm = after
                    .0
                    .iter()
                    .zip(before.0)
                    .map(|(a, b)| (f64::from(*a) - f64::from(b)).powi(2))
                    .sum::<f64>()
                    .sqrt();
                vote_change_sum += vote_changes;
                vote_norm_sum += vote_norm;
                visits += after.1.graph_relax_iters;
                energy += after.2;
                records.push(serde_json::json!({"seed":seed,"selected":selected,"changed":changed,"selected_members":selected_members,"changed_members":new.iter().zip(&old).enumerate().filter_map(|(index,(a,b))| (a!=b).then_some(index)).collect::<Vec<_>>(),"requested_norm":requested,"actual_norm":norm,"exception_preserved":exception_preserved,"vote_changes":vote_changes,"vote_delta_norm":vote_norm}));
            }
            rows.push(serde_json::json!({"layout":if ring {"food_ring_move_offset_0"} else {"nearby_core_present_eat"},"arm":format!("{arm:?}"),"attempted":256,"applied":256,"skipped":0,"numeric_rejections":0,"selected_node":0,"selected_groups":if matches!(arm,Arm::Scalar){0}else{256},"selected_coefficients":touches,"changed_coefficients":changes,"unchanged_proposals":unchanged_proposals,"requested_norm_max":requested_max,"actual_norm_mean":norm_sum/256.0,"actual_norm_max":norm_max,"exceptions_preserved":preserved,"changed_vote_coefficients":vote_change_sum,"vote_delta_norm_mean":vote_norm_sum/256.0,"genomic_units":original.genome_size(),"net_units":0,"native_graph_visits":visits,"plasticity_updates":0,"native_graph_energy":energy,"elapsed_seconds":arm_started.elapsed().as_secs_f64(),"proposals":records}));
        }
    }
    let evidence=serde_json::to_string_pretty(&serde_json::json!({"proposals":1536,"seed_start":29_005_000,"seed_end_exclusive":29_005_256,"learning":false,"inputs":{"food_ring":sensors().typed_local_food.neighbor_food_by_type[0],"nearby_core_present":[sensors().perception.nearby_core[0],sensors().perception.nearby_core[4],sensors().perception.nearby_core[8],sensors().perception.nearby_core[12]]},"rows":rows,"elapsed_seconds":started.elapsed().as_secs_f64(),"verdicts":{"ring":"correctness_pass","slots":"correctness_pass"}})).unwrap();
    assert!(evidence.len() <= 10 * 1024 * 1024);
    println!("{evidence}");
    assert!(started.elapsed().as_secs_f64() < 60.0);
}

#[test]
fn refinement_preserves_compute_plasticity_birth_routes_and_nonvote_outputs() {
    let mut genome = fixture(true);
    let active = active_graph();
    let graph = graph_mut(&mut genome.nodes[0]);
    graph.compute_nodes = active.compute_nodes;
    graph.birth_weights = Some(vec![vec![Some(0.75)], vec![Some(-0.5)]]);
    graph
        .output_sinks
        .iter_mut()
        .find(|s| s.kind == OutputSinkKind::CustomOutput(0))
        .unwrap()
        .inputs
        .push(edge(leaf(0, 0), 0.5));
    // Ring-like compute inputs and a shared-memory vote edge are not members.
    graph.compute_nodes[0]
        .inputs
        .extend([edge(leaf(0, 0), 0.5), edge(leaf(0, 1), 0.5)]);
    graph
        .output_sinks
        .iter_mut()
        .find(|s| s.kind == OutputSinkKind::ActionVote(VoteSink::Eat))
        .unwrap()
        .inputs
        .push(edge(memory(0, false), 0.5));
    genome.nodes[0].targets.push(RouteTarget {
        target_id: NodeId::new(0),
        slot: 0,
        gate_bias: 0.5,
    });
    let before = genome.clone();
    refine(&mut genome.nodes[0], &mut SmallRng::seed_from_u64(23)).unwrap();
    let (old, new) = (
        super::graph(&before.nodes[0]),
        super::graph(&genome.nodes[0]),
    );
    assert_eq!(old.compute_nodes, new.compute_nodes);
    assert_eq!(old.birth_weights, new.birth_weights);
    assert_eq!(before.nodes[0].targets, genome.nodes[0].targets);
    for (old, new) in old.output_sinks.iter().zip(&new.output_sinks) {
        if !matches!(old.kind, OutputSinkKind::ActionVote(VoteSink::Move(_))) {
            assert_eq!(old, new);
        }
    }
    assert_eq!(before.genome_size(), genome.genome_size());
}
