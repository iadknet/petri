//! T25.F03 edge bundles on the genome: size, identity, liveness and the
//! references a bundle keeps.
use proptest::prelude::*;
use rand::{rngs::SmallRng, SeedableRng};

use crate::config::OrdinaryFoodTypeId;
use crate::contracts::votes::DirectedVoteKind;
use crate::contracts::{InputReference, WorldInputKey};
use crate::creature::founder::{v3alpha1_founder_genome, FOUNDER_GENOME_SIZE_UNITS};
use crate::genome::cgp::{ring_offset, validate_graph, BundleId, CgpGraphBackendDef};
use crate::genome::cgp_mesh_annotations::derive_cgp_annotations;
use crate::genome::mesh_annotations::{MeshReadClass, MeshWriteClass};
use crate::genome::{BackendDef, CreatureGenome, NodeGenome};
use crate::mutation::input_ref::{prunable_indices, InputRefMutator, InputRefOperator};
use crate::neighborhood::mutation_effects::genome_identity;

fn reference_strategy() -> impl Strategy<Value = InputReference> {
    prop_oneof![
        (0u16..3).prop_map(|t| InputReference::World(WorldInputKey::neighbor_food_ring(
            OrdinaryFoodTypeId::new(t)
        ))),
        Just(InputReference::World(WorldInputKey::NeighborBarrierRing)),
        Just(InputReference::World(WorldInputKey::NeighborOccupiedRing)),
        Just(InputReference::ActionVotes),
        (0usize..24).prop_map(InputReference::UpstreamSlot),
    ]
}

fn founder_graph_index(genome: &CreatureGenome) -> usize {
    genome
        .nodes
        .iter()
        .position(|node| matches!(node.backend_def, BackendDef::Graph(_)))
        .expect("the founder carries a Graph node")
}

fn graph_mut(node: &mut NodeGenome) -> &mut CgpGraphBackendDef {
    match &mut node.backend_def {
        BackendDef::Graph(def) => def,
        BackendDef::Vm(_) => panic!("Graph fixture"),
    }
}

/// The founder with a barrier ring declared on its graph node; returns the
/// node index and the ring's reference index.
fn founder_with_ring() -> (CreatureGenome, usize, u16) {
    let mut genome = v3alpha1_founder_genome();
    let idx = founder_graph_index(&genome);
    let node = &mut genome.nodes[idx];
    let ring = u16::try_from(node.input_refs.len()).unwrap();
    node.input_refs
        .push(InputReference::World(WorldInputKey::NeighborBarrierRing));
    (genome, idx, ring)
}

proptest! {
    /// A `Ring` gene costs one unit, its record, and its members nothing
    /// (T25.F10): a surviving gene of any number of members costs 1.
    #[test]
    fn a_ring_gene_costs_one_unit_whatever_its_holes(
        bundles in prop::collection::vec((0usize..3, 0u8..8, 0usize..8), 0..6),
    ) {
        let (mut genome, idx, ring) = founder_with_ring();
        let before = genome.genome_size();
        for (earlier, (kind, offset, lost)) in (before..).zip(bundles) {
            let def = graph_mut(&mut genome.nodes[idx]);
            let id = def.add_ring_bundle(ring, DirectedVoteKind::ALL[kind], offset);
            prop_assert_eq!(genome.genome_size(), earlier + 1);
            let def = graph_mut(&mut genome.nodes[idx]);
            for _ in 0..lost {
                let member = def.members(id).next().unwrap();
                def.remove_edge(member);
            }
            prop_assert_eq!(genome.genome_size(), earlier + 1);
        }
        prop_assert!(validate_graph(&genome.nodes[idx]).is_ok());
    }

    /// Removing a reference no bundle addresses keeps every bundle of either
    /// shape (and its members) on the same `InputReference` (T25.F03,
    /// T25.F04).
    #[test]
    fn reindex_keeps_each_bundle_on_its_reference(
        mut refs in prop::collection::vec(reference_strategy(), 2..8),
        picks in prop::collection::vec(
            (any::<prop::sample::Index>(), 0usize..3, 0u8..8, any::<bool>()),
            0..4,
        ),
        removed in any::<prop::sample::Index>(),
    ) {
        refs.push(InputReference::World(WorldInputKey::NeighborBarrierRing));
        refs.push(InputReference::World(WorldInputKey::AreaOccupancySummary));
        let removed = removed.index(refs.len() - 2) as u16;
        let sites = |pair: bool| -> Vec<u16> {
            (0..refs.len() as u16)
                .filter(|&i| i != removed)
                .filter(|&i| {
                    let reference = &refs[usize::from(i)];
                    let InputReference::World(key) = reference else { return false };
                    if pair { !key.direction_pairs().is_empty() } else { key.is_neighbor_ring() }
                })
                .collect()
        };
        let (rings, pairs) = (sites(false), sites(true));
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
        for (index, kind, offset, pair) in &picks {
            let kind = DirectedVoteKind::ALL[*kind];
            if *pair {
                let ref_idx = pairs[index.index(pairs.len())];
                let layout = crate::contracts::pair_layout(&refs[usize::from(ref_idx)], 0);
                def.add_pair_bundle(ref_idx, 0, layout, kind, *offset);
            } else {
                let ring_ref = rings[index.index(rings.len())];
                def.add_ring_bundle(ring_ref, kind, *offset);
            }
        }
        let expected: Vec<InputReference> = def
            .bundles()
            .iter()
            .map(|bundle| {
                let ring_ref = bundle.source().ref_idx().expect("a sensor source");
                refs[usize::from(ring_ref)].clone()
            })
            .collect();
        let mut after_refs = refs.clone();
        after_refs.remove(usize::from(removed));
        def.reindex_input_refs_after_removal(removed);
        prop_assert_eq!(def.bundles().len(), expected.len());
        for (bundle, reference) in def.bundles().iter().zip(&expected) {
            let ring_ref = bundle.source().ref_idx().expect("a sensor source");
            prop_assert_eq!(&after_refs[usize::from(ring_ref)], reference);
        }
        let node = NodeGenome::new(crate::contracts::NodeId(0), after_refs, BackendDef::Graph(def), Vec::new());
        prop_assert!(validate_graph(&node).is_ok());
    }

    /// `Prune` never deletes a bundle's ring or pair reference, and every
    /// bundle reads the same reference after it.
    #[test]
    fn prune_keeps_each_bundle_on_its_reference(
        extra in prop::collection::vec(reference_strategy(), 1..6),
        offset in 0u8..8,
        seed in any::<u64>(),
    ) {
        let mut genome = v3alpha1_founder_genome();
        let idx = founder_graph_index(&genome);
        let node = &mut genome.nodes[idx];
        node.input_refs.extend(extra);
        let ring = node.input_refs.len() as u16;
        node.input_refs
            .push(InputReference::World(WorldInputKey::NeighborBarrierRing));
        graph_mut(node).add_ring_bundle(ring, DirectedVoteKind::Move, offset);
        let area = InputReference::World(WorldInputKey::AreaOccupancySummary);
        let summary = node.input_refs.len() as u16;
        node.input_refs.push(area.clone());
        let layout = crate::contracts::pair_layout(&area, 1);
        graph_mut(node).add_pair_bundle(summary, 1, layout, DirectedVoteKind::StealEnergy, offset);
        prop_assert!(!prunable_indices(node).contains(&usize::from(ring)));
        prop_assert!(!prunable_indices(node).contains(&usize::from(summary)));
        let mut rng = SmallRng::seed_from_u64(seed);
        while InputRefOperator::Prune.applies_to(&genome.nodes[idx], &crate::config::test_sensor_context(1)) {
            InputRefMutator::apply_to_node(&mut genome, InputRefOperator::Prune, idx, &mut rng, &crate::config::test_sensor_context(1))
                .unwrap();
        }
        let node = &mut genome.nodes[idx];
        let ring_ref = graph_mut(node).bundles()[0].source().ref_idx().unwrap();
        prop_assert_eq!(
            &node.input_refs[usize::from(ring_ref)],
            &InputReference::World(WorldInputKey::NeighborBarrierRing)
        );
        let pair_ref = graph_mut(node).bundles()[1].source().ref_idx().unwrap();
        prop_assert_eq!(&node.input_refs[usize::from(pair_ref)], &area);
        prop_assert!(validate_graph(node).is_ok());
    }
}

#[test]
fn genome_identity_distinguishes_bundles_and_a_founder_is_unchanged() {
    let founder = v3alpha1_founder_genome();
    assert_eq!(founder.genome_size(), FOUNDER_GENOME_SIZE_UNITS);
    let (ringed, idx, ring) = founder_with_ring();
    let mut grouped = ringed.clone();
    let id = graph_mut(&mut grouped.nodes[idx]).add_ring_bundle(ring, DirectedVoteKind::Move, 0);
    assert_ne!(genome_identity(&grouped), genome_identity(&ringed));
    let mut rescaled = grouped.clone();
    graph_mut(&mut rescaled.nodes[idx])
        .bundle_mut(id)
        .set_scale(0.5);
    assert_ne!(genome_identity(&rescaled), genome_identity(&grouped));
}

/// A member-only graph is a live component: its bundle record counts one, its
/// members wire eight sinks, the ring counts through the members, and its
/// annotations read the ring and write the action channel.
#[test]
fn a_bundle_is_a_live_component_reading_its_ring_and_writing_action() {
    let ring = InputReference::World(WorldInputKey::NeighborOccupiedRing);
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    assert!(!def.enters_visit());
    let empty = crate::genome::cgp_analysis::cgp_functional_complexity(&def);
    let id = def.add_ring_bundle(0, DirectedVoteKind::Reproduce, 3);
    assert_eq!(id, BundleId::new(0));
    assert!(def.enters_visit());
    // One record, eight wired sinks and one consumed reference.
    assert_eq!(
        crate::genome::cgp_analysis::cgp_functional_complexity(&def),
        empty + 1 + 8 + 1
    );
    let (reads, writes, stateful, _) = derive_cgp_annotations(&def, &[ring]);
    assert_eq!(reads, vec![MeshReadClass::Occupancy]);
    assert_eq!(writes, vec![MeshWriteClass::Action]);
    assert!(!stateful);
}

/// T25.F12 invariant 11: a bundle-only learner (`scale_learning` present,
/// every member `Static` and none crediting) carries learning for the
/// structural classifiers too: the annotations mark the node stateful and
/// the companions report plasticity, as the layout does.
#[test]
fn a_bundle_only_learner_is_classified_as_learning() {
    use crate::genome::cgp::LearningSlot;
    use crate::genome::companions::structural_companions;
    use crate::genome::learning::LearningLayout;
    use crate::genome::{HebbianRule, Learning};

    let ring = InputReference::World(WorldInputKey::NeighborOccupiedRing);
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    let id = def.add_ring_bundle(0, DirectedVoteKind::Move, 0);
    def.bundle_mut(id).set_slot(
        LearningSlot::Scale,
        Some(Learning {
            rule: HebbianRule::Classic,
            learning_rate: 0.1,
            weight_clamp: 2.0,
            lamarckian: false,
            modulation: None,
            retention: 1.0,
        }),
    );
    let refs = vec![ring];
    let layout = LearningLayout::of(&def, &refs);
    assert!(layout.edges.is_empty());
    assert_eq!(layout.scales.len(), 1);
    assert!(layout.scales[0].contributors.is_empty());
    let (_, _, stateful, _) = derive_cgp_annotations(&def, &refs);
    assert!(stateful);
    let genome = CreatureGenome::new(
        crate::contracts::NodeId(0),
        vec![NodeGenome::new(
            crate::contracts::NodeId(0),
            refs,
            BackendDef::Graph(def),
            Vec::new(),
        )],
        crate::creature::founder::default_founder_body(),
    );
    assert!(structural_companions(&genome).has_plasticity);
}

/// The ring offset is `(d − channel) mod 8` for every `u8` pair, not only
/// for the eight directions and channels the census reads.
#[test]
fn ring_offset_is_mod_eight_for_every_u8_pair() {
    let rotation =
        |a: u8, b: u8| u8::try_from((i16::from(a) - i16::from(b)).rem_euclid(8)).expect("below 8");
    for a in 0..=u8::MAX {
        for b in 0..=u8::MAX {
            assert_eq!(ring_offset(a, b), rotation(a, b), "d {a}, channel {b}");
        }
    }
}
