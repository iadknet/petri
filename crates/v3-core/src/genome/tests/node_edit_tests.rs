//! T25.F11: the node's declaration edits and the plan enumeration.
use crate::config::test_sensor_context;
use crate::contracts::votes::{DirectedVoteKind, VoteSink};
use crate::contracts::{FieldSet, InputReference, NodeId, WorldInputKey, MEMORY_BANK};
use crate::genome::cgp::{
    plans, test_support::edit_sinks, BundleBirth, BundleSource, CgpGraphBackendDef, GraphEdge,
    GraphSource, OutputSinkKind, Wiring,
};
use crate::genome::{BackendDef, NodeEditError, NodeGenome, VmBackendDef, VmInstruction};

fn graph_node(input_refs: Vec<InputReference>, def: CgpGraphBackendDef) -> NodeGenome {
    NodeGenome {
        node_id: NodeId::new(0),
        input_refs,
        backend_def: BackendDef::Graph(def),
        targets: vec![],
    }
}

fn vm_node() -> NodeGenome {
    NodeGenome {
        node_id: NodeId::new(0),
        input_refs: vec![],
        backend_def: BackendDef::Vm(VmBackendDef {
            register_count: 1,
            constants: vec![],
            program: vec![VmInstruction::Halt],
        }),
        targets: vec![],
    }
}

fn barrier_ring() -> InputReference {
    InputReference::World(WorldInputKey::NeighborBarrierRing)
}

fn ring_birth(kind: DirectedVoteKind, offset: u8) -> BundleBirth {
    BundleBirth {
        field_set: 0,
        wiring: Wiring::Ring { kind, offset },
        scale: 0.0,
    }
}

#[test]
fn plans_of_a_plain_set_are_every_present_vote_sink_in_catalog_order() {
    let def = CgpGraphBackendDef::new_with_fixed_outputs();
    let expected: Vec<Wiring> = def
        .output_sinks()
        .iter()
        .filter_map(|sink| match sink.kind {
            OutputSinkKind::ActionVote(sink) => Some(Wiring::Plain { sink }),
            _ => None,
        })
        .collect();
    assert_eq!(plans(&def, &MEMORY_BANK), expected);
    assert_eq!(expected.len(), crate::contracts::votes::VOTE_SINK_COUNT);
}

#[test]
fn plans_of_a_geometric_set_are_hostable_kinds_by_eight_offsets() {
    let full = CgpGraphBackendDef::new_with_fixed_outputs();
    let ring = plans(&full, &FieldSet::Ring);
    assert_eq!(ring.len(), DirectedVoteKind::ALL.len() * 8);
    assert_eq!(
        ring[..2],
        [
            Wiring::Ring {
                kind: DirectedVoteKind::Move,
                offset: 0
            },
            Wiring::Ring {
                kind: DirectedVoteKind::Move,
                offset: 1
            }
        ]
    );
    // Without Move's drive a pair cannot be wired to Move, a ring still can.
    let mut no_drive = full.clone();
    edit_sinks(&mut no_drive, |sinks| {
        sinks.retain(|sink| {
            sink.kind != OutputSinkKind::ActionVote(VoteSink::Drive(DirectedVoteKind::Move))
        });
    });
    let pair = InputReference::World(WorldInputKey::AreaBarrierSummary);
    let set = crate::contracts::SensorDescriptor::of(&pair, &test_sensor_context(1))
        .field_set(0)
        .unwrap();
    let pairs = plans(&no_drive, &set);
    assert_eq!(pairs.len(), (DirectedVoteKind::ALL.len() - 1) * 8);
    assert!(pairs.iter().all(|wiring| !matches!(
        wiring,
        Wiring::DirectionPair {
            kind: DirectedVoteKind::Move,
            ..
        }
    )));
    assert_eq!(plans(&no_drive, &FieldSet::Ring).len(), ring.len());
}

#[test]
fn add_sensor_declares_and_wires_one_bundle_per_field_set() {
    let context = test_sensor_context(1);
    let mut node = graph_node(vec![], CgpGraphBackendDef::new_with_fixed_outputs());
    let ref_idx = node
        .add_sensor(
            barrier_ring(),
            &[ring_birth(DirectedVoteKind::Reproduce, 3)],
            &context,
        )
        .unwrap();
    assert_eq!(ref_idx, 0);
    assert_eq!(node.input_refs, vec![barrier_ring()]);
    let BackendDef::Graph(def) = &node.backend_def else {
        unreachable!()
    };
    assert_eq!(def.bundles().len(), 1);
    assert_eq!(def.members(crate::genome::cgp::BundleId::new(0)).count(), 8);
}

#[test]
fn add_sensor_on_a_vm_node_takes_no_births() {
    let context = test_sensor_context(1);
    let mut node = vm_node();
    assert_eq!(node.add_sensor(barrier_ring(), &[], &context), Ok(0));
    let before = node.clone();
    assert_eq!(
        node.add_sensor(
            barrier_ring(),
            &[ring_birth(DirectedVoteKind::Move, 0)],
            &context
        ),
        Err(NodeEditError::Coverage)
    );
    assert_eq!(node, before);
}

#[test]
fn add_sensor_refuses_a_partial_cover_or_an_inadmissible_wiring() {
    let context = test_sensor_context(1);
    let mut node = graph_node(vec![], CgpGraphBackendDef::new_with_fixed_outputs());
    let before = node.clone();
    let area = InputReference::World(WorldInputKey::AreaBarrierSummary);
    let pair = BundleBirth {
        field_set: 0,
        wiring: Wiring::DirectionPair {
            kind: DirectedVoteKind::Move,
            offset: 0,
        },
        scale: 0.0,
    };
    assert_eq!(
        node.add_sensor(area, &[pair], &context),
        Err(NodeEditError::Coverage),
        "an area summary declares two field sets"
    );
    assert_eq!(
        node.add_sensor(barrier_ring(), &[pair], &context),
        Err(NodeEditError::Inadmissible),
        "a ring set takes no pair wiring"
    );
    assert_eq!(node, before);
}

/// `add_sensor` one record below capacity refuses two births and writes
/// nothing (T25.F11 invariant 5).
#[test]
fn add_sensor_one_record_below_capacity_returns_err_and_writes_nothing() {
    let context = test_sensor_context(1);
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    def.exhaust_bundle_capacity();
    def.drop_last_bundle_record();
    let mut node = graph_node(vec![], def);
    let before = node.clone();
    let area = InputReference::World(WorldInputKey::AreaBarrierSummary);
    let births = [0u8, 1].map(|field_set| BundleBirth {
        field_set,
        wiring: Wiring::DirectionPair {
            kind: DirectedVoteKind::Move,
            offset: 0,
        },
        scale: 0.0,
    });
    assert_eq!(
        node.add_sensor(area, &births, &context),
        Err(NodeEditError::BundleCapacity)
    );
    assert_eq!(node, before);
}

#[test]
fn add_bundle_refuses_an_unknown_source_and_a_mismatched_set() {
    let context = test_sensor_context(1);
    let mut node = graph_node(
        vec![barrier_ring()],
        CgpGraphBackendDef::new_with_fixed_outputs(),
    );
    let before = node.clone();
    let missing = BundleSource::Sensor {
        ref_idx: 1,
        field_set: 0,
    };
    assert_eq!(
        node.add_bundle(missing, ring_birth(DirectedVoteKind::Move, 0), &context),
        Err(NodeEditError::UnknownSource)
    );
    let mismatched = BundleBirth {
        field_set: 1,
        ..ring_birth(DirectedVoteKind::Move, 0)
    };
    let ring = BundleSource::Sensor {
        ref_idx: 0,
        field_set: 0,
    };
    assert_eq!(
        node.add_bundle(ring, mismatched, &context),
        Err(NodeEditError::UnknownSource)
    );
    assert_eq!(node, before);
    assert!(node
        .add_bundle(ring, ring_birth(DirectedVoteKind::Move, 0), &context)
        .is_ok());
}

#[test]
fn prune_refuses_an_addressed_declaration_and_renumbers_the_rest() {
    let leaf = |ref_idx| {
        GraphEdge::new(
            GraphSource::InputLeaf {
                ref_idx,
                sub_idx: 0,
            },
            1.0,
        )
    };
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    let surface = def
        .sink_surface(OutputSinkKind::ActionVote(VoteSink::Eat))
        .unwrap();
    def.push_edge(surface, leaf(2));
    let mut node = graph_node(
        vec![InputReference::ActionVotes, barrier_ring(), barrier_ring()],
        def,
    );
    assert_eq!(node.prune(2), Err(NodeEditError::Addressed));
    assert_eq!(node.prune(3), Err(NodeEditError::OutOfRange));
    assert_eq!(node.prune(1), Ok(()));
    assert_eq!(node.input_refs.len(), 2);
    assert!(node.addressed_input_refs().contains(&1));
}

#[test]
fn swap_refuses_another_kind_and_writes_a_same_kind_replacement() {
    let context = test_sensor_context(2);
    let food = |t| {
        InputReference::World(WorldInputKey::food_here(
            crate::config::OrdinaryFoodTypeId::new(t),
        ))
    };
    let mut node = graph_node(vec![food(0)], CgpGraphBackendDef::new_with_fixed_outputs());
    assert_eq!(
        node.swap(0, barrier_ring(), &context),
        Err(NodeEditError::KindMismatch)
    );
    assert_eq!(
        node.swap(1, food(1), &context),
        Err(NodeEditError::OutOfRange)
    );
    assert_eq!(node.swap(0, food(1), &context), Ok(()));
    assert_eq!(node.input_refs, vec![food(1)]);
}
