//! T11.F24: within one birth, a child node is a parent-set member exactly when
//! the parent carried it and the parent's set contained it, however earlier
//! events of the same birth removed or added mesh nodes.

use std::collections::BTreeSet;

use proptest::prelude::*;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use super::*;
use crate::config::SimulationConfig;
use crate::contracts::{InputReference, RouteTarget, WorldInputKey};
use crate::creature::founder::v3alpha1_founder_genome;
use crate::creature::genome::analysis::mesh_reachable_nodes;
use crate::creature::genome::{NodeGenome, VmBackendDef, VmInstruction};
use crate::mutation::MutationEventRecord;

fn rng(seed: u64) -> SmallRng {
    SmallRng::seed_from_u64(seed)
}

fn vm_node(id: u32, constant: f32, target: Option<u32>) -> NodeGenome {
    NodeGenome {
        node_id: NodeId::new(id),
        input_refs: vec![InputReference::World(WorldInputKey::FoodHere {
            type_idx: crate::config::OrdinaryFoodTypeId::default(),
        })],
        backend_def: BackendDef::Vm(VmBackendDef {
            register_count: 4,
            constants: vec![constant],
            program: vec![
                VmInstruction::LoadConst {
                    dst: 0,
                    const_idx: 0,
                },
                VmInstruction::Halt,
            ],
        }),
        targets: target.map_or_else(Vec::new, |id| {
            vec![RouteTarget {
                target_id: NodeId::new(id),
                slot: 0,
                gate_bias: 0.0,
            }]
        }),
    }
}

/// Entry `a` routes to `c`; `b` sits between them in the node vector and is
/// disconnected, so it is the only node `RemoveNode` can take.
fn entry_junk_core(a: u32, b: u32, c: u32) -> CreatureGenome {
    CreatureGenome {
        entry_node_id: NodeId::new(a),
        nodes: vec![
            vm_node(a, 1.0, Some(c)),
            vm_node(b, 2.0, None),
            vm_node(c, 3.0, None),
        ],
    }
}

fn config(mesh_layer_probability: f64, executed_bias: f64) -> MutationConfig {
    MutationConfig {
        per_unit_rate: 1.0,
        mesh_layer_probability,
        executed_bias,
        ..SimulationConfig::default().mutation
    }
}

struct Birth {
    genome: CreatureGenome,
    summary: MutationSummary,
    rng_tail: u64,
}

fn birth(
    parent: &CreatureGenome,
    config: &MutationConfig,
    events: u32,
    reachable: &[usize],
    executed: &[usize],
    seed: u64,
) -> Birth {
    let mut genome = parent.clone();
    let mut random = rng(seed);
    let summary = MutationEngine::apply_mutations_on_units(
        &mut genome,
        events,
        config,
        reachable,
        ParentExecuted::Indices(executed),
        &mut random,
        1,
    );
    Birth {
        genome,
        summary,
        rng_tail: random.gen(),
    }
}

fn applied_removal_of(event: &MutationEventRecord, id: u32) -> bool {
    event.operator == Some(MutationOperator::TopologyRemoveNode)
        && event.outcome.is_applied()
        && event.target == Some(NodeId::new(id))
}

fn targeted_class(event: &MutationEventRecord) -> Option<TargetReachability> {
    match event.outcome {
        MutationEventOutcome::Applied(class) if class != TargetReachability::NotApplicable => {
            event.target.map(|_| class)
        }
        _ => None,
    }
}

/// What the membership rule predicts for one birth, from its event records
/// alone: each applied event's first-target class and the executed-target
/// count. A node is a member when the parent carried its id, no earlier
/// applied `RemoveNode` of this birth removed that id (so a later node on the
/// same id is newly created), and the parent's set holds its parent index.
struct Expected {
    classes: Vec<Option<TargetReachability>>,
    reachable_targets: u32,
    unreachable_targets: u32,
    executed_targets: u32,
}

fn expected_telemetry(
    parent: &CreatureGenome,
    reachable: &[usize],
    executed: &[usize],
    events: &[MutationEventRecord],
) -> Expected {
    let mut removed: BTreeSet<NodeId> = BTreeSet::new();
    let mut expected = Expected {
        classes: Vec::new(),
        reachable_targets: 0,
        unreachable_targets: 0,
        executed_targets: 0,
    };
    for event in events {
        let member = |id: NodeId, set: &[usize]| {
            !removed.contains(&id)
                && parent
                    .nodes
                    .iter()
                    .position(|node| node.node_id == id)
                    .is_some_and(|index| set.binary_search(&index).is_ok())
        };
        let class = targeted_class(event).map(|_| {
            let id = event.target.expect("a targeted class names its target");
            if member(id, reachable) {
                TargetReachability::Reachable
            } else {
                TargetReachability::Unreachable
            }
        });
        match class {
            Some(TargetReachability::Reachable) => expected.reachable_targets += 1,
            Some(_) => expected.unreachable_targets += 1,
            None => {}
        }
        expected.classes.push(class);
        if event.outcome.is_applied() {
            if let Some(id) = event.target {
                expected.executed_targets += u32::from(member(id, executed));
            }
            if event.operator == Some(MutationOperator::TopologyRemoveNode) {
                removed.extend(event.target);
            }
        }
    }
    expected
}

fn assert_truthful(
    parent: &CreatureGenome,
    reachable: &[usize],
    executed: &[usize],
    summary: &MutationSummary,
) {
    let expected = expected_telemetry(parent, reachable, executed, &summary.events);
    let recorded: Vec<_> = summary.events.iter().map(targeted_class).collect();
    assert_eq!(recorded, expected.classes, "{:?}", summary.events);
    assert_eq!(
        summary.executed_target_events, expected.executed_targets,
        "{:?}",
        summary.events
    );
    // Events that applied without a class (`NotApplicable`) never enter the
    // targeted totals, so only the class-carrying ones are compared.
    let untargeted_reachable = summary
        .events
        .iter()
        .filter(|event| {
            event.outcome == MutationEventOutcome::Applied(TargetReachability::Reachable)
                && event.target.is_none()
        })
        .count();
    let untargeted_unreachable = summary
        .events
        .iter()
        .filter(|event| {
            event.outcome == MutationEventOutcome::Applied(TargetReachability::Unreachable)
                && event.target.is_none()
        })
        .count();
    assert_eq!(
        summary.reachable_target_events as usize,
        expected.reachable_targets as usize + untargeted_reachable
    );
    assert_eq!(
        summary.unreachable_target_events as usize,
        expected.unreachable_targets as usize + untargeted_unreachable
    );
}

/// The first seed in `0..limit` whose birth satisfies `shape`.
fn first_birth(
    limit: u64,
    mut run: impl FnMut(u64) -> Birth,
    shape: impl Fn(&Birth) -> bool,
) -> (u64, Birth) {
    (0..limit)
        .map(|seed| (seed, run(seed)))
        .find(|(_, birth)| shape(birth))
        .expect("no seed produced the birth shape")
}

#[test]
fn a_carried_node_after_the_removed_index_keeps_its_membership() {
    // Parent sets [0, 2]: A and C are members, B is not. Removing B moves C
    // to index 1, which the parent's index set does not hold.
    let parent = entry_junk_core(0, 1, 2);
    let (reachable, executed) = ([0, 2], [0, 2]);
    let config = config(0.5, 0.5);
    let (seed, found) = first_birth(
        20_000,
        |seed| birth(&parent, &config, 2, &reachable, &executed, seed),
        |birth| {
            let events = &birth.summary.events;
            applied_removal_of(&events[0], 1)
                && events[1].domain == MutationDomain::Vm
                && events[1].outcome.is_applied()
                && events[1].target == Some(NodeId::new(2))
        },
    );
    let events = &found.summary.events;
    assert_eq!(
        events[0].outcome,
        MutationEventOutcome::Applied(TargetReachability::Unreachable),
        "seed {seed}"
    );
    assert_eq!(
        events[1].outcome,
        MutationEventOutcome::Applied(TargetReachability::Reachable),
        "seed {seed}: C is still the parent's member at its new index"
    );
    assert_eq!(found.summary.reachable_target_events, 1);
    assert_eq!(found.summary.unreachable_target_events, 1);
    assert_eq!(
        found.summary.executed_target_events, 1,
        "seed {seed}: C is the parent's executed node, B is not"
    );
    assert_eq!(found.genome.nodes[1].node_id, NodeId::new(2));
    assert_truthful(&parent, &reachable, &executed, &found.summary);
}

#[test]
fn sparse_ids_keep_membership_through_a_removal() {
    // The same shape on non-contiguous ids that are not in index order.
    let parent = entry_junk_core(40, 7, 19);
    let (reachable, executed) = ([0, 2], [0, 2]);
    let config = config(0.5, 0.5);
    let (seed, found) = first_birth(
        20_000,
        |seed| birth(&parent, &config, 2, &reachable, &executed, seed),
        |birth| {
            let events = &birth.summary.events;
            applied_removal_of(&events[0], 7)
                && events[1].domain == MutationDomain::Vm
                && events[1].outcome.is_applied()
                && events[1].target == Some(NodeId::new(19))
        },
    );
    assert_eq!(
        found.summary.events[1].outcome,
        MutationEventOutcome::Applied(TargetReachability::Reachable),
        "seed {seed}"
    );
    assert_eq!(found.summary.executed_target_events, 1, "seed {seed}");
    assert_truthful(&parent, &reachable, &executed, &found.summary);
}

#[test]
fn a_node_created_on_a_removed_id_and_index_is_never_a_member() {
    // Entry A routes to B; C (the maximum id, at index 2) is disconnected and
    // is the only removable node. Removing it makes id 2 the next free id, so
    // a later added node takes C's id and C's former index, both of which the
    // parent's sets hold.
    let parent = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![
            vm_node(0, 1.0, Some(1)),
            vm_node(1, 2.0, None),
            vm_node(2, 3.0, None),
        ],
    };
    let (reachable, executed) = ([0, 1, 2], [0, 1, 2]);
    let config = config(1.0, 0.5);
    let (seed, found) = first_birth(
        200_000,
        |seed| birth(&parent, &config, 3, &reachable, &executed, seed),
        |birth| {
            let events = &birth.summary.events;
            applied_removal_of(&events[0], 2)
                && events[1].outcome.is_applied()
                && events[2].operator != Some(MutationOperator::TopologyRemoveNode)
                && targeted_class(&events[2]).is_some()
                && events[2].target == Some(NodeId::new(2))
        },
    );
    let events = &found.summary.events;
    assert_eq!(
        found.genome.nodes[2].node_id,
        NodeId::new(2),
        "seed {seed}: the created node reuses C's id at C's former index"
    );
    assert_eq!(
        events[2].outcome,
        MutationEventOutcome::Applied(TargetReachability::Unreachable),
        "seed {seed}: a node created in this birth is never a member"
    );
    let first_two_executed = u32::from(events[1].target.is_some_and(|id| id != NodeId::new(2)));
    assert_eq!(
        found.summary.executed_target_events,
        1 + first_two_executed,
        "seed {seed}: C's removal counts, the created node does not: {events:?}"
    );
    assert_truthful(&parent, &reachable, &executed, &found.summary);
}

#[test]
fn a_discarded_or_skipped_attempt_after_a_removal_leaves_membership_unchanged() {
    // After B leaves, `RemoveNode` has no site on {A -> C, C}, so mesh events
    // often discard it and retry; a node-internal domain can also run out of
    // operators and skip the whole event. Both restore the genome before the
    // next attempt, so C's membership must survive them.
    let parent = entry_junk_core(0, 1, 2);
    let (reachable, executed) = ([0, 2], [0, 2]);
    let config = config(0.5, 0.5);
    let shape = |birth: &Birth, skipped: bool| {
        let events = &birth.summary.events;
        let middle = if skipped {
            !events[1].outcome.is_applied() && !events[1].discarded.is_empty()
        } else {
            events[1].outcome.is_applied() && !events[1].discarded.is_empty()
        };
        applied_removal_of(&events[0], 1)
            && middle
            && events[2].outcome.is_applied()
            && events[2].target == Some(NodeId::new(2))
    };
    for skipped in [false, true] {
        let (seed, found) = first_birth(
            200_000,
            |seed| birth(&parent, &config, 3, &reachable, &executed, seed),
            |birth| shape(birth, skipped),
        );
        let events = &found.summary.events;
        assert_eq!(
            targeted_class(&events[2]),
            Some(TargetReachability::Reachable),
            "seed {seed}: {events:?}"
        );
        // Only an applied event's kept target tallies: B's removal adds
        // nothing, C adds one, and the middle event adds one only when it
        // applied on A or C. Discarded operators never tally.
        let middle = u32::from(
            events[1].outcome.is_applied()
                && matches!(events[1].target, Some(id) if id == NodeId::new(0) || id == NodeId::new(2)),
        );
        assert_eq!(
            found.summary.executed_target_events,
            1 + middle,
            "seed {seed}: {events:?}"
        );
        assert_truthful(&parent, &reachable, &executed, &found.summary);
    }
}

#[test]
fn an_all_executed_eligible_set_after_a_removal_takes_the_no_roll_short_circuit() {
    // After B leaves, the VM domain's eligible set is {A, C}, both executed by
    // the parent, so the executed layer consumes no RNG and the draw is the
    // plain reachable-biased draw. Executed bias 1.0 would otherwise always
    // land on the one executed index, A.
    let parent = entry_junk_core(0, 1, 2);
    let (reachable, executed) = ([0, 2], [0, 2]);
    let with_layer = config(0.5, 1.0);
    let plain = config(0.5, 0.0);
    let shape = |birth: &Birth| {
        let events = &birth.summary.events;
        applied_removal_of(&events[0], 1)
            && events[1].domain == MutationDomain::Vm
            && events[1].outcome.is_applied()
    };
    let mut reached_c = 0;
    for seed in 0..20_000 {
        let layered = birth(&parent, &with_layer, 2, &reachable, &executed, seed);
        if !shape(&layered) {
            continue;
        }
        // `gen_bool(1.0)` draws nothing, so the removal's draw matches too,
        // and the whole birth equals the executed-layer-free birth.
        let unlayered = birth(&parent, &plain, 2, &reachable, &executed, seed);
        assert_eq!(layered.genome, unlayered.genome, "seed {seed}");
        assert_eq!(
            layered.summary.events, unlayered.summary.events,
            "seed {seed}"
        );
        assert_eq!(layered.rng_tail, unlayered.rng_tail, "seed {seed}");
        assert_eq!(
            layered.summary.executed_target_events, 1,
            "seed {seed}: every event-1 target is executed"
        );
        assert_truthful(&parent, &reachable, &executed, &layered.summary);
        if layered.summary.events[1].target == Some(NodeId::new(2)) {
            assert_eq!(
                layered.summary.events[1].outcome,
                MutationEventOutcome::Applied(TargetReachability::Reachable)
            );
            reached_c += 1;
        }
    }
    assert!(
        reached_c > 0,
        "the short-circuit draw must sometimes reach C"
    );
}

/// A parent genome grown from the founder by `growth` rate-1.0 births.
fn grown_parent(seed: u64, growth: u8) -> CreatureGenome {
    let config = config(0.6, 0.5);
    let mut genome = v3alpha1_founder_genome();
    let mut random = rng(seed);
    for _ in 0..growth {
        MutationEngine::apply_mutations_on_units(
            &mut genome,
            4,
            &config,
            &[],
            ParentExecuted::NONE,
            &mut random,
            1,
        );
    }
    genome
}

fn executed_subset(parent: &CreatureGenome, mask: u64) -> Vec<usize> {
    (0..parent.nodes.len())
        .filter(|&index| mask >> (index % 64) & 1 == 1)
        .collect()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    /// Every applied event's class and the executed count follow the
    /// membership rule, over grown genomes and removal-heavy event sequences.
    #[test]
    fn applied_targets_follow_the_birth_membership_rule(
        genome_seed in any::<u64>(),
        growth in 0u8..6,
        mask in any::<u64>(),
        seed in any::<u64>(),
        events in 1u32..12,
        mesh_layer in 0.3f64..=1.0,
        executed_bias in 0.0f64..=1.0,
    ) {
        let parent = grown_parent(genome_seed, growth);
        let reachable = mesh_reachable_nodes(&parent);
        let executed = executed_subset(&parent, mask);
        let found = birth(
            &parent,
            &config(mesh_layer, executed_bias),
            events,
            &reachable,
            &executed,
            seed,
        );
        assert_truthful(&parent, &reachable, &executed, &found.summary);
    }
}

/// The same birth with membership frozen at the parent's index sets, which is
/// exactly the targeting before T11.F24.
fn frozen_birth(
    parent: &CreatureGenome,
    config: &MutationConfig,
    events: u32,
    reachable: &[usize],
    executed: &[usize],
    seed: u64,
) -> Birth {
    use crate::mutation::reachability::FROZEN_MEMBERSHIP;
    FROZEN_MEMBERSHIP.with(|frozen| frozen.set(true));
    let frozen = birth(parent, config, events, reachable, executed, seed);
    FROZEN_MEMBERSHIP.with(|frozen| frozen.set(false));
    frozen
}

/// Whether some event after the birth's first applied `RemoveNode` drew a
/// target: its kept selector or any discarded operator returned a node.
fn draws_after_a_removal(events: &[MutationEventRecord]) -> bool {
    events
        .iter()
        .position(|event| {
            event.operator == Some(MutationOperator::TopologyRemoveNode)
                && event.outcome.is_applied()
        })
        .is_some_and(|first| {
            events[first + 1..].iter().any(|event| {
                event.target.is_some() || event.discarded.iter().any(|(_, pick)| pick.is_some())
            })
        })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    /// A birth with no target draw after its first applied mesh-node removal
    /// is byte-identical to the pre-feature birth: genome, summary with its
    /// event records, and the RNG stream after it.
    #[test]
    fn births_without_a_draw_after_a_removal_are_unchanged(
        genome_seed in any::<u64>(),
        growth in 0u8..6,
        mask in any::<u64>(),
        seed in any::<u64>(),
        events in 1u32..12,
        mesh_layer in 0.0f64..=1.0,
        executed_bias in 0.0f64..=1.0,
    ) {
        let parent = grown_parent(genome_seed, growth);
        let reachable = mesh_reachable_nodes(&parent);
        let executed = executed_subset(&parent, mask);
        let config = config(mesh_layer, executed_bias);
        let current = birth(&parent, &config, events, &reachable, &executed, seed);
        let frozen = frozen_birth(&parent, &config, events, &reachable, &executed, seed);
        if !draws_after_a_removal(&frozen.summary.events) {
            prop_assert_eq!(&current.genome, &frozen.genome);
            prop_assert_eq!(&current.summary, &frozen.summary);
            prop_assert_eq!(current.rng_tail, frozen.rng_tail);
        }
    }
}

/// The within-event identity [`BirthMembership::observe_event`] relies on:
/// every topology operator, applied or rolled back, either leaves the node
/// ids as they were, removes exactly one node keeping the rest in order, or
/// appends nodes on ids the genome did not carry.
#[test]
fn every_topology_event_removes_one_node_or_appends_fresh_ones() {
    let config = config(1.0, 0.5);
    let mut changed = BTreeSet::new();
    for genome_seed in 0..24 {
        let parent = grown_parent(genome_seed, (genome_seed % 6) as u8);
        let reachable = mesh_reachable_nodes(&parent);
        for op in TopologyOperator::ALL {
            for seed in 0..40 {
                let mut genome = parent.clone();
                let before: Vec<NodeId> = genome.nodes.iter().map(|node| node.node_id).collect();
                let sets = crate::mutation::reachability::TargetSets::new(&reachable, &[]);
                let _ = apply_topology_event(
                    &mut genome,
                    op,
                    &mut sets.selector(0.5, 0.0),
                    &mut rng(seed),
                    &config,
                    1,
                );
                let after: Vec<NodeId> = genome.nodes.iter().map(|node| node.node_id).collect();
                let unique: BTreeSet<NodeId> = after.iter().copied().collect();
                assert_eq!(unique.len(), after.len(), "{op:?}: duplicate ids {after:?}");
                if after == before {
                    continue;
                }
                changed.insert(topology_operator_key(op));
                let one_removed = after.len() + 1 == before.len()
                    && (0..before.len()).any(|gone| {
                        before[..gone]
                            .iter()
                            .chain(&before[gone + 1..])
                            .eq(after.iter())
                    });
                let appended_fresh = after.len() > before.len()
                    && after[..before.len()] == before[..]
                    && after[before.len()..].iter().all(|id| !before.contains(id));
                assert!(
                    one_removed || appended_fresh,
                    "{op:?} seed {seed}: {before:?} -> {after:?}"
                );
            }
        }
    }
    for op in [
        MutationOperator::TopologyRemoveNode,
        MutationOperator::TopologyAddNode,
        MutationOperator::TopologySpliceNode,
        MutationOperator::TopologyCopyNode,
        MutationOperator::TopologyCopyMeshBackwardSlice,
        MutationOperator::TopologyCopyMeshForwardSlice,
    ] {
        assert!(
            changed.contains(&op),
            "{op:?} never changed the node vector"
        );
    }
}
