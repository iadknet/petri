use super::*;
use crate::config::RuntimeConfig;
use crate::contracts::InputReference;
use crate::creature::genome::{VmBackendDef, VmInstruction};
use crate::runtime::types::MeshSideOutputs;
use crate::runtime::vm::execute_vm_node;
use crate::sensors::static_inputs::StaticInputs;

fn config() -> RuntimeConfig {
    RuntimeConfig::default()
}

fn empty_si() -> StaticInputs {
    StaticInputs {
        food_here: 0.0,
        neighbor_food: [0.0; 8],
        neighbor_barrier: [0.0; 8],
        neighbor_occupied: [0.0; 8],
        generation: 0.0,
        age_ticks: 0.0,
    }
}

fn assert_equivalent(
    label: &str,
    def: VmBackendDef,
    input_refs: Vec<InputReference>,
    upstream: [f32; 12],
    memory_seed: [u8; 1024],
) {
    let si = empty_si();
    let cfg = config();

    let mut energy_a = 100.0f32;
    let mut memory_a = memory_seed;
    let mut aq_a = MeshSideOutputs::new(cfg.max_actions_per_turn);
    let result_a = execute_vm_node(
        &def,
        &input_refs,
        &upstream,
        &mut energy_a,
        0.0,
        &mut memory_a,
        &si,
        &cfg,
        &mut aq_a,
    );

    let mut energy_b = 100.0f32;
    let mut memory_b = memory_seed;
    let mut aq_b = MeshSideOutputs::new(cfg.max_actions_per_turn);
    let (result_b, _trace) = execute_vm_node_traced(
        &def,
        &input_refs,
        &upstream,
        &mut energy_b,
        0.0,
        &mut memory_b,
        &si,
        &cfg,
        &mut aq_b,
    );

    assert_eq!(result_a, result_b, "{label}: NodeResult mismatch");
    assert!(
        (energy_a - energy_b).abs() < 1e-6,
        "{label}: energy mismatch {energy_a} vs {energy_b}"
    );
    assert_eq!(memory_a, memory_b, "{label}: memory mismatch");
}

/// Result equivalence: traced and non-traced produce identical NodeResult
/// and energy delta.
#[test]
fn result_equivalence_emit_eat() {
    let def = VmBackendDef {
        register_count: 2,
        constants: vec![0.8],
        program: vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            },
            VmInstruction::ToBool { dst: 1, src: 0 },
            VmInstruction::PushAction { action_type: 1 },
            VmInstruction::ExecuteActionQueue,
        ],
    };
    let input_refs: Vec<InputReference> = vec![];
    let upstream = [0.0f32; 12];
    let si = empty_si();
    let cfg = config();

    // Run non-traced
    let mut energy_a = 100.0f32;
    let mut memory_a = [0u8; 1024];
    let mut aq_a = MeshSideOutputs::new(cfg.max_actions_per_turn);
    let result_a = execute_vm_node(
        &def,
        &input_refs,
        &upstream,
        &mut energy_a,
        0.0,
        &mut memory_a,
        &si,
        &cfg,
        &mut aq_a,
    );

    // Run traced
    let mut energy_b = 100.0f32;
    let mut memory_b = [0u8; 1024];
    let mut aq_b = MeshSideOutputs::new(cfg.max_actions_per_turn);
    let (result_b, _trace) = execute_vm_node_traced(
        &def,
        &input_refs,
        &upstream,
        &mut energy_b,
        0.0,
        &mut memory_b,
        &si,
        &cfg,
        &mut aq_b,
    );

    assert_eq!(result_a, result_b);
    assert!(
        (energy_a - energy_b).abs() < 1e-6,
        "energy: {energy_a} vs {energy_b}"
    );
    assert_eq!(memory_a, memory_b);
}

/// Trace contains correct instruction sequence.
#[test]
fn trace_contains_correct_instructions() {
    let def = VmBackendDef {
        register_count: 2,
        constants: vec![5.0],
        program: vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            },
            VmInstruction::Abs { dst: 1, src: 0 },
            VmInstruction::Halt,
        ],
    };
    let si = empty_si();
    let cfg = config();
    let mut energy = 100.0f32;
    let mut memory = [0u8; 1024];
    let mut side_outputs = MeshSideOutputs::new(cfg.max_actions_per_turn);

    let (_result, trace) = execute_vm_node_traced(
        &def,
        &[],
        &[0.0; 12],
        &mut energy,
        0.0,
        &mut memory,
        &si,
        &cfg,
        &mut side_outputs,
    );

    assert_eq!(trace.steps.len(), 3);
    assert_eq!(trace.steps[0].pc, 0);
    assert!(matches!(
        trace.steps[0].instruction,
        VmInstruction::LoadConst { .. }
    ));
    assert_eq!(trace.steps[1].pc, 1);
    assert!(matches!(
        trace.steps[1].instruction,
        VmInstruction::Abs { .. }
    ));
    assert_eq!(trace.steps[2].pc, 2);
    assert!(matches!(trace.steps[2].instruction, VmInstruction::Halt));
}

/// Register changes are accurately captured.
#[test]
fn register_changes_captured() {
    let def = VmBackendDef {
        register_count: 4,
        constants: vec![3.5],
        program: vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            },
            VmInstruction::Halt,
        ],
    };
    let si = empty_si();
    let cfg = config();
    let mut energy = 100.0f32;
    let mut memory = [0u8; 1024];
    let mut side_outputs = MeshSideOutputs::new(cfg.max_actions_per_turn);

    let (_result, trace) = execute_vm_node_traced(
        &def,
        &[],
        &[0.0; 12],
        &mut energy,
        0.0,
        &mut memory,
        &si,
        &cfg,
        &mut side_outputs,
    );

    // LoadConst should change r0 from 0.0 to 3.5
    assert!(!trace.steps[0].register_changes.is_empty());
    let (reg_idx, new_val) = trace.steps[0].register_changes[0];
    assert_eq!(reg_idx, 0);
    assert!((new_val - 3.5).abs() < 1e-6);

    // Final registers should have r0=3.5
    assert!((trace.final_registers[0] - 3.5).abs() < 1e-6);
}

/// Memory writes are tracked.
#[test]
fn memory_writes_tracked() {
    let def = VmBackendDef {
        register_count: 2,
        constants: vec![42.0, 7.0],
        program: vec![
            // Load addr=7 into r0
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 1,
            },
            // Load value=42 into r1
            VmInstruction::LoadConst {
                dst: 1,
                const_idx: 0,
            },
            // Store r1 at addr in r0 → mem[7] = 42
            VmInstruction::StoreMem8 {
                addr_reg: 0,
                src: 1,
            },
            VmInstruction::Halt,
        ],
    };
    let si = empty_si();
    let cfg = config();
    let mut energy = 100.0f32;
    let mut memory = [0u8; 1024];
    let mut side_outputs = MeshSideOutputs::new(cfg.max_actions_per_turn);

    let (_result, trace) = execute_vm_node_traced(
        &def,
        &[],
        &[0.0; 12],
        &mut energy,
        0.0,
        &mut memory,
        &si,
        &cfg,
        &mut side_outputs,
    );

    assert_eq!(trace.memory_writes.len(), 1);
    assert_eq!(trace.memory_writes[0].address, 7);
    assert_eq!(trace.memory_writes[0].old_value, 0);
    assert_eq!(trace.memory_writes[0].new_value, 42);

    // Verify memory was committed
    assert_eq!(memory[7], 42);
}

/// Result equivalence with energy exhaustion.
#[test]
fn result_equivalence_energy_exhaustion() {
    let def = VmBackendDef {
        register_count: 1,
        constants: vec![],
        program: vec![
            VmInstruction::Noop,
            VmInstruction::Noop,
            VmInstruction::Halt,
        ],
    };
    let si = empty_si();
    let cfg = config();

    // Energy just enough for ~1 Noop (0.05), second will exhaust
    let mut energy_a = 0.06f32;
    let mut memory_a = [0u8; 1024];
    let mut aq_a = MeshSideOutputs::new(cfg.max_actions_per_turn);
    let result_a = execute_vm_node(
        &def,
        &[],
        &[0.0; 12],
        &mut energy_a,
        0.0,
        &mut memory_a,
        &si,
        &cfg,
        &mut aq_a,
    );

    let mut energy_b = 0.06f32;
    let mut memory_b = [0u8; 1024];
    let mut aq_b = MeshSideOutputs::new(cfg.max_actions_per_turn);
    let (result_b, _trace) = execute_vm_node_traced(
        &def,
        &[],
        &[0.0; 12],
        &mut energy_b,
        0.0,
        &mut memory_b,
        &si,
        &cfg,
        &mut aq_b,
    );

    assert_eq!(result_a, result_b);
    assert!(
        (energy_a - energy_b).abs() < 1e-6,
        "energy: {energy_a} vs {energy_b}"
    );
}

/// Result equivalence with routing (Halt + WriteRouteTarget).
#[test]
fn result_equivalence_routing() {
    let def = VmBackendDef {
        register_count: 1,
        constants: vec![2.5],
        program: vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            },
            VmInstruction::WriteRouteTarget { src: 0 },
            VmInstruction::Halt,
        ],
    };
    let si = empty_si();
    let cfg = config();

    let mut energy_a = 100.0f32;
    let mut memory_a = [0u8; 1024];
    let mut aq_a = MeshSideOutputs::new(cfg.max_actions_per_turn);
    let result_a = execute_vm_node(
        &def,
        &[],
        &[0.0; 12],
        &mut energy_a,
        0.0,
        &mut memory_a,
        &si,
        &cfg,
        &mut aq_a,
    );

    let mut energy_b = 100.0f32;
    let mut memory_b = [0u8; 1024];
    let mut aq_b = MeshSideOutputs::new(cfg.max_actions_per_turn);
    let (result_b, trace) = execute_vm_node_traced(
        &def,
        &[],
        &[0.0; 12],
        &mut energy_b,
        0.0,
        &mut memory_b,
        &si,
        &cfg,
        &mut aq_b,
    );

    assert_eq!(result_a, result_b);
    assert!(
        (energy_a - energy_b).abs() < 1e-6,
        "energy: {energy_a} vs {energy_b}"
    );
    assert!((trace.final_route_target - 2.5).abs() < 1e-6);
}

#[test]
fn result_equivalence_all_38_opcodes() {
    let zero_upstream = [0.0f32; 12];
    let mut read_input_upstream = [0.0f32; 12];
    read_input_upstream[3] = 42.0;

    let zero_mem = [0u8; 1024];
    let mut mem_for_load = [0u8; 1024];
    mem_for_load[5] = 123;
    let mut mem_for_load_imm = [0u8; 1024];
    mem_for_load_imm[9] = 231;

    type OpcodeCase = (
        &'static str,
        VmBackendDef,
        Vec<InputReference>,
        [f32; 12],
        [u8; 1024],
    );
    let cases: Vec<OpcodeCase> = vec![
        (
            "Noop",
            VmBackendDef {
                register_count: 1,
                constants: vec![],
                program: vec![VmInstruction::Noop, VmInstruction::Halt],
            },
            vec![],
            zero_upstream,
            zero_mem,
        ),
        (
            "LoadConst",
            VmBackendDef {
                register_count: 1,
                constants: vec![7.0],
                program: vec![
                    VmInstruction::LoadConst {
                        dst: 0,
                        const_idx: 0,
                    },
                    VmInstruction::Halt,
                ],
            },
            vec![],
            zero_upstream,
            zero_mem,
        ),
        (
            "Move",
            VmBackendDef {
                register_count: 2,
                constants: vec![9.5],
                program: vec![
                    VmInstruction::LoadConst {
                        dst: 0,
                        const_idx: 0,
                    },
                    VmInstruction::Move { dst: 1, src: 0 },
                    VmInstruction::Halt,
                ],
            },
            vec![],
            zero_upstream,
            zero_mem,
        ),
        (
            "Add",
            VmBackendDef {
                register_count: 3,
                constants: vec![2.0, 3.0],
                program: vec![
                    VmInstruction::LoadConst {
                        dst: 0,
                        const_idx: 0,
                    },
                    VmInstruction::LoadConst {
                        dst: 1,
                        const_idx: 1,
                    },
                    VmInstruction::Add { dst: 2, a: 0, b: 1 },
                    VmInstruction::Halt,
                ],
            },
            vec![],
            zero_upstream,
            zero_mem,
        ),
        (
            "Sub",
            VmBackendDef {
                register_count: 3,
                constants: vec![5.0, 2.0],
                program: vec![
                    VmInstruction::LoadConst {
                        dst: 0,
                        const_idx: 0,
                    },
                    VmInstruction::LoadConst {
                        dst: 1,
                        const_idx: 1,
                    },
                    VmInstruction::Sub { dst: 2, a: 0, b: 1 },
                    VmInstruction::Halt,
                ],
            },
            vec![],
            zero_upstream,
            zero_mem,
        ),
        (
            "Mul",
            VmBackendDef {
                register_count: 3,
                constants: vec![4.0, 6.0],
                program: vec![
                    VmInstruction::LoadConst {
                        dst: 0,
                        const_idx: 0,
                    },
                    VmInstruction::LoadConst {
                        dst: 1,
                        const_idx: 1,
                    },
                    VmInstruction::Mul { dst: 2, a: 0, b: 1 },
                    VmInstruction::Halt,
                ],
            },
            vec![],
            zero_upstream,
            zero_mem,
        ),
        (
            "Div",
            VmBackendDef {
                register_count: 3,
                constants: vec![8.0, 2.0],
                program: vec![
                    VmInstruction::LoadConst {
                        dst: 0,
                        const_idx: 0,
                    },
                    VmInstruction::LoadConst {
                        dst: 1,
                        const_idx: 1,
                    },
                    VmInstruction::Div { dst: 2, a: 0, b: 1 },
                    VmInstruction::Halt,
                ],
            },
            vec![],
            zero_upstream,
            zero_mem,
        ),
        (
            "Min",
            VmBackendDef {
                register_count: 3,
                constants: vec![1.5, 4.0],
                program: vec![
                    VmInstruction::LoadConst {
                        dst: 0,
                        const_idx: 0,
                    },
                    VmInstruction::LoadConst {
                        dst: 1,
                        const_idx: 1,
                    },
                    VmInstruction::Min { dst: 2, a: 0, b: 1 },
                    VmInstruction::Halt,
                ],
            },
            vec![],
            zero_upstream,
            zero_mem,
        ),
        (
            "Max",
            VmBackendDef {
                register_count: 3,
                constants: vec![1.5, 4.0],
                program: vec![
                    VmInstruction::LoadConst {
                        dst: 0,
                        const_idx: 0,
                    },
                    VmInstruction::LoadConst {
                        dst: 1,
                        const_idx: 1,
                    },
                    VmInstruction::Max { dst: 2, a: 0, b: 1 },
                    VmInstruction::Halt,
                ],
            },
            vec![],
            zero_upstream,
            zero_mem,
        ),
        (
            "Abs",
            VmBackendDef {
                register_count: 2,
                constants: vec![-3.2],
                program: vec![
                    VmInstruction::LoadConst {
                        dst: 0,
                        const_idx: 0,
                    },
                    VmInstruction::Abs { dst: 1, src: 0 },
                    VmInstruction::Halt,
                ],
            },
            vec![],
            zero_upstream,
            zero_mem,
        ),
        (
            "Neg",
            VmBackendDef {
                register_count: 2,
                constants: vec![3.2],
                program: vec![
                    VmInstruction::LoadConst {
                        dst: 0,
                        const_idx: 0,
                    },
                    VmInstruction::Neg { dst: 1, src: 0 },
                    VmInstruction::Halt,
                ],
            },
            vec![],
            zero_upstream,
            zero_mem,
        ),
        (
            "Clamp01",
            VmBackendDef {
                register_count: 2,
                constants: vec![2.2],
                program: vec![
                    VmInstruction::LoadConst {
                        dst: 0,
                        const_idx: 0,
                    },
                    VmInstruction::Clamp01 { dst: 1, src: 0 },
                    VmInstruction::Halt,
                ],
            },
            vec![],
            zero_upstream,
            zero_mem,
        ),
        (
            "CmpGt",
            VmBackendDef {
                register_count: 3,
                constants: vec![3.0, 1.0],
                program: vec![
                    VmInstruction::LoadConst {
                        dst: 0,
                        const_idx: 0,
                    },
                    VmInstruction::LoadConst {
                        dst: 1,
                        const_idx: 1,
                    },
                    VmInstruction::CmpGt { dst: 2, a: 0, b: 1 },
                    VmInstruction::Halt,
                ],
            },
            vec![],
            zero_upstream,
            zero_mem,
        ),
        (
            "CmpLt",
            VmBackendDef {
                register_count: 3,
                constants: vec![1.0, 3.0],
                program: vec![
                    VmInstruction::LoadConst {
                        dst: 0,
                        const_idx: 0,
                    },
                    VmInstruction::LoadConst {
                        dst: 1,
                        const_idx: 1,
                    },
                    VmInstruction::CmpLt { dst: 2, a: 0, b: 1 },
                    VmInstruction::Halt,
                ],
            },
            vec![],
            zero_upstream,
            zero_mem,
        ),
        (
            "CmpEq",
            VmBackendDef {
                register_count: 4,
                constants: vec![2.0, 2.0, 0.01],
                program: vec![
                    VmInstruction::LoadConst {
                        dst: 0,
                        const_idx: 0,
                    },
                    VmInstruction::LoadConst {
                        dst: 1,
                        const_idx: 1,
                    },
                    VmInstruction::LoadConst {
                        dst: 2,
                        const_idx: 2,
                    },
                    VmInstruction::CmpEq {
                        dst: 3,
                        a: 0,
                        b: 1,
                        eps: 2,
                    },
                    VmInstruction::Halt,
                ],
            },
            vec![],
            zero_upstream,
            zero_mem,
        ),
        (
            "And",
            VmBackendDef {
                register_count: 3,
                constants: vec![1.0, 1.0],
                program: vec![
                    VmInstruction::LoadConst {
                        dst: 0,
                        const_idx: 0,
                    },
                    VmInstruction::LoadConst {
                        dst: 1,
                        const_idx: 1,
                    },
                    VmInstruction::And { dst: 2, a: 0, b: 1 },
                    VmInstruction::Halt,
                ],
            },
            vec![],
            zero_upstream,
            zero_mem,
        ),
        (
            "Or",
            VmBackendDef {
                register_count: 3,
                constants: vec![0.0, 1.0],
                program: vec![
                    VmInstruction::LoadConst {
                        dst: 0,
                        const_idx: 0,
                    },
                    VmInstruction::LoadConst {
                        dst: 1,
                        const_idx: 1,
                    },
                    VmInstruction::Or { dst: 2, a: 0, b: 1 },
                    VmInstruction::Halt,
                ],
            },
            vec![],
            zero_upstream,
            zero_mem,
        ),
        (
            "Not",
            VmBackendDef {
                register_count: 2,
                constants: vec![0.0],
                program: vec![
                    VmInstruction::LoadConst {
                        dst: 0,
                        const_idx: 0,
                    },
                    VmInstruction::Not { dst: 1, src: 0 },
                    VmInstruction::Halt,
                ],
            },
            vec![],
            zero_upstream,
            zero_mem,
        ),
        (
            "ToI32",
            VmBackendDef {
                register_count: 2,
                constants: vec![2.6],
                program: vec![
                    VmInstruction::LoadConst {
                        dst: 0,
                        const_idx: 0,
                    },
                    VmInstruction::ToI32 { dst: 1, src: 0 },
                    VmInstruction::Halt,
                ],
            },
            vec![],
            zero_upstream,
            zero_mem,
        ),
        (
            "ToU8",
            VmBackendDef {
                register_count: 2,
                constants: vec![300.0],
                program: vec![
                    VmInstruction::LoadConst {
                        dst: 0,
                        const_idx: 0,
                    },
                    VmInstruction::ToU8 { dst: 1, src: 0 },
                    VmInstruction::Halt,
                ],
            },
            vec![],
            zero_upstream,
            zero_mem,
        ),
        (
            "ToBool",
            VmBackendDef {
                register_count: 2,
                constants: vec![0.7],
                program: vec![
                    VmInstruction::LoadConst {
                        dst: 0,
                        const_idx: 0,
                    },
                    VmInstruction::ToBool { dst: 1, src: 0 },
                    VmInstruction::Halt,
                ],
            },
            vec![],
            zero_upstream,
            zero_mem,
        ),
        (
            "JumpIfZero",
            VmBackendDef {
                register_count: 1,
                constants: vec![],
                program: vec![
                    VmInstruction::JumpIfZero { cond: 0, offset: 0 },
                    VmInstruction::Halt,
                ],
            },
            vec![],
            zero_upstream,
            zero_mem,
        ),
        (
            "Jump",
            VmBackendDef {
                register_count: 1,
                constants: vec![],
                program: vec![VmInstruction::Jump { offset: 0 }, VmInstruction::Halt],
            },
            vec![],
            zero_upstream,
            zero_mem,
        ),
        (
            "ReadInput",
            VmBackendDef {
                register_count: 1,
                constants: vec![],
                program: vec![
                    VmInstruction::ReadInput {
                        dst: 0,
                        ref_idx: 0,
                        sub_idx: 0,
                    },
                    VmInstruction::Halt,
                ],
            },
            vec![InputReference::UpstreamSlot(3)],
            read_input_upstream,
            zero_mem,
        ),
        (
            "WriteInternalPayload",
            VmBackendDef {
                register_count: 1,
                constants: vec![5.5],
                program: vec![
                    VmInstruction::LoadConst {
                        dst: 0,
                        const_idx: 0,
                    },
                    VmInstruction::WriteInternalPayload {
                        slot_idx: 2,
                        src: 0,
                    },
                    VmInstruction::Halt,
                ],
            },
            vec![],
            zero_upstream,
            zero_mem,
        ),
        (
            "WriteWorldActionMeta",
            VmBackendDef {
                register_count: 1,
                constants: vec![2.0],
                program: vec![
                    VmInstruction::LoadConst {
                        dst: 0,
                        const_idx: 0,
                    },
                    VmInstruction::WriteWorldActionMeta {
                        slot_idx: 0,
                        src: 0,
                    },
                    VmInstruction::PushAction { action_type: 2 },
                    VmInstruction::ExecuteActionQueue,
                ],
            },
            vec![],
            zero_upstream,
            zero_mem,
        ),
        (
            "PushAction",
            VmBackendDef {
                register_count: 1,
                constants: vec![],
                program: vec![
                    VmInstruction::PushAction { action_type: 1 },
                    VmInstruction::ExecuteActionQueue,
                ],
            },
            vec![],
            zero_upstream,
            zero_mem,
        ),
        (
            "PopAction",
            VmBackendDef {
                register_count: 1,
                constants: vec![],
                program: vec![
                    VmInstruction::PushAction { action_type: 0 },
                    VmInstruction::PopAction,
                    VmInstruction::Halt,
                ],
            },
            vec![],
            zero_upstream,
            zero_mem,
        ),
        (
            "ReadActionQueueLength",
            VmBackendDef {
                register_count: 1,
                constants: vec![],
                program: vec![
                    VmInstruction::ReadActionQueueLength { dst: 0 },
                    VmInstruction::Halt,
                ],
            },
            vec![],
            zero_upstream,
            zero_mem,
        ),
        (
            "ReadActionQueueType",
            VmBackendDef {
                register_count: 1,
                constants: vec![],
                program: vec![
                    VmInstruction::ReadActionQueueType {
                        index_src: 0,
                        dst: 0,
                    },
                    VmInstruction::Halt,
                ],
            },
            vec![],
            zero_upstream,
            zero_mem,
        ),
        (
            "ReadActionQueueParam",
            VmBackendDef {
                register_count: 1,
                constants: vec![],
                program: vec![
                    VmInstruction::ReadActionQueueParam {
                        index_src: 0,
                        param_slot: 0,
                        dst: 0,
                    },
                    VmInstruction::Halt,
                ],
            },
            vec![],
            zero_upstream,
            zero_mem,
        ),
        (
            "ExecuteActionQueue",
            VmBackendDef {
                register_count: 1,
                constants: vec![],
                program: vec![VmInstruction::ExecuteActionQueue],
            },
            vec![],
            zero_upstream,
            zero_mem,
        ),
        (
            "WriteRouteTarget",
            VmBackendDef {
                register_count: 1,
                constants: vec![4.25],
                program: vec![
                    VmInstruction::LoadConst {
                        dst: 0,
                        const_idx: 0,
                    },
                    VmInstruction::WriteRouteTarget { src: 0 },
                    VmInstruction::Halt,
                ],
            },
            vec![],
            zero_upstream,
            zero_mem,
        ),
        (
            "Halt",
            VmBackendDef {
                register_count: 1,
                constants: vec![],
                program: vec![VmInstruction::Halt],
            },
            vec![],
            zero_upstream,
            zero_mem,
        ),
        (
            "LoadMem8",
            VmBackendDef {
                register_count: 2,
                constants: vec![5.0],
                program: vec![
                    VmInstruction::LoadConst {
                        dst: 0,
                        const_idx: 0,
                    },
                    VmInstruction::LoadMem8 {
                        dst: 1,
                        addr_reg: 0,
                    },
                    VmInstruction::Halt,
                ],
            },
            vec![],
            zero_upstream,
            mem_for_load,
        ),
        (
            "StoreMem8",
            VmBackendDef {
                register_count: 2,
                constants: vec![5.0, 77.0],
                program: vec![
                    VmInstruction::LoadConst {
                        dst: 0,
                        const_idx: 0,
                    },
                    VmInstruction::LoadConst {
                        dst: 1,
                        const_idx: 1,
                    },
                    VmInstruction::StoreMem8 {
                        addr_reg: 0,
                        src: 1,
                    },
                    VmInstruction::Halt,
                ],
            },
            vec![],
            zero_upstream,
            zero_mem,
        ),
        (
            "LoadMem8Imm",
            VmBackendDef {
                register_count: 1,
                constants: vec![],
                program: vec![
                    VmInstruction::LoadMem8Imm {
                        dst: 0,
                        imm_addr: 9,
                    },
                    VmInstruction::Halt,
                ],
            },
            vec![],
            zero_upstream,
            mem_for_load_imm,
        ),
        (
            "StoreMem8Imm",
            VmBackendDef {
                register_count: 1,
                constants: vec![88.0],
                program: vec![
                    VmInstruction::LoadConst {
                        dst: 0,
                        const_idx: 0,
                    },
                    VmInstruction::StoreMem8Imm {
                        imm_addr: 9,
                        src: 0,
                    },
                    VmInstruction::Halt,
                ],
            },
            vec![],
            zero_upstream,
            zero_mem,
        ),
    ];

    assert_eq!(
        cases.len(),
        38,
        "every VmInstruction opcode must be covered"
    );

    for (name, def, input_refs, upstream, memory_seed) in cases {
        assert_equivalent(name, def, input_refs, upstream, memory_seed);
    }
}
