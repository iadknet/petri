use super::*;
use crate::config::NeutralInputRecruitment as Arm;
use crate::contracts::NodeId;
use crate::creature::genome::cgp::{GraphEdge, GraphSource, OutputSink, OutputSinkKind};
use crate::creature::genome::vote::VoteSink;
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
            assert_eq!(plan.sources.len(), width as usize);
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
    graph_mut(&mut fixture).output_sinks[0]
        .inputs
        .push(GraphEdge {
            source: GraphSource::InputLeaf {
                ref_idx: 1,
                sub_idx: 0,
            },
            weight: 3.0,
        });
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
    graph.output_sinks.push(OutputSink {
        kind: OutputSinkKind::ActionVote(VoteSink::Move(8)),
        inputs: vec![],
    });
    graph.output_sinks.push(OutputSink {
        kind: OutputSinkKind::ActionVote(VoteSink::Eat),
        inputs: vec![],
    });
    let sites = destinations(graph);
    assert_eq!(sites.len(), 28);
    for vote in VoteSink::all() {
        assert!(sites
            .iter()
            .any(|&i| graph.output_sinks[i].kind == OutputSinkKind::ActionVote(vote)));
    }
    assert!(!sites.contains(&(graph.output_sinks.len() - 2)));
    assert!(sites.contains(&(graph.output_sinks.len() - 1)));
}

proptest! {
    #[test]
    fn recruitment_preserves_incumbents_and_obeys_bounds(seed in any::<u64>(), whole in any::<bool>(), declarations in 0usize..8, weight in -100f32..100f32) {
        let mut fixture = node();
        fixture.input_refs = vec![InputReference::ActionVotes; declarations];
        graph_mut(&mut fixture).output_sinks[64].inputs = vec![GraphEdge { source: GraphSource::SharedMemory { slot: 3, previous: true }, weight }; 2];
        let before = fixture.clone();
        let arm = if whole { Arm::WholeFamily } else { Arm::SingleChannel };
        recruit(&mut fixture, arm, 2, &mut SmallRng::seed_from_u64(seed)).unwrap();
        prop_assert_eq!(&fixture.input_refs[..declarations], &before.input_refs);
        prop_assert!(fixture.input_refs.len() - declarations <= 24);
        let before_graph = graph(&before);
        let after_graph = graph(&fixture);
        let mut changed = 0;
        let mut added = 0;
        for (old, new) in before_graph.output_sinks.iter().zip(&after_graph.output_sinks) {
            prop_assert_eq!(old.kind, new.kind);
            prop_assert_eq!(&new.inputs[..old.inputs.len()], &old.inputs);
            if new.inputs.len() != old.inputs.len() { changed += 1; }
            added += new.inputs.len() - old.inputs.len();
            prop_assert!(new.inputs[old.inputs.len()..].iter().all(|edge| edge.weight == 0.0));
        }
        prop_assert_eq!(changed, 1);
        prop_assert!((1..=27).contains(&added));
        if !whole { prop_assert_eq!(added, 1); }
        prop_assert_eq!(&after_graph.compute_nodes, &before_graph.compute_nodes);
        prop_assert_eq!(&fixture.targets, &before.targets);
        prop_assert_eq!(fixture.node_id, before.node_id);
    }

    #[test]
    fn full_or_dangling_tables_still_apply_without_retry(seed in any::<u64>(), whole in any::<bool>(), full in any::<bool>()) {
        let mut fixture = node();
        if full { fixture.input_refs = vec![InputReference::ActionVotes; 65537]; }
        else { graph_mut(&mut fixture).output_sinks[0].inputs.push(GraphEdge { source: GraphSource::InputLeaf { ref_idx: 0, sub_idx: 0 }, weight: 1.0 }); }
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
    assert_eq!(single.len(), 48);
    let whole = admission.families(Arm::WholeFamily, 2);
    assert_eq!(whole.len(), 24);
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
        graph(&fixture).edges().next().unwrap().source,
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
    graph_mut(&mut fixture)
        .output_sinks
        .iter_mut()
        .for_each(|sink| sink.inputs.clear());
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
        graph(&fixture).edges().last().unwrap().source,
        GraphSource::InputLeaf {
            ref_idx: u16::MAX,
            sub_idx: 0
        }
    );
}

#[test]
fn rejected_explicit_bundle_leaves_every_incumbent_unchanged() {
    let mut fixture = node();
    graph_mut(&mut fixture).output_sinks[0]
        .inputs
        .push(GraphEdge {
            source: GraphSource::InputLeaf {
                ref_idx: 12,
                sub_idx: 4,
            },
            weight: 1.5,
        });
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
        let old: Vec<_> = GraphOperator::ALL.into_iter().filter(|op| *op != GraphOperator::RecruitNeutralInput).collect();
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
