use super::*;

// ── Arithmetic opcodes ────────────────────────────────────────────────

#[test]
fn add_two_constants() {
    let program = vec![
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        }, // r0 = 3.0
        VmInstruction::LoadConst {
            dst: 1,
            const_idx: 1,
        }, // r1 = 4.0
        VmInstruction::Add { dst: 2, a: 0, b: 1 }, // r2 = 7.0
        VmInstruction::WriteInternalPayload {
            slot_idx: 0,
            src: 2,
        },
        VmInstruction::Halt,
    ];
    let (r, _, _) = run_vm(program, 3, vec![3.0, 4.0], &[], zeroed_upstream(), 100.0);
    assert!((r.output_slots[0] - 7.0).abs() < 1e-6);
}

#[test]
fn move_copies_register_value() {
    let program = vec![
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        }, // r0 = 9.5
        VmInstruction::Move { dst: 1, src: 0 }, // r1 = r0
        VmInstruction::WriteInternalPayload {
            slot_idx: 0,
            src: 1,
        },
        VmInstruction::Halt,
    ];
    let (r, _, _) = run_vm(program, 2, vec![9.5], &[], zeroed_upstream(), 100.0);
    assert!((r.output_slots[0] - 9.5).abs() < 1e-6);
}

#[test]
fn mul_result() {
    let program = vec![
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        }, // r0 = 3.0
        VmInstruction::LoadConst {
            dst: 1,
            const_idx: 1,
        }, // r1 = 4.0
        VmInstruction::Mul { dst: 2, a: 0, b: 1 }, // r2 = 12.0
        VmInstruction::WriteInternalPayload {
            slot_idx: 1,
            src: 2,
        },
        VmInstruction::Halt,
    ];
    let (r, _, _) = run_vm(program, 3, vec![3.0, 4.0], &[], zeroed_upstream(), 100.0);
    assert!((r.output_slots[1] - 12.0).abs() < 1e-6);
}

#[test]
fn sub_result() {
    let program = vec![
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        }, // r0 = 10.0
        VmInstruction::LoadConst {
            dst: 1,
            const_idx: 1,
        }, // r1 = 3.0
        VmInstruction::Sub { dst: 2, a: 0, b: 1 }, // r2 = 7.0
        VmInstruction::WriteInternalPayload {
            slot_idx: 1,
            src: 2,
        },
        VmInstruction::Halt,
    ];
    let (r, _, _) = run_vm(program, 3, vec![10.0, 3.0], &[], zeroed_upstream(), 100.0);
    assert!((r.output_slots[1] - 7.0).abs() < 1e-6);
}

#[test]
fn div_by_zero_yields_zero() {
    let program = vec![
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        }, // r0 = 5.0
        // r1 stays 0.0 (initial)
        VmInstruction::Div { dst: 2, a: 0, b: 1 }, // r2 = 5/0 = 0
        VmInstruction::WriteInternalPayload {
            slot_idx: 0,
            src: 2,
        },
        VmInstruction::Halt,
    ];
    let (r, _, _) = run_vm(program, 3, vec![5.0], &[], zeroed_upstream(), 100.0);
    assert_eq!(r.output_slots[0], 0.0);
}

#[test]
fn min_picks_smaller() {
    let program = vec![
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        }, // r0 = 3.0
        VmInstruction::LoadConst {
            dst: 1,
            const_idx: 1,
        }, // r1 = 8.0
        VmInstruction::Min { dst: 2, a: 0, b: 1 },
        VmInstruction::WriteInternalPayload {
            slot_idx: 0,
            src: 2,
        },
        VmInstruction::Halt,
    ];
    let (r, _, _) = run_vm(program, 3, vec![3.0, 8.0], &[], zeroed_upstream(), 100.0);
    assert!((r.output_slots[0] - 3.0).abs() < 1e-6);
}

#[test]
fn max_picks_larger() {
    let program = vec![
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        }, // r0 = 3.0
        VmInstruction::LoadConst {
            dst: 1,
            const_idx: 1,
        }, // r1 = 8.0
        VmInstruction::Max { dst: 2, a: 0, b: 1 },
        VmInstruction::WriteInternalPayload {
            slot_idx: 0,
            src: 2,
        },
        VmInstruction::Halt,
    ];
    let (r, _, _) = run_vm(program, 3, vec![3.0, 8.0], &[], zeroed_upstream(), 100.0);
    assert!((r.output_slots[0] - 8.0).abs() < 1e-6);
}

#[test]
fn abs_of_negative() {
    let program = vec![
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        }, // r0 = -5.0
        VmInstruction::Abs { dst: 1, src: 0 },
        VmInstruction::WriteInternalPayload {
            slot_idx: 0,
            src: 1,
        },
        VmInstruction::Halt,
    ];
    let (r, _, _) = run_vm(program, 2, vec![-5.0], &[], zeroed_upstream(), 100.0);
    assert!((r.output_slots[0] - 5.0).abs() < 1e-6);
}

#[test]
fn neg_flips_sign() {
    let program = vec![
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        }, // r0 = 3.0
        VmInstruction::Neg { dst: 1, src: 0 },
        VmInstruction::WriteInternalPayload {
            slot_idx: 0,
            src: 1,
        },
        VmInstruction::Halt,
    ];
    let (r, _, _) = run_vm(program, 2, vec![3.0], &[], zeroed_upstream(), 100.0);
    assert!((r.output_slots[0] - (-3.0)).abs() < 1e-6);
}

#[test]
fn clamp01_clamps_above_one() {
    let program = vec![
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        }, // r0 = 5.0
        VmInstruction::Clamp01 { dst: 1, src: 0 },
        VmInstruction::WriteInternalPayload {
            slot_idx: 0,
            src: 1,
        },
        VmInstruction::Halt,
    ];
    let (r, _, _) = run_vm(program, 2, vec![5.0], &[], zeroed_upstream(), 100.0);
    assert!((r.output_slots[0] - 1.0).abs() < 1e-6);
}

// ── Comparison opcodes ────────────────────────────────────────────────

#[test]
fn cmpgt_true_when_a_greater() {
    let program = vec![
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        }, // r0 = 5.0
        VmInstruction::LoadConst {
            dst: 1,
            const_idx: 1,
        }, // r1 = 3.0
        VmInstruction::CmpGt { dst: 2, a: 0, b: 1 },
        VmInstruction::WriteInternalPayload {
            slot_idx: 0,
            src: 2,
        },
        VmInstruction::Halt,
    ];
    let (r, _, _) = run_vm(program, 3, vec![5.0, 3.0], &[], zeroed_upstream(), 100.0);
    assert_eq!(r.output_slots[0], 1.0);
}

#[test]
fn cmpgt_false_when_a_equal() {
    let program = vec![
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        }, // r0 = 3.0
        VmInstruction::CmpGt { dst: 1, a: 0, b: 0 }, // 3.0 > 3.0 = false
        VmInstruction::WriteInternalPayload {
            slot_idx: 0,
            src: 1,
        },
        VmInstruction::Halt,
    ];
    let (r, _, _) = run_vm(program, 2, vec![3.0], &[], zeroed_upstream(), 100.0);
    assert_eq!(r.output_slots[0], 0.0);
}

#[test]
fn cmplt_true_when_a_less() {
    let program = vec![
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        }, // r0 = 2.0
        VmInstruction::LoadConst {
            dst: 1,
            const_idx: 1,
        }, // r1 = 5.0
        VmInstruction::CmpLt { dst: 2, a: 0, b: 1 },
        VmInstruction::WriteInternalPayload {
            slot_idx: 0,
            src: 2,
        },
        VmInstruction::Halt,
    ];
    let (r, _, _) = run_vm(program, 3, vec![2.0, 5.0], &[], zeroed_upstream(), 100.0);
    assert_eq!(r.output_slots[0], 1.0);
}

#[test]
fn cmpeq_true_within_epsilon() {
    let program = vec![
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        }, // r0 = 1.0
        VmInstruction::LoadConst {
            dst: 1,
            const_idx: 1,
        }, // r1 = 1.0001
        VmInstruction::LoadConst {
            dst: 2,
            const_idx: 2,
        }, // r2 = 0.01 (eps)
        VmInstruction::CmpEq {
            dst: 3,
            a: 0,
            b: 1,
            eps: 2,
        },
        VmInstruction::WriteInternalPayload {
            slot_idx: 0,
            src: 3,
        },
        VmInstruction::Halt,
    ];
    let (r, _, _) = run_vm(
        program,
        4,
        vec![1.0, 1.0001, 0.01],
        &[],
        zeroed_upstream(),
        100.0,
    );
    assert_eq!(r.output_slots[0], 1.0);
}

#[test]
fn and_both_true() {
    let program = vec![
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        }, // r0 = 1.0 (truthy)
        VmInstruction::LoadConst {
            dst: 1,
            const_idx: 1,
        }, // r1 = 0.8 (truthy)
        VmInstruction::And { dst: 2, a: 0, b: 1 },
        VmInstruction::WriteInternalPayload {
            slot_idx: 0,
            src: 2,
        },
        VmInstruction::Halt,
    ];
    let (r, _, _) = run_vm(program, 3, vec![1.0, 0.8], &[], zeroed_upstream(), 100.0);
    assert_eq!(r.output_slots[0], 1.0);
}

#[test]
fn and_one_false() {
    let program = vec![
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        }, // r0 = 1.0
        VmInstruction::LoadConst {
            dst: 1,
            const_idx: 1,
        }, // r1 = 0.4 (not truthy, < 0.5)
        VmInstruction::And { dst: 2, a: 0, b: 1 },
        VmInstruction::WriteInternalPayload {
            slot_idx: 0,
            src: 2,
        },
        VmInstruction::Halt,
    ];
    let (r, _, _) = run_vm(program, 3, vec![1.0, 0.4], &[], zeroed_upstream(), 100.0);
    assert_eq!(r.output_slots[0], 0.0);
}

#[test]
fn or_one_true() {
    let program = vec![
        // r0 stays 0.0 (false), r1 = 1.0 (true)
        VmInstruction::LoadConst {
            dst: 1,
            const_idx: 0,
        }, // r1 = 1.0
        VmInstruction::Or { dst: 2, a: 0, b: 1 },
        VmInstruction::WriteInternalPayload {
            slot_idx: 0,
            src: 2,
        },
        VmInstruction::Halt,
    ];
    let (r, _, _) = run_vm(program, 3, vec![1.0], &[], zeroed_upstream(), 100.0);
    assert_eq!(r.output_slots[0], 1.0);
}

#[test]
fn not_inverts() {
    let program = vec![
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        }, // r0 = 1.0 (truthy)
        VmInstruction::Not { dst: 1, src: 0 },
        VmInstruction::WriteInternalPayload {
            slot_idx: 0,
            src: 1,
        },
        VmInstruction::Halt,
    ];
    let (r, _, _) = run_vm(program, 2, vec![1.0], &[], zeroed_upstream(), 100.0);
    assert_eq!(r.output_slots[0], 0.0);
}

// ── Type conversion opcodes ───────────────────────────────────────────────

#[test]
fn to_i32_rounds_half_up() {
    let program = vec![
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        }, // r0 = 2.5
        VmInstruction::ToI32 { dst: 1, src: 0 },
        VmInstruction::WriteInternalPayload {
            slot_idx: 0,
            src: 1,
        },
        VmInstruction::Halt,
    ];
    let (r, _, _) = run_vm(program, 2, vec![2.5], &[], zeroed_upstream(), 100.0);
    assert!((r.output_slots[0] - 3.0).abs() < 1e-6); // ties-away-from-zero
}

#[test]
fn to_u8_clamps_above_255() {
    let program = vec![
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        }, // r0 = 300.0
        VmInstruction::ToU8 { dst: 1, src: 0 },
        VmInstruction::WriteInternalPayload {
            slot_idx: 0,
            src: 1,
        },
        VmInstruction::Halt,
    ];
    let (r, _, _) = run_vm(program, 2, vec![300.0], &[], zeroed_upstream(), 100.0);
    assert!((r.output_slots[0] - 255.0).abs() < 1e-6);
}

#[test]
fn slot_imm_idx_255_wraps_and_executes_without_panic() {
    let program = vec![
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        },
        VmInstruction::StoreSlotImm {
            slot_idx: 255,
            src: 0,
        },
        VmInstruction::LoadSlotImm {
            dst: 1,
            slot_idx: 255,
        },
        VmInstruction::WriteInternalPayload {
            slot_idx: 0,
            src: 1,
        },
        VmInstruction::Halt,
    ];
    let (r, _, _) = run_vm(program, 2, vec![42.0], &[], zeroed_upstream(), 100.0);
    assert!((r.output_slots[0] - 42.0).abs() < 1e-6);
}

#[test]
fn to_bool_one_is_truthy() {
    let program = vec![
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        }, // r0 = 0.6
        VmInstruction::ToBool { dst: 1, src: 0 },
        VmInstruction::WriteInternalPayload {
            slot_idx: 0,
            src: 1,
        },
        VmInstruction::Halt,
    ];
    let (r, _, _) = run_vm(program, 2, vec![0.6], &[], zeroed_upstream(), 100.0);
    assert_eq!(r.output_slots[0], 1.0);
}

// ── SetPriorityBid opcode ─────────────────────────────────────────────

#[test]
fn set_priority_bid_records_the_bid_and_pays_only_its_opcode_cost() {
    // Load 5.0 into r0, then SetPriorityBid { src: 0 }: the bid is recorded
    // for the mesh to settle once at evaluation end (T19.F02); the dispatch
    // pays only the opcode costs.
    let program = vec![
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        },
        VmInstruction::SetPriorityBid { src: 0 },
        VmInstruction::Halt,
    ];
    let (_, energy, side_outputs) = run_vm(program, 1, vec![5.0], &[], zeroed_upstream(), 100.0);
    assert_eq!(side_outputs.priority_bid, 5.0);
    assert_eq!(side_outputs.energy_observation.priority_bid, 0.0);
    let multiplier = config().vm.opcode_cost_multiplier;
    let expected = 100.0 - (0.08 + 0.20 + 0.05) * multiplier;
    assert!(
        (energy - expected).abs() < 1e-4,
        "energy {} vs expected {}",
        energy,
        expected
    );
}

#[test]
fn set_priority_bid_negative_clamps_to_zero() {
    // Load -3.0 into r0, then SetPriorityBid { src: 0 }.
    // Negative bid should be clamped to 0.0 — no energy deducted beyond opcode cost.
    let program = vec![
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        },
        VmInstruction::SetPriorityBid { src: 0 },
        VmInstruction::Halt,
    ];
    let (_, energy, side_outputs) = run_vm(program, 1, vec![-3.0], &[], zeroed_upstream(), 100.0);
    assert_eq!(side_outputs.priority_bid, 0.0);
    // Only opcode costs should be deducted, no bid energy.
    let multiplier = config().vm.opcode_cost_multiplier;
    let expected = 100.0 - (0.08 + 0.20 + 0.05) * multiplier;
    assert!(
        (energy - expected).abs() < 1e-4,
        "energy {} vs expected {}",
        energy,
        expected
    );
}

#[test]
fn set_priority_bid_exceeding_energy_does_not_exhaust_the_dispatch() {
    // Load 200.0 into r0 with only 10.0 energy: the dispatch records the bid
    // and halts normally; the all-in is the mesh's settlement (T19.F02).
    let program = vec![
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        },
        VmInstruction::SetPriorityBid { src: 0 },
        VmInstruction::Halt,
    ];
    let (result, energy, side_outputs) =
        run_vm(program, 1, vec![200.0], &[], zeroed_upstream(), 10.0);
    assert!(!result.energy_exhausted);
    assert_eq!(side_outputs.priority_bid, 200.0);
    assert!(energy > 9.0, "only opcode costs were paid: {energy}");
}
