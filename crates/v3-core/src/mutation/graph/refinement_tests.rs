use super::*;
use crate::contracts::{NodeId, OrdinaryFoodTypeId};
use crate::creature::genome::cgp::{GraphEdge, GraphSource, OutputSink, OutputSinkKind};
use crate::creature::genome::vote::VoteSink;
use crate::creature::genome::BackendDef;
use proptest::prelude::*;
use rand::rngs::mock::StepRng;
use rand::{rngs::SmallRng, SeedableRng};

fn node() -> NodeGenome {
    NodeGenome {
        node_id: NodeId::new(7),
        input_refs: vec![],
        targets: vec![],
        backend_def: BackendDef::Graph(CgpGraphBackendDef::new_with_fixed_outputs()),
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
fn wire(node: &mut NodeGenome, vote: VoteSink, reference: u16, channel: u16, weight: f32) {
    let sink = graph_mut(node)
        .output_sinks
        .iter_mut()
        .find(|s| s.kind == OutputSinkKind::ActionVote(vote))
        .unwrap();
    sink.inputs.push(GraphEdge {
        source: GraphSource::InputLeaf {
            ref_idx: reference,
            sub_idx: channel,
        },
        weight,
    });
}
fn directed(kind: VoteKind, d: u8) -> VoteSink {
    match kind {
        VoteKind::Move => VoteSink::Move(d),
        VoteKind::Reproduce => VoteSink::Reproduce(d),
        VoteKind::StealEnergy => VoteSink::StealEnergy(d),
        VoteKind::Eat => unreachable!(),
    }
}
fn ring(family: WorldInputKey, kind: VoteKind, offset: u8, mask: u8) -> NodeGenome {
    let mut node = node();
    node.input_refs.push(InputReference::World(family));
    for d in 0..8 {
        if mask & (1 << d) != 0 {
            wire(
                &mut node,
                directed(kind, (d + offset) % 8),
                0,
                u16::from(d),
                f32::from(d) / 100.0,
            );
        }
    }
    node
}
fn slots(family: WorldInputKey, width: u16, field: u16, vote: VoteSink) -> NodeGenome {
    let mut node = node();
    node.input_refs.push(InputReference::World(family));
    for slot in 0..4 {
        wire(
            &mut node,
            vote,
            0,
            slot * width + field,
            f32::from(slot) / 100.0,
        );
    }
    node
}
fn all_groups(node: &NodeGenome) -> Vec<Group> {
    groups(graph(node), &node.input_refs)
}

#[test]
fn every_ring_family_kind_and_offset_has_exactly_its_semantic_group() {
    for family in [
        WorldInputKey::neighbor_food_ring(OrdinaryFoodTypeId::new(0)),
        WorldInputKey::neighbor_food_ring(OrdinaryFoodTypeId::new(1)),
        WorldInputKey::NeighborBarrierRing,
        WorldInputKey::NeighborOccupiedRing,
    ] {
        for kind in [VoteKind::Move, VoteKind::Reproduce, VoteKind::StealEnergy] {
            for offset in 0..8 {
                let node = ring(family, kind, offset, 255);
                let groups = all_groups(&node);
                assert_eq!(groups.len(), 1);
                assert_eq!(
                    groups[0].key,
                    GroupKey::Ring {
                        family,
                        kind,
                        offset: u16::from(offset)
                    }
                );
                assert_eq!(groups[0].members.len(), 8);
            }
        }
    }
}

#[test]
fn every_nearby_field_and_vote_has_its_own_slot_group() {
    for (family, width) in [
        (WorldInputKey::NearbyCreatureCore, 4),
        (WorldInputKey::NearbyCreatureVitals, 2),
        (WorldInputKey::NearbyCreatureIdentity, 3),
    ] {
        for field in 0..width {
            for vote in VoteSink::all() {
                let node = slots(family, width, field, vote);
                let groups = all_groups(&node);
                assert_eq!(groups.len(), 1);
                assert!(
                    matches!(groups[0].key, GroupKey::Slots { family: f, field: c, .. } if f == family && c == field)
                );
                assert_eq!(
                    groups[0]
                        .members
                        .iter()
                        .map(|m| m.position)
                        .collect::<Vec<_>>(),
                    [0, 1, 2, 3]
                );
            }
        }
    }
}

#[test]
fn duplicate_semantic_positions_and_nonfinite_weights_exclude_only_their_group() {
    let mut node = ring(WorldInputKey::NeighborBarrierRing, VoteKind::Move, 0, 255);
    node.input_refs.push(node.input_refs[0].clone());
    // A duplicate declaration with a unique edge is still the same semantic group.
    graph_mut(&mut node)
        .output_sinks
        .iter_mut()
        .find(|s| s.kind == OutputSinkKind::ActionVote(VoteSink::Move(0)))
        .unwrap()
        .inputs[0]
        .source = GraphSource::InputLeaf {
        ref_idx: 1,
        sub_idx: 0,
    };
    assert_eq!(all_groups(&node).len(), 1);
    wire(&mut node, VoteSink::Move(0), 0, 0, 0.0);
    assert!(all_groups(&node).is_empty());
    wire(&mut node, VoteSink::Reproduce(0), 0, 0, 0.0);
    wire(&mut node, VoteSink::Reproduce(1), 0, 1, 0.0);
    assert_eq!(all_groups(&node).len(), 1);
    graph_mut(&mut node)
        .output_sinks
        .iter_mut()
        .find(|s| s.kind == OutputSinkKind::ActionVote(VoteSink::Reproduce(0)))
        .unwrap()
        .inputs[0]
        .weight = f32::NAN;
    assert!(all_groups(&node).is_empty());
}

#[test]
fn ring_repeated_sink_occurrences_are_ambiguous_but_slot_occurrences_are_independent() {
    let mut node = ring(WorldInputKey::NeighborOccupiedRing, VoteKind::Move, 0, 255);
    let duplicate = graph(&node)
        .output_sinks
        .iter()
        .find(|s| s.kind == OutputSinkKind::ActionVote(VoteSink::Move(0)))
        .unwrap()
        .clone();
    graph_mut(&mut node).output_sinks.push(duplicate);
    assert!(all_groups(&node).is_empty());
    let mut node = slots(WorldInputKey::NearbyCreatureCore, 4, 0, VoteSink::Eat);
    let duplicate = graph(&node)
        .output_sinks
        .iter()
        .find(|s| s.kind == OutputSinkKind::ActionVote(VoteSink::Eat))
        .unwrap()
        .clone();
    graph_mut(&mut node).output_sinks.push(duplicate);
    assert_eq!(all_groups(&node).len(), 2);
    graph_mut(&mut node)
        .output_sinks
        .last_mut()
        .unwrap()
        .inputs
        .push(GraphEdge {
            source: GraphSource::InputLeaf {
                ref_idx: 0,
                sub_idx: 0,
            },
            weight: 0.0,
        });
    assert_eq!(all_groups(&node).len(), 1);
}

#[test]
fn unsupported_inputs_invalid_addresses_and_nonvote_sinks_do_not_form_groups() {
    use crate::mutation::input_ref::input_reference_universe;
    for reference in input_reference_universe(2) {
        if matches!(
            reference,
            InputReference::World(
                WorldInputKey::NeighborFoodRing { .. }
                    | WorldInputKey::NeighborBarrierRing
                    | WorldInputKey::NeighborOccupiedRing
                    | WorldInputKey::NearbyCreatureCore
                    | WorldInputKey::NearbyCreatureVitals
                    | WorldInputKey::NearbyCreatureIdentity
            )
        ) {
            continue;
        }
        let mut node = node();
        node.input_refs.push(reference);
        for d in 0..8 {
            wire(&mut node, VoteSink::Move(d), 0, u16::from(d), 0.0);
        }
        assert!(all_groups(&node).is_empty());
    }
    let mut node = ring(WorldInputKey::NeighborBarrierRing, VoteKind::Move, 0, 0);
    for (reference, channel, vote) in [
        (0, 8, VoteSink::Move(0)),
        (0, 9, VoteSink::Move(1)),
        (1, 0, VoteSink::Move(0)),
        (1, 1, VoteSink::Move(1)),
    ] {
        wire(&mut node, vote, reference, channel, 0.0);
    }
    for d in [8, 9] {
        graph_mut(&mut node).output_sinks.push(OutputSink {
            kind: OutputSinkKind::ActionVote(VoteSink::Move(d)),
            inputs: vec![GraphEdge {
                source: GraphSource::InputLeaf {
                    ref_idx: 0,
                    sub_idx: u16::from(d - 8),
                },
                weight: 0.0,
            }],
        });
    }
    assert!(all_groups(&node).is_empty());
    let mut node = slots(WorldInputKey::NearbyCreatureVitals, 2, 0, VoteSink::Eat);
    for sink in &mut graph_mut(&mut node).output_sinks {
        if !sink.inputs.is_empty() {
            sink.kind = OutputSinkKind::CustomOutput(0);
        }
    }
    assert!(all_groups(&node).is_empty());
}

proptest! {
    #[test]
    fn sparse_refinement_preserves_structure_local_differences_and_vector_budget(mask in any::<u8>(), offset in 0u8..8, seed in any::<u64>()) {
        let mut node = ring(WorldInputKey::NeighborBarrierRing, VoteKind::Move, offset, mask);
        let before = node.clone(); let count = mask.count_ones() as usize;
        prop_assert_eq!(has_group(graph(&node), &node.input_refs), count >= 2);
        let result = refine(&mut node, &mut SmallRng::seed_from_u64(seed));
        if count < 2 { prop_assert_eq!(result, Err(MutationSkipReason::NoApplicableTarget)); prop_assert_eq!(node, before); }
        else {
            prop_assert!(result.is_ok());
            let old: Vec<_> = graph(&before).edges().map(|e| e.weight).collect();
            let new: Vec<_> = graph(&node).edges().map(|e| e.weight).collect();
            prop_assert_eq!(old.len(), new.len());
            let norm = old.iter().zip(&new).map(|(a,b)| (f64::from(*b) - f64::from(*a)).powi(2)).sum::<f64>().sqrt();
            prop_assert!(norm <= 0.100001);
            for i in 1..count { prop_assert!(((new[i]-new[0]) - (old[i]-old[0])).abs() < 1e-7); }
            for (sink_before,sink_after) in graph(&before).output_sinks.iter().zip(&mut graph_mut(&mut node).output_sinks) { for (a,b) in sink_before.inputs.iter().zip(&mut sink_after.inputs) { b.weight=a.weight; } }
            prop_assert_eq!(node,before);
        }
    }
}

#[test]
fn numeric_preflight_rejects_atomically_and_rounding_is_not_an_extra_touch() {
    let mut node = ring(WorldInputKey::NeighborBarrierRing, VoteKind::Move, 0, 3);
    for edge in graph_mut(&mut node)
        .output_sinks
        .iter_mut()
        .flat_map(|s| &mut s.inputs)
    {
        edge.weight = 1_048_576.0;
    }
    let before = node.clone();
    let group = all_groups(&node).remove(0);
    assert_eq!(
        apply_step(graph_mut(&mut node), &group, 0.1),
        Err(MutationSkipReason::NumericProposalRejected)
    );
    assert_eq!(node, before);
    for scalar in [f32::NAN, f32::INFINITY] {
        assert_eq!(
            apply_step(graph_mut(&mut node), &group, scalar),
            Err(MutationSkipReason::NumericProposalRejected)
        );
        assert_eq!(node, before);
    }
    apply_step(graph_mut(&mut node), &group, 0.01).unwrap();
    assert_eq!(
        node, before,
        "selected two, changed zero at this f32 magnitude"
    );
}

#[test]
fn fields_banks_food_types_and_offsets_never_mix() {
    let mut node = node();
    node.input_refs = vec![
        InputReference::World(WorldInputKey::neighbor_food_ring(OrdinaryFoodTypeId::new(
            0,
        ))),
        InputReference::World(WorldInputKey::neighbor_food_ring(OrdinaryFoodTypeId::new(
            1,
        ))),
        InputReference::World(WorldInputKey::NeighborBarrierRing),
        InputReference::World(WorldInputKey::NearbyCreatureCore),
        InputReference::World(WorldInputKey::NearbyCreatureVitals),
    ];
    for (reference, channel, vote) in [
        (0, 0, VoteSink::Move(0)),
        (1, 1, VoteSink::Move(1)),
        (2, 2, VoteSink::Move(2)),
        (0, 3, VoteSink::Move(4)),
        (3, 0, VoteSink::Eat),
        (3, 5, VoteSink::Eat),
        (4, 2, VoteSink::Eat),
    ] {
        wire(&mut node, vote, reference, channel, 0.0);
    }
    assert!(all_groups(&node).is_empty());
    wire(&mut node, VoteSink::Move(7), 0, 7, 0.0);
    assert_eq!(all_groups(&node).len(), 1);
    wire(&mut node, VoteSink::Eat, 3, 8, 0.0);
    assert_eq!(all_groups(&node).len(), 2);
    // Out-of-range nearby channels do not contaminate the valid core group.
    wire(&mut node, VoteSink::Eat, 3, 16, f32::NAN);
    assert_eq!(all_groups(&node).len(), 2);
}

#[test]
fn every_group_has_selection_support_and_a_draw_matches_one_atomic_step() {
    let mut fixture = ring(WorldInputKey::NeighborBarrierRing, VoteKind::Move, 0, 255);
    for d in 0..8 {
        wire(
            &mut fixture,
            VoteSink::Reproduce((d + 3) % 8),
            0,
            u16::from(d),
            0.25,
        );
    }
    let groups = all_groups(&fixture);
    assert_eq!(groups.len(), 2);
    let mut selected = [0; 2];
    for seed in 0..128 {
        let mut rng = SmallRng::seed_from_u64(seed);
        let mut replay = rng.clone();
        let index = replay.gen_range(0..groups.len());
        selected[index] += 1;
        let scalar = replay.gen_range(-0.1f32..=0.1);
        let mut expected = fixture.clone();
        apply_step(graph_mut(&mut expected), &groups[index], scalar).unwrap();
        let mut actual = fixture.clone();
        refine(&mut actual, &mut rng).unwrap();
        assert_eq!(actual, expected);
        assert_eq!(
            rng.gen::<u64>(),
            replay.gen::<u64>(),
            "one group and one amplitude, no useful-effect retries"
        );
    }
    assert!(selected.iter().all(|n| *n > 0));
}

proptest! {
    #[test]
    fn any_finite_weight_proposal_is_bounded_or_rejected_unchanged(weights in prop::collection::vec(any::<u32>(), 2..9), scalar in -0.1f32..=0.1) {
        let mut node = ring(WorldInputKey::NeighborBarrierRing, VoteKind::Move, 0, 255);
        for (sink, bits) in graph_mut(&mut node).output_sinks.iter_mut().filter(|s| !s.inputs.is_empty()).zip(weights.iter().cycle()) {
            let weight = f32::from_bits(*bits); sink.inputs[0].weight = if weight.is_finite() { weight } else { 0.0 };
        }
        let before = node.clone(); let group = all_groups(&node).remove(0);
        let result = apply_step(graph_mut(&mut node), &group, scalar);
        if result.is_err() { prop_assert_eq!(result, Err(MutationSkipReason::NumericProposalRejected)); prop_assert_eq!(node, before); }
        else {
            let actual = graph(&node).edges().zip(graph(&before).edges()).map(|(a,b)| (f64::from(a.weight)-f64::from(b.weight)).powi(2)).sum::<f64>().sqrt();
            prop_assert!(actual <= 0.100001);
            prop_assert!(graph(&node).edges().all(|e| e.weight.is_finite()));
        }
    }
}

#[test]
fn normalized_step_at_both_endpoints_keeps_the_existing_local_exception() {
    for scalar in [-0.1, 0.1] {
        let mut node = slots(WorldInputKey::NearbyCreatureCore, 4, 0, VoteSink::Eat);
        let old: Vec<_> = graph(&node).edges().map(|e| e.weight).collect();
        let group = all_groups(&node).remove(0);
        apply_step(graph_mut(&mut node), &group, scalar).unwrap();
        for (edge, old) in graph(&node).edges().zip(old) {
            assert_eq!(edge.weight, old + scalar / 2.0);
        }
        // The normalization's maximum is accepted at every permitted ring size,
        // including the tiny rounding excess covered by the numeric tolerance.
        for count in 2..=8 {
            let mut node = ring(
                WorldInputKey::NeighborBarrierRing,
                VoteKind::Move,
                0,
                ((1u16 << count) - 1) as u8,
            );
            for sink in &mut graph_mut(&mut node).output_sinks {
                for edge in &mut sink.inputs {
                    edge.weight = 0.0;
                }
            }
            let group = all_groups(&node).remove(0);
            apply_step(graph_mut(&mut node), &group, scalar).unwrap();
            let norm = graph(&node)
                .edges()
                .map(|e| f64::from(e.weight).powi(2))
                .sum::<f64>()
                .sqrt();
            assert!((norm - 0.1).abs() < 1e-7, "m={count}: {norm}");
        }
    }
}

#[test]
fn native_sampler_accepts_the_exact_numeric_tolerance_boundary() {
    let mut node = ring(
        WorldInputKey::NeighborBarrierRing,
        VoteKind::Move,
        0,
        0b0011_1111,
    );
    let weight_bits = [
        0x4380_0000,
        0xb06d_84ba,
        0xb06d_84ba,
        0xb06d_84ba,
        0xb06d_84ba,
        0xa504_0000,
    ];
    for (edge, bits) in graph_mut(&mut node)
        .output_sinks
        .iter_mut()
        .flat_map(|sink| &mut sink.inputs)
        .zip(weight_bits)
    {
        edge.weight = f32::from_bits(bits);
    }
    let before: Vec<_> = graph(&node).edges().map(|edge| edge.weight).collect();
    let scalar = f32::from_bits(0x3dcc_cb75);
    let increment = scalar / (before.len() as f32).sqrt();

    refine(&mut node, &mut StepRng::new(0xffff_2600, 0)).unwrap();

    let after: Vec<_> = graph(&node).edges().map(|edge| edge.weight).collect();
    let norm = before
        .iter()
        .zip(&after)
        .map(|(old, new)| (f64::from(*new) - f64::from(*old)).powi(2))
        .sum::<f64>()
        .sqrt();
    assert_eq!(norm, 0.1 + 1e-6);
    for ((old, new), bits) in before.iter().zip(&after).zip(weight_bits) {
        assert_eq!(old.to_bits(), bits);
        assert_eq!(*new, *old + increment);
    }
}

proptest! {
    #[test]
    fn sparse_slots_follow_ranked_fields_without_rotation_or_regrowth(mask in 0u8..16, bank in 0usize..3, field in any::<u16>(), vote in 0usize..27, seed in any::<u64>()) {
        let (family,width)=[(WorldInputKey::NearbyCreatureCore,4),(WorldInputKey::NearbyCreatureVitals,2),(WorldInputKey::NearbyCreatureIdentity,3)][bank];
        let field=field % width;
        let mut node=slots(family,width,field,VoteSink::from_index(vote).unwrap());
        for sink in &mut graph_mut(&mut node).output_sinks { sink.inputs.retain(|edge| matches!(edge.source,GraphSource::InputLeaf {sub_idx,..} if mask & (1 << (sub_idx/width)) != 0)); }
        let before=node.clone(); let count=mask.count_ones() as usize;
        let groups=all_groups(&node);
        prop_assert_eq!(groups.len(),usize::from(count>=2));
        let result=refine(&mut node,&mut SmallRng::seed_from_u64(seed));
        if count<2 { prop_assert_eq!(result,Err(MutationSkipReason::NoApplicableTarget)); prop_assert_eq!(node,before); }
        else {
            prop_assert!(result.is_ok());
            prop_assert_eq!(groups[0].members.iter().map(|m|m.position).collect::<Vec<_>>(),(0..4).filter(|slot|mask&(1<<slot)!=0).collect::<Vec<_>>());
            let mut rng=SmallRng::seed_from_u64(seed); let _selection=rng.gen_range(0..1usize);
            let increment=rng.gen_range(-0.1f32..=0.1)/(count as f32).sqrt();
            for (old,new) in graph(&before).edges().zip(graph(&node).edges()) {
                prop_assert_eq!(old.source,new.source);
                prop_assert_eq!(new.weight,old.weight+increment);
            }
            prop_assert_eq!(graph(&node).edges().count(),count);
        }
    }
}
