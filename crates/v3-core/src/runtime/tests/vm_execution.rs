use super::*;
use proptest::prelude::*;

// ── Halt and empty program ────────────────────────────────────────────────

#[test]
fn register_count_zero_halts_immediately() {
    let def = VmBackendDef {
        register_count: 0,
        constants: vec![],
        program: vec![
            VmInstruction::PushAction { action_type: 1 },
            VmInstruction::ExecuteActionQueue,
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
    let ss = empty_sensor_snapshot();
    let mut e = 100.0;
    let mut mem = [0.0f32; 16];
    let prev_mem = [0.0f32; 16];
    let mut cfg = config();
    cfg.vm.opcode_cost_multiplier = 1.0;
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
    let ss = empty_sensor_snapshot();
    let mut e = 100.0;
    let mut mem = [0.0f32; 16];
    let prev_mem = [0.0f32; 16];
    let mut cfg = config();
    cfg.vm.opcode_cost_multiplier = 1.0;
    let mut side_outputs = MeshSideOutputs::new(cfg.max_actions_per_turn);
    let _ = execute_vm_node(
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
    let ss = empty_sensor_snapshot();
    let mut e = 0.01;
    let mut mem = [0.0f32; 16];
    let prev_mem = [0.0f32; 16];
    let mut cfg = config();
    cfg.vm.opcode_cost_multiplier = 1.0;
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
            VmInstruction::StoreSlotImm {
                slot_idx: 10,
                src: 0,
            }, // costs 0.12
            VmInstruction::Halt,
        ],
    };
    let ss = empty_sensor_snapshot();
    // Give 0.15 energy with opcode_cost_multiplier=1.0:
    // LoadConst(0.08) → 0.07 left; StoreSlotImm(0.12) → -0.05 → exhausted
    let mut e = 0.15;
    let mut mem = [0.0f32; 16];
    let prev_mem = [0.0f32; 16];
    let mut cfg = config();
    cfg.vm.opcode_cost_multiplier = 1.0;
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
    assert!(r.energy_exhausted);
    // mem[10] should still be 0.0 (not written)
    assert!((mem[10]).abs() < f32::EPSILON);
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
    let ss = empty_sensor_snapshot();
    let mut e = 1000.0;
    let mut mem = [0.0f32; 16];
    let prev_mem = [0.0f32; 16];
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

// ── SetPriorityBid overconsumption ──────────────────────────────────────

#[test]
fn priority_bid_capped_at_available_energy() {
    // A bid of 500.0 with only 100.0 energy should never drive energy negative.
    // Uses opcode_cost_multiplier=1.0 for deterministic cost accounting.
    let def = VmBackendDef {
        register_count: 1,
        constants: vec![500.0],
        program: vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            },
            VmInstruction::SetPriorityBid { src: 0 },
            VmInstruction::Halt,
        ],
    };
    let ss = empty_sensor_snapshot();
    let mut e = 100.0;
    let mut mem = [0.0f32; 16];
    let prev_mem = [0.0f32; 16];
    let mut cfg = config();
    cfg.vm.opcode_cost_multiplier = 1.0;
    let mut side_outputs = MeshSideOutputs::new(cfg.max_actions_per_turn);
    let _r = execute_vm_node(
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
    assert!(
        e >= 0.0,
        "energy must never go negative from a priority bid; got {}",
        e,
    );
}

#[test]
fn oversized_bid_produces_exact_all_in_exhaustion() {
    // When the requested bid (500.0) exceeds available energy, the bid is capped
    // to available energy, producing an exact all-in: energy lands at 0.0 and the
    // creature exhausts. Verify the energy is exactly zero (not negative) and the
    // creature is correctly marked exhausted.
    // Uses opcode_cost_multiplier=1.0 for deterministic cost accounting.
    let starting_energy = 100.0_f32;
    let def = VmBackendDef {
        register_count: 1,
        constants: vec![500.0],
        program: vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            },
            VmInstruction::SetPriorityBid { src: 0 },
            VmInstruction::Halt,
        ],
    };
    let ss = empty_sensor_snapshot();
    let mut e = starting_energy;
    let mut mem = [0.0f32; 16];
    let prev_mem = [0.0f32; 16];
    let mut cfg = config();
    cfg.vm.opcode_cost_multiplier = 1.0;
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
    // Capped bid drains all remaining energy → exact zero, not negative.
    assert_eq!(
        e, 0.0,
        "capped bid should drain energy to exactly 0.0; got {e}"
    );
    // All-in bid correctly triggers exhaustion.
    assert!(
        r.energy_exhausted,
        "creature should be exhausted after all-in capped bid",
    );
}

#[test]
fn inserted_noop_preserves_outputs_with_ample_budget_but_charges_its_cost() {
    let original = vec![
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        },
        VmInstruction::WriteInternalPayload {
            slot_idx: 0,
            src: 0,
        },
        VmInstruction::Halt,
    ];
    let mut inserted = original.clone();
    crate::mutation::vm::insert_new_instruction_with_reference_repair(
        &mut inserted,
        1,
        VmInstruction::Noop,
    )
    .unwrap();

    let mut cfg = config();
    cfg.vm.opcode_cost_multiplier = 1.0;
    let (original_result, original_energy, original_side_outputs) = run_vm_with_config(
        original,
        1,
        vec![7.0],
        &[],
        zeroed_upstream(),
        100.0,
        cfg.clone(),
    );
    let (inserted_result, inserted_energy, inserted_side_outputs) =
        run_vm_with_config(inserted, 1, vec![7.0], &[], zeroed_upstream(), 100.0, cfg);

    assert_eq!(inserted_result, original_result);
    assert_eq!(
        inserted_side_outputs.action_queue.into_actions(),
        original_side_outputs.action_queue.into_actions()
    );
    assert!(
        inserted_energy < original_energy,
        "executed Noop must retain its defined energy cost"
    );
}

#[test]
fn copied_unreachable_middle_span_preserves_outputs_with_ample_budget() {
    use crate::mutation::vm::{splice_program_with_reference_repair, SpliceInstruction};

    // The entry jump skips indices 1..=3.  The copied action suffix remains
    // unreachable because repair follows the original output instruction.
    let original = vec![
        VmInstruction::Jump { offset: 3 },
        VmInstruction::PushAction { action_type: 9 },
        VmInstruction::ExecuteActionQueue,
        VmInstruction::Noop,
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        },
        VmInstruction::WriteInternalPayload {
            slot_idx: 0,
            src: 0,
        },
        VmInstruction::Halt,
    ];
    let mut copied = original.clone();
    let selected = [1, 2]
        .into_iter()
        .map(|source_index| SpliceInstruction {
            instruction: original[source_index].clone(),
            source_index: Some(source_index),
        })
        .collect();
    splice_program_with_reference_repair(&mut copied, 1..1, selected).unwrap();

    let (original_result, _, original_side_outputs) =
        run_vm(original, 1, vec![7.0], &[], zeroed_upstream(), 100.0);
    let (copied_result, _, copied_side_outputs) =
        run_vm(copied, 1, vec![7.0], &[], zeroed_upstream(), 100.0);

    assert_eq!(copied_result, original_result);
    assert_eq!(
        copied_side_outputs.action_queue.into_actions(),
        original_side_outputs.action_queue.into_actions()
    );
}

proptest! {
    #[test]
    fn noop_insertion_preserves_behavior_under_ample_budget(value in -1000.0f32..1000.0) {
        // The jump reaches the suffix after a terminal.  Repair must keep that
        // target tied to the original LoadConst when the entry Noop is inserted.
        let original = vec![
            VmInstruction::Jump { offset: 1 },
            VmInstruction::Halt,
            VmInstruction::LoadConst { dst: 0, const_idx: 0 },
            VmInstruction::WriteInternalPayload { slot_idx: 0, src: 0 },
            VmInstruction::Halt,
        ];
        let mut cfg = config();
        cfg.vm.opcode_cost_multiplier = 1.0;
        for insert_at in 0..=original.len() {
            let mut inserted = original.clone();
            crate::mutation::vm::insert_new_instruction_with_reference_repair(
                &mut inserted,
                insert_at,
                VmInstruction::Noop,
            )
            .unwrap();

            let (original_result, original_energy, original_outputs) = run_vm_with_config(
                original.clone(),
                1,
                vec![value],
                &[],
                zeroed_upstream(),
                100.0,
                cfg.clone(),
            );
            let (inserted_result, inserted_energy, inserted_outputs) =
                run_vm_with_config(
                    inserted,
                    1,
                    vec![value],
                    &[],
                    zeroed_upstream(),
                    100.0,
                    cfg.clone(),
                );

            prop_assert_eq!(inserted_result, original_result);
            prop_assert_eq!(
                inserted_outputs.action_queue.into_actions(),
                original_outputs.action_queue.into_actions()
            );
            if matches!(insert_at, 0 | 3 | 4) {
                prop_assert!(inserted_energy < original_energy);
            } else {
                prop_assert_eq!(inserted_energy, original_energy);
            }
        }
    }

    #[test]
    fn copied_unreachable_suffix_preserves_behavior_under_ample_budget(
        value in -1000.0f32..1000.0,
        action_type in any::<u8>(),
    ) {
        let original = vec![
            VmInstruction::Jump { offset: 3 },
            VmInstruction::PushAction { action_type },
            VmInstruction::ExecuteActionQueue,
            VmInstruction::Noop,
            VmInstruction::LoadConst { dst: 0, const_idx: 0 },
            VmInstruction::WriteInternalPayload { slot_idx: 0, src: 0 },
            VmInstruction::Halt,
        ];
        let mut copied = original.clone();
        let selected = [1, 2]
            .into_iter()
            .map(|source_index| crate::mutation::vm::SpliceInstruction {
                instruction: original[source_index].clone(),
                source_index: Some(source_index),
            })
            .collect();
        crate::mutation::vm::splice_program_with_reference_repair(
            &mut copied,
            original.len()..original.len(),
            selected,
        )
        .unwrap();

        let (original_result, _, original_outputs) =
            run_vm(original, 1, vec![value], &[], zeroed_upstream(), 100.0);
        let (copied_result, _, copied_outputs) =
            run_vm(copied, 1, vec![value], &[], zeroed_upstream(), 100.0);

        prop_assert_eq!(copied_result, original_result);
        prop_assert_eq!(
            copied_outputs.action_queue.into_actions(),
            original_outputs.action_queue.into_actions()
        );
    }
}

#[test]
fn noop_insertion_can_change_step_cap_without_being_an_energy_exhaustion() {
    let original = vec![
        VmInstruction::PushAction { action_type: 3 },
        VmInstruction::ExecuteActionQueue,
    ];
    let mut inserted = original.clone();
    crate::mutation::vm::insert_new_instruction_with_reference_repair(
        &mut inserted,
        0,
        VmInstruction::Noop,
    )
    .unwrap();
    let mut cfg = config();
    cfg.max_vm_steps = 2;
    cfg.vm.opcode_cost_multiplier = 1.0;

    let (original_result, _, _) = run_vm_with_config(
        original,
        1,
        vec![],
        &[],
        zeroed_upstream(),
        100.0,
        cfg.clone(),
    );
    let (inserted_result, _, _) =
        run_vm_with_config(inserted, 1, vec![], &[], zeroed_upstream(), 100.0, cfg);

    assert!(original_result.terminal);
    assert!(
        !inserted_result.terminal,
        "the added step reaches the cap first"
    );
    assert!(!inserted_result.energy_exhausted);
}

#[test]
fn noop_insertion_can_cause_energy_exhaustion_without_reaching_the_step_cap() {
    let original = vec![
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        },
        VmInstruction::WriteInternalPayload {
            slot_idx: 0,
            src: 0,
        },
        VmInstruction::Halt,
    ];
    let mut inserted = original.clone();
    crate::mutation::vm::insert_new_instruction_with_reference_repair(
        &mut inserted,
        0,
        VmInstruction::Noop,
    )
    .unwrap();
    let mut cfg = config();
    cfg.max_vm_steps = 10;
    cfg.vm.opcode_cost_multiplier = 1.0;

    let (original_result, _, _) = run_vm_with_config(
        original,
        1,
        vec![7.0],
        &[],
        zeroed_upstream(),
        0.30,
        cfg.clone(),
    );
    let (inserted_result, _, _) =
        run_vm_with_config(inserted, 1, vec![7.0], &[], zeroed_upstream(), 0.30, cfg);

    assert!(!original_result.energy_exhausted);
    assert!(inserted_result.energy_exhausted);
}
