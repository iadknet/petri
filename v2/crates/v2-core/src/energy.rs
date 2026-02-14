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
