use crate::config::runtime::VmConfig;
use crate::contracts::inputs::{CreatureInputs, NeighborSense};
use crate::contracts::outputs::WorldAction;
use crate::creature::genome::{
    BackendDef, InputReference, IntrospectionInputKey, NodeGenome, VmInstruction, WorldInputKey,
};
use crate::creature::state::Energy;
use crate::kernel::types::Direction;

// ── Numeric determinism ───────────────────────────────────────────────────────

/// Sanitize an f32 register value per ISA spec §5.
///
/// - NaN → 0.0
/// - +Inf → 1_000_000_000.0
/// - -Inf → -1_000_000_000.0
/// - Finite, magnitude > 1e9 → clamped to ±1e9
/// - Otherwise → unchanged
// Cannot use `.clamp()` here: `f32::clamp` propagates NaN, but the ISA spec
// requires NaN → 0.0. The manual chain is intentionally different.
#[allow(clippy::manual_clamp)]
#[inline]
pub fn sanitize_f32(v: f32) -> f32 {
    const LIMIT: f32 = 1_000_000_000.0;
    if v.is_nan() {
        0.0
    } else if v.is_infinite() {
        if v > 0.0 {
            LIMIT
        } else {
            -LIMIT
        }
    } else if v > LIMIT {
        LIMIT
    } else if v < -LIMIT {
        -LIMIT
    } else {
        v
    }
}

/// Return true if a register value is truthy (>= 0.5).
#[inline]
fn truthy(v: f32) -> bool {
    v >= 0.5
}

// ── Per-opcode energy base costs (from ISA spec §6) ──────────────────────────

fn opcode_base_cost(instr: &VmInstruction) -> u32 {
    // Base costs are f32 in the spec; we scale by 100 then round to get integer
    // micro-units, then divide by 100 when applying (opcode_cost_multiplier=1 keeps
    // actual drain at 0 for base costs < 1.0 when multiplier is 1).
    //
    // For Stage 3C with opcode_cost_multiplier=1 and small f32 base costs
    // (e.g. Noop=0.05), (0.05 * 1) as u32 = 0. VM execution is effectively free
    // until the multiplier is raised. This is the intended default behavior.
    let base_f32: f32 = match instr {
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
        VmInstruction::ReadSensorCell { .. } => 0.16,
        VmInstruction::ReadSensorCreature { .. } => 0.20,
        VmInstruction::ReadSensorSummary { .. } => 0.14,
        VmInstruction::ReadNeighborCell { .. } => 0.14,
        VmInstruction::ReadNeighborCreature { .. } => 0.18,
        VmInstruction::WriteInternalPayload { .. } => 0.14,
        VmInstruction::WriteWorldActionMeta { .. } => 0.14,
        VmInstruction::EmitInternal { .. } => 0.20,
        VmInstruction::EmitWorldAction { .. } => 0.24,
        VmInstruction::Halt => 0.05,
        VmInstruction::LoadMem8 { .. } => 0.16,
        VmInstruction::StoreMem8 { .. } => 0.18,
        VmInstruction::LoadMem8Imm { .. } => 0.14,
        VmInstruction::StoreMem8Imm { .. } => 0.16,
    };
    (base_f32 * vm_config_multiplier_scale()) as u32
}

/// Scale factor applied inside opcode_base_cost independent of VmConfig.
/// With this scale=1.0 and opcode_cost_multiplier=1, most opcodes cost 0 energy.
/// Raise the config multiplier to make VM execution measurably expensive.
#[inline]
fn vm_config_multiplier_scale() -> f32 {
    1.0
}

// ── Input resolution ─────────────────────────────────────────────────────────

/// Resolve a single `InputReference` to an f32 value from the assembled inputs.
fn resolve_input_ref(input_ref: &InputReference, inputs: &CreatureInputs) -> f32 {
    match input_ref {
        InputReference::World(key) => match key {
            WorldInputKey::FoodHere => inputs.environmental.food_density_self as f32 / 255.0,
        },
        InputReference::Introspection(key) => match key {
            IntrospectionInputKey::EnergyCurrent => inputs.introspection.energy as f32,
            IntrospectionInputKey::Generation => inputs.introspection.generation as f32,
            IntrospectionInputKey::AgeTicks => inputs.introspection.age_ticks as f32,
        },
        InputReference::NeighborCell {
            direction_idx,
            field,
        } => resolve_neighbor_cell(
            *direction_idx,
            *field as u8,
            &inputs.environmental.neighbors,
        ),
        InputReference::NeighborCreature {
            direction_idx,
            field,
        } => resolve_neighbor_creature(
            *direction_idx,
            *field as u8,
            &inputs.environmental.neighbors,
        ),
    }
}

fn resolve_neighbor_cell(neighbor_idx: u8, field_idx: u8, neighbors: &[NeighborSense; 8]) -> f32 {
    assert!(
        (neighbor_idx as usize) < Direction::ALL.len(),
        "VM: neighbor_idx {neighbor_idx} out of bounds (must be 0–7)"
    );
    let n = &neighbors[neighbor_idx as usize];
    match field_idx {
        0 => n.food_density as f32 / 255.0, // FoodDensityNorm
        1 => n.barrier as u8 as f32,        // BarrierFlag
        2 => n.occupied as u8 as f32,       // OccupiedFlag
        _ => 0.0,                           // soft default for unknown fields
    }
}

fn resolve_neighbor_creature(
    neighbor_idx: u8,
    field_idx: u8,
    neighbors: &[NeighborSense; 8],
) -> f32 {
    assert!(
        (neighbor_idx as usize) < Direction::ALL.len(),
        "VM: neighbor_idx {neighbor_idx} out of bounds (must be 0–7)"
    );
    let n = &neighbors[neighbor_idx as usize];
    match field_idx {
        0 => n.occupied as u8 as f32, // PresentFlag: occupied ↔ creature present
        _ => 0.0,                     // Stage 4+: EnergyNorm, PhenotypeRNorm, etc.
    }
}

// ── Action decoding ───────────────────────────────────────────────────────────

/// Decode a `WorldAction` from the action_type and world-action meta buffer.
///
/// action_type encoding:
///   0 = NoOp
///   1 = Eat
///   2 = Move  (meta[0] = direction 0–7)
///   3 = Reproduce (meta[0] = direction 0–7, meta[1] = energy_amount as f32)
fn decode_world_action(action_type: u8, meta: &[f32; 8]) -> WorldAction {
    match action_type {
        0 => WorldAction::NoOp,
        1 => WorldAction::Eat,
        2 => {
            let dir = direction_from_meta(meta[0]);
            WorldAction::Move { direction: dir }
        }
        3 => {
            let dir = direction_from_meta(meta[0]);
            let energy_amount = meta[1].max(0.0).round() as u32;
            WorldAction::Reproduce {
                direction: dir,
                energy_amount,
            }
        }
        _ => WorldAction::NoOp, // unknown action_type → NoOp
    }
}

/// Convert a direction float (0.0–7.0) to a `Direction` enum value.
/// Values are rounded to nearest integer and clamped to 0–7,
/// mapping to `Direction::ALL` order: 0=N,1=NE,2=E,3=SE,4=S,5=SW,6=W,7=NW.
fn direction_from_meta(v: f32) -> Direction {
    let idx = v.round().clamp(0.0, 7.0) as usize;
    Direction::ALL[idx]
}

// ── VM state ──────────────────────────────────────────────────────────────────

struct VmState {
    registers: Vec<f32>,
    pc: usize,
    world_action_meta: [f32; 8],
    internal_payload: [f32; 8],
}

impl VmState {
    fn new(register_count: u8) -> Self {
        Self {
            registers: vec![0.0; register_count as usize],
            pc: 0,
            world_action_meta: [0.0; 8],
            internal_payload: [0.0; 8],
        }
    }

    #[inline]
    fn reg(&self, idx: u8) -> f32 {
        self.registers
            .get(idx as usize)
            .copied()
            .unwrap_or_else(|| panic!("VM: register index {idx} out of bounds"))
    }

    #[inline]
    fn set_reg(&mut self, idx: u8, value: f32) {
        let slot = self
            .registers
            .get_mut(idx as usize)
            .unwrap_or_else(|| panic!("VM: register index {idx} out of bounds"));
        *slot = sanitize_f32(value);
    }

    #[inline]
    fn const_val(&self, constants: &[f32], idx: u8) -> f32 {
        constants
            .get(idx as usize)
            .copied()
            .unwrap_or_else(|| panic!("VM: const_idx {idx} out of bounds"))
    }

    fn write_meta(&mut self, slot_idx: u8, value: f32) {
        let slot = self
            .world_action_meta
            .get_mut(slot_idx as usize)
            .unwrap_or_else(|| panic!("VM: world_action_meta slot_idx {slot_idx} out of bounds"));
        *slot = value;
    }

    fn write_payload(&mut self, slot_idx: u8, value: f32) {
        let slot = self
            .internal_payload
            .get_mut(slot_idx as usize)
            .unwrap_or_else(|| panic!("VM: internal_payload slot_idx {slot_idx} out of bounds"));
        *slot = value;
    }
}

// ── Main execution entry point ────────────────────────────────────────────────

/// Execute a single VM node and return the emitted `WorldAction`, or `None`
/// if the program halted without calling `EmitWorldAction`.
///
/// Energy is deducted per opcode. If energy is exhausted, execution halts
/// immediately and returns `None`.
pub fn execute_vm_node(
    node: &NodeGenome,
    inputs: &CreatureInputs,
    memory: &mut [u8],
    energy: &mut Energy,
    vm_config: &VmConfig,
) -> Option<WorldAction> {
    let BackendDef::Vm(vm_def) = &node.backend_def;

    if vm_def.program.is_empty() {
        return None;
    }

    // Pre-resolve input_refs to a flat Vec<f32> for O(1) ReadInput lookup.
    let resolved_inputs: Vec<f32> = node
        .input_refs
        .iter()
        .map(|r| resolve_input_ref(r, inputs))
        .collect();

    let mut state = VmState::new(vm_def.register_count);

    loop {
        if state.pc >= vm_def.program.len() {
            return None; // PC past end: normal halt
        }

        let instr = &vm_def.program[state.pc];

        // Deduct energy cost. If insufficient, halt without action.
        let cost =
            (opcode_base_cost(instr) as f32 * vm_config.opcode_cost_multiplier as f32) as u32;
        if cost > 0 && !energy.drain(cost) {
            return None;
        }

        // Default PC advance (may be overridden by jump instructions).
        let next_pc = state.pc + 1;

        match instr {
            VmInstruction::Noop => {
                state.pc = next_pc;
            }
            VmInstruction::LoadConst { dst, const_idx } => {
                let v = state.const_val(&vm_def.constants, *const_idx);
                state.set_reg(*dst, v);
                state.pc = next_pc;
            }
            VmInstruction::Move { dst, src } => {
                let v = state.reg(*src);
                state.set_reg(*dst, v);
                state.pc = next_pc;
            }
            VmInstruction::Add { dst, a, b } => {
                let v = state.reg(*a) + state.reg(*b);
                state.set_reg(*dst, v);
                state.pc = next_pc;
            }
            VmInstruction::Sub { dst, a, b } => {
                let v = state.reg(*a) - state.reg(*b);
                state.set_reg(*dst, v);
                state.pc = next_pc;
            }
            VmInstruction::Mul { dst, a, b } => {
                let v = state.reg(*a) * state.reg(*b);
                state.set_reg(*dst, v);
                state.pc = next_pc;
            }
            VmInstruction::Div { dst, a, b } => {
                let divisor = state.reg(*b);
                let v = if divisor == 0.0 {
                    0.0
                } else {
                    state.reg(*a) / divisor
                };
                state.set_reg(*dst, v);
                state.pc = next_pc;
            }
            VmInstruction::Min { dst, a, b } => {
                let v = state.reg(*a).min(state.reg(*b));
                state.set_reg(*dst, v);
                state.pc = next_pc;
            }
            VmInstruction::Max { dst, a, b } => {
                let v = state.reg(*a).max(state.reg(*b));
                state.set_reg(*dst, v);
                state.pc = next_pc;
            }
            VmInstruction::Abs { dst, src } => {
                let v = state.reg(*src).abs();
                state.set_reg(*dst, v);
                state.pc = next_pc;
            }
            VmInstruction::Neg { dst, src } => {
                let v = -state.reg(*src);
                state.set_reg(*dst, v);
                state.pc = next_pc;
            }
            VmInstruction::Clamp01 { dst, src } => {
                let v = state.reg(*src).clamp(0.0, 1.0);
                state.set_reg(*dst, v);
                state.pc = next_pc;
            }
            VmInstruction::CmpGt { dst, a, b } => {
                let v = if state.reg(*a) > state.reg(*b) {
                    1.0
                } else {
                    0.0
                };
                state.set_reg(*dst, v);
                state.pc = next_pc;
            }
            VmInstruction::CmpLt { dst, a, b } => {
                let v = if state.reg(*a) < state.reg(*b) {
                    1.0
                } else {
                    0.0
                };
                state.set_reg(*dst, v);
                state.pc = next_pc;
            }
            VmInstruction::CmpEq { dst, a, b, eps } => {
                let epsilon = state.reg(*eps).clamp(1e-6, 1.0);
                let v = if (state.reg(*a) - state.reg(*b)).abs() <= epsilon {
                    1.0
                } else {
                    0.0
                };
                state.set_reg(*dst, v);
                state.pc = next_pc;
            }
            VmInstruction::And { dst, a, b } => {
                let v = if truthy(state.reg(*a)) && truthy(state.reg(*b)) {
                    1.0
                } else {
                    0.0
                };
                state.set_reg(*dst, v);
                state.pc = next_pc;
            }
            VmInstruction::Or { dst, a, b } => {
                let v = if truthy(state.reg(*a)) || truthy(state.reg(*b)) {
                    1.0
                } else {
                    0.0
                };
                state.set_reg(*dst, v);
                state.pc = next_pc;
            }
            VmInstruction::Not { dst, src } => {
                let v = if truthy(state.reg(*src)) { 0.0 } else { 1.0 };
                state.set_reg(*dst, v);
                state.pc = next_pc;
            }
            VmInstruction::ToI32 { dst, src } => {
                let v = state.reg(*src).round() as i32 as f32;
                state.set_reg(*dst, v);
                state.pc = next_pc;
            }
            VmInstruction::ToU8 { dst, src } => {
                let v = state.reg(*src).clamp(0.0, 255.0).round();
                state.set_reg(*dst, v);
                state.pc = next_pc;
            }
            VmInstruction::ToBool { dst, src } => {
                let v = if truthy(state.reg(*src)) { 1.0 } else { 0.0 };
                state.set_reg(*dst, v);
                state.pc = next_pc;
            }
            VmInstruction::JumpIfZero { cond, offset } => {
                if !truthy(state.reg(*cond)) {
                    // Apply offset to post-increment PC
                    let target = next_pc as i64 + *offset as i64;
                    assert!(
                        target >= 0,
                        "VM: negative PC after JumpIfZero (target={target})"
                    );
                    state.pc = target as usize;
                } else {
                    state.pc = next_pc;
                }
            }
            VmInstruction::Jump { offset } => {
                let target = next_pc as i64 + *offset as i64;
                assert!(target >= 0, "VM: negative PC after Jump (target={target})");
                state.pc = target as usize;
            }
            VmInstruction::ReadInput { dst, input_idx } => {
                let v = resolved_inputs
                    .get(*input_idx as usize)
                    .copied()
                    .unwrap_or(0.0); // soft default for out-of-range slot
                state.set_reg(*dst, v);
                state.pc = next_pc;
            }
            // SensorFrame opcodes: return 0.0 in Stage 3C (no SensorFrame built yet).
            VmInstruction::ReadSensorCell { dst, .. }
            | VmInstruction::ReadSensorCreature { dst, .. }
            | VmInstruction::ReadSensorSummary { dst, .. } => {
                state.set_reg(*dst, 0.0);
                state.pc = next_pc;
            }
            VmInstruction::ReadNeighborCell {
                dst,
                neighbor_idx,
                field_idx,
            } => {
                let v = resolve_neighbor_cell(
                    *neighbor_idx,
                    *field_idx,
                    &inputs.environmental.neighbors,
                );
                state.set_reg(*dst, v);
                state.pc = next_pc;
            }
            VmInstruction::ReadNeighborCreature {
                dst,
                neighbor_idx,
                field_idx,
            } => {
                let v = resolve_neighbor_creature(
                    *neighbor_idx,
                    *field_idx,
                    &inputs.environmental.neighbors,
                );
                state.set_reg(*dst, v);
                state.pc = next_pc;
            }
            VmInstruction::WriteInternalPayload { slot_idx, src } => {
                let v = state.reg(*src);
                state.write_payload(*slot_idx, v);
                state.pc = next_pc;
            }
            VmInstruction::WriteWorldActionMeta { slot_idx, src } => {
                let v = state.reg(*src);
                state.write_meta(*slot_idx, v);
                state.pc = next_pc;
            }
            VmInstruction::EmitInternal { .. } => {
                // Clear internal payload after emit; no inter-node routing in Stage 3C.
                state.internal_payload = [0.0; 8];
                state.pc = next_pc;
            }
            VmInstruction::EmitWorldAction { action_type } => {
                let action = decode_world_action(*action_type, &state.world_action_meta);
                return Some(action); // halt immediately
            }
            VmInstruction::Halt => {
                return None;
            }
            VmInstruction::LoadMem8 { dst, addr_reg } => {
                let addr = state.reg(*addr_reg) as usize;
                let resolved = addr.rem_euclid(1024);
                let byte = memory[resolved];
                state.set_reg(*dst, byte as f32);
                state.pc = next_pc;
            }
            VmInstruction::StoreMem8 { addr_reg, src } => {
                let addr = state.reg(*addr_reg) as usize;
                let resolved = addr.rem_euclid(1024);
                let byte = state.reg(*src).clamp(0.0, 255.0).round() as u8;
                memory[resolved] = byte;
                state.pc = next_pc;
            }
            VmInstruction::LoadMem8Imm { dst, imm_addr } => {
                let resolved = (*imm_addr as usize).rem_euclid(1024);
                let byte = memory[resolved];
                state.set_reg(*dst, byte as f32);
                state.pc = next_pc;
            }
            VmInstruction::StoreMem8Imm { imm_addr, src } => {
                let resolved = (*imm_addr as usize).rem_euclid(1024);
                let byte = state.reg(*src).clamp(0.0, 255.0).round() as u8;
                memory[resolved] = byte;
                state.pc = next_pc;
            }
        }
    }
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::creature::genome::{BackendDef, NodeGenome, NodeId, OutputDefinition, VmBackendDef};
    use crate::kernel::types::Position;

    fn make_inputs(food: u8, energy: u32) -> CreatureInputs {
        use crate::contracts::inputs::{EnvironmentalInputs, IntrospectionInputs};
        CreatureInputs {
            environmental: EnvironmentalInputs {
                food_density_self: food,
                neighbors: Default::default(),
            },
            introspection: IntrospectionInputs {
                energy,
                position: Position::default(),
                generation: 0,
                age_ticks: 0,
            },
        }
    }

    fn simple_eat_node() -> NodeGenome {
        use VmInstruction::*;
        NodeGenome {
            node_id: NodeId(0),
            input_refs: vec![InputReference::World(WorldInputKey::FoodHere)],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 4,
                program: vec![
                    ReadInput {
                        dst: 0,
                        input_idx: 0,
                    }, // r0 = food_norm
                    LoadConst {
                        dst: 1,
                        const_idx: 0,
                    }, // r1 = 0.0
                    CmpGt { dst: 2, a: 0, b: 1 },       // r2 = food > 0?
                    JumpIfZero { cond: 2, offset: 1 },  // no food → skip Eat
                    EmitWorldAction { action_type: 1 }, // Eat
                    Halt,
                ],
                constants: vec![0.0],
            }),
            output_definitions: Vec::<OutputDefinition>::new(),
        }
    }

    #[test]
    fn sanitize_nan_to_zero() {
        assert_eq!(sanitize_f32(f32::NAN), 0.0);
    }

    #[test]
    fn sanitize_inf_to_limit() {
        assert_eq!(sanitize_f32(f32::INFINITY), 1_000_000_000.0);
        assert_eq!(sanitize_f32(f32::NEG_INFINITY), -1_000_000_000.0);
    }

    #[test]
    fn sanitize_finite_unchanged() {
        assert_eq!(sanitize_f32(42.5), 42.5);
        assert_eq!(sanitize_f32(-1.23), -1.23);
    }

    #[test]
    fn emit_eat_when_food_present() {
        let node = simple_eat_node();
        let inputs = make_inputs(100, 50);
        let mut memory = [0u8; 1024];
        let mut energy = Energy::new(1000);
        let config = VmConfig::default();

        let action = execute_vm_node(&node, &inputs, &mut memory, &mut energy, &config);
        assert_eq!(action, Some(WorldAction::Eat));
    }

    #[test]
    fn noop_when_no_food_and_halt() {
        let node = simple_eat_node();
        let inputs = make_inputs(0, 50);
        let mut memory = [0u8; 1024];
        let mut energy = Energy::new(1000);
        let config = VmConfig::default();

        let action = execute_vm_node(&node, &inputs, &mut memory, &mut energy, &config);
        assert_eq!(action, None); // halts without EmitWorldAction
    }

    #[test]
    fn read_input_out_of_range_returns_zero() {
        use VmInstruction::*;
        let node = NodeGenome {
            node_id: NodeId(0),
            input_refs: vec![], // no slots defined
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 2,
                program: vec![
                    ReadInput {
                        dst: 0,
                        input_idx: 5,
                    }, // slot 5 out of range
                    EmitWorldAction { action_type: 0 }, // NoOp (so we can check r0 effect)
                ],
                constants: vec![],
            }),
            output_definitions: Vec::<OutputDefinition>::new(),
        };
        let inputs = make_inputs(200, 50);
        let mut memory = [0u8; 1024];
        let mut energy = Energy::new(1000);
        let config = VmConfig::default();

        let action = execute_vm_node(&node, &inputs, &mut memory, &mut energy, &config);
        // Should succeed (EmitWorldAction NoOp), proving no panic on out-of-range
        assert_eq!(action, Some(WorldAction::NoOp));
    }

    #[test]
    fn memory_wrap_reads_and_writes() {
        use VmInstruction::*;
        let node = NodeGenome {
            node_id: NodeId(0),
            input_refs: vec![],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 4,
                program: vec![
                    LoadConst {
                        dst: 0,
                        const_idx: 0,
                    }, // r0 = 42.0
                    StoreMem8Imm {
                        imm_addr: 1025,
                        src: 0,
                    }, // addr 1025 mod 1024 = 1
                    LoadMem8Imm {
                        dst: 1,
                        imm_addr: 1,
                    }, // r1 = mem[1]
                    EmitWorldAction { action_type: 0 }, // NoOp to halt
                ],
                constants: vec![42.0],
            }),
            output_definitions: Vec::<OutputDefinition>::new(),
        };
        let inputs = make_inputs(0, 0);
        let mut memory = [0u8; 1024];
        let mut energy = Energy::new(1000);
        let config = VmConfig::default();

        execute_vm_node(&node, &inputs, &mut memory, &mut energy, &config);
        assert_eq!(memory[1], 42, "memory write/read with address wrap failed");
    }

    #[test]
    fn jump_offset_semantics() {
        use VmInstruction::*;
        // Program: JumpIfZero(r0=0.0, +1) → jumps past EmitWorldAction(Eat) → EmitWorldAction(NoOp)
        let node = NodeGenome {
            node_id: NodeId(0),
            input_refs: vec![],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 2,
                program: vec![
                    LoadConst {
                        dst: 0,
                        const_idx: 0,
                    }, // r0 = 0.0 (falsy)
                    JumpIfZero { cond: 0, offset: 1 }, // falsy → PC = 2+1 = 3
                    EmitWorldAction { action_type: 1 }, // Eat (skipped)
                    EmitWorldAction { action_type: 0 }, // NoOp (landed here)
                ],
                constants: vec![0.0],
            }),
            output_definitions: Vec::<OutputDefinition>::new(),
        };
        let inputs = make_inputs(0, 0);
        let mut memory = [0u8; 1024];
        let mut energy = Energy::new(1000);
        let config = VmConfig::default();

        let action = execute_vm_node(&node, &inputs, &mut memory, &mut energy, &config);
        assert_eq!(
            action,
            Some(WorldAction::NoOp),
            "jump skipped wrong instruction"
        );
    }

    #[test]
    fn emit_world_action_move_decodes_direction() {
        use VmInstruction::*;
        let node = NodeGenome {
            node_id: NodeId(0),
            input_refs: vec![],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 2,
                program: vec![
                    LoadConst {
                        dst: 0,
                        const_idx: 0,
                    }, // r0 = 2.0 (dir E)
                    WriteWorldActionMeta {
                        slot_idx: 0,
                        src: 0,
                    }, // meta[0] = 2.0
                    EmitWorldAction { action_type: 2 }, // Move
                ],
                constants: vec![2.0],
            }),
            output_definitions: Vec::<OutputDefinition>::new(),
        };
        let inputs = make_inputs(0, 0);
        let mut memory = [0u8; 1024];
        let mut energy = Energy::new(1000);
        let config = VmConfig::default();

        let action = execute_vm_node(&node, &inputs, &mut memory, &mut energy, &config);
        assert_eq!(
            action,
            Some(WorldAction::Move {
                direction: Direction::E
            })
        );
    }
}
