use crate::mesh::VmInstruction;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeteringResult {
    pub remaining_energy: f32,
    pub charged_energy: f32,
    pub exhausted: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VmMeteringResult {
    pub executed_ops: usize,
    pub remaining_energy: f32,
    pub charged_energy: f32,
    pub exhausted: bool,
}

fn charge(remaining_energy: f32, requested_cost: f32) -> MeteringResult {
    let bounded_remaining = remaining_energy.max(0.0);
    let bounded_cost = requested_cost.max(0.0);
    let charged_energy = bounded_cost.min(bounded_remaining);
    let remaining = bounded_remaining - charged_energy;
    let exhausted = remaining <= f32::EPSILON;

    MeteringResult {
        remaining_energy: if exhausted { 0.0 } else { remaining },
        charged_energy,
        exhausted,
    }
}

pub fn charge_dispatch_entry(remaining_energy: f32, dispatch_entry_cost: f32) -> MeteringResult {
    charge(remaining_energy, dispatch_entry_cost)
}

pub fn charge_backend_graph(remaining_energy: f32, graph_static_tariff: f32) -> MeteringResult {
    charge(remaining_energy, graph_static_tariff)
}

pub fn charge_action(remaining_energy: f32, action_cost: f32) -> MeteringResult {
    charge(remaining_energy, action_cost)
}

pub fn meter_vm_ops(
    initial_energy: f32,
    per_op_cost: f32,
    requested_ops: usize,
) -> VmMeteringResult {
    let mut remaining = initial_energy.max(0.0);
    let mut charged_energy = 0.0_f32;
    let mut executed = 0_usize;
    let bounded_per_op = per_op_cost.max(0.0);

    if bounded_per_op <= f32::EPSILON {
        return VmMeteringResult {
            executed_ops: requested_ops,
            remaining_energy: remaining,
            charged_energy,
            exhausted: remaining <= f32::EPSILON,
        };
    }

    while executed < requested_ops && remaining > f32::EPSILON {
        let cost = bounded_per_op.min(remaining);
        remaining -= cost;
        charged_energy += cost;
        executed += 1;
    }

    let exhausted = remaining <= f32::EPSILON;
    VmMeteringResult {
        executed_ops: executed,
        remaining_energy: if exhausted { 0.0 } else { remaining },
        charged_energy,
        exhausted,
    }
}

#[must_use]
pub fn vm_opcode_base_cost(opcode: &VmInstruction) -> f32 {
    match opcode {
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
    }
}
