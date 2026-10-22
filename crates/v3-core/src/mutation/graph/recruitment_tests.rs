use super::*;
use crate::config::NeutralInputRecruitment as Arm;
use crate::contracts::NodeId;
use crate::creature::genome::cgp::test_support::{edit_sinks, push_sink};
use crate::creature::genome::cgp::{
    EdgeSurface, GraphEdge, GraphSource, OutputSink, OutputSinkKind,
};
use crate::creature::genome::vote::{VoteSink, VOTE_SINK_COUNT};
use crate::creature::genome::NodeGenome;
use crate::mutation::input_ref::input_reference_universe;
use proptest::prelude::*;
use rand::{rngs::SmallRng, SeedableRng};

fn node() -> NodeGenome {
    NodeGenome {
        node_id: NodeId::new(7),
        input_refs: vec![],
        backend_def: BackendDef::Graph(CgpGraphBackendDef::new_with_fixed_outputs()),
        targets: vec![],
    }
}

fn graph(node: &NodeGenome) -> &CgpGraphBackendDef {
    let BackendDef::Graph(graph) = &node.backend_def else {
        panic!("graph fixture")
    };
    graph
}

fn graph_mut(node: &mut NodeGenome) -> &mut CgpGraphBackendDef {
    let BackendDef::Graph(graph) = &mut node.backend_def else {
        panic!("graph fixture")
    };
    graph
}

#[test]
fn recruitment_off_rejects_an_eligible_vote_sink_without_editing() {
    let mut fixture = node();
    let before = fixture.clone();

    let result = recruit(&mut fixture, Arm::Off, 2, &mut SmallRng::seed_from_u64(0));

    assert_eq!(result, Err(MutationSkipReason::NoApplicableTarget));
    assert_eq!(fixture, before);
}

#[test]
fn every_legal_reference_and_memory_bank_has_canonical_access() {
    for reference in input_reference_universe(2) {
        let width = if reference == InputReference::ActionQueue {
            65536u32
        } else {
            u32::from(crate::mutation::compound::sub_value_count(&reference))
        };
        let family = SourceFamily::Input(reference.clone());
        let fixture = node();
        let admission = Admission::new(&fixture.input_refs, graph(&fixture));
        for channel in 0..width {
            let plan = admission
                .plan(&family, Arm::SingleChannel, u16::try_from(channel).unwrap())
                .unwrap();
            assert_eq!(plan.declarations, std::slice::from_ref(&reference));
            assert_eq!(
                plan.sources,
                [GraphSource::InputLeaf {
                    ref_idx: 0,
                    sub_idx: channel as u16
                }]
            );
        }
        if reference != InputReference::ActionQueue
            && !matches!(reference, InputReference::UpstreamSlot(_))
        {
            let plan = admission.plan(&family, Arm::WholeFamily, 0).unwrap();
            // A slot family resolves one slot at append (T25.F05).
            let whole = if reference.slot_pairs() { 0 } else { width };
            assert_eq!(plan.sources.len(), whole as usize);
            assert_eq!(
                plan.pairs.is_some(),
                !reference.direction_pairs().is_empty()
            );
        }
    }
    let fixture = node();
    let admission = Admission::new(&fixture.input_refs, graph(&fixture));
    let upstream = admission
        .plan(&SourceFamily::Upstream, Arm::WholeFamily, 0)
        .unwrap();
    assert_eq!(
        upstream.declarations,
        (0..24)
            .map(InputReference::UpstreamSlot)
            .collect::<Vec<_>>()
    );
    assert_eq!(upstream.sources.len(), 24);
    for previous in [false, true] {
        let plan = admission
            .plan(&SourceFamily::Memory(previous), Arm::WholeFamily, 0)
            .unwrap();
        assert!(plan.declarations.is_empty());
        assert_eq!(
            plan.sources,
            (0..16)
                .map(|slot| GraphSource::SharedMemory { slot, previous })
                .collect::<Vec<_>>()
        );
    }
    assert!(!admission
        .families(Arm::WholeFamily, 2)
        .contains(&SourceFamily::Input(InputReference::ActionQueue)));
}

#[test]
fn reference_capacity_and_dangling_incumbents_are_preflighted() {
    let mut fixture = node();
    fixture.input_refs = vec![InputReference::ActionVotes; 65536];
    let admission = Admission::new(&fixture.input_refs, graph(&fixture));
    assert!(admission
        .plan(
            &SourceFamily::Input(InputReference::ActionVotes),
            Arm::WholeFamily,
            0
        )
        .is_some());
    assert!(admission
        .plan(
            &SourceFamily::Input(InputReference::PreviousPassVotes),
            Arm::SingleChannel,
            0
        )
        .is_none());
    assert!(admission
        .plan(&SourceFamily::Memory(true), Arm::WholeFamily, 0)
        .is_some());
    fixture.input_refs = vec![InputReference::ActionVotes; 65513];
    let before = fixture.clone();
    let admission = Admission::new(&fixture.input_refs, graph(&fixture));
    assert!(admission
        .plan(&SourceFamily::Upstream, Arm::WholeFamily, 0)
        .is_none());
    assert_eq!(fixture, before);
    fixture.input_refs.clear();
    graph_mut(&mut fixture).push_edge(
        EdgeSurface::SinkInput(0),
        GraphEdge::new(
            GraphSource::InputLeaf {
                ref_idx: 1,
                sub_idx: 0,
            },
            3.0,
        ),
    );
    let admission = Admission::new(&fixture.input_refs, graph(&fixture));
    assert!(admission
        .plan(
            &SourceFamily::Input(InputReference::ActionVotes),
            Arm::WholeFamily,
            0
        )
        .is_some());
    assert!(admission
        .plan(&SourceFamily::Upstream, Arm::WholeFamily, 0)
        .is_none());
    fixture.input_refs.push(InputReference::ActionVotes);
    let admission = Admission::new(&fixture.input_refs, graph(&fixture));
    assert!(admission
        .plan(
            &SourceFamily::Input(InputReference::PreviousPassVotes),
            Arm::SingleChannel,
            0
        )
        .is_none());
    assert!(admission
        .plan(
            &SourceFamily::Input(InputReference::ActionVotes),
            Arm::WholeFamily,
            0
        )
        .is_some());
}

#[test]
fn every_valid_vote_occurrence_and_only_valid_votes_are_destinations() {
    let mut fixture = node();
    let graph = graph_mut(&mut fixture);
    push_sink(
        graph,
        OutputSink::new(
            OutputSinkKind::ActionVote(VoteSink::Directed(DirectedVoteKind::Move, 8)),
            vec![],
        ),
    );
    push_sink(
        graph,
        OutputSink::new(OutputSinkKind::ActionVote(VoteSink::Eat), vec![]),
    );
    let sites = destinations(graph);
    assert_eq!(sites.len(), VOTE_SINK_COUNT + 1);
    for vote in VoteSink::all() {
        assert!(sites
            .iter()
            .any(|&i| graph.output_sinks()[i].kind == OutputSinkKind::ActionVote(vote)));
    }
    assert!(!sites.contains(&(graph.output_sinks().len() - 2)));
    assert!(sites.contains(&(graph.output_sinks().len() - 1)));
}

proptest! {
    #[test]
    fn recruitment_preserves_incumbents_and_obeys_bounds(seed in any::<u64>(), whole in any::<bool>(), declarations in 0usize..8, weight in -100f32..100f32) {
        let mut fixture = node();
        fixture.input_refs = vec![InputReference::ActionVotes; declarations];
        edit_sinks(graph_mut(&mut fixture), |sinks| sinks[64].inputs = vec![GraphEdge::new(GraphSource::SharedMemory { slot: 3, previous: true }, weight); 2]);
        let before = fixture.clone();
        let arm = if whole { Arm::WholeFamily } else { Arm::SingleChannel };
        recruit(&mut fixture, arm, 2, &mut SmallRng::seed_from_u64(seed)).unwrap();
        prop_assert_eq!(&fixture.input_refs[..declarations], &before.input_refs);
        prop_assert!(fixture.input_refs.len() - declarations <= 24);
        let before_graph = graph(&before);
        let after_graph = graph(&fixture);
        let mut changed = 0;
        let mut added = 0;
        for (old, new) in before_graph.output_sinks().iter().zip(after_graph.output_sinks()) {
            prop_assert_eq!(old.kind, new.kind);
            prop_assert_eq!(&new.inputs[..old.inputs.len()], &old.inputs[..]);
            if new.inputs.len() != old.inputs.len() { changed += 1; }
            added += new.inputs.len() - old.inputs.len();
            prop_assert!(new.inputs[old.inputs.len()..].iter().all(|edge| edge.own_weight() == 0.0));
        }
        // A ring family landing on a directed vote sink becomes one silent
        // `Ring` group of eight born-0 members in place of its edges
        // (T25.F03); a pair family under WholeFamily, one silent
        // `DirectionPair` gene of twelve born-0 direction members over the
        // kind's eight direction sinks plus its fact members on the kind's
        // drive (T25.F10), every edge a member: 15 for an area family
        // (T25.F04), 20 for a creature slot (T25.F05, ten fields since
        // T27.F01).
        let grouped = after_graph.groups().len() - before_graph.groups().len();
        prop_assert_eq!(&after_graph.groups()[..before_graph.groups().len()], before_graph.groups());
        prop_assert!(after_graph.groups().iter().all(|group| group.scale() == 0.0));
        if grouped == 0 {
            prop_assert_eq!(changed, 1);
            prop_assert!((1..=VOTE_SINK_COUNT).contains(&added));
            if !whole { prop_assert_eq!(added, 1); }
            prop_assert!(after_graph.edges().all(|edge| edge.membership().is_none()));
        } else {
            prop_assert_eq!(grouped, 1);
            let id = crate::creature::genome::cgp::GroupId::new(0);
            match after_graph.groups()[0].shape() {
                crate::creature::genome::cgp::Shape::Ring { .. } => {
                    prop_assert_eq!((changed, added), (8, 8));
                    prop_assert_eq!(after_graph.members(id).count(), 8);
                }
                crate::creature::genome::cgp::Shape::DirectionPair { .. } => {
                    prop_assert!(whole);
                    prop_assert_eq!(changed, 9);
                    prop_assert!(added == 15 || added == 20, "{}", added);
                    prop_assert!((1..=30).contains(&added));
                    prop_assert_eq!(after_graph.members(id).count(), added);
                }
            }
        }
        prop_assert_eq!(after_graph.compute_nodes(), before_graph.compute_nodes());
        prop_assert_eq!(&fixture.targets, &before.targets);
        prop_assert_eq!(fixture.node_id, before.node_id);
    }

    #[test]
    fn full_or_dangling_tables_still_apply_without_retry(seed in any::<u64>(), whole in any::<bool>(), full in any::<bool>()) {
        let mut fixture = node();
        if full { fixture.input_refs = vec![InputReference::ActionVotes; 65537]; }
        else { graph_mut(&mut fixture).push_edge(EdgeSurface::SinkInput(0), GraphEdge::new(GraphSource::InputLeaf { ref_idx: 0, sub_idx: 0 }, 1.0)); }
        let before = fixture.input_refs.clone();
        recruit(&mut fixture, if whole { Arm::WholeFamily } else { Arm::SingleChannel }, 2, &mut SmallRng::seed_from_u64(seed)).unwrap();
        prop_assert_eq!(&fixture.input_refs, &before);
    }
}

#[test]
fn admissible_sampling_pool_contains_all_declared_families_and_channels() {
    let fixture = node();
    let admission = Admission::new(&fixture.input_refs, graph(&fixture));
    let single = admission.families(Arm::SingleChannel, 2);
    // The three own-colour keys join both universes (T27.F01).
    assert_eq!(single.len(), 51);
    let whole = admission.families(Arm::WholeFamily, 2);
    assert_eq!(whole.len(), 27);
    for reference in input_reference_universe(2) {
        assert!(single.contains(&SourceFamily::Input(reference.clone())));
        if !matches!(
            reference,
            InputReference::ActionQueue | InputReference::UpstreamSlot(_)
        ) {
            assert!(whole.contains(&SourceFamily::Input(reference)));
        }
    }
    assert!(whole.contains(&SourceFamily::Upstream));
    for previous in [false, true] {
        assert!(single.contains(&SourceFamily::Memory(previous)));
        assert!(whole.contains(&SourceFamily::Memory(previous)));
    }
}

#[test]
fn last_representable_index_is_usable_and_unrepresentable_matches_are_not() {
    let mut fixture = node();
    fixture.input_refs = vec![InputReference::ActionVotes; 65535];
    recruit_source(
        &mut fixture,
        &SourceFamily::Input(InputReference::PreviousPassVotes),
        Arm::SingleChannel,
        26,
        VoteSink::Eat,
    )
    .unwrap();
    assert_eq!(fixture.input_refs.len(), 65536);
    assert_eq!(
        graph(&fixture).edges().next().unwrap().source(),
        GraphSource::InputLeaf {
            ref_idx: u16::MAX,
            sub_idx: 26
        }
    );
    fixture.input_refs.push(InputReference::CommitCounts);
    let before = fixture.clone();
    assert!(recruit_source(
        &mut fixture,
        &SourceFamily::Input(InputReference::CommitCounts),
        Arm::SingleChannel,
        0,
        VoteSink::Eat
    )
    .is_err());
    assert_eq!(fixture, before);
    fixture.input_refs = vec![InputReference::ActionVotes; 65512];
    edit_sinks(graph_mut(&mut fixture), |sinks| {
        sinks.iter_mut().for_each(|sink| sink.inputs.clear());
    });
    recruit_source(
        &mut fixture,
        &SourceFamily::Upstream,
        Arm::WholeFamily,
        0,
        VoteSink::Eat,
    )
    .unwrap();
    assert_eq!(fixture.input_refs.len(), 65536);
    assert_eq!(
        graph(&fixture).edges().last().unwrap().source(),
        GraphSource::InputLeaf {
            ref_idx: u16::MAX,
            sub_idx: 0
        }
    );
}

#[test]
fn rejected_explicit_bundle_leaves_every_incumbent_unchanged() {
    let mut fixture = node();
    graph_mut(&mut fixture).push_edge(
        EdgeSurface::SinkInput(0),
        GraphEdge::new(
            GraphSource::InputLeaf {
                ref_idx: 12,
                sub_idx: 4,
            },
            1.5,
        ),
    );
    let before = fixture.clone();
    assert!(recruit_source(
        &mut fixture,
        &SourceFamily::Upstream,
        Arm::WholeFamily,
        0,
        VoteSink::Eat
    )
    .is_err());
    assert_eq!(fixture, before);
    assert!(recruit_source(
        &mut fixture,
        &SourceFamily::Input(InputReference::ActionQueue),
        Arm::WholeFamily,
        0,
        VoteSink::Eat
    )
    .is_err());
    assert_eq!(fixture, before);
}

proptest! {
    #[test]
    fn standalone_default_operator_sampler_keeps_the_original_weights_and_rng(seed in any::<u64>()) {
        use super::super::GraphOperator;
        let mut rng = SmallRng::seed_from_u64(seed);
        let mut replay = rng.clone();
        let old: Vec<_> = GraphOperator::ALL.into_iter().filter(|&op| op != GraphOperator::RecruitNeutralInput).collect();
        let total: u16 = old.iter().map(|op| u16::from(op.weight())).sum();
        let mut roll = replay.gen_range(0..total);
        let expected = old.into_iter().find(|op| {
            let weight = u16::from(op.weight());
            if roll < weight { true } else { roll -= weight; false }
        }).unwrap();
        prop_assert_eq!(GraphOperator::random(&mut rng), expected);
        prop_assert_eq!(rng.gen::<u64>(), replay.gen::<u64>());
    }
}

#[test]
fn a_ring_on_a_directed_sink_becomes_one_silent_group() {
    use crate::creature::genome::cgp::{GroupId, Shape};
    use crate::creature::genome::vote::DirectedVoteKind;
    let ring = InputReference::World(crate::contracts::WorldInputKey::NeighborBarrierRing);
    let family = SourceFamily::Input(ring.clone());
    // SingleChannel: channel 5 into Move(3) is offset (3 - 5) mod 8 = 6.
    let mut fixture = node();
    recruit_source(
        &mut fixture,
        &family,
        Arm::SingleChannel,
        5,
        VoteSink::Directed(DirectedVoteKind::Move, 3),
    )
    .unwrap();
    assert_eq!(fixture.input_refs, vec![ring.clone()]);
    let def = graph(&fixture);
    assert_eq!(def.groups().len(), 1);
    let group = def.groups()[0];
    assert_eq!(
        (group.shape(), group.kind(), group.offset(), group.scale()),
        (Shape::Ring { ring_ref: 0 }, DirectedVoteKind::Move, 6, 0.0)
    );
    assert_eq!(def.edges().count(), 8);
    assert!(def.edges().all(|edge| edge.own_weight() == 0.0));
    assert_eq!(def.members(GroupId::new(0)).count(), 8);
    // WholeFamily reads channel 0, and an existing declaration is reused.
    recruit_source(
        &mut fixture,
        &family,
        Arm::WholeFamily,
        0,
        VoteSink::Directed(DirectedVoteKind::StealEnergy, 7),
    )
    .unwrap();
    assert_eq!(fixture.input_refs, vec![ring.clone()]);
    let group = graph(&fixture).groups()[1];
    assert_eq!(
        (group.kind(), group.offset()),
        (DirectedVoteKind::StealEnergy, 7)
    );
    // An undirected destination keeps today's edges.
    recruit_source(&mut fixture, &family, Arm::SingleChannel, 2, VoteSink::Eat).unwrap();
    assert_eq!(graph(&fixture).groups().len(), 2);
    assert_eq!(graph(&fixture).edges().count(), 17);
    // A non-ring family on a directed sink keeps today's edge.
    let mut other = node();
    recruit_source(
        &mut other,
        &SourceFamily::Input(InputReference::ActionVotes),
        Arm::SingleChannel,
        1,
        VoteSink::Directed(DirectedVoteKind::Move, 0),
    )
    .unwrap();
    assert!(graph(&other).groups().is_empty());
    assert_eq!(graph(&other).edges().count(), 1);
}

#[test]
fn a_dangling_member_reference_bounds_admission_like_a_dangling_leaf() {
    use crate::creature::genome::cgp::Shape;
    use crate::creature::genome::vote::DirectedVoteKind;
    let mut fixture = node();
    graph_mut(&mut fixture).add_group(Shape::Ring { ring_ref: 0 }, DirectedVoteKind::Move, 0);
    let admission = Admission::new(&fixture.input_refs, graph(&fixture));
    assert!(admission
        .plan(
            &SourceFamily::Input(InputReference::ActionVotes),
            Arm::SingleChannel,
            0
        )
        .is_none());
    assert!(admission
        .plan(&SourceFamily::Memory(false), Arm::SingleChannel, 0)
        .is_some());
}

/// Ring families leave recruitment's universe exactly when the graph cannot
/// host a group, so a drawn ring plan always completes; every other family
/// is unchanged.
#[test]
fn ring_families_leave_the_universe_when_no_group_can_be_hosted() {
    for arm in [Arm::SingleChannel, Arm::WholeFamily] {
        let full = node();
        let hosting = Admission::new(&full.input_refs, graph(&full)).families(arm, 2);
        assert!(graph(&full).can_host_group());
        let mut partial = node();
        edit_sinks(graph_mut(&mut partial), |sinks| {
            sinks.retain(|sink| {
                sink.kind
                    != OutputSinkKind::ActionVote(VoteSink::Directed(
                        DirectedVoteKind::Reproduce,
                        4,
                    ))
            });
        });
        assert!(!graph(&partial).can_host_group());
        let without = Admission::new(&partial.input_refs, graph(&partial)).families(arm, 2);
        let is_ring = |family: &SourceFamily| matches!(family, SourceFamily::Input(reference) if reference.is_ring());
        // Under WholeFamily the pair case's preflight drops pair-declaring
        // families too (T25.F04, `pair_families_leave_the_whole_family_universe_without_a_host_or_a_drive`).
        let pair_plan = |family: &SourceFamily| {
            arm == Arm::WholeFamily
                && matches!(family, SourceFamily::Input(reference) if !reference.direction_pairs().is_empty())
        };
        assert!(hosting.iter().any(is_ring));
        assert!(!without.iter().any(is_ring));
        let others: Vec<_> = hosting
            .into_iter()
            .filter(|family| !is_ring(family) && !pair_plan(family))
            .collect();
        assert_eq!(without, others);
    }
}

// ── The pair case (T25.F04) ─────────────────────────────────────────────────

fn area_food() -> InputReference {
    InputReference::World(crate::contracts::WorldInputKey::area_food_summary(
        crate::contracts::OrdinaryFoodTypeId::new(0),
    ))
}

fn nearby() -> InputReference {
    InputReference::World(crate::contracts::WorldInputKey::NearbyCreature)
}

/// The edges of `kind`'s drive sink.
fn drive_edges(
    def: &CgpGraphBackendDef,
    kind: crate::creature::genome::vote::DirectedVoteKind,
) -> Vec<GraphEdge> {
    def.sink(OutputSinkKind::ActionVote(kind.drive()))
        .unwrap()
        .inputs
        .to_vec()
}

#[test]
fn a_pair_family_on_a_directed_sink_becomes_one_gene_with_its_facts_on_the_drive() {
    use crate::creature::genome::cgp::{GroupId, Membership, Scope, Shape};
    use crate::creature::genome::vote::DirectedVoteKind;
    for (reference, pair, facts) in [
        (area_food(), 1u8, vec![0u16, 5, 6]),
        (nearby(), 2, vec![20, 23, 24, 25, 26, 27, 28, 29]),
    ] {
        let mut fixture = node();
        recruit_source_with_pair(
            &mut fixture,
            &SourceFamily::Input(reference.clone()),
            Arm::WholeFamily,
            VoteSink::Directed(DirectedVoteKind::StealEnergy, 3),
            pair,
        )
        .unwrap();
        assert_eq!(fixture.input_refs, vec![reference.clone()]);
        let def = graph(&fixture);
        assert_eq!(def.groups().len(), 1);
        let group = def.groups()[0];
        assert_eq!(
            (group.shape(), group.kind(), group.offset(), group.scale()),
            (
                Shape::DirectionPair { ref_idx: 0, pair },
                DirectedVoteKind::StealEnergy,
                3,
                0.0
            )
        );
        assert_eq!(def.members(GroupId::new(0)).count(), 12 + facts.len());
        let drive: Vec<_> = drive_edges(def, DirectedVoteKind::StealEnergy);
        let want: Vec<_> = facts
            .iter()
            .map(|&sub_idx| {
                (
                    GraphSource::InputLeaf {
                        ref_idx: 0,
                        sub_idx,
                    },
                    Some(Membership {
                        group: GroupId::new(0),
                        scope: Scope::Edge,
                    }),
                )
            })
            .collect();
        let got: Vec<_> = drive
            .iter()
            .map(|edge| (edge.source(), edge.membership()))
            .collect();
        assert_eq!(
            got, want,
            "the gene's fact members in order, born 0 (T25.F10)"
        );
        assert!(drive.iter().all(|edge| edge.plasticity().is_none()));
        assert_eq!(def.edges().count(), 12 + facts.len());
        assert!(def.edges().all(|edge| edge.own_weight() == 0.0));
        let node_genome = fixture.clone();
        assert_eq!(
            crate::creature::genome::cgp::validate_graph(&node_genome),
            Ok(())
        );
    }
}

#[test]
fn single_channel_and_undirected_destinations_keep_their_plans_for_a_pair_family() {
    // SingleChannel on a directed sink: one ordinary edge.
    let mut single = node();
    recruit_source(
        &mut single,
        &SourceFamily::Input(area_food()),
        Arm::SingleChannel,
        4,
        VoteSink::Directed(DirectedVoteKind::Move, 2),
    )
    .unwrap();
    assert!(graph(&single).groups().is_empty());
    assert_eq!(graph(&single).edges().count(), 1);
    // WholeFamily on an undirected destination: the whole-family edges.
    let mut whole = node();
    recruit_source(
        &mut whole,
        &SourceFamily::Input(area_food()),
        Arm::WholeFamily,
        0,
        VoteSink::Eat,
    )
    .unwrap();
    assert!(graph(&whole).groups().is_empty());
    assert_eq!(graph(&whole).edges().count(), 7);
}

/// A slot family under `WholeFamily` on an undirected destination wires the
/// drawn slot's ten channels ascending, born 0, never the family's 40
/// (T25.F05, T27.F01); under `SingleChannel` it keeps one edge.
#[test]
fn a_slot_family_on_an_undirected_destination_wires_one_slot() {
    for slot in 0..4u8 {
        let mut whole = node();
        recruit_source_with_pair(
            &mut whole,
            &SourceFamily::Input(nearby()),
            Arm::WholeFamily,
            VoteSink::Eat,
            slot,
        )
        .unwrap();
        assert_eq!(whole.input_refs, vec![nearby()]);
        let def = graph(&whole);
        assert!(def.groups().is_empty());
        let eat: Vec<_> = def
            .sink(OutputSinkKind::ActionVote(VoteSink::Eat))
            .unwrap()
            .inputs
            .to_vec();
        let base = 10 * u16::from(slot);
        let want: Vec<_> = (base..base + 10)
            .map(|sub_idx| {
                GraphEdge::new(
                    GraphSource::InputLeaf {
                        ref_idx: 0,
                        sub_idx,
                    },
                    0.0,
                )
            })
            .collect();
        assert_eq!(eat, want, "slot {slot}");
        assert_eq!(def.edges().count(), 10);
        assert_eq!(crate::creature::genome::cgp::validate_graph(&whole), Ok(()));
    }
    let mut single = node();
    recruit_source(
        &mut single,
        &SourceFamily::Input(nearby()),
        Arm::SingleChannel,
        35,
        VoteSink::Eat,
    )
    .unwrap();
    assert_eq!(graph(&single).edges().count(), 1);
}

/// `recruit`'s draws under `WholeFamily`: the family, the destination, and
/// for a pair-declaring family landing on a directed sink, or a slot family
/// landing anywhere (T25.F05), one pair `gen_range(0..pairs)`; nothing after.
#[test]
fn the_pair_case_draws_family_then_destination_then_pair() {
    use crate::creature::genome::vote::DirectedVoteKind;
    let mut paired = 0;
    let mut undirected_slots = 0;
    for seed in 0..512 {
        let mut fixture = node();
        let mut rng = SmallRng::seed_from_u64(seed);
        recruit(&mut fixture, Arm::WholeFamily, 2, &mut rng).unwrap();

        let mut replay = SmallRng::seed_from_u64(seed);
        let base = node();
        let admission = Admission::new(&base.input_refs, graph(&base));
        let families = admission.families(Arm::WholeFamily, 2);
        let family = families.choose(&mut replay).unwrap().clone();
        let sites = destinations(graph(&base));
        let destination = *sites.choose(&mut replay).unwrap();
        let OutputSinkKind::ActionVote(vote) = graph(&base).output_sinks()[destination].kind else {
            unreachable!("destinations are vote sinks")
        };
        let (pairs, slots) = match &family {
            SourceFamily::Input(reference) => {
                (reference.direction_pairs().len(), reference.slot_pairs())
            }
            _ => (0, false),
        };
        let directed = DirectedVoteKind::of_sink(vote).is_some();
        let mut expected = base.clone();
        if pairs > 0 && (directed || slots) {
            let pair = u8::try_from(replay.gen_range(0..pairs)).unwrap();
            recruit_source_with_pair(&mut expected, &family, Arm::WholeFamily, vote, pair).unwrap();
            paired += 1;
            undirected_slots += usize::from(!directed);
        } else {
            recruit_source(&mut expected, &family, Arm::WholeFamily, 0, vote).unwrap();
        }
        assert_eq!(fixture, expected, "seed {seed}");
        assert_eq!(rng.gen::<u64>(), replay.gen::<u64>(), "seed {seed}");
    }
    assert!(paired > 0, "some seed takes the pair case");
    assert!(
        undirected_slots > 0,
        "some seed draws a slot on an undirected destination"
    );
}

/// The merged creature family is three entries of recruitment's family
/// universe under both arms, one per family it replaced (T25.F05), while
/// `input_reference_universe` lists it once; every other entry appears once.
#[test]
fn the_merged_family_is_three_entries_of_the_recruitment_universe() {
    let nearby_family = SourceFamily::Input(nearby());
    assert_eq!(
        input_reference_universe(2)
            .iter()
            .filter(|reference| **reference == nearby())
            .count(),
        1
    );
    let full = node();
    for arm in [Arm::SingleChannel, Arm::WholeFamily] {
        let families = Admission::new(&full.input_refs, graph(&full)).families(arm, 2);
        assert_eq!(
            families.iter().filter(|f| **f == nearby_family).count(),
            3,
            "{arm:?}"
        );
        for family in families.iter().filter(|f| **f != nearby_family) {
            assert_eq!(
                families.iter().filter(|f| *f == family).count(),
                1,
                "{arm:?}: {family:?} once"
            );
        }
    }
}

/// The preflight: under `WholeFamily`, pair-declaring families leave the
/// universe when the graph cannot host a group or any kind's drive sink is
/// absent; under `SingleChannel` they stay. All three entries of the merged
/// family leave together, on a partial sink catalog and on an exhausted
/// group table alike (T25.F05).
#[test]
fn pair_families_leave_the_whole_family_universe_without_a_host_or_a_drive() {
    use crate::creature::genome::vote::DirectedVoteKind;
    let declares = |family: &SourceFamily| matches!(family, SourceFamily::Input(reference) if !reference.direction_pairs().is_empty());
    let full = node();
    let hosting = Admission::new(&full.input_refs, graph(&full)).families(Arm::WholeFamily, 2);
    assert!(hosting.iter().any(declares));
    let no_direction = {
        let mut partial = node();
        edit_sinks(graph_mut(&mut partial), |sinks| {
            sinks.retain(|sink| {
                sink.kind
                    != OutputSinkKind::ActionVote(VoteSink::Directed(DirectedVoteKind::Move, 6))
            });
        });
        partial
    };
    for kind in DirectedVoteKind::ALL {
        let mut no_drive = node();
        edit_sinks(graph_mut(&mut no_drive), |sinks| {
            sinks.retain(|sink| sink.kind != OutputSinkKind::ActionVote(kind.drive()));
        });
        assert!(graph(&no_drive).can_host_group());
        for fixture in [&no_drive, &no_direction] {
            let whole =
                Admission::new(&fixture.input_refs, graph(fixture)).families(Arm::WholeFamily, 2);
            assert!(!whole.iter().any(declares), "{kind:?}");
            let others: Vec<_> = hosting
                .iter()
                .filter(|family| !declares(family))
                .filter(|family| {
                    graph(fixture).can_host_group()
                        || !matches!(family, SourceFamily::Input(reference) if reference.is_ring())
                })
                .cloned()
                .collect();
            assert_eq!(whole, others);
            let single =
                Admission::new(&fixture.input_refs, graph(fixture)).families(Arm::SingleChannel, 2);
            assert!(single.iter().any(declares));
        }
    }
    let mut exhausted = node();
    graph_mut(&mut exhausted).exhaust_group_capacity();
    assert!(!graph(&exhausted).can_host_group());
    let nearby_family = SourceFamily::Input(nearby());
    for fixture in [&no_direction, &exhausted] {
        let whole =
            Admission::new(&fixture.input_refs, graph(fixture)).families(Arm::WholeFamily, 2);
        assert!(!whole.contains(&nearby_family), "no merged entry remains");
        assert!(!whole.iter().any(declares));
        let single =
            Admission::new(&fixture.input_refs, graph(fixture)).families(Arm::SingleChannel, 2);
        assert_eq!(single.iter().filter(|f| **f == nearby_family).count(), 3);
    }
}

/// F03's ring admission is untouched by the pair case: ring families stay in
/// the universe whenever the graph can host a group, a missing drive sink
/// included.
#[test]
fn ring_admission_is_unchanged_by_the_pair_case() {
    use crate::creature::genome::vote::DirectedVoteKind;
    let is_ring = |family: &SourceFamily| matches!(family, SourceFamily::Input(reference) if reference.is_ring());
    for arm in [Arm::SingleChannel, Arm::WholeFamily] {
        let mut no_drive = node();
        edit_sinks(graph_mut(&mut no_drive), |sinks| {
            sinks.retain(|sink| {
                sink.kind != OutputSinkKind::ActionVote(DirectedVoteKind::Reproduce.drive())
            });
        });
        let families = Admission::new(&no_drive.input_refs, graph(&no_drive)).families(arm, 2);
        assert_eq!(families.iter().filter(|f| is_ring(f)).count(), 4, "{arm:?}");
    }
}
