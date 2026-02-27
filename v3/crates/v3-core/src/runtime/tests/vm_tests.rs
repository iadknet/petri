use super::*;
use crate::config::RuntimeConfig;
use crate::contracts::Direction;
use crate::creature::genome::{VmBackendDef, VmInstruction};
use crate::sensors::static_inputs::StaticInputs;

fn config() -> RuntimeConfig {
    RuntimeConfig::default()
}

fn empty_static_inputs() -> StaticInputs {
    StaticInputs {
        food_here: 0.0,
        neighbor_food: [0.0; 8],
        neighbor_barrier: [0.0; 8],
        neighbor_occupied: [0.0; 8],
        generation: 0.0,
        age_ticks: 0.0,
    }
}

fn zeroed_upstream() -> [f32; 12] {
    [0.0; 12]
}

fn run_vm(
    program: Vec<VmInstruction>,
    register_count: u8,
    constants: Vec<f32>,
    input_refs: &[InputReference],
    upstream: [f32; 12],
    energy: f32,
) -> (NodeResult, f32) {
    let def = VmBackendDef {
        register_count,
        constants,
        program,
    };
    let si = empty_static_inputs();
    let mut e = energy;
    let mut mem = [0u8; 1024];
    let result = execute_vm_node(
        &def,
        input_refs,
        &upstream,
        &mut e,
        0.0,
        &mut mem,
        &si,
        &config(),
    );
    (result, e)
}

#[path = "vm_opcodes.rs"]
mod opcodes;

#[path = "vm_execution.rs"]
mod execution;

#[path = "vm_io_memory.rs"]
mod io_memory;
