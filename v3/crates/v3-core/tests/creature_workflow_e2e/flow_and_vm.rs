use slotmap::SlotMap;
use v3_core::contracts::{
    CreatureId, Direction, DynamicIntrospectionKey, InputReference, NodeId, Position,
    StaticIntrospectionKey, WorldAction, WorldInputKey,
};
use v3_core::creature::genome::{
    BackendDef, CreatureGenome, GraphBackendDef, GraphInput, GraphInternalNode, GraphNodeKind,
    NodeGenome, VmBackendDef, VmInstruction,
};
use v3_core::creature::state::CreatureState;
use v3_core::kernel::WorldState;
use v3_core::runtime::trace::BackendTrace;
use v3_core::simulation::Simulation;

use crate::support::{
    insert_creature, run_one_traced_tick, test_config, vm_emit_noop_genome, vm_hop,
};

#[test]
fn outputs_flow_graph_to_graph_to_vm_with_sensor_reads_e2e() {
    let cfg = test_config();
    let mut world = WorldState::new(cfg.world.width, cfg.world.height, cfg.world.edge_mode);
    let pos = Position::new(3, 3);
    world.set_food(pos, 0.5);

    let id_a = NodeId::new(0);
    let id_b = NodeId::new(1);
    let id_c = NodeId::new(2);

    // Node A: writes 0.8 into slot 2, routes to B.
    let node_a = NodeGenome {
        node_id: id_a,
        input_refs: vec![],
        backend_def: BackendDef::Graph(GraphBackendDef {
            internal_nodes: vec![
                GraphInternalNode {
                    kind: GraphNodeKind::Constant(0.8),
                    inputs: vec![],
                    hebbian: None,
                },
                GraphInternalNode {
                    kind: GraphNodeKind::CustomOutput(2),
                    inputs: vec![GraphInput {
                        source_idx: 0,
                        weight: 1.0,
                    }],
                    hebbian: None,
                },
                GraphInternalNode {
                    kind: GraphNodeKind::RouterOutput,
                    inputs: vec![],
                    hebbian: None,
                },
            ],
        }),
        targets: vec![id_b],
    };

    // Node B: reads upstream slot 2 and writes it into slot 4, routes to C.
    let node_b = NodeGenome {
        node_id: id_b,
        input_refs: vec![InputReference::UpstreamSlot(2)],
        backend_def: BackendDef::Graph(GraphBackendDef {
            internal_nodes: vec![
                GraphInternalNode {
                    kind: GraphNodeKind::InputRef {
                        ref_idx: 0,
                        sub_idx: 0,
                    },
                    inputs: vec![],
                    hebbian: None,
                },
                GraphInternalNode {
                    kind: GraphNodeKind::CustomOutput(4),
                    inputs: vec![GraphInput {
                        source_idx: 0,
                        weight: 1.0,
                    }],
                    hebbian: None,
                },
                GraphInternalNode {
                    kind: GraphNodeKind::RouterOutput,
                    inputs: vec![],
                    hebbian: None,
                },
            ],
        }),
        targets: vec![id_c],
    };

    // Node C (VM): reads upstream slot 4 + FoodHere; emits Eat when sum > 1.2.
    let node_c = NodeGenome {
        node_id: id_c,
        input_refs: vec![
            InputReference::UpstreamSlot(4),
            InputReference::World(WorldInputKey::FoodHere),
        ],
        backend_def: BackendDef::Vm(VmBackendDef {
            register_count: 4,
            constants: vec![1.2],
            program: vec![
                VmInstruction::ReadInput {
                    dst: 0,
                    ref_idx: 0,
                    sub_idx: 0,
                },
                VmInstruction::ReadInput {
                    dst: 1,
                    ref_idx: 1,
                    sub_idx: 0,
                },
                VmInstruction::Add { dst: 2, a: 0, b: 1 },
                VmInstruction::LoadConst {
                    dst: 3,
                    const_idx: 0,
                },
                VmInstruction::CmpGt { dst: 2, a: 2, b: 3 },
                VmInstruction::JumpIfZero { cond: 2, offset: 2 },
                VmInstruction::PushAction { action_type: 1 }, // Eat
                VmInstruction::ExecuteActionQueue,
                VmInstruction::PushAction { action_type: 0 }, // NoOp fallback
                VmInstruction::ExecuteActionQueue,
            ],
        }),
        targets: vec![],
    };

    let genome = CreatureGenome {
        entry_node_id: id_a,
        nodes: vec![node_a, node_b, node_c],
    };

    let mut creatures: SlotMap<CreatureId, CreatureState> = SlotMap::with_key();
    let target = insert_creature(&mut creatures, &mut world, genome, pos, 100.0, 0);
    let mut sim = Simulation::new(world, creatures, 0, cfg, 13);

    let tick = run_one_traced_tick(&mut sim, target);

    assert_eq!(tick.final_actions[0], WorldAction::Eat);
    assert_eq!(tick.hops.len(), 3);
    assert!(matches!(tick.hops[0].backend_trace, BackendTrace::Graph(_)));
    assert!(matches!(tick.hops[1].backend_trace, BackendTrace::Graph(_)));
    assert!(matches!(tick.hops[2].backend_trace, BackendTrace::Vm(_)));
    assert!(!vm_hop(&tick, 2).steps.is_empty());

    assert!((tick.hops[0].output_slots[2] - 0.8).abs() < 1e-6);
    assert!((tick.hops[1].upstream_slots[2] - 0.8).abs() < 1e-6);
    assert!((tick.hops[1].output_slots[4] - 0.8).abs() < 1e-6);
    assert!((tick.hops[2].upstream_slots[4] - 0.8).abs() < 1e-6);
    assert!(
        (sim.world.food_at(pos) - 0.0).abs() < 1e-6,
        "Eat action from VM should consume food on current cell"
    );
}

#[test]
fn vm_reads_all_inputs_e2e() {
    let cfg = test_config();
    let mut world = WorldState::new(cfg.world.width, cfg.world.height, cfg.world.edge_mode);
    let pos = Position::new(6, 6);
    world.set_food(pos, 0.91);

    let mut creatures: SlotMap<CreatureId, CreatureState> = SlotMap::with_key();

    for dir in Direction::ALL {
        let idx = dir.to_index();
        let npos = world
            .resolve_neighbor(pos, dir)
            .expect("center position should have all neighbors");

        world.set_food(npos, 0.11 + (idx as f32 * 0.07));
        if idx % 2 == 0 {
            world.set_barrier(npos, true);
        } else {
            insert_creature(
                &mut creatures,
                &mut world,
                vm_emit_noop_genome(),
                npos,
                40.0,
                0,
            );
        }
    }

    let id_graph = NodeId::new(0);
    let id_vm = NodeId::new(1);

    let graph_node = NodeGenome {
        node_id: id_graph,
        input_refs: vec![],
        backend_def: BackendDef::Graph(GraphBackendDef {
            internal_nodes: vec![
                GraphInternalNode {
                    kind: GraphNodeKind::Constant(0.73),
                    inputs: vec![],
                    hebbian: None,
                },
                GraphInternalNode {
                    kind: GraphNodeKind::CustomOutput(11),
                    inputs: vec![GraphInput {
                        source_idx: 0,
                        weight: 1.0,
                    }],
                    hebbian: None,
                },
                GraphInternalNode {
                    kind: GraphNodeKind::RouterOutput,
                    inputs: vec![],
                    hebbian: None,
                },
            ],
        }),
        targets: vec![id_vm],
    };

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
    input_refs.push(InputReference::StaticIntrospection(
        StaticIntrospectionKey::Generation,
    ));
    input_refs.push(InputReference::StaticIntrospection(
        StaticIntrospectionKey::AgeTicks,
    ));
    input_refs.push(InputReference::DynamicIntrospection(
        DynamicIntrospectionKey::EnergyCurrent,
    ));
    input_refs.push(InputReference::DynamicIntrospection(
        DynamicIntrospectionKey::EnergyConsumedThisTick,
    ));
    input_refs.push(InputReference::UpstreamSlot(11));

    let mut program = Vec::new();
    for idx in 0..input_refs.len() {
        program.push(VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        });
        program.push(VmInstruction::ReadInput {
            dst: 0,
            ref_idx: idx as u16,
            sub_idx: 0,
        });
    }
    program.push(VmInstruction::PushAction { action_type: 0 });
    program.push(VmInstruction::ExecuteActionQueue);

    let vm_node = NodeGenome {
        node_id: id_vm,
        input_refs: input_refs.clone(),
        backend_def: BackendDef::Vm(VmBackendDef {
            register_count: 1,
            constants: vec![99.0],
            program,
        }),
        targets: vec![],
    };

    let genome = CreatureGenome {
        entry_node_id: id_graph,
        nodes: vec![graph_node, vm_node],
    };

    let target = insert_creature(&mut creatures, &mut world, genome, pos, 150.0, 9);
    creatures.get_mut(target).expect("target creature").age = 2;

    let mut sim = Simulation::new(world, creatures, 0, cfg, 17);
    let tick = run_one_traced_tick(&mut sim, target);

    assert_eq!(tick.hops.len(), 2);
    assert_eq!(tick.final_actions[0], WorldAction::NoOp);
    assert!(matches!(tick.hops[0].backend_trace, BackendTrace::Graph(_)));
    assert!(matches!(tick.hops[1].backend_trace, BackendTrace::Vm(_)));
    assert!((tick.hops[1].upstream_slots[11] - 0.73).abs() < 1e-6);

    let trace = vm_hop(&tick, 1);
    let mut observed = vec![f32::NAN; input_refs.len()];
    let mut seen_reads = 0usize;

    for step in &trace.steps {
        if let VmInstruction::ReadInput { ref_idx, .. } = &step.instruction {
            let (_, value) = step
                .register_changes
                .iter()
                .find(|(reg, _)| *reg == 0)
                .copied()
                .expect("ReadInput should update register 0");
            observed[*ref_idx as usize] = value;
            seen_reads += 1;
        }
    }

    assert_eq!(seen_reads, input_refs.len());

    assert!((observed[0] - 0.91).abs() < 1e-6, "FoodHere");

    for dir in Direction::ALL {
        let idx = dir.to_index();
        let expected_food = 0.11 + (idx as f32 * 0.07);
        assert!(
            (observed[1 + idx] - expected_food).abs() < 1e-6,
            "NeighborCellFood({dir:?})"
        );
    }

    for dir in Direction::ALL {
        let idx = dir.to_index();
        let expected = if idx % 2 == 0 { 1.0 } else { 0.0 };
        assert!(
            (observed[9 + idx] - expected).abs() < 1e-6,
            "NeighborCellBarrier({dir:?})"
        );
    }

    for dir in Direction::ALL {
        let idx = dir.to_index();
        let expected = if idx % 2 == 1 { 1.0 } else { 0.0 };
        assert!(
            (observed[17 + idx] - expected).abs() < 1e-6,
            "NeighborCellOccupied({dir:?})"
        );
    }

    assert!((observed[25] - 9.0).abs() < 1e-6, "Generation");
    assert!((observed[26] - 3.0).abs() < 1e-6, "AgeTicks");

    let energy_current = observed[27];
    assert!(energy_current > 0.0, "EnergyCurrent should stay positive");
    assert!(
        energy_current < 150.0,
        "EnergyCurrent should stay below starting energy"
    );

    let energy_consumed = observed[28];
    assert!(
        energy_consumed > 0.0,
        "EnergyConsumedThisTick should include graph hop costs"
    );

    assert!((observed[29] - 0.73).abs() < 1e-6, "UpstreamSlot(11)");
}

#[test]
fn vm_uses_neighbor_barrier_sensor_to_choose_action_e2e() {
    for (north_barrier, expected_action, expected_barrier_value) in [
        (false, WorldAction::Eat, 0.0f32),
        (true, WorldAction::NoOp, 1.0f32),
    ] {
        let cfg = test_config();
        let mut world = WorldState::new(cfg.world.width, cfg.world.height, cfg.world.edge_mode);
        let pos = Position::new(6, 6);
        world.set_food(pos, 1.0);
        let north = world
            .resolve_neighbor(pos, Direction::N)
            .expect("center position should have north neighbor");
        world.set_barrier(north, north_barrier);

        let vm_node = NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![InputReference::World(WorldInputKey::NeighborCellBarrier(
                Direction::N,
            ))],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 3,
                constants: vec![99.0, 0.5],
                program: vec![
                    VmInstruction::LoadConst {
                        dst: 0,
                        const_idx: 0,
                    },
                    VmInstruction::ReadInput {
                        dst: 0,
                        ref_idx: 0,
                        sub_idx: 0,
                    },
                    VmInstruction::LoadConst {
                        dst: 1,
                        const_idx: 1,
                    },
                    VmInstruction::CmpGt { dst: 2, a: 0, b: 1 },
                    VmInstruction::JumpIfZero { cond: 2, offset: 2 },
                    VmInstruction::PushAction { action_type: 0 }, // NoOp when barrier present
                    VmInstruction::ExecuteActionQueue,
                    VmInstruction::PushAction { action_type: 1 }, // Eat when barrier absent
                    VmInstruction::ExecuteActionQueue,
                ],
            }),
            targets: vec![],
        };

        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![vm_node],
        };

        let mut creatures: SlotMap<CreatureId, CreatureState> = SlotMap::with_key();
        let target = insert_creature(&mut creatures, &mut world, genome, pos, 100.0, 0);
        let mut sim = Simulation::new(world, creatures, 0, cfg, 29);
        let tick = run_one_traced_tick(&mut sim, target);

        assert_eq!(tick.hops.len(), 1);
        assert_eq!(tick.final_actions[0], expected_action);

        let trace = vm_hop(&tick, 0);
        let read_step = trace
            .steps
            .iter()
            .find(|s| {
                matches!(
                    s.instruction,
                    VmInstruction::ReadInput {
                        dst: 0,
                        ref_idx: 0,
                        sub_idx: 0,
                    }
                )
            })
            .expect("missing ReadInput step");
        let (_, read_value) = read_step
            .register_changes
            .iter()
            .find(|(reg, _)| *reg == 0)
            .copied()
            .expect("ReadInput should update register 0");
        assert!(
            (read_value - expected_barrier_value).abs() < 1e-6,
            "VM should read NeighborCellBarrier(N) accurately"
        );
    }
}
