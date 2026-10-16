//! T25.F03 edge groups on the genome: size, identity, liveness and the
//! references a group keeps.
use proptest::prelude::*;
use rand::{rngs::SmallRng, SeedableRng};

use crate::config::OrdinaryFoodTypeId;
use crate::contracts::{InputReference, WorldInputKey};
use crate::creature::founder::{v3alpha1_founder_genome, FOUNDER_GENOME_SIZE_UNITS};
use crate::creature::genome::cgp::{
    ring_offset, validate_graph, CgpGraphBackendDef, GroupId, Shape,
};
use crate::creature::genome::cgp_mesh_annotations::derive_cgp_annotations;
use crate::creature::genome::mesh_annotations::{MeshReadClass, MeshWriteClass};
use crate::creature::genome::vote::DirectedVoteKind;
use crate::creature::genome::{BackendDef, CreatureGenome, NodeGenome};
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
    /// A `Ring` group costs nine units, eight members and one record, so the
    /// replacement of a projection is size-neutral; a surviving group of `n`
    /// members costs `n + 1`.
    #[test]
    fn a_ring_group_costs_its_members_plus_one_record(
        groups in prop::collection::vec((0usize..3, 0u8..8, 0usize..8), 0..6),
    ) {
        let (mut genome, idx, ring) = founder_with_ring();
        let before = genome.genome_size();
        let mut expected = before;
        for (kind, offset, lost) in groups {
            let def = graph_mut(&mut genome.nodes[idx]);
            let id = def.add_group(Shape::Ring { ring_ref: ring }, DirectedVoteKind::ALL[kind], offset);
            prop_assert_eq!(genome.genome_size(), expected + 9);
            let def = graph_mut(&mut genome.nodes[idx]);
            for _ in 0..lost {
                let member = def.members(id).next().unwrap();
                def.remove_edge(member);
            }
            expected += 9 - lost as u32;
            prop_assert_eq!(genome.genome_size(), expected);
        }
        prop_assert!(validate_graph(&genome.nodes[idx]).is_ok());
    }

    /// Removing a reference no group addresses keeps every group of either
    /// shape (and its members) on the same `InputReference` (T25.F03,
    /// T25.F04).
    #[test]
    fn reindex_keeps_each_group_on_its_reference(
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
                    if pair { !reference.direction_pairs().is_empty() } else { reference.is_ring() }
                })
                .collect()
        };
        let (rings, pairs) = (sites(false), sites(true));
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
        for (index, kind, offset, pair) in &picks {
            let kind = DirectedVoteKind::ALL[*kind];
            if *pair {
                let ref_idx = pairs[index.index(pairs.len())];
                let layout = refs[usize::from(ref_idx)].direction_pairs()[0];
                def.add_pair_group(ref_idx, 0, layout, kind, *offset);
            } else {
                let ring_ref = rings[index.index(rings.len())];
                def.add_group(Shape::Ring { ring_ref }, kind, *offset);
            }
        }
        let expected: Vec<InputReference> = def
            .groups()
            .iter()
            .map(|group| {
                let ring_ref = group.shape().ref_idx();
                refs[usize::from(ring_ref)].clone()
            })
            .collect();
        let mut after_refs = refs.clone();
        after_refs.remove(usize::from(removed));
        def.reindex_input_refs_after_removal(removed);
        prop_assert_eq!(def.groups().len(), expected.len());
        for (group, reference) in def.groups().iter().zip(&expected) {
            let ring_ref = group.shape().ref_idx();
            prop_assert_eq!(&after_refs[usize::from(ring_ref)], reference);
        }
        let node = NodeGenome {
            node_id: crate::contracts::NodeId(0),
            input_refs: after_refs,
            backend_def: BackendDef::Graph(def),
            targets: Vec::new(),
        };
        prop_assert!(validate_graph(&node).is_ok());
    }

    /// `Prune` never deletes a group's ring or pair reference, and every
    /// group reads the same reference after it.
    #[test]
    fn prune_keeps_each_group_on_its_reference(
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
        graph_mut(node).add_group(Shape::Ring { ring_ref: ring }, DirectedVoteKind::Move, offset);
        let area = InputReference::World(WorldInputKey::AreaOccupancySummary);
        let summary = node.input_refs.len() as u16;
        node.input_refs.push(area.clone());
        let layout = area.direction_pairs()[1];
        graph_mut(node).add_pair_group(summary, 1, layout, DirectedVoteKind::StealEnergy, offset);
        prop_assert!(!prunable_indices(node).contains(&usize::from(ring)));
        prop_assert!(!prunable_indices(node).contains(&usize::from(summary)));
        let mut rng = SmallRng::seed_from_u64(seed);
        while InputRefOperator::Prune.applies_to(&genome.nodes[idx], 1) {
            InputRefMutator::apply_to_node(&mut genome, InputRefOperator::Prune, idx, &mut rng, 1)
                .unwrap();
        }
        let node = &mut genome.nodes[idx];
        let ring_ref = graph_mut(node).groups()[0].shape().ref_idx();
        prop_assert_eq!(
            &node.input_refs[usize::from(ring_ref)],
            &InputReference::World(WorldInputKey::NeighborBarrierRing)
        );
        let pair_ref = graph_mut(node).groups()[1].shape().ref_idx();
        prop_assert_eq!(&node.input_refs[usize::from(pair_ref)], &area);
        prop_assert!(validate_graph(node).is_ok());
    }
}

#[test]
fn genome_identity_distinguishes_groups_and_a_founder_is_unchanged() {
    let founder = v3alpha1_founder_genome();
    assert_eq!(founder.genome_size(), FOUNDER_GENOME_SIZE_UNITS);
    let (ringed, idx, ring) = founder_with_ring();
    let mut grouped = ringed.clone();
    let id = graph_mut(&mut grouped.nodes[idx]).add_group(
        Shape::Ring { ring_ref: ring },
        DirectedVoteKind::Move,
        0,
    );
    assert_ne!(genome_identity(&grouped), genome_identity(&ringed));
    let mut rescaled = grouped.clone();
    graph_mut(&mut rescaled.nodes[idx])
        .group_mut(id)
        .set_scale(0.5);
    assert_ne!(genome_identity(&rescaled), genome_identity(&grouped));
}

/// A member-only graph is a live component: its group record counts one, its
/// members wire eight sinks, the ring counts through the members, and its
/// annotations read the ring and write the action channel.
#[test]
fn a_group_is_a_live_component_reading_its_ring_and_writing_action() {
    let ring = InputReference::World(WorldInputKey::NeighborOccupiedRing);
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    assert!(!def.enters_visit());
    let empty = crate::creature::genome::cgp_analysis::cgp_functional_complexity(&def);
    let id = def.add_group(Shape::Ring { ring_ref: 0 }, DirectedVoteKind::Reproduce, 3);
    assert_eq!(id, GroupId::new(0));
    assert!(def.enters_visit());
    // One record, eight wired sinks and one consumed reference.
    assert_eq!(
        crate::creature::genome::cgp_analysis::cgp_functional_complexity(&def),
        empty + 1 + 8 + 1
    );
    let (reads, writes, stateful, _) = derive_cgp_annotations(&def, &[ring]);
    assert_eq!(reads, vec![MeshReadClass::Occupancy]);
    assert_eq!(writes, vec![MeshWriteClass::Action]);
    assert!(!stateful);
}

/// The ring offset is `(d − channel) mod 8` for every `u8` pair, not only
/// for the eight directions and channels recruitment and the census draw.
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
