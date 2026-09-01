use super::*;
use crate::config::RuntimeConfig;
use crate::contracts::Direction;
use crate::creature::genome::{VmBackendDef, VmInstruction};
use crate::runtime::types::{MeshSideOutputs, OUTPUT_SLOT_COUNT};
use crate::sensors::perception::{PerceptionSnapshot, SensorSnapshot};
use crate::sensors::static_inputs::StaticInputs;
use crate::sensors::typed_food::TypedFoodLocalSnapshot;

fn config() -> RuntimeConfig {
    RuntimeConfig::default()
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

fn zeroed_upstream() -> [f32; OUTPUT_SLOT_COUNT] {
    [0.0; OUTPUT_SLOT_COUNT]
}

fn run_vm(
    program: Vec<VmInstruction>,
    register_count: u8,
    constants: Vec<f32>,
    input_refs: &[InputReference],
    upstream: [f32; OUTPUT_SLOT_COUNT],
    energy: f32,
) -> (NodeResult, f32, MeshSideOutputs) {
    let def = VmBackendDef {
        register_count,
        constants,
        program,
    };
    let ss = empty_sensor_snapshot();
    let mut e = energy;
    let mut mem = [0.0f32; 16];
    let prev_mem = [0.0f32; 16];
    let cfg = config();
    let mut side_outputs = MeshSideOutputs::new(cfg.max_actions_per_turn);
    let result = execute_vm_node(
        &def,
        input_refs,
        &upstream,
        &mut e,
        0.0,
        0.0,
        &mut mem,
        &prev_mem,
        &ss,
        &cfg,
        &mut side_outputs,
    );
    (result, e, side_outputs)
}

#[path = "vm_opcodes.rs"]
mod opcodes;

#[path = "vm_execution.rs"]
mod execution;

#[path = "vm_io_memory.rs"]
mod io_memory;
