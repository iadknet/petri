use crate::config::RuntimeConfig;
use crate::contracts::InputReference;
use crate::creature::genome::VmBackendDef;
use crate::runtime::action_decode::decode_world_action;
use crate::runtime::inputs::{resolve_input, ResolveCtx};
use crate::runtime::routing::RouteDecision;
use crate::runtime::types::{sanitize_f32, MeshSideOutputs, NodeResult};
use crate::sensors::perception::SensorSnapshot;

/// Execute a VM backend node.
///
/// # Arguments
/// - `def`: the VM backend genome definition
/// - `input_refs`: the node's InputReference list (from NodeGenome.input_refs)
/// - `upstream_slots`: incoming output slots from the previous node (or zeroed for entry)
/// - `energy`: creature's current energy; decremented by opcode costs; NOT restored on exhaustion
/// - `energy_consumed`: total energy consumed this tick so far (for dynamic introspection)
/// - `shared_memory`: creature's persistent shared memory (16 f32 slots); NOT modified on energy exhaustion
/// - `prev_shared_memory`: snapshot of shared memory from previous tick (read-only)
/// - `sensors`: pre-assembled sensor snapshot (local + extended perception)
/// - `config`: runtime config (max_vm_steps, vm.opcode_cost_multiplier)
/// - `side_outputs`: mesh-scoped side outputs (action queue, priority bid) that persist across hops
///
/// # Returns
/// `NodeResult` — the mesh executor checks `terminal` and `energy_exhausted` to decide routing.
#[allow(clippy::too_many_arguments)]
pub(crate) fn execute_vm_node(
    def: &VmBackendDef,
    input_refs: &[InputReference],
    upstream_slots: &[f32; 12],
    energy: &mut f32,
    energy_consumed: f32,
    shared_memory: &mut [f32; 16],
    prev_shared_memory: &[f32; 16],
    sensors: &SensorSnapshot,
    config: &RuntimeConfig,
    side_outputs: &mut MeshSideOutputs,
) -> NodeResult {
    // Safety: register_count == 0 → immediate halt.
    let reg_count = def.register_count as usize;
    if reg_count == 0 {
        return NodeResult::halted(*upstream_slots, RouteDecision::VmWrap { raw_value: 0.0 });
    }

    // Safety: empty program → immediate halt.
    let program_len = def.program.len();
    if program_len == 0 {
        return NodeResult::halted(*upstream_slots, RouteDecision::VmWrap { raw_value: 0.0 });
    }

    let max_steps = config.max_vm_steps.max(1) as usize;
    let cost_mult = config.vm.opcode_cost_multiplier;

    // Stack-allocated registers covering the full u8 range (256 * 4 = 1KB).
    const MAX_REGS: usize = 256;
    let mut regs = [0.0f32; MAX_REGS];
    let mut payload: [f32; 12] = *upstream_slots;
    let mut meta: [f32; 8] = [0.0; 8];
    let mut route_target: f32 = 0.0;
    let mut pc: usize = 0;
    let mut steps: usize = 0;

    // Working copy of shared memory slots (64 bytes — unconditional copy).
    let mut slot_copy: [f32; 16] = *shared_memory;

    use crate::creature::genome::VmInstruction;

    /// Commit the working slot copy back to the creature's shared memory.
    macro_rules! commit_slots {
        () => {
            *shared_memory = slot_copy;
        };
    }

    loop {
        if steps >= max_steps {
            commit_slots!();
            return NodeResult::halted(
                payload,
                RouteDecision::VmWrap {
                    raw_value: route_target,
                },
            );
        }

        // Soft default: if control flow lands outside the program, halt cleanly.
        if pc >= program_len {
            commit_slots!();
            return NodeResult::halted(
                payload,
                RouteDecision::VmWrap {
                    raw_value: route_target,
                },
            );
        }

        let instr = &def.program[pc];
        let opcode_cost = opcode_base_cost(instr) * cost_mult;

        // Deduct energy before executing; exhaustion halts without side effects.
        *energy -= opcode_cost;
        if *energy <= 0.0 {
            // Do NOT commit memory.
            return NodeResult::exhausted();
        }

        steps += 1;
        let mut next_pc = pc + 1;

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
                // eps register value clamped to [1e-6, 1.0]
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
                // Round ties-away-from-zero, store as f32.
                let val = regs[nr(*src, reg_count)].round();
                regs[nr(*dst, reg_count)] = sanitize_f32(val);
            }

            VmInstruction::ToU8 { dst, src } => {
                // Clamp [0, 255], round, store as f32.
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

            VmInstruction::ReadInput {
                dst,
                ref_idx,
                sub_idx,
            } => {
                let val = if (*ref_idx as usize) < input_refs.len() {
                    let ctx = ResolveCtx {
                        sensors,
                        upstream_slots,
                        energy: *energy,
                        energy_consumed,
                        action_queue: &side_outputs.action_queue,
                    };
                    resolve_input(&input_refs[*ref_idx as usize], *sub_idx, &ctx)
                } else {
                    0.0 // soft default: out-of-range ref_idx
                };
                regs[nr(*dst, reg_count)] = sanitize_f32(val);
            }

            VmInstruction::WriteInternalPayload { slot_idx, src } => {
                if (*slot_idx as usize) < 12 {
                    payload[*slot_idx as usize] = regs[nr(*src, reg_count)];
                }
                // invalid slot: write ignored
            }

            VmInstruction::WriteWorldActionMeta { slot_idx, src } => {
                if (*slot_idx as usize) < 8 {
                    meta[*slot_idx as usize] = regs[nr(*src, reg_count)];
                }
                // invalid slot: write ignored
            }

            VmInstruction::PushAction { action_type } => {
                let action = decode_world_action(*action_type, &meta);
                side_outputs.action_queue.push(action);
            }

            VmInstruction::PopAction => {
                side_outputs.action_queue.pop();
            }

            VmInstruction::ReadActionQueueLength { dst } => {
                regs[nr(*dst, reg_count)] = side_outputs.action_queue.len() as f32;
            }

            VmInstruction::ReadActionQueueType { index_src, dst } => {
                let idx = regs[nr(*index_src, reg_count)] as usize;
                regs[nr(*dst, reg_count)] = side_outputs.action_queue.action_type_at(idx);
            }

            VmInstruction::ReadActionQueueParam {
                index_src,
                param_slot,
                dst,
            } => {
                let idx = regs[nr(*index_src, reg_count)] as usize;
                regs[nr(*dst, reg_count)] = side_outputs
                    .action_queue
                    .param_at(idx, *param_slot as usize);
            }

            VmInstruction::SetPriorityBid { src } => {
                let raw = regs[nr(*src, reg_count)];
                let bid = if raw > 0.0 { raw.min(*energy) } else { 0.0 };
                *energy -= bid;
                if *energy <= 0.0 {
                    return NodeResult::exhausted();
                }
                side_outputs.priority_bid = bid;
            }

            VmInstruction::ExecuteActionQueue => {
                commit_slots!();
                return NodeResult::terminal(
                    payload,
                    RouteDecision::VmWrap {
                        raw_value: route_target,
                    },
                );
            }

            VmInstruction::WriteRouteTarget { src } => {
                route_target = regs[nr(*src, reg_count)]; // last-write-wins
            }

            VmInstruction::Halt => {
                commit_slots!();
                return NodeResult::halted(
                    payload,
                    RouteDecision::VmWrap {
                        raw_value: route_target,
                    },
                );
            }

            VmInstruction::LoadSlot { dst, slot_reg } => {
                let idx = (regs[nr(*slot_reg, reg_count)] as i64).rem_euclid(16) as usize;
                regs[nr(*dst, reg_count)] = sanitize_f32(slot_copy[idx]);
            }

            VmInstruction::StoreSlot { slot_reg, src } => {
                let idx = (regs[nr(*slot_reg, reg_count)] as i64).rem_euclid(16) as usize;
                slot_copy[idx] = sanitize_f32(regs[nr(*src, reg_count)]);
            }

            VmInstruction::LoadSlotImm { dst, slot_idx } => {
                let idx = (*slot_idx as usize) % 16;
                regs[nr(*dst, reg_count)] = sanitize_f32(slot_copy[idx]);
            }

            VmInstruction::StoreSlotImm { slot_idx, src } => {
                let idx = (*slot_idx as usize) % 16;
                slot_copy[idx] = sanitize_f32(regs[nr(*src, reg_count)]);
            }

            VmInstruction::LoadSlotPrev { dst, slot_idx } => {
                let idx = (*slot_idx as usize) % 16;
                regs[nr(*dst, reg_count)] = sanitize_f32(prev_shared_memory[idx]);
            }

            VmInstruction::ClearSlot { slot_idx } => {
                let idx = (*slot_idx as usize) % 16;
                slot_copy[idx] = 0.0;
            }
        }

        pc = next_pc;
    }
}

/// Normalize a register index via rem_euclid wrapping.
#[inline]
pub(crate) fn nr(idx: u8, reg_count: usize) -> usize {
    (idx as usize).rem_euclid(reg_count)
}

/// Boolean truthiness: value >= 0.5.
#[inline]
pub(crate) fn is_truthy(v: f32) -> bool {
    v >= 0.5
}

/// Compute jump target PC with rem_euclid wrapping.
/// `offset` is signed relative to the instruction AFTER the jump.
#[inline]
pub(crate) fn jump_target(pc: usize, offset: i32, program_len: usize) -> usize {
    let provisional = pc as i64 + 1 + offset as i64;
    provisional.rem_euclid(program_len as i64) as usize
}

/// Base energy cost per opcode per v3-vm-isa-spec.md Section 6.
pub(crate) fn opcode_base_cost(instr: &crate::creature::genome::VmInstruction) -> f32 {
    use crate::creature::genome::VmInstruction;
    match instr {
        VmInstruction::Noop => 0.05,
        VmInstruction::LoadConst { .. } => 0.08,
        VmInstruction::Move { .. } => 0.08,
        VmInstruction::Add { .. } => 0.12,
        VmInstruction::Sub { .. } => 0.12,
        VmInstruction::Mul { .. } => 0.12,
        VmInstruction::Div { .. } => 0.16,
        VmInstruction::Min { .. } => 0.12,
        VmInstruction::Max { .. } => 0.12,
        VmInstruction::Abs { .. } => 0.10,
        VmInstruction::Neg { .. } => 0.10,
        VmInstruction::Clamp01 { .. } => 0.10,
        VmInstruction::CmpGt { .. } => 0.12,
        VmInstruction::CmpLt { .. } => 0.12,
        VmInstruction::CmpEq { .. } => 0.12,
        VmInstruction::And { .. } => 0.12,
        VmInstruction::Or { .. } => 0.12,
        VmInstruction::Not { .. } => 0.10,
        VmInstruction::ToI32 { .. } => 0.10,
        VmInstruction::ToU8 { .. } => 0.10,
        VmInstruction::ToBool { .. } => 0.10,
        VmInstruction::JumpIfZero { .. } => 0.14,
        VmInstruction::Jump { .. } => 0.10,
        VmInstruction::ReadInput { .. } => 0.12,
        VmInstruction::WriteInternalPayload { .. } => 0.14,
        VmInstruction::WriteWorldActionMeta { .. } => 0.14,
        VmInstruction::PushAction { .. } => 0.24,
        VmInstruction::PopAction => 0.10,
        VmInstruction::ReadActionQueueLength { .. } => 0.08,
        VmInstruction::ReadActionQueueType { .. } => 0.12,
        VmInstruction::ReadActionQueueParam { .. } => 0.12,
        VmInstruction::SetPriorityBid { .. } => 0.20,
        VmInstruction::ExecuteActionQueue => 0.24,
        VmInstruction::WriteRouteTarget { .. } => 0.10,
        VmInstruction::Halt => 0.05,
        VmInstruction::LoadSlot { .. } => 0.12,
        VmInstruction::StoreSlot { .. } => 0.14,
        VmInstruction::LoadSlotImm { .. } => 0.10,
        VmInstruction::StoreSlotImm { .. } => 0.12,
        VmInstruction::LoadSlotPrev { .. } => 0.10,
        VmInstruction::ClearSlot { .. } => 0.12,
    }
}

#[cfg(test)]
#[path = "tests/vm_tests.rs"]
mod tests;
