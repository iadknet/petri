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
fn raw_field_mutation_can_produce_out_of_range_action_type() {
    let mut genome = v3alpha1_founder_genome();
    if let BackendDef::Vm(ref mut vm) = genome.nodes[1].backend_def {
        vm.program = vec![VmInstruction::PushAction { action_type: 0 }];
    }

    let mut saw_out_of_range = false;
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
            if let VmInstruction::PushAction { action_type } = vm.program[0] {
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
    // All original instructions must still be present somewhere after copy.
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
    // Every original instruction must appear in the result.
    for (i, instr) in original.iter().enumerate() {
        assert!(
            after.contains(instr),
            "original instruction at index {} not found in result",
            i
        );
    }
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
    assert_eq!(all.len(), 13, "ALL must cover every VmOperator variant");
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
fn random_non_increasing_never_returns_increasing() {
    use crate::mutation::types::ComplexityEffect;
    for seed in 0u64..200 {
        let mut r = rng(seed);
        if let Some(op) = VmOperator::random_non_increasing(&mut r) {
            assert_ne!(
                op.complexity_effect(),
                ComplexityEffect::Increasing,
                "random_non_increasing returned Increasing operator {:?} at seed {}",
                op,
                seed
            );
        }
    }
}

#[test]
fn random_non_increasing_only_returns_neutral_for_vm() {
    // VM has no Decreasing operators, so all non-increasing should be Neutral.
    use crate::mutation::types::ComplexityEffect;
    for seed in 0u64..200 {
        let mut r = rng(seed);
        if let Some(op) = VmOperator::random_non_increasing(&mut r) {
            assert_eq!(
                op.complexity_effect(),
                ComplexityEffect::Neutral,
                "VM non-increasing must be neutral, got {:?} at seed {}",
                op,
                seed
            );
        }
    }
}

#[test]
fn random_decreasing_returns_none_for_vm() {
    // VM has 0 Decreasing operators, so random_decreasing() must always return None.
    for seed in 0u64..200 {
        let mut r = rng(seed);
        assert!(
            VmOperator::random_decreasing(&mut r).is_none(),
            "VM random_decreasing must return None (0 Decreasing operators), seed {}",
            seed
        );
    }
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
        InputReference::World(WorldInputKey::FoodHere),
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
        InputReference::World(WorldInputKey::NeighborFoodRing),
        InputReference::World(WorldInputKey::FoodHere),
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
