use super::*;
use crate::creature::identity::CreatureIdentityState;

// ── PushAction + ExecuteActionQueue ────────────────────────────────────────

#[test]
fn emit_noop_action_type_0() {
    let (_r, _, aq) = run_vm(
        vec![
            VmInstruction::PushAction { action_type: 0 },
            VmInstruction::ExecuteActionQueue,
        ],
        1,
        vec![],
        &[],
        zeroed_upstream(),
        100.0,
    );
    let actions = aq.action_queue.into_actions();
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0], crate::contracts::WorldAction::NoOp);
}

#[test]
fn emit_eat_action_type_1() {
    let (_r, _, aq) = run_vm(
        vec![
            VmInstruction::PushAction { action_type: 1 },
            VmInstruction::ExecuteActionQueue,
        ],
        1,
        vec![],
        &[],
        zeroed_upstream(),
        100.0,
    );
    let actions = aq.action_queue.into_actions();
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0], crate::contracts::WorldAction::Eat);
}

#[test]
fn emit_unknown_action_type_255_defaults_to_noop() {
    let (_r, _, aq) = run_vm(
        vec![
            VmInstruction::PushAction { action_type: 255 },
            VmInstruction::ExecuteActionQueue,
        ],
        1,
        vec![],
        &[],
        zeroed_upstream(),
        100.0,
    );
    let actions = aq.action_queue.into_actions();
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0], crate::contracts::WorldAction::NoOp);
}

#[test]
fn emit_move_with_meta() {
    // Set meta[0] = 2.0 (East), then emit Move
    let program = vec![
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        }, // r0 = 2.0
        VmInstruction::WriteWorldActionMeta {
            slot_idx: 0,
            src: 0,
        },
        VmInstruction::PushAction { action_type: 2 },
        VmInstruction::ExecuteActionQueue,
    ];
    let (_r, _, aq) = run_vm(program, 1, vec![2.0], &[], zeroed_upstream(), 100.0);
    let actions = aq.action_queue.into_actions();
    assert_eq!(actions.len(), 1);
    assert_eq!(
        actions[0],
        crate::contracts::WorldAction::Move(Direction::E)
    );
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
    let si = empty_static_inputs();
    let mut e = 100.0;
    let mut mem = [0u8; 1024];
    let cfg = config();
    let mut side_outputs = MeshSideOutputs::new(cfg.max_actions_per_turn);
    let r = execute_vm_node(
        &def,
        &input_refs,
        &upstream,
        &mut e,
        0.0,
        &mut mem,
        &si,
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
            slot_idx: 12,
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
fn write_route_target_sets_output() {
    let program = vec![
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        }, // r0 = 2.0
        VmInstruction::WriteRouteTarget { src: 0 },
        VmInstruction::Halt,
    ];
    let (r, _, _aq) = run_vm(program, 1, vec![2.0], &[], zeroed_upstream(), 100.0);
    assert!((r.route_target_idx - 2.0).abs() < 1e-6);
}

// ── Memory opcodes ────────────────────────────────────────────────────────

#[test]
fn store_and_load_mem8() {
    let def = VmBackendDef {
        register_count: 2,
        constants: vec![42.0],
        program: vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            }, // r0 = 42.0 (byte value)
            VmInstruction::LoadConst {
                dst: 1,
                const_idx: 0,
            }, // r1 = 42 (address)
            VmInstruction::StoreMem8 {
                addr_reg: 1,
                src: 0,
            }, // mem[42] = 42
            VmInstruction::LoadMem8 {
                dst: 0,
                addr_reg: 1,
            }, // r0 = mem[42]
            VmInstruction::WriteInternalPayload {
                slot_idx: 0,
                src: 0,
            },
            VmInstruction::Halt,
        ],
    };
    let si = empty_static_inputs();
    let mut e = 100.0;
    let mut mem = [0u8; 1024];
    let cfg = config();
    let mut side_outputs = MeshSideOutputs::new(cfg.max_actions_per_turn);
    let r = execute_vm_node(
        &def,
        &[],
        &zeroed_upstream(),
        &mut e,
        0.0,
        &mut mem,
        &si,
        &cfg,
        &mut side_outputs,
    );
    assert!((r.output_slots[0] - 42.0).abs() < 1e-6);
    assert_eq!(mem[42], 42); // memory committed
}

#[test]
fn store_and_load_mem8_imm() {
    let def = VmBackendDef {
        register_count: 1,
        constants: vec![77.0],
        program: vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            }, // r0 = 77.0
            VmInstruction::StoreMem8Imm {
                imm_addr: 100,
                src: 0,
            }, // mem[100] = 77
            VmInstruction::LoadMem8Imm {
                dst: 0,
                imm_addr: 100,
            }, // r0 = mem[100]
            VmInstruction::WriteInternalPayload {
                slot_idx: 0,
                src: 0,
            },
            VmInstruction::Halt,
        ],
    };
    let si = empty_static_inputs();
    let mut e = 100.0;
    let mut mem = [0u8; 1024];
    let cfg = config();
    let mut side_outputs = MeshSideOutputs::new(cfg.max_actions_per_turn);
    let r = execute_vm_node(
        &def,
        &[],
        &zeroed_upstream(),
        &mut e,
        0.0,
        &mut mem,
        &si,
        &cfg,
        &mut side_outputs,
    );
    assert!((r.output_slots[0] - 77.0).abs() < 1e-6);
    assert_eq!(mem[100], 77);
}

#[test]
fn memory_address_wraps_via_rem_euclid() {
    let def = VmBackendDef {
        register_count: 2,
        constants: vec![1024.0, 55.0],
        program: vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            }, // r0 = 1024 (addr)
            VmInstruction::LoadConst {
                dst: 1,
                const_idx: 1,
            }, // r1 = 55 (val)
            VmInstruction::StoreMem8 {
                addr_reg: 0,
                src: 1,
            }, // mem[1024%1024=0] = 55
            VmInstruction::LoadMem8 {
                dst: 0,
                addr_reg: 0,
            }, // r0 = mem[0]
            VmInstruction::WriteInternalPayload {
                slot_idx: 0,
                src: 0,
            },
            VmInstruction::Halt,
        ],
    };
    let si = empty_static_inputs();
    let mut e = 100.0;
    let mut mem = [0u8; 1024];
    let cfg = config();
    let mut side_outputs = MeshSideOutputs::new(cfg.max_actions_per_turn);
    let r = execute_vm_node(
        &def,
        &[],
        &zeroed_upstream(),
        &mut e,
        0.0,
        &mut mem,
        &si,
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
    use crate::sensors::static_inputs::assemble_static_inputs;
    use rand::rngs::SmallRng;
    use rand::SeedableRng;
    use slotmap::SlotMap;

    // Build a world with food everywhere
    let mut world = WorldState::new(5, 5, WorldEdgeMode::Wrap);
    let mut cfg = SimulationConfig::default();
    cfg.world.food.initial_coverage = 1.0;
    cfg.world.food.initial_density = 1.0;
    let mut rng = SmallRng::seed_from_u64(42);
    world.seed_food(&mut rng, &cfg);

    let pos = Position::new(1, 1);
    let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
    let id = sm.insert(());

    // Single-node VM: read food_here into r0, compare > 0, jump-if-zero past Eat, else Eat
    let input_refs = vec![InputReference::World(WorldInputKey::FoodHere)];
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
                    VmInstruction::PushAction { action_type: 1 }, // Eat
                    VmInstruction::ExecuteActionQueue,
                    VmInstruction::PushAction { action_type: 0 }, // NoOp fallback
                    VmInstruction::ExecuteActionQueue,
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
    );

    let si = assemble_static_inputs(&world, &creature);
    // food_here should be > 0.0
    assert!(si.food_here > 0.0);

    let def = if let BackendDef::Vm(ref v) = creature.genome.nodes[0].backend_def {
        v
    } else {
        panic!("expected VM backend");
    };

    let upstream = [0.0f32; 12];
    let mut energy = creature.energy;
    let mut mem = [0u8; 1024];
    let cfg = config();
    let mut side_outputs = MeshSideOutputs::new(cfg.max_actions_per_turn);
    let _result = execute_vm_node(
        def,
        &creature.genome.nodes[0].input_refs,
        &upstream,
        &mut energy,
        0.0,
        &mut mem,
        &si,
        &cfg,
        &mut side_outputs,
    );
    let actions = side_outputs.action_queue.into_actions();
    assert_eq!(
        actions.first(),
        Some(&crate::contracts::WorldAction::Eat),
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
    use crate::sensors::static_inputs::assemble_static_inputs;
    use slotmap::SlotMap;

    // World with no food
    let world = WorldState::new(5, 5, WorldEdgeMode::Wrap);
    let pos = Position::new(2, 2);
    let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
    let id = sm.insert(());

    let input_refs = vec![InputReference::World(WorldInputKey::FoodHere)];
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
                    VmInstruction::PushAction { action_type: 1 }, // Eat (skipped)
                    VmInstruction::ExecuteActionQueue,
                    VmInstruction::PushAction { action_type: 0 }, // NoOp fallback
                    VmInstruction::ExecuteActionQueue,
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
    );
    let si = assemble_static_inputs(&world, &creature);
    assert_eq!(si.food_here, 0.0);

    let def = if let BackendDef::Vm(ref v) = creature.genome.nodes[0].backend_def {
        v
    } else {
        panic!()
    };

    let upstream = [0.0f32; 12];
    let mut energy = 30.0;
    let mut mem = [0u8; 1024];
    let cfg = config();
    let mut side_outputs = MeshSideOutputs::new(cfg.max_actions_per_turn);
    let _result = execute_vm_node(
        def,
        &creature.genome.nodes[0].input_refs,
        &upstream,
        &mut energy,
        0.0,
        &mut mem,
        &si,
        &cfg,
        &mut side_outputs,
    );
    let actions = side_outputs.action_queue.into_actions();
    assert_eq!(
        actions.first(),
        Some(&crate::contracts::WorldAction::NoOp),
        "Expected NoOp when no food"
    );
}
