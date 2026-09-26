//! Existing mutation, copy, serde and reference-repair seams for zero edges.
use super::*;
use crate::config::MutationConfig;
use crate::mutation::graph::operators::{alter_edge_weight_in_def, remove_edge};
use crate::mutation::input_ref::{prunable_indices, InputRefMutator, InputRefOperator};
use crate::mutation::reachability::TargetSelector;
use crate::mutation::topology::{TopologyMutator, TopologyOperator};
use crate::mutation::types::MutationSkipReason;
use rand::{rngs::SmallRng, Rng, SeedableRng};

fn duplicate_node() -> NodeGenome {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    for weight in [0.0, 0.0, 0.0] {
        wire(
            &mut def,
            OutputSinkKind::ActionVote(VoteSink::Move(5)),
            edge(leaf(0, 6), weight),
        );
    }
    graph_node(
        def,
        vec![InputReference::World(WorldInputKey::NeighborBarrierRing)],
    )
}

#[test]
fn existing_weight_mutation_refines_one_zero_duplicate_without_touching_the_original() {
    let original = duplicate_node();
    let mut copy = original.clone();
    alter_edge_weight_in_def(graph_mut(&mut copy), &mut SmallRng::seed_from_u64(11)).unwrap();
    let before: Vec<_> = graph(&original).edges().cloned().collect();
    let after: Vec<_> = graph(&copy).edges().cloned().collect();
    assert_eq!(after.iter().zip(&before).filter(|(a, b)| a != b).count(), 1);
    assert!(before.iter().all(|e| e.weight == 0.0));
    let votes = effects(graph(&copy), &copy.input_refs, 0.25)
        .side
        .dispatch_effects()
        .0;
    assert_ne!(votes[VoteSink::Move(5).index()], 0.0);
    assert_eq!(votes.iter().filter(|v| **v != 0.0).count(), 1);
    remove_edge(graph_mut(&mut copy), &mut SmallRng::seed_from_u64(5)).unwrap();
    assert_eq!(graph(&copy).edges().count(), 2);
    assert_eq!(graph(&original).edges().count(), 3);
}

proptest! {
    #[test]
    fn zero_duplicate_mutation_and_deletion_follow_independent_occurrence_draws(seed in any::<u64>()) {
        let original = duplicate_node();
        let mut mutated = original.clone();
        let mut rng = SmallRng::seed_from_u64(seed);
        let mut replay = rng.clone();
        let selected = replay.gen_range(0usize..3);
        let delta = replay.gen_range(-0.1f32..=0.1);
        alter_edge_weight_in_def(graph_mut(&mut mutated), &mut rng).unwrap();
        let weights: Vec<_> = graph(&mutated).edges().map(|e| e.weight).collect();
        for (index, weight) in weights.iter().enumerate() {
            prop_assert_eq!(*weight, if index == selected { delta } else { 0.0 });
        }
        prop_assert_eq!(rng.gen::<u64>(), replay.gen::<u64>());
        // Distinct weights expose which occurrence deletion removed.
        for (index, e) in graph_mut(&mut mutated).output_sinks.iter_mut().flat_map(|s| &mut s.inputs).enumerate() {
            e.weight = index as f32 + 0.25;
        }
        let before: Vec<_> = graph(&mutated).edges().cloned().collect();
        let mut replay = rng.clone();
        let removed = replay.gen_range(0usize..3);
        remove_edge(graph_mut(&mut mutated), &mut rng).unwrap();
        let mut expected = before;
        expected.remove(removed);
        prop_assert_eq!(graph(&mutated).edges().cloned().collect::<Vec<_>>(), expected);
        prop_assert_eq!(rng.gen::<u64>(), replay.gen::<u64>());
        prop_assert_eq!(graph(&original).edges().count(), 3);
    }

    #[test]
    fn pruning_an_unused_earlier_declaration_preserves_zero_edge_source_meaning(
        unused in 1usize..12,
        channel in 0u16..8,
        seed in any::<u64>(),
    ) {
        let source_ref = InputReference::World(WorldInputKey::NeighborFoodRing { type_idx: OrdinaryFoodTypeId::new(1) });
        let mut refs = vec![InputReference::StaticIntrospection(StaticIntrospectionKey::AgeTicks); unused];
        refs.push(source_ref.clone());
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
        for weight in [0.0, 0.5] {
            wire(&mut def, OutputSinkKind::ActionVote(VoteSink::Reproduce(7)), edge(leaf(unused as u16, channel), weight));
        }
        let mut genome = CreatureGenome { entry_node_id: NodeId::new(0), nodes: vec![graph_node(def, refs)] };
        let before = effects(graph(&genome.nodes[0]), &genome.nodes[0].input_refs, 0.25).side.dispatch_effects();
        let mut rng = SmallRng::seed_from_u64(seed);
        InputRefMutator::apply_to_node(&mut genome, InputRefOperator::Prune, 0, &mut rng, 2).unwrap();
        let node = &genome.nodes[0];
        prop_assert_eq!(&node.input_refs[unused - 1], &source_ref);
        for edge in graph(node).edges() {
            prop_assert_eq!(edge.source, leaf((unused - 1) as u16, channel));
        }
        prop_assert_eq!(effects(graph(node), &node.input_refs, 0.25).side.dispatch_effects(), before);
        prop_assert!(!prunable_indices(node).contains(&(unused - 1)));
    }

    #[test]
    fn zero_connection_storage_units_count_edge_sink_and_optional_declaration(
        incumbent_edges in 0usize..10,
        add_declaration in any::<bool>(),
        vote_index in 0usize..27,
    ) {
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
        let kind = OutputSinkKind::ActionVote(VoteSink::from_index(vote_index).unwrap());
        for _ in 0..incumbent_edges { wire(&mut def, kind, edge(memory(0, false), 0.5)); }
        let mut genome = CreatureGenome { entry_node_id: NodeId::new(0), nodes: vec![graph_node(def, vec![])] };
        let before = genome.genome_size();
        let source = if add_declaration {
            genome.nodes[0].input_refs.push(InputReference::World(WorldInputKey::NearbyCreatureVitals));
            leaf(0, 7)
        } else { memory(15, true) };
        wire(graph_mut(&mut genome.nodes[0]), kind, edge(source, 0.0));
        prop_assert_eq!(genome.genome_size() - before, 1 + u32::from(incumbent_edges == 0) + u32::from(add_declaration));
        let roundtrip: CreatureGenome = serde_json::from_str(&serde_json::to_string(&genome).unwrap()).unwrap();
        prop_assert_eq!(&roundtrip, &genome);
        prop_assert_eq!(roundtrip.genome_size(), genome.genome_size());
    }
}

#[test]
fn last_zero_consumer_removal_enables_pruning_and_never_recreates_the_edge() {
    let mut genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![duplicate_node()],
    };
    let mut rng = SmallRng::seed_from_u64(17);
    for remaining in (1..=3).rev() {
        assert_eq!(graph(&genome.nodes[0]).edges().count(), remaining);
        assert!(prunable_indices(&genome.nodes[0]).is_empty());
        assert_eq!(
            InputRefMutator::apply_to_node(&mut genome, InputRefOperator::Prune, 0, &mut rng, 1),
            Err(MutationSkipReason::NoApplicableTarget)
        );
        remove_edge(graph_mut(&mut genome.nodes[0]), &mut rng).unwrap();
    }
    assert_eq!(prunable_indices(&genome.nodes[0]), vec![0]);
    InputRefMutator::apply_to_node(&mut genome, InputRefOperator::Prune, 0, &mut rng, 1).unwrap();
    assert!(genome.nodes[0].input_refs.is_empty());
    assert_eq!(
        remove_edge(graph_mut(&mut genome.nodes[0]), &mut rng),
        Err(MutationSkipReason::NoApplicableTarget)
    );
    assert_eq!(graph(&genome.nodes[0]).edges().count(), 0);
}

#[test]
fn native_node_copy_and_genome_serde_retain_channels_sink_identity_and_independent_weights() {
    let mut entry = graph_node(CgpGraphBackendDef::new_with_fixed_outputs(), vec![]);
    entry.targets.push(RouteTarget {
        target_id: NodeId::new(1),
        slot: 0,
        gate_bias: 1.0,
    });
    let mut worker = duplicate_node();
    worker.node_id = NodeId::new(1);
    for vote in VoteSink::all() {
        wire(
            graph_mut(&mut worker),
            OutputSinkKind::ActionVote(vote),
            edge(memory(7, true), 0.25),
        );
    }
    let original_worker = worker.clone();
    let mut genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![entry, worker],
    };
    TopologyMutator::apply(
        &mut genome,
        TopologyOperator::CopyNode,
        &mut TargetSelector::reachable_only(&[0, 1], 0.0),
        &mut SmallRng::seed_from_u64(7),
        &MutationConfig::default(),
    )
    .unwrap();
    assert_eq!(genome.nodes.len(), 3);
    assert_eq!(genome.nodes[2].input_refs, original_worker.input_refs);
    assert_eq!(genome.nodes[2].backend_def, original_worker.backend_def);
    assert_ne!(genome.nodes[2].node_id, original_worker.node_id);
    let mut decoded: CreatureGenome =
        serde_json::from_str(&serde_json::to_string(&genome).unwrap()).unwrap();
    assert_eq!(decoded, genome);
    alter_edge_weight_in_def(
        graph_mut(&mut decoded.nodes[2]),
        &mut SmallRng::seed_from_u64(11),
    )
    .unwrap();
    remove_edge(
        graph_mut(&mut decoded.nodes[2]),
        &mut SmallRng::seed_from_u64(5),
    )
    .unwrap();
    assert_eq!(decoded.nodes[1], original_worker);
    assert_eq!(genome.nodes[1], original_worker);
    assert_eq!(graph(&genome.nodes[2]).edges().count(), 30);
    assert_eq!(graph(&decoded.nodes[2]).edges().count(), 29);
}
