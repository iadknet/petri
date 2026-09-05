use rand::Rng;
use std::ops::Range;

use crate::config::MutationConfig;
use crate::contracts::MAX_GATE_SLOTS;
use crate::creature::genome::analysis::{vm_backward_slice_random, vm_forward_slice_random};
use crate::creature::genome::{BackendDef, CreatureGenome, VmInstruction};
use crate::mutation::types::MutationSkipReason;

pub(super) fn apply_constant_mutation(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let node = &mut genome.nodes[node_idx];
    if let BackendDef::Vm(ref mut vm) = node.backend_def {
        if vm.constants.is_empty() {
            // If pool is empty, add one random constant.
            vm.constants.push(rng.gen_range(-1.0f32..=1.0));
        } else {
            let idx = rng.gen_range(0..vm.constants.len());
            let scale = 1.0f32;
            vm.constants[idx] += rng.gen_range(-1.0f32..=1.0) * scale;
        }
    }
    Ok(())
}

pub(super) fn apply_register_count_mutation(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let node = &mut genome.nodes[node_idx];
    if let BackendDef::Vm(ref mut vm) = node.backend_def {
        let old_width = vm.register_count;
        if !(1..=32).contains(&old_width) {
            return Err(MutationSkipReason::NoApplicableTarget);
        }
        let grow = rng.gen_bool(0.5);
        let new_width = if grow {
            old_width.checked_add(1)
        } else {
            old_width.checked_sub(1)
        }
        .filter(|width| (1..=32).contains(width))
        .ok_or(MutationSkipReason::NoApplicableTarget)?;

        let mut canonical_program = vm.program.clone();
        let removed_register = old_width - 1;
        let mut uses_removed_register = false;
        for instruction in &mut canonical_program {
            for_each_register_ref(instruction, &mut |register| {
                *register %= old_width;
                uses_removed_register |= !grow && *register == removed_register;
            });
        }
        if uses_removed_register {
            return Err(MutationSkipReason::NoApplicableTarget);
        }
        vm.program = canonical_program;
        vm.register_count = new_width;
    }
    Ok(())
}

/// Generate a random VM instruction with contextually valid field values.
///
/// All register indices are bounded by `register_count`, constant indices by `constants_len`,
/// and input indices by `input_refs_len`. Zero-length parameters are clamped to produce
/// index 0 (safe — the VM treats out-of-range as a soft default).
pub(super) fn random_vm_instruction(
    rng: &mut impl Rng,
    register_count: u8,
    constants_len: usize,
    input_refs_len: usize,
) -> VmInstruction {
    // Inline helpers to avoid closure borrow conflicts on `rng`.
    let rc = register_count.max(1);
    let cl = constants_len.clamp(1, 255) as u8;
    let il = input_refs_len.clamp(1, 255) as u8;

    match rng.gen_range(0u8..41) {
        0 => VmInstruction::Noop,
        1 => VmInstruction::LoadConst {
            dst: rng.gen_range(0..rc),
            const_idx: rng.gen_range(0..cl),
        },
        2 => VmInstruction::Move {
            dst: rng.gen_range(0..rc),
            src: rng.gen_range(0..rc),
        },
        3 => VmInstruction::Add {
            dst: rng.gen_range(0..rc),
            a: rng.gen_range(0..rc),
            b: rng.gen_range(0..rc),
        },
        4 => VmInstruction::Sub {
            dst: rng.gen_range(0..rc),
            a: rng.gen_range(0..rc),
            b: rng.gen_range(0..rc),
        },
        5 => VmInstruction::Mul {
            dst: rng.gen_range(0..rc),
            a: rng.gen_range(0..rc),
            b: rng.gen_range(0..rc),
        },
        6 => VmInstruction::Div {
            dst: rng.gen_range(0..rc),
            a: rng.gen_range(0..rc),
            b: rng.gen_range(0..rc),
        },
        7 => VmInstruction::Min {
            dst: rng.gen_range(0..rc),
            a: rng.gen_range(0..rc),
            b: rng.gen_range(0..rc),
        },
        8 => VmInstruction::Max {
            dst: rng.gen_range(0..rc),
            a: rng.gen_range(0..rc),
            b: rng.gen_range(0..rc),
        },
        9 => VmInstruction::Abs {
            dst: rng.gen_range(0..rc),
            src: rng.gen_range(0..rc),
        },
        10 => VmInstruction::Neg {
            dst: rng.gen_range(0..rc),
            src: rng.gen_range(0..rc),
        },
        11 => VmInstruction::Clamp01 {
            dst: rng.gen_range(0..rc),
            src: rng.gen_range(0..rc),
        },
        12 => VmInstruction::CmpGt {
            dst: rng.gen_range(0..rc),
            a: rng.gen_range(0..rc),
            b: rng.gen_range(0..rc),
        },
        13 => VmInstruction::CmpLt {
            dst: rng.gen_range(0..rc),
            a: rng.gen_range(0..rc),
            b: rng.gen_range(0..rc),
        },
        14 => VmInstruction::CmpEq {
            dst: rng.gen_range(0..rc),
            a: rng.gen_range(0..rc),
            b: rng.gen_range(0..rc),
            eps: rng.gen_range(0..rc),
        },
        15 => VmInstruction::And {
            dst: rng.gen_range(0..rc),
            a: rng.gen_range(0..rc),
            b: rng.gen_range(0..rc),
        },
        16 => VmInstruction::Or {
            dst: rng.gen_range(0..rc),
            a: rng.gen_range(0..rc),
            b: rng.gen_range(0..rc),
        },
        17 => VmInstruction::Not {
            dst: rng.gen_range(0..rc),
            src: rng.gen_range(0..rc),
        },
        18 => VmInstruction::ToI32 {
            dst: rng.gen_range(0..rc),
            src: rng.gen_range(0..rc),
        },
        19 => VmInstruction::ToU8 {
            dst: rng.gen_range(0..rc),
            src: rng.gen_range(0..rc),
        },
        20 => VmInstruction::ToBool {
            dst: rng.gen_range(0..rc),
            src: rng.gen_range(0..rc),
        },
        21 => VmInstruction::JumpIfZero {
            cond: rng.gen_range(0..rc),
            offset: rng.gen_range(-16i32..=16),
        },
        22 => VmInstruction::Jump {
            offset: rng.gen_range(-16i32..=16),
        },
        23 => VmInstruction::ReadInput {
            dst: rng.gen_range(0..rc),
            ref_idx: rng.gen_range(0..il) as u16,
            sub_idx: 0,
        },
        24 => VmInstruction::WriteInternalPayload {
            slot_idx: rng.gen_range(0u8..8),
            src: rng.gen_range(0..rc),
        },
        25 => VmInstruction::WriteWorldActionMeta {
            slot_idx: rng.gen_range(0u8..8),
            src: rng.gen_range(0..rc),
        },
        26 => VmInstruction::WriteRouteGate {
            slot: rng.gen_range(0..MAX_GATE_SLOTS as u8),
            src: rng.gen_range(0..rc),
        },
        27 => VmInstruction::PushAction {
            action_type: rng.gen(),
        },
        28 => VmInstruction::PopAction,
        29 => VmInstruction::ReadActionQueueLength {
            dst: rng.gen_range(0..rc),
        },
        30 => VmInstruction::ReadActionQueueType {
            index_src: rng.gen_range(0..rc),
            dst: rng.gen_range(0..rc),
        },
        31 => VmInstruction::ReadActionQueueParam {
            index_src: rng.gen_range(0..rc),
            param_slot: rng.gen_range(0..8),
            dst: rng.gen_range(0..rc),
        },
        32 => VmInstruction::ExecuteActionQueue,
        33 => VmInstruction::Halt,
        34 => VmInstruction::LoadSlot {
            dst: rng.gen_range(0..rc),
            slot_reg: rng.gen_range(0..rc),
        },
        35 => VmInstruction::StoreSlot {
            slot_reg: rng.gen_range(0..rc),
            src: rng.gen_range(0..rc),
        },
        36 => VmInstruction::LoadSlotImm {
            dst: rng.gen_range(0..rc),
            slot_idx: rng.gen_range(0..16),
        },
        37 => VmInstruction::StoreSlotImm {
            slot_idx: rng.gen_range(0..16),
            src: rng.gen_range(0..rc),
        },
        38 => VmInstruction::SetPriorityBid {
            src: rng.gen_range(0..rc),
        },
        39 => VmInstruction::LoadSlotPrev {
            dst: rng.gen_range(0..rc),
            slot_idx: rng.gen_range(0..16),
        },
        40 => VmInstruction::ClearSlot {
            slot_idx: rng.gen_range(0..16),
        },
        _ => unreachable!("gen_range(0..41) cannot produce values >= 41"),
    }
}

fn nudge_u8(value: &mut u8, rng: &mut impl Rng) {
    *value = match *value {
        0 => 1,
        u8::MAX => u8::MAX - 1,
        _ if rng.gen_bool(0.5) => value.saturating_add(1),
        _ => value.saturating_sub(1),
    };
}

fn nudge_u16(value: &mut u16, rng: &mut impl Rng) {
    *value = match *value {
        0 => 1,
        u16::MAX => u16::MAX - 1,
        _ if rng.gen_bool(0.5) => value.saturating_add(1),
        _ => value.saturating_sub(1),
    };
}

fn nudge_i32(value: &mut i32, rng: &mut impl Rng) {
    *value = match *value {
        i32::MIN => i32::MIN + 1,
        i32::MAX => i32::MAX - 1,
        _ if rng.gen_bool(0.5) => value.saturating_add(1),
        _ => value.saturating_sub(1),
    };
}

fn nudge_one_u8(fields: &mut [&mut u8], rng: &mut impl Rng) {
    let selected = rng.gen_range(0..fields.len());
    nudge_u8(fields[selected], rng);
}

pub(super) fn mutate_one_instruction_field(
    instruction: &mut VmInstruction,
    rng: &mut impl Rng,
) -> bool {
    match instruction {
        VmInstruction::Noop
        | VmInstruction::Halt
        | VmInstruction::ExecuteActionQueue
        | VmInstruction::PopAction => false,
        VmInstruction::LoadConst { dst, const_idx } => {
            nudge_one_u8(&mut [dst, const_idx], rng);
            true
        }
        VmInstruction::Move { dst, src }
        | VmInstruction::Abs { dst, src }
        | VmInstruction::Neg { dst, src }
        | VmInstruction::Clamp01 { dst, src }
        | VmInstruction::Not { dst, src }
        | VmInstruction::ToI32 { dst, src }
        | VmInstruction::ToU8 { dst, src }
        | VmInstruction::ToBool { dst, src }
        | VmInstruction::ReadActionQueueType {
            index_src: dst,
            dst: src,
        } => {
            nudge_one_u8(&mut [dst, src], rng);
            true
        }
        VmInstruction::Add { dst, a, b }
        | VmInstruction::Sub { dst, a, b }
        | VmInstruction::Mul { dst, a, b }
        | VmInstruction::Div { dst, a, b }
        | VmInstruction::Min { dst, a, b }
        | VmInstruction::Max { dst, a, b }
        | VmInstruction::CmpGt { dst, a, b }
        | VmInstruction::CmpLt { dst, a, b }
        | VmInstruction::And { dst, a, b }
        | VmInstruction::Or { dst, a, b } => {
            nudge_one_u8(&mut [dst, a, b], rng);
            true
        }
        VmInstruction::CmpEq { dst, a, b, eps } => {
            nudge_one_u8(&mut [dst, a, b, eps], rng);
            true
        }
        VmInstruction::JumpIfZero { cond, offset } => {
            if rng.gen_bool(0.5) {
                nudge_u8(cond, rng);
            } else {
                nudge_i32(offset, rng);
            }
            true
        }
        VmInstruction::Jump { offset } => {
            nudge_i32(offset, rng);
            true
        }
        VmInstruction::ReadInput {
            dst,
            ref_idx,
            sub_idx,
        } => {
            match rng.gen_range(0..3) {
                0 => nudge_u8(dst, rng),
                1 => nudge_u16(ref_idx, rng),
                _ => nudge_u16(sub_idx, rng),
            }
            true
        }
        VmInstruction::WriteInternalPayload { slot_idx, src }
        | VmInstruction::WriteWorldActionMeta { slot_idx, src } => {
            nudge_one_u8(&mut [slot_idx, src], rng);
            true
        }
        VmInstruction::WriteRouteGate { slot, src } => {
            nudge_one_u8(&mut [slot, src], rng);
            true
        }
        VmInstruction::PushAction { action_type }
        | VmInstruction::SetPriorityBid { src: action_type }
        | VmInstruction::ReadActionQueueLength { dst: action_type }
        | VmInstruction::ClearSlot {
            slot_idx: action_type,
        } => {
            nudge_u8(action_type, rng);
            true
        }
        VmInstruction::ReadActionQueueParam {
            index_src,
            param_slot,
            dst,
        } => {
            nudge_one_u8(&mut [index_src, param_slot, dst], rng);
            true
        }
        VmInstruction::LoadSlot { dst, slot_reg } => {
            nudge_one_u8(&mut [dst, slot_reg], rng);
            true
        }
        VmInstruction::StoreSlot { slot_reg, src } => {
            nudge_one_u8(&mut [slot_reg, src], rng);
            true
        }
        VmInstruction::LoadSlotImm { dst, slot_idx }
        | VmInstruction::LoadSlotPrev { dst, slot_idx } => {
            nudge_one_u8(&mut [dst, slot_idx], rng);
            true
        }
        VmInstruction::StoreSlotImm { slot_idx, src } => {
            nudge_one_u8(&mut [slot_idx, src], rng);
            true
        }
    }
}

pub(super) fn apply_instruction_mutation(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    // Extract context values before entering mutable borrow on the node
    // (own-borrow-over-clone: avoid conflicting borrows on genome.nodes[node_idx]).
    let (register_count, constants_len, input_refs_len) = {
        let node = &genome.nodes[node_idx];
        let (rc, cl) = if let BackendDef::Vm(ref vm) = node.backend_def {
            (vm.register_count, vm.constants.len())
        } else {
            return Ok(());
        };
        (rc, cl, node.input_refs.len())
    };

    let node = &mut genome.nodes[node_idx];
    if let BackendDef::Vm(ref mut vm) = node.backend_def {
        if vm.program.is_empty() {
            // Edge case: empty program — insert a random instruction.
            insert_new_instruction_with_reference_repair(
                &mut vm.program,
                0,
                random_vm_instruction(rng, register_count, constants_len, input_refs_len),
            )?;
            return Ok(());
        }

        // Choose: insert (0), replace (1), delete (2).
        let choice = rng.gen_range(0u8..3);
        match choice {
            0 => {
                // Insert: push a random instruction at a random position.
                let pos = rng.gen_range(0..=vm.program.len());
                let instr =
                    random_vm_instruction(rng, register_count, constants_len, input_refs_len);
                insert_new_instruction_with_reference_repair(&mut vm.program, pos, instr)?;
            }
            1 => {
                // Replace: replace a random instruction with a random one.
                let idx = rng.gen_range(0..vm.program.len());
                splice_program_with_reference_repair(
                    &mut vm.program,
                    idx..idx + 1,
                    vec![SpliceInstruction {
                        instruction: random_vm_instruction(
                            rng,
                            register_count,
                            constants_len,
                            input_refs_len,
                        ),
                        source_index: None,
                    }],
                )?;
            }
            _ => {
                // Delete: remove a random instruction, keep at least 1.
                if vm.program.len() > 1 {
                    let idx = rng.gen_range(0..vm.program.len());
                    splice_program_with_reference_repair(&mut vm.program, idx..idx + 1, vec![])?;
                } else {
                    // Single instruction — replace with random rather than emptying the program.
                    splice_program_with_reference_repair(
                        &mut vm.program,
                        0..1,
                        vec![SpliceInstruction {
                            instruction: random_vm_instruction(
                                rng,
                                register_count,
                                constants_len,
                                input_refs_len,
                            ),
                            source_index: None,
                        }],
                    )?;
                }
            }
        }
    }
    Ok(())
}

pub(super) fn apply_delete_instruction(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let node = &mut genome.nodes[node_idx];
    if let BackendDef::Vm(ref mut vm) = node.backend_def {
        if vm.program.len() <= 1 {
            return Err(MutationSkipReason::NoApplicableTarget);
        }
        let idx = rng.gen_range(0..vm.program.len());
        splice_program_with_reference_repair(&mut vm.program, idx..idx + 1, vec![])?;
    }
    Ok(())
}

pub(super) fn apply_instruction_raw_field_mutation(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
    _config: &MutationConfig,
) -> Result<(), MutationSkipReason> {
    let node = &mut genome.nodes[node_idx];
    let BackendDef::Vm(ref mut vm) = node.backend_def else {
        return Ok(());
    };
    let eligible: Vec<usize> = vm
        .program
        .iter()
        .enumerate()
        .filter(|(_, instruction)| {
            !matches!(
                instruction,
                VmInstruction::Noop
                    | VmInstruction::Halt
                    | VmInstruction::ExecuteActionQueue
                    | VmInstruction::PopAction
            )
        })
        .map(|(index, _)| index)
        .collect();
    let index = *eligible
        .get(rng.gen_range(0..eligible.len().max(1)))
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
    if mutate_one_instruction_field(&mut vm.program[index], rng) {
        Ok(())
    } else {
        Err(MutationSkipReason::NoApplicableTarget)
    }
}

/// Remap all register-typed fields: `(reg + offset) % register_count`.
fn remap_register_refs(instr: &mut VmInstruction, offset: u8, register_count: u8) {
    let rc = register_count.max(1);
    for_each_register_ref(instr, &mut |reg| {
        let canonical = *reg % rc;
        let shifted = u16::from(canonical) + u16::from(offset);
        *reg = u8::try_from(shifted % u16::from(rc))
            .expect("register remapping result is bounded by register_count");
    });
}

fn for_each_register_ref(instruction: &mut VmInstruction, callback: &mut impl FnMut(&mut u8)) {
    match instruction {
        VmInstruction::Noop | VmInstruction::Halt | VmInstruction::Jump { .. } => {}
        VmInstruction::LoadConst { dst, .. }
        | VmInstruction::ReadInput { dst, .. }
        | VmInstruction::ReadActionQueueLength { dst }
        | VmInstruction::LoadSlotImm { dst, .. }
        | VmInstruction::LoadSlotPrev { dst, .. } => callback(dst),
        VmInstruction::Move { dst, src }
        | VmInstruction::Abs { dst, src }
        | VmInstruction::Neg { dst, src }
        | VmInstruction::Clamp01 { dst, src }
        | VmInstruction::Not { dst, src }
        | VmInstruction::ToI32 { dst, src }
        | VmInstruction::ToU8 { dst, src }
        | VmInstruction::ToBool { dst, src } => {
            callback(dst);
            callback(src);
        }
        VmInstruction::Add { dst, a, b }
        | VmInstruction::Sub { dst, a, b }
        | VmInstruction::Mul { dst, a, b }
        | VmInstruction::Div { dst, a, b }
        | VmInstruction::Min { dst, a, b }
        | VmInstruction::Max { dst, a, b }
        | VmInstruction::CmpGt { dst, a, b }
        | VmInstruction::CmpLt { dst, a, b }
        | VmInstruction::And { dst, a, b }
        | VmInstruction::Or { dst, a, b } => {
            callback(dst);
            callback(a);
            callback(b);
        }
        VmInstruction::CmpEq { dst, a, b, eps } => {
            callback(dst);
            callback(a);
            callback(b);
            callback(eps);
        }
        VmInstruction::JumpIfZero { cond, .. } => callback(cond),
        VmInstruction::WriteInternalPayload { src, .. }
        | VmInstruction::WriteWorldActionMeta { src, .. }
        | VmInstruction::WriteRouteGate { src, .. }
        | VmInstruction::SetPriorityBid { src }
        | VmInstruction::StoreSlotImm { src, .. } => callback(src),
        VmInstruction::PushAction { .. }
        | VmInstruction::PopAction
        | VmInstruction::ExecuteActionQueue
        | VmInstruction::ClearSlot { .. } => {}
        VmInstruction::ReadActionQueueType { index_src, dst }
        | VmInstruction::ReadActionQueueParam { index_src, dst, .. } => {
            callback(index_src);
            callback(dst);
        }
        VmInstruction::LoadSlot { dst, slot_reg } => {
            callback(dst);
            callback(slot_reg);
        }
        VmInstruction::StoreSlot { slot_reg, src } => {
            callback(slot_reg);
            callback(src);
        }
    }
}

#[derive(Clone)]
pub(crate) struct SpliceInstruction {
    pub(crate) instruction: VmInstruction,
    pub(crate) source_index: Option<usize>,
}

pub(crate) fn insert_new_instruction_with_reference_repair(
    program: &mut Vec<VmInstruction>,
    insert_at: usize,
    instruction: VmInstruction,
) -> Result<(), MutationSkipReason> {
    splice_program_with_reference_repair(
        program,
        insert_at..insert_at,
        vec![SpliceInstruction {
            instruction,
            source_index: None,
        }],
    )
}

pub(crate) fn splice_program_with_reference_repair(
    program: &mut Vec<VmInstruction>,
    replaced: Range<usize>,
    inserted: Vec<SpliceInstruction>,
) -> Result<(), MutationSkipReason> {
    let old_program = program.clone();
    let old_len = old_program.len();
    if replaced.start > replaced.end || replaced.end > old_len {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    if inserted
        .iter()
        .any(|item| item.source_index.is_some_and(|source| source >= old_len))
    {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    let inserted_len = inserted.len();
    let new_len = old_len
        .checked_sub(replaced.end - replaced.start)
        .and_then(|remaining| remaining.checked_add(inserted.len()))
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
    if new_len > i32::MAX as usize {
        return Err(MutationSkipReason::NoApplicableTarget);
    }

    let mut new_items = Vec::with_capacity(new_len);
    for (old_index, instruction) in old_program.iter().take(replaced.start).enumerate() {
        new_items.push(SpliceInstruction {
            instruction: instruction.clone(),
            source_index: Some(old_index),
        });
    }
    new_items.extend(inserted);
    for (old_index, instruction) in old_program.iter().enumerate().skip(replaced.end) {
        new_items.push(SpliceInstruction {
            instruction: instruction.clone(),
            source_index: Some(old_index),
        });
    }

    let mut old_target_positions = vec![None; old_len];
    let mut copied_target_positions = vec![None; old_len];
    for (new_index, item) in new_items.iter().enumerate() {
        if let Some(source_index) = item.source_index {
            if (replaced.start..replaced.start + inserted_len).contains(&new_index) {
                copied_target_positions[source_index] = Some(new_index);
            } else {
                old_target_positions[source_index] = Some(new_index);
            }
        }
    }
    if replaced.end - replaced.start == 1 && new_items.len() == old_len {
        old_target_positions[replaced.start] = Some(replaced.start);
    }

    for (new_pc, item) in new_items.iter_mut().enumerate() {
        let Some(source_index) = item.source_index else {
            continue;
        };
        let old_offset = match old_program[source_index] {
            VmInstruction::Jump { offset } | VmInstruction::JumpIfZero { offset, .. } => offset,
            _ => continue,
        };
        let old_target = crate::runtime::vm::jump_target(source_index, old_offset, old_len);
        let target = if (replaced.start..replaced.start + inserted_len).contains(&new_pc) {
            copied_target_positions[old_target].or(old_target_positions[old_target])
        } else {
            old_target_positions[old_target]
        }
        .or_else(|| {
            (1..old_len)
                .map(|distance| (old_target + distance) % old_len)
                .find_map(|candidate| old_target_positions[candidate])
        })
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
        let encoded = i32::try_from(target as i64 - (new_pc as i64 + 1))
            .map_err(|_| MutationSkipReason::NoApplicableTarget)?;
        match &mut item.instruction {
            VmInstruction::Jump { offset } | VmInstruction::JumpIfZero { offset, .. } => {
                *offset = encoded;
            }
            _ => unreachable!("only old jump instructions reach this branch"),
        }
    }
    *program = new_items.into_iter().map(|item| item.instruction).collect();
    Ok(())
}

// ── VM Copy Operators ──

pub(super) fn apply_copy_instruction_block(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let node = &mut genome.nodes[node_idx];
    if let BackendDef::Vm(ref mut vm) = node.backend_def {
        if vm.program.is_empty() {
            return Err(MutationSkipReason::NoApplicableTarget);
        }
        let block_size = rng.gen_range(2..=32).min(vm.program.len());
        let source_start = rng.gen_range(0..=vm.program.len() - block_size);
        let block = vm.program[source_start..source_start + block_size]
            .iter()
            .enumerate()
            .map(|(offset, instruction)| SpliceInstruction {
                instruction: instruction.clone(),
                source_index: Some(source_start + offset),
            })
            .collect();
        let insert_at = rng.gen_range(0..=vm.program.len());
        splice_program_with_reference_repair(&mut vm.program, insert_at..insert_at, block)?;
    }
    Ok(())
}

pub(super) fn apply_copy_instruction_block_remapped(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let register_count = if let BackendDef::Vm(ref vm) = genome.nodes[node_idx].backend_def {
        vm.register_count
    } else {
        return Ok(());
    };

    let node = &mut genome.nodes[node_idx];
    if let BackendDef::Vm(ref mut vm) = node.backend_def {
        if vm.program.is_empty() {
            return Err(MutationSkipReason::NoApplicableTarget);
        }
        let block_size = rng.gen_range(2..=32).min(vm.program.len());
        let source_start = rng.gen_range(0..=vm.program.len() - block_size);
        let mut block = vm.program[source_start..source_start + block_size]
            .iter()
            .enumerate()
            .map(|(offset, instruction)| SpliceInstruction {
                instruction: instruction.clone(),
                source_index: Some(source_start + offset),
            })
            .collect::<Vec<_>>();
        let reg_offset = rng.gen_range(1..register_count.max(2));
        let insert_at = rng.gen_range(0..=vm.program.len());
        for item in &mut block {
            remap_register_refs(&mut item.instruction, reg_offset, register_count);
        }
        splice_program_with_reference_repair(&mut vm.program, insert_at..insert_at, block)?;
    }
    Ok(())
}

pub(super) fn apply_copy_constant_block(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let node = &mut genome.nodes[node_idx];
    if let BackendDef::Vm(ref mut vm) = node.backend_def {
        if vm.constants.is_empty() {
            return Err(MutationSkipReason::NoApplicableTarget);
        }
        let block_size = rng.gen_range(1..=16).min(vm.constants.len());
        let source_start = rng.gen_range(0..=vm.constants.len() - block_size);
        let block: Vec<f32> = vm.constants[source_start..source_start + block_size].to_vec();
        // Append-only: inserting mid-pool would invalidate existing const_idx references.
        vm.constants.extend_from_slice(&block);
    }
    Ok(())
}

pub(super) fn apply_copy_gene_backward_slice(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let node = &mut genome.nodes[node_idx];
    if let BackendDef::Vm(ref mut vm) = node.backend_def {
        if vm.program.is_empty() {
            return Err(MutationSkipReason::NoApplicableTarget);
        }
        if let Some(gene) = vm_backward_slice_random(&vm.program, rng) {
            let extracted = gene
                .indices
                .iter()
                .map(|&source_index| SpliceInstruction {
                    instruction: vm.program[source_index].clone(),
                    source_index: Some(source_index),
                })
                .collect();
            let insert_at = rng.gen_range(0..=vm.program.len());
            splice_program_with_reference_repair(&mut vm.program, insert_at..insert_at, extracted)?;
        } else {
            return Err(MutationSkipReason::NoApplicableTarget);
        }
    }
    Ok(())
}

pub(super) fn apply_copy_gene_forward_slice(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let node = &mut genome.nodes[node_idx];
    if let BackendDef::Vm(ref mut vm) = node.backend_def {
        if vm.program.is_empty() {
            return Err(MutationSkipReason::NoApplicableTarget);
        }
        if let Some(gene) = vm_forward_slice_random(&vm.program, rng) {
            let extracted = gene
                .indices
                .iter()
                .map(|&source_index| SpliceInstruction {
                    instruction: vm.program[source_index].clone(),
                    source_index: Some(source_index),
                })
                .collect();
            let insert_at = rng.gen_range(0..=vm.program.len());
            splice_program_with_reference_repair(&mut vm.program, insert_at..insert_at, extracted)?;
        } else {
            return Err(MutationSkipReason::NoApplicableTarget);
        }
    }
    Ok(())
}

// ── Slot Motif Operators ──

pub(super) fn apply_insert_read_store_motif(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let input_refs_len = genome.nodes[node_idx].input_refs.len();
    if input_refs_len == 0 {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    let node = &mut genome.nodes[node_idx];
    if let BackendDef::Vm(ref mut vm) = node.backend_def {
        let rc = vm.register_count.max(1);
        let il = input_refs_len.min(255) as u16;
        let dst = rng.gen_range(0..rc);
        let slot_idx = rng.gen_range(0u8..16);
        let pair = [
            VmInstruction::ReadInput {
                dst,
                ref_idx: rng.gen_range(0..il),
                sub_idx: 0,
            },
            VmInstruction::StoreSlotImm { slot_idx, src: dst },
        ];
        let pos = rng.gen_range(0..=vm.program.len());
        for (offset, instruction) in pair.into_iter().enumerate() {
            insert_new_instruction_with_reference_repair(
                &mut vm.program,
                pos + offset,
                instruction,
            )?;
        }
    }
    Ok(())
}

pub(super) fn apply_insert_read_bid_motif(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let input_refs_len = genome.nodes[node_idx].input_refs.len();
    if input_refs_len == 0 {
        return Err(MutationSkipReason::NoApplicableTarget);
    }

    let node = &mut genome.nodes[node_idx];
    if let BackendDef::Vm(ref mut vm) = node.backend_def {
        let rc = vm.register_count.max(1);
        let il = input_refs_len.min(255) as u16;
        let dst = rng.gen_range(0..rc);
        let pair = [
            VmInstruction::ReadInput {
                dst,
                ref_idx: rng.gen_range(0..il),
                sub_idx: 0,
            },
            VmInstruction::SetPriorityBid { src: dst },
        ];

        let terminal_positions: Vec<usize> = vm
            .program
            .iter()
            .enumerate()
            .filter(|(_, instr)| {
                matches!(
                    instr,
                    VmInstruction::ExecuteActionQueue | VmInstruction::Halt
                )
            })
            .map(|(idx, _)| idx)
            .collect();

        let pos = if terminal_positions.is_empty() {
            rng.gen_range(0..=vm.program.len())
        } else {
            terminal_positions[rng.gen_range(0..terminal_positions.len())]
        };

        for (offset, instruction) in pair.into_iter().enumerate() {
            insert_new_instruction_with_reference_repair(
                &mut vm.program,
                pos + offset,
                instruction,
            )?;
        }
    }
    Ok(())
}

pub(super) fn apply_insert_load_compare_motif(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let node = &mut genome.nodes[node_idx];
    if let BackendDef::Vm(ref mut vm) = node.backend_def {
        let rc = vm.register_count.max(1);
        let load_dst = rng.gen_range(0..rc);
        let slot_idx = rng.gen_range(0u8..16);
        let cmp_dst = rng.gen_range(0..rc);
        let cmp_b = rng.gen_range(0..rc);
        let pair = [
            VmInstruction::LoadSlotImm {
                dst: load_dst,
                slot_idx,
            },
            VmInstruction::CmpGt {
                dst: cmp_dst,
                a: load_dst,
                b: cmp_b,
            },
        ];
        let pos = rng.gen_range(0..=vm.program.len());
        for (offset, instruction) in pair.into_iter().enumerate() {
            insert_new_instruction_with_reference_repair(
                &mut vm.program,
                pos + offset,
                instruction,
            )?;
        }
    }
    Ok(())
}

pub(super) fn apply_mutate_slot_address(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let node = &mut genome.nodes[node_idx];
    if let BackendDef::Vm(ref mut vm) = node.backend_def {
        let slot_indices: Vec<usize> = vm
            .program
            .iter()
            .enumerate()
            .filter(|(_, instr)| is_slot_instruction(instr))
            .map(|(i, _)| i)
            .collect();
        if slot_indices.is_empty() {
            return Err(MutationSkipReason::NoApplicableTarget);
        }
        let idx = slot_indices[rng.gen_range(0..slot_indices.len())];
        mutate_slot_idx_field(&mut vm.program[idx], rng);
    }
    Ok(())
}

pub(super) fn apply_mutate_paired_slot_address(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let node = &mut genome.nodes[node_idx];
    if let BackendDef::Vm(ref mut vm) = node.backend_def {
        // Group immediate-addressed slot instructions by slot_idx, tracking which have
        // loads and stores. Register-indirect variants (LoadSlot/StoreSlot) are excluded
        // because they have no static slot_idx field to co-mutate.
        //
        // The map is ordered (not a std HashMap) so that `eligible` below is ordered by
        // ascending slot index, making the candidate the seeded draw picks a function of
        // genome content alone rather than of per-process hash order (T10.F11).
        let mut groups: std::collections::BTreeMap<u8, (Vec<usize>, bool, bool)> =
            std::collections::BTreeMap::new();
        for (i, instr) in vm.program.iter().enumerate() {
            match instr {
                VmInstruction::LoadSlotImm { slot_idx, .. }
                | VmInstruction::LoadSlotPrev { slot_idx, .. } => {
                    let entry = groups
                        .entry(*slot_idx)
                        .or_insert((Vec::new(), false, false));
                    entry.0.push(i);
                    entry.1 = true; // has load
                }
                VmInstruction::StoreSlotImm { slot_idx, .. }
                | VmInstruction::ClearSlot { slot_idx } => {
                    let entry = groups
                        .entry(*slot_idx)
                        .or_insert((Vec::new(), false, false));
                    entry.0.push(i);
                    entry.2 = true; // has store
                }
                _ => {}
            }
        }
        // Filter to groups with both load and store.
        let eligible: Vec<(u8, Vec<usize>)> = groups
            .into_iter()
            .filter(|(_, (_, has_load, has_store))| *has_load && *has_store)
            .map(|(slot, (indices, _, _))| (slot, indices))
            .collect();
        if eligible.is_empty() {
            return Err(MutationSkipReason::NoApplicableTarget);
        }
        let (old_slot, indices) = &eligible[rng.gen_range(0..eligible.len())];
        let new_slot = (*old_slot + 1 + rng.gen_range(0u8..15)) % 16;
        for &idx in indices {
            set_slot_idx_field(&mut vm.program[idx], new_slot);
        }
    }
    Ok(())
}

fn is_slot_instruction(instr: &VmInstruction) -> bool {
    matches!(
        instr,
        VmInstruction::LoadSlot { .. }
            | VmInstruction::StoreSlot { .. }
            | VmInstruction::LoadSlotImm { .. }
            | VmInstruction::StoreSlotImm { .. }
            | VmInstruction::LoadSlotPrev { .. }
            | VmInstruction::ClearSlot { .. }
    )
}

fn mutate_slot_idx_field(instr: &mut VmInstruction, rng: &mut impl Rng) {
    match instr {
        VmInstruction::LoadSlotImm { slot_idx, .. }
        | VmInstruction::StoreSlotImm { slot_idx, .. }
        | VmInstruction::LoadSlotPrev { slot_idx, .. }
        | VmInstruction::ClearSlot { slot_idx } => {
            nudge_u8(slot_idx, rng);
        }
        VmInstruction::LoadSlot { slot_reg, .. } | VmInstruction::StoreSlot { slot_reg, .. } => {
            // For register-indirect slot access, mutate the slot_reg.
            nudge_u8(slot_reg, rng);
        }
        _ => {}
    }
}

fn set_slot_idx_field(instr: &mut VmInstruction, new_slot: u8) {
    match instr {
        VmInstruction::LoadSlotImm { slot_idx, .. }
        | VmInstruction::StoreSlotImm { slot_idx, .. }
        | VmInstruction::LoadSlotPrev { slot_idx, .. }
        | VmInstruction::ClearSlot { slot_idx } => {
            *slot_idx = new_slot;
        }
        _ => {}
    }
}
