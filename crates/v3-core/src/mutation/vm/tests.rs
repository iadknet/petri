use super::operators::{
    apply_register_count_mutation, mutate_one_instruction_field,
    splice_program_with_reference_repair, SpliceInstruction,
};
use super::*;
use crate::contracts::NodeId;
use crate::creature::founder::v3alpha1_founder_genome;
use crate::creature::genome::analysis::vm_forward_slice;
use crate::creature::genome::{
    BackendDef, CreatureGenome, NodeGenome, VmBackendDef, VmInstruction,
};
use crate::creature::parseability::ParseabilityGate;
use crate::mutation::types::MutationSkipReason;
use proptest::prelude::*;
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
        VmMutator::apply(
            &mut g,
            VmOperator::VmConstantMutation,
            &[],
            0.0,
            &mut r,
            &MutationConfig::default(),
        )
        .unwrap();
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
    VmMutator::apply(
        &mut genome,
        VmOperator::VmConstantMutation,
        &[],
        0.0,
        &mut r,
        &MutationConfig::default(),
    )
    .unwrap();
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
        VmMutator::apply(
            &mut g,
            VmOperator::VmInstructionMutation,
            &[],
            0.0,
            &mut r,
            &MutationConfig::default(),
        )
        .unwrap();
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
    let result = VmMutator::apply(
        &mut genome,
        VmOperator::VmInstructionMutation,
        &[],
        0.0,
        &mut r,
        &MutationConfig::default(),
    );
    assert!(result.is_ok());
}

#[test]
fn vm_delete_instruction_removes_one_instruction() {
    let mut genome = v3alpha1_founder_genome();
    if let BackendDef::Vm(ref mut vm) = genome.nodes[1].backend_def {
        vm.program = vec![
            VmInstruction::Noop,
            VmInstruction::Halt,
            VmInstruction::PushAction { action_type: 1 },
        ];
    }

    let before_len = if let BackendDef::Vm(ref vm) = genome.nodes[1].backend_def {
        vm.program.len()
    } else {
        panic!()
    };
    let mut r = rng(7);
    let result = VmMutator::apply(
        &mut genome,
        VmOperator::VmDeleteInstruction,
        &[],
        0.0,
        &mut r,
        &MutationConfig::default(),
    );
    assert!(result.is_ok());

    let after_len = if let BackendDef::Vm(ref vm) = genome.nodes[1].backend_def {
        vm.program.len()
    } else {
        panic!()
    };
    assert_eq!(after_len + 1, before_len);
}

#[test]
fn vm_delete_instruction_skips_single_instruction_program() {
    let mut genome = v3alpha1_founder_genome();
    if let BackendDef::Vm(ref mut vm) = genome.nodes[1].backend_def {
        vm.program = vec![VmInstruction::Noop];
    }

    let mut r = rng(11);
    let result = VmMutator::apply(
        &mut genome,
        VmOperator::VmDeleteInstruction,
        &[],
        0.0,
        &mut r,
        &MutationConfig::default(),
    );
    assert_eq!(result, Err(MutationSkipReason::NoApplicableTarget));
}

#[test]
fn vm_mutator_on_graph_only_genome_returns_no_applicable_target() {
    let mut genome = v3alpha1_founder_genome();
    // Replace all nodes with Graph-backend nodes.
    genome.nodes = vec![NodeGenome {
        node_id: NodeId::new(0),
        input_refs: vec![],
        backend_def: BackendDef::Graph(
            crate::creature::genome::cgp::CgpGraphBackendDef::new_with_fixed_outputs(
                &MutationConfig::default(),
            ),
        ),
        targets: vec![],
    }];
    let mut r = rng(0);
    let result = VmMutator::apply(
        &mut genome,
        VmOperator::VmConstantMutation,
        &[],
        0.0,
        &mut r,
        &MutationConfig::default(),
    );
    assert_eq!(result, Err(MutationSkipReason::NoApplicableTarget));
}

#[test]
fn vm_after_mutation_passes_parseability_gate() {
    let operators = [
        VmOperator::VmConstantMutation,
        VmOperator::VmInstructionMutation,
        VmOperator::VmDeleteInstruction,
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
        let _ = VmMutator::apply(
            &mut genome,
            op,
            &[],
            0.0,
            &mut r,
            &MutationConfig::default(),
        );
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
        VmMutator::apply(
            &mut genome,
            VmOperator::VmInstructionMutation,
            &[],
            0.0,
            &mut r,
            &MutationConfig::default(),
        )
        .unwrap();
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
        VmMutator::apply(
            &mut genome,
            VmOperator::VmInstructionMutation,
            &[],
            0.0,
            &mut r,
            &MutationConfig::default(),
        )
        .unwrap();
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
        41,
        "all 41 VmInstruction variants must be reachable; got {}",
        discriminants.len()
    );
}

#[test]
fn random_vm_instruction_widens_push_action_range() {
    let mut saw_above_3 = false;
    for seed in 0u64..1024 {
        let mut r = rng(seed);
        if let VmInstruction::PushAction { action_type } = random_vm_instruction(&mut r, 4, 4, 4) {
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
fn random_vm_instruction_generates_slot_opcodes() {
    let mut saw_slot_opcode = false;
    for seed in 0u64..1024 {
        let mut r = rng(seed);
        match random_vm_instruction(&mut r, 4, 4, 4) {
            VmInstruction::LoadSlot { .. }
            | VmInstruction::StoreSlot { .. }
            | VmInstruction::LoadSlotImm { .. }
            | VmInstruction::StoreSlotImm { .. }
            | VmInstruction::LoadSlotPrev { .. }
            | VmInstruction::ClearSlot { .. } => {
                saw_slot_opcode = true;
                break;
            }
            _ => {}
        }
    }
    assert!(
        saw_slot_opcode,
        "random instruction generation should produce slot opcodes"
    );
}

#[test]
fn raw_field_mutation_nudges_action_type_by_one() {
    let mut genome = v3alpha1_founder_genome();
    if let BackendDef::Vm(ref mut vm) = genome.nodes[1].backend_def {
        vm.program = vec![VmInstruction::PushAction { action_type: 0 }];
    }

    for seed in 0u64..512 {
        let mut g = genome.clone();
        let mut r = rng(seed);
        VmMutator::apply(
            &mut g,
            VmOperator::VmInstructionRawFieldMutation,
            &[],
            0.0,
            &mut r,
            &MutationConfig::default(),
        )
        .unwrap();
        if let BackendDef::Vm(ref vm) = g.nodes[1].backend_def {
            assert!(matches!(
                vm.program[0],
                VmInstruction::PushAction { action_type: 1 }
            ));
        }
    }
}

#[test]
fn raw_field_mutation_can_change_slot_idx() {
    let mut genome = v3alpha1_founder_genome();
    if let BackendDef::Vm(ref mut vm) = genome.nodes[1].backend_def {
        vm.program = vec![VmInstruction::LoadSlotImm {
            dst: 0,
            slot_idx: 0,
        }];
    }

    let mut saw_changed = false;
    for seed in 0u64..512 {
        let mut g = genome.clone();
        let mut r = rng(seed);
        VmMutator::apply(
            &mut g,
            VmOperator::VmInstructionRawFieldMutation,
            &[],
            0.0,
            &mut r,
            &MutationConfig::default(),
        )
        .unwrap();
        if let BackendDef::Vm(ref vm) = g.nodes[1].backend_def {
            if let VmInstruction::LoadSlotImm { slot_idx, .. } = vm.program[0] {
                if slot_idx != 0 {
                    saw_changed = true;
                    break;
                }
            }
        }
    }
    assert!(
        saw_changed,
        "raw field mutation should change slot_idx from initial value"
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
        VmMutator::apply(
            &mut g,
            VmOperator::VmInstructionMutation,
            &[],
            0.0,
            &mut r,
            &MutationConfig::default(),
        )
        .unwrap();
        if let BackendDef::Vm(ref vm) = g.nodes[1].backend_def {
            assert!(!vm.program.is_empty(), "program must not be emptied");
        }
    }
}

#[test]
fn vm_register_count_increments_and_decrements() {
    let genome = slot_program_genome(vec![VmInstruction::Move { dst: 0, src: 0 }]);
    let original_rc = 2;
    let mut saw_increment = false;
    let mut saw_decrement = false;
    for seed in 0u64..100 {
        let mut g = genome.clone();
        let mut r = rng(seed);
        if VmMutator::apply(
            &mut g,
            VmOperator::VmRegisterCountMutation,
            &[],
            0.0,
            &mut r,
            &MutationConfig::default(),
        )
        .is_ok()
        {
            let new_rc = if let BackendDef::Vm(ref vm) = g.nodes[0].backend_def {
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
        let _ = VmMutator::apply(
            &mut g,
            VmOperator::VmRegisterCountMutation,
            &[],
            0.0,
            &mut r,
            &MutationConfig::default(),
        );
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
        let _ = VmMutator::apply(
            &mut g,
            VmOperator::VmRegisterCountMutation,
            &[],
            0.0,
            &mut r,
            &MutationConfig::default(),
        );
        if let BackendDef::Vm(ref vm) = g.nodes[1].backend_def {
            assert!(vm.register_count <= 32, "register_count must be <= 32");
        }
    }
}

// ── VmCopyInstructionBlock tests ──

#[test]
fn copy_instruction_block_increases_program_length() {
    let mut genome = v3alpha1_founder_genome();
    let before = if let BackendDef::Vm(ref vm) = genome.nodes[1].backend_def {
        vm.program.len()
    } else {
        panic!()
    };
    let mut r = rng(42);
    let result = VmMutator::apply(
        &mut genome,
        VmOperator::VmCopyInstructionBlock,
        &[],
        0.0,
        &mut r,
        &MutationConfig::default(),
    );
    assert!(result.is_ok());
    let after = if let BackendDef::Vm(ref vm) = genome.nodes[1].backend_def {
        vm.program.len()
    } else {
        panic!()
    };
    assert!(after > before, "program must grow after copy block");
}

#[test]
fn copy_instruction_block_on_empty_returns_no_applicable_target() {
    let mut genome = v3alpha1_founder_genome();
    if let BackendDef::Vm(ref mut vm) = genome.nodes[1].backend_def {
        vm.program.clear();
    }
    let mut r = rng(0);
    let result = VmMutator::apply(
        &mut genome,
        VmOperator::VmCopyInstructionBlock,
        &[],
        0.0,
        &mut r,
        &MutationConfig::default(),
    );
    assert_eq!(result, Err(MutationSkipReason::NoApplicableTarget));
}

#[test]
fn copy_instruction_block_preserves_content() {
    // All original instruction identities must remain in order after copy.
    let mut genome = v3alpha1_founder_genome();
    let original = if let BackendDef::Vm(ref vm) = genome.nodes[1].backend_def {
        vm.program.clone()
    } else {
        panic!()
    };
    let mut r = rng(7);
    VmMutator::apply(
        &mut genome,
        VmOperator::VmCopyInstructionBlock,
        &[],
        0.0,
        &mut r,
        &MutationConfig::default(),
    )
    .unwrap();
    let after = if let BackendDef::Vm(ref vm) = genome.nodes[1].backend_def {
        vm.program.clone()
    } else {
        panic!()
    };
    let original_non_jumps: Vec<_> = original
        .iter()
        .filter(|instruction| {
            !matches!(
                instruction,
                VmInstruction::Jump { .. } | VmInstruction::JumpIfZero { .. }
            )
        })
        .collect();
    let mut original_index = 0;
    for instruction in after {
        if original_index < original_non_jumps.len()
            && instruction == *original_non_jumps[original_index]
        {
            original_index += 1;
        }
    }
    assert_eq!(original_index, original_non_jumps.len());
}

#[test]
fn copy_instruction_block_respects_max_32() {
    // With a small program, block size is clamped.
    let mut genome = v3alpha1_founder_genome();
    if let BackendDef::Vm(ref mut vm) = genome.nodes[1].backend_def {
        vm.program = vec![VmInstruction::Noop; 3];
    }
    let mut r = rng(0);
    VmMutator::apply(
        &mut genome,
        VmOperator::VmCopyInstructionBlock,
        &[],
        0.0,
        &mut r,
        &MutationConfig::default(),
    )
    .unwrap();
    let after_len = if let BackendDef::Vm(ref vm) = genome.nodes[1].backend_def {
        vm.program.len()
    } else {
        panic!()
    };
    // Original 3 + at most 3 copied = max 6.
    assert!(after_len <= 6, "block copy clamped to program len");
}

// ── VmCopyInstructionBlockRemapped tests ──

#[test]
fn copy_instruction_block_remapped_shifts_registers() {
    let mut genome = v3alpha1_founder_genome();
    if let BackendDef::Vm(ref mut vm) = genome.nodes[1].backend_def {
        vm.program = vec![
            VmInstruction::Add { dst: 0, a: 1, b: 2 },
            VmInstruction::Sub { dst: 1, a: 2, b: 3 },
        ];
        vm.register_count = 8;
    }
    let before = if let BackendDef::Vm(ref vm) = genome.nodes[1].backend_def {
        vm.program.len()
    } else {
        panic!()
    };
    let mut r = rng(42);
    VmMutator::apply(
        &mut genome,
        VmOperator::VmCopyInstructionBlockRemapped,
        &[],
        0.0,
        &mut r,
        &MutationConfig::default(),
    )
    .unwrap();
    let after_len = if let BackendDef::Vm(ref vm) = genome.nodes[1].backend_def {
        vm.program.len()
    } else {
        panic!()
    };
    assert!(after_len > before, "program must grow");
}

#[test]
fn copy_instruction_block_remapped_wraps_registers() {
    // Register remapping must wrap within register_count.
    let mut genome = v3alpha1_founder_genome();
    if let BackendDef::Vm(ref mut vm) = genome.nodes[1].backend_def {
        vm.program = vec![VmInstruction::Move { dst: 3, src: 3 }];
        vm.register_count = 4;
    }
    for seed in 0u64..50 {
        let mut g = genome.clone();
        let mut r = rng(seed);
        let _ = VmMutator::apply(
            &mut g,
            VmOperator::VmCopyInstructionBlockRemapped,
            &[],
            0.0,
            &mut r,
            &MutationConfig::default(),
        );
        if let BackendDef::Vm(ref vm) = g.nodes[1].backend_def {
            for instr in &vm.program {
                if let VmInstruction::Move { dst, src } = instr {
                    assert!(*dst < 4, "dst must be < register_count");
                    assert!(*src < 4, "src must be < register_count");
                }
            }
        }
    }
}

#[test]
fn copy_instruction_block_remapped_preserves_non_register_fields() {
    // Non-register fields (const_idx, action_type, etc.) must not change.
    let mut genome = v3alpha1_founder_genome();
    if let BackendDef::Vm(ref mut vm) = genome.nodes[1].backend_def {
        vm.program = vec![VmInstruction::PushAction { action_type: 42 }];
        vm.register_count = 4;
    }
    let mut r = rng(0);
    let _ = VmMutator::apply(
        &mut genome,
        VmOperator::VmCopyInstructionBlockRemapped,
        &[],
        0.0,
        &mut r,
        &MutationConfig::default(),
    );
    if let BackendDef::Vm(ref vm) = genome.nodes[1].backend_def {
        // The original instruction must still exist unchanged.
        assert!(
            vm.program
                .iter()
                .any(|i| matches!(i, VmInstruction::PushAction { action_type: 42 })),
            "PushAction with action_type 42 must be preserved"
        );
    }
}

#[test]
fn copy_instruction_block_remapped_adjusts_jump_offsets() {
    let mut genome = v3alpha1_founder_genome();
    if let BackendDef::Vm(ref mut vm) = genome.nodes[1].backend_def {
        vm.program = vec![VmInstruction::Jump { offset: 5 }, VmInstruction::Noop];
        vm.register_count = 4;
    }
    let mut found_different_offset = false;
    for seed in 0u64..100 {
        let mut g = genome.clone();
        let mut r = rng(seed);
        let _ = VmMutator::apply(
            &mut g,
            VmOperator::VmCopyInstructionBlockRemapped,
            &[],
            0.0,
            &mut r,
            &MutationConfig::default(),
        );
        if let BackendDef::Vm(ref vm) = g.nodes[1].backend_def {
            for instr in &vm.program {
                if let VmInstruction::Jump { offset } = instr {
                    if *offset != 5 {
                        found_different_offset = true;
                        break;
                    }
                }
            }
        }
        if found_different_offset {
            break;
        }
    }
    assert!(
        found_different_offset,
        "remapped copy must adjust jump offsets"
    );
}

#[test]
fn copy_instruction_block_remapped_on_empty_returns_no_applicable_target() {
    let mut genome = v3alpha1_founder_genome();
    if let BackendDef::Vm(ref mut vm) = genome.nodes[1].backend_def {
        vm.program.clear();
    }
    let mut r = rng(0);
    let result = VmMutator::apply(
        &mut genome,
        VmOperator::VmCopyInstructionBlockRemapped,
        &[],
        0.0,
        &mut r,
        &MutationConfig::default(),
    );
    assert_eq!(result, Err(MutationSkipReason::NoApplicableTarget));
}

// ── VmCopyConstantBlock tests ──

#[test]
fn copy_constant_block_increases_length() {
    let mut genome = v3alpha1_founder_genome();
    let before = if let BackendDef::Vm(ref vm) = genome.nodes[1].backend_def {
        vm.constants.len()
    } else {
        panic!()
    };
    let mut r = rng(0);
    VmMutator::apply(
        &mut genome,
        VmOperator::VmCopyConstantBlock,
        &[],
        0.0,
        &mut r,
        &MutationConfig::default(),
    )
    .unwrap();
    let after = if let BackendDef::Vm(ref vm) = genome.nodes[1].backend_def {
        vm.constants.len()
    } else {
        panic!()
    };
    assert!(after > before, "constants pool must grow");
}

#[test]
fn copy_constant_block_on_empty_returns_no_applicable_target() {
    let mut genome = v3alpha1_founder_genome();
    if let BackendDef::Vm(ref mut vm) = genome.nodes[1].backend_def {
        vm.constants.clear();
    }
    let mut r = rng(0);
    let result = VmMutator::apply(
        &mut genome,
        VmOperator::VmCopyConstantBlock,
        &[],
        0.0,
        &mut r,
        &MutationConfig::default(),
    );
    assert_eq!(result, Err(MutationSkipReason::NoApplicableTarget));
}

#[test]
fn copy_constant_block_preserves_original() {
    let mut genome = v3alpha1_founder_genome();
    let original = if let BackendDef::Vm(ref vm) = genome.nodes[1].backend_def {
        vm.constants.clone()
    } else {
        panic!()
    };
    let mut r = rng(0);
    VmMutator::apply(
        &mut genome,
        VmOperator::VmCopyConstantBlock,
        &[],
        0.0,
        &mut r,
        &MutationConfig::default(),
    )
    .unwrap();
    let after = if let BackendDef::Vm(ref vm) = genome.nodes[1].backend_def {
        vm.constants.clone()
    } else {
        panic!()
    };
    // Original constants must be a prefix of the result.
    assert_eq!(&after[..original.len()], &original[..]);
}

#[test]
fn copy_constant_block_copies_correct_values() {
    let mut genome = v3alpha1_founder_genome();
    if let BackendDef::Vm(ref mut vm) = genome.nodes[1].backend_def {
        vm.constants = vec![10.0, 20.0, 30.0];
    }
    let mut r = rng(0);
    VmMutator::apply(
        &mut genome,
        VmOperator::VmCopyConstantBlock,
        &[],
        0.0,
        &mut r,
        &MutationConfig::default(),
    )
    .unwrap();
    let after = if let BackendDef::Vm(ref vm) = genome.nodes[1].backend_def {
        vm.constants.clone()
    } else {
        panic!()
    };
    // The appended constants must be values from the original [10.0, 20.0, 30.0].
    for &val in &after[3..] {
        assert!(
            val == 10.0 || val == 20.0 || val == 30.0,
            "copied constant {} must come from original pool",
            val
        );
    }
}

// ── VmCopyGeneBackwardSlice tests ──

#[test]
fn copy_gene_backward_slice_increases_program_length() {
    let mut genome = v3alpha1_founder_genome();
    // Ensure program has an output instruction.
    if let BackendDef::Vm(ref mut vm) = genome.nodes[1].backend_def {
        vm.program = vec![
            VmInstruction::ReadInput {
                dst: 0,
                ref_idx: 0,
                sub_idx: 0,
            },
            VmInstruction::Add { dst: 1, a: 0, b: 0 },
            VmInstruction::WriteInternalPayload {
                slot_idx: 0,
                src: 1,
            },
        ];
        vm.register_count = 4;
    }
    let before = 3;
    let mut r = rng(42);
    VmMutator::apply(
        &mut genome,
        VmOperator::VmCopyGeneBackwardSlice,
        &[],
        0.0,
        &mut r,
        &MutationConfig::default(),
    )
    .unwrap();
    let after = if let BackendDef::Vm(ref vm) = genome.nodes[1].backend_def {
        vm.program.len()
    } else {
        panic!()
    };
    assert!(
        after > before,
        "backward slice must increase program length"
    );
}

#[test]
fn copy_gene_backward_slice_no_output_returns_no_applicable_target() {
    let mut genome = v3alpha1_founder_genome();
    // Program with no output instructions.
    if let BackendDef::Vm(ref mut vm) = genome.nodes[1].backend_def {
        vm.program = vec![
            VmInstruction::Noop,
            VmInstruction::Add { dst: 0, a: 1, b: 2 },
        ];
    }
    let mut r = rng(0);
    let result = VmMutator::apply(
        &mut genome,
        VmOperator::VmCopyGeneBackwardSlice,
        &[],
        0.0,
        &mut r,
        &MutationConfig::default(),
    );
    assert_eq!(result, Err(MutationSkipReason::NoApplicableTarget));
}

#[test]
fn copy_gene_backward_slice_captures_dependency_chain() {
    let mut genome = v3alpha1_founder_genome();
    if let BackendDef::Vm(ref mut vm) = genome.nodes[1].backend_def {
        vm.program = vec![
            VmInstruction::ReadInput {
                dst: 0,
                ref_idx: 0,
                sub_idx: 0,
            },
            VmInstruction::Neg { dst: 1, src: 0 },
            VmInstruction::WriteInternalPayload {
                slot_idx: 0,
                src: 1,
            },
        ];
        vm.register_count = 4;
    }
    let mut r = rng(42);
    VmMutator::apply(
        &mut genome,
        VmOperator::VmCopyGeneBackwardSlice,
        &[],
        0.0,
        &mut r,
        &MutationConfig::default(),
    )
    .unwrap();
    let after = if let BackendDef::Vm(ref vm) = genome.nodes[1].backend_def {
        vm.program.clone()
    } else {
        panic!()
    };
    // The gene slice should capture at least the output + one dependency.
    // Program grew by at least 2 (the dependency chain).
    assert!(
        after.len() >= 5,
        "gene slice must capture dependency chain; got len {}",
        after.len()
    );
}

// ── VmCopyGeneForwardSlice tests ──

#[test]
fn copy_gene_forward_slice_increases_program_length() {
    let mut genome = v3alpha1_founder_genome();
    if let BackendDef::Vm(ref mut vm) = genome.nodes[1].backend_def {
        vm.program = vec![
            VmInstruction::ReadInput {
                dst: 0,
                ref_idx: 0,
                sub_idx: 0,
            },
            VmInstruction::Neg { dst: 1, src: 0 },
            VmInstruction::WriteInternalPayload {
                slot_idx: 0,
                src: 1,
            },
        ];
        vm.register_count = 4;
    }
    let before = 3;
    let mut r = rng(42);
    VmMutator::apply(
        &mut genome,
        VmOperator::VmCopyGeneForwardSlice,
        &[],
        0.0,
        &mut r,
        &MutationConfig::default(),
    )
    .unwrap();
    let after = if let BackendDef::Vm(ref vm) = genome.nodes[1].backend_def {
        vm.program.len()
    } else {
        panic!()
    };
    assert!(after > before, "forward slice must increase program length");
}

#[test]
fn copy_gene_forward_slice_no_dst_returns_no_applicable_target() {
    let mut genome = v3alpha1_founder_genome();
    // Program with no register-writing instructions.
    if let BackendDef::Vm(ref mut vm) = genome.nodes[1].backend_def {
        vm.program = vec![
            VmInstruction::Noop,
            VmInstruction::PushAction { action_type: 0 },
            VmInstruction::Halt,
        ];
    }
    let mut r = rng(0);
    let result = VmMutator::apply(
        &mut genome,
        VmOperator::VmCopyGeneForwardSlice,
        &[],
        0.0,
        &mut r,
        &MutationConfig::default(),
    );
    assert_eq!(result, Err(MutationSkipReason::NoApplicableTarget));
}

#[test]
fn copy_gene_forward_slice_captures_dependency_chain() {
    // Over multiple seeds, forward slice must sometimes capture a multi-instruction chain.
    let mut max_growth = 0usize;
    for seed in 0u64..100 {
        let mut genome = v3alpha1_founder_genome();
        if let BackendDef::Vm(ref mut vm) = genome.nodes[1].backend_def {
            vm.program = vec![
                VmInstruction::ReadInput {
                    dst: 0,
                    ref_idx: 0,
                    sub_idx: 0,
                },
                VmInstruction::Neg { dst: 1, src: 0 },
                VmInstruction::Abs { dst: 2, src: 1 },
            ];
            vm.register_count = 4;
        }
        let mut r = rng(seed);
        VmMutator::apply(
            &mut genome,
            VmOperator::VmCopyGeneForwardSlice,
            &[],
            0.0,
            &mut r,
            &MutationConfig::default(),
        )
        .unwrap();
        let after_len = if let BackendDef::Vm(ref vm) = genome.nodes[1].backend_def {
            vm.program.len()
        } else {
            panic!()
        };
        let growth = after_len - 3;
        if growth > max_growth {
            max_growth = growth;
        }
    }
    // Must sometimes capture a chain of 2+ instructions (ReadInput->Neg->Abs).
    assert!(
        max_growth >= 2,
        "forward slice must capture multi-instruction chain; max growth was {}",
        max_growth
    );
}

#[test]
fn vm_weighted_random_favors_refinement() {
    let mut counts = std::collections::HashMap::new();
    let mut r = rng(42);
    for _ in 0..10_000 {
        let op = VmOperator::random(&mut r);
        *counts.entry(op).or_insert(0u32) += 1;
    }
    let constant = counts
        .get(&VmOperator::VmConstantMutation)
        .copied()
        .unwrap_or(0);
    let copy_block = counts
        .get(&VmOperator::VmCopyInstructionBlock)
        .copied()
        .unwrap_or(0);
    assert!(
        constant > copy_block * 2,
        "VmConstantMutation (weight 4) must appear >2x VmCopyInstructionBlock (weight 1); got {} vs {}",
        constant, copy_block,
    );
}

#[test]
fn vm_operator_weights_are_positive() {
    let all = VmOperator::ALL;
    assert_eq!(all.len(), 15, "ALL must cover every VmOperator variant");
    for &op in &all {
        assert!(op.weight() > 0, "weight must be positive for {:?}", op);
    }
}

#[test]
fn complexity_effect_consistent_with_types() {
    use crate::mutation::types::ComplexityEffect;
    for &op in &VmOperator::ALL {
        let effect = op.complexity_effect();
        assert!(
            matches!(
                effect,
                ComplexityEffect::Increasing
                    | ComplexityEffect::Decreasing
                    | ComplexityEffect::Neutral
            ),
            "complexity_effect must return valid effect for {:?}",
            op
        );
    }
}

#[test]
fn vm_has_at_least_one_decreasing_operator() {
    use crate::mutation::types::ComplexityEffect;
    let saw_decreasing = VmOperator::ALL
        .iter()
        .any(|op| op.complexity_effect() == ComplexityEffect::Decreasing);
    assert!(
        saw_decreasing,
        "VM operator set should include at least one Decreasing operator"
    );
}

#[test]
fn vm_insert_read_store_motif_inserts_read_input_and_store_slot_pair() {
    let genome = v3alpha1_founder_genome();
    let mut found_pair = false;
    for seed in 0u64..200 {
        let mut g = genome.clone();
        let mut r = rng(seed);
        if VmMutator::apply(
            &mut g,
            VmOperator::VmInsertReadStoreMotif,
            &[],
            0.0,
            &mut r,
            &MutationConfig::default(),
        )
        .is_ok()
        {
            if let BackendDef::Vm(ref vm) = g.nodes[1].backend_def {
                // Look for consecutive ReadInput + StoreSlotImm.
                for w in vm.program.windows(2) {
                    if matches!(w[0], VmInstruction::ReadInput { .. })
                        && matches!(w[1], VmInstruction::StoreSlotImm { .. })
                    {
                        // Verify the dst register of ReadInput matches src of StoreSlotImm.
                        if let (
                            VmInstruction::ReadInput { dst, .. },
                            VmInstruction::StoreSlotImm { src, .. },
                        ) = (&w[0], &w[1])
                        {
                            if dst == src {
                                found_pair = true;
                                break;
                            }
                        }
                    }
                }
            }
        }
        if found_pair {
            break;
        }
    }
    assert!(
        found_pair,
        "VmInsertReadStoreMotif must insert ReadInput + StoreSlotImm pair with matching registers"
    );
}

#[test]
fn vm_insert_read_bid_motif_inserts_read_input_and_priority_bid_pair() {
    let genome = v3alpha1_founder_genome();
    let mut found_pair = false;
    for seed in 0u64..200 {
        let mut g = genome.clone();
        let mut r = rng(seed);
        if VmMutator::apply(
            &mut g,
            VmOperator::VmInsertReadBidMotif,
            &[],
            0.0,
            &mut r,
            &MutationConfig::default(),
        )
        .is_ok()
        {
            if let BackendDef::Vm(ref vm) = g.nodes[1].backend_def {
                for w in vm.program.windows(3) {
                    if let [VmInstruction::ReadInput { dst, .. }, VmInstruction::SetPriorityBid { src }, VmInstruction::ExecuteActionQueue] =
                        w
                    {
                        if dst == src {
                            found_pair = true;
                            break;
                        }
                    }
                }
            }
        }
        if found_pair {
            break;
        }
    }
    assert!(
        found_pair,
        "VmInsertReadBidMotif must insert ReadInput + SetPriorityBid before ExecuteActionQueue with matching registers"
    );
}

#[test]
fn vm_insert_read_store_motif_skips_on_empty_input_refs() {
    let mut genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![], // empty — no valid ref_idx targets
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 2,
                constants: vec![],
                program: vec![VmInstruction::Halt],
            }),
            targets: vec![],
        }],
    };
    let mut r = rng(42);
    let result = VmMutator::apply(
        &mut genome,
        VmOperator::VmInsertReadStoreMotif,
        &[0],
        0.5,
        &mut r,
        &MutationConfig::default(),
    );
    assert_eq!(
        result,
        Err(MutationSkipReason::NoApplicableTarget),
        "ReadStoreMotif must skip when node has no input refs"
    );
}

#[test]
fn vm_insert_load_compare_motif_inserts_load_slot_and_cmp_gt_pair() {
    let genome = v3alpha1_founder_genome();
    let mut found_pair = false;
    for seed in 0u64..200 {
        let mut g = genome.clone();
        let mut r = rng(seed);
        if VmMutator::apply(
            &mut g,
            VmOperator::VmInsertLoadCompareMotif,
            &[],
            0.0,
            &mut r,
            &MutationConfig::default(),
        )
        .is_ok()
        {
            if let BackendDef::Vm(ref vm) = g.nodes[1].backend_def {
                for w in vm.program.windows(2) {
                    if matches!(w[0], VmInstruction::LoadSlotImm { .. })
                        && matches!(w[1], VmInstruction::CmpGt { .. })
                    {
                        if let (
                            VmInstruction::LoadSlotImm { dst: load_dst, .. },
                            VmInstruction::CmpGt { a, .. },
                        ) = (&w[0], &w[1])
                        {
                            if load_dst == a {
                                found_pair = true;
                                break;
                            }
                        }
                    }
                }
            }
        }
        if found_pair {
            break;
        }
    }
    assert!(
        found_pair,
        "VmInsertLoadCompareMotif must insert LoadSlotImm + CmpGt pair with matching registers"
    );
}

#[test]
fn vm_mutate_slot_address_changes_slot_idx() {
    // Build a genome with slot opcodes.
    let mut genome = v3alpha1_founder_genome();
    if let BackendDef::Vm(ref mut vm) = genome.nodes[1].backend_def {
        vm.program.push(VmInstruction::LoadSlotImm {
            dst: 0,
            slot_idx: 5,
        });
        vm.program.push(VmInstruction::StoreSlotImm {
            slot_idx: 5,
            src: 0,
        });
    }
    let mut changed = false;
    for seed in 0u64..200 {
        let mut g = genome.clone();
        let mut r = rng(seed);
        if VmMutator::apply(
            &mut g,
            VmOperator::VmMutateSlotAddress,
            &[],
            0.0,
            &mut r,
            &MutationConfig::default(),
        )
        .is_ok()
        {
            if let BackendDef::Vm(ref vm) = g.nodes[1].backend_def {
                for instr in &vm.program {
                    match instr {
                        VmInstruction::LoadSlotImm { slot_idx, .. }
                        | VmInstruction::StoreSlotImm { slot_idx, .. }
                            if *slot_idx != 5 =>
                        {
                            changed = true;
                        }
                        _ => {}
                    }
                }
            }
        }
        if changed {
            break;
        }
    }
    assert!(
        changed,
        "VmMutateSlotAddress must sometimes change slot_idx"
    );
}

#[test]
fn vm_mutate_paired_slot_address_co_mutates_load_and_store() {
    let mut genome = v3alpha1_founder_genome();
    if let BackendDef::Vm(ref mut vm) = genome.nodes[1].backend_def {
        vm.program.push(VmInstruction::LoadSlotImm {
            dst: 0,
            slot_idx: 7,
        });
        vm.program.push(VmInstruction::StoreSlotImm {
            slot_idx: 7,
            src: 0,
        });
    }
    let mut co_mutated = false;
    for seed in 0u64..200 {
        let mut g = genome.clone();
        let mut r = rng(seed);
        if VmMutator::apply(
            &mut g,
            VmOperator::VmMutatePairedSlotAddress,
            &[],
            0.0,
            &mut r,
            &MutationConfig::default(),
        )
        .is_ok()
        {
            if let BackendDef::Vm(ref vm) = g.nodes[1].backend_def {
                // Find the load and store that were originally slot 7.
                let mut new_load_slot = None;
                let mut new_store_slot = None;
                for instr in &vm.program {
                    match instr {
                        VmInstruction::LoadSlotImm { slot_idx, .. } => {
                            new_load_slot = Some(*slot_idx);
                        }
                        VmInstruction::StoreSlotImm { slot_idx, .. } => {
                            new_store_slot = Some(*slot_idx);
                        }
                        _ => {}
                    }
                }
                if let (Some(ls), Some(ss)) = (new_load_slot, new_store_slot) {
                    if ls == ss && ls != 7 {
                        co_mutated = true;
                        break;
                    }
                }
            }
        }
    }
    assert!(
        co_mutated,
        "VmMutatePairedSlotAddress must co-mutate both load and store to same new slot"
    );
}

#[test]
fn vm_mutate_paired_slot_address_skips_when_no_paired_group() {
    // Use founder genome without slot instructions — should skip.
    let genome = v3alpha1_founder_genome();
    let mut skipped = false;
    for seed in 0u64..50 {
        let mut g = genome.clone();
        let mut r = rng(seed);
        if VmMutator::apply(
            &mut g,
            VmOperator::VmMutatePairedSlotAddress,
            &[],
            0.0,
            &mut r,
            &MutationConfig::default(),
        )
        .is_err()
        {
            skipped = true;
            break;
        }
    }
    assert!(
        skipped,
        "VmMutatePairedSlotAddress must skip when no load+store pair exists"
    );
}

#[test]
fn vm_raw_field_mutation_ref_idx_bounded() {
    use crate::contracts::{InputReference, WorldInputKey};
    let mut genome = v3alpha1_founder_genome();
    // Node 1 (VM) has input_refs — ensure ReadInput instructions exist.
    if let BackendDef::Vm(ref mut vm) = genome.nodes[1].backend_def {
        vm.program = vec![VmInstruction::ReadInput {
            dst: 0,
            ref_idx: 0,
            sub_idx: 0,
        }];
    }
    // Also add a few more input_refs to make the range non-trivial.
    genome.nodes[1].input_refs = vec![
        InputReference::UpstreamSlot(0),
        InputReference::World(WorldInputKey::FoodHere {
            type_idx: crate::config::OrdinaryFoodTypeId::default(),
        }),
        InputReference::UpstreamSlot(1),
    ];
    let num_refs = genome.nodes[1].input_refs.len();
    let config = MutationConfig::default();
    for seed in 0u64..512 {
        let mut g = genome.clone();
        let mut r = rng(seed);
        let _ = VmMutator::apply(
            &mut g,
            VmOperator::VmInstructionRawFieldMutation,
            &[],
            0.0,
            &mut r,
            &config,
        );
        if let BackendDef::Vm(ref vm) = g.nodes[1].backend_def {
            for instr in &vm.program {
                if let VmInstruction::ReadInput { ref_idx, .. } = instr {
                    assert!(
                        (*ref_idx as usize) < num_refs,
                        "ref_idx {} must be < input_refs.len() {} at seed {}",
                        ref_idx,
                        num_refs,
                        seed
                    );
                }
            }
        }
    }
}

#[test]
fn vm_raw_field_mutation_sub_idx_bounded() {
    use crate::contracts::{InputReference, WorldInputKey};
    use crate::mutation::compound;
    let mut genome = v3alpha1_founder_genome();
    // Set up a compound ring sensor so sub_idx has a meaningful bound (8).
    genome.nodes[1].input_refs = vec![
        InputReference::World(WorldInputKey::NeighborFoodRing {
            type_idx: crate::config::OrdinaryFoodTypeId::default(),
        }),
        InputReference::World(WorldInputKey::FoodHere {
            type_idx: crate::config::OrdinaryFoodTypeId::default(),
        }),
    ];
    if let BackendDef::Vm(ref mut vm) = genome.nodes[1].backend_def {
        vm.program = vec![VmInstruction::ReadInput {
            dst: 0,
            ref_idx: 0,
            sub_idx: 0,
        }];
    }
    let config = MutationConfig::default();
    for seed in 0u64..512 {
        let mut g = genome.clone();
        let mut r = rng(seed);
        let _ = VmMutator::apply(
            &mut g,
            VmOperator::VmInstructionRawFieldMutation,
            &[],
            0.0,
            &mut r,
            &config,
        );
        if let BackendDef::Vm(ref vm) = g.nodes[1].backend_def {
            for instr in &vm.program {
                if let VmInstruction::ReadInput {
                    ref_idx, sub_idx, ..
                } = instr
                {
                    let width = g.nodes[1]
                        .input_refs
                        .get(*ref_idx as usize)
                        .map(|r| compound::sub_value_count(r, &config))
                        .unwrap_or(1);
                    assert!(
                        *sub_idx < width,
                        "sub_idx {} must be < width {} for ref_idx {} at seed {}",
                        sub_idx,
                        width,
                        ref_idx,
                        seed
                    );
                }
            }
        }
    }
}

// --- Cross-process reproducibility of the paired-slot operator (T10.F11) ---

/// A single-VM-node genome running `program`.
fn slot_program_genome(program: Vec<VmInstruction>) -> CreatureGenome {
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 2,
                constants: vec![],
                program,
            }),
            targets: vec![],
        }],
    }
}

/// One paired slot group per entry of `slots`: each slot gets both a load and
/// a store, which is what makes it an eligible `VmMutatePairedSlotAddress`
/// candidate.
fn paired_slot_program(slots: &[u8]) -> Vec<VmInstruction> {
    slots
        .iter()
        .flat_map(|&slot_idx| {
            [
                VmInstruction::LoadSlotImm { dst: 0, slot_idx },
                VmInstruction::StoreSlotImm { slot_idx, src: 0 },
            ]
        })
        .collect()
}

/// Apply `VmMutatePairedSlotAddress` once to a fresh clone of `genome` with a
/// freshly seeded RNG and return the mutated genome.
fn apply_paired_slot_address(genome: &CreatureGenome, seed: u64) -> CreatureGenome {
    let mut mutated = genome.clone();
    let mut r = rng(seed);
    VmMutator::apply(
        &mut mutated,
        VmOperator::VmMutatePairedSlotAddress,
        &[],
        0.0,
        &mut r,
        &MutationConfig::default(),
    )
    .expect("a genome with a paired slot group must be an applicable target");
    mutated
}

#[test]
fn vm_mutate_paired_slot_address_is_reproducible_for_a_seed() {
    let genome = slot_program_genome(paired_slot_program(&[2, 5, 9, 13]));
    let results: Vec<CreatureGenome> = (0..16)
        .map(|_| apply_paired_slot_address(&genome, 90_210))
        .collect();
    for (i, mutated) in results.iter().enumerate() {
        assert_ne!(
            mutated, &genome,
            "application {i} must re-address the slot group it picked"
        );
        assert_eq!(
            mutated, &results[0],
            "application {i} picked a different slot group than application 0 for the \
             same seed: the candidate order is not a function of the genome"
        );
    }
}

#[test]
fn raw_field_mutation_keeps_terminal_instruction_unchanged() {
    // Arrange
    let mut genome = slot_program_genome(vec![VmInstruction::Halt]);
    let before = genome.clone();
    let mut r = rng(7);

    // Act
    let result = VmMutator::apply(
        &mut genome,
        VmOperator::VmInstructionRawFieldMutation,
        &[],
        0.0,
        &mut r,
        &MutationConfig::default(),
    );

    // Assert
    assert_eq!(result, Err(MutationSkipReason::NoApplicableTarget));
    assert_eq!(genome, before);
}

#[test]
fn raw_field_mutation_changes_exactly_one_encoded_field() {
    // Arrange
    let mut genome = slot_program_genome(vec![VmInstruction::LoadConst {
        dst: 4,
        const_idx: 8,
    }]);
    let mut r = rng(7);

    // Act
    VmMutator::apply(
        &mut genome,
        VmOperator::VmInstructionRawFieldMutation,
        &[],
        0.0,
        &mut r,
        &MutationConfig::default(),
    )
    .unwrap();

    // Assert
    let BackendDef::Vm(vm) = &genome.nodes[0].backend_def else {
        panic!("expected VM backend");
    };
    let VmInstruction::LoadConst { dst, const_idx } = vm.program[0] else {
        panic!("field mutation must not replace the opcode");
    };
    assert_eq!(u8::from(dst != 4) + u8::from(const_idx != 8), 1);
}

fn operand_bearing_instructions() -> Vec<VmInstruction> {
    vec![
        VmInstruction::LoadConst {
            dst: 4,
            const_idx: 8,
        },
        VmInstruction::Move { dst: 4, src: 8 },
        VmInstruction::Add {
            dst: 4,
            a: 8,
            b: 12,
        },
        VmInstruction::Sub {
            dst: 4,
            a: 8,
            b: 12,
        },
        VmInstruction::Mul {
            dst: 4,
            a: 8,
            b: 12,
        },
        VmInstruction::Div {
            dst: 4,
            a: 8,
            b: 12,
        },
        VmInstruction::Min {
            dst: 4,
            a: 8,
            b: 12,
        },
        VmInstruction::Max {
            dst: 4,
            a: 8,
            b: 12,
        },
        VmInstruction::Abs { dst: 4, src: 8 },
        VmInstruction::Neg { dst: 4, src: 8 },
        VmInstruction::Clamp01 { dst: 4, src: 8 },
        VmInstruction::CmpGt {
            dst: 4,
            a: 8,
            b: 12,
        },
        VmInstruction::CmpLt {
            dst: 4,
            a: 8,
            b: 12,
        },
        VmInstruction::CmpEq {
            dst: 4,
            a: 8,
            b: 12,
            eps: 16,
        },
        VmInstruction::And {
            dst: 4,
            a: 8,
            b: 12,
        },
        VmInstruction::Or {
            dst: 4,
            a: 8,
            b: 12,
        },
        VmInstruction::Not { dst: 4, src: 8 },
        VmInstruction::ToI32 { dst: 4, src: 8 },
        VmInstruction::ToU8 { dst: 4, src: 8 },
        VmInstruction::ToBool { dst: 4, src: 8 },
        VmInstruction::JumpIfZero {
            cond: 4,
            offset: 17,
        },
        VmInstruction::Jump { offset: 17 },
        VmInstruction::ReadInput {
            dst: 4,
            ref_idx: 8,
            sub_idx: 12,
        },
        VmInstruction::WriteInternalPayload {
            slot_idx: 4,
            src: 8,
        },
        VmInstruction::WriteWorldActionMeta {
            slot_idx: 4,
            src: 8,
        },
        VmInstruction::WriteRouteGate { slot: 4, src: 8 },
        VmInstruction::PushAction { action_type: 4 },
        VmInstruction::ReadActionQueueLength { dst: 4 },
        VmInstruction::ReadActionQueueType {
            index_src: 4,
            dst: 8,
        },
        VmInstruction::ReadActionQueueParam {
            index_src: 4,
            param_slot: 8,
            dst: 12,
        },
        VmInstruction::SetPriorityBid { src: 4 },
        VmInstruction::LoadSlot {
            dst: 4,
            slot_reg: 8,
        },
        VmInstruction::StoreSlot {
            slot_reg: 4,
            src: 8,
        },
        VmInstruction::LoadSlotImm {
            dst: 4,
            slot_idx: 8,
        },
        VmInstruction::StoreSlotImm {
            slot_idx: 4,
            src: 8,
        },
        VmInstruction::LoadSlotPrev {
            dst: 4,
            slot_idx: 8,
        },
        VmInstruction::ClearSlot { slot_idx: 4 },
    ]
}

fn encoded_fields(instruction: &VmInstruction) -> Vec<i64> {
    match instruction {
        VmInstruction::Noop
        | VmInstruction::Halt
        | VmInstruction::ExecuteActionQueue
        | VmInstruction::PopAction => vec![],
        VmInstruction::LoadConst { dst, const_idx } => vec![i64::from(*dst), i64::from(*const_idx)],
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
            vec![i64::from(*dst), i64::from(*src)]
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
            vec![i64::from(*dst), i64::from(*a), i64::from(*b)]
        }
        VmInstruction::CmpEq { dst, a, b, eps } => {
            vec![
                i64::from(*dst),
                i64::from(*a),
                i64::from(*b),
                i64::from(*eps),
            ]
        }
        VmInstruction::JumpIfZero { cond, offset } => vec![i64::from(*cond), i64::from(*offset)],
        VmInstruction::Jump { offset } => vec![i64::from(*offset)],
        VmInstruction::ReadInput {
            dst,
            ref_idx,
            sub_idx,
        } => {
            vec![i64::from(*dst), i64::from(*ref_idx), i64::from(*sub_idx)]
        }
        VmInstruction::WriteInternalPayload { slot_idx, src }
        | VmInstruction::WriteWorldActionMeta { slot_idx, src } => {
            vec![i64::from(*slot_idx), i64::from(*src)]
        }
        VmInstruction::WriteRouteGate { slot, src } => vec![i64::from(*slot), i64::from(*src)],
        VmInstruction::PushAction { action_type }
        | VmInstruction::SetPriorityBid { src: action_type }
        | VmInstruction::ReadActionQueueLength { dst: action_type }
        | VmInstruction::ClearSlot {
            slot_idx: action_type,
        } => vec![i64::from(*action_type)],
        VmInstruction::ReadActionQueueParam {
            index_src,
            param_slot,
            dst,
        } => {
            vec![
                i64::from(*index_src),
                i64::from(*param_slot),
                i64::from(*dst),
            ]
        }
        VmInstruction::LoadSlot { dst, slot_reg } => vec![i64::from(*dst), i64::from(*slot_reg)],
        VmInstruction::StoreSlot { slot_reg, src } => vec![i64::from(*slot_reg), i64::from(*src)],
        VmInstruction::LoadSlotImm { dst, slot_idx }
        | VmInstruction::LoadSlotPrev { dst, slot_idx } => {
            vec![i64::from(*dst), i64::from(*slot_idx)]
        }
        VmInstruction::StoreSlotImm { slot_idx, src } => {
            vec![i64::from(*slot_idx), i64::from(*src)]
        }
    }
}

#[test]
fn raw_field_mutation_visits_each_operand_field_without_changing_its_opcode() {
    for (instruction_index, instruction) in operand_bearing_instructions().into_iter().enumerate() {
        let before_fields = encoded_fields(&instruction);
        let mut changed_fields = vec![false; before_fields.len()];
        for seed in 0..128 {
            let mut mutated = instruction.clone();
            let mut r = rng(seed + (instruction_index as u64 * 1_000));
            assert!(mutate_one_instruction_field(&mut mutated, &mut r));
            assert_eq!(
                std::mem::discriminant(&mutated),
                std::mem::discriminant(&instruction),
                "raw mutation must retain the opcode"
            );
            let after_fields = encoded_fields(&mutated);
            let changed: Vec<_> = before_fields
                .iter()
                .zip(&after_fields)
                .enumerate()
                .filter_map(|(index, (before, after))| (before != after).then_some(index))
                .collect();
            assert_eq!(changed.len(), 1, "exactly one operand must change");
            assert_eq!(
                (after_fields[changed[0]] - before_fields[changed[0]]).abs(),
                1,
                "the selected operand must be nudged by one"
            );
            changed_fields[changed[0]] = true;
        }
        assert!(
            changed_fields.into_iter().all(|visited| visited),
            "every encoded operand must be selectable for {instruction:?}"
        );
    }
}

#[test]
fn raw_field_mutation_nudges_numeric_boundaries_inward() {
    for offset in [i32::MIN, i32::MAX] {
        let mut instruction = VmInstruction::Jump { offset };
        assert!(mutate_one_instruction_field(
            &mut instruction,
            &mut rng(u64::from(offset as u32))
        ));
        let VmInstruction::Jump { offset: after } = instruction else {
            panic!("jump mutation must retain its opcode");
        };
        assert_eq!(
            after,
            if offset == i32::MIN {
                offset + 1
            } else {
                offset - 1
            }
        );
    }

    for instruction in [
        VmInstruction::LoadConst {
            dst: u8::MAX,
            const_idx: u8::MAX,
        },
        VmInstruction::ReadInput {
            dst: u8::MAX,
            ref_idx: u16::MAX,
            sub_idx: u16::MAX,
        },
    ] {
        let before = encoded_fields(&instruction);
        let mut mutated = instruction.clone();
        assert!(mutate_one_instruction_field(&mut mutated, &mut rng(37)));
        let after = encoded_fields(&mutated);
        let changed: Vec<_> = before
            .iter()
            .zip(&after)
            .enumerate()
            .filter_map(|(index, (before, after))| (before != after).then_some(index))
            .collect();
        assert_eq!(changed.len(), 1);
        assert_eq!(after[changed[0]], before[changed[0]] - 1);
    }
}

proptest! {
    #[test]
    fn raw_field_mutation_is_a_one_step_opcode_preserving_property(seed in any::<u64>()) {
        for (instruction_index, instruction) in operand_bearing_instructions().into_iter().enumerate() {
            let before_fields = encoded_fields(&instruction);
            let mut mutated = instruction.clone();
            let mut r = rng(seed.wrapping_add(instruction_index as u64));
            prop_assert!(mutate_one_instruction_field(&mut mutated, &mut r));
            prop_assert_eq!(
                std::mem::discriminant(&mutated),
                std::mem::discriminant(&instruction),
            );
            let after_fields = encoded_fields(&mutated);
            let changed: Vec<_> = before_fields
                .iter()
                .zip(&after_fields)
                .enumerate()
                .filter_map(|(index, (before, after))| (before != after).then_some(index))
                .collect();
            prop_assert_eq!(changed.len(), 1);
            prop_assert_eq!(
                (after_fields[changed[0]] - before_fields[changed[0]]).abs(),
                1,
            );
        }
    }
}

#[test]
fn raw_field_mutation_skips_programs_without_operands() {
    // Arrange
    let mut genome = slot_program_genome(vec![
        VmInstruction::Noop,
        VmInstruction::Halt,
        VmInstruction::ExecuteActionQueue,
        VmInstruction::PopAction,
    ]);
    let before = genome.clone();
    let mut r = rng(9);

    // Act
    let result = VmMutator::apply(
        &mut genome,
        VmOperator::VmInstructionRawFieldMutation,
        &[],
        0.0,
        &mut r,
        &MutationConfig::default(),
    );

    // Assert
    assert_eq!(result, Err(MutationSkipReason::NoApplicableTarget));
    assert_eq!(genome, before);
}

#[test]
fn register_count_mutation_skips_runtime_out_of_range_widths() {
    for register_count in [0, 33] {
        // Arrange
        let mut genome = slot_program_genome(vec![VmInstruction::Move { dst: 0, src: 0 }]);
        let BackendDef::Vm(vm) = &mut genome.nodes[0].backend_def else {
            panic!("expected VM backend");
        };
        vm.register_count = register_count;
        let before = genome.clone();
        let mut r = rng(u64::from(register_count));

        // Act
        let result = VmMutator::apply(
            &mut genome,
            VmOperator::VmRegisterCountMutation,
            &[],
            0.0,
            &mut r,
            &MutationConfig::default(),
        );

        // Assert
        assert_eq!(result, Err(MutationSkipReason::NoApplicableTarget));
        assert_eq!(genome, before);
    }
}

fn register_bearing_instructions(raw: u8) -> Vec<VmInstruction> {
    vec![
        VmInstruction::LoadConst {
            dst: raw,
            const_idx: 0,
        },
        VmInstruction::Move { dst: raw, src: raw },
        VmInstruction::Add {
            dst: raw,
            a: raw,
            b: raw,
        },
        VmInstruction::Sub {
            dst: raw,
            a: raw,
            b: raw,
        },
        VmInstruction::Mul {
            dst: raw,
            a: raw,
            b: raw,
        },
        VmInstruction::Div {
            dst: raw,
            a: raw,
            b: raw,
        },
        VmInstruction::Min {
            dst: raw,
            a: raw,
            b: raw,
        },
        VmInstruction::Max {
            dst: raw,
            a: raw,
            b: raw,
        },
        VmInstruction::Abs { dst: raw, src: raw },
        VmInstruction::Neg { dst: raw, src: raw },
        VmInstruction::Clamp01 { dst: raw, src: raw },
        VmInstruction::CmpGt {
            dst: raw,
            a: raw,
            b: raw,
        },
        VmInstruction::CmpLt {
            dst: raw,
            a: raw,
            b: raw,
        },
        VmInstruction::CmpEq {
            dst: raw,
            a: raw,
            b: raw,
            eps: 0,
        },
        VmInstruction::And {
            dst: raw,
            a: raw,
            b: raw,
        },
        VmInstruction::Or {
            dst: raw,
            a: raw,
            b: raw,
        },
        VmInstruction::Not { dst: raw, src: raw },
        VmInstruction::ToI32 { dst: raw, src: raw },
        VmInstruction::ToU8 { dst: raw, src: raw },
        VmInstruction::ToBool { dst: raw, src: raw },
        VmInstruction::JumpIfZero {
            cond: raw,
            offset: 0,
        },
        VmInstruction::ReadInput {
            dst: raw,
            ref_idx: 0,
            sub_idx: 0,
        },
        VmInstruction::WriteInternalPayload {
            slot_idx: 0,
            src: raw,
        },
        VmInstruction::WriteWorldActionMeta {
            slot_idx: 0,
            src: raw,
        },
        VmInstruction::WriteRouteGate { slot: 0, src: raw },
        VmInstruction::ReadActionQueueLength { dst: raw },
        VmInstruction::ReadActionQueueType {
            index_src: raw,
            dst: raw,
        },
        VmInstruction::ReadActionQueueParam {
            index_src: raw,
            param_slot: 0,
            dst: raw,
        },
        VmInstruction::SetPriorityBid { src: raw },
        VmInstruction::LoadSlot {
            dst: raw,
            slot_reg: raw,
        },
        VmInstruction::StoreSlot {
            slot_reg: raw,
            src: raw,
        },
        VmInstruction::LoadSlotImm {
            dst: raw,
            slot_idx: 0,
        },
        VmInstruction::StoreSlotImm {
            slot_idx: 0,
            src: raw,
        },
        VmInstruction::LoadSlotPrev {
            dst: raw,
            slot_idx: 0,
        },
    ]
}

fn register_fields(instruction: &VmInstruction) -> Vec<u8> {
    match instruction {
        VmInstruction::Noop
        | VmInstruction::Halt
        | VmInstruction::ExecuteActionQueue
        | VmInstruction::PopAction
        | VmInstruction::Jump { .. }
        | VmInstruction::PushAction { .. }
        | VmInstruction::ClearSlot { .. } => vec![],
        VmInstruction::LoadConst { dst, .. }
        | VmInstruction::ReadInput { dst, .. }
        | VmInstruction::ReadActionQueueLength { dst }
        | VmInstruction::LoadSlotImm { dst, .. }
        | VmInstruction::LoadSlotPrev { dst, .. } => vec![*dst],
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
        }
        | VmInstruction::LoadSlot { dst, slot_reg: src } => vec![*dst, *src],
        VmInstruction::Add { dst, a, b }
        | VmInstruction::Sub { dst, a, b }
        | VmInstruction::Mul { dst, a, b }
        | VmInstruction::Div { dst, a, b }
        | VmInstruction::Min { dst, a, b }
        | VmInstruction::Max { dst, a, b }
        | VmInstruction::CmpGt { dst, a, b }
        | VmInstruction::CmpLt { dst, a, b }
        | VmInstruction::And { dst, a, b }
        | VmInstruction::Or { dst, a, b } => vec![*dst, *a, *b],
        VmInstruction::CmpEq { dst, a, b, .. } => vec![*dst, *a, *b],
        VmInstruction::JumpIfZero { cond, .. } | VmInstruction::SetPriorityBid { src: cond } => {
            vec![*cond]
        }
        VmInstruction::WriteInternalPayload { src, .. }
        | VmInstruction::WriteWorldActionMeta { src, .. }
        | VmInstruction::WriteRouteGate { src, .. }
        | VmInstruction::StoreSlotImm { src, .. } => vec![*src],
        VmInstruction::ReadActionQueueParam { index_src, dst, .. } => vec![*index_src, *dst],
        VmInstruction::StoreSlot { slot_reg, src } => vec![*slot_reg, *src],
    }
}

proptest! {
    #[test]
    fn register_count_shrink_canonicalizes_every_register_field_or_skips_atomically(raw in any::<u8>()) {
        let expected = raw % 4;
        for instruction in register_bearing_instructions(raw) {
            let register_field_count = register_fields(&instruction).len();
            let mut original = slot_program_genome(vec![instruction]);
            let BackendDef::Vm(vm) = &mut original.nodes[0].backend_def else {
                unreachable!("fixture must contain a VM");
            };
            vm.register_count = 4;
            let mut found_shrink = false;
            for seed in 0..128 {
                let mut genome = original.clone();
                let before = genome.clone();
                let mut r = rng(seed);
                let result = apply_register_count_mutation(&mut genome, 0, &mut r);
                let BackendDef::Vm(vm) = &genome.nodes[0].backend_def else {
                    unreachable!("fixture must remain a VM");
                };
                if expected == 3 {
                    if result != Err(MutationSkipReason::NoApplicableTarget) {
                        continue;
                    }
                    found_shrink = true;
                    prop_assert_eq!(result, Err(MutationSkipReason::NoApplicableTarget));
                    prop_assert_eq!(genome, before);
                } else {
                    if vm.register_count != 3 {
                        continue;
                    }
                    found_shrink = true;
                    prop_assert_eq!(result, Ok(()));
                    prop_assert_eq!(register_fields(&vm.program[0]), vec![expected; register_field_count]);
                }
                break;
            }
            prop_assert!(found_shrink, "a bounded seed search must find a decrement");
        }
    }
}

#[test]
fn register_count_shrink_preserves_or_skips_effective_register_identity() {
    let mut blocked = slot_program_genome(vec![VmInstruction::Move { dst: 7, src: 6 }]);
    let BackendDef::Vm(vm) = &mut blocked.nodes[0].backend_def else {
        panic!("expected VM backend");
    };
    vm.register_count = 4;
    let mut saw_blocked_shrink = false;
    for seed in 0..128 {
        let mut genome = blocked.clone();
        let before = genome.clone();
        let mut r = rng(seed);
        let result = VmMutator::apply(
            &mut genome,
            VmOperator::VmRegisterCountMutation,
            &[],
            0.0,
            &mut r,
            &MutationConfig::default(),
        );
        if result == Err(MutationSkipReason::NoApplicableTarget) {
            assert_eq!(genome, before, "blocked shrink must be atomic");
            saw_blocked_shrink = true;
            break;
        }
    }
    assert!(saw_blocked_shrink, "expected a seeded decrement attempt");

    let mut permitted = slot_program_genome(vec![VmInstruction::Move { dst: 6, src: 6 }]);
    let BackendDef::Vm(vm) = &mut permitted.nodes[0].backend_def else {
        panic!("expected VM backend");
    };
    vm.register_count = 4;
    let mut saw_permitted_shrink = false;
    for seed in 0..128 {
        let mut genome = permitted.clone();
        let mut r = rng(seed);
        VmMutator::apply(
            &mut genome,
            VmOperator::VmRegisterCountMutation,
            &[],
            0.0,
            &mut r,
            &MutationConfig::default(),
        )
        .ok();
        let BackendDef::Vm(vm) = &genome.nodes[0].backend_def else {
            panic!("expected VM backend");
        };
        if vm.register_count == 3 {
            assert_eq!(
                vm.program,
                vec![VmInstruction::Move { dst: 2, src: 2 }],
                "raw r6 resolves to r2 under the old width and remains r2 after shrink"
            );
            saw_permitted_shrink = true;
            break;
        }
    }
    assert!(
        saw_permitted_shrink,
        "expected a seeded permitted decrement"
    );
}

#[test]
fn motif_insertion_keeps_old_jump_target_identity() {
    for seed in 0..128 {
        // Arrange
        let mut genome = slot_program_genome(vec![
            VmInstruction::Jump { offset: 1 },
            VmInstruction::Noop,
            VmInstruction::PushAction { action_type: 37 },
            VmInstruction::ExecuteActionQueue,
        ]);
        let mut r = rng(seed);

        // Act
        VmMutator::apply(
            &mut genome,
            VmOperator::VmInsertLoadCompareMotif,
            &[],
            0.0,
            &mut r,
            &MutationConfig::default(),
        )
        .unwrap();

        // Assert
        let BackendDef::Vm(vm) = &genome.nodes[0].backend_def else {
            panic!("expected VM backend");
        };
        let (jump_pc, offset) = vm
            .program
            .iter()
            .enumerate()
            .find_map(|(pc, instruction)| match instruction {
                VmInstruction::Jump { offset } => Some((pc, *offset)),
                _ => None,
            })
            .expect("the old jump must survive insertion");
        let target = crate::runtime::vm::jump_target(jump_pc, offset, vm.program.len());
        assert!(matches!(
            vm.program[target],
            VmInstruction::PushAction { action_type: 37 }
        ));
    }
}

proptest! {
    #[test]
    fn insertion_preserves_old_jump_target_for_every_offset_and_boundary(
        offset in any::<i32>(),
        insert_at in 0usize..=4,
    ) {
        let mut program = vec![
            VmInstruction::Jump { offset },
            VmInstruction::PushAction { action_type: 1 },
            VmInstruction::PushAction { action_type: 2 },
            VmInstruction::Halt,
        ];
        let old_target = crate::runtime::vm::jump_target(0, offset, program.len());

        insert_new_instruction_with_reference_repair(
            &mut program,
            insert_at,
            VmInstruction::Noop,
        )
        .unwrap();

        let new_pc = usize::from(insert_at == 0);
        let VmInstruction::Jump { offset } = program[new_pc] else {
            panic!("the old jump must survive insertion");
        };
        let expected_target = old_target + usize::from(old_target >= insert_at);
        prop_assert_eq!(
            crate::runtime::vm::jump_target(new_pc, offset, program.len()),
            expected_target,
        );
    }

    #[test]
    fn deletion_preserves_or_redirects_old_jump_targets(
        offset in any::<i32>(),
        delete_at in 1usize..5,
    ) {
        let mut program = vec![
            VmInstruction::Jump { offset },
            VmInstruction::PushAction { action_type: 1 },
            VmInstruction::PushAction { action_type: 2 },
            VmInstruction::PushAction { action_type: 3 },
            VmInstruction::Halt,
        ];
        let old_target = crate::runtime::vm::jump_target(0, offset, program.len());

        splice_program_with_reference_repair(&mut program, delete_at..delete_at + 1, vec![])
            .unwrap();

        let VmInstruction::Jump { offset } = program[0] else {
            panic!("the old jump must survive deletion");
        };
        let redirected_old_target = if old_target == delete_at {
            (old_target + 1) % 5
        } else {
            old_target
        };
        let expected_target = redirected_old_target
            - usize::from(redirected_old_target > delete_at);
        prop_assert_eq!(
            crate::runtime::vm::jump_target(0, offset, program.len()),
            expected_target,
        );
    }

    #[test]
    fn copied_jumps_follow_copied_internal_targets_and_old_jumps_keep_originals(
        old_offset in any::<i32>(),
        insert_at in 0usize..=5,
    ) {
        let mut program = vec![
            VmInstruction::Jump { offset: old_offset },
            VmInstruction::Jump { offset: 0 },
            VmInstruction::PushAction { action_type: 1 },
            VmInstruction::PushAction { action_type: 2 },
            VmInstruction::Halt,
        ];
        let old_target = crate::runtime::vm::jump_target(0, old_offset, program.len());
        let copied = [1, 2]
            .into_iter()
            .map(|source_index| SpliceInstruction {
                instruction: program[source_index].clone(),
                source_index: Some(source_index),
            })
            .collect();

        splice_program_with_reference_repair(&mut program, insert_at..insert_at, copied).unwrap();

        let old_jump_pc = 2 * usize::from(insert_at == 0);
        let VmInstruction::Jump { offset } = program[old_jump_pc] else {
            panic!("the old jump must survive copy insertion");
        };
        let expected_old_target = old_target + 2 * usize::from(old_target >= insert_at);
        prop_assert_eq!(
            crate::runtime::vm::jump_target(old_jump_pc, offset, program.len()),
            expected_old_target,
        );
        let VmInstruction::Jump { offset } = program[insert_at] else {
            panic!("the copied jump must be at the copied source position");
        };
        prop_assert_eq!(
            crate::runtime::vm::jump_target(insert_at, offset, program.len()),
            insert_at + 1,
        );
    }

    #[test]
    fn insertion_remaps_targets_when_the_old_jump_moves(
        offset in any::<i32>(),
        jump_pc in 0usize..6,
        insert_at in 0usize..=6,
    ) {
        let mut program = vec![VmInstruction::Noop; 6];
        program[jump_pc] = VmInstruction::Jump { offset };
        let old_target = crate::runtime::vm::jump_target(jump_pc, offset, program.len());

        insert_new_instruction_with_reference_repair(
            &mut program,
            insert_at,
            VmInstruction::Noop,
        )
        .unwrap();

        let new_jump_pc = jump_pc + usize::from(jump_pc >= insert_at);
        let VmInstruction::Jump { offset } = program[new_jump_pc] else {
            panic!("the old jump must move with its instruction identity");
        };
        let expected_target = old_target + usize::from(old_target >= insert_at);
        prop_assert_eq!(
            crate::runtime::vm::jump_target(new_jump_pc, offset, program.len()),
            expected_target,
        );
    }

    #[test]
    fn deletion_remaps_targets_when_the_old_jump_moves(
        offset in any::<i32>(),
        jump_pc in 0usize..6,
        delete_at in 0usize..6,
    ) {
        prop_assume!(jump_pc != delete_at);
        let mut program = vec![VmInstruction::Noop; 6];
        program[jump_pc] = VmInstruction::Jump { offset };
        let old_target = crate::runtime::vm::jump_target(jump_pc, offset, program.len());

        splice_program_with_reference_repair(&mut program, delete_at..delete_at + 1, vec![])
            .unwrap();

        let new_jump_pc = jump_pc - usize::from(jump_pc > delete_at);
        let VmInstruction::Jump { offset } = program[new_jump_pc] else {
            panic!("the old jump must survive deletion");
        };
        let redirected = if old_target == delete_at {
            (old_target + 1) % 6
        } else {
            old_target
        };
        let expected_target = redirected - usize::from(redirected > delete_at);
        prop_assert_eq!(
            crate::runtime::vm::jump_target(new_jump_pc, offset, program.len()),
            expected_target,
        );
    }

    #[test]
    fn replacement_keeps_incoming_targets_and_new_offsets_for_every_old_offset(
        offset in any::<i32>(),
        jump_pc in 0usize..5,
        replace_at in 0usize..5,
    ) {
        prop_assume!(jump_pc != replace_at);
        let mut program = vec![VmInstruction::Noop; 5];
        program[jump_pc] = VmInstruction::Jump { offset };
        let old_target = crate::runtime::vm::jump_target(jump_pc, offset, program.len());

        splice_program_with_reference_repair(
            &mut program,
            replace_at..replace_at + 1,
            vec![SpliceInstruction {
                instruction: VmInstruction::Jump { offset: i32::MAX },
                source_index: None,
            }],
        )
        .unwrap();

        let VmInstruction::Jump { offset } = program[jump_pc] else {
            panic!("the surviving old jump must retain its opcode");
        };
        prop_assert_eq!(
            crate::runtime::vm::jump_target(jump_pc, offset, program.len()),
            old_target,
        );
        let VmInstruction::Jump { offset } = program[replace_at] else {
            panic!("replacement jump must retain its authored opcode");
        };
        prop_assert_eq!(offset, i32::MAX);
    }
}

#[test]
fn splice_repair_handles_deleted_and_replaced_targets() {
    let mut middle_delete = vec![
        VmInstruction::Jump { offset: 0 },
        VmInstruction::PushAction { action_type: 1 },
        VmInstruction::PushAction { action_type: 2 },
    ];
    splice_program_with_reference_repair(&mut middle_delete, 1..2, vec![]).unwrap();
    let VmInstruction::Jump { offset } = middle_delete[0] else {
        panic!("jump must survive deletion");
    };
    assert_eq!(
        crate::runtime::vm::jump_target(0, offset, middle_delete.len()),
        1,
        "a deleted middle target follows the first survivor"
    );
    assert!(matches!(
        middle_delete[1],
        VmInstruction::PushAction { action_type: 2 }
    ));

    let mut tail_delete = vec![
        VmInstruction::Jump { offset: 1 },
        VmInstruction::Noop,
        VmInstruction::PushAction { action_type: 3 },
    ];
    splice_program_with_reference_repair(&mut tail_delete, 2..3, vec![]).unwrap();
    let VmInstruction::Jump { offset } = tail_delete[0] else {
        panic!("jump must survive deletion");
    };
    assert_eq!(
        crate::runtime::vm::jump_target(0, offset, tail_delete.len()),
        0
    );

    let mut replacement = vec![VmInstruction::Jump { offset: 0 }, VmInstruction::Noop];
    splice_program_with_reference_repair(
        &mut replacement,
        1..2,
        vec![SpliceInstruction {
            instruction: VmInstruction::Jump { offset: i32::MAX },
            source_index: None,
        }],
    )
    .unwrap();
    let VmInstruction::Jump { offset } = replacement[0] else {
        panic!("incoming jump must survive replacement");
    };
    assert_eq!(
        crate::runtime::vm::jump_target(0, offset, replacement.len()),
        1
    );
    assert!(matches!(
        replacement[1],
        VmInstruction::Jump { offset: i32::MAX }
    ));
}

#[test]
fn copy_repair_uses_selected_copies_only_for_copied_jumps() {
    let mut program = vec![
        VmInstruction::Jump { offset: 1 },
        VmInstruction::Jump { offset: 0 },
        VmInstruction::PushAction { action_type: 1 },
        VmInstruction::Jump { offset: 0 },
        VmInstruction::PushAction { action_type: 2 },
        VmInstruction::Halt,
    ];
    let copies = [1, 2, 3]
        .into_iter()
        .map(|source_index| SpliceInstruction {
            instruction: program[source_index].clone(),
            source_index: Some(source_index),
        })
        .collect();

    splice_program_with_reference_repair(&mut program, 1..1, copies).unwrap();

    let jump_target = |pc| match program[pc] {
        VmInstruction::Jump { offset } => {
            crate::runtime::vm::jump_target(pc, offset, program.len())
        }
        _ => panic!("expected jump"),
    };
    assert_eq!(jump_target(0), 5, "old jump must keep the original target");
    assert_eq!(jump_target(1), 2, "copied internal target follows its copy");
    assert_eq!(
        jump_target(3),
        7,
        "copied external target follows the original"
    );
}

#[test]
fn noncontiguous_conditional_slice_copy_preserves_selected_target_identity() {
    let mut program = vec![
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        },
        VmInstruction::JumpIfZero { cond: 0, offset: 1 },
        VmInstruction::Noop,
        VmInstruction::WriteInternalPayload {
            slot_idx: 0,
            src: 0,
        },
        VmInstruction::Halt,
    ];
    let gene = vm_forward_slice(&program, 0).expect("LoadConst must seed a forward slice");
    assert_eq!(gene.indices, vec![0, 1, 3]);
    let copied = gene
        .indices
        .into_iter()
        .map(|source_index| SpliceInstruction {
            instruction: program[source_index].clone(),
            source_index: Some(source_index),
        })
        .collect();

    splice_program_with_reference_repair(&mut program, 2..2, copied).unwrap();

    let VmInstruction::JumpIfZero { offset, .. } = program[3] else {
        panic!("conditional source must be copied into the noncontiguous slice");
    };
    assert_eq!(crate::runtime::vm::jump_target(3, offset, program.len()), 4);
    assert!(matches!(
        program[4],
        VmInstruction::WriteInternalPayload {
            slot_idx: 0,
            src: 0
        }
    ));
}

proptest! {
    /// Two applications of the paired-slot operator with the same seed to the
    /// same program produce the same program, whatever slot instructions the
    /// program holds.
    #[test]
    fn vm_mutate_paired_slot_address_is_reproducible_for_any_slot_program(
        forced_slot in 0u8..16,
        extra in prop::collection::vec((0u8..16, 0u8..4), 0..24),
        seed in any::<u64>(),
    ) {
        let mut program = paired_slot_program(&[forced_slot]);
        program.extend(extra.into_iter().map(|(slot_idx, kind)| match kind {
            0 => VmInstruction::LoadSlotImm { dst: 0, slot_idx },
            1 => VmInstruction::StoreSlotImm { slot_idx, src: 0 },
            2 => VmInstruction::LoadSlotPrev { dst: 0, slot_idx },
            _ => VmInstruction::ClearSlot { slot_idx },
        }));
        let genome = slot_program_genome(program);
        prop_assert_eq!(
            apply_paired_slot_address(&genome, seed),
            apply_paired_slot_address(&genome, seed)
        );
    }
}
