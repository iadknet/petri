use rand::Rng;

use crate::config::MutationConfig;
use crate::contracts::{InputReference, MAX_GATE_SLOTS};
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
        if rng.gen_bool(0.5) {
            // Increment, clamped to 32.
            vm.register_count = vm.register_count.saturating_add(1).min(32);
        } else {
            // Decrement, clamped to 1.
            vm.register_count = vm.register_count.saturating_sub(1).max(1);
        }
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

fn mutate_instruction_raw_fields(
    instr: &mut VmInstruction,
    rng: &mut impl Rng,
    input_refs: &[InputReference],
    config: &MutationConfig,
) {
    match instr {
        VmInstruction::Noop | VmInstruction::Halt => {
            *instr = VmInstruction::PushAction {
                action_type: rng.gen(),
            };
        }
        VmInstruction::LoadConst { dst, const_idx } => {
            *dst = rng.gen();
            *const_idx = rng.gen();
        }
        VmInstruction::Move { dst, src }
        | VmInstruction::Abs { dst, src }
        | VmInstruction::Neg { dst, src }
        | VmInstruction::Clamp01 { dst, src }
        | VmInstruction::Not { dst, src }
        | VmInstruction::ToI32 { dst, src }
        | VmInstruction::ToU8 { dst, src }
        | VmInstruction::ToBool { dst, src } => {
            *dst = rng.gen();
            *src = rng.gen();
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
            *dst = rng.gen();
            *a = rng.gen();
            *b = rng.gen();
        }
        VmInstruction::CmpEq { dst, a, b, eps } => {
            *dst = rng.gen();
            *a = rng.gen();
            *b = rng.gen();
            *eps = rng.gen();
        }
        VmInstruction::JumpIfZero { cond, offset } => {
            *cond = rng.gen();
            *offset = rng.gen();
        }
        VmInstruction::Jump { offset } => {
            *offset = rng.gen();
        }
        VmInstruction::ReadInput {
            dst,
            ref_idx,
            sub_idx,
        } => {
            *dst = rng.gen();
            *ref_idx = rng.gen_range(0..input_refs.len().max(1) as u16);
            let width = input_refs
                .get(*ref_idx as usize)
                .map(|r| crate::mutation::compound::sub_value_count(r, config))
                .unwrap_or(1);
            *sub_idx = rng.gen_range(0..width);
        }
        VmInstruction::WriteInternalPayload { slot_idx, src }
        | VmInstruction::WriteWorldActionMeta { slot_idx, src } => {
            *slot_idx = rng.gen();
            *src = rng.gen();
        }
        VmInstruction::WriteRouteGate { slot, src } => {
            if rng.gen_bool(0.5) {
                *slot = rng.gen_range(0..MAX_GATE_SLOTS as u8);
            } else {
                *src = rng.gen::<u8>();
            }
        }
        VmInstruction::LoadSlot { dst, slot_reg } => {
            *dst = rng.gen();
            *slot_reg = rng.gen();
        }
        VmInstruction::StoreSlot { slot_reg, src } => {
            *slot_reg = rng.gen();
            *src = rng.gen();
        }
        VmInstruction::LoadSlotImm { dst, slot_idx } => {
            *dst = rng.gen();
            *slot_idx = rng.gen_range(0..16);
        }
        VmInstruction::StoreSlotImm { slot_idx, src } => {
            *slot_idx = rng.gen_range(0..16);
            *src = rng.gen();
        }
        VmInstruction::LoadSlotPrev { dst, slot_idx } => {
            *dst = rng.gen();
            *slot_idx = rng.gen_range(0..16);
        }
        VmInstruction::ClearSlot { slot_idx } => {
            *slot_idx = rng.gen_range(0..16);
        }
        VmInstruction::PushAction { action_type } => {
            *action_type = rng.gen();
        }
        VmInstruction::PopAction => {
            // No fields to mutate; swap to a different instruction.
            *instr = VmInstruction::PushAction {
                action_type: rng.gen(),
            };
        }
        VmInstruction::ReadActionQueueLength { dst } => {
            *dst = rng.gen();
        }
        VmInstruction::ReadActionQueueType { index_src, dst } => {
            *index_src = rng.gen();
            *dst = rng.gen();
        }
        VmInstruction::ReadActionQueueParam {
            index_src,
            param_slot,
            dst,
        } => {
            *index_src = rng.gen();
            *param_slot = rng.gen();
            *dst = rng.gen();
        }
        VmInstruction::ExecuteActionQueue => {
            // No fields to mutate; swap to a different instruction.
            *instr = VmInstruction::PushAction {
                action_type: rng.gen(),
            };
        }
        VmInstruction::SetPriorityBid { src } => {
            *src = rng.gen();
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
            vm.program.push(random_vm_instruction(
                rng,
                register_count,
                constants_len,
                input_refs_len,
            ));
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
                vm.program.insert(pos, instr);
            }
            1 => {
                // Replace: replace a random instruction with a random one.
                let idx = rng.gen_range(0..vm.program.len());
                vm.program[idx] =
                    random_vm_instruction(rng, register_count, constants_len, input_refs_len);
            }
            _ => {
                // Delete: remove a random instruction, keep at least 1.
                if vm.program.len() > 1 {
                    let idx = rng.gen_range(0..vm.program.len());
                    vm.program.remove(idx);
                } else {
                    // Single instruction — replace with random rather than emptying the program.
                    vm.program[0] =
                        random_vm_instruction(rng, register_count, constants_len, input_refs_len);
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
        vm.program.remove(idx);
    }
    Ok(())
}

pub(super) fn apply_instruction_raw_field_mutation(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
    config: &MutationConfig,
) -> Result<(), MutationSkipReason> {
    let node = &mut genome.nodes[node_idx];
    let BackendDef::Vm(ref mut vm) = node.backend_def else {
        return Ok(());
    };
    if vm.program.is_empty() {
        vm.program.push(VmInstruction::PushAction {
            action_type: rng.gen(),
        });
        return Ok(());
    }
    let idx = rng.gen_range(0..vm.program.len());
    mutate_instruction_raw_fields(&mut vm.program[idx], rng, &node.input_refs, config);
    Ok(())
}

/// Remap all register-typed fields: `(reg + offset) % register_count`.
fn remap_register_refs(instr: &mut VmInstruction, offset: u8, register_count: u8) {
    let rc = register_count.max(1);
    let remap = |reg: &mut u8| {
        *reg = (*reg).wrapping_add(offset) % rc;
    };
    match instr {
        VmInstruction::Noop | VmInstruction::Halt => {}
        VmInstruction::LoadConst { dst, .. } => remap(dst),
        VmInstruction::Move { dst, src }
        | VmInstruction::Abs { dst, src }
        | VmInstruction::Neg { dst, src }
        | VmInstruction::Clamp01 { dst, src }
        | VmInstruction::Not { dst, src }
        | VmInstruction::ToI32 { dst, src }
        | VmInstruction::ToU8 { dst, src }
        | VmInstruction::ToBool { dst, src } => {
            remap(dst);
            remap(src);
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
            remap(dst);
            remap(a);
            remap(b);
        }
        VmInstruction::CmpEq { dst, a, b, eps } => {
            remap(dst);
            remap(a);
            remap(b);
            remap(eps);
        }
        VmInstruction::JumpIfZero { cond, .. } => remap(cond),
        VmInstruction::Jump { .. } => {}
        VmInstruction::ReadInput { dst, .. } => remap(dst),
        VmInstruction::WriteInternalPayload { src, .. }
        | VmInstruction::WriteWorldActionMeta { src, .. }
        | VmInstruction::WriteRouteGate { src, .. } => remap(src),
        VmInstruction::PushAction { .. }
        | VmInstruction::PopAction
        | VmInstruction::ExecuteActionQueue => {}
        VmInstruction::SetPriorityBid { src } => remap(src),
        VmInstruction::ReadActionQueueLength { dst } => remap(dst),
        VmInstruction::ReadActionQueueType { index_src, dst } => {
            remap(index_src);
            remap(dst);
        }
        VmInstruction::ReadActionQueueParam { index_src, dst, .. } => {
            remap(index_src);
            remap(dst);
        }
        VmInstruction::LoadSlot { dst, slot_reg } => {
            remap(dst);
            remap(slot_reg);
        }
        VmInstruction::StoreSlot { slot_reg, src } => {
            remap(slot_reg);
            remap(src);
        }
        VmInstruction::LoadSlotImm { dst, .. } => remap(dst),
        VmInstruction::StoreSlotImm { src, .. } => remap(src),
        VmInstruction::LoadSlotPrev { dst, .. } => remap(dst),
        VmInstruction::ClearSlot { .. } => {}
    }
}

/// Adjust jump offsets for Jump and JumpIfZero instructions.
fn adjust_jump_offset(instr: &mut VmInstruction, delta: i32) {
    match instr {
        VmInstruction::Jump { offset } | VmInstruction::JumpIfZero { offset, .. } => {
            *offset = offset.wrapping_add(delta);
        }
        _ => {}
    }
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
        let block: Vec<VmInstruction> =
            vm.program[source_start..source_start + block_size].to_vec();
        let insert_at = rng.gen_range(0..=vm.program.len());
        vm.program.splice(insert_at..insert_at, block);
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
        let mut block: Vec<VmInstruction> =
            vm.program[source_start..source_start + block_size].to_vec();
        let reg_offset = rng.gen_range(1..register_count.max(2));
        let insert_at = rng.gen_range(0..=vm.program.len());
        let delta = insert_at as i32 - source_start as i32;
        for instr in &mut block {
            remap_register_refs(instr, reg_offset, register_count);
            adjust_jump_offset(instr, delta);
        }
        vm.program.splice(insert_at..insert_at, block);
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
            let extracted: Vec<VmInstruction> = gene
                .indices
                .iter()
                .map(|&i| vm.program[i].clone())
                .collect();
            let insert_at = rng.gen_range(0..=vm.program.len());
            vm.program.splice(insert_at..insert_at, extracted);
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
            let extracted: Vec<VmInstruction> = gene
                .indices
                .iter()
                .map(|&i| vm.program[i].clone())
                .collect();
            let insert_at = rng.gen_range(0..=vm.program.len());
            vm.program.splice(insert_at..insert_at, extracted);
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
        vm.program.splice(pos..pos, pair);
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

        vm.program.splice(pos..pos, pair);
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
        vm.program.splice(pos..pos, pair);
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
        let mut groups: std::collections::HashMap<u8, (Vec<usize>, bool, bool)> =
            std::collections::HashMap::new();
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
            *slot_idx = rng.gen_range(0u8..16);
        }
        VmInstruction::LoadSlot { slot_reg, .. } | VmInstruction::StoreSlot { slot_reg, .. } => {
            // For register-indirect slot access, mutate the slot_reg.
            *slot_reg = rng.gen();
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
