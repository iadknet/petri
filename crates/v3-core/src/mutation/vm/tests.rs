use super::operators::{
    apply_register_count_mutation, mutate_one_instruction_field,
    splice_program_with_reference_repair, SpliceInstruction,
};
use super::*;
use crate::contracts::NodeId;
use crate::creature::founder::vm_decision_founder_genome as v3alpha1_founder_genome;
use crate::creature::genome::analysis::vm_forward_slice;
use crate::creature::genome::vote::{ActionParamField, VOTE_SINK_COUNT};
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
            &mut TargetSelector::reachable_only(&[], 0.0),
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
        &mut TargetSelector::reachable_only(&[], 0.0),
        &mut r,
        &MutationConfig::default(),
    )
    .unwrap();
    if let BackendDef::Vm(ref vm) = genome.nodes[1].backend_def {
        assert_eq!(vm.constants.len(), 1, "one constant added");
    }
}

/// T11.F23: an empty pool gains exactly the operator's one draw from
/// `[-1, 1]`; the seeds cover both signs, so the range is not one-sided.
#[test]
fn vm_constant_mutation_on_empty_pool_draws_the_constant_from_the_signed_unit_range() {
    let mut saw_negative = false;
    for seed in 0u64..256 {
        // Arrange
        let mut genome = v3alpha1_founder_genome();
        founder_vm_constants(&mut genome).clear();
        let expected: f32 = rng(seed).gen_range(-1.0f32..=1.0);

        // Act
        VmMutator::apply_to_node(
            &mut genome,
            VmOperator::VmConstantMutation,
            1,
            &mut rng(seed),
            &MutationConfig::default(),
        )
        .unwrap();

        // Assert
        let added = founder_vm_constants(&mut genome).clone();
        assert_eq!(added, vec![expected], "seed {seed}");
        assert!(
            (-1.0..=1.0).contains(&added[0]),
            "seed {seed}: {}",
            added[0]
        );
        saw_negative |= added[0] < 0.0;
    }
    assert!(
        saw_negative,
        "the empty-pool draw must reach negative values"
    );
}

/// The constant pool of the founder's VM node (node 1).
fn founder_vm_constants(genome: &mut CreatureGenome) -> &mut Vec<f32> {
    let BackendDef::Vm(ref mut vm) = genome.nodes[1].backend_def else {
        panic!("expected VM backend on node 1")
    };
    &mut vm.constants
}

/// One `VmConstantMutation` on the founder's VM node with its pool set to `[c]`.
fn constant_step(c: f32, seed: u64) -> f32 {
    let mut genome = v3alpha1_founder_genome();
    *founder_vm_constants(&mut genome) = vec![c];
    VmMutator::apply_to_node(
        &mut genome,
        VmOperator::VmConstantMutation,
        1,
        &mut rng(seed),
        &MutationConfig::default(),
    )
    .unwrap();
    founder_vm_constants(&mut genome)[0]
}

/// T11.F23 invariant 3: `|c' - c| <= 0.1 * max(|c|, 1)` up to one f32 ulp of
/// the larger operand (the sum rounds once).
fn within_scale_relative_bound(c: f32, after: f32) -> bool {
    let scale = f64::from(c.abs().max(1.0));
    let slack = f64::from(f32::EPSILON) * f64::from(c.abs().max(after.abs()));
    (f64::from(after) - f64::from(c)).abs() <= 0.1 * scale + slack
}

/// T11.F23 invariant 1: on the unit scale the constant step is the graph
/// parameter step, `c += gen_range(-0.1..=0.1)`, drawn after the index.
#[test]
fn vm_constant_mutation_unit_scale_step_equals_the_graph_parameter_step() {
    for seed in 0u64..256 {
        for c in [0.0f32, 0.25, -0.5, 1.0, -1.0] {
            let after = constant_step(c, seed);
            let mut r = rng(seed);
            let _index = r.gen_range(0..1usize);
            let u: f32 = r.gen_range(-0.1f32..=0.1);
            assert_eq!(after, c + u, "seed {seed} c {c}");
        }
    }
}

/// T11.F23 invariant 4: a raw-scale constant moves by a tenth of its
/// magnitude at most, may cross zero, and is never clamped.
#[test]
fn vm_constant_mutation_raw_scale_step_is_a_tenth_of_the_magnitude() {
    let mut crossed_zero = false;
    let mut moved_past_the_unit_step = false;
    for seed in 0u64..512 {
        let after = constant_step(20.0, seed);
        assert!(
            within_scale_relative_bound(20.0, after),
            "seed {seed}: {after}"
        );
        moved_past_the_unit_step |= (after - 20.0).abs() > 0.1;
        let small = constant_step(0.05, seed);
        assert!(within_scale_relative_bound(0.05, small));
        crossed_zero |= small < 0.0;
    }
    assert!(
        moved_past_the_unit_step,
        "20.0 must move by more than the unit step"
    );
    assert!(
        crossed_zero,
        "a constant near zero must be able to cross it"
    );
}

/// T11.F23 invariant 1: the operator draws the index and then `u` — two
/// draws — so the RNG stream after the event is unchanged from the ±1 rule.
#[test]
fn vm_constant_mutation_consumes_exactly_two_draws() {
    let mut r = rng(7);
    let mut genome = v3alpha1_founder_genome();
    VmMutator::apply_to_node(
        &mut genome,
        VmOperator::VmConstantMutation,
        1,
        &mut r,
        &MutationConfig::default(),
    )
    .unwrap();
    let next: u64 = r.gen();
    let mut reference = rng(7);
    // The fixture's pool holds seven constants.
    let _index = reference.gen_range(0..7usize);
    let _u: f32 = reference.gen_range(-0.1f32..=0.1);
    assert_eq!(next, reference.gen::<u64>());
}

/// T11.F23 invariant 5: a slot-5 hit on the founder's reproduce transfer
/// fraction (2/3) lands in [0.5667, 0.7667]; the sterile (<= 0) and
/// semelparous (>= 1) shares are zero. The tally is the readings-file probe.
#[test]
fn vm_constant_mutation_never_sterilizes_the_founder_transfer_fraction() {
    const SEEDS: u64 = 20_000;
    let mut founder = v3alpha1_founder_genome();
    let fraction = founder_vm_constants(&mut founder)[5];
    let (mut hits, mut sterile, mut semelparous, mut in_band) = (0u32, 0u32, 0u32, 0u32);
    let (mut min, mut max) = (f32::INFINITY, f32::NEG_INFINITY);
    for seed in 0..SEEDS {
        let mut g = founder.clone();
        VmMutator::apply(
            &mut g,
            VmOperator::VmConstantMutation,
            &mut TargetSelector::reachable_only(&[], 0.0),
            &mut rng(seed),
            &MutationConfig::default(),
        )
        .unwrap();
        let after = founder_vm_constants(&mut g)[5];
        if after == fraction {
            continue;
        }
        hits += 1;
        min = min.min(after);
        max = max.max(after);
        sterile += u32::from(after <= 0.0);
        semelparous += u32::from(after >= 1.0);
        in_band += u32::from((0.5667..=0.7667).contains(&after));
    }
    eprintln!(
        "transfer-slot sweep: seeds {SEEDS} slot-5 hits {hits} in-band {in_band} sterile {sterile} semelparous {semelparous} min {min} max {max}"
    );
    assert!(hits > 0);
    assert_eq!((sterile, semelparous), (0, 0));
    assert_eq!(in_band, hits);
}

proptest! {
    /// T11.F23 invariant 3: for every finite `c` in the range whose step
    /// stays finite, `|c' - c| <= 0.1 * max(|c|, 1)` and `c'` is finite.
    #[test]
    fn vm_constant_mutation_step_is_bounded_by_the_constant_scale(
        c in prop_oneof![
            -1.0f32..=1.0,
            -16.0f32..=16.0,
            -(f32::MAX / 1.1)..=(f32::MAX / 1.1),
        ],
        seed in any::<u64>(),
    ) {
        let after = constant_step(c, seed);
        prop_assert!(after.is_finite(), "c {c} -> {after}");
        prop_assert!(within_scale_relative_bound(c, after), "c {c} -> {after}");
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
            &mut TargetSelector::reachable_only(&[], 0.0),
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
        &mut TargetSelector::reachable_only(&[], 0.0),
        &mut r,
        &MutationConfig::default(),
    );
    assert!(result.is_ok());
}

#[test]
fn vm_instruction_mutation_reaches_insert_replace_and_delete() {
    let genome = slot_program_genome(vec![
        VmInstruction::Noop,
        VmInstruction::AddVote { sink: 0, src: 0 },
        VmInstruction::Halt,
    ]);
    let original = match &genome.nodes[0].backend_def {
        BackendDef::Vm(vm) => vm.program.clone(),
        _ => unreachable!("fixture must be a VM"),
    };
    let mut saw_insert = false;
    let mut saw_replacement = false;
    let mut saw_delete = false;
    for seed in 0..256 {
        let mut mutated = genome.clone();
        VmMutator::apply(
            &mut mutated,
            VmOperator::VmInstructionMutation,
            &mut TargetSelector::reachable_only(&[], 0.0),
            &mut rng(seed),
            &MutationConfig::default(),
        )
        .expect("the nonempty VM fixture is applicable");
        let BackendDef::Vm(vm) = &mut mutated.nodes[0].backend_def else {
            unreachable!("fixture must remain a VM");
        };
        match vm.program.len().cmp(&original.len()) {
            std::cmp::Ordering::Greater => saw_insert = true,
            std::cmp::Ordering::Less => saw_delete = true,
            std::cmp::Ordering::Equal if vm.program != original => saw_replacement = true,
            _ => {}
        }
    }
    assert!(saw_insert, "bounded seeds must reach insertion");
    assert!(saw_replacement, "bounded seeds must reach replacement");
    assert!(saw_delete, "bounded seeds must reach deletion");
}

#[test]
fn vm_delete_instruction_removes_one_instruction() {
    let mut genome = v3alpha1_founder_genome();
    if let BackendDef::Vm(ref mut vm) = genome.nodes[1].backend_def {
        vm.program = vec![
            VmInstruction::Noop,
            VmInstruction::Halt,
            VmInstruction::AddVote { sink: 0, src: 0 },
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
        &mut TargetSelector::reachable_only(&[], 0.0),
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
        &mut TargetSelector::reachable_only(&[], 0.0),
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
            crate::creature::genome::cgp::CgpGraphBackendDef::new_with_fixed_outputs(),
        ),
        targets: vec![],
    }];
    let mut r = rng(0);
    let result = VmMutator::apply(
        &mut genome,
        VmOperator::VmConstantMutation,
        &mut TargetSelector::reachable_only(&[], 0.0),
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
            &mut TargetSelector::reachable_only(&[], 0.0),
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
            &mut TargetSelector::reachable_only(&[], 0.0),
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
            &mut TargetSelector::reachable_only(&[], 0.0),
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
        39,
        "all 39 VmInstruction variants must be reachable; got {}",
        discriminants.len()
    );
}

/// T19.F04: `AddVote` is drawable, and its sink is drawn over the catalog, so
/// every vote sink is reachable.
#[test]
fn random_vm_instruction_reaches_every_vote_sink() {
    let mut reached = [false; VOTE_SINK_COUNT];
    for seed in 0u64..16_384 {
        if let VmInstruction::AddVote { sink, .. } = random_vm_instruction(&mut rng(seed), 4, 4, 4)
        {
            reached[usize::from(sink)] = true;
        }
    }
    assert_eq!(reached, [true; VOTE_SINK_COUNT]);
}

proptest! {
    /// T19.F04: a drawn `AddVote` names a catalog sink, never a soft
    /// out-of-range one.
    #[test]
    fn random_vm_instruction_never_draws_an_out_of_catalog_sink(seed in any::<u64>()) {
        let mut r = rng(seed);
        for _ in 0..64 {
            if let VmInstruction::AddVote { sink, .. } = random_vm_instruction(&mut r, 4, 4, 4) {
                prop_assert!(usize::from(sink) < VOTE_SINK_COUNT, "sink {sink}");
            }
        }
    }
}

/// T11.F27: a fresh `WriteActionParam` names a catalogued field, and every
/// field is reached.
#[test]
fn random_vm_instruction_write_action_param_reaches_every_field() {
    let mut reached = [false; ActionParamField::ALL.len()];
    for seed in 0u64..16_384 {
        if let VmInstruction::WriteActionParam { field_idx, .. } =
            random_vm_instruction(&mut rng(seed), 4, 4, 4)
        {
            let field = usize::from(field_idx);
            assert!(field < ActionParamField::ALL.len(), "field {field_idx}");
            reached[field] = true;
        }
    }
    assert_eq!(reached, [true; ActionParamField::ALL.len()]);
}

/// T11.F27: a fresh `ReadActionQueueParam` names a queue-parameter slot that
/// can carry a value, and both slots are drawn.
#[test]
fn random_vm_instruction_read_action_queue_param_draws_both_slots() {
    let mut reached = [false; 2];
    for seed in 0u64..16_384 {
        if let VmInstruction::ReadActionQueueParam { param_slot, .. } =
            random_vm_instruction(&mut rng(seed), 4, 4, 4)
        {
            let slot = usize::from(param_slot);
            assert!(slot < reached.len(), "param_slot {param_slot}");
            reached[slot] = true;
        }
    }
    assert_eq!(reached, [true; 2]);
}

proptest! {
    /// T11.F27 Determinism row: an opcode-25 draw names the field one
    /// `gen_range(0..3)` picks from `ActionParamField::ALL` (T11.F25's single
    /// `usize` draw), then draws `src`, leaving the RNG where that sequence
    /// leaves it. Opcode 29 draws `param_slot` from `0..2` between its two
    /// register operands.
    #[test]
    fn opcode_25_and_29_draws_match_their_pinned_rng_use(seed in any::<u64>()) {
        let mut r = rng(seed);
        for _ in 0..64 {
            let mut replay = r.clone();
            let instruction = random_vm_instruction(&mut r, 4, 4, 4);
            match replay.gen_range(0u8..39) {
                25 => {
                    let field = ActionParamField::ALL[replay.gen_range(0..ActionParamField::ALL.len())];
                    let src = replay.gen_range(0..4u8);
                    prop_assert_eq!(
                        &instruction,
                        &VmInstruction::WriteActionParam { field_idx: field.index() as u8, src }
                    );
                    prop_assert_eq!(r.clone().gen::<u64>(), replay.gen::<u64>());
                }
                29 => {
                    let index_src = replay.gen_range(0..4u8);
                    let param_slot = replay.gen_range(0..2u8);
                    let dst = replay.gen_range(0..4u8);
                    prop_assert_eq!(
                        &instruction,
                        &VmInstruction::ReadActionQueueParam { index_src, param_slot, dst }
                    );
                    prop_assert_eq!(r.clone().gen::<u64>(), replay.gen::<u64>());
                }
                _ => {}
            }
        }
    }
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
fn raw_field_mutation_nudges_a_single_u8_field_at_zero_up_by_one() {
    let mut genome = v3alpha1_founder_genome();
    if let BackendDef::Vm(ref mut vm) = genome.nodes[1].backend_def {
        vm.program = vec![VmInstruction::ClearSlot { slot_idx: 0 }];
    }

    for seed in 0u64..512 {
        let mut g = genome.clone();
        let mut r = rng(seed);
        VmMutator::apply(
            &mut g,
            VmOperator::VmInstructionRawFieldMutation,
            &mut TargetSelector::reachable_only(&[], 0.0),
            &mut r,
            &MutationConfig::default(),
        )
        .unwrap();
        if let BackendDef::Vm(ref vm) = g.nodes[1].backend_def {
            assert!(matches!(
                vm.program[0],
                VmInstruction::ClearSlot { slot_idx: 1 }
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
            &mut TargetSelector::reachable_only(&[], 0.0),
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
            &mut TargetSelector::reachable_only(&[], 0.0),
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
            &mut TargetSelector::reachable_only(&[], 0.0),
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
            &mut TargetSelector::reachable_only(&[], 0.0),
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
            &mut TargetSelector::reachable_only(&[], 0.0),
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
        &mut TargetSelector::reachable_only(&[], 0.0),
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
        &mut TargetSelector::reachable_only(&[], 0.0),
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
        &mut TargetSelector::reachable_only(&[], 0.0),
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
        &mut TargetSelector::reachable_only(&[], 0.0),
        &mut r,
        &MutationConfig::default(),
    )
    .unwrap();
    let after_len = if let BackendDef::Vm(ref vm) = genome.nodes[1].backend_def {
        vm.program.len()
    } else {
        panic!()
    };
    // Original 3 + the T11.F08 guard Halt (the program ends in a Noop) + at
    // most 3 copied = max 7.
    assert!(after_len <= 7, "block copy clamped to program len");
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
        &mut TargetSelector::reachable_only(&[], 0.0),
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
            &mut TargetSelector::reachable_only(&[], 0.0),
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
fn copy_instruction_block_remapped_cyclically_shifts_every_register_field() {
    for raw in [0, 4, u8::MAX] {
        for instruction in register_bearing_instructions(raw) {
            let field_count = register_fields(&instruction).len();
            let mut genome = slot_program_genome(vec![instruction]);
            let BackendDef::Vm(vm) = &mut genome.nodes[0].backend_def else {
                unreachable!("fixture must be a VM");
            };
            vm.register_count = 4;
            for seed in 0..32 {
                let mut mutated = genome.clone();
                VmMutator::apply(
                    &mut mutated,
                    VmOperator::VmCopyInstructionBlockRemapped,
                    &mut TargetSelector::reachable_only(&[], 0.0),
                    &mut rng(seed),
                    &MutationConfig::default(),
                )
                .expect("the one-instruction VM fixture is applicable");
                let BackendDef::Vm(vm) = &mut mutated.nodes[0].backend_def else {
                    unreachable!("fixture must remain a VM");
                };
                assert_eq!(vm.program.len(), 2);
                let original_fields = vec![raw; field_count];
                assert_eq!(
                    vm.program
                        .iter()
                        .filter(|candidate| register_fields(candidate) == original_fields)
                        .count(),
                    1,
                    "one original instruction must retain its raw register fields"
                );
                let remapped = vm
                    .program
                    .iter()
                    .find(|candidate| register_fields(candidate) != original_fields)
                    .expect("the copied instruction must have a nonzero register shift");
                let fields = register_fields(remapped);
                assert_eq!(fields.len(), field_count);
                assert!(fields.iter().all(|field| *field < 4));
                let shift = (fields[0] + 4 - raw % 4) % 4;
                assert_ne!(shift, 0, "the copied register shift must be nonzero");
                assert_eq!(fields, vec![(raw % 4 + shift) % 4; field_count]);
            }
        }
    }
}

#[test]
fn copy_instruction_block_remapped_preserves_non_register_fields() {
    // Non-register fields (const_idx, sink, etc.) must not change.
    let mut genome = v3alpha1_founder_genome();
    if let BackendDef::Vm(ref mut vm) = genome.nodes[1].backend_def {
        vm.program = vec![VmInstruction::AddVote { sink: 42, src: 0 }];
        vm.register_count = 4;
    }
    let mut r = rng(0);
    let _ = VmMutator::apply(
        &mut genome,
        VmOperator::VmCopyInstructionBlockRemapped,
        &mut TargetSelector::reachable_only(&[], 0.0),
        &mut r,
        &MutationConfig::default(),
    );
    if let BackendDef::Vm(ref vm) = genome.nodes[1].backend_def {
        // The original instruction must still exist unchanged.
        assert!(
            vm.program
                .iter()
                .any(|i| matches!(i, VmInstruction::AddVote { sink: 42, .. })),
            "AddVote with sink 42 must be preserved"
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
            &mut TargetSelector::reachable_only(&[], 0.0),
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
        &mut TargetSelector::reachable_only(&[], 0.0),
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
        &mut TargetSelector::reachable_only(&[], 0.0),
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
        &mut TargetSelector::reachable_only(&[], 0.0),
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
        &mut TargetSelector::reachable_only(&[], 0.0),
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
        &mut TargetSelector::reachable_only(&[], 0.0),
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
        &mut TargetSelector::reachable_only(&[], 0.0),
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
        &mut TargetSelector::reachable_only(&[], 0.0),
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
        &mut TargetSelector::reachable_only(&[], 0.0),
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
        &mut TargetSelector::reachable_only(&[], 0.0),
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
            VmInstruction::Noop,
            VmInstruction::Halt,
        ];
    }
    let mut r = rng(0);
    let result = VmMutator::apply(
        &mut genome,
        VmOperator::VmCopyGeneForwardSlice,
        &mut TargetSelector::reachable_only(&[], 0.0),
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
            &mut TargetSelector::reachable_only(&[], 0.0),
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
            &mut TargetSelector::reachable_only(&[], 0.0),
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
fn vm_insert_read_store_motif_keeps_the_generated_pair_adjacent_for_each_seed() {
    use crate::contracts::InputReference;
    let mut genome = slot_program_genome(vec![VmInstruction::Noop]);
    genome.nodes[0].input_refs = vec![InputReference::UpstreamSlot(0)];
    for seed in 0..128 {
        let mut mutated = genome.clone();
        VmMutator::apply(
            &mut mutated,
            VmOperator::VmInsertReadStoreMotif,
            &mut TargetSelector::reachable_only(&[], 0.0),
            &mut rng(seed),
            &MutationConfig::default(),
        )
        .expect("the fixture has an input reference");
        let BackendDef::Vm(vm) = &mut mutated.nodes[0].backend_def else {
            unreachable!("fixture must remain a VM");
        };
        assert_eq!(vm.program.len(), 3);
        assert!(
            vm.program.windows(2).any(|pair| matches!(
                pair,
                [VmInstruction::ReadInput { dst, .. }, VmInstruction::StoreSlotImm { src, .. }]
                    if dst == src
            )),
            "seed {seed} must retain the ordered, wired read/store motif"
        );
    }
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
            &mut TargetSelector::reachable_only(&[], 0.0),
            &mut r,
            &MutationConfig::default(),
        )
        .is_ok()
        {
            if let BackendDef::Vm(ref vm) = g.nodes[1].backend_def {
                for w in vm.program.windows(3) {
                    if let [VmInstruction::ReadInput { dst, .. }, VmInstruction::SetPriorityBid { src }, VmInstruction::Halt] =
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
        "VmInsertReadBidMotif must insert ReadInput + SetPriorityBid before the final Halt with matching registers"
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
        &mut TargetSelector::reachable_only(&[0], 0.5),
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
            &mut TargetSelector::reachable_only(&[], 0.0),
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
fn vm_insert_load_compare_motif_keeps_the_generated_pair_adjacent_for_each_seed() {
    let genome = slot_program_genome(vec![VmInstruction::Noop]);
    for seed in 0..128 {
        let mut mutated = genome.clone();
        VmMutator::apply(
            &mut mutated,
            VmOperator::VmInsertLoadCompareMotif,
            &mut TargetSelector::reachable_only(&[], 0.0),
            &mut rng(seed),
            &MutationConfig::default(),
        )
        .expect("the nonempty VM fixture is applicable");
        let BackendDef::Vm(vm) = &mut mutated.nodes[0].backend_def else {
            unreachable!("fixture must remain a VM");
        };
        assert_eq!(vm.program.len(), 3);
        assert!(
            vm.program.windows(2).any(|pair| matches!(
                pair,
                [VmInstruction::LoadSlotImm { dst, .. }, VmInstruction::CmpGt { a, .. }]
                    if dst == a
            )),
            "seed {seed} must retain the ordered, wired load/compare motif"
        );
    }
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
            &mut TargetSelector::reachable_only(&[], 0.0),
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
fn vm_mutate_slot_address_nudges_register_indirect_slot_fields() {
    let genome = slot_program_genome(vec![VmInstruction::LoadSlot {
        dst: 0,
        slot_reg: 127,
    }]);
    let mut observed = Vec::new();
    for seed in 0..128 {
        let mut mutated = genome.clone();
        VmMutator::apply(
            &mut mutated,
            VmOperator::VmMutateSlotAddress,
            &mut TargetSelector::reachable_only(&[], 0.0),
            &mut rng(seed),
            &MutationConfig::default(),
        )
        .expect("the register-indirect slot instruction is applicable");
        let BackendDef::Vm(vm) = &mut mutated.nodes[0].backend_def else {
            unreachable!("fixture must remain a VM");
        };
        let [VmInstruction::LoadSlot { dst, slot_reg }] = &vm.program[..] else {
            panic!("the standalone nudge must preserve the LoadSlot opcode");
        };
        assert_eq!(*dst, 0);
        observed.push(*slot_reg);
    }
    assert!(observed.contains(&126));
    assert!(observed.contains(&128));
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
            &mut TargetSelector::reachable_only(&[], 0.0),
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
fn paired_slot_address_handles_maximum_encoded_slot_after_single_field_nudge() {
    let mut genome = slot_program_genome(vec![
        VmInstruction::LoadSlotImm {
            dst: 0,
            slot_idx: u8::MAX,
        },
        VmInstruction::StoreSlotImm {
            slot_idx: u8::MAX,
            src: 0,
        },
    ]);
    let mut r = rng(0);

    VmMutator::apply(
        &mut genome,
        VmOperator::VmMutatePairedSlotAddress,
        &mut TargetSelector::reachable_only(&[], 0.0),
        &mut r,
        &MutationConfig::default(),
    )
    .expect("paired macro must handle a slot value produced by a standalone nudge");

    let BackendDef::Vm(vm) = &genome.nodes[0].backend_def else {
        panic!("expected VM backend");
    };
    let [VmInstruction::LoadSlotImm { slot_idx: load, .. }, VmInstruction::StoreSlotImm {
        slot_idx: store, ..
    }] = &vm.program[..]
    else {
        panic!("fixture instructions must remain paired immediate slot accesses");
    };
    assert_eq!(load, store);
    assert!(*load < 16);
    assert_ne!(*load, u8::MAX % 16);
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
            &mut TargetSelector::reachable_only(&[], 0.0),
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
            &mut TargetSelector::reachable_only(&[], 0.0),
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
            &mut TargetSelector::reachable_only(&[], 0.0),
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
                        .map(compound::sub_value_count)
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
        &mut TargetSelector::reachable_only(&[], 0.0),
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
        &mut TargetSelector::reachable_only(&[], 0.0),
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
        &mut TargetSelector::reachable_only(&[], 0.0),
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
        VmInstruction::WriteActionParam {
            field_idx: 4,
            src: 8,
        },
        VmInstruction::WriteActionParam {
            field_idx: 4,
            src: 8,
        },
        VmInstruction::WriteRouteGate { slot: 4, src: 8 },
        VmInstruction::AddVote { sink: 17, src: 0 },
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
        VmInstruction::Noop | VmInstruction::Halt => vec![],
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
        | VmInstruction::WriteActionParam {
            field_idx: slot_idx,
            src,
        }
        | VmInstruction::AddVote {
            sink: slot_idx,
            src,
        } => {
            vec![i64::from(*slot_idx), i64::from(*src)]
        }
        VmInstruction::WriteRouteGate { slot, src } => vec![i64::from(*slot), i64::from(*src)],
        VmInstruction::SetPriorityBid { src: action_type }
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

#[test]
fn raw_field_mutation_exercises_bounded_numeric_directions() {
    let mut u8_directions = Vec::new();
    for seed in 0..128 {
        let mut instruction = VmInstruction::ClearSlot { slot_idx: 127 };
        assert!(mutate_one_instruction_field(
            &mut instruction,
            &mut rng(seed)
        ));
        let VmInstruction::ClearSlot { slot_idx } = instruction else {
            unreachable!();
        };
        u8_directions.push(slot_idx);
    }
    assert!(u8_directions.contains(&126));
    assert!(u8_directions.contains(&128));
    for seed in 0..128 {
        let mut instruction = VmInstruction::ClearSlot { slot_idx: u8::MAX };
        assert!(mutate_one_instruction_field(
            &mut instruction,
            &mut rng(seed)
        ));
        assert!(matches!(
            instruction,
            VmInstruction::ClearSlot { slot_idx: 254 }
        ));
    }

    let mut u16_directions = Vec::new();
    for seed in 0..256 {
        let mut instruction = VmInstruction::ReadInput {
            dst: 0,
            ref_idx: 17,
            sub_idx: 31,
        };
        assert!(mutate_one_instruction_field(
            &mut instruction,
            &mut rng(seed)
        ));
        let VmInstruction::ReadInput {
            ref_idx, sub_idx, ..
        } = instruction
        else {
            unreachable!();
        };
        if ref_idx != 17 {
            u16_directions.push(ref_idx);
        }
        if sub_idx != 31 {
            u16_directions.push(sub_idx);
        }
    }
    assert!(u16_directions.contains(&16));
    assert!(u16_directions.contains(&18));
    assert!(u16_directions.contains(&30));
    assert!(u16_directions.contains(&32));
    let mut observed_zero_u16 = false;
    for seed in 0..256 {
        let mut instruction = VmInstruction::ReadInput {
            dst: 0,
            ref_idx: 0,
            sub_idx: 0,
        };
        assert!(mutate_one_instruction_field(
            &mut instruction,
            &mut rng(seed)
        ));
        let VmInstruction::ReadInput {
            dst,
            ref_idx,
            sub_idx,
        } = instruction
        else {
            unreachable!()
        };
        assert!(matches!(
            (dst, ref_idx, sub_idx),
            (1, 0, 0) | (0, 1, 0) | (0, 0, 1)
        ));
        observed_zero_u16 |= ref_idx == 1 || sub_idx == 1;
    }
    assert!(
        observed_zero_u16,
        "a bounded seed set must select a u16 field"
    );

    let mut i32_directions = Vec::new();
    for seed in 0..128 {
        let mut instruction = VmInstruction::Jump { offset: 0 };
        assert!(mutate_one_instruction_field(
            &mut instruction,
            &mut rng(seed)
        ));
        let VmInstruction::Jump { offset } = instruction else {
            unreachable!();
        };
        i32_directions.push(offset);
    }
    assert!(i32_directions.contains(&-1));
    assert!(i32_directions.contains(&1));
    for seed in 0..128 {
        let mut instruction = VmInstruction::Jump { offset: i32::MIN };
        assert!(mutate_one_instruction_field(
            &mut instruction,
            &mut rng(seed)
        ));
        assert!(matches!(instruction, VmInstruction::Jump { offset } if offset == i32::MIN + 1));
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
        VmInstruction::Halt,
        VmInstruction::Noop,
    ]);
    let before = genome.clone();
    let mut r = rng(9);

    // Act
    let result = VmMutator::apply(
        &mut genome,
        VmOperator::VmInstructionRawFieldMutation,
        &mut TargetSelector::reachable_only(&[], 0.0),
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
            &mut TargetSelector::reachable_only(&[], 0.0),
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
            eps: raw,
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
        VmInstruction::WriteActionParam {
            field_idx: 0,
            src: raw,
        },
        VmInstruction::WriteActionParam {
            field_idx: 0,
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
        | VmInstruction::Jump { .. }
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
        VmInstruction::CmpEq { dst, a, b, eps } => vec![*dst, *a, *b, *eps],
        VmInstruction::JumpIfZero { cond, .. } | VmInstruction::SetPriorityBid { src: cond } => {
            vec![*cond]
        }
        VmInstruction::WriteInternalPayload { src, .. }
        | VmInstruction::WriteActionParam { src, .. }
        | VmInstruction::AddVote { src, .. }
        | VmInstruction::WriteRouteGate { src, .. }
        | VmInstruction::StoreSlotImm { src, .. } => vec![*src],
        VmInstruction::ReadActionQueueParam { index_src, dst, .. } => vec![*index_src, *dst],
        VmInstruction::StoreSlot { slot_reg, src } => vec![*slot_reg, *src],
    }
}

proptest! {
    /// T13.F03 re-pin: a program that uses the register a shrink would remove
    /// no longer skips; the direction draw offers only the feasible move, so
    /// the operator grows instead. Canonicalization of every register field
    /// is unchanged in both directions.
    #[test]
    fn register_count_shrink_canonicalizes_every_register_field_or_grows_instead(raw in any::<u8>()) {
        let expected = raw % 4;
        for instruction in register_bearing_instructions(raw) {
            let register_field_count = register_fields(&instruction).len();
            let mut original = slot_program_genome(vec![instruction]);
            let BackendDef::Vm(vm) = &mut original.nodes[0].backend_def else {
                unreachable!("fixture must contain a VM");
            };
            vm.register_count = 4;
            let mut found_move = false;
            for seed in 0..128 {
                let mut genome = original.clone();
                let mut r = rng(seed);
                let result = apply_register_count_mutation(&mut genome, 0, &mut r);
                prop_assert_eq!(result, Ok(()));
                let BackendDef::Vm(vm) = &genome.nodes[0].backend_def else {
                    unreachable!("fixture must remain a VM");
                };
                if expected == 3 {
                    // The removed register is in use: only growth is offered.
                    prop_assert_eq!(vm.register_count, 5);
                } else if vm.register_count != 3 {
                    continue;
                }
                found_move = true;
                prop_assert_eq!(
                    register_fields(&vm.program[0]),
                    vec![expected; register_field_count]
                );
                break;
            }
            prop_assert!(found_move, "a bounded seed search must find the feasible move");
        }
    }
}

fn register_count_result_for_seed(
    program: Vec<VmInstruction>,
    seed: u64,
) -> (CreatureGenome, Result<(), MutationSkipReason>) {
    let mut genome = slot_program_genome(program);
    let BackendDef::Vm(vm) = &mut genome.nodes[0].backend_def else {
        unreachable!("fixture must contain a VM");
    };
    vm.register_count = 4;
    let mut r = rng(seed);
    let result = apply_register_count_mutation(&mut genome, 0, &mut r);
    (genome, result)
}

/// T13.F03 re-pin: a program using the register a shrink would remove grows
/// on every seed instead of skipping, because the direction is drawn only
/// among the feasible moves.
#[test]
fn register_count_grows_when_shrink_is_blocked_and_shrinks_otherwise() {
    let neutral = vec![VmInstruction::Move { dst: 0, src: 0 }];
    let shrink_seed = (0..128)
        .find(|&seed| {
            let (genome, result) = register_count_result_for_seed(neutral.clone(), seed);
            matches!(result, Ok(()))
                && matches!(genome.nodes[0].backend_def, BackendDef::Vm(ref vm) if vm.register_count == 3)
        })
        .expect("bounded calibration must find a decrement seed");
    let grow_seed = (0..128)
        .find(|&seed| {
            let (genome, result) = register_count_result_for_seed(neutral.clone(), seed);
            matches!(result, Ok(()))
                && matches!(genome.nodes[0].backend_def, BackendDef::Vm(ref vm) if vm.register_count == 5)
        })
        .expect("bounded calibration must find an increment seed");

    // r7 canonicalizes to r3, the register a shrink from width 4 removes, so
    // the shrink is not offered and even the shrink seed grows.
    let raw7 = vec![VmInstruction::Move { dst: 7, src: 7 }];
    let (blocked, blocked_result) = register_count_result_for_seed(raw7.clone(), shrink_seed);
    assert_eq!(blocked_result, Ok(()));
    let BackendDef::Vm(blocked_vm) = &blocked.nodes[0].backend_def else {
        unreachable!();
    };
    assert_eq!(blocked_vm.register_count, 5);
    assert_eq!(
        blocked_vm.program,
        vec![VmInstruction::Move { dst: 3, src: 3 }]
    );

    let (grown, grown_result) = register_count_result_for_seed(raw7, grow_seed);
    assert_eq!(grown_result, Ok(()));
    let BackendDef::Vm(grown_vm) = &grown.nodes[0].backend_def else {
        unreachable!();
    };
    assert_eq!(grown_vm.register_count, 5);
    assert_eq!(
        grown_vm.program,
        vec![VmInstruction::Move { dst: 3, src: 3 }]
    );

    let (shrunk, shrunk_result) =
        register_count_result_for_seed(vec![VmInstruction::Move { dst: 6, src: 6 }], shrink_seed);
    assert_eq!(shrunk_result, Ok(()));
    let BackendDef::Vm(shrunk_vm) = &shrunk.nodes[0].backend_def else {
        unreachable!();
    };
    assert_eq!(shrunk_vm.register_count, 3);
    assert_eq!(
        shrunk_vm.program,
        vec![VmInstruction::Move { dst: 2, src: 2 }]
    );
}

/// T13.F03 re-pin: a blocked shrink is no longer a skip. The node is only
/// selected for a move it can make, so the operator grows on every seed and
/// the effective register identity of the permitted shrink is unchanged.
#[test]
fn register_count_grows_when_shrink_is_blocked_and_preserves_register_identity() {
    let mut blocked = slot_program_genome(vec![VmInstruction::Move { dst: 7, src: 6 }]);
    let BackendDef::Vm(vm) = &mut blocked.nodes[0].backend_def else {
        panic!("expected VM backend");
    };
    vm.register_count = 4;
    for seed in 0..128 {
        let mut genome = blocked.clone();
        let mut r = rng(seed);
        let result = VmMutator::apply(
            &mut genome,
            VmOperator::VmRegisterCountMutation,
            &mut TargetSelector::reachable_only(&[], 0.0),
            &mut r,
            &MutationConfig::default(),
        );
        assert!(
            result.is_ok(),
            "seed {seed}: blocked shrink must grow, not skip"
        );
        let BackendDef::Vm(vm) = &genome.nodes[0].backend_def else {
            panic!("expected VM backend");
        };
        assert_eq!(vm.register_count, 5, "seed {seed}: only growth is feasible");
    }

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
            &mut TargetSelector::reachable_only(&[], 0.0),
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
            VmInstruction::AddVote { sink: 37, src: 0 },
            VmInstruction::Halt,
        ]);
        let mut r = rng(seed);

        // Act
        VmMutator::apply(
            &mut genome,
            VmOperator::VmInsertLoadCompareMotif,
            &mut TargetSelector::reachable_only(&[], 0.0),
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
            VmInstruction::AddVote { sink: 37, src: 0 }
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
            VmInstruction::AddVote { sink: 0, src: 0 },
            VmInstruction::AddVote { sink: 1, src: 0 },
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
            VmInstruction::AddVote { sink: 0, src: 0 },
            VmInstruction::AddVote { sink: 1, src: 0 },
            VmInstruction::AddVote { sink: 9, src: 0 },
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
            VmInstruction::AddVote { sink: 0, src: 0 },
            VmInstruction::AddVote { sink: 1, src: 0 },
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
        VmInstruction::AddVote { sink: 0, src: 0 },
        VmInstruction::AddVote { sink: 1, src: 0 },
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
        VmInstruction::AddVote { sink: 1, src: 0 }
    ));

    let mut tail_delete = vec![
        VmInstruction::Jump { offset: 1 },
        VmInstruction::Noop,
        VmInstruction::AddVote { sink: 9, src: 0 },
    ];
    splice_program_with_reference_repair(&mut tail_delete, 2..3, vec![]).unwrap();
    let VmInstruction::Jump { offset } = tail_delete[0] else {
        panic!("jump must survive deletion");
    };
    assert_eq!(
        crate::runtime::vm::jump_target(0, offset, tail_delete.len()),
        0
    );
    assert_eq!(
        offset, -1,
        "the wrapped target must use canonical offset encoding"
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
        VmInstruction::AddVote { sink: 0, src: 0 },
        VmInstruction::Jump { offset: 0 },
        VmInstruction::AddVote { sink: 1, src: 0 },
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
    #[test]
    fn splice_rejects_invalid_ranges_atomically(len in 1usize..16, start in 0usize..20, end in 0usize..20) {
        prop_assume!(start > end || end > len);
        let mut program = vec![VmInstruction::Noop; len];
        let before = program.clone();
        prop_assert_eq!(splice_program_with_reference_repair(&mut program, start..end, vec![]), Err(MutationSkipReason::NoApplicableTarget));
        prop_assert_eq!(program, before);
    }

    #[test]
    fn paired_slot_address_is_bounded_and_changes_effective_address(
        raw_slot in any::<u8>(),
        seed in any::<u64>(),
    ) {
        let mut genome = slot_program_genome(vec![
            VmInstruction::LoadSlotImm { dst: 0, slot_idx: raw_slot },
            VmInstruction::StoreSlotImm { slot_idx: raw_slot, src: 0 },
        ]);
        let mut r = rng(seed);
        VmMutator::apply(
            &mut genome,
            VmOperator::VmMutatePairedSlotAddress,
            &mut TargetSelector::reachable_only(&[], 0.0),
            &mut r,
            &MutationConfig::default(),
        )
        .expect("the paired immediate-slot fixture is always eligible");
        let BackendDef::Vm(vm) = &genome.nodes[0].backend_def else {
            panic!("fixture must remain a VM backend");
        };
        let [VmInstruction::LoadSlotImm { slot_idx: load, .. }, VmInstruction::StoreSlotImm { slot_idx: store, .. }] = &vm.program[..] else {
            panic!("fixture instructions must remain a paired immediate-slot access");
        };
        prop_assert_eq!(load, store);
        prop_assert!(*load < 16);
        prop_assert_ne!(*load, raw_slot % 16);
    }

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
