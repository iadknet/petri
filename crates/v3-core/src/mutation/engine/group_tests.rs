//! T25.F03: the mutation gate validates every Graph backend after every
//! applied event of any domain, and the group operators change
//! `genome_size` by their declared amounts.

use proptest::prelude::*;
use rand::rngs::SmallRng;
use rand::SeedableRng;

use super::*;
use crate::contracts::{InputReference, NodeId, RouteTarget, WorldInputKey};
use crate::creature::founder::v3alpha1_founder_genome;
use crate::creature::genome::cgp::{
    test_support::edit_sinks, CgpGraphBackendDef, GroupId, OutputSinkKind, Shape,
};
use crate::creature::genome::vote::{DirectedVoteKind, VoteSink};
use crate::creature::genome::{NodeGenome, VmBackendDef, VmInstruction};
use crate::creature::parseability::ParseabilityGate;

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
                if sink.kind == OutputSinkKind::ActionVote(VoteSink::Move(2)) {
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
            2,
        );
        assert!(result.is_ok());
        assert_eq!(edits.groups_removed, 1);

        let mut rolled_back = broken.clone();
        let (result, edits, refined) = apply_graph_event(
            &mut rolled_back,
            GraphOperator::RemoveGroup,
            &mut TargetSelector::reachable_only(&all, 0.0),
            &mut SmallRng::seed_from_u64(seed),
            &config,
            2,
        );
        assert_eq!(result, Err(MutationSkipReason::GraphInvariantViolation));
        assert_eq!(edits, GroupEdits::default());
        assert_eq!(refined, None);
        assert_eq!(rolled_back, broken);
    }
}

/// A committed `RefineHeritableStructure` reports the stepped set's key and
/// the summary counts it once per application (T25.F04).
#[test]
fn a_committed_refinement_reports_its_key() {
    use crate::creature::genome::cgp::{GraphEdge, GraphSource};
    let mut genome = v3alpha1_founder_genome();
    let idx = graph_index(&genome);
    let node = &mut genome.nodes[idx];
    let core = u16::try_from(node.input_refs.len()).unwrap();
    node.input_refs
        .push(InputReference::World(WorldInputKey::NearbyCreature));
    let def = graph_mut(node);
    let sink = def
        .sink_surface(OutputSinkKind::ActionVote(VoteSink::Eat))
        .unwrap();
    for slot in 0..2u16 {
        let source = GraphSource::InputLeaf {
            ref_idx: core,
            sub_idx: 9 * slot + 3,
        };
        def.push_edge(sink, GraphEdge::new(source, 0.25));
    }
    let config = MutationConfig {
        structured_heritable_refinement: true,
        ..MutationConfig::default()
    };
    let all: Vec<usize> = (0..genome.nodes.len()).collect();
    let (result, _, refined) = apply_graph_event(
        &mut genome,
        GraphOperator::RefineHeritableStructure,
        &mut TargetSelector::reachable_only(&all, 0.0),
        &mut SmallRng::seed_from_u64(3),
        &config,
        2,
    );
    assert!(result.is_ok());
    let key = crate::mutation::types::RefinementKey {
        family: WorldInputKey::NearbyCreature,
        field: 3,
    };
    assert_eq!(refined, Some(key));
    assert_eq!(key.as_key(), "NearbyCreature:3");
    let mut summary = MutationSummary::zero();
    summary.record_refinement(key);
    summary.record_refinement(key);
    assert_eq!(summary.refinement_applications_by_key.get(&key), Some(&2));
}

#[test]
fn group_edits_count_detaches_and_removals_by_operator() {
    let edits = |op, before, after| group_edits(op, before, after);
    assert_eq!(
        edits(GraphOperator::RetargetGraphEdge, (8, 1), (7, 1)),
        GroupEdits {
            detached: 1,
            groups_removed: 0
        }
    );
    assert_eq!(
        edits(GraphOperator::GraphRawFieldMutation, (1, 1), (0, 0)),
        GroupEdits {
            detached: 1,
            groups_removed: 1
        }
    );
    assert_eq!(
        edits(GraphOperator::RemoveGraphEdge, (1, 1), (0, 0)),
        GroupEdits {
            detached: 0,
            groups_removed: 1
        }
    );
    assert_eq!(
        edits(GraphOperator::RemoveGroup, (16, 2), (8, 1)),
        GroupEdits {
            detached: 0,
            groups_removed: 1
        }
    );
    assert_eq!(
        edits(GraphOperator::AddRingGroup, (0, 0), (8, 1)),
        GroupEdits::default()
    );
}

/// The tallies count every member edge and every group record: one ring
/// group is eight members, a second doubles both.
#[test]
fn group_tallies_count_members_and_group_records() {
    let (mut genome, idx, ring) = grouped_founder();
    assert_eq!(group_tallies(&genome), (8, 1));
    graph_mut(&mut genome.nodes[idx]).add_group(
        Shape::Ring { ring_ref: ring },
        DirectedVoteKind::Move,
        5,
    );
    assert_eq!(group_tallies(&genome), (16, 2));
}

/// A summary accumulates every committed event's detaches and removals.
#[test]
fn record_group_edits_accumulates_across_events() {
    let mut summary = MutationSummary::zero();
    summary.record_group_edits(GroupEdits {
        detached: 2,
        groups_removed: 1,
    });
    summary.record_group_edits(GroupEdits {
        detached: 3,
        groups_removed: 4,
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
    let member_counts: Vec<i64> = (0..before.groups().len())
        .map(|i| before.members(GroupId::new(i as u16)).count() as i64)
        .collect();
    match op {
        MutationOperator::GraphAddRingGroup => Some(vec![9]),
        MutationOperator::GraphAddDirectionPairGroup => Some(vec![13]),
        MutationOperator::GraphRemoveGroup => {
            Some(member_counts.iter().map(|n| -(n + 1)).collect())
        }
        MutationOperator::GraphRemoveGraphEdge if after.groups().len() < before.groups().len() => {
            Some(vec![-2])
        }
        MutationOperator::GraphAlterGroupScale
        | MutationOperator::GraphShiftGroupOffset
        | MutationOperator::GraphRetargetGroupKind => Some(vec![0]),
        _ => None,
    }
}

/// One generated group: kind, offset, scale, the members to drop (a bit per
/// member, the first always kept) and the shape (0 a `Ring`, 1 or 2 a
/// `DirectionPair` on that pair index minus one).
type GroupSpec = (usize, u8, f32, u16, u8);

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
            0 => def.add_group(Shape::Ring { ring_ref: ring }, kind, offset),
            _ => {
                let pair = shape - 1;
                let layout = area.direction_pairs()[usize::from(pair)];
                def.add_pair_group(summary, pair, layout, kind, offset)
            }
        };
        def.group_mut(id).set_scale(scale);
        let members: Vec<_> = def.members(id).collect();
        for (index, occurrence) in members.into_iter().enumerate().rev() {
            if index > 0 && drop & (1 << index) != 0 {
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
    /// its declared amount (`AddRingGroup` +9, `AddDirectionPairGroup` +13,
    /// `RemoveGroup` −(n + 1), a last
    /// member's `RemoveGraphEdge` −2, the three record edits 0).
    #[test]
    fn every_applied_event_keeps_the_graph_invariants(
        seed in any::<u64>(),
        mesh_layer_probability in 0.0f64..=1.0,
        groups in [
            prop::collection::vec((0usize..3, 0u8..8, -2.0f32..2.0, any::<u16>(), 0u8..3), 0..4),
            prop::collection::vec((0usize..3, 0u8..8, -2.0f32..2.0, any::<u16>(), 0u8..3), 0..4),
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
                2,
            );
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
            if matches!(op, MutationOperator::GraphRemoveGroup | MutationOperator::GraphRemoveGraphEdge) {
                prop_assert_eq!(summary.groups_removed as usize, removed);
            }
        }
    }
}
