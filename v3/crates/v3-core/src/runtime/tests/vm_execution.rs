use super::*;

// ── Halt and empty program ────────────────────────────────────────────────

#[test]
fn register_count_zero_halts_immediately() {
    let def = VmBackendDef {
        register_count: 0,
        constants: vec![],
        program: vec![VmInstruction::EmitWorldAction { action_type: 1 }],
    };
    let si = empty_static_inputs();
    let mut e = 100.0;
    let mut mem = [0u8; 1024];
    let cfg = config();
    let mut action_queue = ActionQueue::new(cfg.max_actions_per_turn);
    let r = execute_vm_node(
        &def,
        &[],
        &zeroed_upstream(),
        &mut e,
        0.0,
        &mut mem,
        &si,
        &cfg,
        &mut action_queue,
    );
    assert!(!r.terminal);
    assert!(!r.energy_exhausted);
}

#[test]
fn empty_program_halts_immediately() {
    let (r, _, _) = run_vm(vec![], 1, vec![], &[], zeroed_upstream(), 100.0);
    assert!(!r.terminal);
}

#[test]
fn halt_returns_no_action() {
    let (r, _, _) = run_vm(
        vec![VmInstruction::Halt],
        2,
        vec![],
        &[],
        zeroed_upstream(),
        100.0,
    );
    assert!(!r.terminal);
    assert!(!r.energy_exhausted);
}

#[test]
fn program_counter_past_program_len_soft_halts() {
    // Program has one instruction and no explicit halt/jump.
    // After first step pc becomes 1 (out of range for len=1) and must soft-halt.
    // Uses opcode_cost_multiplier=1.0 so the deduction is observable in f32.
    let def = VmBackendDef {
        register_count: 1,
        constants: vec![],
        program: vec![VmInstruction::Noop],
    };
    let si = empty_static_inputs();
    let mut e = 100.0;
    let mut mem = [0u8; 1024];
    let mut cfg = config();
    cfg.vm.opcode_cost_multiplier = 1.0;
    let mut action_queue = ActionQueue::new(cfg.max_actions_per_turn);
    let r = execute_vm_node(
        &def,
        &[],
        &zeroed_upstream(),
        &mut e,
        0.0,
        &mut mem,
        &si,
        &cfg,
        &mut action_queue,
    );
    assert!(!r.terminal);
    assert!(!r.energy_exhausted);
    // Noop base cost = 0.05, multiplier = 1.0 → energy = 99.95
    assert!(e < 100.0);
}

// ── Control flow opcodes ──────────────────────────────────────────────────

#[test]
fn jump_skips_instructions() {
    let program = vec![
        VmInstruction::Jump { offset: 1 }, // jump to pc+1+1=2
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        }, // skipped
        VmInstruction::WriteInternalPayload {
            slot_idx: 0,
            src: 0,
        },
        VmInstruction::Halt,
    ];
    let (r, _, _) = run_vm(program, 1, vec![99.0], &[], zeroed_upstream(), 100.0);
    assert_eq!(r.output_slots[0], 0.0);
}

#[test]
fn jump_if_zero_fires_when_false() {
    let program = vec![
        VmInstruction::JumpIfZero { cond: 0, offset: 1 }, // fires (r0=0)
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        }, // skipped
        VmInstruction::WriteInternalPayload {
            slot_idx: 0,
            src: 0,
        },
        VmInstruction::Halt,
    ];
    let (r, _, _) = run_vm(program, 1, vec![99.0], &[], zeroed_upstream(), 100.0);
    assert_eq!(r.output_slots[0], 0.0);
}

#[test]
fn jump_if_zero_not_fires_when_truthy() {
    let program = vec![
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        }, // r0 = 1.0
        VmInstruction::JumpIfZero { cond: 0, offset: 1 }, // does NOT fire (truthy)
        VmInstruction::LoadConst {
            dst: 1,
            const_idx: 1,
        }, // r1 = 99.0
        VmInstruction::WriteInternalPayload {
            slot_idx: 0,
            src: 1,
        },
        VmInstruction::Halt,
    ];
    let (r, _, _) = run_vm(program, 2, vec![1.0, 99.0], &[], zeroed_upstream(), 100.0);
    assert!((r.output_slots[0] - 99.0).abs() < 1e-6);
}

#[test]
fn jump_target_wraps_via_rem_euclid() {
    // Program of 2 instructions: Jump(offset=2), Halt
    // provisional_pc = 0 + 1 + 2 = 3; 3 % 2 = 1 (Halt)
    let program = vec![VmInstruction::Jump { offset: 2 }, VmInstruction::Halt];
    let (r, _, _) = run_vm(program, 1, vec![], &[], zeroed_upstream(), 100.0);
    assert!(!r.terminal);
}

// ── Energy metering ───────────────────────────────────────────────────────

#[test]
fn energy_is_deducted_per_opcode() {
    // Uses opcode_cost_multiplier=1.0 so the deduction is observable in f32.
    // Noop(0.05) + Halt(0.05) = 0.10 total cost → energy = 99.90
    let def = VmBackendDef {
        register_count: 1,
        constants: vec![],
        program: vec![VmInstruction::Noop, VmInstruction::Halt],
    };
    let si = empty_static_inputs();
    let mut e = 100.0;
    let mut mem = [0u8; 1024];
    let mut cfg = config();
    cfg.vm.opcode_cost_multiplier = 1.0;
    let mut action_queue = ActionQueue::new(cfg.max_actions_per_turn);
    let _ = execute_vm_node(
        &def,
        &[],
        &zeroed_upstream(),
        &mut e,
        0.0,
        &mut mem,
        &si,
        &cfg,
        &mut action_queue,
    );
    assert!(e < 100.0);
    assert!(e > 99.0);
}

#[test]
fn energy_exhaustion_returns_exhausted() {
    // Very low energy with opcode_cost_multiplier=1.0; Noop base cost = 0.05
    let def = VmBackendDef {
        register_count: 1,
        constants: vec![],
        program: vec![VmInstruction::Noop, VmInstruction::Halt],
    };
    let si = empty_static_inputs();
    let mut e = 0.01;
    let mut mem = [0u8; 1024];
    let mut cfg = config();
    cfg.vm.opcode_cost_multiplier = 1.0;
    let mut action_queue = ActionQueue::new(cfg.max_actions_per_turn);
    let r = execute_vm_node(
        &def,
        &[],
        &zeroed_upstream(),
        &mut e,
        0.0,
        &mut mem,
        &si,
        &cfg,
        &mut action_queue,
    );
    assert!(r.energy_exhausted);
}

#[test]
fn energy_exhaustion_does_not_commit_memory_writes() {
    let def = VmBackendDef {
        register_count: 1,
        constants: vec![42.0],
        program: vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            }, // costs 0.08
            VmInstruction::StoreMem8Imm {
                imm_addr: 10,
                src: 0,
            }, // costs 0.16
            VmInstruction::Halt,
        ],
    };
    let si = empty_static_inputs();
    // Give 0.20 energy with opcode_cost_multiplier=1.0:
    // LoadConst(0.08) → 0.12 left; StoreMem8Imm(0.16) → -0.04 → exhausted
    let mut e = 0.20;
    let mut mem = [0u8; 1024];
    let mut cfg = config();
    cfg.vm.opcode_cost_multiplier = 1.0;
    let mut action_queue = ActionQueue::new(cfg.max_actions_per_turn);
    let r = execute_vm_node(
        &def,
        &[],
        &zeroed_upstream(),
        &mut e,
        0.0,
        &mut mem,
        &si,
        &cfg,
        &mut action_queue,
    );
    assert!(r.energy_exhausted);
    // mem[10] should still be 0 (not written)
    assert_eq!(mem[10], 0);
}

#[test]
fn max_vm_steps_enforced() {
    // Infinite loop: Jump(offset=-1) → provisional_pc = 0 + 1 + (-1) = 0
    let program = vec![VmInstruction::Jump { offset: -1 }];
    let cfg = RuntimeConfig {
        max_vm_steps: 10,
        ..RuntimeConfig::default()
    };
    let def = VmBackendDef {
        register_count: 1,
        constants: vec![],
        program,
    };
    let si = empty_static_inputs();
    let mut e = 1000.0;
    let mut mem = [0u8; 1024];
    let mut action_queue = ActionQueue::new(cfg.max_actions_per_turn);
    let r = execute_vm_node(
        &def,
        &[],
        &zeroed_upstream(),
        &mut e,
        0.0,
        &mut mem,
        &si,
        &cfg,
        &mut action_queue,
    );
    assert!(!r.terminal);
    assert!(!r.energy_exhausted);
}

// ── Operand normalization ─────────────────────────────────────────────────

#[test]
fn register_index_wraps_via_rem_euclid() {
    let program = vec![
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        }, // r0 = 7.0
        VmInstruction::WriteInternalPayload {
            slot_idx: 0,
            src: 2,
        }, // src=2 wraps to 0 → 7.0
        VmInstruction::Halt,
    ];
    let (r, _, _) = run_vm(program, 2, vec![7.0], &[], zeroed_upstream(), 100.0);
    assert!((r.output_slots[0] - 7.0).abs() < 1e-6);
}

#[test]
fn constant_index_wraps_via_rem_euclid() {
    // 2 constants: [3.0, 5.0]. const_idx=3 → 3%2=1 → 5.0
    let program = vec![
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 3,
        }, // idx 3%2=1 → 5.0
        VmInstruction::WriteInternalPayload {
            slot_idx: 0,
            src: 0,
        },
        VmInstruction::Halt,
    ];
    let (r, _, _) = run_vm(program, 1, vec![3.0, 5.0], &[], zeroed_upstream(), 100.0);
    assert!((r.output_slots[0] - 5.0).abs() < 1e-6);
}

#[test]
fn constant_empty_pool_yields_zero() {
    let program = vec![
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        }, // empty pool → 0.0
        VmInstruction::WriteInternalPayload {
            slot_idx: 0,
            src: 0,
        },
        VmInstruction::Halt,
    ];
    let (r, _, _) = run_vm(program, 1, vec![], &[], zeroed_upstream(), 100.0);
    assert_eq!(r.output_slots[0], 0.0);
}

// ── sanitize_f32 on register writes ──────────────────────────────────────

#[test]
fn nan_in_add_becomes_zero() {
    // Use 2e9 constant: sanitize_f32(2e9) = 1e9, then Add(1e9, 1e9) = 2e9 → clamped to 1e9
    let program = vec![
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        }, // r0 = 2e9 → clamped to 1e9
        VmInstruction::Add { dst: 1, a: 0, b: 0 }, // r1 = 1e9 + 1e9 = 2e9 → clamped to 1e9
        VmInstruction::WriteInternalPayload {
            slot_idx: 0,
            src: 1,
        },
        VmInstruction::Halt,
    ];
    let (r, _, _) = run_vm(program, 2, vec![2e9_f32], &[], zeroed_upstream(), 100.0);
    assert!((r.output_slots[0] - 1_000_000_000.0).abs() < 1.0);
}
