//! T11.F08: the three VM copy operators duplicate a span into a dormant tail.
//!
//! The copy is spliced at the program tail behind a `Halt` guard authored
//! only when the program's last instruction is not already a terminal, so no
//! surviving control-flow reference and no fall-through reaches it. A later
//! jump mutation is the only way in; then the copy runs in its original's
//! place unless a dormant mutation diverged it first.

use super::operators::mutate_one_instruction_field;
use super::{VmMutator, VmOperator};
use crate::config::{MutationConfig, RuntimeConfig};
use crate::contracts::{NodeId, WorldAction};
use crate::creature::genome::{
    BackendDef, CreatureGenome, NodeGenome, VmBackendDef, VmInstruction,
};
use crate::neighborhood::battery::{Battery, Signature};
use crate::runtime::types::{MeshSideOutputs, NodeResult, OUTPUT_SLOT_COUNT};
use crate::runtime::vm::execute_vm_node_with_reserve;
use crate::sensors::perception::{PerceptionSnapshot, SensorSnapshot};
use crate::sensors::static_inputs::StaticInputs;
use crate::sensors::typed_food::TypedFoodLocalSnapshot;
use proptest::prelude::*;
use rand::rngs::SmallRng;
use rand::{Rng as _, SeedableRng};

/// The three copy operators this feature moves to the dormant tail.
/// `VmCopyInstructionBlockRemapped` is deliberately absent: it stays an
/// ordinary behavior-changing macro (see the spec's Non-Goals).
const TAIL_COPY_OPERATORS: [VmOperator; 3] = [
    VmOperator::VmCopyInstructionBlock,
    VmOperator::VmCopyGeneBackwardSlice,
    VmOperator::VmCopyGeneForwardSlice,
];

const REGISTER_COUNT: u8 = 4;

fn constants() -> Vec<f32> {
    vec![0.5, -1.0, 2.0]
}

fn vm_genome(program: Vec<VmInstruction>) -> CreatureGenome {
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: Vec::new(),
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: REGISTER_COUNT,
                constants: constants(),
                program,
            }),
            targets: Vec::new(),
        }],
    }
}

fn program_of(genome: &CreatureGenome) -> Vec<VmInstruction> {
    match &genome.nodes[0].backend_def {
        BackendDef::Vm(vm) => vm.program.clone(),
        BackendDef::Graph(_) => panic!("expected a VM backend"),
    }
}

fn set_program(genome: &mut CreatureGenome, program: Vec<VmInstruction>) {
    match &mut genome.nodes[0].backend_def {
        BackendDef::Vm(vm) => vm.program = program,
        BackendDef::Graph(_) => panic!("expected a VM backend"),
    }
}

fn apply(
    genome: &mut CreatureGenome,
    op: VmOperator,
    seed: u64,
) -> Result<(), crate::mutation::types::MutationSkipReason> {
    VmMutator::apply(
        genome,
        op,
        &[],
        0.0,
        &mut SmallRng::seed_from_u64(seed),
        &MutationConfig::default(),
    )
    .map(|_| ())
}

fn empty_sensor_snapshot() -> SensorSnapshot {
    SensorSnapshot {
        local: StaticInputs {
            food_here: 0.0,
            neighbor_food: [0.0; 8],
            neighbor_barrier: [0.0; 8],
            neighbor_occupied: [0.0; 8],
            generation: 0.0,
            age_ticks: 0.0,
        },
        typed_local_food: TypedFoodLocalSnapshot::zeroed(1),
        perception: PerceptionSnapshot::zeroed(1),
    }
}

/// One VM dispatch's full observable outcome plus the budget readings the
/// neutrality precondition needs.
struct Run {
    result: NodeResult,
    actions: Vec<WorldAction>,
    memory: [f32; 16],
    steps: u32,
}

fn run(program: &[VmInstruction], config: &RuntimeConfig) -> Run {
    let def = VmBackendDef {
        register_count: REGISTER_COUNT,
        constants: constants(),
        program: program.to_vec(),
    };
    let mut energy = 1.0e6f32;
    let mut memory = [0.0f32; 16];
    let prev_memory = [0.0f32; 16];
    let mut side_outputs = MeshSideOutputs::new(config.max_actions_per_turn);
    let result = execute_vm_node_with_reserve(
        &def,
        &[],
        &[0.0f32; OUTPUT_SLOT_COUNT],
        &mut energy,
        0.0,
        0.0,
        &mut memory,
        &prev_memory,
        &empty_sensor_snapshot(),
        config,
        &mut side_outputs,
    );
    let steps = side_outputs.work_counters.vm_steps;
    Run {
        result,
        actions: side_outputs.action_queue.into_actions(),
        memory,
        steps,
    }
}

fn signature(genome: &CreatureGenome) -> Signature {
    Battery::generate(1).signature(genome, &RuntimeConfig::default(), 0.0)
}

/// A program drawn by the production VM instruction generator, optionally
/// with every jump offset redrawn across the whole `i32` range so the
/// property covers wrapped targets as well as the generator's ±16 range.
fn generated_program(seed: u64, len: usize, wild_offsets: bool) -> Vec<VmInstruction> {
    let mut rng = SmallRng::seed_from_u64(seed);
    let mut program: Vec<VmInstruction> = (0..len)
        .map(|_| {
            super::operators::random_vm_instruction(&mut rng, REGISTER_COUNT, constants().len(), 0)
        })
        .collect();
    if wild_offsets {
        for instruction in &mut program {
            match instruction {
                VmInstruction::Jump { offset } | VmInstruction::JumpIfZero { offset, .. } => {
                    *offset = rng.gen();
                }
                _ => {}
            }
        }
    }
    program
}

// ── Tail-copy neutrality over generated programs ───────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(96))]

    /// The dormant tail copy leaves the dispatch's `NodeResult`, action
    /// queue, output slots, and shared memory untouched whenever the
    /// original run neither exhausts energy nor reaches the step cap.
    #[test]
    fn tail_copy_is_neutral_under_ample_budget(
        program_seed in any::<u64>(),
        len in 1usize..12,
        op_seed in any::<u64>(),
        op_index in 0usize..TAIL_COPY_OPERATORS.len(),
        wild_offsets in any::<bool>(),
    ) {
        let config = RuntimeConfig::default();
        let original = generated_program(program_seed, len, wild_offsets);
        let before = run(&original, &config);
        // Ample budget: the original halts on its own terms, and the guard's
        // extra step still fits under the cap.
        prop_assume!(!before.result.energy_exhausted);
        prop_assume!(before.steps + 1 < config.max_vm_steps);

        let mut genome = vm_genome(original.clone());
        if apply(&mut genome, TAIL_COPY_OPERATORS[op_index], op_seed).is_err() {
            return Ok(());
        }
        let copied = program_of(&genome);
        prop_assume!(copied.len() > original.len());

        let after = run(&copied, &config);
        prop_assert_eq!(&after.result, &before.result);
        prop_assert_eq!(&after.actions, &before.actions);
        prop_assert_eq!(after.memory, before.memory);
    }
}

// ── Placement: tail span behind a terminal ─────────────────────────────────

#[test]
fn tail_copy_preserves_the_program_prefix_and_guards_a_non_terminal_program() {
    // No instruction in this program is `Halt`, so a `Halt` at the copy
    // boundary can only be the newly authored guard.
    let original = vec![
        VmInstruction::Noop,
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        },
        VmInstruction::WriteInternalPayload {
            slot_idx: 0,
            src: 0,
        },
    ];
    for op in TAIL_COPY_OPERATORS {
        for seed in 0u64..24 {
            let mut genome = vm_genome(original.clone());
            if apply(&mut genome, op, seed).is_err() {
                continue;
            }
            let copied = program_of(&genome);
            assert_eq!(
                copied[..original.len()],
                original[..],
                "{op:?} seed {seed}: the surviving prefix must be untouched"
            );
            assert_eq!(
                copied[original.len()],
                VmInstruction::Halt,
                "{op:?} seed {seed}: a non-terminal program gains a guard Halt"
            );
            assert!(
                copied.len() > original.len() + 1,
                "{op:?} seed {seed}: the guard is followed by the copied span"
            );
        }
    }
}

#[test]
fn tail_copy_authors_no_guard_when_the_program_already_ends_in_a_terminal() {
    for (label, terminal) in [
        ("execute", VmInstruction::ExecuteActionQueue),
        ("halt", VmInstruction::Halt),
    ] {
        let original = vec![VmInstruction::PushAction { action_type: 3 }, terminal];
        for seed in 0u64..24 {
            let mut genome = vm_genome(original.clone());
            if apply(&mut genome, VmOperator::VmCopyInstructionBlock, seed).is_err() {
                continue;
            }
            let copied = program_of(&genome);
            // The whole two-instruction program is the only block it can copy.
            assert_eq!(
                copied,
                [original.clone(), original.clone()].concat(),
                "{label} seed {seed}: an already-terminal program needs no guard"
            );
        }
    }
}

#[test]
fn the_guard_stops_fall_through_into_the_copied_span() {
    // Without the guard, fall-through past the original's last instruction
    // would run the copy and push the action a second time.
    let original = vec![VmInstruction::PushAction { action_type: 3 }];
    let config = RuntimeConfig::default();
    let before = run(&original, &config);
    let mut genome = vm_genome(original.clone());
    apply(&mut genome, VmOperator::VmCopyInstructionBlock, 5).unwrap();
    let copied = program_of(&genome);
    assert_eq!(
        copied,
        vec![
            VmInstruction::PushAction { action_type: 3 },
            VmInstruction::Halt,
            VmInstruction::PushAction { action_type: 3 },
        ]
    );
    let after = run(&copied, &config);
    assert_eq!(after.actions, before.actions);
    assert_eq!(after.result, before.result);
}

// ── Copy, silent divergence, activation ────────────────────────────────────

/// Copy the whole two-instruction program to the dormant tail, diverge the
/// dormant copy with the production single-field step, then activate the span
/// with a newly authored `Jump` at pc 0: the exact copy reproduces the
/// original's behavior in its place and the diverged one does not.
#[test]
fn vm_copy_diverge_and_activate_trajectory() {
    let original = vec![
        VmInstruction::PushAction { action_type: 3 },
        VmInstruction::ExecuteActionQueue,
    ];
    let base = vm_genome(original.clone());
    let base_signature = signature(&base);

    let mut copied = base.clone();
    apply(&mut copied, VmOperator::VmCopyInstructionBlock, 11).unwrap();
    let span_start = original.len();
    assert_eq!(
        program_of(&copied),
        [original.clone(), original.clone()].concat(),
        "the copy occupies the tail span with no guard after a terminal"
    );
    assert_eq!(
        signature(&copied),
        base_signature,
        "copying the program to the dormant tail is silent"
    );

    let mut diverged = copied.clone();
    let mut program = program_of(&diverged);
    let mut rng = SmallRng::seed_from_u64(3);
    assert!(
        mutate_one_instruction_field(&mut program[span_start], &mut rng),
        "the copied PushAction carries a mutable field"
    );
    assert_ne!(
        program[span_start], original[0],
        "the dormant instruction actually changed"
    );
    set_program(&mut diverged, program);
    assert_eq!(
        signature(&diverged),
        base_signature,
        "diverging the dormant copy is still silent"
    );

    // Activation: one newly authored jump at pc 0 into the copied span.
    for (label, genome, expect_equal) in [("exact", copied, true), ("diverged", diverged, false)] {
        let mut activated = genome;
        let mut program = program_of(&activated);
        crate::mutation::vm::insert_new_instruction_with_reference_repair(
            &mut program,
            0,
            VmInstruction::Jump {
                offset: span_start as i32,
            },
        )
        .unwrap();
        assert_eq!(
            crate::runtime::vm::jump_target(0, span_start as i32, program.len()),
            span_start + 1,
            "{label}: the authored jump lands on the first copied instruction"
        );
        set_program(&mut activated, program);
        let activated_signature = signature(&activated);
        if expect_equal {
            assert_eq!(
                activated_signature, base_signature,
                "{label}: the activated copy works in the original's place"
            );
        } else {
            assert_ne!(
                activated_signature, base_signature,
                "{label}: the activated diverged copy behaves differently"
            );
        }
    }
}
