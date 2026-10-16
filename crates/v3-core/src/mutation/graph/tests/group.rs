//! T25.F03 edge-group operators: catalog, applicability, pinned draw orders,
//! and the operators whose draws a group must not change.

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use crate::config::{MutationConfig, OrdinaryFoodTypeId};
use crate::contracts::{InputReference, NodeId, WorldInputKey};
use crate::creature::founder::v3alpha1_founder_genome;
use crate::creature::genome::cgp::test_support::edit_sinks;
use crate::creature::genome::cgp::{
    validate_graph, CgpGraphBackendDef, GroupId, OutputSinkKind, Shape,
};
use crate::creature::genome::vote::{DirectedVoteKind, VoteSink};
use crate::creature::genome::{BackendDef, CreatureGenome, NodeGenome, VmBackendDef};
use crate::mutation::engine::graph_operator_key;
use crate::mutation::graph::{group, refinement, GraphMutator, GraphOperator};
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

#[test]
fn the_catalog_keeps_six_operators_at_their_edge_tiers() {
    assert_eq!(GraphOperator::ALL.len(), 29);
    assert_eq!(crate::mutation::types::MutationOperator::all().len(), 62);
    assert_eq!(&GraphOperator::ALL[23..], &GROUP_OPERATORS);
    let weights = GROUP_OPERATORS.map(GraphOperator::weight);
    assert_eq!(weights, [2, 2, 4, 2, 2, 2]);
    let default_arm: u16 = GraphOperator::ALL
        .iter()
        .filter(|op| {
            !matches!(
                op,
                GraphOperator::RecruitNeutralInput | GraphOperator::RefineHeritableStructure
            )
        })
        .map(|op| u16::from(op.weight()))
        .sum();
    assert_eq!(default_arm, 58);
    let keys = GROUP_OPERATORS.map(|op| graph_operator_key(op).as_key());
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
    assert_eq!(pair, Some(24), "directly after AddRingGroup");
    for op in GROUP_OPERATORS {
        assert_eq!(
            op.complexity_effect(),
            graph_operator_key(op).complexity_effect(),
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
        sinks.retain(|sink| sink.kind != OutputSinkKind::ActionVote(VoteSink::StealEnergy(3)));
    });
    assert!(!partial.can_host_group());
    let unhosted = genome(refs(), partial);
    for op in GROUP_OPERATORS {
        let applies = |g: &CreatureGenome| !GraphMutator::applicable_indices(g, op).is_empty();
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
        assert!(GraphMutator::applicable_indices(&vm, op).is_empty());
    }
}

#[test]
fn add_draws_ring_then_kind_then_offset_and_is_born_silent() {
    let refs = vec![InputReference::ActionVotes, food(1), barrier(), food(0)];
    for seed in 0..64 {
        let mut node = graph_node(refs.clone(), CgpGraphBackendDef::new_with_fixed_outputs());
        let mut rng = SmallRng::seed_from_u64(seed);
        let mut expected = SmallRng::seed_from_u64(seed);
        group::add(&mut node, &mut rng).unwrap();
        let rings = [1u16, 2, 3];
        let ring_ref = rings[expected.gen_range(0..rings.len())];
        let kind = DirectedVoteKind::ALL[expected.gen_range(0..3)];
        let offset = expected.gen_range(0..8u8);
        let mut want = CgpGraphBackendDef::new_with_fixed_outputs();
        want.add_group(Shape::Ring { ring_ref }, kind, offset);
        assert_eq!(graph(&node), &want);
        assert_eq!(rng.gen::<u64>(), expected.gen::<u64>(), "no further draw");
        assert!(validate_graph(&node).is_ok());
    }
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
        want.relocate_group(id(idx), others[expected.gen_range(0..2)], record.offset());
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

/// `InputRef.Add` on a Graph node and structured refinement draw and write
/// exactly what they would without groups, apart from the members.
#[test]
fn add_and_refinement_are_unchanged_by_groups() {
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
            InputRefMutator::apply_to_node(genome, InputRefOperator::Add, idx, rng, 2).unwrap();
            let _ = refinement::refine(&mut genome.nodes[idx], rng);
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
        (GraphOperator::RemoveGroup, group::remove),
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
                1,
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
    let added = group::add(&mut last, &mut SmallRng::seed_from_u64(0));
    let skipped = group::add(&mut past, &mut SmallRng::seed_from_u64(0));

    // Assert
    assert_eq!(added, Ok(()));
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
        sinks.retain(|sink| sink.kind != OutputSinkKind::ActionVote(VoteSink::Reproduce(0)));
    });
    let mut node = graph_node(vec![barrier()], partial);
    let before = node.clone();
    let mut rng = SmallRng::seed_from_u64(9);
    let mut untouched = SmallRng::seed_from_u64(9);
    assert_eq!(
        group::add(&mut node, &mut rng),
        Err(MutationSkipReason::NoApplicableTarget)
    );
    assert_eq!(node, before);
    assert_eq!(rng.gen::<u64>(), untouched.gen::<u64>());
}

// ── `AddDirectionPairGroup` (T25.F04) ───────────────────────────────────────

fn nearby_core() -> InputReference {
    InputReference::World(WorldInputKey::NearbyCreatureCore)
}

/// Three draws in order — the pair over the flat list `(ref_idx ascending,
/// pair ascending)`, the kind, the offset — and no further draw.
#[test]
fn add_pair_draws_pair_then_kind_then_offset_and_is_born_silent() {
    let refs = vec![
        InputReference::ActionVotes,
        nearby_core(),
        barrier(),
        area_food(1),
        area_barrier(),
    ];
    // NearbyCreatureCore: 4 pairs; AreaFoodSummary 2; AreaBarrierSummary 2.
    let flat: Vec<(u16, u8)> = (0..4)
        .map(|p| (1, p))
        .chain((0..2).map(|p| (3, p)))
        .chain((0..2).map(|p| (4, p)))
        .collect();
    for seed in 0..64 {
        let mut node = graph_node(refs.clone(), CgpGraphBackendDef::new_with_fixed_outputs());
        let mut rng = SmallRng::seed_from_u64(seed);
        let mut expected = SmallRng::seed_from_u64(seed);
        group::add_pair(&mut node, &mut rng).unwrap();
        let (ref_idx, pair) = flat[expected.gen_range(0..flat.len())];
        let kind = DirectedVoteKind::ALL[expected.gen_range(0..3)];
        let offset = expected.gen_range(0..8u8);
        let layout = refs[usize::from(ref_idx)].direction_pairs()[usize::from(pair)];
        let mut want = CgpGraphBackendDef::new_with_fixed_outputs();
        want.add_pair_group(ref_idx, pair, layout, kind, offset);
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
        sinks.retain(|sink| sink.kind != OutputSinkKind::ActionVote(VoteSink::Move(5)));
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
            group::add_pair(&mut node, &mut rng),
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
            1,
        )
        .unwrap();
        group::add_pair(&mut direct, &mut SmallRng::seed_from_u64(seed)).unwrap();
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
        let id = def.add_pair_group(1, 1, layout, DirectedVoteKind::Reproduce, 6);
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
    assert_eq!(
        group::add_pair(&mut last, &mut SmallRng::seed_from_u64(0)),
        Ok(())
    );
    assert_eq!(graph(&last).groups()[0].shape().ref_idx(), u16::MAX);
    assert_eq!(
        group::add_pair(&mut past, &mut SmallRng::seed_from_u64(0)),
        Err(MutationSkipReason::NoApplicableTarget)
    );
}
