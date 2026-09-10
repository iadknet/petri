use super::*;
use crate::contracts::{InputReference, RouteTarget};
use crate::creature::genome::{VmBackendDef, VmInstruction};
use crate::mutation::{MutationSemanticCategory, TargetReachability};
use proptest::prelude::*;

fn vm_node(id: u32, register_count: u8) -> NodeGenome {
    NodeGenome {
        node_id: NodeId::new(id),
        input_refs: Vec::new(),
        backend_def: BackendDef::Vm(VmBackendDef {
            register_count,
            constants: Vec::new(),
            program: vec![VmInstruction::Halt],
        }),
        targets: Vec::new(),
    }
}

fn graph_node(id: u32) -> NodeGenome {
    NodeGenome {
        node_id: NodeId::new(id),
        input_refs: vec![InputReference::ActionQueue],
        backend_def: BackendDef::Graph(crate::creature::genome::cgp::CgpGraphBackendDef {
            compute_nodes: Vec::new(),
            output_sinks: Vec::new(),
            action_bank: Vec::new(),
            execute_gate: crate::creature::genome::cgp::ExecuteGate { inputs: Vec::new() },
        }),
        targets: vec![RouteTarget {
            target_id: NodeId::new(0),
            slot: 0,
            gate_bias: 0.0,
        }],
    }
}

/// A summary carrying exactly the given event records, with the totals the
/// engine would have produced for them.
fn summary_with(events: Vec<MutationEventRecord>) -> MutationSummary {
    let mut summary = MutationSummary::zero();
    for event in events {
        match (event.operator, event.outcome) {
            (Some(operator), MutationEventOutcome::Applied(reachability)) => {
                summary.record_attempt(event.domain, operator);
                summary.record_applied(
                    event.domain,
                    operator,
                    MutationSemanticCategory::SemanticChange,
                );
                summary.record_reachability(reachability);
            }
            (Some(operator), MutationEventOutcome::Skipped(reason)) => {
                summary.record_attempt(event.domain, operator);
                summary.record_skipped(operator, reason);
            }
            (None, MutationEventOutcome::Skipped(reason)) => {
                summary.record_domain_skip(event.domain, reason);
            }
            (None, MutationEventOutcome::Applied(_)) => unreachable!("no operator applied"),
        }
        summary.record_event(event);
    }
    summary
}

fn applied(operator: MutationOperator, target: u32) -> MutationEventRecord {
    MutationEventRecord {
        domain: operator.domain(),
        operator: Some(operator),
        target: Some(NodeId::new(target)),
        outcome: MutationEventOutcome::Applied(TargetReachability::Reachable),
    }
}

fn selected_inapplicable(domain: MutationDomain, target: u32) -> MutationEventRecord {
    MutationEventRecord {
        domain,
        operator: None,
        target: Some(NodeId::new(target)),
        outcome: MutationEventOutcome::Skipped(MutationSkipReason::NoApplicableTarget),
    }
}

fn no_eligible(domain: MutationDomain) -> MutationEventRecord {
    MutationEventRecord {
        domain,
        operator: None,
        target: None,
        outcome: MutationEventOutcome::Skipped(MutationSkipReason::NoApplicableTarget),
    }
}

fn birth(
    tracker: &mut RecruitmentTracker,
    depth: u64,
    before: &[NodeGenome],
    after: &[NodeGenome],
    summary: &MutationSummary,
) {
    tracker.record_birth(BirthObservation {
        lineage: 0,
        depth,
        before,
        after,
        summary,
    });
}

fn ids(nodes: &[u32]) -> BTreeSet<NodeId> {
    nodes.iter().map(|&id| NodeId::new(id)).collect()
}

#[test]
fn an_id_reused_after_deletion_is_two_modules() {
    let founder = vec![vm_node(0, 1), vm_node(1, 1)];
    let mut tracker = RecruitmentTracker::new(1);
    tracker.seed_founder(0, &founder);
    let after_removal = vec![vm_node(0, 1)];
    birth(
        &mut tracker,
        1,
        &founder,
        &after_removal,
        &summary_with(vec![applied(MutationOperator::TopologyRemoveNode, 1)]),
    );
    // Node id 1 is free again and a later birth reuses it.
    let reborn = vec![vm_node(0, 1), vm_node(1, 2)];
    birth(
        &mut tracker,
        5,
        &after_removal,
        &reborn,
        &summary_with(vec![applied(MutationOperator::TopologyAddNode, 0)]),
    );
    let modules: Vec<_> = tracker
        .modules()
        .filter(|module| module.node == NodeId::new(1))
        .collect();
    assert_eq!(modules.len(), 2);
    assert_eq!(modules[0].created_depth, 0);
    assert_eq!(modules[0].deleted_depth, Some(1));
    assert_eq!(modules[0].provenance, Provenance::Founder);
    assert_eq!(modules[1].created_depth, 5);
    assert_eq!(modules[1].deleted_depth, None);
    assert_eq!(modules[1].provenance, Provenance::New);
    // The dead module keeps its facts; the new one starts empty.
    assert_eq!(modules[1].first_selection, None);
}

#[test]
fn copy_provenance_needs_both_a_copy_event_and_matching_content() {
    let before = vec![vm_node(0, 3)];
    let mut copied = RecruitmentTracker::new(1);
    copied.seed_founder(0, &before);
    birth(
        &mut copied,
        1,
        &before,
        &[vm_node(0, 3), vm_node(1, 3)],
        &summary_with(vec![applied(MutationOperator::TopologyCopyNode, 0)]),
    );
    assert_eq!(
        copied.modules().last().map(|module| module.provenance),
        Some(Provenance::Copy)
    );

    // A blank detour matching nothing, added without a copy event, is new.
    let mut detour = RecruitmentTracker::new(1);
    detour.seed_founder(0, &before);
    birth(
        &mut detour,
        1,
        &before,
        &[vm_node(0, 3), vm_node(1, 7)],
        &summary_with(vec![applied(MutationOperator::TopologyAddNode, 0)]),
    );
    assert_eq!(
        detour.modules().last().map(|module| module.provenance),
        Some(Provenance::New)
    );

    // Content that happens to match an existing node, but no copy event.
    let mut coincidence = RecruitmentTracker::new(1);
    coincidence.seed_founder(0, &before);
    birth(
        &mut coincidence,
        1,
        &before,
        &[vm_node(0, 3), vm_node(1, 3)],
        &summary_with(vec![applied(MutationOperator::TopologySpliceNode, 0)]),
    );
    assert_eq!(
        coincidence.modules().last().map(|module| module.provenance),
        Some(Provenance::New)
    );
}

#[test]
fn the_censoring_split_keeps_every_module_in_the_denominator() {
    let founder = vec![vm_node(0, 1)];
    let mut tracker = RecruitmentTracker::new(1);
    tracker.seed_founder(0, &founder);
    // Three new modules: one reaches dispatch, one is deleted before any
    // fact, one lives on untouched.
    let grown = vec![vm_node(0, 1), vm_node(1, 1), vm_node(2, 1), vm_node(3, 1)];
    birth(
        &mut tracker,
        1,
        &founder,
        &grown,
        &summary_with(vec![applied(MutationOperator::TopologyAddNode, 0)]),
    );
    let pruned = vec![vm_node(0, 1), vm_node(1, 1), vm_node(3, 1)];
    birth(
        &mut tracker,
        2,
        &grown,
        &pruned,
        &summary_with(vec![applied(MutationOperator::TopologyRemoveNode, 2)]),
    );
    tracker.record_reading(0, 3, &ids(&[0, 1]), Some(&ids(&[1])));
    let reading = tracker.checkpoint(3);

    assert_eq!(reading.cohort.created, 3);
    assert_eq!(reading.cohort.deleted, 1);
    assert_eq!(reading.cohort.present, 2);
    assert_eq!(reading.cohort.contributing, 1);
    assert_eq!(reading.cohort.never_selected, 1);
    let dispatch = reading.time_to_first(CohortFact::Dispatch);
    assert_eq!(dispatch.reached, 1);
    assert_eq!(dispatch.median_generations, Some(2));
    assert_eq!(dispatch.censored_deleted, 1);
    assert_eq!(dispatch.censored_present, 1);
    assert_eq!(
        dispatch.reached + dispatch.censored_deleted + dispatch.censored_present,
        reading.cohort.created
    );
    // Founders are a separate reference row, never in the cohort.
    assert_eq!(reading.founders.created, 1);
    assert_eq!(reading.founders.present, 1);
    assert_eq!(reading.founders.dispatched, 1);
    assert_eq!(reading.founders.contributing, 0);
}

#[test]
fn a_selected_inapplicable_event_is_separated_from_a_missing_eligible_node() {
    let founder = vec![graph_node(0)];
    let mut tracker = RecruitmentTracker::new(1);
    tracker.seed_founder(0, &founder);
    birth(
        &mut tracker,
        1,
        &founder,
        &founder,
        &summary_with(vec![
            selected_inapplicable(MutationDomain::Graph, 0),
            no_eligible(MutationDomain::Vm),
        ]),
    );
    let reading = tracker.checkpoint(1);
    let pooled = &reading.opportunities;
    assert_eq!(pooled.attempted, 2);
    assert_eq!(pooled.skipped, 2);
    assert_eq!(
        pooled.selected_inapplicable_by_domain,
        BTreeMap::from([(MutationDomain::Graph, 1)])
    );
    assert_eq!(
        pooled.no_eligible_node_by_domain,
        BTreeMap::from([(MutationDomain::Vm, 1)])
    );
    // The selected node reached selection but not applicable selection.
    let founder_module = tracker.modules().next().expect("the founder module");
    assert_eq!(founder_module.first_selection, Some(1));
    assert_eq!(founder_module.first_applicable_selection, None);
}

#[test]
fn retention_splits_the_previous_checkpoints_contributors() {
    let founder = vec![vm_node(0, 1)];
    let mut tracker = RecruitmentTracker::new(1);
    tracker.seed_founder(0, &founder);
    let grown = vec![vm_node(0, 1), vm_node(1, 1), vm_node(2, 1), vm_node(3, 1)];
    birth(
        &mut tracker,
        1,
        &founder,
        &grown,
        &summary_with(vec![applied(MutationOperator::TopologyAddNode, 0)]),
    );
    tracker.record_reading(0, 1, &ids(&[1, 2, 3]), Some(&ids(&[1, 2, 3])));
    let first = tracker.checkpoint(1);
    assert_eq!(first.retention, None);
    assert_eq!(first.cohort.contributing, 3);

    let pruned = vec![vm_node(0, 1), vm_node(1, 1), vm_node(2, 1)];
    birth(
        &mut tracker,
        2,
        &grown,
        &pruned,
        &summary_with(vec![applied(MutationOperator::TopologyRemoveNode, 3)]),
    );
    tracker.record_reading(0, 2, &ids(&[1, 2]), Some(&ids(&[1])));
    let second = tracker.checkpoint(2);
    let retention = second.retention.expect("a second checkpoint has retention");
    assert_eq!(retention.from_depth, 1);
    assert_eq!(retention.contributing_before, 3);
    assert_eq!(retention.still_contributing, 1);
    assert_eq!(retention.present_not_contributing, 1);
    assert_eq!(retention.deleted, 1);
}

/// One generated generation for one lineage: which ids survive, which are
/// added, and which of the survivors an event selected.
#[derive(Debug, Clone)]
struct Generation {
    kept: Vec<u32>,
    added: Vec<u32>,
    selected: Vec<(u32, bool)>,
    executed: Vec<u32>,
    contributing: Vec<u32>,
}

fn generation_strategy() -> impl Strategy<Value = Generation> {
    (
        prop::collection::vec(0u32..8, 0..8),
        prop::collection::vec(0u32..8, 0..3),
        prop::collection::vec((0u32..8, any::<bool>()), 0..4),
        prop::collection::vec(0u32..8, 0..6),
        prop::collection::vec(0u32..8, 0..6),
    )
        .prop_map(
            |(kept, added, selected, executed, contributing)| Generation {
                kept,
                added,
                selected,
                executed,
                contributing,
            },
        )
}

proptest! {
    /// Whatever sequence of births and readings the tracker sees, its
    /// checkpoint reading keeps the cohort accounting consistent.
    #[test]
    fn cohort_readings_partition_every_module_they_count(
        generations in prop::collection::vec(generation_strategy(), 1..12),
        lineages in 1u32..4,
    ) {
        let founder: Vec<_> = (0..3).map(|id| vm_node(id, 1)).collect();
        let mut tracker = RecruitmentTracker::new(lineages);
        let mut current = vec![founder.clone(); lineages as usize];
        for lineage in 0..lineages {
            tracker.seed_founder(lineage, &founder);
        }
        let mut checkpoints = Vec::new();
        for (step, generation) in generations.iter().enumerate() {
            let depth = step as u64 + 1;
            for lineage in 0..lineages {
                let before = current[lineage as usize].clone();
                let live: BTreeSet<u32> = before.iter().map(|node| node.node_id.0).collect();
                let mut after: Vec<NodeGenome> = before.iter()
                    .filter(|node| generation.kept.contains(&node.node_id.0))
                    .cloned()
                    .collect();
                for &id in &generation.added {
                    if after.iter().all(|node| node.node_id.0 != id) {
                        after.push(if id % 2 == 0 { vm_node(id, 9) } else { graph_node(id) });
                    }
                }
                let events: Vec<_> = generation.selected.iter()
                    .filter(|(id, _)| live.contains(id))
                    .map(|&(id, apply)| if apply {
                        applied(MutationOperator::TopologyAddNode, id)
                    } else {
                        selected_inapplicable(MutationDomain::Topology, id)
                    })
                    .collect();
                tracker.record_birth(BirthObservation {
                    lineage,
                    depth,
                    before: &before,
                    after: &after,
                    summary: &summary_with(events),
                });
                current[lineage as usize] = after;
            }
            // Every other generation is a checkpoint reading with knockouts.
            let checkpoint = step % 2 == 1;
            for lineage in 0..lineages {
                let present: BTreeSet<u32> = current[lineage as usize]
                    .iter().map(|node| node.node_id.0).collect();
                let executed: BTreeSet<NodeId> = generation.executed.iter()
                    .filter(|id| present.contains(id))
                    .map(|&id| NodeId::new(id)).collect();
                // Contributing is drawn as a subset of executed, as the
                // battery reading guarantees.
                let contributing: BTreeSet<NodeId> = generation.contributing.iter()
                    .map(|&id| NodeId::new(id))
                    .filter(|id| executed.contains(id))
                    .collect();
                tracker.record_reading(
                    lineage,
                    depth,
                    &executed,
                    checkpoint.then_some(&contributing),
                );
            }
            if checkpoint {
                checkpoints.push(tracker.checkpoint(depth));
            }
        }

        for reading in &checkpoints {
            let cohort = reading.cohort;
            prop_assert_eq!(cohort.created, cohort.present + cohort.deleted);
            prop_assert_eq!(
                cohort.never_selected + cohort.selected_only + cohort.applied_only
                    + cohort.changed_only + cohort.dispatched_not_contributing
                    + cohort.contributing,
                cohort.present);
            prop_assert!(cohort.contributing <= cohort.dispatched());
            prop_assert!(cohort.dispatched() <= cohort.present);
            // Backend splits sum to the totals, rung by rung.
            let (graph, vm) = (reading.graph, reading.vm);
            prop_assert_eq!(graph.created + vm.created, cohort.created);
            prop_assert_eq!(graph.present + vm.present, cohort.present);
            prop_assert_eq!(graph.deleted + vm.deleted, cohort.deleted);
            prop_assert_eq!(graph.contributing + vm.contributing, cohort.contributing);
            prop_assert_eq!(
                graph.dispatched_not_contributing + vm.dispatched_not_contributing,
                cohort.dispatched_not_contributing);
            // Per-lineage rows sum to the pooled totals.
            prop_assert_eq!(reading.lineage_rows.len(), lineages as usize);
            prop_assert_eq!(
                reading.lineage_rows.iter().map(|row| row.created).sum::<u64>(), cohort.created);
            prop_assert_eq!(
                reading.lineage_rows.iter().map(|row| row.present).sum::<u64>(), cohort.present);
            prop_assert_eq!(
                reading.lineage_rows.iter().map(|row| row.dispatched).sum::<u64>(),
                cohort.dispatched());
            prop_assert_eq!(
                reading.lineage_rows.iter().map(|row| row.contributing).sum::<u64>(),
                cohort.contributing);
            // Every fact accounts for every created module.
            for fact in FACTS {
                let time = reading.time_to_first(fact);
                prop_assert_eq!(
                    time.reached + time.censored_deleted + time.censored_present,
                    cohort.created);
                prop_assert_eq!(time.median_generations.is_some(), time.reached > 0);
            }
            // Per-lineage opportunities sum to the pooled ones.
            let mut summed = Opportunities::default();
            for lineage in &reading.lineage_opportunities {
                summed.merge(lineage);
            }
            prop_assert_eq!(&summed, &reading.opportunities);
            prop_assert_eq!(
                reading.opportunities.attempted,
                reading.opportunities.applied + reading.opportunities.skipped);
            prop_assert_eq!(
                reading.opportunities.attempted_by_domain.values().sum::<u64>(),
                reading.opportunities.attempted);
        }
        for pair in checkpoints.windows(2) {
            let retention = pair[1].retention.expect("later checkpoints carry retention");
            prop_assert_eq!(retention.from_depth, pair[0].depth);
            prop_assert_eq!(retention.contributing_before,
                retention.still_contributing + retention.present_not_contributing
                    + retention.deleted);
        }
    }
}
