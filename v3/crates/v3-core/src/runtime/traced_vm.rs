//! Traced VM execution — identical logic to [`super::vm::execute_vm_node`] but
//! records per-instruction trace data for the Execution Sampler.
//!
//! **Maintenance note:** This module intentionally duplicates the instruction loop
//! from `vm.rs` with interleaved trace recording. When updating `vm.rs` execution
//! semantics, apply the same changes here and verify with equivalence tests.

use crate::config::RuntimeConfig;
use crate::contracts::InputReference;
use crate::creature::genome::VmBackendDef;
use crate::runtime::action_decode::decode_world_action;
use crate::runtime::inputs::{resolve_input, ResolveCtx};
use crate::runtime::trace::{MemoryWrite, VmStepTrace, VmTrace};
use crate::runtime::types::{sanitize_f32, NodeResult};
use crate::runtime::vm::{is_truthy, jump_target, nr, opcode_base_cost};
use crate::sensors::static_inputs::StaticInputs;

/// Execute a VM backend node with trace recording.
///
/// Identical behavior to [`super::vm::execute_vm_node`] but additionally
/// returns a [`VmTrace`] capturing per-instruction register changes,
/// memory writes, and final state.
#[allow(clippy::too_many_arguments)]
pub fn execute_vm_node_traced(
    def: &VmBackendDef,
    input_refs: &[InputReference],
    upstream_slots: &[f32; 12],
    energy: &mut f32,
    energy_consumed: f32,
    memory: &mut [u8; 1024],
    static_inputs: &StaticInputs,
    config: &RuntimeConfig,
) -> (NodeResult, VmTrace) {
    let reg_count = def.register_count as usize;

    let empty_trace = || VmTrace {
        register_count: def.register_count,
        constants: def.constants.clone(),
        steps: Vec::new(),
        final_registers: Vec::new(),
        final_payload: *upstream_slots,
        final_meta: [0.0; 8],
        final_route_target: 0.0,
        memory_writes: Vec::new(),
    };

    if reg_count == 0 {
        return (NodeResult::halted(*upstream_slots, 0.0), empty_trace());
    }

    let program_len = def.program.len();
    if program_len == 0 {
        return (NodeResult::halted(*upstream_slots, 0.0), empty_trace());
    }

    let max_steps = config.max_vm_steps.max(1) as usize;
    let cost_mult = config.vm.opcode_cost_multiplier;

    const MAX_REGS: usize = 256;
    let mut regs = [0.0f32; MAX_REGS];
    let mut payload: [f32; 12] = *upstream_slots;
    let mut meta: [f32; 8] = [0.0; 8];
    let mut route_target: f32 = 0.0;
    let mut pc: usize = 0;
    let mut steps_count: usize = 0;

    let uses_mem = def.has_memory_ops();
    let mut mem_copy: [u8; 1024] = if uses_mem { *memory } else { [0u8; 1024] };

    // Trace recording state
    let mut trace_steps: Vec<VmStepTrace> = Vec::with_capacity(program_len.min(max_steps));
    let mut mem_writes: Vec<MemoryWrite> = Vec::new();

    use crate::creature::genome::VmInstruction;

    macro_rules! commit_memory {
        () => {
            if uses_mem {
                *memory = mem_copy;
            }
        };
    }

    /// Collect register indices that an instruction writes to.
    macro_rules! written_regs {
        ($instr:expr, $reg_count:expr) => {
            match $instr {
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
                | VmInstruction::LoadMem8 { dst, .. }
                | VmInstruction::LoadMem8Imm { dst, .. } => Some(nr(*dst, $reg_count)),
                _ => None,
            }
        };
    }

    // Macro to build VmTrace at exit points, avoiding repeated struct construction.
    // Only one exit point executes per call, so the single clone is the actual cost.
    macro_rules! build_trace {
        ($steps:expr, $mem_writes:expr) => {
            VmTrace {
                register_count: def.register_count,
                constants: def.constants.clone(),
                steps: $steps,
                final_registers: regs[..reg_count].to_vec(),
                final_payload: payload,
                final_meta: meta,
                final_route_target: route_target,
                memory_writes: $mem_writes,
            }
        };
    }

    loop {
        // Match vm.rs loop structure: check max_steps and program bounds separately.
        if steps_count >= max_steps {
            commit_memory!();
            return (
                NodeResult::halted(payload, route_target),
                build_trace!(trace_steps, mem_writes),
            );
        }

        if pc >= program_len {
            commit_memory!();
            return (
                NodeResult::halted(payload, route_target),
                build_trace!(trace_steps, mem_writes),
            );
        }

        let instr = &def.program[pc];
        let opcode_cost = opcode_base_cost(instr) * cost_mult;

        // Snapshot register that will be written (before execution)
        let target_reg_idx = written_regs!(instr, reg_count);
        let reg_before = target_reg_idx.map(|i| regs[i]);

        *energy -= opcode_cost;
        if *energy <= 0.0 {
            // Record the step that caused exhaustion
            trace_steps.push(VmStepTrace {
                pc,
                instruction: instr.clone(),
                energy_cost: opcode_cost,
                energy_after: *energy,
                register_changes: Vec::new(),
            });
            return (
                NodeResult::exhausted(),
                build_trace!(trace_steps, mem_writes),
            );
        }

        steps_count += 1;
        let mut next_pc = pc + 1;

        // ── Execute instruction (identical to vm.rs) ──
        match instr {
            VmInstruction::Noop => {}

            VmInstruction::LoadConst { dst, const_idx } => {
                let val = if def.constants.is_empty() {
                    0.0
                } else {
                    def.constants[(*const_idx as usize).rem_euclid(def.constants.len())]
                };
                regs[nr(*dst, reg_count)] = sanitize_f32(val);
            }

            VmInstruction::Move { dst, src } => {
                let val = regs[nr(*src, reg_count)];
                regs[nr(*dst, reg_count)] = sanitize_f32(val);
            }

            VmInstruction::Add { dst, a, b } => {
                regs[nr(*dst, reg_count)] =
                    sanitize_f32(regs[nr(*a, reg_count)] + regs[nr(*b, reg_count)]);
            }

            VmInstruction::Sub { dst, a, b } => {
                regs[nr(*dst, reg_count)] =
                    sanitize_f32(regs[nr(*a, reg_count)] - regs[nr(*b, reg_count)]);
            }

            VmInstruction::Mul { dst, a, b } => {
                regs[nr(*dst, reg_count)] =
                    sanitize_f32(regs[nr(*a, reg_count)] * regs[nr(*b, reg_count)]);
            }

            VmInstruction::Div { dst, a, b } => {
                let divisor = regs[nr(*b, reg_count)];
                let val = if divisor == 0.0 {
                    0.0
                } else {
                    regs[nr(*a, reg_count)] / divisor
                };
                regs[nr(*dst, reg_count)] = sanitize_f32(val);
            }

            VmInstruction::Min { dst, a, b } => {
                regs[nr(*dst, reg_count)] =
                    sanitize_f32(regs[nr(*a, reg_count)].min(regs[nr(*b, reg_count)]));
            }

            VmInstruction::Max { dst, a, b } => {
                regs[nr(*dst, reg_count)] =
                    sanitize_f32(regs[nr(*a, reg_count)].max(regs[nr(*b, reg_count)]));
            }

            VmInstruction::Abs { dst, src } => {
                regs[nr(*dst, reg_count)] = sanitize_f32(regs[nr(*src, reg_count)].abs());
            }

            VmInstruction::Neg { dst, src } => {
                regs[nr(*dst, reg_count)] = sanitize_f32(-regs[nr(*src, reg_count)]);
            }

            VmInstruction::Clamp01 { dst, src } => {
                regs[nr(*dst, reg_count)] = sanitize_f32(regs[nr(*src, reg_count)].clamp(0.0, 1.0));
            }

            VmInstruction::CmpGt { dst, a, b } => {
                regs[nr(*dst, reg_count)] = if regs[nr(*a, reg_count)] > regs[nr(*b, reg_count)] {
                    1.0
                } else {
                    0.0
                };
            }

            VmInstruction::CmpLt { dst, a, b } => {
                regs[nr(*dst, reg_count)] = if regs[nr(*a, reg_count)] < regs[nr(*b, reg_count)] {
                    1.0
                } else {
                    0.0
                };
            }

            VmInstruction::CmpEq { dst, a, b, eps } => {
                let eps_val = regs[nr(*eps, reg_count)].clamp(1e-6, 1.0);
                let diff = (regs[nr(*a, reg_count)] - regs[nr(*b, reg_count)]).abs();
                regs[nr(*dst, reg_count)] = if diff <= eps_val { 1.0 } else { 0.0 };
            }

            VmInstruction::And { dst, a, b } => {
                let ta = is_truthy(regs[nr(*a, reg_count)]);
                let tb = is_truthy(regs[nr(*b, reg_count)]);
                regs[nr(*dst, reg_count)] = if ta && tb { 1.0 } else { 0.0 };
            }

            VmInstruction::Or { dst, a, b } => {
                let ta = is_truthy(regs[nr(*a, reg_count)]);
                let tb = is_truthy(regs[nr(*b, reg_count)]);
                regs[nr(*dst, reg_count)] = if ta || tb { 1.0 } else { 0.0 };
            }

            VmInstruction::Not { dst, src } => {
                regs[nr(*dst, reg_count)] = if is_truthy(regs[nr(*src, reg_count)]) {
                    0.0
                } else {
                    1.0
                };
            }

            VmInstruction::ToI32 { dst, src } => {
                let val = regs[nr(*src, reg_count)].round();
                regs[nr(*dst, reg_count)] = sanitize_f32(val);
            }

            VmInstruction::ToU8 { dst, src } => {
                let clamped = regs[nr(*src, reg_count)].clamp(0.0, 255.0);
                regs[nr(*dst, reg_count)] = sanitize_f32(clamped.round());
            }

            VmInstruction::ToBool { dst, src } => {
                regs[nr(*dst, reg_count)] = if is_truthy(regs[nr(*src, reg_count)]) {
                    1.0
                } else {
                    0.0
                };
            }

            VmInstruction::JumpIfZero { cond, offset } => {
                if !is_truthy(regs[nr(*cond, reg_count)]) {
                    next_pc = jump_target(pc, *offset, program_len);
                }
            }

            VmInstruction::Jump { offset } => {
                next_pc = jump_target(pc, *offset, program_len);
            }

            VmInstruction::ReadInput { dst, input_idx } => {
                let val = if (*input_idx as usize) < input_refs.len() {
                    let ctx = ResolveCtx {
                        static_inputs,
                        upstream_slots,
                        energy: *energy,
                        energy_consumed,
                    };
                    resolve_input(&input_refs[*input_idx as usize], 0, &ctx)
                } else {
                    0.0
                };
                regs[nr(*dst, reg_count)] = sanitize_f32(val);
            }

            VmInstruction::WriteInternalPayload { slot_idx, src } => {
                if (*slot_idx as usize) < 12 {
                    payload[*slot_idx as usize] = regs[nr(*src, reg_count)];
                }
            }

            VmInstruction::WriteWorldActionMeta { slot_idx, src } => {
                if (*slot_idx as usize) < 8 {
                    meta[*slot_idx as usize] = regs[nr(*src, reg_count)];
                }
            }

            VmInstruction::EmitWorldAction { action_type } => {
                let action = decode_world_action(*action_type, &meta);
                commit_memory!();
                // Record this final step
                trace_steps.push(VmStepTrace {
                    pc,
                    instruction: instr.clone(),
                    energy_cost: opcode_cost,
                    energy_after: *energy,
                    register_changes: Vec::new(),
                });
                return (
                    NodeResult::action(payload, route_target, action),
                    build_trace!(trace_steps, mem_writes),
                );
            }

            VmInstruction::WriteRouteTarget { src } => {
                route_target = regs[nr(*src, reg_count)];
            }

            VmInstruction::Halt => {
                commit_memory!();
                trace_steps.push(VmStepTrace {
                    pc,
                    instruction: instr.clone(),
                    energy_cost: opcode_cost,
                    energy_after: *energy,
                    register_changes: Vec::new(),
                });
                return (
                    NodeResult::halted(payload, route_target),
                    build_trace!(trace_steps, mem_writes),
                );
            }

            VmInstruction::LoadMem8 { dst, addr_reg } => {
                let addr = (regs[nr(*addr_reg, reg_count)] as i64).rem_euclid(1024) as usize;
                regs[nr(*dst, reg_count)] = sanitize_f32(mem_copy[addr] as f32);
            }

            VmInstruction::StoreMem8 { addr_reg, src } => {
                let addr = (regs[nr(*addr_reg, reg_count)] as i64).rem_euclid(1024) as usize;
                let old_val = mem_copy[addr];
                let val = regs[nr(*src, reg_count)].clamp(0.0, 255.0) as u8;
                mem_copy[addr] = val;
                if old_val != val {
                    mem_writes.push(MemoryWrite {
                        address: addr as u16,
                        old_value: old_val,
                        new_value: val,
                    });
                }
            }

            VmInstruction::LoadMem8Imm { dst, imm_addr } => {
                let addr = (*imm_addr as usize).rem_euclid(1024);
                regs[nr(*dst, reg_count)] = sanitize_f32(mem_copy[addr] as f32);
            }

            VmInstruction::StoreMem8Imm { imm_addr, src } => {
                let addr = (*imm_addr as usize).rem_euclid(1024);
                let old_val = mem_copy[addr];
                let val = regs[nr(*src, reg_count)].clamp(0.0, 255.0) as u8;
                mem_copy[addr] = val;
                if old_val != val {
                    mem_writes.push(MemoryWrite {
                        address: addr as u16,
                        old_value: old_val,
                        new_value: val,
                    });
                }
            }
        }

        // Record register changes (diff-only)
        let mut register_changes = Vec::new();
        if let Some(reg_idx) = target_reg_idx {
            let new_val = regs[reg_idx];
            if let Some(old_val) = reg_before {
                if (new_val - old_val).abs() > f32::EPSILON {
                    register_changes.push((reg_idx as u8, new_val));
                }
            } else {
                // No previous value means first write
                if new_val != 0.0 {
                    register_changes.push((reg_idx as u8, new_val));
                }
            }
        }

        // Don't re-record EmitWorldAction/Halt (already recorded above)
        if !matches!(
            instr,
            VmInstruction::EmitWorldAction { .. } | VmInstruction::Halt
        ) {
            trace_steps.push(VmStepTrace {
                pc,
                instruction: instr.clone(),
                energy_cost: opcode_cost,
                energy_after: *energy,
                register_changes,
            });
        }

        pc = next_pc;
    }
}

#[cfg(test)]
#[path = "tests/traced_vm_tests.rs"]
mod tests;
