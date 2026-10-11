//! T11.F31 ring projection operators: catalog, applicability, pinned draw
//! orders, and the operators whose draws a projection must not change.

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use crate::config::{MutationConfig, OrdinaryFoodTypeId};
use crate::contracts::{InputReference, NodeId, WorldInputKey};
use crate::creature::founder::v3alpha1_founder_genome;
use crate::creature::genome::cgp::{CgpGraphBackendDef, RingProjection};
use crate::creature::genome::vote::DirectedVoteKind;
use crate::creature::genome::{BackendDef, CreatureGenome, NodeGenome, VmBackendDef};
use crate::mutation::engine::graph_operator_key;
use crate::mutation::graph::{projection, refinement, GraphMutator, GraphOperator};
use crate::mutation::input_ref::{InputRefMutator, InputRefOperator};
use crate::mutation::types::{ComplexityEffect, MutationSkipReason};

const RING_OPERATORS: [GraphOperator; 5] = [
    GraphOperator::AddRingProjection,
    GraphOperator::AlterRingProjectionWeight,
    GraphOperator::ShiftRingProjectionOffset,
    GraphOperator::RetargetRingProjectionKind,
    GraphOperator::RemoveRingProjection,
];

fn barrier() -> InputReference {
    InputReference::World(WorldInputKey::NeighborBarrierRing)
}

fn food(t: u16) -> InputReference {
    InputReference::World(WorldInputKey::neighbor_food_ring(OrdinaryFoodTypeId::new(
        t,
    )))
}

fn graph_node(refs: Vec<InputReference>, projections: Vec<RingProjection>) -> NodeGenome {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    def.projections = projections;
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

fn three() -> Vec<RingProjection> {
    vec![
        RingProjection::new(0, DirectedVoteKind::Move, 0, 0.0),
        RingProjection::new(2, DirectedVoteKind::Reproduce, 5, 0.5),
        RingProjection::new(0, DirectedVoteKind::StealEnergy, 7, -2.0),
    ]
}

#[test]
fn the_catalog_appends_five_operators_at_their_edge_tiers() {
    assert_eq!(GraphOperator::ALL.len(), 28);
    assert_eq!(&GraphOperator::ALL[23..], &RING_OPERATORS);
    let weights = RING_OPERATORS.map(GraphOperator::weight);
    assert_eq!(weights, [2, 4, 2, 2, 2]);
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
    assert_eq!(default_arm, 56);
    let keys = RING_OPERATORS.map(|op| graph_operator_key(op).as_key());
    assert_eq!(
        keys,
        [
            "Graph.AddRingProjection",
            "Graph.AlterRingProjectionWeight",
            "Graph.ShiftRingProjectionOffset",
            "Graph.RetargetRingProjectionKind",
            "Graph.RemoveRingProjection",
        ]
    );
    // Restricted size pressure admits only the decreasing operator.
    let admitted: Vec<_> = RING_OPERATORS
        .into_iter()
        .filter(|op| op.complexity_effect().is_decreasing())
        .collect();
    assert_eq!(admitted, vec![GraphOperator::RemoveRingProjection]);
    assert_eq!(
        GraphOperator::AddRingProjection.complexity_effect(),
        ComplexityEffect::Increasing
    );
    for op in RING_OPERATORS {
        assert!(op.enabled(&MutationConfig::default()));
    }
}

#[test]
fn applicability_follows_rings_and_projections() {
    let genome = |refs, projections| CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![graph_node(refs, projections)],
    };
    let none = genome(vec![InputReference::ActionVotes], vec![]);
    let ring = genome(vec![InputReference::ActionVotes, barrier()], vec![]);
    let projected = genome(vec![InputReference::ActionVotes], three());
    for op in RING_OPERATORS {
        assert!(
            GraphMutator::applicable_indices(&none, op).is_empty(),
            "{op:?}"
        );
        let expected_ring = op == GraphOperator::AddRingProjection;
        assert_eq!(
            !GraphMutator::applicable_indices(&ring, op).is_empty(),
            expected_ring,
            "{op:?}"
        );
        assert_eq!(
            !GraphMutator::applicable_indices(&projected, op).is_empty(),
            !expected_ring,
            "{op:?}"
        );
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
    for op in RING_OPERATORS {
        assert!(GraphMutator::applicable_indices(&vm, op).is_empty());
    }
}

#[test]
fn add_draws_ring_then_kind_then_offset_and_is_born_silent() {
    let refs = vec![InputReference::ActionVotes, food(1), barrier(), food(0)];
    for seed in 0..64 {
        let mut node = graph_node(refs.clone(), three());
        let mut rng = SmallRng::seed_from_u64(seed);
        let mut expected = SmallRng::seed_from_u64(seed);
        projection::add(&mut node, &mut rng).unwrap();
        let rings = [1u16, 2, 3];
        let ref_idx = rings[expected.gen_range(0..rings.len())];
        let kind = DirectedVoteKind::ALL[expected.gen_range(0..3)];
        let offset = expected.gen_range(0..8u8);
        let def = graph(&node);
        assert_eq!(&def.projections[..3], &three()[..]);
        assert_eq!(
            def.projections[3],
            RingProjection::new(ref_idx, kind, offset, 0.0)
        );
        assert_eq!(rng.gen::<u64>(), expected.gen::<u64>(), "no further draw");
    }
}

#[test]
fn unit_operators_draw_the_projection_first_and_change_one_field() {
    for seed in 0..64 {
        let refs = vec![barrier(), InputReference::ActionVotes, food(0)];
        let before = three();

        let mut node = graph_node(refs.clone(), before.clone());
        let mut rng = SmallRng::seed_from_u64(seed);
        let mut expected = SmallRng::seed_from_u64(seed);
        projection::alter_weight(graph_mut(&mut node), &mut rng).unwrap();
        let idx = expected.gen_range(0..3);
        let mut w = before[idx].weight;
        crate::mutation::graph::operators::perturb_weight(&mut w, &mut expected);
        let mut want = before.clone();
        want[idx].weight = w;
        assert_eq!(graph(&node).projections, want);
        assert_eq!(rng.gen::<u64>(), expected.gen::<u64>());

        let mut node = graph_node(refs.clone(), before.clone());
        let mut rng = SmallRng::seed_from_u64(seed);
        let mut expected = SmallRng::seed_from_u64(seed);
        projection::shift_offset(graph_mut(&mut node), &mut rng).unwrap();
        let idx = expected.gen_range(0..3);
        let step = if expected.gen_bool(0.5) { 1 } else { 7 };
        let mut want = before.clone();
        want[idx].offset = (want[idx].offset + step) % 8;
        assert_eq!(graph(&node).projections, want);
        assert_eq!(rng.gen::<u64>(), expected.gen::<u64>());

        let mut node = graph_node(refs.clone(), before.clone());
        let mut rng = SmallRng::seed_from_u64(seed);
        let mut expected = SmallRng::seed_from_u64(seed);
        projection::retarget_kind(graph_mut(&mut node), &mut rng).unwrap();
        let idx = expected.gen_range(0..3);
        let others: Vec<_> = DirectedVoteKind::ALL
            .into_iter()
            .filter(|&k| k != before[idx].kind)
            .collect();
        let mut want = before.clone();
        want[idx].kind = others[expected.gen_range(0..2)];
        assert_eq!(graph(&node).projections, want);
        assert_eq!(rng.gen::<u64>(), expected.gen::<u64>());

        let mut node = graph_node(refs, before.clone());
        let mut rng = SmallRng::seed_from_u64(seed);
        let mut expected = SmallRng::seed_from_u64(seed);
        projection::remove(graph_mut(&mut node), &mut rng).unwrap();
        let mut want = before.clone();
        want.remove(expected.gen_range(0..3));
        assert_eq!(graph(&node).projections, want);
        assert_eq!(rng.gen::<u64>(), expected.gen::<u64>());
    }
    let mut empty = CgpGraphBackendDef::new_with_fixed_outputs();
    assert_eq!(
        projection::remove(&mut empty, &mut SmallRng::seed_from_u64(0)),
        Err(MutationSkipReason::NoApplicableTarget)
    );
}

/// `InputRef.Add` on a Graph node and structured refinement draw and write
/// exactly what they would without projections.
#[test]
fn add_and_refinement_are_unchanged_by_projections() {
    for seed in 0..64 {
        let mut plain = v3alpha1_founder_genome();
        let idx = plain
            .nodes
            .iter()
            .position(|node| matches!(node.backend_def, BackendDef::Graph(_)))
            .unwrap();
        let mut projected = plain.clone();
        graph_mut(&mut projected.nodes[idx]).projections = three();

        let mut rngs = [SmallRng::seed_from_u64(seed), SmallRng::seed_from_u64(seed)];
        for (genome, rng) in [&mut plain, &mut projected].into_iter().zip(&mut rngs) {
            InputRefMutator::apply_to_node(genome, InputRefOperator::Add, idx, rng, 2).unwrap();
            let _ = refinement::refine(&mut genome.nodes[idx], rng);
        }
        let [a, b] = &mut rngs;
        assert_eq!(a.gen::<u64>(), b.gen::<u64>(), "seed {seed}");
        assert_eq!(plain.nodes[idx].input_refs, projected.nodes[idx].input_refs);
        let (p, q) = (graph(&plain.nodes[idx]), graph(&projected.nodes[idx]));
        assert_eq!(p.output_sinks, q.output_sinks);
        assert_eq!(p.compute_nodes, q.compute_nodes);
        assert_eq!(q.projections, three());
    }
}

/// Each unit operator reaches its own edit through the mutator's dispatch.
#[test]
fn the_mutator_dispatches_each_unit_operator_to_its_projection_edit() {
    type Edit = fn(&mut CgpGraphBackendDef, &mut SmallRng) -> Result<(), MutationSkipReason>;
    let cases: [(GraphOperator, Edit); 4] = [
        (
            GraphOperator::AlterRingProjectionWeight,
            projection::alter_weight,
        ),
        (
            GraphOperator::ShiftRingProjectionOffset,
            projection::shift_offset,
        ),
        (
            GraphOperator::RetargetRingProjectionKind,
            projection::retarget_kind,
        ),
        (GraphOperator::RemoveRingProjection, projection::remove),
    ];
    let config = MutationConfig::default();
    for (op, edit) in cases {
        for seed in 0..16 {
            // Arrange
            let refs = vec![barrier(), food(0)];
            let mut genome = CreatureGenome {
                entry_node_id: NodeId::new(0),
                nodes: vec![graph_node(refs.clone(), three())],
            };
            let mut direct = graph_node(refs, three());

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
                graph(&genome.nodes[0]).projections,
                graph(&direct).projections,
                "{op:?} seed {seed}"
            );
        }
    }
}

/// A ring at index 65,535 is a projection site; one at 65,536 is not.
#[test]
fn add_reaches_a_ring_at_the_last_u16_index_and_none_past_it() {
    // Arrange
    let mut refs = vec![InputReference::ActionVotes; usize::from(u16::MAX)];
    refs.push(barrier());
    let mut last = graph_node(refs.clone(), vec![]);
    refs.insert(0, InputReference::ActionVotes);
    let mut past = graph_node(refs, vec![]);

    // Act
    let added = projection::add(&mut last, &mut SmallRng::seed_from_u64(0));
    let skipped = projection::add(&mut past, &mut SmallRng::seed_from_u64(0));

    // Assert
    assert_eq!(added, Ok(()));
    assert_eq!(graph(&last).projections.len(), 1);
    assert_eq!(graph(&last).projections[0].ref_idx, u16::MAX);
    assert_eq!(skipped, Err(MutationSkipReason::NoApplicableTarget));
    assert!(graph(&past).projections.is_empty());
}

/// A stored offset is read mod 8, so the largest `u8` shifts like 7.
#[test]
fn shift_reads_any_stored_offset_mod_eight() {
    for seed in 0..16 {
        // Arrange
        let mut node = graph_node(
            vec![barrier()],
            vec![RingProjection::new(0, DirectedVoteKind::Move, u8::MAX, 0.5)],
        );

        // Act
        projection::shift_offset(graph_mut(&mut node), &mut SmallRng::seed_from_u64(seed)).unwrap();

        // Assert: 7 + 1 and 7 − 1, mod 8.
        let offset = graph(&node).projections[0].offset;
        assert!(offset == 0 || offset == 6, "seed {seed}: {offset}");
    }
}
