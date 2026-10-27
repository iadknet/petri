//! T25.F03: the mutation gate validates every Graph backend after every
//! applied event of any domain, and the group operators change
//! `genome_size` by their declared amounts.

use proptest::prelude::*;
use rand::rngs::SmallRng;
use rand::SeedableRng;

use super::*;
use crate::contracts::votes::{DirectedVoteKind, VoteSink};
use crate::contracts::{InputReference, NodeId, RouteTarget, WorldInputKey};
use crate::creature::founder::v3alpha1_founder_genome;
use crate::creature::parseability::ParseabilityGate;
use crate::genome::cgp::{
    test_support::edit_sinks, CgpGraphBackendDef, GroupId, OutputSinkKind, Scope, Shape,
};
use crate::genome::{NodeGenome, VmBackendDef, VmInstruction};

fn graph_index(genome: &CreatureGenome) -> usize {
    genome
        .nodes
        .iter()
        .position(|node| matches!(node.backend_def, BackendDef::Graph(_)))
        .expect("the founder carries a Graph node")
}

fn graph_mut(node: &mut NodeGenome) -> &mut CgpGraphBackendDef {
    match &mut node.backend_def {
        BackendDef::Graph(def) => def,
        BackendDef::Vm(_) => panic!("a Graph node"),
    }
}

/// The founder with a barrier ring on its Graph node and one `Ring` group
/// reading it; returns the genome, the node index and the ring index.
fn grouped_founder() -> (CreatureGenome, usize, u16) {
    let mut genome = v3alpha1_founder_genome();
    let idx = graph_index(&genome);
    let node = &mut genome.nodes[idx];
    let ring = u16::try_from(node.input_refs.len()).unwrap();
    node.input_refs
        .push(InputReference::World(WorldInputKey::NeighborBarrierRing));
    let id = graph_mut(node).add_group(Shape::Ring { ring_ref: ring }, DirectedVoteKind::Move, 2);
    graph_mut(node).group_mut(id).set_scale(0.5);
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

/// An input-reference edit that turns a group's ring into a non-ring is
/// rolled back.
#[test]
fn the_gate_rolls_back_an_input_ref_event_that_breaks_an_invariant() {
    gate_rejects(|genome, idx, ring| {
        genome.nodes[idx].input_refs[usize::from(ring)] = InputReference::ActionVotes;
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
        let backend = genome.nodes[idx].backend_def.clone();
        genome.nodes[other].backend_def = backend;
    });
}

/// `RemoveGroup` removes a group and reports it when it commits; when the
/// gate rolls it back (another node breaks an invariant no group operator
/// can reach), it reports nothing and the draft is unchanged.
#[test]
fn a_rolled_back_event_adds_nothing_to_either_counter() {
    let (valid, idx, _) = grouped_founder();
    let other = (0..valid.nodes.len()).find(|&i| i != idx).unwrap();
    let mut broken = valid.clone();
    // `other` carries a member whose group record is missing (I3), so no
    // group operator applies there and every gate check fails.
    let mut json = serde_json::to_value(&valid.nodes[idx].backend_def).unwrap();
    json["Graph"]
        .as_object_mut()
        .expect("a Graph backend")
        .remove("groups");
    broken.nodes[other].backend_def = serde_json::from_value(json).unwrap();
    broken.nodes[other].input_refs = valid.nodes[idx].input_refs.clone();
    assert!(validate_graph(&broken.nodes[other]).is_err());
    let config = MutationConfig::default();
    let all: Vec<usize> = (0..valid.nodes.len()).collect();
    for seed in 0..8 {
        let mut committed = valid.clone();
        let (result, edits, _) = apply_graph_event(
            &mut committed,
            GraphOperator::RemoveGroup,
            &mut TargetSelector::reachable_only(&all, 0.0),
            &mut SmallRng::seed_from_u64(seed),
            &config,
            &crate::config::test_sensor_context(2),
        );
        assert!(result.is_ok());
        assert_eq!(edits.removed, 1);

        let mut rolled_back = broken.clone();
        let (result, edits, scope) = apply_graph_event(
            &mut rolled_back,
            GraphOperator::RemoveGroup,
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

/// An attribute operator that reports its scope when it commits (here
/// `EnableHebbian` and `SetScope`) reports none when the gate rolls it back:
/// the group's ring reference is no longer a ring, which no attribute
/// operator reads, so the operator applies and only the gate rejects it.
#[test]
fn a_rolled_back_attribute_change_reports_no_scope() {
    let (valid, idx, ring) = grouped_founder();
    let mut broken = valid.clone();
    broken.nodes[idx].input_refs[usize::from(ring)] = InputReference::ActionVotes;
    assert!(validate_graph(&broken.nodes[idx]).is_err());
    let config = MutationConfig::default();
    let all = every_node(&valid);
    for op in [GraphOperator::EnableHebbian, GraphOperator::SetScope] {
        for seed in 0..16 {
            let mut committed = valid.clone();
            let (result, _, scope) = apply_graph_event(
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
            let (result, edits, scope) = apply_graph_event(
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

/// The ten operators whose committed change reports its scope (T25.F08
/// invariant 8).
const ATTRIBUTE_OPERATORS: [GraphOperator; 13] = [
    GraphOperator::EnableHebbian,
    GraphOperator::DisableHebbian,
    GraphOperator::MutateHebbianRule,
    GraphOperator::MutateHebbianRate,
    GraphOperator::ToggleHebbianLamarckian,
    GraphOperator::EnableRewardModulation,
    GraphOperator::DisableRewardModulation,
    GraphOperator::MutateRewardSource,
    GraphOperator::MutateTraceDecay,
    GraphOperator::SetScope,
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
        let (result, edits, scope) = apply_graph_event(
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
        assert_eq!(def.groups().len(), 1);
        assert_eq!(def.members(GroupId::new(0)).count(), 7);
    }
}

/// A last member's `DetachGraphEdge` removes the group: one detach, one
/// removal, and the record's genome unit becomes the detached edge's
/// (T25.F08 invariant 3, T25.F10).
#[test]
fn a_last_member_detach_removes_the_group_and_counts_one_removal() {
    let (mut genome, idx, _) = grouped_founder();
    let refs = genome.nodes[idx].input_refs.clone();
    let def = graph_mut(&mut genome.nodes[idx]);
    while def.members(GroupId::new(0)).count() > 1 {
        let member = def.members(GroupId::new(0)).next().unwrap();
        crate::genome::cgp::detach(def, member, &refs);
    }
    let all = every_node(&genome);
    let mut detached = genome.clone();
    let (result, edits, _) = apply_graph_event(
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
    assert!(graph_mut(&mut detached.nodes[idx]).groups().is_empty());
    assert_eq!(detached.genome_size(), genome.genome_size());
    let mut summary = MutationSummary::zero();
    summary.record_bundle_edits(edits);
    assert_eq!(
        (summary.graph_edges_detached, summary.groups_removed),
        (1, 1)
    );
}

/// A one-node genome whose only edge is the sole surviving member of a
/// fact-bearing pair gene: a fact member when `fact`, else a direction
/// member.
fn sole_member_genome(fact: bool) -> CreatureGenome {
    let area = InputReference::World(WorldInputKey::AreaOccupancySummary);
    let layout = area.direction_pairs()[0];
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    let id = def.add_pair_group(0, 0, layout, DirectedVoteKind::Move, 3, layout.facts);
    def.group_mut(id).set_scale(0.5);
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
    assert_eq!(def.is_fact_member(kept), fact);
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![area],
            backend_def: BackendDef::Graph(def),
            targets: vec![],
        }],
    }
}

/// The `genome_size` change of the record and member a sole member's detach
/// turns into one independent edge: `+1 − 1`, the record's unit becomes
/// the edge's (T25.F10).
const SOLE_MEMBER_DETACH_DELTA: i64 = 0;

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
                let (result, edits, _) = apply_graph_event(
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

/// The thirteen attribute operators (T25.F09 adds the three edge retention
/// operators, `retention.rs`) report the scope their committed change
/// landed on (T25.F08 invariant 8): `Group` when a member was picked and
/// the coin was true (the change reaches every admitted member), else
/// `Edge` (one edge changed: a member with a false coin, or a non-member).
/// Creation and detach report none.
#[test]
fn a_committed_attribute_change_reports_the_scope_it_landed_on() {
    let (genome, idx, _) = grouped_founder();
    let all = every_node(&genome);
    let plasticity = |genome: &CreatureGenome| -> Vec<_> {
        genome
            .nodes
            .iter()
            .filter_map(|node| match &node.backend_def {
                BackendDef::Graph(def) => Some(def),
                BackendDef::Vm(_) => None,
            })
            .flat_map(|def| {
                def.edge_sites()
                    .map(|at| {
                        let edge = def.edge_at(at);
                        (edge.plasticity(), edge.membership())
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    };
    // EnableHebbian: (member, true coin), (member, false coin), non-member.
    let mut seen = [false; 3];
    for seed in 0..256 {
        let mut changed = genome.clone();
        let (result, _, scope) = apply_graph_event(
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
            Some(Scope::Group) => {
                assert_eq!(diffs.len(), 8, "every member, seed {seed}");
                seen[0] = true;
            }
            Some(Scope::Edge) => {
                assert_eq!(diffs.len(), 1, "seed {seed}");
                seen[if diffs[0].is_some() { 1 } else { 2 }] = true;
            }
            None => panic!("an attribute change reports its scope, seed {seed}"),
        }
    }
    assert_eq!(seen, [true; 3], "every case drawn");

    // SetScope: `Group` flips every member, `Edge` one.
    let mut scopes = [false; 2];
    for seed in 0..64 {
        let mut changed = genome.clone();
        let (result, _, scope) = apply_graph_event(
            &mut changed,
            GraphOperator::SetScope,
            &mut TargetSelector::reachable_only(&all, 0.0),
            &mut SmallRng::seed_from_u64(seed),
            &MutationConfig::default(),
            &crate::config::test_sensor_context(2),
        );
        assert!(result.is_ok());
        let def = graph_mut(&mut changed.nodes[idx]);
        let group_scope = def
            .members(GroupId::new(0))
            .filter(|&at| def.edge_at(at).membership().unwrap().scope == Scope::Group)
            .count();
        let scope = scope.expect("SetScope reports its scope");
        assert_eq!(group_scope, if scope == Scope::Group { 8 } else { 1 });
        scopes[scope.index()] = true;
    }
    assert_eq!(scopes, [true; 2]);

    for op in [
        GraphOperator::AddRingGroup,
        GraphOperator::AddRandomizedRingGroup,
        GraphOperator::DetachGraphEdge,
        GraphOperator::AlterGraphEdgeWeight,
    ] {
        let mut changed = genome.clone();
        let (result, _, scope) = apply_graph_event(
            &mut changed,
            op,
            &mut TargetSelector::reachable_only(&all, 0.0),
            &mut SmallRng::seed_from_u64(1),
            &MutationConfig::default(),
            &crate::config::test_sensor_context(2),
        );
        assert!(result.is_ok(), "{op:?}");
        assert_eq!(scope, None, "{op:?} reports no attribute scope");
    }

    let mut summary = MutationSummary::zero();
    summary.record_attribute_scope(Scope::Group);
    summary.record_attribute_scope(Scope::Edge);
    summary.record_attribute_scope(Scope::Group);
    assert_eq!(summary.attribute_changes_by_scope, [1, 2]);
}

/// A summary accumulates every committed event's detaches and removals.
#[test]
fn record_bundle_edits_accumulates_across_events() {
    let mut summary = MutationSummary::zero();
    summary.record_bundle_edits(BundleEdits {
        detached: 2,
        removed: 1,
        ..BundleEdits::default()
    });
    summary.record_bundle_edits(BundleEdits {
        detached: 3,
        removed: 4,
        ..BundleEdits::default()
    });
    assert_eq!(summary.graph_edges_detached, 5);
    assert_eq!(summary.groups_removed, 5);
}

/// The `genome_size` change of one applied group-touching event, read from
/// the target node before and after.
fn expected_delta(
    op: MutationOperator,
    before: &CgpGraphBackendDef,
    after: &CgpGraphBackendDef,
) -> Option<Vec<i64>> {
    match op {
        MutationOperator::GraphAddRingGroup
        | MutationOperator::GraphAddRandomizedRingGroup
        | MutationOperator::GraphAddDirectionPairGroup
        | MutationOperator::GraphAddRandomizedDirectionPairGroup => Some(vec![1]),
        MutationOperator::GraphDetachGraphEdge => {
            Some(vec![if after.groups().len() < before.groups().len() {
                0
            } else {
                1
            }])
        }
        MutationOperator::GraphRemoveGroup => Some(vec![-1]),
        MutationOperator::GraphRemoveGraphEdge if after.groups().len() < before.groups().len() => {
            Some(vec![-1])
        }
        MutationOperator::GraphRemoveGraphEdge if member_removed(before, after) => Some(vec![0]),
        MutationOperator::GraphAlterGroupScale
        | MutationOperator::GraphShiftGroupOffset
        | MutationOperator::GraphRetargetGroupKind
        | MutationOperator::GraphEnableRetention
        | MutationOperator::GraphDisableRetention
        | MutationOperator::GraphMutateRetention
        | MutationOperator::GraphEnableGroupRetention
        | MutationOperator::GraphDisableGroupRetention
        | MutationOperator::GraphMutateGroupRetention => Some(vec![0]),
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

/// The shape of one generated group.
#[derive(Clone, Copy, Debug)]
enum GroupShape {
    /// A `Ring` group on an appended occupied-ring reference.
    Ring,
    /// A `DirectionPair` gene on occupancy pair `pair`; `with_facts` adds its
    /// fact members (T25.F10). The last member of a fact-bearing gene is a
    /// fact member (the drive sinks follow every direction sink), so the
    /// drops generate fact holes and fact-only survivors as well as sole
    /// direction members.
    Pair { pair: u8, with_facts: bool },
}

/// One generated group: kind, offset, scale, the members to drop (a bit per
/// member in canonical order, the last always kept) and the shape.
type GroupSpec = (usize, u8, f32, u16, GroupShape);

/// Draws a `GroupSpec` over every shape: a ring, or each occupancy pair with
/// and without its fact members.
fn group_spec() -> impl Strategy<Value = GroupSpec> {
    let pairs = InputReference::World(WorldInputKey::AreaOccupancySummary)
        .direction_pairs()
        .len();
    let pair = (0..u8::try_from(pairs).unwrap(), any::<bool>())
        .prop_map(|(pair, with_facts)| GroupShape::Pair { pair, with_facts });
    let shape = prop_oneof![1 => Just(GroupShape::Ring), 4 => pair];
    (0usize..3, 0u8..8, -2.0f32..2.0, any::<u16>(), shape)
}

/// Adds one group per entry of `groups` to the Graph `node`: a `Ring` reads
/// an appended occupied-ring reference, a `DirectionPair` an appended
/// occupancy summary; each with members dropped (a group keeps at least
/// one).
fn add_groups(node: &mut NodeGenome, groups: &[GroupSpec]) {
    let ring = u16::try_from(node.input_refs.len()).unwrap();
    node.input_refs
        .push(InputReference::World(WorldInputKey::NeighborOccupiedRing));
    let area = InputReference::World(WorldInputKey::AreaOccupancySummary);
    let summary = ring + 1;
    node.input_refs.push(area.clone());
    let def = graph_mut(node);
    for &(kind, offset, scale, drop, shape) in groups {
        let kind = DirectedVoteKind::ALL[kind];
        let id = match shape {
            GroupShape::Ring => def.add_group(Shape::Ring { ring_ref: ring }, kind, offset),
            GroupShape::Pair { pair, with_facts } => {
                let layout = area.direction_pairs()[usize::from(pair)];
                let facts = if with_facts { layout.facts } else { &[] };
                def.add_pair_group(summary, pair, layout, kind, offset, facts)
            }
        };
        def.group_mut(id).set_scale(scale);
        let members: Vec<_> = def.members(id).collect();
        let last = members.len() - 1;
        for (index, occurrence) in members.into_iter().enumerate().rev() {
            if index < last && drop & (1 << index) != 0 {
                def.remove_edge(occurrence);
            }
        }
    }
}

/// The founder with ring groups on both Graph nodes — node 1 is routed from
/// node 0, so Topology copies and swaps reach a grouped backend — and an
/// appended VM node routed from node 1, so VM events have a target.
fn grouped_mixed_genome(groups: &[Vec<GroupSpec>; 2]) -> CreatureGenome {
    let mut genome = v3alpha1_founder_genome();
    let graphs: Vec<usize> = (0..genome.nodes.len())
        .filter(|&i| matches!(genome.nodes[i].backend_def, BackendDef::Graph(_)))
        .collect();
    assert_eq!(graphs.len(), 2, "the founder carries two Graph nodes");
    for (&idx, groups) in graphs.iter().zip(groups) {
        add_groups(&mut genome.nodes[idx], groups);
    }
    let vm_id = NodeId::new(genome.nodes.iter().map(|n| n.node_id.0).max().unwrap() + 1);
    genome.nodes[graphs[1]].targets.push(RouteTarget {
        target_id: vm_id,
        slot: 0,
        gate_bias: 0.0,
    });
    genome.nodes.push(NodeGenome {
        node_id: vm_id,
        input_refs: vec![],
        backend_def: BackendDef::Vm(VmBackendDef {
            register_count: 2,
            constants: vec![1.0],
            program: vec![VmInstruction::Halt],
        }),
        targets: vec![],
    });
    genome
}

proptest! {
    /// Random grouped genomes (two grouped Graph nodes and a VM node) under
    /// random events of all four domains, the mesh layer's share drawn over
    /// [0, 1]: every applied event leaves every Graph backend valid, the gate
    /// never rejects one, and each group operator changes `genome_size` by
    /// its declared amount under the unit rule (T25.F10): every creation +1,
    /// `DetachGraphEdge` +1 or 0 when the record goes, `RemoveGroup` −1, a
    /// member's `RemoveGraphEdge` 0 or −1 when the record goes, the record
    /// edits 0).
    #[test]
    fn every_applied_event_keeps_the_graph_invariants(
        seed in any::<u64>(),
        mesh_layer_probability in 0.0f64..=1.0,
        groups in [
            prop::collection::vec(group_spec(), 0..4),
            prop::collection::vec(group_spec(), 0..4),
        ],
    ) {
        let mut genome = grouped_mixed_genome(&groups);
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
            );
            // Every committed attribute change reports one scope (T25.F08).
            let attribute_events: u32 = ATTRIBUTE_OPERATORS
                .iter()
                .filter_map(|op| summary.applied_by_operator.get(&op.operator()))
                .sum();
            prop_assert_eq!(summary.attribute_changes_by_scope.iter().sum::<u32>(), attribute_events);
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
                &before.nodes[target].backend_def,
                genome.nodes.iter().find(|n| n.node_id == before.nodes[target].node_id).map(|n| &n.backend_def),
            ) else {
                continue;
            };
            if let Some(allowed) = expected_delta(op, old, new) {
                let delta = i64::from(genome.genome_size()) - i64::from(before.genome_size());
                prop_assert!(allowed.contains(&delta), "{:?}: {} not in {:?}", op, delta, allowed);
            }
            let removed = before.nodes.iter().map(|n| match &n.backend_def {
                BackendDef::Graph(d) => d.groups().len(),
                BackendDef::Vm(_) => 0,
            }).sum::<usize>().saturating_sub(genome.nodes.iter().map(|n| match &n.backend_def {
                BackendDef::Graph(d) => d.groups().len(),
                BackendDef::Vm(_) => 0,
            }).sum::<usize>());
            if matches!(
                op,
                MutationOperator::GraphRemoveGroup
                    | MutationOperator::GraphRemoveGraphEdge
                    | MutationOperator::GraphDetachGraphEdge
            ) {
                prop_assert_eq!(summary.groups_removed as usize, removed);
            }
            if op == MutationOperator::GraphDetachGraphEdge {
                prop_assert_eq!(summary.graph_edges_detached, 1);
            }
        }
    }
}
