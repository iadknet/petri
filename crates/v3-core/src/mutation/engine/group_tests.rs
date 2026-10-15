//! T25.F03: the mutation gate validates every Graph backend after every
//! applied event of any domain, and the group operators change
//! `genome_size` by their declared amounts.

use proptest::prelude::*;
use rand::rngs::SmallRng;
use rand::SeedableRng;

use super::*;
use crate::contracts::{InputReference, WorldInputKey};
use crate::creature::founder::v3alpha1_founder_genome;
use crate::creature::genome::cgp::{
    test_support::edit_sinks, CgpGraphBackendDef, GroupId, OutputSinkKind, Shape,
};
use crate::creature::genome::vote::{DirectedVoteKind, VoteSink};
use crate::creature::genome::NodeGenome;

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
        let (result, edits) = apply_graph_event(
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
        let (result, edits) = apply_graph_event(
            &mut rolled_back,
            GraphOperator::RemoveGroup,
            &mut TargetSelector::reachable_only(&all, 0.0),
            &mut SmallRng::seed_from_u64(seed),
            &config,
            2,
        );
        assert_eq!(result, Err(MutationSkipReason::GraphInvariantViolation));
        assert_eq!(edits, GroupEdits::default());
        assert_eq!(rolled_back, broken);
    }
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

proptest! {
    /// Random grouped genomes under random events of all four domains: every
    /// applied event leaves every Graph backend valid, the gate never rejects
    /// one, and each group operator changes `genome_size` by its declared
    /// amount (`AddRingGroup` +9, `RemoveGroup` −(n + 1), a last member's
    /// `RemoveGraphEdge` −2, the three record edits 0).
    #[test]
    fn every_applied_event_keeps_the_graph_invariants(
        seed in any::<u64>(),
        groups in prop::collection::vec((0usize..3, 0u8..8, -2.0f32..2.0, 0u8..=u8::MAX), 0..4),
    ) {
        let mut genome = v3alpha1_founder_genome();
        let idx = graph_index(&genome);
        {
            let node = &mut genome.nodes[idx];
            let ring = u16::try_from(node.input_refs.len()).unwrap();
            node.input_refs.push(InputReference::World(WorldInputKey::NeighborOccupiedRing));
            let def = graph_mut(node);
            for (kind, offset, scale, drop) in groups {
                let id = def.add_group(Shape::Ring { ring_ref: ring }, DirectedVoteKind::ALL[kind], offset);
                def.group_mut(id).set_scale(scale);
                let members: Vec<_> = def.members(id).collect();
                // Drop up to seven members; a group keeps at least one.
                for (channel, occurrence) in members.into_iter().enumerate().rev().skip(1) {
                    if drop & (1 << channel) != 0 {
                        def.remove_edge(occurrence);
                    }
                }
            }
        }
        let config = MutationConfig {
            per_unit_rate: 1.0,
            mesh_layer_probability: 0.0,
            ..MutationConfig::default()
        };
        let mut rng = SmallRng::seed_from_u64(seed);
        for _ in 0..24 {
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
