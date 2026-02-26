use crate::config::RuntimeConfig;
use crate::contracts::InputReference;
use crate::creature::genome::VmBackendDef;
use crate::runtime::action_decode::decode_world_action;
use crate::runtime::inputs::resolve_input;
use crate::runtime::types::{sanitize_f32, NodeResult};
use crate::sensors::static_inputs::StaticInputs;

/// Execute a VM backend node.
///
/// # Arguments
/// - `def`: the VM backend genome definition
/// - `input_refs`: the node's InputReference list (from NodeGenome.input_refs)
/// - `upstream_slots`: incoming output slots from the previous node (or zeroed for entry)
/// - `energy`: creature's current energy; decremented by opcode costs; NOT restored on exhaustion
/// - `energy_consumed`: total energy consumed this tick so far (for dynamic introspection)
/// - `memory`: creature's persistent 1024-byte memory; NOT modified on energy exhaustion
/// - `static_inputs`: pre-assembled world/static sensor snapshot
/// - `config`: runtime config (max_vm_steps, vm.opcode_cost_multiplier)
///
/// # Returns
/// `NodeResult` — the mesh executor checks `energy_exhausted` and routes accordingly.
#[allow(clippy::too_many_arguments)]
pub fn execute_vm_node(
    def: &VmBackendDef,
    input_refs: &[InputReference],
    upstream_slots: &[f32; 12],
    energy: &mut f32,
    energy_consumed: f32,
    memory: &mut [u8; 1024],
    static_inputs: &StaticInputs,
    config: &RuntimeConfig,
) -> NodeResult {
    // Safety: register_count == 0 → immediate halt.
    let reg_count = def.register_count as usize;
    if reg_count == 0 {
        return NodeResult::halted(*upstream_slots, 0.0);
    }

    // Safety: empty program → immediate halt.
    let program_len = def.program.len();
    if program_len == 0 {
        return NodeResult::halted(*upstream_slots, 0.0);
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

    // Only copy memory when the program contains memory instructions.
    // This avoids a 1 KiB copy for the majority of genomes.
    let uses_mem = def.has_memory_ops();
    let mut mem_copy: [u8; 1024] = if uses_mem { *memory } else { [0u8; 1024] };

    use crate::creature::genome::VmInstruction;

    /// Commit the working memory copy back to the creature's persistent memory,
    /// but only when the program actually uses memory operations.
    macro_rules! commit_memory {
        () => {
            if uses_mem {
                *memory = mem_copy;
            }
        };
    }

    loop {
        if steps >= max_steps {
            commit_memory!();
            return NodeResult::halted(payload, route_target);
        }

        // Soft default: if control flow lands outside the program, halt cleanly.
        if pc >= program_len {
            commit_memory!();
            return NodeResult::halted(payload, route_target);
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

            VmInstruction::ReadInput { dst, input_idx } => {
                let val = if (*input_idx as usize) < input_refs.len() {
                    resolve_input(
                        &input_refs[*input_idx as usize],
                        static_inputs,
                        upstream_slots,
                        *energy,
                        energy_consumed,
                    )
                } else {
                    0.0 // soft default: out-of-range input_idx
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

            VmInstruction::EmitWorldAction { action_type } => {
                let action = decode_world_action(*action_type, &meta);
                commit_memory!();
                return NodeResult::action(payload, route_target, action);
            }

            VmInstruction::WriteRouteTarget { src } => {
                route_target = regs[nr(*src, reg_count)]; // last-write-wins
            }

            VmInstruction::Halt => {
                commit_memory!();
                return NodeResult::halted(payload, route_target);
            }

            VmInstruction::LoadMem8 { dst, addr_reg } => {
                let addr = (regs[nr(*addr_reg, reg_count)] as i64).rem_euclid(1024) as usize;
                regs[nr(*dst, reg_count)] = sanitize_f32(mem_copy[addr] as f32);
            }

            VmInstruction::StoreMem8 { addr_reg, src } => {
                let addr = (regs[nr(*addr_reg, reg_count)] as i64).rem_euclid(1024) as usize;
                let val = regs[nr(*src, reg_count)].clamp(0.0, 255.0) as u8;
                mem_copy[addr] = val;
            }

            VmInstruction::LoadMem8Imm { dst, imm_addr } => {
                let addr = (*imm_addr as usize).rem_euclid(1024);
                regs[nr(*dst, reg_count)] = sanitize_f32(mem_copy[addr] as f32);
            }

            VmInstruction::StoreMem8Imm { imm_addr, src } => {
                let addr = (*imm_addr as usize).rem_euclid(1024);
                let val = regs[nr(*src, reg_count)].clamp(0.0, 255.0) as u8;
                mem_copy[addr] = val;
            }
        }

        pc = next_pc;
    }
}

/// Normalize a register index via rem_euclid wrapping.
#[inline]
fn nr(idx: u8, reg_count: usize) -> usize {
    (idx as usize).rem_euclid(reg_count)
}

/// Boolean truthiness: value >= 0.5.
#[inline]
fn is_truthy(v: f32) -> bool {
    v >= 0.5
}

/// Compute jump target PC with rem_euclid wrapping.
/// `offset` is signed relative to the instruction AFTER the jump.
#[inline]
fn jump_target(pc: usize, offset: i32, program_len: usize) -> usize {
    let provisional = pc as i64 + 1 + offset as i64;
    provisional.rem_euclid(program_len as i64) as usize
}

/// Base energy cost per opcode per v3-vm-isa-spec.md Section 6.
fn opcode_base_cost(instr: &crate::creature::genome::VmInstruction) -> f32 {
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
        VmInstruction::EmitWorldAction { .. } => 0.24,
        VmInstruction::WriteRouteTarget { .. } => 0.10,
        VmInstruction::Halt => 0.05,
        VmInstruction::LoadMem8 { .. } => 0.16,
        VmInstruction::StoreMem8 { .. } => 0.18,
        VmInstruction::LoadMem8Imm { .. } => 0.14,
        VmInstruction::StoreMem8Imm { .. } => 0.16,
    }
}

#[cfg(test)]
mod tests {
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

    // ── Halt and empty program ────────────────────────────────────────────────

    #[test]
    fn register_count_zero_halts_immediately() {
        let def = VmBackendDef {
            register_count: 0,
            constants: vec![],
            program: vec![VmInstruction::EmitWorldAction { action_type: 1 }],
        };
        let si = empty_static_inputs();
        let mut e = 100.0;
        let mut mem = [0u8; 1024];
        let r = execute_vm_node(
            &def,
            &[],
            &zeroed_upstream(),
            &mut e,
            0.0,
            &mut mem,
            &si,
            &config(),
        );
        assert!(r.world_action.is_none());
        assert!(!r.energy_exhausted);
    }

    #[test]
    fn empty_program_halts_immediately() {
        let (r, _) = run_vm(vec![], 1, vec![], &[], zeroed_upstream(), 100.0);
        assert!(r.world_action.is_none());
    }

    #[test]
    fn halt_returns_no_action() {
        let (r, _) = run_vm(
            vec![VmInstruction::Halt],
            2,
            vec![],
            &[],
            zeroed_upstream(),
            100.0,
        );
        assert!(r.world_action.is_none());
        assert!(!r.energy_exhausted);
    }

    #[test]
    fn program_counter_past_program_len_soft_halts() {
        // Program has one instruction and no explicit halt/jump.
        // After first step pc becomes 1 (out of range for len=1) and must soft-halt.
        // Uses opcode_cost_multiplier=1.0 so the deduction is observable in f32.
        let def = VmBackendDef {
            register_count: 1,
            constants: vec![],
            program: vec![VmInstruction::Noop],
        };
        let si = empty_static_inputs();
        let mut e = 100.0;
        let mut mem = [0u8; 1024];
        let mut cfg = config();
        cfg.vm.opcode_cost_multiplier = 1.0;
        let r = execute_vm_node(
            &def,
            &[],
            &zeroed_upstream(),
            &mut e,
            0.0,
            &mut mem,
            &si,
            &cfg,
        );
        assert!(r.world_action.is_none());
        assert!(!r.energy_exhausted);
        // Noop base cost = 0.05, multiplier = 1.0 → energy = 99.95
        assert!(e < 100.0);
    }

    // ── EmitWorldAction ───────────────────────────────────────────────────────

    #[test]
    fn emit_noop_action_type_0() {
        let (r, _) = run_vm(
            vec![VmInstruction::EmitWorldAction { action_type: 0 }],
            1,
            vec![],
            &[],
            zeroed_upstream(),
            100.0,
        );
        assert_eq!(r.world_action, Some(crate::contracts::WorldAction::NoOp));
    }

    #[test]
    fn emit_eat_action_type_1() {
        let (r, _) = run_vm(
            vec![VmInstruction::EmitWorldAction { action_type: 1 }],
            1,
            vec![],
            &[],
            zeroed_upstream(),
            100.0,
        );
        assert_eq!(r.world_action, Some(crate::contracts::WorldAction::Eat));
    }

    #[test]
    fn emit_move_with_meta() {
        // Set meta[0] = 2.0 (East), then emit Move
        let program = vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            }, // r0 = 2.0
            VmInstruction::WriteWorldActionMeta {
                slot_idx: 0,
                src: 0,
            },
            VmInstruction::EmitWorldAction { action_type: 2 },
        ];
        let (r, _) = run_vm(program, 1, vec![2.0], &[], zeroed_upstream(), 100.0);
        assert_eq!(
            r.world_action,
            Some(crate::contracts::WorldAction::Move(Direction::E))
        );
    }

    // ── Arithmetic opcodes ────────────────────────────────────────────────────

    #[test]
    fn add_two_constants() {
        let program = vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            }, // r0 = 3.0
            VmInstruction::LoadConst {
                dst: 1,
                const_idx: 1,
            }, // r1 = 4.0
            VmInstruction::Add { dst: 2, a: 0, b: 1 }, // r2 = 7.0
            VmInstruction::WriteInternalPayload {
                slot_idx: 0,
                src: 2,
            },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 3, vec![3.0, 4.0], &[], zeroed_upstream(), 100.0);
        assert!((r.output_slots[0] - 7.0).abs() < 1e-6);
    }

    #[test]
    fn sub_result() {
        let program = vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            }, // r0 = 10.0
            VmInstruction::LoadConst {
                dst: 1,
                const_idx: 1,
            }, // r1 = 3.0
            VmInstruction::Sub { dst: 2, a: 0, b: 1 }, // r2 = 7.0
            VmInstruction::WriteInternalPayload {
                slot_idx: 1,
                src: 2,
            },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 3, vec![10.0, 3.0], &[], zeroed_upstream(), 100.0);
        assert!((r.output_slots[1] - 7.0).abs() < 1e-6);
    }

    #[test]
    fn div_by_zero_yields_zero() {
        let program = vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            }, // r0 = 5.0
            // r1 stays 0.0 (initial)
            VmInstruction::Div { dst: 2, a: 0, b: 1 }, // r2 = 5/0 = 0
            VmInstruction::WriteInternalPayload {
                slot_idx: 0,
                src: 2,
            },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 3, vec![5.0], &[], zeroed_upstream(), 100.0);
        assert_eq!(r.output_slots[0], 0.0);
    }

    #[test]
    fn min_picks_smaller() {
        let program = vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            }, // r0 = 3.0
            VmInstruction::LoadConst {
                dst: 1,
                const_idx: 1,
            }, // r1 = 8.0
            VmInstruction::Min { dst: 2, a: 0, b: 1 },
            VmInstruction::WriteInternalPayload {
                slot_idx: 0,
                src: 2,
            },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 3, vec![3.0, 8.0], &[], zeroed_upstream(), 100.0);
        assert!((r.output_slots[0] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn max_picks_larger() {
        let program = vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            }, // r0 = 3.0
            VmInstruction::LoadConst {
                dst: 1,
                const_idx: 1,
            }, // r1 = 8.0
            VmInstruction::Max { dst: 2, a: 0, b: 1 },
            VmInstruction::WriteInternalPayload {
                slot_idx: 0,
                src: 2,
            },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 3, vec![3.0, 8.0], &[], zeroed_upstream(), 100.0);
        assert!((r.output_slots[0] - 8.0).abs() < 1e-6);
    }

    #[test]
    fn abs_of_negative() {
        let program = vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            }, // r0 = -5.0
            VmInstruction::Abs { dst: 1, src: 0 },
            VmInstruction::WriteInternalPayload {
                slot_idx: 0,
                src: 1,
            },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 2, vec![-5.0], &[], zeroed_upstream(), 100.0);
        assert!((r.output_slots[0] - 5.0).abs() < 1e-6);
    }

    #[test]
    fn neg_flips_sign() {
        let program = vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            }, // r0 = 3.0
            VmInstruction::Neg { dst: 1, src: 0 },
            VmInstruction::WriteInternalPayload {
                slot_idx: 0,
                src: 1,
            },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 2, vec![3.0], &[], zeroed_upstream(), 100.0);
        assert!((r.output_slots[0] - (-3.0)).abs() < 1e-6);
    }

    #[test]
    fn clamp01_clamps_above_one() {
        let program = vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            }, // r0 = 5.0
            VmInstruction::Clamp01 { dst: 1, src: 0 },
            VmInstruction::WriteInternalPayload {
                slot_idx: 0,
                src: 1,
            },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 2, vec![5.0], &[], zeroed_upstream(), 100.0);
        assert!((r.output_slots[0] - 1.0).abs() < 1e-6);
    }

    // ── Comparison opcodes ────────────────────────────────────────────────────

    #[test]
    fn cmpgt_true_when_a_greater() {
        let program = vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            }, // r0 = 5.0
            VmInstruction::LoadConst {
                dst: 1,
                const_idx: 1,
            }, // r1 = 3.0
            VmInstruction::CmpGt { dst: 2, a: 0, b: 1 },
            VmInstruction::WriteInternalPayload {
                slot_idx: 0,
                src: 2,
            },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 3, vec![5.0, 3.0], &[], zeroed_upstream(), 100.0);
        assert_eq!(r.output_slots[0], 1.0);
    }

    #[test]
    fn cmpgt_false_when_a_equal() {
        let program = vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            }, // r0 = 3.0
            VmInstruction::CmpGt { dst: 1, a: 0, b: 0 }, // 3.0 > 3.0 = false
            VmInstruction::WriteInternalPayload {
                slot_idx: 0,
                src: 1,
            },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 2, vec![3.0], &[], zeroed_upstream(), 100.0);
        assert_eq!(r.output_slots[0], 0.0);
    }

    #[test]
    fn cmplt_true_when_a_less() {
        let program = vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            }, // r0 = 2.0
            VmInstruction::LoadConst {
                dst: 1,
                const_idx: 1,
            }, // r1 = 5.0
            VmInstruction::CmpLt { dst: 2, a: 0, b: 1 },
            VmInstruction::WriteInternalPayload {
                slot_idx: 0,
                src: 2,
            },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 3, vec![2.0, 5.0], &[], zeroed_upstream(), 100.0);
        assert_eq!(r.output_slots[0], 1.0);
    }

    #[test]
    fn cmpeq_true_within_epsilon() {
        let program = vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            }, // r0 = 1.0
            VmInstruction::LoadConst {
                dst: 1,
                const_idx: 1,
            }, // r1 = 1.0001
            VmInstruction::LoadConst {
                dst: 2,
                const_idx: 2,
            }, // r2 = 0.01 (eps)
            VmInstruction::CmpEq {
                dst: 3,
                a: 0,
                b: 1,
                eps: 2,
            },
            VmInstruction::WriteInternalPayload {
                slot_idx: 0,
                src: 3,
            },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(
            program,
            4,
            vec![1.0, 1.0001, 0.01],
            &[],
            zeroed_upstream(),
            100.0,
        );
        assert_eq!(r.output_slots[0], 1.0);
    }

    #[test]
    fn and_both_true() {
        let program = vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            }, // r0 = 1.0 (truthy)
            VmInstruction::LoadConst {
                dst: 1,
                const_idx: 1,
            }, // r1 = 0.8 (truthy)
            VmInstruction::And { dst: 2, a: 0, b: 1 },
            VmInstruction::WriteInternalPayload {
                slot_idx: 0,
                src: 2,
            },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 3, vec![1.0, 0.8], &[], zeroed_upstream(), 100.0);
        assert_eq!(r.output_slots[0], 1.0);
    }

    #[test]
    fn and_one_false() {
        let program = vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            }, // r0 = 1.0
            VmInstruction::LoadConst {
                dst: 1,
                const_idx: 1,
            }, // r1 = 0.4 (not truthy, < 0.5)
            VmInstruction::And { dst: 2, a: 0, b: 1 },
            VmInstruction::WriteInternalPayload {
                slot_idx: 0,
                src: 2,
            },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 3, vec![1.0, 0.4], &[], zeroed_upstream(), 100.0);
        assert_eq!(r.output_slots[0], 0.0);
    }

    #[test]
    fn or_one_true() {
        let program = vec![
            // r0 stays 0.0 (false), r1 = 1.0 (true)
            VmInstruction::LoadConst {
                dst: 1,
                const_idx: 0,
            }, // r1 = 1.0
            VmInstruction::Or { dst: 2, a: 0, b: 1 },
            VmInstruction::WriteInternalPayload {
                slot_idx: 0,
                src: 2,
            },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 3, vec![1.0], &[], zeroed_upstream(), 100.0);
        assert_eq!(r.output_slots[0], 1.0);
    }

    #[test]
    fn not_inverts() {
        let program = vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            }, // r0 = 1.0 (truthy)
            VmInstruction::Not { dst: 1, src: 0 },
            VmInstruction::WriteInternalPayload {
                slot_idx: 0,
                src: 1,
            },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 2, vec![1.0], &[], zeroed_upstream(), 100.0);
        assert_eq!(r.output_slots[0], 0.0);
    }

    // ── Type conversion opcodes ───────────────────────────────────────────────

    #[test]
    fn to_i32_rounds_half_up() {
        let program = vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            }, // r0 = 2.5
            VmInstruction::ToI32 { dst: 1, src: 0 },
            VmInstruction::WriteInternalPayload {
                slot_idx: 0,
                src: 1,
            },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 2, vec![2.5], &[], zeroed_upstream(), 100.0);
        assert!((r.output_slots[0] - 3.0).abs() < 1e-6); // ties-away-from-zero
    }

    #[test]
    fn to_u8_clamps_above_255() {
        let program = vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            }, // r0 = 300.0
            VmInstruction::ToU8 { dst: 1, src: 0 },
            VmInstruction::WriteInternalPayload {
                slot_idx: 0,
                src: 1,
            },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 2, vec![300.0], &[], zeroed_upstream(), 100.0);
        assert!((r.output_slots[0] - 255.0).abs() < 1e-6);
    }

    #[test]
    fn to_bool_one_is_truthy() {
        let program = vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            }, // r0 = 0.6
            VmInstruction::ToBool { dst: 1, src: 0 },
            VmInstruction::WriteInternalPayload {
                slot_idx: 0,
                src: 1,
            },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 2, vec![0.6], &[], zeroed_upstream(), 100.0);
        assert_eq!(r.output_slots[0], 1.0);
    }

    // ── Control flow opcodes ──────────────────────────────────────────────────

    #[test]
    fn jump_skips_instructions() {
        let program = vec![
            VmInstruction::Jump { offset: 1 }, // jump to pc+1+1=2
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            }, // skipped
            VmInstruction::WriteInternalPayload {
                slot_idx: 0,
                src: 0,
            },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 1, vec![99.0], &[], zeroed_upstream(), 100.0);
        assert_eq!(r.output_slots[0], 0.0);
    }

    #[test]
    fn jump_if_zero_fires_when_false() {
        let program = vec![
            VmInstruction::JumpIfZero { cond: 0, offset: 1 }, // fires (r0=0)
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            }, // skipped
            VmInstruction::WriteInternalPayload {
                slot_idx: 0,
                src: 0,
            },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 1, vec![99.0], &[], zeroed_upstream(), 100.0);
        assert_eq!(r.output_slots[0], 0.0);
    }

    #[test]
    fn jump_if_zero_not_fires_when_truthy() {
        let program = vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            }, // r0 = 1.0
            VmInstruction::JumpIfZero { cond: 0, offset: 1 }, // does NOT fire (truthy)
            VmInstruction::LoadConst {
                dst: 1,
                const_idx: 1,
            }, // r1 = 99.0
            VmInstruction::WriteInternalPayload {
                slot_idx: 0,
                src: 1,
            },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 2, vec![1.0, 99.0], &[], zeroed_upstream(), 100.0);
        assert!((r.output_slots[0] - 99.0).abs() < 1e-6);
    }

    #[test]
    fn jump_target_wraps_via_rem_euclid() {
        // Program of 2 instructions: Jump(offset=2), Halt
        // provisional_pc = 0 + 1 + 2 = 3; 3 % 2 = 1 (Halt)
        let program = vec![VmInstruction::Jump { offset: 2 }, VmInstruction::Halt];
        let (r, _) = run_vm(program, 1, vec![], &[], zeroed_upstream(), 100.0);
        assert!(r.world_action.is_none());
    }

    // ── ReadInput opcode ──────────────────────────────────────────────────────

    #[test]
    fn read_input_upstream_slot() {
        use crate::contracts::InputReference;
        let input_refs = vec![InputReference::UpstreamSlot(0)];
        let mut upstream = zeroed_upstream();
        upstream[0] = 42.0;
        let program = vec![
            VmInstruction::ReadInput {
                dst: 0,
                input_idx: 0,
            },
            VmInstruction::WriteInternalPayload {
                slot_idx: 0,
                src: 0,
            },
            VmInstruction::Halt,
        ];
        let def = VmBackendDef {
            register_count: 1,
            constants: vec![],
            program,
        };
        let si = empty_static_inputs();
        let mut e = 100.0;
        let mut mem = [0u8; 1024];
        let r = execute_vm_node(
            &def,
            &input_refs,
            &upstream,
            &mut e,
            0.0,
            &mut mem,
            &si,
            &config(),
        );
        assert!((r.output_slots[0] - 42.0).abs() < 1e-6);
    }

    #[test]
    fn read_input_out_of_range_yields_zero() {
        let program = vec![
            VmInstruction::ReadInput {
                dst: 0,
                input_idx: 5,
            }, // no input_refs at all
            VmInstruction::WriteInternalPayload {
                slot_idx: 0,
                src: 0,
            },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 1, vec![], &[], zeroed_upstream(), 100.0);
        assert_eq!(r.output_slots[0], 0.0);
    }

    // ── Payload and routing ───────────────────────────────────────────────────

    #[test]
    fn payload_initialized_from_upstream_slots() {
        let mut upstream = zeroed_upstream();
        upstream[3] = 7.7;
        let program = vec![VmInstruction::Halt];
        let (r, _) = run_vm(program, 1, vec![], &[], upstream, 100.0);
        assert!((r.output_slots[3] - 7.7).abs() < 1e-6);
    }

    #[test]
    fn write_internal_payload_updates_slot() {
        let program = vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            },
            VmInstruction::WriteInternalPayload {
                slot_idx: 5,
                src: 0,
            },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 1, vec![3.5], &[], zeroed_upstream(), 100.0);
        assert!((r.output_slots[5] - 3.5).abs() < 1e-4);
    }

    #[test]
    fn write_internal_payload_invalid_slot_ignored() {
        let program = vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            },
            VmInstruction::WriteInternalPayload {
                slot_idx: 12,
                src: 0,
            }, // ignored
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 1, vec![99.0], &[], zeroed_upstream(), 100.0);
        for s in r.output_slots {
            assert_eq!(s, 0.0);
        }
    }

    #[test]
    fn write_route_target_sets_output() {
        let program = vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            }, // r0 = 2.0
            VmInstruction::WriteRouteTarget { src: 0 },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 1, vec![2.0], &[], zeroed_upstream(), 100.0);
        assert!((r.route_target_idx - 2.0).abs() < 1e-6);
    }

    // ── Memory opcodes ────────────────────────────────────────────────────────

    #[test]
    fn store_and_load_mem8() {
        let def = VmBackendDef {
            register_count: 2,
            constants: vec![42.0],
            program: vec![
                VmInstruction::LoadConst {
                    dst: 0,
                    const_idx: 0,
                }, // r0 = 42.0 (byte value)
                VmInstruction::LoadConst {
                    dst: 1,
                    const_idx: 0,
                }, // r1 = 42 (address)
                VmInstruction::StoreMem8 {
                    addr_reg: 1,
                    src: 0,
                }, // mem[42] = 42
                VmInstruction::LoadMem8 {
                    dst: 0,
                    addr_reg: 1,
                }, // r0 = mem[42]
                VmInstruction::WriteInternalPayload {
                    slot_idx: 0,
                    src: 0,
                },
                VmInstruction::Halt,
            ],
        };
        let si = empty_static_inputs();
        let mut e = 100.0;
        let mut mem = [0u8; 1024];
        let r = execute_vm_node(
            &def,
            &[],
            &zeroed_upstream(),
            &mut e,
            0.0,
            &mut mem,
            &si,
            &config(),
        );
        assert!((r.output_slots[0] - 42.0).abs() < 1e-6);
        assert_eq!(mem[42], 42); // memory committed
    }

    #[test]
    fn store_and_load_mem8_imm() {
        let def = VmBackendDef {
            register_count: 1,
            constants: vec![77.0],
            program: vec![
                VmInstruction::LoadConst {
                    dst: 0,
                    const_idx: 0,
                }, // r0 = 77.0
                VmInstruction::StoreMem8Imm {
                    imm_addr: 100,
                    src: 0,
                }, // mem[100] = 77
                VmInstruction::LoadMem8Imm {
                    dst: 0,
                    imm_addr: 100,
                }, // r0 = mem[100]
                VmInstruction::WriteInternalPayload {
                    slot_idx: 0,
                    src: 0,
                },
                VmInstruction::Halt,
            ],
        };
        let si = empty_static_inputs();
        let mut e = 100.0;
        let mut mem = [0u8; 1024];
        let r = execute_vm_node(
            &def,
            &[],
            &zeroed_upstream(),
            &mut e,
            0.0,
            &mut mem,
            &si,
            &config(),
        );
        assert!((r.output_slots[0] - 77.0).abs() < 1e-6);
        assert_eq!(mem[100], 77);
    }

    #[test]
    fn memory_address_wraps_via_rem_euclid() {
        let def = VmBackendDef {
            register_count: 2,
            constants: vec![1024.0, 55.0],
            program: vec![
                VmInstruction::LoadConst {
                    dst: 0,
                    const_idx: 0,
                }, // r0 = 1024 (addr)
                VmInstruction::LoadConst {
                    dst: 1,
                    const_idx: 1,
                }, // r1 = 55 (val)
                VmInstruction::StoreMem8 {
                    addr_reg: 0,
                    src: 1,
                }, // mem[1024%1024=0] = 55
                VmInstruction::LoadMem8 {
                    dst: 0,
                    addr_reg: 0,
                }, // r0 = mem[0]
                VmInstruction::WriteInternalPayload {
                    slot_idx: 0,
                    src: 0,
                },
                VmInstruction::Halt,
            ],
        };
        let si = empty_static_inputs();
        let mut e = 100.0;
        let mut mem = [0u8; 1024];
        let r = execute_vm_node(
            &def,
            &[],
            &zeroed_upstream(),
            &mut e,
            0.0,
            &mut mem,
            &si,
            &config(),
        );
        assert!((r.output_slots[0] - 55.0).abs() < 1e-6);
    }

    // ── Energy metering ───────────────────────────────────────────────────────

    #[test]
    fn energy_is_deducted_per_opcode() {
        // Uses opcode_cost_multiplier=1.0 so the deduction is observable in f32.
        // Noop(0.05) + Halt(0.05) = 0.10 total cost → energy = 99.90
        let def = VmBackendDef {
            register_count: 1,
            constants: vec![],
            program: vec![VmInstruction::Noop, VmInstruction::Halt],
        };
        let si = empty_static_inputs();
        let mut e = 100.0;
        let mut mem = [0u8; 1024];
        let mut cfg = config();
        cfg.vm.opcode_cost_multiplier = 1.0;
        let _ = execute_vm_node(
            &def,
            &[],
            &zeroed_upstream(),
            &mut e,
            0.0,
            &mut mem,
            &si,
            &cfg,
        );
        assert!(e < 100.0);
        assert!(e > 99.0);
    }

    #[test]
    fn energy_exhaustion_returns_exhausted() {
        // Very low energy with opcode_cost_multiplier=1.0; Noop base cost = 0.05
        let def = VmBackendDef {
            register_count: 1,
            constants: vec![],
            program: vec![VmInstruction::Noop, VmInstruction::Halt],
        };
        let si = empty_static_inputs();
        let mut e = 0.01;
        let mut mem = [0u8; 1024];
        let mut cfg = config();
        cfg.vm.opcode_cost_multiplier = 1.0;
        let r = execute_vm_node(
            &def,
            &[],
            &zeroed_upstream(),
            &mut e,
            0.0,
            &mut mem,
            &si,
            &cfg,
        );
        assert!(r.energy_exhausted);
    }

    #[test]
    fn energy_exhaustion_does_not_commit_memory_writes() {
        let def = VmBackendDef {
            register_count: 1,
            constants: vec![42.0],
            program: vec![
                VmInstruction::LoadConst {
                    dst: 0,
                    const_idx: 0,
                }, // costs 0.08
                VmInstruction::StoreMem8Imm {
                    imm_addr: 10,
                    src: 0,
                }, // costs 0.16
                VmInstruction::Halt,
            ],
        };
        let si = empty_static_inputs();
        // Give 0.20 energy with opcode_cost_multiplier=1.0:
        // LoadConst(0.08) → 0.12 left; StoreMem8Imm(0.16) → -0.04 → exhausted
        let mut e = 0.20;
        let mut mem = [0u8; 1024];
        let mut cfg = config();
        cfg.vm.opcode_cost_multiplier = 1.0;
        let r = execute_vm_node(
            &def,
            &[],
            &zeroed_upstream(),
            &mut e,
            0.0,
            &mut mem,
            &si,
            &cfg,
        );
        assert!(r.energy_exhausted);
        // mem[10] should still be 0 (not written)
        assert_eq!(mem[10], 0);
    }

    #[test]
    fn max_vm_steps_enforced() {
        // Infinite loop: Jump(offset=-1) → provisional_pc = 0 + 1 + (-1) = 0
        let program = vec![VmInstruction::Jump { offset: -1 }];
        let cfg = RuntimeConfig {
            max_vm_steps: 10,
            ..RuntimeConfig::default()
        };
        let def = VmBackendDef {
            register_count: 1,
            constants: vec![],
            program,
        };
        let si = empty_static_inputs();
        let mut e = 1000.0;
        let mut mem = [0u8; 1024];
        let r = execute_vm_node(
            &def,
            &[],
            &zeroed_upstream(),
            &mut e,
            0.0,
            &mut mem,
            &si,
            &cfg,
        );
        assert!(r.world_action.is_none());
        assert!(!r.energy_exhausted);
    }

    // ── Operand normalization ─────────────────────────────────────────────────

    #[test]
    fn register_index_wraps_via_rem_euclid() {
        let program = vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            }, // r0 = 7.0
            VmInstruction::WriteInternalPayload {
                slot_idx: 0,
                src: 2,
            }, // src=2 wraps to 0 → 7.0
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 2, vec![7.0], &[], zeroed_upstream(), 100.0);
        assert!((r.output_slots[0] - 7.0).abs() < 1e-6);
    }

    #[test]
    fn constant_index_wraps_via_rem_euclid() {
        // 2 constants: [3.0, 5.0]. const_idx=3 → 3%2=1 → 5.0
        let program = vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 3,
            }, // idx 3%2=1 → 5.0
            VmInstruction::WriteInternalPayload {
                slot_idx: 0,
                src: 0,
            },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 1, vec![3.0, 5.0], &[], zeroed_upstream(), 100.0);
        assert!((r.output_slots[0] - 5.0).abs() < 1e-6);
    }

    #[test]
    fn constant_empty_pool_yields_zero() {
        let program = vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            }, // empty pool → 0.0
            VmInstruction::WriteInternalPayload {
                slot_idx: 0,
                src: 0,
            },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 1, vec![], &[], zeroed_upstream(), 100.0);
        assert_eq!(r.output_slots[0], 0.0);
    }

    // ── sanitize_f32 on register writes ──────────────────────────────────────

    #[test]
    fn nan_in_add_becomes_zero() {
        // Use 2e9 constant: sanitize_f32(2e9) = 1e9, then Add(1e9, 1e9) = 2e9 → clamped to 1e9
        let program = vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            }, // r0 = 2e9 → clamped to 1e9
            VmInstruction::Add { dst: 1, a: 0, b: 0 }, // r1 = 1e9 + 1e9 = 2e9 → clamped to 1e9
            VmInstruction::WriteInternalPayload {
                slot_idx: 0,
                src: 1,
            },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 2, vec![2e9_f32], &[], zeroed_upstream(), 100.0);
        assert!((r.output_slots[0] - 1_000_000_000.0).abs() < 1.0);
    }

    // ── Integration: full input resolution pipeline ────────────────────────

    #[test]
    fn vm_eats_when_food_here() {
        use crate::config::{SimulationConfig, WorldEdgeMode};
        use crate::contracts::{CreatureId, NodeId, Position, WorldInputKey};
        use crate::creature::genome::{
            BackendDef, CreatureGenome, NodeGenome, VmBackendDef, VmInstruction,
        };
        use crate::creature::state::CreatureState;
        use crate::kernel::WorldState;
        use crate::sensors::static_inputs::assemble_static_inputs;
        use rand::rngs::SmallRng;
        use rand::SeedableRng;
        use slotmap::SlotMap;

        // Build a world with food everywhere
        let mut world = WorldState::new(5, 5, WorldEdgeMode::Wrap);
        let mut cfg = SimulationConfig::default();
        cfg.world.food.initial_coverage = 1.0;
        cfg.world.food.initial_density = 1.0;
        let mut rng = SmallRng::seed_from_u64(42);
        world.seed_food(&mut rng, &cfg);

        let pos = Position::new(1, 1);
        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        let id = sm.insert(());

        // Single-node VM: read food_here into r0, compare > 0, jump-if-zero past Eat, else Eat
        let input_refs = vec![InputReference::World(WorldInputKey::FoodHere)];
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(0),
                input_refs: input_refs.clone(),
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 2,
                    constants: vec![],
                    program: vec![
                        VmInstruction::ReadInput {
                            dst: 0,
                            input_idx: 0,
                        }, // r0 = food_here
                        // r1 is 0.0; compare r0 > r1
                        VmInstruction::CmpGt { dst: 0, a: 0, b: 1 }, // r0 = (food > 0)?
                        VmInstruction::JumpIfZero { cond: 0, offset: 1 }, // skip Eat if no food
                        VmInstruction::EmitWorldAction { action_type: 1 }, // Eat
                        VmInstruction::EmitWorldAction { action_type: 0 }, // NoOp fallback
                    ],
                }),
                targets: vec![],
            }],
        };
        let creature = CreatureState::new(id, genome, pos, 30.0, 0, [0, 0, 0], 0, [true; 3]);

        let si = assemble_static_inputs(&world, &creature);
        // food_here should be > 0.0
        assert!(si.food_here > 0.0);

        let def = if let BackendDef::Vm(ref v) = creature.genome.nodes[0].backend_def {
            v
        } else {
            panic!("expected VM backend");
        };

        let upstream = [0.0f32; 12];
        let mut energy = creature.energy;
        let mut mem = [0u8; 1024];
        let result = execute_vm_node(
            def,
            &creature.genome.nodes[0].input_refs,
            &upstream,
            &mut energy,
            0.0,
            &mut mem,
            &si,
            &config(),
        );
        assert_eq!(
            result.world_action,
            Some(crate::contracts::WorldAction::Eat),
            "Expected Eat when food is present"
        );
    }

    #[test]
    fn vm_noop_when_no_food() {
        use crate::config::WorldEdgeMode;
        use crate::contracts::{CreatureId, NodeId, Position, WorldInputKey};
        use crate::creature::genome::{
            BackendDef, CreatureGenome, NodeGenome, VmBackendDef, VmInstruction,
        };
        use crate::creature::state::CreatureState;
        use crate::kernel::WorldState;
        use crate::sensors::static_inputs::assemble_static_inputs;
        use slotmap::SlotMap;

        // World with no food
        let world = WorldState::new(5, 5, WorldEdgeMode::Wrap);
        let pos = Position::new(2, 2);
        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        let id = sm.insert(());

        let input_refs = vec![InputReference::World(WorldInputKey::FoodHere)];
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(0),
                input_refs: input_refs.clone(),
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 2,
                    constants: vec![],
                    program: vec![
                        VmInstruction::ReadInput {
                            dst: 0,
                            input_idx: 0,
                        }, // r0 = food_here = 0
                        VmInstruction::CmpGt { dst: 0, a: 0, b: 1 }, // r0 = (0 > 0) = 0
                        VmInstruction::JumpIfZero { cond: 0, offset: 1 }, // fires → skip Eat
                        VmInstruction::EmitWorldAction { action_type: 1 }, // Eat (skipped)
                        VmInstruction::EmitWorldAction { action_type: 0 }, // NoOp fallback
                    ],
                }),
                targets: vec![],
            }],
        };
        let creature = CreatureState::new(id, genome, pos, 30.0, 0, [0, 0, 0], 0, [true; 3]);
        let si = assemble_static_inputs(&world, &creature);
        assert_eq!(si.food_here, 0.0);

        let def = if let BackendDef::Vm(ref v) = creature.genome.nodes[0].backend_def {
            v
        } else {
            panic!()
        };

        let upstream = [0.0f32; 12];
        let mut energy = 30.0;
        let mut mem = [0u8; 1024];
        let result = execute_vm_node(
            def,
            &creature.genome.nodes[0].input_refs,
            &upstream,
            &mut energy,
            0.0,
            &mut mem,
            &si,
            &config(),
        );
        assert_eq!(
            result.world_action,
            Some(crate::contracts::WorldAction::NoOp),
            "Expected NoOp when no food"
        );
    }
}
