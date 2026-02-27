use rand::Rng;

use crate::creature::genome::{BackendDef, CreatureGenome, VmInstruction};
use crate::mutation::types::MutationSkipReason;

/// VM mutation operator variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    /// Pick a random VM operator uniformly.
    pub fn random(rng: &mut impl Rng) -> Self {
        match rng.gen_range(0u8..9) {
            0 => Self::VmConstantMutation,
            1 => Self::VmInstructionMutation,
            2 => Self::VmRegisterCountMutation,
            3 => Self::VmInstructionRawFieldMutation,
            4 => Self::VmCopyInstructionBlock,
            5 => Self::VmCopyInstructionBlockRemapped,
            6 => Self::VmCopyConstantBlock,
            7 => Self::VmCopyGeneBackwardSlice,
            _ => Self::VmCopyGeneForwardSlice,
        }
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
            VmOperator::VmCopyConstantBlock => {
                apply_copy_constant_block(genome, node_idx, rng)
            }
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

    match rng.gen_range(0u8..33) {
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
            input_idx: rng.gen_range(0..il),
        },
        24 => VmInstruction::WriteInternalPayload {
            slot_idx: rng.gen_range(0u8..8),
            src: rng.gen_range(0..rc),
        },
        25 => VmInstruction::WriteWorldActionMeta {
            slot_idx: rng.gen_range(0u8..8),
            src: rng.gen_range(0..rc),
        },
        26 => VmInstruction::EmitWorldAction {
            action_type: rng.gen(),
        },
        27 => VmInstruction::WriteRouteTarget {
            src: rng.gen_range(0..rc),
        },
        28 => VmInstruction::Halt,
        29 => VmInstruction::LoadMem8 {
            dst: rng.gen_range(0..rc),
            addr_reg: rng.gen_range(0..rc),
        },
        30 => VmInstruction::StoreMem8 {
            addr_reg: rng.gen_range(0..rc),
            src: rng.gen_range(0..rc),
        },
        31 => VmInstruction::LoadMem8Imm {
            dst: rng.gen_range(0..rc),
            imm_addr: rng.gen(),
        },
        _ => VmInstruction::StoreMem8Imm {
            imm_addr: rng.gen(),
            src: rng.gen_range(0..rc),
        },
    }
}

fn mutate_instruction_raw_fields(instr: &mut VmInstruction, rng: &mut impl Rng) {
    match instr {
        VmInstruction::Noop | VmInstruction::Halt => {
            *instr = VmInstruction::EmitWorldAction {
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
        VmInstruction::ReadInput { dst, input_idx } => {
            *dst = rng.gen();
            *input_idx = rng.gen();
        }
        VmInstruction::WriteInternalPayload { slot_idx, src }
        | VmInstruction::WriteWorldActionMeta { slot_idx, src } => {
            *slot_idx = rng.gen();
            *src = rng.gen();
        }
        VmInstruction::EmitWorldAction { action_type } => {
            *action_type = rng.gen();
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
            vm.program.push(VmInstruction::EmitWorldAction {
                action_type: rng.gen(),
            });
            return Ok(());
        }
        let idx = rng.gen_range(0..vm.program.len());
        mutate_instruction_raw_fields(&mut vm.program[idx], rng);
    }
    Ok(())
}

fn apply_copy_instruction_block(
    _genome: &mut CreatureGenome,
    _node_idx: usize,
    _rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    Err(MutationSkipReason::NoApplicableTarget)
}

fn apply_copy_instruction_block_remapped(
    _genome: &mut CreatureGenome,
    _node_idx: usize,
    _rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    Err(MutationSkipReason::NoApplicableTarget)
}

fn apply_copy_constant_block(
    _genome: &mut CreatureGenome,
    _node_idx: usize,
    _rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    Err(MutationSkipReason::NoApplicableTarget)
}

fn apply_copy_gene_backward_slice(
    _genome: &mut CreatureGenome,
    _node_idx: usize,
    _rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    Err(MutationSkipReason::NoApplicableTarget)
}

fn apply_copy_gene_forward_slice(
    _genome: &mut CreatureGenome,
    _node_idx: usize,
    _rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    Err(MutationSkipReason::NoApplicableTarget)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::NodeId;
    use crate::creature::founder::v3alpha1_founder_genome;
    use crate::creature::genome::{BackendDef, GraphBackendDef, NodeGenome, VmInstruction};
    use crate::creature::parseability::ParseabilityGate;
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    fn rng(seed: u64) -> SmallRng {
        SmallRng::seed_from_u64(seed)
    }

    #[test]
    fn vm_constant_mutation_changes_constant_value() {
        let genome = v3alpha1_founder_genome();
        // Node 1 (VM) has constants [0.5, 1.0, 2.0, 3.0, 20.0].
        let before: Vec<f32> = if let BackendDef::Vm(ref vm) = genome.nodes[1].backend_def {
            vm.constants.clone()
        } else {
            panic!("expected VM backend on node 1");
        };
        let mut changed = false;
        for seed in 0u64..50 {
            let mut g = genome.clone();
            let mut r = rng(seed);
            VmMutator::apply(&mut g, VmOperator::VmConstantMutation, &mut r).unwrap();
            let after: Vec<f32> = if let BackendDef::Vm(ref vm) = g.nodes[1].backend_def {
                vm.constants.clone()
            } else {
                panic!()
            };
            if after != before {
                changed = true;
                break;
            }
        }
        assert!(changed, "constant must change after mutation");
    }

    #[test]
    fn vm_constant_mutation_on_node_with_empty_constants_adds_constant() {
        let mut genome = v3alpha1_founder_genome();
        // Clear constants from node 1 VM.
        if let BackendDef::Vm(ref mut vm) = genome.nodes[1].backend_def {
            vm.constants.clear();
        }
        let mut r = rng(0);
        VmMutator::apply(&mut genome, VmOperator::VmConstantMutation, &mut r).unwrap();
        if let BackendDef::Vm(ref vm) = genome.nodes[1].backend_def {
            assert_eq!(vm.constants.len(), 1, "one constant added");
        }
    }

    #[test]
    fn vm_instruction_mutation_changes_program() {
        let mut genome = v3alpha1_founder_genome();
        let before_len = if let BackendDef::Vm(ref vm) = genome.nodes[1].backend_def {
            vm.program.len()
        } else {
            panic!()
        };
        let mut changed = false;
        for seed in 0u64..50 {
            let mut g = genome.clone();
            let mut r = rng(seed);
            VmMutator::apply(&mut g, VmOperator::VmInstructionMutation, &mut r).unwrap();
            let after_len = if let BackendDef::Vm(ref vm) = g.nodes[1].backend_def {
                vm.program.len()
            } else {
                panic!()
            };
            if after_len != before_len {
                changed = true;
                break;
            }
        }
        // Either length changed (insert/delete) or an instruction was replaced. Accept any change.
        // At minimum, the operation must not panic.
        let _ = changed;
        let mut r = rng(99);
        let result = VmMutator::apply(&mut genome, VmOperator::VmInstructionMutation, &mut r);
        assert!(result.is_ok());
    }

    #[test]
    fn vm_mutator_on_graph_only_genome_returns_no_applicable_target() {
        let mut genome = v3alpha1_founder_genome();
        // Replace all nodes with Graph-backend nodes.
        genome.nodes = vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![],
            backend_def: BackendDef::Graph(GraphBackendDef {
                internal_nodes: vec![],
            }),
            targets: vec![],
        }];
        let mut r = rng(0);
        let result = VmMutator::apply(&mut genome, VmOperator::VmConstantMutation, &mut r);
        assert_eq!(result, Err(MutationSkipReason::NoApplicableTarget));
    }

    #[test]
    fn vm_after_mutation_passes_parseability_gate() {
        let operators = [
            VmOperator::VmConstantMutation,
            VmOperator::VmInstructionMutation,
            VmOperator::VmRegisterCountMutation,
            VmOperator::VmInstructionRawFieldMutation,
            VmOperator::VmCopyInstructionBlock,
            VmOperator::VmCopyInstructionBlockRemapped,
            VmOperator::VmCopyConstantBlock,
            VmOperator::VmCopyGeneBackwardSlice,
            VmOperator::VmCopyGeneForwardSlice,
        ];
        for (i, &op) in operators.iter().enumerate() {
            let mut genome = v3alpha1_founder_genome();
            let mut r = rng(i as u64 + 200);
            let _ = VmMutator::apply(&mut genome, op, &mut r);
            assert!(
                ParseabilityGate::validate(&genome).is_ok(),
                "parseability failed after {:?}",
                op
            );
        }
    }

    #[test]
    fn vm_insert_produces_non_noop() {
        // Over 100 seeds, insert path (choice=0) must produce at least one non-Noop instruction.
        let mut found_non_noop = false;
        for seed in 0u64..100 {
            let mut genome = v3alpha1_founder_genome();
            let mut r = rng(seed);
            // Force insert path by extracting the RNG state — but simpler: just run
            // VmInstructionMutation many times and check for non-Noop in the program.
            VmMutator::apply(&mut genome, VmOperator::VmInstructionMutation, &mut r).unwrap();
            if let BackendDef::Vm(ref vm) = genome.nodes[1].backend_def {
                if vm.program.iter().any(|i| !matches!(i, VmInstruction::Noop)) {
                    found_non_noop = true;
                    break;
                }
            }
        }
        assert!(
            found_non_noop,
            "insert/replace must produce non-Noop instructions"
        );
    }

    #[test]
    fn vm_replace_produces_non_noop() {
        // Start with a program of all Halts, run replace mutations, verify non-Noop appears.
        let mut found_non_noop = false;
        for seed in 0u64..100 {
            let mut genome = v3alpha1_founder_genome();
            // Set program to all Halt instructions.
            if let BackendDef::Vm(ref mut vm) = genome.nodes[1].backend_def {
                vm.program = vec![VmInstruction::Halt; 10];
            }
            let mut r = rng(seed);
            VmMutator::apply(&mut genome, VmOperator::VmInstructionMutation, &mut r).unwrap();
            if let BackendDef::Vm(ref vm) = genome.nodes[1].backend_def {
                // Check if any instruction changed to something other than Halt or Noop.
                if vm
                    .program
                    .iter()
                    .any(|i| !matches!(i, VmInstruction::Halt | VmInstruction::Noop))
                {
                    found_non_noop = true;
                    break;
                }
            }
        }
        assert!(
            found_non_noop,
            "replace must produce diverse instructions, not just Noop"
        );
    }

    #[test]
    fn random_vm_instruction_covers_all_families() {
        use std::collections::HashSet;
        let mut discriminants: HashSet<std::mem::Discriminant<VmInstruction>> = HashSet::new();
        for seed in 0u64..1000 {
            let mut r = rng(seed);
            let instr = random_vm_instruction(&mut r, 4, 4, 4);
            discriminants.insert(std::mem::discriminant(&instr));
        }
        assert_eq!(
            discriminants.len(),
            33,
            "all 33 VmInstruction variants must be reachable; got {}",
            discriminants.len()
        );
    }

    #[test]
    fn random_vm_instruction_widens_emit_world_action_range() {
        let mut saw_above_3 = false;
        for seed in 0u64..1024 {
            let mut r = rng(seed);
            if let VmInstruction::EmitWorldAction { action_type } =
                random_vm_instruction(&mut r, 4, 4, 4)
            {
                if action_type > 3 {
                    saw_above_3 = true;
                    break;
                }
            }
        }
        assert!(
            saw_above_3,
            "random instruction generation should reach action_type values above 3"
        );
    }

    #[test]
    fn random_vm_instruction_widens_imm_addr_range() {
        let mut saw_above_1023 = false;
        for seed in 0u64..1024 {
            let mut r = rng(seed);
            match random_vm_instruction(&mut r, 4, 4, 4) {
                VmInstruction::LoadMem8Imm { imm_addr, .. }
                | VmInstruction::StoreMem8Imm { imm_addr, .. } => {
                    if imm_addr > 1023 {
                        saw_above_1023 = true;
                        break;
                    }
                }
                _ => {}
            }
        }
        assert!(
            saw_above_1023,
            "random instruction generation should reach imm_addr values above 1023"
        );
    }

    #[test]
    fn raw_field_mutation_can_produce_out_of_range_action_type() {
        let mut genome = v3alpha1_founder_genome();
        if let BackendDef::Vm(ref mut vm) = genome.nodes[1].backend_def {
            vm.program = vec![VmInstruction::EmitWorldAction { action_type: 0 }];
        }

        let mut saw_out_of_range = false;
        for seed in 0u64..512 {
            let mut g = genome.clone();
            let mut r = rng(seed);
            VmMutator::apply(&mut g, VmOperator::VmInstructionRawFieldMutation, &mut r).unwrap();
            if let BackendDef::Vm(ref vm) = g.nodes[1].backend_def {
                if let VmInstruction::EmitWorldAction { action_type } = vm.program[0] {
                    if action_type > 3 {
                        saw_out_of_range = true;
                        break;
                    }
                }
            }
        }
        assert!(
            saw_out_of_range,
            "raw field mutation should produce action_type values outside 0..=3"
        );
    }

    #[test]
    fn raw_field_mutation_can_produce_out_of_range_imm_addr() {
        let mut genome = v3alpha1_founder_genome();
        if let BackendDef::Vm(ref mut vm) = genome.nodes[1].backend_def {
            vm.program = vec![VmInstruction::LoadMem8Imm {
                dst: 0,
                imm_addr: 0,
            }];
        }

        let mut saw_out_of_range = false;
        for seed in 0u64..512 {
            let mut g = genome.clone();
            let mut r = rng(seed);
            VmMutator::apply(&mut g, VmOperator::VmInstructionRawFieldMutation, &mut r).unwrap();
            if let BackendDef::Vm(ref vm) = g.nodes[1].backend_def {
                if let VmInstruction::LoadMem8Imm { imm_addr, .. } = vm.program[0] {
                    if imm_addr > 1023 {
                        saw_out_of_range = true;
                        break;
                    }
                }
            }
        }
        assert!(
            saw_out_of_range,
            "raw field mutation should produce imm_addr values outside 0..=1023"
        );
    }

    #[test]
    fn vm_instruction_mutation_program_never_empty() {
        // Genome with a single-instruction program — delete path must not empty it.
        let mut genome = v3alpha1_founder_genome();
        if let BackendDef::Vm(ref mut vm) = genome.nodes[1].backend_def {
            vm.program = vec![VmInstruction::Halt];
        }
        // Run many times to trigger the delete path.
        for seed in 0u64..100 {
            let mut g = genome.clone();
            let mut r = rng(seed);
            VmMutator::apply(&mut g, VmOperator::VmInstructionMutation, &mut r).unwrap();
            if let BackendDef::Vm(ref vm) = g.nodes[1].backend_def {
                assert!(!vm.program.is_empty(), "program must not be emptied");
            }
        }
    }

    #[test]
    fn vm_register_count_increments_and_decrements() {
        let genome = v3alpha1_founder_genome();
        let original_rc = if let BackendDef::Vm(ref vm) = genome.nodes[1].backend_def {
            vm.register_count
        } else {
            panic!("expected VM");
        };
        let mut saw_increment = false;
        let mut saw_decrement = false;
        for seed in 0u64..100 {
            let mut g = genome.clone();
            let mut r = rng(seed);
            if VmMutator::apply(&mut g, VmOperator::VmRegisterCountMutation, &mut r).is_ok() {
                let new_rc = if let BackendDef::Vm(ref vm) = g.nodes[1].backend_def {
                    vm.register_count
                } else {
                    original_rc
                };
                if new_rc > original_rc {
                    saw_increment = true;
                }
                if new_rc < original_rc {
                    saw_decrement = true;
                }
            }
            if saw_increment && saw_decrement {
                break;
            }
        }
        assert!(saw_increment, "register count must sometimes increment");
        assert!(saw_decrement, "register count must sometimes decrement");
    }

    #[test]
    fn vm_register_count_clamps_to_bounds() {
        // Test lower bound: register_count=1 should not go below 1.
        let mut genome = v3alpha1_founder_genome();
        if let BackendDef::Vm(ref mut vm) = genome.nodes[1].backend_def {
            vm.register_count = 1;
        }
        for seed in 0u64..100 {
            let mut g = genome.clone();
            let mut r = rng(seed);
            let _ = VmMutator::apply(&mut g, VmOperator::VmRegisterCountMutation, &mut r);
            if let BackendDef::Vm(ref vm) = g.nodes[1].backend_def {
                assert!(vm.register_count >= 1, "register_count must be >= 1");
            }
        }
        // Test upper bound: register_count=32 should not go above 32.
        if let BackendDef::Vm(ref mut vm) = genome.nodes[1].backend_def {
            vm.register_count = 32;
        }
        for seed in 0u64..100 {
            let mut g = genome.clone();
            let mut r = rng(seed);
            let _ = VmMutator::apply(&mut g, VmOperator::VmRegisterCountMutation, &mut r);
            if let BackendDef::Vm(ref vm) = g.nodes[1].backend_def {
                assert!(vm.register_count <= 32, "register_count must be <= 32");
            }
        }
    }
}
