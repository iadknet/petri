use super::*;
use crate::config::EnergyLifecycleConfig;
use crate::creature::genome::vote::{VoteSink, VOTE_SINK_COUNT};
use crate::creature::identity::CreatureIdentityState;

// ── AddVote and WriteActionParam (T19.F04) ─────────────────────────────────

#[test]
fn add_vote_sums_into_the_staged_contribution_and_never_touches_the_queue() {
    let (_, _, mut side) = run_vm(
        vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            },
            VmInstruction::AddVote {
                sink: VoteSink::Eat.index() as u8,
                src: 0,
            },
            VmInstruction::AddVote {
                sink: VoteSink::Eat.index() as u8,
                src: 0,
            },
            // Past the catalog: writes nothing.
            VmInstruction::AddVote {
                sink: VOTE_SINK_COUNT as u8,
                src: 0,
            },
            VmInstruction::Halt,
        ],
        1,
        vec![1.5],
        &[],
        zeroed_upstream(),
        100.0,
    );
    assert!(side.action_queue.is_empty());
    let committed = side.commit_vote_contribution(0);
    assert_eq!(committed[VoteSink::Eat.index()], 3.0);
    assert_eq!(committed.iter().filter(|v| **v != 0.0).count(), 1);
}

#[test]
fn write_action_param_addresses_the_surface_kind_major() {
    for slot in 0u8..8 {
        let (_, _, side) = run_vm(
            vec![
                VmInstruction::LoadConst {
                    dst: 0,
                    const_idx: 0,
                },
                VmInstruction::WriteActionParam {
                    slot_idx: slot,
                    src: 0,
                },
                VmInstruction::Halt,
            ],
            1,
            vec![7.0],
            &[],
            zeroed_upstream(),
            100.0,
        );
        let mut expected = [[0.0f32; 2]; 4];
        expected[usize::from(slot / 2)][usize::from(slot % 2)] = 7.0;
        assert_eq!(side.action_params, expected, "slot {slot}");
    }
}

#[test]
fn write_action_param_overwrites_and_ignores_an_invalid_slot() {
    let (_, _, side) = run_vm(
        vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            },
            VmInstruction::WriteActionParam {
                slot_idx: 3,
                src: 0,
            },
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 1,
            },
            VmInstruction::WriteActionParam {
                slot_idx: 3,
                src: 0,
            },
            VmInstruction::WriteActionParam {
                slot_idx: 8,
                src: 0,
            },
            VmInstruction::WriteActionParam {
                slot_idx: 255,
                src: 0,
            },
            VmInstruction::Halt,
        ],
        1,
        vec![7.0, -2.0],
        &[],
        zeroed_upstream(),
        100.0,
    );
    let mut expected = [[0.0f32; 2]; 4];
    expected[1][1] = -2.0;
    assert_eq!(side.action_params, expected);
}

// ── ReadInput opcode ──────────────────────────────────────────────────────

#[test]
fn read_input_upstream_slot() {
    use crate::contracts::InputReference;
    let input_refs = vec![InputReference::UpstreamSlot(0)];
    let mut upstream = zeroed_upstream();
    upstream[0] = 42.0;
    let program = vec![
        VmInstruction::ReadInput {
            dst: 0,
            ref_idx: 0,
            sub_idx: 0,
        },
        VmInstruction::WriteInternalPayload {
            slot_idx: 0,
            src: 0,
        },
        VmInstruction::Halt,
    ];
    let def = VmBackendDef {
        register_count: 1,
        constants: vec![],
        program,
    };
    let ss = empty_sensor_snapshot();
    let mut e = 100.0;
    let mut mem = [0.0f32; 16];
    let prev_mem = [0.0f32; 16];
    let cfg = config();
    let mut side_outputs = MeshSideOutputs::new(cfg.max_actions_per_turn);
    let r = execute_vm_node(
        &def,
        &input_refs,
        &upstream,
        &mut e,
        0.0,
        &mut mem,
        &prev_mem,
        &ss,
        &cfg,
        &mut side_outputs,
    );
    assert!((r.output_slots[0] - 42.0).abs() < 1e-6);
}

#[test]
fn read_input_out_of_range_yields_zero() {
    let program = vec![
        VmInstruction::ReadInput {
            dst: 0,
            ref_idx: 5,
            sub_idx: 0,
        }, // no input_refs at all
        VmInstruction::WriteInternalPayload {
            slot_idx: 0,
            src: 0,
        },
        VmInstruction::Halt,
    ];
    let (r, _, _aq) = run_vm(program, 1, vec![], &[], zeroed_upstream(), 100.0);
    assert_eq!(r.output_slots[0], 0.0);
}

// ── Payload and routing ───────────────────────────────────────────────────

#[test]
fn payload_initialized_from_upstream_slots() {
    let mut upstream = zeroed_upstream();
    upstream[3] = 7.7;
    let program = vec![VmInstruction::Halt];
    let (r, _, _aq) = run_vm(program, 1, vec![], &[], upstream, 100.0);
    assert!((r.output_slots[3] - 7.7).abs() < 1e-6);
}

#[test]
fn write_internal_payload_updates_slot() {
    let program = vec![
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        },
        VmInstruction::WriteInternalPayload {
            slot_idx: 5,
            src: 0,
        },
        VmInstruction::Halt,
    ];
    let (r, _, _aq) = run_vm(program, 1, vec![3.5], &[], zeroed_upstream(), 100.0);
    assert!((r.output_slots[5] - 3.5).abs() < 1e-4);
}

#[test]
fn write_internal_payload_invalid_slot_ignored() {
    let program = vec![
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        },
        VmInstruction::WriteInternalPayload {
            slot_idx: OUTPUT_SLOT_COUNT as u8,
            src: 0,
        }, // ignored
        VmInstruction::Halt,
    ];
    let (r, _, _aq) = run_vm(program, 1, vec![99.0], &[], zeroed_upstream(), 100.0);
    for s in r.output_slots {
        assert_eq!(s, 0.0);
    }
}

#[test]
fn write_route_gate_sets_slot_zero() {
    let program = vec![
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        }, // r0 = 2.0
        VmInstruction::WriteRouteGate { slot: 0, src: 0 },
        VmInstruction::Halt,
    ];
    let (r, _, _aq) = run_vm(program, 1, vec![2.0], &[], zeroed_upstream(), 100.0);
    assert!((r.route_gates.scores[0] - 2.0).abs() < 1e-6);
}

#[test]
fn vm_write_route_gate_sets_slot_score() {
    // Build a VM program:
    // LoadConst { dst: 0, const_idx: 0 }  // r0 = 7.5
    // WriteRouteGate { slot: 3, src: 0 }  // gate[3] = 7.5
    // Halt
    let program = vec![
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        },
        VmInstruction::WriteRouteGate { slot: 3, src: 0 },
        VmInstruction::Halt,
    ];
    let (r, _, _) = run_vm(program, 1, vec![7.5], &[], zeroed_upstream(), 100.0);
    assert!(
        (r.route_gates.scores[3] - 7.5).abs() < 1e-6,
        "gate slot 3 should be 7.5, got {}",
        r.route_gates.scores[3]
    );
    // All other slots remain 0.0
    for (i, &score) in r.route_gates.scores.iter().enumerate() {
        if i != 3 {
            assert_eq!(score, 0.0, "gate slot {i} should be 0.0, got {score}");
        }
    }
}

#[test]
fn vm_write_route_gate_invalid_slot_is_noop() {
    // WriteRouteGate { slot: 255, src: 0 } — out of range, should be a no-op
    let program = vec![
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        },
        VmInstruction::WriteRouteGate { slot: 255, src: 0 },
        VmInstruction::Halt,
    ];
    let (r, _, _) = run_vm(program, 1, vec![42.0], &[], zeroed_upstream(), 100.0);
    // All gate scores remain 0.0
    for (i, &score) in r.route_gates.scores.iter().enumerate() {
        assert_eq!(score, 0.0, "gate slot {i} should be 0.0, got {score}");
    }
}

// ── Memory opcodes ────────────────────────────────────────────────────────

#[test]
fn store_and_load_slot() {
    let def = VmBackendDef {
        register_count: 2,
        constants: vec![42.0],
        program: vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            }, // r0 = 42.0 (value)
            VmInstruction::LoadConst {
                dst: 1,
                const_idx: 0,
            }, // r1 = 42.0 (slot address, wraps to 42%16=10)
            VmInstruction::StoreSlot {
                slot_reg: 1,
                src: 0,
            }, // mem[10] = 42.0
            VmInstruction::LoadSlot {
                dst: 0,
                slot_reg: 1,
            }, // r0 = mem[10]
            VmInstruction::WriteInternalPayload {
                slot_idx: 0,
                src: 0,
            },
            VmInstruction::Halt,
        ],
    };
    let ss = empty_sensor_snapshot();
    let mut e = 100.0;
    let mut mem = [0.0f32; 16];
    let prev_mem = [0.0f32; 16];
    let cfg = config();
    let mut side_outputs = MeshSideOutputs::new(cfg.max_actions_per_turn);
    let r = execute_vm_node(
        &def,
        &[],
        &zeroed_upstream(),
        &mut e,
        0.0,
        &mut mem,
        &prev_mem,
        &ss,
        &cfg,
        &mut side_outputs,
    );
    assert!((r.output_slots[0] - 42.0).abs() < 1e-6);
    assert!((mem[10] - 42.0).abs() < f32::EPSILON); // memory committed
}

#[test]
fn store_and_load_slot_imm() {
    let def = VmBackendDef {
        register_count: 1,
        constants: vec![77.0],
        program: vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            }, // r0 = 77.0
            VmInstruction::StoreSlotImm {
                slot_idx: 4,
                src: 0,
            }, // mem[4] = 77.0
            VmInstruction::LoadSlotImm {
                dst: 0,
                slot_idx: 4,
            }, // r0 = mem[4]
            VmInstruction::WriteInternalPayload {
                slot_idx: 0,
                src: 0,
            },
            VmInstruction::Halt,
        ],
    };
    let ss = empty_sensor_snapshot();
    let mut e = 100.0;
    let mut mem = [0.0f32; 16];
    let prev_mem = [0.0f32; 16];
    let cfg = config();
    let mut side_outputs = MeshSideOutputs::new(cfg.max_actions_per_turn);
    let r = execute_vm_node(
        &def,
        &[],
        &zeroed_upstream(),
        &mut e,
        0.0,
        &mut mem,
        &prev_mem,
        &ss,
        &cfg,
        &mut side_outputs,
    );
    assert!((r.output_slots[0] - 77.0).abs() < 1e-6);
    assert!((mem[4] - 77.0).abs() < f32::EPSILON);
}

#[test]
fn slot_address_wraps_via_rem_euclid() {
    let def = VmBackendDef {
        register_count: 2,
        constants: vec![1024.0, 55.0],
        program: vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            }, // r0 = 1024 (addr, wraps to 1024%16=0)
            VmInstruction::LoadConst {
                dst: 1,
                const_idx: 1,
            }, // r1 = 55.0 (val)
            VmInstruction::StoreSlot {
                slot_reg: 0,
                src: 1,
            }, // mem[0] = 55.0
            VmInstruction::LoadSlot {
                dst: 0,
                slot_reg: 0,
            }, // r0 = mem[0]
            VmInstruction::WriteInternalPayload {
                slot_idx: 0,
                src: 0,
            },
            VmInstruction::Halt,
        ],
    };
    let ss = empty_sensor_snapshot();
    let mut e = 100.0;
    let mut mem = [0.0f32; 16];
    let prev_mem = [0.0f32; 16];
    let cfg = config();
    let mut side_outputs = MeshSideOutputs::new(cfg.max_actions_per_turn);
    let r = execute_vm_node(
        &def,
        &[],
        &zeroed_upstream(),
        &mut e,
        0.0,
        &mut mem,
        &prev_mem,
        &ss,
        &cfg,
        &mut side_outputs,
    );
    assert!((r.output_slots[0] - 55.0).abs() < 1e-6);
}

// ── Integration: full input resolution pipeline ────────────────────────

#[test]
fn vm_eats_when_food_here() {
    use crate::config::{SimulationConfig, WorldEdgeMode};
    use crate::contracts::{CreatureId, NodeId, Position, WorldInputKey};
    use crate::creature::genome::{
        BackendDef, CreatureGenome, NodeGenome, VmBackendDef, VmInstruction,
    };
    use crate::creature::state::CreatureState;
    use crate::kernel::WorldState;
    use crate::sensors::perception::{PerceptionSnapshot, SensorSnapshot};
    use crate::sensors::static_inputs::assemble_static_inputs;
    use crate::sensors::typed_food::assemble_typed_food_local_snapshot;
    use rand::rngs::SmallRng;
    use rand::SeedableRng;
    use slotmap::SlotMap;

    // Build a world with food everywhere
    let mut world = WorldState::new(5, 5, WorldEdgeMode::Wrap);
    let food_cfg = SimulationConfig::default().world.food;
    world.reconfigure_food(food_cfg);
    let mut rng = SmallRng::seed_from_u64(42);
    world.seed_food(&mut rng);

    let pos = Position::new(1, 1);
    let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
    let id = sm.insert(());

    // Single-node VM: read food_here into r0, compare > 0, jump-if-zero past Eat, else Eat
    let input_refs = vec![InputReference::World(WorldInputKey::FoodHere {
        type_idx: crate::config::OrdinaryFoodTypeId::default(),
    })];
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: input_refs.clone(),
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 2,
                constants: vec![],
                program: vec![
                    VmInstruction::ReadInput {
                        dst: 0,
                        ref_idx: 0,
                        sub_idx: 0,
                    }, // r0 = food_here
                    // r1 is 0.0; compare r0 > r1
                    VmInstruction::CmpGt { dst: 0, a: 0, b: 1 }, // r0 = (food > 0)?
                    VmInstruction::JumpIfZero { cond: 0, offset: 2 }, // skip Eat if no food
                    VmInstruction::AddVote { sink: 0, src: 0 },  // Eat
                    VmInstruction::Halt,
                    VmInstruction::Noop, // NoOp fallback
                    VmInstruction::Halt,
                ],
            }),
            targets: vec![],
        }],
    };
    let creature = CreatureState::new(
        id,
        genome,
        pos,
        30.0,
        0,
        [0; 6],
        0,
        [true; 6],
        CreatureIdentityState::default(),
        [0.0; 16],
    );

    let local = assemble_static_inputs(&world, &creature, &EnergyLifecycleConfig::default());
    // food_here should be > 0.0
    assert!(local.food_here > 0.0);
    let ss = SensorSnapshot {
        local,
        typed_local_food: assemble_typed_food_local_snapshot(&world, creature.position),
        perception: PerceptionSnapshot::zeroed(1),
    };

    // One tick of the pass loop: a vote of 1.0 commits once.
    let mut energy = creature.energy;
    let actions = crate::runtime::mesh::execute_creature_mesh(
        &creature.genome,
        &ss,
        &mut energy,
        &mut [0.0f32; 16],
        &[0.0f32; 16],
        &mut crate::creature::state::GraphRuntimeState::new(),
        &config(),
    )
    .actions;
    assert_eq!(
        actions.first(),
        Some(&crate::contracts::WorldAction::Eat {
            type_idx: crate::config::OrdinaryFoodTypeId::default()
        }),
        "Expected Eat when food is present"
    );
}

#[test]
fn vm_noop_when_no_food() {
    use crate::config::WorldEdgeMode;
    use crate::contracts::{CreatureId, NodeId, Position, WorldInputKey};
    use crate::creature::genome::{
        BackendDef, CreatureGenome, NodeGenome, VmBackendDef, VmInstruction,
    };
    use crate::creature::state::CreatureState;
    use crate::kernel::WorldState;
    use crate::sensors::perception::{PerceptionSnapshot, SensorSnapshot};
    use crate::sensors::static_inputs::assemble_static_inputs;
    use crate::sensors::typed_food::assemble_typed_food_local_snapshot;
    use slotmap::SlotMap;

    // World with no food
    let world = WorldState::new(5, 5, WorldEdgeMode::Wrap);
    let pos = Position::new(2, 2);
    let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
    let id = sm.insert(());

    let input_refs = vec![InputReference::World(WorldInputKey::FoodHere {
        type_idx: crate::config::OrdinaryFoodTypeId::default(),
    })];
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: input_refs.clone(),
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 2,
                constants: vec![],
                program: vec![
                    VmInstruction::ReadInput {
                        dst: 0,
                        ref_idx: 0,
                        sub_idx: 0,
                    }, // r0 = food_here = 0
                    VmInstruction::CmpGt { dst: 0, a: 0, b: 1 }, // r0 = (0 > 0) = 0
                    VmInstruction::JumpIfZero { cond: 0, offset: 2 }, // fires → skip Eat
                    VmInstruction::AddVote { sink: 0, src: 0 },  // Eat (skipped)
                    VmInstruction::Halt,
                    VmInstruction::Noop, // NoOp fallback
                    VmInstruction::Halt,
                ],
            }),
            targets: vec![],
        }],
    };
    let creature = CreatureState::new(
        id,
        genome,
        pos,
        30.0,
        0,
        [0; 6],
        0,
        [true; 6],
        CreatureIdentityState::default(),
        [0.0; 16],
    );
    let local = assemble_static_inputs(&world, &creature, &EnergyLifecycleConfig::default());
    assert_eq!(local.food_here, 0.0);
    let ss = SensorSnapshot {
        local,
        typed_local_food: assemble_typed_food_local_snapshot(&world, creature.position),
        perception: PerceptionSnapshot::zeroed(1),
    };

    // One tick of the pass loop: a vote of 1.0 commits once.
    let mut energy = 30.0;
    let actions = crate::runtime::mesh::execute_creature_mesh(
        &creature.genome,
        &ss,
        &mut energy,
        &mut [0.0f32; 16],
        &[0.0f32; 16],
        &mut crate::creature::state::GraphRuntimeState::new(),
        &config(),
    )
    .actions;
    assert_eq!(
        actions.first(),
        Some(&crate::contracts::WorldAction::NoOp),
        "Expected NoOp when no food"
    );
}
