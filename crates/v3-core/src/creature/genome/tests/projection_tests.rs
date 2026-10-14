//! T11.F31 ring projections on the genome: serde, identity, size, liveness
//! and the references a projection keeps.
use proptest::prelude::*;
use rand::{rngs::SmallRng, SeedableRng};

use crate::config::OrdinaryFoodTypeId;
use crate::contracts::{InputReference, WorldInputKey};
use crate::creature::founder::{v3alpha1_founder_genome, FOUNDER_GENOME_SIZE_UNITS};
use crate::creature::genome::cgp::{CgpGraphBackendDef, RingProjection};
use crate::creature::genome::cgp_mesh_annotations::derive_cgp_annotations;
use crate::creature::genome::mesh_annotations::{MeshReadClass, MeshWriteClass};
use crate::creature::genome::vote::DirectedVoteKind;
use crate::creature::genome::{BackendDef, CreatureGenome, NodeGenome};
use crate::mutation::input_ref::{prunable_indices, InputRefMutator, InputRefOperator};
use crate::neighborhood::mutation_effects::genome_identity;

fn projection_strategy() -> impl Strategy<Value = RingProjection> {
    (any::<u16>(), 0usize..3, any::<u8>(), -8.0f32..8.0).prop_map(
        |(ref_idx, kind, offset, weight)| {
            RingProjection::new(ref_idx, DirectedVoteKind::ALL[kind], offset, weight)
        },
    )
}

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

proptest! {
    #[test]
    fn serde_round_trips_projections_and_omits_an_empty_list(
        projections in prop::collection::vec(projection_strategy(), 0..5),
        ablated in any::<u8>(),
    ) {
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
        def.projections = projections;
        for projection in &mut def.projections {
            projection.ablated_channels = ablated;
        }
        let json = serde_json::to_string(&def).unwrap();
        prop_assert_eq!(json.contains("projections"), !def.projections.is_empty());
        prop_assert!(!json.contains("ablated"));
        let back: CgpGraphBackendDef = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&back, &def);
        prop_assert!(back.projections.iter().all(|p| p.ablated_channels == 0));
        prop_assert_eq!(format!("{back:?}"), format!("{def:?}"));
    }

    /// T25.F02: a projection costs its record plus its eight implied
    /// synapses, so F03's eight-member groups replace it size-neutrally.
    #[test]
    fn a_ring_projection_costs_nine_units(
        projections in prop::collection::vec(projection_strategy(), 0..12),
    ) {
        let mut genome = v3alpha1_founder_genome();
        let before = genome.genome_size();
        let idx = founder_graph_index(&genome);
        graph_mut(&mut genome.nodes[idx]).projections = projections.clone();
        prop_assert_eq!(genome.genome_size(), before + 9 * projections.len() as u32);
    }

    /// Removing a reference no projection names keeps every projection on
    /// the same `InputReference`.
    #[test]
    fn reindex_keeps_each_projection_on_its_reference(
        refs in prop::collection::vec(reference_strategy(), 2..8),
        picks in prop::collection::vec((any::<prop::sample::Index>(), projection_strategy()), 0..6),
        removed in any::<prop::sample::Index>(),
    ) {
        let removed = removed.index(refs.len()) as u16;
        // Every projection names a reference other than the removed one.
        let others: Vec<u16> = (0..refs.len() as u16).filter(|&i| i != removed).collect();
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
        def.projections = picks
            .iter()
            .map(|(index, p)| RingProjection { ref_idx: others[index.index(others.len())], ..*p })
            .collect();
        let expected: Vec<(InputReference, RingProjection)> = def
            .projections
            .iter()
            .map(|p| (refs[usize::from(p.ref_idx)].clone(), *p))
            .collect();
        let mut after_refs = refs.clone();
        after_refs.remove(usize::from(removed));
        def.reindex_input_refs_after_removal(removed);
        prop_assert_eq!(def.projections.len(), expected.len());
        for (p, (reference, old)) in def.projections.iter().zip(&expected) {
            prop_assert_eq!(&after_refs[usize::from(p.ref_idx)], reference);
            prop_assert_eq!((p.kind, p.offset, p.weight), (old.kind, old.offset, old.weight));
        }
    }

    /// `Prune` never deletes a projection's ring, and every projection reads
    /// the same reference after it.
    #[test]
    fn prune_keeps_each_projection_on_its_reference(
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
        graph_mut(node)
            .projections
            .push(RingProjection::new(ring, DirectedVoteKind::Move, offset, 0.5));
        prop_assert!(!prunable_indices(node).contains(&usize::from(ring)));
        let mut rng = SmallRng::seed_from_u64(seed);
        while InputRefOperator::Prune.applies_to(&genome.nodes[idx], 1) {
            InputRefMutator::apply_to_node(&mut genome, InputRefOperator::Prune, idx, &mut rng, 1)
                .unwrap();
        }
        let node = &mut genome.nodes[idx];
        let projection = graph_mut(node).projections[0];
        prop_assert_eq!(
            &node.input_refs[usize::from(projection.ref_idx)],
            &InputReference::World(WorldInputKey::NeighborBarrierRing)
        );
    }
}

#[test]
fn equality_and_debug_ignore_ablation_state_and_distinguish_projections() {
    let mut a = CgpGraphBackendDef::new_with_fixed_outputs();
    let empty_debug = format!("{a:?}");
    assert!(!empty_debug.contains("projections"));
    a.projections
        .push(RingProjection::new(0, DirectedVoteKind::Move, 1, 0.0));
    let mut b = a.clone();
    b.projections[0].ablated_channels = 0b1010_0101;
    assert_eq!(a, b);
    assert_eq!(format!("{a:?}"), format!("{b:?}"));
    assert!(format!("{a:?}").contains("projections"));
    let mut c = a.clone();
    c.projections[0].weight = 0.25;
    assert_ne!(a, c);
}

#[test]
fn genome_identity_distinguishes_projections_and_a_founder_is_unchanged() {
    let founder = v3alpha1_founder_genome();
    assert_eq!(founder.genome_size(), FOUNDER_GENOME_SIZE_UNITS);
    let mut projected = founder.clone();
    let idx = founder_graph_index(&projected);
    graph_mut(&mut projected.nodes[idx])
        .projections
        .push(RingProjection::new(0, DirectedVoteKind::Move, 0, 0.0));
    assert_ne!(genome_identity(&projected), genome_identity(&founder));
    let mut reweighted = projected.clone();
    graph_mut(&mut reweighted.nodes[idx]).projections[0].weight = 0.5;
    assert_ne!(genome_identity(&reweighted), genome_identity(&projected));
}

#[test]
fn a_projection_is_a_live_component_reading_its_ring_and_writing_action() {
    let ring = InputReference::World(WorldInputKey::NeighborOccupiedRing);
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    assert!(!def.enters_visit());
    let empty = crate::creature::genome::cgp_analysis::cgp_functional_complexity(&def);
    def.projections
        .push(RingProjection::new(0, DirectedVoteKind::Reproduce, 3, 0.0));
    assert!(def.enters_visit());
    // One live component and one consumed reference.
    assert_eq!(
        crate::creature::genome::cgp_analysis::cgp_functional_complexity(&def),
        empty + 2
    );
    let (reads, writes, stateful, _) = derive_cgp_annotations(&def, &[ring]);
    assert_eq!(reads, vec![MeshReadClass::Occupancy]);
    assert_eq!(writes, vec![MeshWriteClass::Action]);
    assert!(!stateful);
}

/// Both rotations are `(a − b) mod 8` for every `u8` pair, not only for the
/// eight directions and offsets the census and mutation draw.
#[test]
fn channel_and_offset_rotations_are_mod_eight_for_every_u8_pair() {
    let rotation =
        |a: u8, b: u8| u8::try_from((i16::from(a) - i16::from(b)).rem_euclid(8)).expect("below 8");
    for a in 0..=u8::MAX {
        for b in 0..=u8::MAX {
            let projection = RingProjection::new(0, DirectedVoteKind::Move, b, 0.0);
            assert_eq!(
                projection.channel_for(a),
                rotation(a, b),
                "d {a}, offset {b}"
            );
            assert_eq!(
                RingProjection::offset_for(a, b),
                rotation(a, b),
                "d {a}, channel {b}"
            );
        }
    }
}
