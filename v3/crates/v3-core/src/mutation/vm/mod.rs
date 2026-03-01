use rand::Rng;

use crate::creature::genome::analysis::{vm_backward_slice_random, vm_forward_slice_random};
use crate::creature::genome::{BackendDef, CreatureGenome, VmInstruction};
use crate::mutation::types::MutationSkipReason;

/// VM mutation operator variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VmOperator {
    VmConstantMutation,
    VmInstructionMutation,
    VmRegisterCountMutation,
    VmInstructionRawFieldMutation,
    VmCopyInstructionBlock,
    VmCopyInstructionBlockRemapped,
    VmCopyConstantBlock,
    VmCopyGeneBackwardSlice,
    VmCopyGeneForwardSlice,
}

impl VmOperator {
    pub const ALL: [Self; 9] = [
        Self::VmConstantMutation,
        Self::VmInstructionMutation,
        Self::VmRegisterCountMutation,
        Self::VmInstructionRawFieldMutation,
        Self::VmCopyInstructionBlock,
        Self::VmCopyInstructionBlockRemapped,
        Self::VmCopyConstantBlock,
        Self::VmCopyGeneBackwardSlice,
        Self::VmCopyGeneForwardSlice,
    ];

    /// Per-operator weight reflecting impact tier.
    /// 4 = refinement, 2 = moderate, 1 = structural.
    #[must_use]
    pub const fn weight(self) -> u8 {
        match self {
            Self::VmConstantMutation => 4,
            Self::VmInstructionMutation => 2,
            Self::VmRegisterCountMutation => 2,
            Self::VmInstructionRawFieldMutation => 4,
            Self::VmCopyInstructionBlock => 1,
            Self::VmCopyInstructionBlockRemapped => 1,
            Self::VmCopyConstantBlock => 1,
            Self::VmCopyGeneBackwardSlice => 1,
            Self::VmCopyGeneForwardSlice => 1,
        }
    }

    const TOTAL_WEIGHT: u16 = {
        assert!(
            Self::ALL.len() == 9,
            "ALL must cover every VmOperator variant"
        );
        let mut sum = 0u16;
        let mut i = 0;
        while i < Self::ALL.len() {
            sum += Self::ALL[i].weight() as u16;
            i += 1;
        }
        sum
    };

    /// Whether this operator increases, decreases, or preserves genome complexity.
    #[must_use]
    pub const fn complexity_effect(self) -> crate::mutation::types::ComplexityEffect {
        use crate::mutation::types::ComplexityEffect;
        match self {
            Self::VmCopyInstructionBlock
            | Self::VmCopyInstructionBlockRemapped
            | Self::VmCopyConstantBlock
            | Self::VmCopyGeneBackwardSlice
            | Self::VmCopyGeneForwardSlice => ComplexityEffect::Increasing,
            Self::VmConstantMutation
            | Self::VmInstructionMutation
            | Self::VmRegisterCountMutation
            | Self::VmInstructionRawFieldMutation => ComplexityEffect::Neutral,
        }
    }

    const NON_INCREASING_WEIGHT: u16 = {
        let mut sum = 0u16;
        let mut i = 0;
        while i < Self::ALL.len() {
            if !Self::ALL[i].complexity_effect().is_increasing() {
                sum += Self::ALL[i].weight() as u16;
            }
            i += 1;
        }
        sum
    };

    /// Pick a random non-increasing operator (Neutral or Decreasing) weighted by impact tier.
    pub fn random_non_increasing(rng: &mut impl Rng) -> Option<Self> {
        if Self::NON_INCREASING_WEIGHT == 0 {
            return None;
        }
        let mut r = rng.gen_range(0..Self::NON_INCREASING_WEIGHT);
        for &op in &Self::ALL {
            if op.complexity_effect().is_increasing() {
                continue;
            }
            let w = op.weight() as u16;
            if r < w {
                return Some(op);
            }
            r -= w;
        }
        unreachable!()
    }

    /// Pick a random VM operator weighted by impact tier.
    pub fn random(rng: &mut impl Rng) -> Self {
        let mut r = rng.gen_range(0..Self::TOTAL_WEIGHT);
        for &op in &Self::ALL {
            let w = op.weight() as u16;
            if r < w {
                return op;
            }
            r -= w;
        }
        unreachable!()
    }
}

/// VM domain mutator.
pub struct VmMutator;

impl VmMutator {
    /// Apply a VM operator to the genome.
    ///
    /// Returns `Ok(())` on success, or `Err(MutationSkipReason::NoApplicableTarget)` if the
    /// genome contains no VM-backend nodes.
    pub fn apply(
        genome: &mut CreatureGenome,
        op: VmOperator,
        rng: &mut impl Rng,
    ) -> Result<(), MutationSkipReason> {
        // Pre-guard: must have at least one VM-backend node.
        let vm_indices: Vec<usize> = genome
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| matches!(n.backend_def, BackendDef::Vm(_)))
            .map(|(i, _)| i)
            .collect();
        if vm_indices.is_empty() {
            return Err(MutationSkipReason::NoApplicableTarget);
        }

        let node_idx = vm_indices[rng.gen_range(0..vm_indices.len())];
        match op {
            VmOperator::VmConstantMutation => apply_constant_mutation(genome, node_idx, rng),
            VmOperator::VmInstructionMutation => apply_instruction_mutation(genome, node_idx, rng),
            VmOperator::VmRegisterCountMutation => {
                apply_register_count_mutation(genome, node_idx, rng)
            }
            VmOperator::VmInstructionRawFieldMutation => {
                apply_instruction_raw_field_mutation(genome, node_idx, rng)
            }
            VmOperator::VmCopyInstructionBlock => {
                apply_copy_instruction_block(genome, node_idx, rng)
            }
            VmOperator::VmCopyInstructionBlockRemapped => {
                apply_copy_instruction_block_remapped(genome, node_idx, rng)
            }
            VmOperator::VmCopyConstantBlock => apply_copy_constant_block(genome, node_idx, rng),
            VmOperator::VmCopyGeneBackwardSlice => {
                apply_copy_gene_backward_slice(genome, node_idx, rng)
            }
            VmOperator::VmCopyGeneForwardSlice => {
                apply_copy_gene_forward_slice(genome, node_idx, rng)
            }
        }
    }
}

fn apply_constant_mutation(
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

fn apply_register_count_mutation(
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
fn random_vm_instruction(
    rng: &mut impl Rng,
    register_count: u8,
    constants_len: usize,
    input_refs_len: usize,
) -> VmInstruction {
    // Inline helpers to avoid closure borrow conflicts on `rng`.
    let rc = register_count.max(1);
    let cl = constants_len.clamp(1, 255) as u8;
    let il = input_refs_len.clamp(1, 255) as u8;

    match rng.gen_range(0u8..39) {
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
        26 => VmInstruction::WriteRouteTarget {
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
        34 => VmInstruction::LoadMem8 {
            dst: rng.gen_range(0..rc),
            addr_reg: rng.gen_range(0..rc),
        },
        35 => VmInstruction::StoreMem8 {
            addr_reg: rng.gen_range(0..rc),
            src: rng.gen_range(0..rc),
        },
        36 => VmInstruction::LoadMem8Imm {
            dst: rng.gen_range(0..rc),
            imm_addr: rng.gen(),
        },
        37 => VmInstruction::StoreMem8Imm {
            imm_addr: rng.gen(),
            src: rng.gen_range(0..rc),
        },
        38 => VmInstruction::SetPriorityBid {
            src: rng.gen_range(0..rc),
        },
        _ => unreachable!("gen_range(0..39) cannot produce values >= 39"),
    }
}

fn mutate_instruction_raw_fields(instr: &mut VmInstruction, rng: &mut impl Rng) {
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
            *ref_idx = rng.gen();
            *sub_idx = rng.gen();
        }
        VmInstruction::WriteInternalPayload { slot_idx, src }
        | VmInstruction::WriteWorldActionMeta { slot_idx, src } => {
            *slot_idx = rng.gen();
            *src = rng.gen();
        }
        VmInstruction::WriteRouteTarget { src } => {
            *src = rng.gen();
        }
        VmInstruction::LoadMem8 { dst, addr_reg } => {
            *dst = rng.gen();
            *addr_reg = rng.gen();
        }
        VmInstruction::StoreMem8 { addr_reg, src } => {
            *addr_reg = rng.gen();
            *src = rng.gen();
        }
        VmInstruction::LoadMem8Imm { dst, imm_addr } => {
            *dst = rng.gen();
            *imm_addr = rng.gen();
        }
        VmInstruction::StoreMem8Imm { imm_addr, src } => {
            *imm_addr = rng.gen();
            *src = rng.gen();
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

fn apply_instruction_mutation(
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

fn apply_instruction_raw_field_mutation(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let node = &mut genome.nodes[node_idx];
    if let BackendDef::Vm(ref mut vm) = node.backend_def {
        if vm.program.is_empty() {
            vm.program.push(VmInstruction::PushAction {
                action_type: rng.gen(),
            });
            return Ok(());
        }
        let idx = rng.gen_range(0..vm.program.len());
        mutate_instruction_raw_fields(&mut vm.program[idx], rng);
    }
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
        | VmInstruction::WriteRouteTarget { src } => remap(src),
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
        VmInstruction::LoadMem8 { dst, addr_reg } => {
            remap(dst);
            remap(addr_reg);
        }
        VmInstruction::StoreMem8 { addr_reg, src } => {
            remap(addr_reg);
            remap(src);
        }
        VmInstruction::LoadMem8Imm { dst, .. } => remap(dst),
        VmInstruction::StoreMem8Imm { src, .. } => remap(src),
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

fn apply_copy_instruction_block(
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

fn apply_copy_instruction_block_remapped(
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

fn apply_copy_constant_block(
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

fn apply_copy_gene_backward_slice(
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

fn apply_copy_gene_forward_slice(
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

#[cfg(test)]
mod tests;
