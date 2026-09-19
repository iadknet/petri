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
            max_energy: 200.0,
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
    run_vm_with_config(
        program,
        register_count,
        constants,
        input_refs,
        upstream,
        energy,
        config(),
    )
}

fn run_vm_with_config(
    program: Vec<VmInstruction>,
    register_count: u8,
    constants: Vec<f32>,
    input_refs: &[InputReference],
    upstream: [f32; OUTPUT_SLOT_COUNT],
    energy: f32,
    cfg: RuntimeConfig,
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
    let mut side_outputs = MeshSideOutputs::new(cfg.max_actions_per_turn);
    let result = execute_vm_node(
        &def,
        input_refs,
        &upstream,
        &mut e,
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

#[path = "vm_step_ramp.rs"]
mod step_ramp;

#[test]
fn applied_vm_settlement_observation_survives_every_exit() {
    let mut cfg = config();
    cfg.max_vm_steps = 2;
    cfg.vm.opcode_cost_multiplier = 1.0;
    cfg.vm.step_ramp_cost = 0.0;
    for (program, registers) in [
        (vec![], 1),
        (vec![VmInstruction::Halt], 0),
        (vec![VmInstruction::Halt], 1),
        (vec![VmInstruction::Noop], 1),
        (vec![VmInstruction::ExecuteActionQueue], 1),
        (vec![VmInstruction::Jump { offset: -1 }], 1),
    ] {
        let (_, energy, side) = run_vm_with_config(
            program.clone(),
            registers,
            vec![],
            &[],
            zeroed_upstream(),
            100.0,
            cfg.clone(),
        );
        assert_eq!(
            side.energy_observation.vm_compute,
            100.0 - f64::from(energy),
            "{program:?}, {registers} registers"
        );
        assert_eq!(side.energy_observation.priority_bid, 0.0);
        assert_eq!(side.energy_observation.pending_cause, None);
    }
}

#[test]
fn applied_vm_exhaustion_distinguishes_unexecuted_bid_all_in_and_settlement_rounding() {
    use crate::simulation::energy_accounting::DeathCause;
    let mut cfg = config();
    cfg.vm.opcode_cost_multiplier = 1.0;
    cfg.vm.step_ramp_cost = 0.0;
    let (result, energy, side) = run_vm_with_config(
        vec![VmInstruction::SetPriorityBid { src: 0 }],
        1,
        vec![],
        &[],
        zeroed_upstream(),
        0.1,
        cfg.clone(),
    );
    assert!(result.energy_exhausted);
    assert_eq!(
        side.energy_observation.pending_cause,
        Some(DeathCause::VmCompute)
    );
    assert_eq!(side.energy_observation.priority_bid, 0.0);
    assert_eq!(
        side.energy_observation.vm_compute,
        f64::from(0.1f32) - f64::from(energy)
    );

    let (result, energy, side) = run_vm_with_config(
        vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            },
            VmInstruction::SetPriorityBid { src: 0 },
        ],
        1,
        vec![100.0],
        &[],
        zeroed_upstream(),
        10.0,
        cfg.clone(),
    );
    assert!(result.energy_exhausted);
    assert_eq!(energy, 0.0);
    assert_eq!(
        side.energy_observation.pending_cause,
        Some(DeathCause::PriorityBid)
    );
    let debt = (f64::from(0.08f32) + f64::from(0.20f32)) as f32;
    assert_eq!(side.energy_observation.priority_bid, 10.0 - f64::from(debt));
    assert_eq!(side.energy_observation.vm_compute, f64::from(debt));
    assert_eq!(
        side.priority_bid, 0.0,
        "exhausting bid never publishes a priority"
    );

    cfg.vm.opcode_cost_multiplier = 3.0;
    let (result, energy, side) = run_vm_with_config(
        vec![VmInstruction::Noop],
        1,
        vec![],
        &[],
        zeroed_upstream(),
        0.15,
        cfg,
    );
    assert!(
        !result.energy_exhausted,
        "positive effective remainder keeps the original exit"
    );
    assert_eq!(
        energy, 0.0,
        "settlement rounds the effective remainder to zero"
    );
    assert_eq!(
        side.energy_observation.pending_cause,
        Some(DeathCause::VmCompute)
    );
}

#[test]
fn applied_vm_accounting_counts_overwritten_bids_and_separate_settlement() {
    let mut cfg = config();
    cfg.vm.opcode_cost_multiplier = 0.0;
    cfg.vm.step_ramp_cost = 0.0;
    let (_, energy, side) = run_vm_with_config(
        vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            },
            VmInstruction::SetPriorityBid { src: 0 },
            VmInstruction::SetPriorityBid { src: 0 },
        ],
        1,
        vec![2.0],
        &[],
        zeroed_upstream(),
        10.0,
        cfg,
    );
    assert_eq!(energy, 6.0);
    assert_eq!(side.energy_observation.priority_bid, 4.0);
    assert_eq!(side.energy_observation.vm_compute, 0.0);
    assert_eq!(side.priority_bid, 2.0);
}
