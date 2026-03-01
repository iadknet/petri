use slotmap::SlotMap;
use v3_core::contracts::{
    CreatureId, Direction, DynamicIntrospectionKey, InputReference, NodeId, Position,
    StaticIntrospectionKey, WorldAction, WorldInputKey,
};
use v3_core::creature::genome::{
    BackendDef, CreatureGenome, GraphBackendDef, GraphInput, GraphInternalNode, GraphNodeKind,
    NodeGenome,
};
use v3_core::creature::state::CreatureState;
use v3_core::kernel::WorldState;
use v3_core::simulation::Simulation;

use crate::support::{
    graph_hop, insert_creature, run_one_traced_tick, test_config, vm_emit_noop_genome,
};

#[test]
fn graph_reads_all_neighbor_sensor_directions_e2e() {
    let mut cfg = test_config();
    cfg.runtime.max_graph_relax_iters = 1;
    cfg.runtime.graph_convergence_stable_passes = 1;

    let mut world = WorldState::new(cfg.world.width, cfg.world.height, cfg.world.edge_mode);
    let target_pos = Position::new(6, 6);
    world.set_food(target_pos, 0.9);

    let mut creatures: SlotMap<CreatureId, CreatureState> = SlotMap::with_key();

    // Fill all 8 neighbor cells with known sensor patterns:
    // - food: unique per direction (0.1, 0.2, ... 0.8)
    // - barrier: 1 for even direction index, 0 for odd
    // - occupied: 1 for odd direction index, 0 for even
    for dir in Direction::ALL {
        let idx = dir.to_index();
        let npos = world
            .resolve_neighbor(target_pos, dir)
            .expect("center position should have all neighbors");
        world.set_food(npos, (idx as f32 + 1.0) / 10.0);

        if idx % 2 == 0 {
            world.set_barrier(npos, true);
        } else {
            insert_creature(
                &mut creatures,
                &mut world,
                vm_emit_noop_genome(),
                npos,
                50.0,
                0,
            );
        }
    }

    let mut input_refs = vec![InputReference::World(WorldInputKey::FoodHere)];
    for dir in Direction::ALL {
        input_refs.push(InputReference::World(WorldInputKey::NeighborCellFood(dir)));
    }
    for dir in Direction::ALL {
        input_refs.push(InputReference::World(WorldInputKey::NeighborCellBarrier(
            dir,
        )));
    }
    for dir in Direction::ALL {
        input_refs.push(InputReference::World(WorldInputKey::NeighborCellOccupied(
            dir,
        )));
    }

    let internal_nodes: Vec<GraphInternalNode> = (0..input_refs.len())
        .map(|i| GraphInternalNode {
            kind: GraphNodeKind::InputRef {
                ref_idx: i as u16,
                sub_idx: 0,
            },
            inputs: vec![],
            plasticity: None,
        })
        .collect();

    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs,
            backend_def: BackendDef::Graph(GraphBackendDef { internal_nodes }),
            targets: vec![],
        }],
    };

    let target = insert_creature(&mut creatures, &mut world, genome, target_pos, 120.0, 0);
    let mut sim = Simulation::new(world, creatures, 0, cfg, 31);
    let tick = run_one_traced_tick(&mut sim, target);

    assert_eq!(tick.final_actions[0], WorldAction::NoOp);
    assert_eq!(tick.hops.len(), 1);
    let gtrace = graph_hop(&tick, 0);
    assert_eq!(gtrace.passes.len(), 1, "forced to single graph pass");
    let evals = &gtrace.passes[0].node_evaluations;
    assert_eq!(
        evals.len(),
        25,
        "FoodHere + 8 neighbor food + 8 barrier + 8 occupied"
    );

    // slot 0: FoodHere
    assert!((evals[0].output - 0.9).abs() < 1e-6, "FoodHere");

    // slots 1..=8: NeighborCellFood per direction index
    for dir in Direction::ALL {
        let idx = dir.to_index();
        let expected_food = (idx as f32 + 1.0) / 10.0;
        assert!(
            (evals[1 + idx].output - expected_food).abs() < 1e-6,
            "NeighborCellFood({dir:?})"
        );
    }

    // slots 9..=16: NeighborCellBarrier per direction index
    for dir in Direction::ALL {
        let idx = dir.to_index();
        let expected = if idx % 2 == 0 { 1.0 } else { 0.0 };
        assert!(
            (evals[9 + idx].output - expected).abs() < 1e-6,
            "NeighborCellBarrier({dir:?})"
        );
    }

    // slots 17..=24: NeighborCellOccupied per direction index
    for dir in Direction::ALL {
        let idx = dir.to_index();
        let expected = if idx % 2 == 1 { 1.0 } else { 0.0 };
        assert!(
            (evals[17 + idx].output - expected).abs() < 1e-6,
            "NeighborCellOccupied({dir:?})"
        );
    }
}

#[test]
fn graph_reads_inputs_and_writes_outputs_e2e() {
    let cfg = test_config();
    let mut world = WorldState::new(cfg.world.width, cfg.world.height, cfg.world.edge_mode);

    let target_pos = Position::new(5, 5);
    let north = Position::new(5, 4);
    let east = Position::new(6, 5);
    let west = Position::new(4, 5);
    world.set_food(target_pos, 0.6);
    world.set_food(north, 0.25);
    world.set_barrier(east, true);

    let input_refs = vec![
        InputReference::World(WorldInputKey::FoodHere), // slot 0
        InputReference::World(WorldInputKey::NeighborCellFood(Direction::N)), // slot 1
        InputReference::World(WorldInputKey::NeighborCellBarrier(Direction::E)), // slot 2
        InputReference::World(WorldInputKey::NeighborCellOccupied(Direction::W)), // slot 3
        InputReference::StaticIntrospection(StaticIntrospectionKey::Generation), // slot 4
        InputReference::StaticIntrospection(StaticIntrospectionKey::AgeTicks), // slot 5
        InputReference::DynamicIntrospection(DynamicIntrospectionKey::EnergyCurrent), // slot 6
        InputReference::DynamicIntrospection(DynamicIntrospectionKey::EnergyConsumedThisTick), // slot 7
        InputReference::UpstreamSlot(3), // slot 8
    ];

    let mut internal_nodes: Vec<GraphInternalNode> = Vec::new();
    for i in 0..input_refs.len() {
        let input_node_idx = (i * 2) as u16;
        internal_nodes.push(GraphInternalNode {
            kind: GraphNodeKind::InputRef {
                ref_idx: i as u16,
                sub_idx: 0,
            },
            inputs: vec![],
            plasticity: None,
        });
        internal_nodes.push(GraphInternalNode {
            kind: GraphNodeKind::CustomOutput(i as u8),
            inputs: vec![GraphInput {
                source_idx: input_node_idx,
                weight: 1.0,
            }],
            plasticity: None,
        });
    }

    let target_genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs,
            backend_def: BackendDef::Graph(GraphBackendDef { internal_nodes }),
            targets: vec![],
        }],
    };

    let mut creatures: SlotMap<CreatureId, CreatureState> = SlotMap::with_key();
    // West neighbor occupancy source.
    let _blocker = insert_creature(
        &mut creatures,
        &mut world,
        vm_emit_noop_genome(),
        west,
        50.0,
        0,
    );
    let target = insert_creature(
        &mut creatures,
        &mut world,
        target_genome,
        target_pos,
        120.0,
        7,
    );
    creatures.get_mut(target).expect("target creature").age = 3;

    let mut sim = Simulation::new(world, creatures, 0, cfg, 7);
    let tick = run_one_traced_tick(&mut sim, target);

    assert_eq!(tick.final_actions[0], WorldAction::NoOp);
    assert_eq!(tick.hops.len(), 1);
    assert!(!graph_hop(&tick, 0).passes.is_empty());

    let out = tick.hops[0].output_slots;
    assert!((out[0] - 0.6).abs() < 1e-6, "FoodHere");
    assert!((out[1] - 0.25).abs() < 1e-6, "NeighborFood(N)");
    assert!((out[2] - 1.0).abs() < 1e-6, "NeighborBarrier(E)");
    assert!((out[3] - 1.0).abs() < 1e-6, "NeighborOccupied(W)");
    assert!((out[4] - 7.0).abs() < 1e-6, "Generation");
    assert!(
        (out[5] - 4.0).abs() < 1e-6,
        "AgeTicks after Phase 0 increment"
    );
    assert!(out[6] > 0.0, "EnergyCurrent should be positive");
    assert!(
        (out[7] - 0.0).abs() < 1e-6,
        "EnergyConsumedThisTick first hop"
    );
    assert!(
        (out[8] - 0.0).abs() < 1e-6,
        "UpstreamSlot on entry should be zero"
    );
}
