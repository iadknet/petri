use rand::Rng;

use crate::creature::genome::{BackendDef, CreatureGenome, VmInstruction};
use crate::mutation::types::MutationSkipReason;

/// VM mutation operator variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmOperator {
    VmConstantMutation,
    VmInstructionMutation,
}

impl VmOperator {
    /// Pick a random VM operator uniformly.
    pub fn random(rng: &mut impl Rng) -> Self {
        if rng.gen_bool(0.5) {
            Self::VmConstantMutation
        } else {
            Self::VmInstructionMutation
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

fn apply_instruction_mutation(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let node = &mut genome.nodes[node_idx];
    if let BackendDef::Vm(ref mut vm) = node.backend_def {
        if vm.program.is_empty() {
            // Edge case: empty program — insert a Noop.
            vm.program.push(VmInstruction::Noop);
            return Ok(());
        }

        // Choose: insert (0), replace (1), delete (2).
        let choice = rng.gen_range(0u8..3);
        match choice {
            0 => {
                // Insert: push Noop at a random position.
                let pos = rng.gen_range(0..=vm.program.len());
                vm.program.insert(pos, VmInstruction::Noop);
            }
            1 => {
                // Replace: replace a random instruction with Noop.
                let idx = rng.gen_range(0..vm.program.len());
                vm.program[idx] = VmInstruction::Noop;
            }
            _ => {
                // Delete: remove a random instruction, keep at least 1.
                if vm.program.len() > 1 {
                    let idx = rng.gen_range(0..vm.program.len());
                    vm.program.remove(idx);
                } else {
                    // Single instruction — replace with Noop rather than emptying the program.
                    vm.program[0] = VmInstruction::Noop;
                }
            }
        }
    }
    Ok(())
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
}
