//! T25.F03 edge-group operators: catalog, applicability, pinned draw orders,
//! and the operators whose draws a group must not change.

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use crate::config::{MutationConfig, OrdinaryFoodTypeId};
use crate::contracts::votes::{DirectedVoteKind, VoteSink};
use crate::contracts::OutcomeChannel;
use crate::contracts::{InputReference, NodeId, WorldInputKey};
use crate::creature::founder::v3alpha1_founder_genome;
use crate::genome::cgp::test_support::edit_sinks;
use crate::genome::cgp::{
    compose, detach, validate_graph, BirthSeeds, CgpGraphBackendDef, EdgeOccurrence, EdgeSurface,
    GroupId, OutputSinkKind, Scope, Shape,
};
use crate::genome::{
    BackendDef, CreatureGenome, HebbianRule, NodeGenome, PlasticityConfig, RewardModulationConfig,
    VmBackendDef,
};
use crate::mutation::graph::{group, GraphMutator, GraphOperator};
use crate::mutation::input_ref::{InputRefMutator, InputRefOperator};
use crate::mutation::types::{ComplexityEffect, MutationSkipReason};

const GROUP_OPERATORS: [GraphOperator; 6] = [
    GraphOperator::AddRingGroup,
    GraphOperator::AddDirectionPairGroup,
    GraphOperator::AlterGroupScale,
    GraphOperator::ShiftGroupOffset,
    GraphOperator::RetargetGroupKind,
    GraphOperator::RemoveGroup,
];

fn barrier() -> InputReference {
    InputReference::World(WorldInputKey::NeighborBarrierRing)
}

fn food(t: u16) -> InputReference {
    InputReference::World(WorldInputKey::neighbor_food_ring(OrdinaryFoodTypeId::new(
        t,
    )))
}

fn area_barrier() -> InputReference {
    InputReference::World(WorldInputKey::AreaBarrierSummary)
}

fn area_food(t: u16) -> InputReference {
    InputReference::World(WorldInputKey::area_food_summary(OrdinaryFoodTypeId::new(t)))
}

/// The refs every grouped fixture reads: rings at 0 and 2.
fn refs() -> Vec<InputReference> {
    vec![barrier(), InputReference::ActionVotes, food(0)]
}

fn graph_node(refs: Vec<InputReference>, def: CgpGraphBackendDef) -> NodeGenome {
    NodeGenome {
        node_id: NodeId::new(0),
        input_refs: refs,
        backend_def: BackendDef::Graph(def),
        targets: vec![],
    }
}

fn graph(node: &NodeGenome) -> &CgpGraphBackendDef {
    match &node.backend_def {
        BackendDef::Graph(def) => def,
        BackendDef::Vm(_) => panic!("Graph fixture"),
    }
}

fn graph_mut(node: &mut NodeGenome) -> &mut CgpGraphBackendDef {
    match &mut node.backend_def {
        BackendDef::Graph(def) => def,
        BackendDef::Vm(_) => panic!("Graph fixture"),
    }
}

/// Three groups: Move at 0 (scale 0), Reproduce at 5 (0.5) and StealEnergy
/// at 7 (−2), on rings 0, 2, 0.
fn three() -> CgpGraphBackendDef {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    for (ring_ref, kind, offset, scale) in [
        (0, DirectedVoteKind::Move, 0, 0.0),
        (2, DirectedVoteKind::Reproduce, 5, 0.5),
        (0, DirectedVoteKind::StealEnergy, 7, -2.0),
    ] {
        let id = def.add_group(Shape::Ring { ring_ref }, kind, offset);
        def.group_mut(id).set_scale(scale);
    }
    def
}

fn id(index: usize) -> GroupId {
    GroupId::new(u16::try_from(index).unwrap())
}

const RULES: [HebbianRule; 4] = [
    HebbianRule::Classic,
    HebbianRule::Oja,
    HebbianRule::AntiHebb,
    HebbianRule::Covariance,
];

const CHANNELS: [OutcomeChannel; 4] = [
    OutcomeChannel::EnergyDelta,
    OutcomeChannel::ActionSuccess,
    OutcomeChannel::DamageDelta,
    OutcomeChannel::OffspringSuccess,
];

/// One learning draw sequence, replayed from `rng` in its pinned order
/// (T25.F07 invariant 7, T25.F08 invariant 2): a mode in `0..3`; for a
/// learning mode the rule, rate, clamp and inheritance of `EnableHebbian`,
/// then for mode 2 the channel and decay of `EnableRewardModulation`; then
/// one scope coin. Learned creation draws it once per group, the
/// randomized form once per member. Returns the mode, config and scope.
fn replay_member(rng: &mut SmallRng) -> (u8, Option<PlasticityConfig>, Scope) {
    let mode = rng.gen_range(0..3u8);
    let config = (mode > 0).then(|| {
        let rule = RULES[rng.gen_range(0..4)];
        let learning_rate = rng.gen_range(0.01f32..0.2);
        let weight_clamp = rng.gen_range(1.0f32..5.0);
        let lamarckian = rng.gen_bool(0.5);
        let modulation = (mode == 2).then(|| {
            let reward_source = CHANNELS[rng.gen_range(0..4)];
            RewardModulationConfig {
                reward_source,
                trace_decay: rng.gen_range(0.5f32..0.99),
            }
        });
        PlasticityConfig {
            rule,
            learning_rate,
            weight_clamp,
            lamarckian,
            modulation,
            retention: 1.0,
        }
    });
    let scope = if rng.gen_bool(0.5) {
        Scope::Group
    } else {
        Scope::Edge
    };
    (mode, config, scope)
}

/// Learned creation's draws on group `group` of `want`: one
/// [`replay_member`] sequence shared by every member, then one
/// `inherit_scale` coin. Returns the mode.
fn replay_learning(want: &mut CgpGraphBackendDef, group: GroupId, rng: &mut SmallRng) -> u8 {
    let (mode, config, scope) = replay_member(rng);
    let inherit_scale = rng.gen_bool(0.5);
    for member in want.members(group).collect::<Vec<_>>() {
        let mut edge = want.edge_mut(member);
        edge.set_plasticity(config);
        edge.set_scope(scope);
    }
    want.group_mut(group).set_inherit_scale(inherit_scale);
    mode
}

#[test]
fn the_catalog_keeps_six_operators_at_their_edge_tiers() {
    // T25.F09 appends the six retention operators (`retention.rs`).
    assert_eq!(GraphOperator::ALL.len(), 38);
    assert_eq!(crate::mutation::types::MutationOperator::all().len(), 71);
    assert_eq!(&GraphOperator::ALL[22..28], &GROUP_OPERATORS);
    // T25.F08: the randomized twins and `DetachGraphEdge` follow `SetScope`.
    assert_eq!(
        &GraphOperator::ALL[29..32],
        &[
            GraphOperator::AddRandomizedRingGroup,
            GraphOperator::AddRandomizedDirectionPairGroup,
            GraphOperator::DetachGraphEdge,
        ]
    );
    for (op, key, effect) in [
        (
            GraphOperator::AddRandomizedRingGroup,
            "Graph.AddRandomizedRingGroup",
            ComplexityEffect::Increasing,
        ),
        (
            GraphOperator::AddRandomizedDirectionPairGroup,
            "Graph.AddRandomizedDirectionPairGroup",
            ComplexityEffect::Increasing,
        ),
        (
            GraphOperator::DetachGraphEdge,
            "Graph.DetachGraphEdge",
            ComplexityEffect::Neutral,
        ),
    ] {
        assert_eq!(op.weight(), 2, "{op:?}");
        assert_eq!(op.operator().as_key(), key);
        assert_eq!(op.complexity_effect(), effect, "{op:?}");
        assert!(op.enabled(&MutationConfig::default()), "{op:?}");
    }
    let enabled: u16 = GraphOperator::ALL
        .iter()
        .filter(|op| op.enabled(&MutationConfig::default()))
        .map(|op| u16::from(op.weight()))
        .sum();
    assert_eq!(enabled, 79, "the engine's enabled Graph weight at defaults");
    // T25.F07: `SetScope` follows `RemoveGroup`, weight 2, neutral.
    assert_eq!(GraphOperator::ALL[28], GraphOperator::SetScope);
    assert_eq!(GraphOperator::SetScope.weight(), 2);
    assert_eq!(
        GraphOperator::SetScope.complexity_effect(),
        ComplexityEffect::Neutral
    );
    assert_eq!(
        GraphOperator::SetScope.operator().as_key(),
        "Graph.SetScope"
    );
    assert_eq!(
        GraphOperator::SetScope.operator().complexity_effect(),
        ComplexityEffect::Neutral
    );
    let weights = GROUP_OPERATORS.map(GraphOperator::weight);
    assert_eq!(weights, [2, 2, 4, 2, 2, 2]);
    let default_arm: u16 = GraphOperator::ALL
        .iter()
        .filter(|&&op| op != GraphOperator::RecruitNeutralInput)
        .map(|op| u16::from(op.weight()))
        .sum();
    assert_eq!(default_arm, 78);
    let keys = GROUP_OPERATORS.map(|op| op.operator().as_key());
    assert_eq!(
        keys,
        [
            "Graph.AddRingGroup",
            "Graph.AddDirectionPairGroup",
            "Graph.AlterGroupScale",
            "Graph.ShiftGroupOffset",
            "Graph.RetargetGroupKind",
            "Graph.RemoveGroup",
        ]
    );
    // Restricted size pressure admits only the decreasing operator, and the
    // operator and its key agree on the complexity effect.
    let admitted: Vec<_> = GROUP_OPERATORS
        .into_iter()
        .filter(|op| op.complexity_effect().is_decreasing())
        .collect();
    assert_eq!(admitted, vec![GraphOperator::RemoveGroup]);
    for add in [
        GraphOperator::AddRingGroup,
        GraphOperator::AddDirectionPairGroup,
    ] {
        assert_eq!(add.complexity_effect(), ComplexityEffect::Increasing);
    }
    let pair = GraphOperator::ALL
        .iter()
        .position(|&op| op == GraphOperator::AddDirectionPairGroup);
    assert_eq!(pair, Some(23), "directly after AddRingGroup");
    for op in GROUP_OPERATORS {
        assert_eq!(
            op.complexity_effect(),
            op.operator().complexity_effect(),
            "{op:?}"
        );
        assert!(op.enabled(&MutationConfig::default()));
    }
}

#[test]
fn applicability_follows_rings_groups_and_hosting() {
    let genome = |refs, def| CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![graph_node(refs, def)],
    };
    let plain = CgpGraphBackendDef::new_with_fixed_outputs;
    let none = genome(vec![InputReference::ActionVotes], plain());
    let ring = genome(vec![InputReference::ActionVotes, barrier()], plain());
    let pair = genome(vec![InputReference::ActionVotes, area_barrier()], plain());
    let grouped = genome(refs(), three());
    // A graph missing a direction sink hosts no new group, while the group
    // it carries stays an edit site.
    let mut partial = CgpGraphBackendDef::new_with_fixed_outputs();
    partial.add_group(Shape::Ring { ring_ref: 0 }, DirectedVoteKind::Move, 1);
    edit_sinks(&mut partial, |sinks| {
        sinks.retain(|sink| {
            sink.kind
                != OutputSinkKind::ActionVote(VoteSink::Directed(DirectedVoteKind::StealEnergy, 3))
        });
    });
    assert!(!partial.can_host_group());
    let unhosted = genome(refs(), partial);
    for op in GROUP_OPERATORS {
        let applies = |g: &CreatureGenome| {
            !GraphMutator::applicable_indices(g, op, &crate::config::test_sensor_context(1))
                .is_empty()
        };
        let add_ring = op == GraphOperator::AddRingGroup;
        let add_pair = op == GraphOperator::AddDirectionPairGroup;
        assert!(!applies(&none), "{op:?}");
        assert_eq!(applies(&ring), add_ring, "{op:?}");
        assert_eq!(applies(&pair), add_pair, "{op:?}");
        assert_eq!(applies(&grouped), !add_pair, "{op:?}");
        assert_eq!(applies(&unhosted), !add_ring && !add_pair, "{op:?}");
    }
    let vm = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![barrier()],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 4,
                constants: vec![],
                program: vec![],
            }),
            targets: vec![],
        }],
    };
    for op in GROUP_OPERATORS {
        assert!(
            GraphMutator::applicable_indices(&vm, op, &crate::config::test_sensor_context(1))
                .is_empty()
        );
    }
}

#[test]
fn add_draws_ring_then_kind_then_offset_then_learning_and_is_born_silent() {
    let refs = vec![InputReference::ActionVotes, food(1), barrier(), food(0)];
    let mut modes = [false; 3];
    for seed in 0..64 {
        let mut node = graph_node(refs.clone(), CgpGraphBackendDef::new_with_fixed_outputs());
        let mut rng = SmallRng::seed_from_u64(seed);
        let mut expected = SmallRng::seed_from_u64(seed);
        group::add_in_form(
            &mut node,
            group::CreationForm::Uniform,
            &crate::config::test_sensor_context(1),
            &mut rng,
        )
        .unwrap();
        let rings = [1u16, 2, 3];
        let ring_ref = rings[expected.gen_range(0..rings.len())];
        let kind = DirectedVoteKind::ALL[expected.gen_range(0..DirectedVoteKind::ALL.len())];
        let offset = expected.gen_range(0..8u8);
        let mut want = CgpGraphBackendDef::new_with_fixed_outputs();
        let group = want.add_group(Shape::Ring { ring_ref }, kind, offset);
        modes[usize::from(replay_learning(&mut want, group, &mut expected))] = true;
        assert_eq!(graph(&node), &want);
        assert_eq!(rng.gen::<u64>(), expected.gen::<u64>(), "no further draw");
        assert!(validate_graph(&node).is_ok());
        assert_eq!(graph(&node).groups()[0].scale(), 0.0, "born silent");
    }
    assert_eq!(modes, [true; 3], "every learning mode drawn");
}

#[test]
fn unit_operators_draw_the_group_first_and_change_its_record() {
    for seed in 0..64 {
        let before = three();

        let mut node = graph_node(refs(), before.clone());
        let mut rng = SmallRng::seed_from_u64(seed);
        let mut expected = SmallRng::seed_from_u64(seed);
        group::alter_scale(graph_mut(&mut node), &mut rng).unwrap();
        let idx = expected.gen_range(0..3);
        let mut scale = before.groups()[idx].scale();
        crate::mutation::graph::operators::perturb_weight(&mut scale, &mut expected);
        let mut want = before.clone();
        want.group_mut(id(idx)).set_scale(scale);
        assert_eq!(graph(&node), &want);
        assert_eq!(rng.gen::<u64>(), expected.gen::<u64>());

        let mut node = graph_node(refs(), before.clone());
        let mut rng = SmallRng::seed_from_u64(seed);
        let mut expected = SmallRng::seed_from_u64(seed);
        group::shift_offset(graph_mut(&mut node), &mut rng).unwrap();
        let idx = expected.gen_range(0..3);
        let step = if expected.gen_bool(0.5) { 1 } else { 7 };
        let mut want = before.clone();
        let record = before.groups()[idx];
        want.relocate_group(id(idx), record.kind(), (record.offset() + step) % 8);
        assert_eq!(graph(&node), &want);
        assert_eq!(rng.gen::<u64>(), expected.gen::<u64>());
        assert!(validate_graph(&node).is_ok());

        let mut node = graph_node(refs(), before.clone());
        let mut rng = SmallRng::seed_from_u64(seed);
        let mut expected = SmallRng::seed_from_u64(seed);
        group::retarget_kind(graph_mut(&mut node), &mut rng).unwrap();
        let idx = expected.gen_range(0..3);
        let record = before.groups()[idx];
        let others: Vec<_> = DirectedVoteKind::ALL
            .into_iter()
            .filter(|&k| k != record.kind())
            .collect();
        let mut want = before.clone();
        want.relocate_group(
            id(idx),
            others[expected.gen_range(0..others.len())],
            record.offset(),
        );
        assert_eq!(graph(&node), &want);
        assert_eq!(rng.gen::<u64>(), expected.gen::<u64>());
        assert!(validate_graph(&node).is_ok());

        let mut node = graph_node(refs(), before.clone());
        let mut rng = SmallRng::seed_from_u64(seed);
        let mut expected = SmallRng::seed_from_u64(seed);
        group::remove(graph_mut(&mut node), &mut rng).unwrap();
        let mut want = before.clone();
        want.remove_group(id(expected.gen_range(0..3)));
        assert_eq!(graph(&node), &want);
        assert_eq!(rng.gen::<u64>(), expected.gen::<u64>());
        assert!(validate_graph(&node).is_ok());
    }
    let mut empty = CgpGraphBackendDef::new_with_fixed_outputs();
    assert_eq!(
        group::remove(&mut empty, &mut SmallRng::seed_from_u64(0)),
        Err(MutationSkipReason::NoApplicableTarget)
    );
}

/// `InputRef.Add` on a Graph node draws and writes exactly what it would
/// without groups, apart from the members.
#[test]
fn add_is_unchanged_by_groups() {
    for seed in 0..64 {
        let mut plain = v3alpha1_founder_genome();
        let idx = plain
            .nodes
            .iter()
            .position(|node| matches!(node.backend_def, BackendDef::Graph(_)))
            .unwrap();
        let ring = u16::try_from(plain.nodes[idx].input_refs.len()).unwrap();
        plain.nodes[idx].input_refs.push(barrier());
        let mut grouped = plain.clone();
        graph_mut(&mut grouped.nodes[idx]).add_group(
            Shape::Ring { ring_ref: ring },
            DirectedVoteKind::StealEnergy,
            2,
        );

        let mut rngs = [SmallRng::seed_from_u64(seed), SmallRng::seed_from_u64(seed)];
        for (genome, rng) in [&mut plain, &mut grouped].into_iter().zip(&mut rngs) {
            InputRefMutator::apply_to_node(
                genome,
                InputRefOperator::Add,
                idx,
                rng,
                &crate::config::test_sensor_context(2),
            )
            .unwrap();
        }
        let [a, b] = &mut rngs;
        assert_eq!(a.gen::<u64>(), b.gen::<u64>(), "seed {seed}");
        assert_eq!(plain.nodes[idx].input_refs, grouped.nodes[idx].input_refs);
        let (p, q) = (graph(&plain.nodes[idx]), graph(&grouped.nodes[idx]));
        let independent = |def: &CgpGraphBackendDef| -> Vec<_> {
            def.edges()
                .filter(|edge| edge.membership().is_none())
                .copied()
                .collect()
        };
        assert_eq!(independent(p), independent(q));
        assert_eq!(p.compute_nodes(), q.compute_nodes());
        assert_eq!(q.groups().len(), 1);
        assert_eq!(q.members(id(0)).count(), 8);
    }
}

/// Each unit operator reaches its own edit through the mutator's dispatch.
#[test]
fn the_mutator_dispatches_each_unit_operator_to_its_group_edit() {
    type Edit = fn(&mut CgpGraphBackendDef, &mut SmallRng) -> Result<(), MutationSkipReason>;
    let cases: [(GraphOperator, Edit); 4] = [
        (GraphOperator::AlterGroupScale, group::alter_scale),
        (GraphOperator::ShiftGroupOffset, group::shift_offset),
        (GraphOperator::RetargetGroupKind, group::retarget_kind),
        (GraphOperator::RemoveGroup, |def, rng| {
            group::remove(def, rng).map(|_| ())
        }),
    ];
    let config = MutationConfig::default();
    for (op, edit) in cases {
        for seed in 0..16 {
            // Arrange
            let mut genome = CreatureGenome {
                entry_node_id: NodeId::new(0),
                nodes: vec![graph_node(refs(), three())],
            };
            let mut direct = graph_node(refs(), three());

            // Act
            GraphMutator::apply_to_node(
                &mut genome,
                op,
                0,
                &mut SmallRng::seed_from_u64(seed),
                &config,
                &crate::config::test_sensor_context(1),
            )
            .unwrap();
            edit(graph_mut(&mut direct), &mut SmallRng::seed_from_u64(seed)).unwrap();

            // Assert
            assert_eq!(
                graph(&genome.nodes[0]),
                graph(&direct),
                "{op:?} seed {seed}"
            );
        }
    }
}

/// A ring at index 65,535 is a group site; one at 65,536 is not.
#[test]
fn add_reaches_a_ring_at_the_last_u16_index_and_none_past_it() {
    // Arrange
    let mut refs = vec![InputReference::ActionVotes; usize::from(u16::MAX)];
    refs.push(barrier());
    let mut last = graph_node(refs.clone(), CgpGraphBackendDef::new_with_fixed_outputs());
    refs.insert(0, InputReference::ActionVotes);
    let mut past = graph_node(refs, CgpGraphBackendDef::new_with_fixed_outputs());

    // Act
    let added = group::add_in_form(
        &mut last,
        group::CreationForm::Uniform,
        &crate::config::test_sensor_context(1),
        &mut SmallRng::seed_from_u64(0),
    );
    let skipped = group::add_in_form(
        &mut past,
        group::CreationForm::Uniform,
        &crate::config::test_sensor_context(1),
        &mut SmallRng::seed_from_u64(0),
    );

    // Assert
    assert!(added.is_ok());
    assert_eq!(graph(&last).groups().len(), 1);
    assert_eq!(
        graph(&last).groups()[0].shape(),
        Shape::Ring { ring_ref: u16::MAX }
    );
    assert_eq!(skipped, Err(MutationSkipReason::NoApplicableTarget));
    assert!(graph(&past).groups().is_empty());
}

/// `AddRingGroup` spends no draw and skips on a graph that cannot host a
/// group, even beside a ring.
#[test]
fn add_skips_without_a_draw_when_the_graph_cannot_host_a_group() {
    let mut partial = CgpGraphBackendDef::new_with_fixed_outputs();
    edit_sinks(&mut partial, |sinks| {
        sinks.retain(|sink| {
            sink.kind
                != OutputSinkKind::ActionVote(VoteSink::Directed(DirectedVoteKind::Reproduce, 0))
        });
    });
    let mut node = graph_node(vec![barrier()], partial);
    let before = node.clone();
    let mut rng = SmallRng::seed_from_u64(9);
    let mut untouched = SmallRng::seed_from_u64(9);
    assert_eq!(
        group::add_in_form(
            &mut node,
            group::CreationForm::Uniform,
            &crate::config::test_sensor_context(1),
            &mut rng
        ),
        Err(MutationSkipReason::NoApplicableTarget)
    );
    assert_eq!(node, before);
    assert_eq!(rng.gen::<u64>(), untouched.gen::<u64>());
}

// ── `AddDirectionPairGroup` (T25.F04) ───────────────────────────────────────

fn nearby() -> InputReference {
    InputReference::World(WorldInputKey::NearbyCreature)
}

/// The three shape draws in order — the pair over the flat list `(ref_idx
/// ascending, pair ascending)`, the kind, the offset — then the learning
/// draws `replay_learning` replays, and no further draw.
#[test]
fn add_pair_draws_pair_then_kind_then_offset_then_learning_and_is_born_silent() {
    let refs = vec![
        InputReference::ActionVotes,
        nearby(),
        barrier(),
        area_food(1),
        area_barrier(),
    ];
    // NearbyCreature: 4 pairs; AreaFoodSummary 2; AreaBarrierSummary 2.
    let flat: Vec<(u16, u8)> = (0..4)
        .map(|p| (1, p))
        .chain((0..2).map(|p| (3, p)))
        .chain((0..2).map(|p| (4, p)))
        .collect();
    for seed in 0..64 {
        let mut node = graph_node(refs.clone(), CgpGraphBackendDef::new_with_fixed_outputs());
        let mut rng = SmallRng::seed_from_u64(seed);
        let mut expected = SmallRng::seed_from_u64(seed);
        group::add_pair_in_form(
            &mut node,
            group::CreationForm::Uniform,
            &crate::config::test_sensor_context(1),
            &mut rng,
        )
        .unwrap();
        let (ref_idx, pair) = flat[expected.gen_range(0..flat.len())];
        let kind = DirectedVoteKind::ALL[expected.gen_range(0..DirectedVoteKind::ALL.len())];
        let offset = expected.gen_range(0..8u8);
        let layout = refs[usize::from(ref_idx)].direction_pairs()[usize::from(pair)];
        let mut want = CgpGraphBackendDef::new_with_fixed_outputs();
        let group = want.add_pair_group(ref_idx, pair, layout, kind, offset, &[]);
        replay_learning(&mut want, group, &mut expected);
        assert_eq!(graph(&node), &want);
        assert_eq!(rng.gen::<u64>(), expected.gen::<u64>(), "no further draw");
        assert!(validate_graph(&node).is_ok());
        let def = graph(&node);
        assert_eq!(def.groups()[0].scale(), 0.0);
        assert_eq!(def.members(id(0)).count(), 12);
    }
}

/// `AddDirectionPairGroup` spends no draw and skips on a graph that cannot
/// host a group, and on a node declaring no pair.
#[test]
fn add_pair_skips_without_a_draw_without_a_site() {
    let mut partial = CgpGraphBackendDef::new_with_fixed_outputs();
    edit_sinks(&mut partial, |sinks| {
        sinks.retain(|sink| {
            sink.kind != OutputSinkKind::ActionVote(VoteSink::Directed(DirectedVoteKind::Move, 5))
        });
    });
    for mut node in [
        graph_node(vec![area_barrier()], partial),
        graph_node(
            vec![barrier(), InputReference::ActionVotes],
            CgpGraphBackendDef::new_with_fixed_outputs(),
        ),
    ] {
        let before = node.clone();
        let mut rng = SmallRng::seed_from_u64(9);
        let mut untouched = SmallRng::seed_from_u64(9);
        assert_eq!(
            group::add_pair_in_form(
                &mut node,
                group::CreationForm::Uniform,
                &crate::config::test_sensor_context(1),
                &mut rng
            ),
            Err(MutationSkipReason::NoApplicableTarget)
        );
        assert_eq!(node, before);
        assert_eq!(rng.gen::<u64>(), untouched.gen::<u64>());
    }
}

/// The dispatcher reaches `add_pair` with the same draws.
#[test]
fn the_mutator_dispatches_add_direction_pair_group() {
    let refs = vec![area_food(0), barrier()];
    for seed in 0..16 {
        let mut genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![graph_node(refs.clone(), three())],
        };
        let mut direct = graph_node(refs.clone(), three());
        GraphMutator::apply_to_node(
            &mut genome,
            GraphOperator::AddDirectionPairGroup,
            0,
            &mut SmallRng::seed_from_u64(seed),
            &MutationConfig::default(),
            &crate::config::test_sensor_context(1),
        )
        .unwrap();
        group::add_pair_in_form(
            &mut direct,
            group::CreationForm::Uniform,
            &crate::config::test_sensor_context(1),
            &mut SmallRng::seed_from_u64(seed),
        )
        .unwrap();
        assert_eq!(graph(&genome.nodes[0]), graph(&direct), "seed {seed}");
        assert_eq!(graph(&direct).groups().len(), 4);
    }
}

/// The unit operators pick one group uniformly over a list mixing both
/// shapes and keep their draws.
#[test]
fn unit_operators_pick_uniformly_over_both_shapes() {
    let refs = vec![barrier(), area_food(0)];
    let mixed = || {
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
        def.add_group(Shape::Ring { ring_ref: 0 }, DirectedVoteKind::Move, 1);
        let layout = refs[1].direction_pairs()[1];
        let id = def.add_pair_group(1, 1, layout, DirectedVoteKind::Reproduce, 6, &[]);
        def.group_mut(id).set_scale(0.25);
        def
    };
    let mut seen = [false; 2];
    for seed in 0..64 {
        let before = mixed();
        let mut node = graph_node(refs.clone(), before.clone());
        let mut rng = SmallRng::seed_from_u64(seed);
        let mut expected = SmallRng::seed_from_u64(seed);
        group::shift_offset(graph_mut(&mut node), &mut rng).unwrap();
        let idx = expected.gen_range(0..2);
        seen[idx] = true;
        let step = if expected.gen_bool(0.5) { 1 } else { 7 };
        let record = before.groups()[idx];
        let mut want = before.clone();
        want.relocate_group(id(idx), record.kind(), (record.offset() + step) % 8);
        assert_eq!(graph(&node), &want);
        assert_eq!(rng.gen::<u64>(), expected.gen::<u64>());
        assert!(validate_graph(&node).is_ok());

        let mut node = graph_node(refs.clone(), before.clone());
        let mut rng = SmallRng::seed_from_u64(seed);
        let mut expected = SmallRng::seed_from_u64(seed);
        group::alter_scale(graph_mut(&mut node), &mut rng).unwrap();
        let idx = expected.gen_range(0..2);
        let mut scale = before.groups()[idx].scale();
        crate::mutation::graph::operators::perturb_weight(&mut scale, &mut expected);
        let mut want = before.clone();
        want.group_mut(id(idx)).set_scale(scale);
        assert_eq!(graph(&node), &want);

        let mut node = graph_node(refs.clone(), before.clone());
        let mut rng = SmallRng::seed_from_u64(seed);
        group::retarget_kind(graph_mut(&mut node), &mut rng).unwrap();
        assert!(validate_graph(&node).is_ok());

        let mut node = graph_node(refs.clone(), before.clone());
        let mut rng = SmallRng::seed_from_u64(seed);
        let mut expected = SmallRng::seed_from_u64(seed);
        group::remove(graph_mut(&mut node), &mut rng).unwrap();
        let mut want = before;
        want.remove_group(id(expected.gen_range(0..2)));
        assert_eq!(graph(&node), &want);
        assert!(validate_graph(&node).is_ok());
    }
    assert_eq!(seen, [true, true]);
}

/// A pair reference at index 65,535 is a site; one at 65,536 is not.
#[test]
fn add_pair_reaches_a_pair_at_the_last_u16_index_and_none_past_it() {
    let mut refs = vec![InputReference::ActionVotes; usize::from(u16::MAX)];
    refs.push(area_barrier());
    let mut last = graph_node(refs.clone(), CgpGraphBackendDef::new_with_fixed_outputs());
    refs.insert(0, InputReference::ActionVotes);
    let mut past = graph_node(refs, CgpGraphBackendDef::new_with_fixed_outputs());
    assert!(group::add_pair_in_form(
        &mut last,
        group::CreationForm::Uniform,
        &crate::config::test_sensor_context(1),
        &mut SmallRng::seed_from_u64(0)
    )
    .is_ok());
    assert_eq!(graph(&last).groups()[0].shape().ref_idx(), u16::MAX);
    assert_eq!(
        group::add_pair_in_form(
            &mut past,
            group::CreationForm::Uniform,
            &crate::config::test_sensor_context(1),
            &mut SmallRng::seed_from_u64(0)
        ),
        Err(MutationSkipReason::NoApplicableTarget)
    );
}

/// The randomized form's learning draws on group `group` of `want`: one
/// [`replay_member`] sequence per member in canonical order, then one
/// `inherit_scale` coin.
fn replay_randomized(want: &mut CgpGraphBackendDef, group: GroupId, rng: &mut SmallRng) {
    for member in want.members(group).collect::<Vec<_>>() {
        let (_, config, scope) = replay_member(rng);
        let mut edge = want.edge_mut(member);
        edge.set_plasticity(config);
        edge.set_scope(scope);
    }
    let inherit_scale = rng.gen_bool(0.5);
    want.group_mut(group).set_inherit_scale(inherit_scale);
}

/// The refs the creation tests read: rings at 1, 2 and 3 and the
/// pair-declaring references at 4 and 5.
fn creation_refs() -> Vec<InputReference> {
    vec![
        InputReference::ActionVotes,
        food(1),
        barrier(),
        food(0),
        nearby(),
        area_barrier(),
    ]
}

/// One creation event `op` through the dispatch on an empty graph; returns
/// the node and the event's RNG after it.
fn create(op: GraphOperator, seed: u64) -> (NodeGenome, SmallRng) {
    let mut genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![graph_node(
            creation_refs(),
            CgpGraphBackendDef::new_with_fixed_outputs(),
        )],
    };
    let mut rng = SmallRng::seed_from_u64(seed);
    GraphMutator::apply_to_node(
        &mut genome,
        op,
        0,
        &mut rng,
        &MutationConfig::default(),
        &crate::config::test_sensor_context(2),
    )
    .unwrap();
    (genome.nodes.remove(0), rng)
}

/// Either shape's draws, replayed: a `Ring` draws the ring, kind and
/// offset; a `DirectionPair` the pair over the flat list, kind and offset.
/// Returns the silent group the container builds from them.
fn replay_shape(pair: bool, rng: &mut SmallRng) -> (CgpGraphBackendDef, GroupId) {
    let refs = creation_refs();
    let mut want = CgpGraphBackendDef::new_with_fixed_outputs();
    let group = if pair {
        // NearbyCreature: 4 pairs; AreaBarrierSummary 2.
        let flat: Vec<(u16, u8)> = (0..4)
            .map(|p| (4, p))
            .chain((0..2).map(|p| (5, p)))
            .collect();
        let (ref_idx, pair) = flat[rng.gen_range(0..flat.len())];
        let kind = DirectedVoteKind::ALL[rng.gen_range(0..DirectedVoteKind::ALL.len())];
        let offset = rng.gen_range(0..8u8);
        let layout = refs[usize::from(ref_idx)].direction_pairs()[usize::from(pair)];
        want.add_pair_group(ref_idx, pair, layout, kind, offset, &[])
    } else {
        let rings = [1u16, 2, 3];
        let ring_ref = rings[rng.gen_range(0..rings.len())];
        let kind = DirectedVoteKind::ALL[rng.gen_range(0..DirectedVoteKind::ALL.len())];
        let offset = rng.gen_range(0..8u8);
        want.add_group(Shape::Ring { ring_ref }, kind, offset)
    };
    (want, group)
}

proptest::proptest! {
    /// T25.F08 invariant 2: a randomized creation makes its uniform twin's
    /// shape draws, then per member its mode, fields and scope, then one
    /// `inherit_scale`, and no further draw; the group is born silent.
    #[test]
    fn randomized_creation_draws_shape_then_per_member_mode_fields_scope_then_inherit_scale(
        seed in proptest::prelude::any::<u64>(),
        pair in proptest::prelude::any::<bool>(),
    ) {
        let op = if pair {
            GraphOperator::AddRandomizedDirectionPairGroup
        } else {
            GraphOperator::AddRandomizedRingGroup
        };
        let (node, mut rng) = create(op, seed);
        let mut expected = SmallRng::seed_from_u64(seed);
        let (mut want, group) = replay_shape(pair, &mut expected);
        replay_randomized(&mut want, group, &mut expected);
        proptest::prop_assert_eq!(graph(&node), &want);
        proptest::prop_assert_eq!(rng.gen::<u64>(), expected.gen::<u64>(), "no further draw");
        proptest::prop_assert!(validate_graph(&node).is_ok());
        let def = graph(&node);
        proptest::prop_assert_eq!(def.groups()[0].scale(), 0.0, "born silent");
        proptest::prop_assert!(def.edges().all(|edge| edge.own_weight() == 0.0));
    }

    /// The uniform operators' draws are unchanged through the dispatch:
    /// the shape draws, then learned creation's one mode, one scope and one
    /// `inherit_scale`, every member alike.
    #[test]
    fn uniform_creation_draws_are_unchanged(
        seed in proptest::prelude::any::<u64>(),
        pair in proptest::prelude::any::<bool>(),
    ) {
        let op = if pair {
            GraphOperator::AddDirectionPairGroup
        } else {
            GraphOperator::AddRingGroup
        };
        let (node, mut rng) = create(op, seed);
        let mut expected = SmallRng::seed_from_u64(seed);
        let (mut want, group) = replay_shape(pair, &mut expected);
        replay_learning(&mut want, group, &mut expected);
        proptest::prop_assert_eq!(graph(&node), &want);
        proptest::prop_assert_eq!(rng.gen::<u64>(), expected.gen::<u64>(), "no further draw");
    }
}

/// Randomized members draw their own attributes: some seed gives one group
/// members both static and plastic, and members of both scopes.
#[test]
fn randomized_members_differ_within_one_group() {
    let mut mixed = [false; 2];
    for seed in 0..32 {
        for op in [
            GraphOperator::AddRandomizedRingGroup,
            GraphOperator::AddRandomizedDirectionPairGroup,
        ] {
            let (node, _) = create(op, seed);
            let def = graph(&node);
            let edges: Vec<_> = def.members(id(0)).map(|at| def.edge_at(at)).collect();
            mixed[0] |= edges.iter().any(|e| e.plasticity().is_some())
                && edges.iter().any(|e| e.plasticity().is_none());
            let scope = |e: &&crate::genome::cgp::GraphEdge| e.membership().unwrap().scope;
            mixed[1] |= edges.iter().any(|e| scope(e) == Scope::Group)
                && edges.iter().any(|e| scope(e) == Scope::Edge);
        }
    }
    assert_eq!(mixed, [true; 2]);
}

/// A one-ring-group node at scale 0.5 whose first member (canonical order)
/// is a Lamarckian learner at own weight 0.25 with an edge seed of 9.0; the
/// other seven are static and unseeded, and the group carries no seed.
fn detach_fixture() -> (NodeGenome, EdgeOccurrence) {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    let group = def.add_group(Shape::Ring { ring_ref: 0 }, DirectedVoteKind::Move, 3);
    def.group_mut(group).set_scale(0.5);
    let first = def.members(group).next().unwrap();
    let config = PlasticityConfig {
        rule: HebbianRule::Oja,
        learning_rate: 0.1,
        weight_clamp: 2.0,
        lamarckian: true,
        modulation: None,
        retention: 1.0,
    };
    def.edge_mut(first).set_plasticity(Some(config));
    def.edge_mut(first).set_weight(0.25);
    let compute = def
        .compute_nodes()
        .iter()
        .map(|node| vec![None; node.inputs.len()])
        .collect();
    let mut sinks: Vec<Vec<Option<f32>>> = def
        .output_sinks()
        .iter()
        .map(|sink| vec![None; sink.inputs.len()])
        .collect();
    let EdgeSurface::SinkInput(sink) = first.surface else {
        panic!("a member is a sink edge")
    };
    sinks[sink][first.index] = Some(9.0);
    def.seed_birth(BirthSeeds::Rows {
        compute,
        sinks,
        groups: vec![None],
    });
    (graph_node(refs(), def), first)
}

proptest::proptest! {
    /// T25.F08 invariant 3: `DetachGraphEdge` draws one member uniformly in
    /// canonical order and nothing else, then detaches it as the F03
    /// primitive does: own weight `compose(own, scale, pattern)` bitwise, no
    /// membership, plasticity kept; the Lamarckian member's seed composes
    /// with the group's scale, a static unseeded member carries none.
    #[test]
    fn detach_draws_one_member_and_keeps_its_composed_weight_plasticity_and_seed(
        seed in proptest::prelude::any::<u64>(),
    ) {
        let (original, learner) = detach_fixture();
        let mut node = original.clone();
        let mut rng = SmallRng::seed_from_u64(seed);
        group::detach_member(&mut node, &mut rng).unwrap();
        let mut expected = SmallRng::seed_from_u64(seed);
        let def = graph(&original);
        let members: Vec<_> = def.members(id(0)).collect();
        let picked = members[expected.gen_range(0..members.len())];
        proptest::prop_assert_eq!(rng.gen::<u64>(), expected.gen::<u64>(), "one draw");

        let mut want = def.clone();
        let (at, removed) = detach(&mut want, picked, &original.input_refs);
        proptest::prop_assert!(!removed);
        proptest::prop_assert_eq!(graph(&node), &want);
        let pattern = def.pattern(picked, &original.input_refs);
        let before = def.edge_at(picked);
        let after = graph(&node).edge_at(at);
        proptest::prop_assert_eq!(
            after.own_weight().to_bits(),
            compose(before.own_weight(), 0.5, pattern).to_bits()
        );
        proptest::prop_assert_eq!(after.membership(), None);
        proptest::prop_assert_eq!(after.plasticity(), before.plasticity());
        let seed_value = graph(&node).birth_seeds().unwrap().value(at);
        if picked == learner {
            proptest::prop_assert_eq!(seed_value, Some(compose(9.0, 0.5, pattern)));
        } else {
            proptest::prop_assert_eq!(seed_value, None);
        }
        proptest::prop_assert_eq!(graph(&node).members(id(0)).count(), 7);
        proptest::prop_assert!(validate_graph(&node).is_ok());
    }
}

/// `DetachGraphEdge` applies exactly where a member exists: not on a graph
/// without a group, on one with a group, and not once the groups are gone;
/// it skips without a draw where none is.
#[test]
fn applicability_follows_members_for_detach() {
    let genome = |def| CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![graph_node(refs(), def)],
    };
    let op = GraphOperator::DetachGraphEdge;
    let plain = genome(CgpGraphBackendDef::new_with_fixed_outputs());
    assert!(
        GraphMutator::applicable_indices(&plain, op, &crate::config::test_sensor_context(1))
            .is_empty()
    );
    let grouped = genome(three());
    assert_eq!(
        GraphMutator::applicable_indices(&grouped, op, &crate::config::test_sensor_context(1)),
        vec![0]
    );
    let mut emptied = three();
    while !emptied.groups().is_empty() {
        emptied.remove_group(id(0));
    }
    assert!(GraphMutator::applicable_indices(
        &genome(emptied),
        op,
        &crate::config::test_sensor_context(1)
    )
    .is_empty());

    let mut node = graph_node(refs(), CgpGraphBackendDef::new_with_fixed_outputs());
    let before = node.clone();
    let mut rng = SmallRng::seed_from_u64(5);
    let mut replay = rng.clone();
    assert_eq!(
        group::detach_member(&mut node, &mut rng),
        Err(MutationSkipReason::NoApplicableTarget)
    );
    assert_eq!(node, before);
    assert_eq!(rng.gen::<u64>(), replay.gen::<u64>(), "no draw");
}

/// T25.F08 invariant 4: `AddGraphEdge` on a node whose sinks hold groups
/// pushes an ordinary edge, never a member: every group keeps its record
/// and its member set.
#[test]
fn add_graph_edge_never_joins_a_group() {
    let mut onto_a_group_sink = false;
    for seed in 0..256 {
        let mut genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![graph_node(refs(), three())],
        };
        let before = graph(&genome.nodes[0]).clone();
        let mut rng = SmallRng::seed_from_u64(seed);
        let config = MutationConfig::default();
        let op = GraphOperator::AddGraphEdge;
        if GraphMutator::apply_to_node(
            &mut genome,
            op,
            0,
            &mut rng,
            &config,
            &crate::config::test_sensor_context(2),
        )
        .is_err()
        {
            continue;
        }
        let after = graph(&genome.nodes[0]);
        assert_eq!(after.groups(), before.groups(), "seed {seed}");
        for index in 0..before.groups().len() {
            let members = |def: &CgpGraphBackendDef| -> Vec<_> {
                def.members(id(index)).map(|at| *def.edge_at(at)).collect()
            };
            assert_eq!(members(after), members(&before), "seed {seed}");
        }
        let ordinary = |def: &CgpGraphBackendDef| {
            def.edges()
                .filter(|edge| edge.membership().is_none())
                .count()
        };
        assert_eq!(ordinary(after), ordinary(&before) + 1, "seed {seed}");
        onto_a_group_sink |=
            after
                .output_sinks()
                .iter()
                .zip(before.output_sinks())
                .any(|(new, old)| {
                    new.inputs.len() > old.inputs.len()
                        && old.inputs.iter().any(|edge| edge.membership().is_some())
                });
    }
    assert!(
        onto_a_group_sink,
        "some push lands on a sink holding a group"
    );
}

/// T25.F10: `RetargetGroupKind` moves a gene's fact members to the new
/// kind's drive, and skips, after its draws, when that kind cannot host
/// facts; a gene without facts retargets regardless.
#[test]
fn retarget_kind_moves_fact_members_only_to_a_kind_that_can_host_them() {
    let refs = vec![area_food(0)];
    let layout = refs[0].direction_pairs()[1];
    let gene = |facts: &[u16]| {
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
        def.add_pair_group(0, 1, layout, DirectedVoteKind::Move, 2, facts);
        // Reproduce loses its drive sink.
        let drive = OutputSinkKind::ActionVote(DirectedVoteKind::Reproduce.drive());
        edit_sinks(&mut def, |sinks| sinks.retain(|sink| sink.kind != drive));
        def
    };
    let (mut moved, mut skipped) = (false, false);
    for seed in 0..32 {
        let mut def = gene(layout.facts);
        let before = def.clone();
        let mut rng = SmallRng::seed_from_u64(seed);
        let target = {
            let mut expected = SmallRng::seed_from_u64(seed);
            expected.gen_range(0..1);
            [
                DirectedVoteKind::Reproduce,
                DirectedVoteKind::StealEnergy,
                DirectedVoteKind::GiveEnergy,
            ][expected.gen_range(0..3)]
        };
        let mut factless = gene(&[]);
        assert_eq!(
            group::retarget_kind(&mut factless, &mut SmallRng::seed_from_u64(seed)),
            Ok(())
        );
        let result = group::retarget_kind(&mut def, &mut rng);
        if target == DirectedVoteKind::Reproduce {
            assert_eq!(result, Err(MutationSkipReason::NoApplicableTarget));
            skipped = true;
            continue;
        }
        assert_eq!(result, Ok(()));
        moved = true;
        assert_ne!(def, before);
        let drive = def
            .sink_surface(OutputSinkKind::ActionVote(target.drive()))
            .unwrap();
        let facts = def
            .members(id(0))
            .filter(|&member| member.surface == drive)
            .count();
        assert_eq!(facts, layout.facts.len());
        assert!(validate_graph(&graph_node(refs.clone(), def)).is_ok());
    }
    assert!(moved && skipped);
}
