use slotmap::SlotMap;
use v3_core::config::MutationConfig;
use v3_core::contracts::{
    CreatureId, Direction, DynamicIntrospectionKey, InputReference, NodeId, Position,
    StaticIntrospectionKey, WorldAction, WorldInputKey,
};
use v3_core::creature::genome::cgp::{
    CgpGraphBackendDef, ComputeNode, ComputeNodeKind, GraphEdge, GraphSource, OutputSinkKind,
};
use v3_core::creature::genome::{
    BackendDef, CreatureGenome, NodeGenome, VmBackendDef, VmInstruction,
};
use v3_core::creature::state::CreatureState;
use v3_core::kernel::WorldState;
use v3_core::runtime::trace::domain::BackendTrace;
use v3_core::simulation::Simulation;

use crate::support::{
    insert_creature, run_one_traced_tick, test_config, vm_emit_noop_genome, vm_hop,
};

/// Build a CGP graph backend with a single Constant compute node whose output
/// is wired to the specified CustomOutput sink and optionally to RouterOutput.
fn cgp_constant_to_custom_output(value: f32, custom_slot: u8) -> CgpGraphBackendDef {
    let config = MutationConfig::default();
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs(&config);
    def.compute_nodes.push(ComputeNode {
        kind: ComputeNodeKind::Constant(value),
        inputs: Vec::new(),
        plasticity: None,
    });
    // Wire the CustomOutput(custom_slot) sink to CN0
    if let Some(sink) = def
        .output_sinks
        .iter_mut()
        .find(|s| s.kind == OutputSinkKind::CustomOutput(custom_slot))
    {
        sink.inputs.push(GraphEdge {
            source: GraphSource::ComputeNode(0),
            weight: 1.0,
        });
    }
    def
}

/// Build a CGP graph backend with a compute node that reads an InputLeaf and
/// writes the result to a CustomOutput sink.
fn cgp_passthrough_input_to_custom_output(
    ref_idx: u16,
    sub_idx: u16,
    custom_slot: u8,
) -> CgpGraphBackendDef {
    let config = MutationConfig::default();
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs(&config);
    // CN0: Add with single InputLeaf edge (acts as passthrough)
    def.compute_nodes.push(ComputeNode {
        kind: ComputeNodeKind::Add,
        inputs: vec![GraphEdge {
            source: GraphSource::InputLeaf { ref_idx, sub_idx },
            weight: 1.0,
        }],
        plasticity: None,
    });
    // Wire CustomOutput(custom_slot) to CN0
    if let Some(sink) = def
        .output_sinks
        .iter_mut()
        .find(|s| s.kind == OutputSinkKind::CustomOutput(custom_slot))
    {
        sink.inputs.push(GraphEdge {
            source: GraphSource::ComputeNode(0),
            weight: 1.0,
        });
    }
    def
}

#[test]
fn outputs_flow_graph_to_graph_to_vm_with_sensor_reads_e2e() {
    let cfg = test_config();
    let mut world = WorldState::new(cfg.world.width, cfg.world.height, cfg.world.edge_mode);
    let pos = Position::new(3, 3);
    world.set_food(pos, 0.5);

    let id_a = NodeId::new(0);
    let id_b = NodeId::new(1);
    let id_c = NodeId::new(2);

    // Node A: writes 0.8 into slot 2, routes to B (RouterOutput unwired → 0.0 → targets[0]).
    let node_a = NodeGenome {
        node_id: id_a,
        input_refs: vec![],
        backend_def: BackendDef::Graph(cgp_constant_to_custom_output(0.8, 2)),
        targets: vec![id_b],
    };

    // Node B: reads upstream slot 2 via InputLeaf and writes it into slot 4, routes to C.
    let node_b = NodeGenome {
        node_id: id_b,
        input_refs: vec![InputReference::UpstreamSlot(2)],
        backend_def: BackendDef::Graph(cgp_passthrough_input_to_custom_output(0, 0, 4)),
        targets: vec![id_c],
    };

    // Node C (VM): reads upstream slot 4 + FoodHere; emits Eat when sum > 1.2.
    let node_c = NodeGenome {
        node_id: id_c,
        input_refs: vec![
            InputReference::UpstreamSlot(4),
            InputReference::World(WorldInputKey::FoodHere {
                type_idx: v3_core::config::OrdinaryFoodTypeId::default(),
            }),
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

    assert_eq!(
        tick.final_actions[0],
        WorldAction::Eat {
            type_idx: v3_core::config::OrdinaryFoodTypeId::default()
        }
    );
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
        backend_def: BackendDef::Graph(cgp_constant_to_custom_output(0.73, 11)),
        targets: vec![id_vm],
    };

    // Ring sensors are compound (8 sub-values each, indexed by Direction::to_index()).
    // ref 0: FoodHere
    // ref 1: NeighborFoodRing (compound, 8 sub-values)
    // ref 2: NeighborBarrierRing (compound, 8 sub-values)
    // ref 3: NeighborOccupiedRing (compound, 8 sub-values)
    // ref 4: Generation
    // ref 5: AgeTicks
    // ref 6: EnergyCurrent
    // ref 7: EnergyConsumedThisTick
    // ref 8: UpstreamSlot(11)
    let input_refs = vec![
        InputReference::World(WorldInputKey::FoodHere {
            type_idx: v3_core::config::OrdinaryFoodTypeId::default(),
        }),
        InputReference::World(WorldInputKey::NeighborFoodRing {
            type_idx: v3_core::config::OrdinaryFoodTypeId::default(),
        }),
        InputReference::World(WorldInputKey::NeighborBarrierRing),
        InputReference::World(WorldInputKey::NeighborOccupiedRing),
        InputReference::StaticIntrospection(StaticIntrospectionKey::Generation),
        InputReference::StaticIntrospection(StaticIntrospectionKey::AgeTicks),
        InputReference::DynamicIntrospection(DynamicIntrospectionKey::EnergyCurrent),
        InputReference::DynamicIntrospection(DynamicIntrospectionKey::EnergyConsumedThisTick),
        InputReference::UpstreamSlot(11),
    ];

    // Build a program that reads every sub-value of every input ref.
    // For compound ring sensors (refs 1-3), read sub_idx 0..7.
    // For scalar refs (0, 4-8), read sub_idx 0.
    let mut program = Vec::new();
    // Track (ref_idx, sub_idx) order so we can match observed values later.
    let mut read_schedule: Vec<(u16, u16)> = Vec::new();
    for (ref_idx, iref) in input_refs.iter().enumerate() {
        let width = match iref {
            InputReference::World(k) => k.compound_width(),
            _ => 1,
        };
        for sub in 0..width {
            program.push(VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            });
            program.push(VmInstruction::ReadInput {
                dst: 0,
                ref_idx: ref_idx as u16,
                sub_idx: sub,
            });
            read_schedule.push((ref_idx as u16, sub));
        }
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
    // Collect all ReadInput results keyed by (ref_idx, sub_idx).
    let mut observed: std::collections::HashMap<(u16, u16), f32> = std::collections::HashMap::new();
    let mut seen_reads = 0usize;

    for step in &trace.steps {
        if let VmInstruction::ReadInput {
            ref_idx, sub_idx, ..
        } = &step.instruction
        {
            let (_, value) = step
                .register_changes
                .iter()
                .find(|(reg, _)| *reg == 0)
                .copied()
                .expect("ReadInput should update register 0");
            observed.insert((*ref_idx, *sub_idx), value);
            seen_reads += 1;
        }
    }

    assert_eq!(seen_reads, read_schedule.len());

    // ref 0: FoodHere (scalar)
    assert!((observed[&(0, 0)] - 0.91).abs() < 1e-6, "FoodHere");

    // ref 1: NeighborFoodRing (compound, 8 sub-values)
    for dir in Direction::ALL {
        let idx = dir.to_index();
        let expected_food = if idx % 2 == 0 {
            0.0
        } else {
            0.11 + (idx as f32 * 0.07)
        };
        assert!(
            (observed[&(1, idx as u16)] - expected_food).abs() < 1e-6,
            "NeighborFoodRing[{dir:?}]"
        );
    }

    // ref 2: NeighborBarrierRing (compound, 8 sub-values)
    for dir in Direction::ALL {
        let idx = dir.to_index();
        let expected = if idx % 2 == 0 { 1.0 } else { 0.0 };
        assert!(
            (observed[&(2, idx as u16)] - expected).abs() < 1e-6,
            "NeighborBarrierRing[{dir:?}]"
        );
    }

    // ref 3: NeighborOccupiedRing (compound, 8 sub-values)
    for dir in Direction::ALL {
        let idx = dir.to_index();
        let expected = if idx % 2 == 1 { 1.0 } else { 0.0 };
        assert!(
            (observed[&(3, idx as u16)] - expected).abs() < 1e-6,
            "NeighborOccupiedRing[{dir:?}]"
        );
    }

    // ref 4: Generation, ref 5: AgeTicks
    assert!((observed[&(4, 0)] - 9.0).abs() < 1e-6, "Generation");
    assert!((observed[&(5, 0)] - 3.0).abs() < 1e-6, "AgeTicks");

    let energy_current = observed[&(6, 0)];
    assert!(energy_current > 0.0, "EnergyCurrent should stay positive");
    assert!(
        energy_current < 150.0,
        "EnergyCurrent should stay below starting energy"
    );

    // ref 7: EnergyConsumedThisTick
    let energy_consumed = observed[&(7, 0)];
    assert!(
        energy_consumed > 0.0,
        "EnergyConsumedThisTick should include graph hop costs"
    );

    // ref 8: UpstreamSlot(11)
    assert!((observed[&(8, 0)] - 0.73).abs() < 1e-6, "UpstreamSlot(11)");
}

#[test]
fn vm_uses_neighbor_barrier_sensor_to_choose_action_e2e() {
    for (north_barrier, expected_action, expected_barrier_value) in [
        (
            false,
            WorldAction::Eat {
                type_idx: v3_core::config::OrdinaryFoodTypeId::default(),
            },
            0.0f32,
        ),
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
            input_refs: vec![InputReference::World(WorldInputKey::NeighborBarrierRing)],
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
            "VM should read NeighborBarrierRing[N] accurately"
        );
    }
}
