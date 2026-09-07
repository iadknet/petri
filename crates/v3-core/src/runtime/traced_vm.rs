//! Trace recording adapter for the shared VM execution loop.

use crate::config::RuntimeConfig;
use crate::contracts::InputReference;
use crate::creature::genome::{VmBackendDef, VmInstruction};
use crate::runtime::trace::domain::{SlotWrite, VmStepTrace, VmTrace};
use crate::runtime::types::{MeshSideOutputs, NodeResult, OUTPUT_SLOT_COUNT};
use crate::runtime::vm::{execute_vm_node_impl, nr, VmTraceSink};
use crate::sensors::perception::SensorSnapshot;

/// Execute a VM backend node with trace recording.
#[allow(clippy::too_many_arguments)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn execute_vm_node_traced(
    def: &VmBackendDef,
    input_refs: &[InputReference],
    upstream_slots: &[f32; OUTPUT_SLOT_COUNT],
    energy: &mut f32,
    energy_consumed: f32,
    shared_memory: &mut [f32; 16],
    prev_shared_memory: &[f32; 16],
    sensors: &SensorSnapshot,
    config: &RuntimeConfig,
    side_outputs: &mut MeshSideOutputs,
) -> (NodeResult, VmTrace) {
    execute_vm_node_impl(
        def,
        input_refs,
        upstream_slots,
        energy,
        energy_consumed,
        shared_memory,
        prev_shared_memory,
        sensors,
        config,
        side_outputs,
        RecordingVmTraceSink::new(def, config),
    )
}

struct RecordingVmTraceSink {
    steps: Vec<VmStepTrace>,
    slot_writes: Vec<SlotWrite>,
    pending_register: Option<(usize, f32)>,
}

impl RecordingVmTraceSink {
    fn new(def: &VmBackendDef, config: &RuntimeConfig) -> Self {
        let capacity = def.program.len().min(config.max_vm_steps.max(1) as usize);
        Self {
            steps: Vec::with_capacity(capacity),
            slot_writes: Vec::new(),
            pending_register: None,
        }
    }
}

impl VmTraceSink for RecordingVmTraceSink {
    type Output = VmTrace;

    fn finish_empty(
        self,
        def: &VmBackendDef,
        upstream_slots: &[f32; OUTPUT_SLOT_COUNT],
    ) -> VmTrace {
        VmTrace {
            register_count: def.register_count,
            constants: def.constants.clone(),
            steps: self.steps,
            final_registers: Vec::new(),
            final_payload: *upstream_slots,
            final_meta: [0.0; 8],
            slot_writes: self.slot_writes,
        }
    }

    fn before_instruction(&mut self, _pc: usize, instruction: &VmInstruction, registers: &[f32]) {
        self.pending_register =
            written_register(instruction, registers.len()).map(|index| (index, registers[index]));
    }

    fn record_slot_write(&mut self, slot_idx: usize, old_value: f32, new_value: f32) {
        if (old_value - new_value).abs() > f32::EPSILON {
            self.slot_writes.push(SlotWrite {
                slot_idx: slot_idx as u8,
                old_value,
                new_value,
            });
        }
    }

    fn after_instruction(
        &mut self,
        pc: usize,
        instruction: &VmInstruction,
        energy_cost: f32,
        energy_after: f32,
        registers: &[f32],
    ) {
        let register_changes = self
            .pending_register
            .take()
            .and_then(|(index, old_value)| {
                let new_value = registers[index];
                ((new_value - old_value).abs() > f32::EPSILON)
                    .then_some(vec![(index as u8, new_value)])
            })
            .unwrap_or_default();

        self.steps.push(VmStepTrace {
            pc,
            instruction: instruction.clone(),
            energy_cost,
            energy_after,
            register_changes,
        });
    }

    fn finish(
        self,
        def: &VmBackendDef,
        registers: &[f32],
        payload: [f32; OUTPUT_SLOT_COUNT],
        meta: [f32; 8],
    ) -> VmTrace {
        VmTrace {
            register_count: def.register_count,
            constants: def.constants.clone(),
            steps: self.steps,
            final_registers: registers.to_vec(),
            final_payload: payload,
            final_meta: meta,
            slot_writes: self.slot_writes,
        }
    }
}

fn written_register(instruction: &VmInstruction, register_count: usize) -> Option<usize> {
    match instruction {
        VmInstruction::LoadConst { dst, .. }
        | VmInstruction::Move { dst, .. }
        | VmInstruction::Add { dst, .. }
        | VmInstruction::Sub { dst, .. }
        | VmInstruction::Mul { dst, .. }
        | VmInstruction::Div { dst, .. }
        | VmInstruction::Min { dst, .. }
        | VmInstruction::Max { dst, .. }
        | VmInstruction::Abs { dst, .. }
        | VmInstruction::Neg { dst, .. }
        | VmInstruction::Clamp01 { dst, .. }
        | VmInstruction::CmpGt { dst, .. }
        | VmInstruction::CmpLt { dst, .. }
        | VmInstruction::CmpEq { dst, .. }
        | VmInstruction::And { dst, .. }
        | VmInstruction::Or { dst, .. }
        | VmInstruction::Not { dst, .. }
        | VmInstruction::ToI32 { dst, .. }
        | VmInstruction::ToU8 { dst, .. }
        | VmInstruction::ToBool { dst, .. }
        | VmInstruction::ReadInput { dst, .. }
        | VmInstruction::ReadActionQueueLength { dst, .. }
        | VmInstruction::ReadActionQueueType { dst, .. }
        | VmInstruction::ReadActionQueueParam { dst, .. }
        | VmInstruction::LoadSlot { dst, .. }
        | VmInstruction::LoadSlotImm { dst, .. }
        | VmInstruction::LoadSlotPrev { dst, .. } => Some(nr(*dst, register_count)),
        _ => None,
    }
}

#[cfg(test)]
#[path = "tests/traced_vm_tests.rs"]
#[allow(
    clippy::too_many_lines,
    reason = "the opcode equivalence test enumerates all 41 VM instructions in \
              one table-driven case"
)]
mod tests;
