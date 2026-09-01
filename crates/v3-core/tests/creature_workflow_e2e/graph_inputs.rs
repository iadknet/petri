use slotmap::SlotMap;
use v3_core::config::MutationConfig;
use v3_core::contracts::{
    CreatureId, Direction, DynamicIntrospectionKey, InputReference, NodeId, Position,
    StaticIntrospectionKey, WorldAction, WorldInputKey,
};
use v3_core::creature::genome::cgp::{
    CgpGraphBackendDef, ComputeNode, ComputeNodeKind, GraphEdge, GraphSource, OutputSinkKind,
};
use v3_core::creature::genome::{BackendDef, CreatureGenome, NodeGenome};
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
    world.reconfigure_food(cfg.world.food.clone());
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

    let input_refs = vec![
        InputReference::World(WorldInputKey::FoodHere {
            type_idx: v3_core::config::OrdinaryFoodTypeId::default(),
        }), // ref 0: scalar
        InputReference::World(WorldInputKey::NeighborFoodRing {
            type_idx: v3_core::config::OrdinaryFoodTypeId::default(),
        }), // ref 1: compound(8)
        InputReference::World(WorldInputKey::NeighborBarrierRing), // ref 2: compound(8)
        InputReference::World(WorldInputKey::NeighborOccupiedRing), // ref 3: compound(8)
    ];

    // Build 25 compute nodes: each reads an InputLeaf via a single edge on an Add node.
    // CN0: FoodHere (scalar)
    // CN1..=8: NeighborFoodRing, sub_idx = direction index 0..7
    // CN9..=16: NeighborBarrierRing, sub_idx = direction index 0..7
    // CN17..=24: NeighborOccupiedRing, sub_idx = direction index 0..7
    let graph_def = {
        let config = MutationConfig::default();
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs(&config);

        // CN0: FoodHere
        def.compute_nodes.push(ComputeNode {
            kind: ComputeNodeKind::Add,
            inputs: vec![GraphEdge {
                source: GraphSource::InputLeaf {
                    ref_idx: 0,
                    sub_idx: 0,
                },
                weight: 1.0,
            }],
            plasticity: None,
        });
        // CN1..=8: NeighborFoodRing per direction
        for dir_idx in 0u16..8 {
            def.compute_nodes.push(ComputeNode {
                kind: ComputeNodeKind::Add,
                inputs: vec![GraphEdge {
                    source: GraphSource::InputLeaf {
                        ref_idx: 1,
                        sub_idx: dir_idx,
                    },
                    weight: 1.0,
                }],
                plasticity: None,
            });
        }
        // CN9..=16: NeighborBarrierRing per direction
        for dir_idx in 0u16..8 {
            def.compute_nodes.push(ComputeNode {
                kind: ComputeNodeKind::Add,
                inputs: vec![GraphEdge {
                    source: GraphSource::InputLeaf {
                        ref_idx: 2,
                        sub_idx: dir_idx,
                    },
                    weight: 1.0,
                }],
                plasticity: None,
            });
        }
        // CN17..=24: NeighborOccupiedRing per direction
        for dir_idx in 0u16..8 {
            def.compute_nodes.push(ComputeNode {
                kind: ComputeNodeKind::Add,
                inputs: vec![GraphEdge {
                    source: GraphSource::InputLeaf {
                        ref_idx: 3,
                        sub_idx: dir_idx,
                    },
                    weight: 1.0,
                }],
                plasticity: None,
            });
        }
        def
    };

    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs,
            backend_def: BackendDef::Graph(graph_def),
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

    // slots 1..=8: NeighborFoodRing per direction index
    for dir in Direction::ALL {
        let idx = dir.to_index();
        let expected_food = if idx % 2 == 0 {
            0.0
        } else {
            (idx as f32 + 1.0) / 10.0
        };
        assert!(
            (evals[1 + idx].output - expected_food).abs() < 1e-6,
            "NeighborFoodRing[{dir:?}]"
        );
    }

    // slots 9..=16: NeighborBarrierRing per direction index
    for dir in Direction::ALL {
        let idx = dir.to_index();
        let expected = if idx % 2 == 0 { 1.0 } else { 0.0 };
        assert!(
            (evals[9 + idx].output - expected).abs() < 1e-6,
            "NeighborBarrierRing[{dir:?}]"
        );
    }

    // slots 17..=24: NeighborOccupiedRing per direction index
    for dir in Direction::ALL {
        let idx = dir.to_index();
        let expected = if idx % 2 == 1 { 1.0 } else { 0.0 };
        assert!(
            (evals[17 + idx].output - expected).abs() < 1e-6,
            "NeighborOccupiedRing[{dir:?}]"
        );
    }
}

#[test]
fn graph_reads_inputs_and_writes_outputs_e2e() {
    let cfg = test_config();
    let mut world = WorldState::new(cfg.world.width, cfg.world.height, cfg.world.edge_mode);
    world.reconfigure_food(cfg.world.food.clone());

    let target_pos = Position::new(5, 5);
    let north = Position::new(5, 4);
    let east = Position::new(6, 5);
    let west = Position::new(4, 5);
    world.set_food(target_pos, 0.6);
    world.set_food(north, 0.25);
    world.set_barrier(east, true);

    let input_refs = vec![
        InputReference::World(WorldInputKey::FoodHere {
            type_idx: v3_core::config::OrdinaryFoodTypeId::default(),
        }), // ref 0 (scalar)
        InputReference::World(WorldInputKey::NeighborFoodRing {
            type_idx: v3_core::config::OrdinaryFoodTypeId::default(),
        }), // ref 1 (compound, sub_idx=N=0)
        InputReference::World(WorldInputKey::NeighborBarrierRing), // ref 2 (compound, sub_idx=E=2)
        InputReference::World(WorldInputKey::NeighborOccupiedRing), // ref 3 (compound, sub_idx=W=6)
        InputReference::StaticIntrospection(StaticIntrospectionKey::Generation), // ref 4
        InputReference::StaticIntrospection(StaticIntrospectionKey::AgeTicks), // ref 5
        InputReference::DynamicIntrospection(DynamicIntrospectionKey::EnergyCurrent), // ref 6
        InputReference::DynamicIntrospection(DynamicIntrospectionKey::EnergyConsumedThisTick), // ref 7
        InputReference::UpstreamSlot(3), // ref 8
    ];

    // sub_idx per input ref: compound ring inputs use the direction index,
    // scalar inputs use 0.
    let sub_indices: [u16; 9] = [
        0,                              // FoodHere: scalar
        Direction::N.to_index() as u16, // NeighborFoodRing → N=0
        Direction::E.to_index() as u16, // NeighborBarrierRing → E=2
        Direction::W.to_index() as u16, // NeighborOccupiedRing → W=6
        0,                              // Generation: scalar
        0,                              // AgeTicks: scalar
        0,                              // EnergyCurrent: scalar
        0,                              // EnergyConsumedThisTick: scalar
        0,                              // UpstreamSlot: scalar
    ];

    // Build CGP graph: one compute node per input ref (Add with InputLeaf edge),
    // each wired to the corresponding CustomOutput sink.
    let graph_def = {
        let config = MutationConfig::default();
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs(&config);

        for (i, &sub_idx) in sub_indices.iter().enumerate().take(input_refs.len()) {
            // Add compute node reading InputLeaf(ref_idx=i, sub_idx)
            def.compute_nodes.push(ComputeNode {
                kind: ComputeNodeKind::Add,
                inputs: vec![GraphEdge {
                    source: GraphSource::InputLeaf {
                        ref_idx: i as u16,
                        sub_idx,
                    },
                    weight: 1.0,
                }],
                plasticity: None,
            });
            // Wire CustomOutput(i) sink to the compute node we just added
            let cn_idx = def.compute_nodes.len() - 1;
            if let Some(sink) = def
                .output_sinks
                .iter_mut()
                .find(|s| s.kind == OutputSinkKind::CustomOutput(i as u8))
            {
                sink.inputs.push(GraphEdge {
                    source: GraphSource::ComputeNode(cn_idx as u16),
                    weight: 1.0,
                });
            }
        }
        def
    };

    let target_genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs,
            backend_def: BackendDef::Graph(graph_def),
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
    assert!((out[1] - 0.25).abs() < 1e-6, "NeighborFoodRing[N]");
    assert!((out[2] - 1.0).abs() < 1e-6, "NeighborBarrierRing[E]");
    assert!((out[3] - 1.0).abs() < 1e-6, "NeighborOccupiedRing[W]");
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
