//! T25.F03: the mutation gate validates every Graph backend after every
//! applied event of any domain, and the bundle operators change
//! `genome_size` by their declared amounts.

use proptest::prelude::*;
use rand::rngs::SmallRng;
use rand::SeedableRng;

use super::*;
use crate::contracts::votes::{DirectedVoteKind, VoteSink};
use crate::contracts::{InputReference, NodeId, RouteTarget, WorldInputKey};
use crate::creature::founder::v3alpha1_founder_genome;
use crate::creature::parseability::ParseabilityGate;
use crate::genome::cgp::{test_support::edit_sinks, BundleId, CgpGraphBackendDef, OutputSinkKind};
use crate::genome::{NodeGenome, VmBackendDef, VmInstruction};
use crate::mutation::event::Broadcast;
use crate::mutation::graph::GraphOperator;
use crate::mutation::input_ref::InputRefOperator;
use crate::mutation::BundleEdits;

/// One gated Graph event through the engine's [`apply_event`], with the
/// bundle edits and broadcast it reported: empty when the gate rolled
/// it back, since only a committed event carries its [`EventReport`].
fn gated_graph_event(
    genome: &mut CreatureGenome,
    op: GraphOperator,
    targets: &mut TargetSelector<'_>,
    rng: &mut impl rand::Rng,
    config: &MutationConfig,
    context: &SensorContext,
) -> (
    Result<(TargetReachability, EventReport), MutationSkipReason>,
    BundleEdits,
    Option<Broadcast>,
) {
    let result = apply_event(genome, op.into(), targets, rng, config, context);
    let report = result
        .as_ref()
        .map_or_else(|_| EventReport::default(), |(_, report)| report.clone());
    (result, report.bundle_edits, report.broadcast)
}

fn graph_index(genome: &CreatureGenome) -> usize {
    genome
        .nodes
        .iter()
        .position(|node| matches!(node.backend_def(), BackendDef::Graph(_)))
        .expect("the founder carries a Graph node")
}

fn graph_mut(node: &mut NodeGenome) -> &mut CgpGraphBackendDef {
    match node.backend_edit().1 {
        BackendDef::Graph(def) => def,
        BackendDef::Vm(_) => panic!("a Graph node"),
    }
}

/// The founder with a barrier ring on its Graph node and one `Ring` bundle
/// reading it; returns the genome, the node index and the ring index.
fn grouped_founder() -> (CreatureGenome, usize, u16) {
    let mut genome = v3alpha1_founder_genome();
    let idx = graph_index(&genome);
    let node = &mut genome.nodes[idx];
    let ring = u16::try_from(node.input_refs().len()).unwrap();
    node.input_refs_mut()
        .push(InputReference::World(WorldInputKey::NeighborBarrierRing));
    let id = graph_mut(node).add_ring_bundle(ring, DirectedVoteKind::Move, 2);
    graph_mut(node).bundle_mut(id).set_scale(0.5);
    (genome, idx, ring)
}

fn gate_rejects(edit: impl FnOnce(&mut CreatureGenome, usize, u16)) {
    let (mut genome, idx, ring) = grouped_founder();
    assert!(genome.nodes.iter().all(|node| validate_graph(node).is_ok()));
    let snapshot = genome.clone();
    let result = apply_gated(&mut genome, |genome| {
        edit(genome, idx, ring);
        Ok(())
    });
    assert_eq!(result, Err(MutationSkipReason::GraphInvariantViolation));
    assert_eq!(genome, snapshot, "the draft is restored");
}

/// A Graph-domain edit that leaves a member on a sink its shape does not
/// name is rolled back.
#[test]
fn the_gate_rolls_back_a_graph_event_that_breaks_an_invariant() {
    gate_rejects(|genome, idx, _| {
        edit_sinks(graph_mut(&mut genome.nodes[idx]), |sinks| {
            for sink in sinks.iter_mut() {
                if sink.kind
                    == OutputSinkKind::ActionVote(VoteSink::Directed(DirectedVoteKind::Move, 2))
                {
                    sink.kind = OutputSinkKind::ActionVote(VoteSink::Eat);
                }
            }
        });
    });
}

/// An input-reference edit that turns a bundle's ring into a non-ring is
/// rolled back.
#[test]
fn the_gate_rolls_back_an_input_ref_event_that_breaks_an_invariant() {
    gate_rejects(|genome, idx, ring| {
        genome.nodes[idx].input_refs_mut()[usize::from(ring)] = InputReference::ActionVotes;
    });
}

/// A topology edit that moves a grouped backend onto a node whose
/// references hold no ring there is rolled back.
#[test]
fn the_gate_rolls_back_a_topology_event_that_breaks_an_invariant() {
    gate_rejects(|genome, idx, _| {
        let other = (0..genome.nodes.len())
            .find(|&i| i != idx)
            .expect("the founder carries two nodes");
        let backend = genome.nodes[idx].backend_def().clone();
        *genome.nodes[other].backend_edit().1 = backend;
    });
}

/// `RemoveBundle` removes a bundle and reports it when it commits; when the
/// gate rolls it back (another node breaks an invariant no bundle operator
/// can reach), it reports nothing and the draft is unchanged.
#[test]
fn a_rolled_back_event_adds_nothing_to_either_counter() {
    let (valid, idx, _) = grouped_founder();
    let other = (0..valid.nodes.len()).find(|&i| i != idx).unwrap();
    let mut broken = valid.clone();
    // `other` carries a member whose bundle record is missing (I3), so no
    // bundle operator applies there and every gate check fails.
    let mut json = serde_json::to_value(valid.nodes[idx].backend_def()).unwrap();
    json["Graph"]
        .as_object_mut()
        .expect("a Graph backend")
        .remove("bundles");
    *broken.nodes[other].backend_edit().1 = serde_json::from_value(json).unwrap();
    *broken.nodes[other].input_refs_mut() = valid.nodes[idx].input_refs().to_vec();
    assert!(validate_graph(&broken.nodes[other]).is_err());
    let config = MutationConfig::default();
    let all: Vec<usize> = (0..valid.nodes.len()).collect();
    for seed in 0..8 {
        let mut committed = valid.clone();
        let (result, edits, _) = gated_graph_event(
            &mut committed,
            GraphOperator::RemoveBundle,
            &mut TargetSelector::reachable_only(&all, 0.0),
            &mut SmallRng::seed_from_u64(seed),
            &config,
            &crate::config::test_sensor_context(2),
        );
        assert!(result.is_ok());
        assert_eq!(edits.removed, 1);

        let mut rolled_back = broken.clone();
        let (result, edits, scope) = gated_graph_event(
            &mut rolled_back,
            GraphOperator::RemoveBundle,
            &mut TargetSelector::reachable_only(&all, 0.0),
            &mut SmallRng::seed_from_u64(seed),
            &config,
            &crate::config::test_sensor_context(2),
        );
        assert_eq!(result, Err(MutationSkipReason::GraphInvariantViolation));
        assert_eq!(edits, BundleEdits::default());
        assert_eq!(scope, None);
        assert_eq!(rolled_back, broken);
    }
}

/// A learning operator that reports its reach when it commits (here
/// `EnableHebbian` and `ToggleCredits`) reports none when the gate rolls it
/// back: the bundle's ring reference is no longer a ring, which no learning
/// operator reads, so the operator applies and only the gate rejects it.
#[test]
fn a_rolled_back_learning_edit_reports_no_broadcast() {
    let (valid, idx, ring) = grouped_founder();
    let mut broken = valid.clone();
    broken.nodes[idx].input_refs_mut()[usize::from(ring)] = InputReference::ActionVotes;
    assert!(validate_graph(&broken.nodes[idx]).is_err());
    let config = MutationConfig::default();
    let all = every_node(&valid);
    for op in [GraphOperator::EnableHebbian, GraphOperator::ToggleCredits] {
        for seed in 0..16 {
            let mut committed = valid.clone();
            let (result, _, scope) = gated_graph_event(
                &mut committed,
                op,
                &mut TargetSelector::reachable_only(&all, 0.0),
                &mut SmallRng::seed_from_u64(seed),
                &config,
                &crate::config::test_sensor_context(2),
            );
            assert!(result.is_ok(), "{op:?} seed {seed}");
            assert!(scope.is_some(), "{op:?} seed {seed} reports its scope");

            let mut rolled_back = broken.clone();
            let (result, edits, scope) = gated_graph_event(
                &mut rolled_back,
                op,
                &mut TargetSelector::reachable_only(&all, 0.0),
                &mut SmallRng::seed_from_u64(seed),
                &config,
                &crate::config::test_sensor_context(2),
            );
            assert_eq!(
                result,
                Err(MutationSkipReason::GraphInvariantViolation),
                "{op:?} seed {seed}"
            );
            assert_eq!(edits, BundleEdits::default());
            assert_eq!(scope, None, "{op:?} seed {seed} reports no scope");
            assert_eq!(rolled_back, broken);
        }
    }
}

/// The fourteen operators whose committed change reports its reach (T25.F12
/// invariant 9).
const ATTRIBUTE_OPERATORS: [GraphOperator; 14] = [
    GraphOperator::EnableHebbian,
    GraphOperator::DisableHebbian,
    GraphOperator::ClearLearningOverride,
    GraphOperator::MutateHebbianRule,
    GraphOperator::MutateHebbianRate,
    GraphOperator::ToggleHebbianLamarckian,
    GraphOperator::EnableRewardModulation,
    GraphOperator::DisableRewardModulation,
    GraphOperator::MutateRewardSource,
    GraphOperator::MutateTraceDecay,
    GraphOperator::ToggleCredits,
    GraphOperator::EnableRetention,
    GraphOperator::DisableRetention,
    GraphOperator::MutateRetention,
];

/// Every node index of `genome`, the selection universe of the event tests.
fn every_node(genome: &CreatureGenome) -> Vec<usize> {
    (0..genome.nodes.len()).collect()
}

/// A committed `DetachGraphEdge` counts one detach and no removal while
/// members remain, keeps the record and adds one genome unit, the detached
/// edge (T25.F08, T25.F10).
#[test]
fn a_committed_detach_counts_one_graph_edge_detached() {
    let (genome, idx, _) = grouped_founder();
    let all = every_node(&genome);
    for seed in 0..16 {
        let mut detached = genome.clone();
        let (result, edits, scope) = gated_graph_event(
            &mut detached,
            GraphOperator::DetachGraphEdge,
            &mut TargetSelector::reachable_only(&all, 0.0),
            &mut SmallRng::seed_from_u64(seed),
            &MutationConfig::default(),
            &crate::config::test_sensor_context(2),
        );
        assert!(result.is_ok(), "seed {seed}");
        assert_eq!(
            edits,
            BundleEdits {
                detached: 1,
                removed: 0,
                ..BundleEdits::default()
            }
        );
        assert_eq!(scope, None, "a detach reports no attribute scope");
        assert_eq!(detached.genome_size(), genome.genome_size() + 1);
        let def = graph_mut(&mut detached.nodes[idx]);
        assert_eq!(def.bundles().len(), 1);
        assert_eq!(def.members(BundleId::new(0)).count(), 7);
    }
}

/// A last member's `DetachGraphEdge` removes the bundle: one detach, one
/// removal, and the record's genome unit becomes the detached edge's
/// (T25.F08 invariant 3, T25.F10).
#[test]
fn a_last_member_detach_removes_the_bundle_and_counts_one_removal() {
    let (mut genome, idx, _) = grouped_founder();
    let refs = genome.nodes[idx].input_refs().to_vec();
    let def = graph_mut(&mut genome.nodes[idx]);
    while def.members(BundleId::new(0)).count() > 1 {
        let member = def.members(BundleId::new(0)).next().unwrap();
        crate::genome::cgp::detach(def, member, &refs);
    }
    let all = every_node(&genome);
    let mut detached = genome.clone();
    let (result, edits, _) = gated_graph_event(
        &mut detached,
        GraphOperator::DetachGraphEdge,
        &mut TargetSelector::reachable_only(&all, 0.0),
        &mut SmallRng::seed_from_u64(0),
        &MutationConfig::default(),
        &crate::config::test_sensor_context(2),
    );
    assert!(result.is_ok());
    assert_eq!(
        edits,
        BundleEdits {
            detached: 1,
            removed: 1,
            ..BundleEdits::default()
        }
    );
    assert!(graph_mut(&mut detached.nodes[idx]).bundles().is_empty());
    assert_eq!(detached.genome_size(), genome.genome_size());
    let mut summary = MutationSummary::zero();
    summary.record_bundle_edits(
        MutationOperator::Graph(GraphOperator::DetachGraphEdge),
        edits,
    );
    assert_eq!(
        (summary.graph_edges_detached, summary.bundles_removed),
        (1, 1)
    );
}

/// A one-node genome whose only edge is the sole surviving member of a
/// fact-bearing pair gene: a fact member when `fact`, else a direction
/// member.
fn sole_member_genome(fact: bool) -> CreatureGenome {
    let area = InputReference::World(WorldInputKey::AreaOccupancySummary);
    let layout = crate::contracts::pair_layout(&area, 0);
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    let id = def.add_pair_bundle(0, 0, layout, DirectedVoteKind::Move, 3);
    def.bundle_mut(id).set_scale(0.5);
    let keep = if fact {
        def.members(id).last()
    } else {
        def.members(id).next()
    }
    .unwrap();
    let doomed: Vec<_> = def.members(id).filter(|&m| m != keep).collect();
    for member in doomed.into_iter().rev() {
        def.remove_edge(member);
    }
    let kept = def.members(id).next().unwrap();
    assert_eq!(def.members(id).count(), 1);
    assert_eq!(
        def.site_of(kept) == Some(crate::genome::cgp::Site::Drive),
        fact
    );
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome::new(
            NodeId::new(0),
            vec![area],
            BackendDef::Graph(def),
            vec![],
        )],
    }
}

/// The `genome_size` change of the record and member a sole member's detach
/// turns into one independent edge: `+1 − 1`, the record's unit becomes
/// the edge's (T25.F10).
const SOLE_MEMBER_DETACH_DELTA: i64 = 0;

/// T25.F11 invariant 8: every creation operator changes `genome_size` by
/// its unit delta: `AddSensor*` on a Graph node 1 + the bundles it made
/// (`NearbyCreature` 5, an area summary 3, any other sensor 2), on a VM
/// node 1; `AddBundle*` 1. Seeds are walked until every sensor class has
/// been seen on the Graph node.
#[test]
fn every_creation_operator_moves_genome_size_by_its_unit_delta() {
    use crate::contracts::SensorDescriptor;
    use crate::mutation::graph::GraphMutator;
    use crate::mutation::input_ref::{InputRefMutator, InputRefOperator};
    let context = crate::config::test_sensor_context(1);
    let base = v3alpha1_founder_genome();
    let graph = graph_index(&base);
    let mut vm_genome = base.clone();
    *vm_genome.nodes[graph].backend_edit().1 = BackendDef::Vm(VmBackendDef {
        register_count: 1,
        constants: vec![],
        program: vec![VmInstruction::Halt],
    });
    let mut seen = std::collections::BTreeSet::new();
    for op in [
        InputRefOperator::AddSensorLive,
        InputRefOperator::AddSensorSilent,
    ] {
        for seed in 0..512 {
            let mut genome = base.clone();
            InputRefMutator::apply_to_node(
                &mut genome,
                op,
                graph,
                &mut SmallRng::seed_from_u64(seed),
                &context,
            )
            .expect("a full catalog wires every sensor");
            let reference = genome.nodes[graph].input_refs().last().unwrap();
            let sets = SensorDescriptor::of(reference, &context).field_set_count();
            let delta = genome.genome_size() - base.genome_size();
            assert_eq!(delta as usize, 1 + sets, "{op:?} {reference:?}");
            seen.insert(sets);

            let mut vm = vm_genome.clone();
            InputRefMutator::apply_to_node(
                &mut vm,
                op,
                graph,
                &mut SmallRng::seed_from_u64(seed),
                &context,
            )
            .unwrap();
            assert_eq!(vm.genome_size(), vm_genome.genome_size() + 1, "{op:?}");
        }
    }
    assert_eq!(
        seen,
        [1, 2, 4].into(),
        "a plain sensor, an area summary, NearbyCreature"
    );
    for op in [GraphOperator::AddBundleLive, GraphOperator::AddBundleSilent] {
        for seed in 0..64 {
            let mut genome = base.clone();
            GraphMutator::apply_to_node(
                &mut genome,
                op,
                graph,
                &mut SmallRng::seed_from_u64(seed),
                &MutationConfig::default(),
                &context,
            )
            .expect("the founder's Graph node has an admissible source");
            assert_eq!(genome.genome_size(), base.genome_size() + 1, "{op:?}");
        }
    }
}

/// Invariant 1: a sole direction member and a sole fact member under each
/// detaching operator (`DetachGraphEdge`, `RetargetGraphEdge`,
/// `GraphRawFieldMutation`, the split form of `AddInternalGraphNode`)
/// change `genome_size` by the operator's own delta plus the detach's.
#[test]
fn a_sole_members_detach_moves_genome_size_by_the_unit_rule() {
    for fact in [false, true] {
        let genome = sole_member_genome(fact);
        assert_eq!(validate_graph(&genome.nodes[0]), Ok(()));
        let all = every_node(&genome);
        // The operator's own delta: a split adds an identity node and its
        // input edge; the others add nothing.
        for (op, own) in [
            (GraphOperator::DetachGraphEdge, 0),
            (GraphOperator::RetargetGraphEdge, 0),
            (GraphOperator::GraphRawFieldMutation, 0),
            (GraphOperator::AddInternalGraphNode, 2),
        ] {
            let mut detached_once = false;
            for seed in 0..64 {
                let mut mutated = genome.clone();
                let (result, edits, _) = gated_graph_event(
                    &mut mutated,
                    op,
                    &mut TargetSelector::reachable_only(&all, 0.0),
                    &mut SmallRng::seed_from_u64(seed),
                    &MutationConfig::default(),
                    &crate::config::test_sensor_context(2),
                );
                if result.is_err() || edits.removed == 0 {
                    continue;
                }
                detached_once = true;
                assert_eq!(edits.detached, 1, "{op:?} fact {fact}");
                let delta = i64::from(mutated.genome_size()) - i64::from(genome.genome_size());
                assert_eq!(
                    delta,
                    own + SOLE_MEMBER_DETACH_DELTA,
                    "{op:?} fact {fact} seed {seed}"
                );
            }
            assert!(detached_once, "{op:?} fact {fact} detached its member");
        }
    }
}

/// The fourteen learning operators report the reach of their committed
/// change (T25.F12 invariant 9): `Members` when a member was picked and the
/// coin was true (the change reaches every admitted member), else `One`
/// (one target changed: a member with a false coin, a lone edge or a
/// slot). Creation and detach report none.
#[test]
fn a_committed_learning_edit_reports_its_reach() {
    let (genome, idx, _) = grouped_founder();
    let all = every_node(&genome);
    let plasticity = |genome: &CreatureGenome| -> Vec<_> {
        genome
            .nodes
            .iter()
            .filter_map(|node| match node.backend_def() {
                BackendDef::Graph(def) => Some(def),
                BackendDef::Vm(_) => None,
            })
            .flat_map(|def| {
                def.edge_sites()
                    .map(|at| {
                        let edge = def.edge_at(at);
                        (edge.stored_learning(), edge.membership())
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    };
    // EnableHebbian: (member, true coin), (member, false coin), a lone
    // edge, a slot (no edge changes).
    let mut seen = [false; 4];
    for seed in 0..256 {
        let mut changed = genome.clone();
        let (result, _, scope) = gated_graph_event(
            &mut changed,
            GraphOperator::EnableHebbian,
            &mut TargetSelector::reachable_only(&all, 0.0),
            &mut SmallRng::seed_from_u64(seed),
            &MutationConfig::default(),
            &crate::config::test_sensor_context(2),
        );
        assert!(result.is_ok(), "seed {seed}");
        let (before, after) = (plasticity(&genome), plasticity(&changed));
        let diffs: Vec<_> = before
            .iter()
            .zip(&after)
            .filter(|(old, new)| old.0 != new.0)
            .map(|(_, new)| new.1)
            .collect();
        match scope {
            Some(Broadcast::Members) => {
                assert_eq!(diffs.len(), 8, "every member, seed {seed}");
                seen[0] = true;
            }
            Some(Broadcast::One) => {
                assert!(diffs.len() <= 1, "seed {seed}");
                seen[match diffs.first() {
                    Some(Some(_)) => 1,
                    Some(None) => 2,
                    None => 3,
                }] = true;
            }
            None => panic!("a learning edit reports its reach, seed {seed}"),
        }
    }
    assert_eq!(seen, [true; 4], "every case drawn");

    // ToggleCredits: `Members` flips every member, `One` one.
    let mut reaches = [false; 2];
    for seed in 0..64 {
        let mut changed = genome.clone();
        let (result, _, scope) = gated_graph_event(
            &mut changed,
            GraphOperator::ToggleCredits,
            &mut TargetSelector::reachable_only(&all, 0.0),
            &mut SmallRng::seed_from_u64(seed),
            &MutationConfig::default(),
            &crate::config::test_sensor_context(2),
        );
        assert!(result.is_ok());
        let def = graph_mut(&mut changed.nodes[idx]);
        let crediting = def
            .members(BundleId::new(0))
            .filter(|&at| def.edge_at(at).membership().unwrap().credits)
            .count();
        let reach = scope.expect("ToggleCredits reports its reach");
        assert_eq!(crediting, if reach == Broadcast::Members { 8 } else { 1 });
        reaches[reach.index()] = true;
    }
    assert_eq!(reaches, [true; 2]);

    for op in [
        GraphOperator::AddBundleSilent,
        GraphOperator::AddBundleLive,
        GraphOperator::DetachGraphEdge,
        GraphOperator::AlterGraphEdgeWeight,
    ] {
        let mut changed = genome.clone();
        let (result, _, scope) = gated_graph_event(
            &mut changed,
            op,
            &mut TargetSelector::reachable_only(&all, 0.0),
            &mut SmallRng::seed_from_u64(1),
            &MutationConfig::default(),
            &crate::config::test_sensor_context(2),
        );
        assert!(result.is_ok(), "{op:?}");
        assert_eq!(scope, None, "{op:?} reports no broadcast");
    }

    let mut summary = MutationSummary::zero();
    summary.record_broadcast(Broadcast::Members);
    summary.record_broadcast(Broadcast::One);
    summary.record_broadcast(Broadcast::Members);
    assert_eq!(summary.attribute_changes_by_broadcast, [1, 2]);
}

/// A summary accumulates every committed event's detaches and removals.
#[test]
fn record_bundle_edits_accumulates_across_events() {
    let mut summary = MutationSummary::zero();
    summary.record_bundle_edits(
        MutationOperator::Graph(GraphOperator::RemoveBundle),
        BundleEdits {
            detached: 2,
            removed: 1,
            ..BundleEdits::default()
        },
    );
    summary.record_bundle_edits(
        MutationOperator::Graph(GraphOperator::RemoveBundle),
        BundleEdits {
            detached: 3,
            removed: 4,
            ..BundleEdits::default()
        },
    );
    assert_eq!(summary.graph_edges_detached, 5);
    assert_eq!(summary.bundles_removed, 5);
}

/// The `genome_size` change of one applied bundle-touching event, read from
/// the target node before and after.
fn expected_delta(
    op: MutationOperator,
    before: &CgpGraphBackendDef,
    after: &CgpGraphBackendDef,
) -> Option<Vec<i64>> {
    match op {
        MutationOperator::Graph(GraphOperator::AddBundleSilent)
        | MutationOperator::Graph(GraphOperator::AddBundleLive) => Some(vec![1]),
        // One declaration and one record per field set (T25.F11
        // invariant 8).
        MutationOperator::InputRef(InputRefOperator::AddSensorLive)
        | MutationOperator::InputRef(InputRefOperator::AddSensorSilent) => {
            let made = after.bundles().len() - before.bundles().len();
            Some(vec![1 + i64::try_from(made).unwrap()])
        }
        MutationOperator::Graph(GraphOperator::DetachGraphEdge) => {
            Some(vec![if after.bundles().len() < before.bundles().len() {
                0
            } else {
                1
            }])
        }
        MutationOperator::Graph(GraphOperator::RemoveBundle) => Some(vec![-1]),
        MutationOperator::Graph(GraphOperator::RemoveGraphEdge)
            if after.bundles().len() < before.bundles().len() =>
        {
            Some(vec![-1])
        }
        MutationOperator::Graph(GraphOperator::RemoveGraphEdge)
            if member_removed(before, after) =>
        {
            Some(vec![0])
        }
        MutationOperator::Graph(GraphOperator::AlterBundleScale)
        | MutationOperator::Graph(GraphOperator::ShiftBundleOffset)
        | MutationOperator::Graph(GraphOperator::RetargetBundle)
        | MutationOperator::Graph(GraphOperator::EnableRetention)
        | MutationOperator::Graph(GraphOperator::DisableRetention)
        | MutationOperator::Graph(GraphOperator::MutateRetention) => Some(vec![0]),
        _ => None,
    }
}

/// Whether a committed edit removed a member: fewer members after it.
fn member_removed(before: &CgpGraphBackendDef, after: &CgpGraphBackendDef) -> bool {
    let members = |def: &CgpGraphBackendDef| {
        def.output_sinks()
            .iter()
            .flat_map(|sink| sink.inputs.iter())
            .filter(|edge| edge.membership().is_some())
            .count()
    };
    members(after) < members(before)
}

/// The shape of one generated bundle: every wiring and both sources
/// (T25.F11).
#[derive(Clone, Copy, Debug)]
enum BundleShape {
    /// A `Ring` bundle on an appended occupied-ring reference.
    Ring,
    /// A `DirectionPair` bundle on occupancy pair `pair`, its facts on the
    /// drive. The last member is a fact member (the drive sinks follow
    /// every direction sink), so the drops generate fact holes and
    /// fact-only survivors as well as sole direction members.
    Pair { pair: u8 },
    /// A `Plain` bundle on vote sink `sink` (a catalog index) reading the
    /// appended `CommitCounts` sensor, or a memory bank when `memory` is
    /// set (`previous` the bank).
    Plain { sink: usize, memory: Option<bool> },
}

/// One generated bundle: kind, offset, scale, the members to drop (a bit per
/// member in canonical order, the last always kept) and the shape.
type BundleSpec = (usize, u8, f32, u16, BundleShape);

/// Draws a `BundleSpec` over every shape: a ring, or each occupancy pair with
/// and without its fact members.
fn bundle_spec() -> impl Strategy<Value = BundleSpec> {
    let pairs = WorldInputKey::AreaOccupancySummary.direction_pairs().len();
    let pair = (0..u8::try_from(pairs).unwrap()).prop_map(|pair| BundleShape::Pair { pair });
    let plain = (
        0..crate::contracts::votes::VOTE_SINK_COUNT,
        prop::option::of(any::<bool>()),
    )
        .prop_map(|(sink, memory)| BundleShape::Plain { sink, memory });
    let shape = prop_oneof![1 => Just(BundleShape::Ring), 2 => pair, 2 => plain];
    (0usize..3, 0u8..8, -2.0f32..2.0, any::<u16>(), shape)
}

/// Adds one bundle per entry of `bundles` to the Graph `node`: a `Ring` reads
/// an appended occupied-ring reference, a `DirectionPair` an appended
/// occupancy summary, a `Plain` an appended `CommitCounts` or a memory bank;
/// each with members dropped (a bundle keeps at least one).
fn add_bundles(node: &mut NodeGenome, bundles: &[BundleSpec]) {
    use crate::genome::cgp::{BundleBirth, BundleSource, Wiring};
    let ring = u16::try_from(node.input_refs().len()).unwrap();
    node.input_refs_mut()
        .push(InputReference::World(WorldInputKey::NeighborOccupiedRing));
    let area = InputReference::World(WorldInputKey::AreaOccupancySummary);
    let summary = ring + 1;
    node.input_refs_mut().push(area.clone());
    let counts = ring + 2;
    node.input_refs_mut().push(InputReference::CommitCounts);
    let context = crate::config::test_sensor_context(1);
    for &(kind, offset, scale, drop, shape) in bundles {
        let kind = DirectedVoteKind::ALL[kind];
        let id = match shape {
            BundleShape::Ring => graph_mut(node).add_ring_bundle(ring, kind, offset),
            BundleShape::Pair { pair } => {
                let layout = crate::contracts::pair_layout(&area, pair);
                graph_mut(node).add_pair_bundle(summary, pair, layout, kind, offset)
            }
            BundleShape::Plain { sink, memory } => {
                let source = match memory {
                    Some(previous) => BundleSource::Memory { previous },
                    None => BundleSource::Sensor {
                        ref_idx: counts,
                        field_set: 0,
                    },
                };
                let birth = BundleBirth {
                    field_set: 0,
                    wiring: Wiring::Plain {
                        sink: VoteSink::from_index(sink).unwrap(),
                    },
                    scale: 0.0,
                };
                node.add_bundle(source, birth, &context).unwrap().0
            }
        };
        let def = graph_mut(node);
        def.bundle_mut(id).set_scale(scale);
        let members: Vec<_> = def.members(id).collect();
        let last = members.len() - 1;
        for (index, occurrence) in members.into_iter().enumerate().rev() {
            if index < last && index < 16 && drop & (1 << index) != 0 {
                def.remove_edge(occurrence);
            }
        }
    }
}

/// The founder with ring bundles on both Graph nodes — node 1 is routed from
/// node 0, so Topology copies and swaps reach a grouped backend — and an
/// appended VM node routed from node 1, so VM events have a target.
fn grouped_mixed_genome(bundles: &[Vec<BundleSpec>; 2]) -> CreatureGenome {
    let mut genome = v3alpha1_founder_genome();
    let graphs: Vec<usize> = (0..genome.nodes.len())
        .filter(|&i| matches!(genome.nodes[i].backend_def(), BackendDef::Graph(_)))
        .collect();
    assert_eq!(graphs.len(), 2, "the founder carries two Graph nodes");
    for (&idx, bundles) in graphs.iter().zip(bundles) {
        add_bundles(&mut genome.nodes[idx], bundles);
    }
    let vm_id = NodeId::new(genome.nodes.iter().map(|n| n.node_id.0).max().unwrap() + 1);
    genome.nodes[graphs[1]].targets.push(RouteTarget {
        target_id: vm_id,
        slot: 0,
        gate_bias: 0.0,
    });
    genome.nodes.push(NodeGenome::new(
        vm_id,
        vec![],
        BackendDef::Vm(VmBackendDef {
            register_count: 2,
            constants: vec![1.0],
            program: vec![VmInstruction::Halt],
        }),
        vec![],
    ));
    genome
}

proptest! {
    /// Random grouped genomes (two grouped Graph nodes and a VM node) under
    /// random events of all four domains, the mesh layer's share drawn over
    /// [0, 1]: every applied event leaves every Graph backend valid, the gate
    /// never rejects one, and each bundle operator changes `genome_size` by
    /// its declared amount under the unit rule (T25.F10): every creation +1,
    /// `DetachGraphEdge` +1 or 0 when the record goes, `RemoveBundle` −1, a
    /// member's `RemoveGraphEdge` 0 or −1 when the record goes, the record
    /// edits 0).
    #[test]
    fn every_applied_event_keeps_the_graph_invariants(
        seed in any::<u64>(),
        mesh_layer_probability in 0.0f64..=1.0,
        bundles in [
            prop::collection::vec(bundle_spec(), 0..4),
            prop::collection::vec(bundle_spec(), 0..4),
        ],
    ) {
        let mut genome = grouped_mixed_genome(&bundles);
        prop_assert!(ParseabilityGate::validate(&genome).is_ok());
        for node in &genome.nodes {
            prop_assert_eq!(validate_graph(node), Ok(()));
        }
        let config = MutationConfig {
            per_unit_rate: 1.0,
            mesh_layer_probability,
            ..MutationConfig::default()
        };
        let mut rng = SmallRng::seed_from_u64(seed);
        for _ in 0..32 {
            let before = genome.clone();
            let reachable: Vec<usize> = (0..genome.nodes.len()).collect();
            let summary = MutationEngine::apply_mutations_on_units(
                &mut genome,
                1,
                &config,
                &reachable,
                ParentExecuted::NONE,
                &mut rng,
                &crate::config::test_sensor_context(2),
            ).0;
            // Every committed learning edit reports one reach (T25.F12).
            let attribute_events: u32 = ATTRIBUTE_OPERATORS
                .iter()
                .filter_map(|op| summary.applied_by_operator.get(&MutationOperator::from(*op)))
                .sum();
            prop_assert_eq!(summary.attribute_changes_by_broadcast.iter().sum::<u32>(), attribute_events);
            prop_assert!(!summary.skip_reasons.contains_key(&MutationSkipReason::GraphInvariantViolation));
            for node in &genome.nodes {
                prop_assert_eq!(validate_graph(node), Ok(()));
            }
            let Some(event) = summary.events.first() else { continue };
            let (Some(op), MutationEventOutcome::Applied(_)) = (event.operator, &event.outcome) else {
                continue;
            };
            let target = event.target.and_then(|id| before.nodes.iter().position(|n| n.node_id == id));
            let Some(target) = target else { continue };
            let (BackendDef::Graph(old), Some(BackendDef::Graph(new))) = (
                before.nodes[target].backend_def(),
                genome.nodes.iter().find(|n| n.node_id == before.nodes[target].node_id).map(|n| n.backend_def()),
            ) else {
                continue;
            };
            if let Some(allowed) = expected_delta(op, old, new) {
                let delta = i64::from(genome.genome_size()) - i64::from(before.genome_size());
                prop_assert!(allowed.contains(&delta), "{:?}: {} not in {:?}", op, delta, allowed);
            }
            let removed = before.nodes.iter().map(|n| match n.backend_def() {
                BackendDef::Graph(d) => d.bundles().len(),
                BackendDef::Vm(_) => 0,
            }).sum::<usize>().saturating_sub(genome.nodes.iter().map(|n| match n.backend_def() {
                BackendDef::Graph(d) => d.bundles().len(),
                BackendDef::Vm(_) => 0,
            }).sum::<usize>());
            if matches!(
                op,
                MutationOperator::Graph(GraphOperator::RemoveBundle)
                    | MutationOperator::Graph(GraphOperator::RemoveGraphEdge)
                    | MutationOperator::Graph(GraphOperator::DetachGraphEdge)
            ) {
                prop_assert_eq!(summary.bundles_removed as usize, removed);
            }
            if op == MutationOperator::Graph(GraphOperator::DetachGraphEdge) {
                prop_assert_eq!(summary.graph_edges_detached, 1);
            }
        }
    }
}
