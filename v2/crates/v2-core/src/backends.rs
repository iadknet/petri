use crate::energy::meter_vm_ops;
use crate::mesh::{EmittedOutput, OutputDefinition};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EnergyBudget {
    pub graph_static_tariff: f32,
    pub vm_per_op_cost: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BackendExecutionRequest {
    pub remaining_energy: f32,
    pub energy_budget: EnergyBudget,
    pub output_definitions: Vec<OutputDefinition>,
    pub vm_requested_ops: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BackendExecution {
    pub emitted: Vec<EmittedOutput>,
    pub compute_energy: f32,
    pub exhausted_energy: bool,
}

pub fn execute_graph_backend(request: &BackendExecutionRequest) -> BackendExecution {
    let remaining_energy = request.remaining_energy.max(0.0);
    let requested_cost = request.energy_budget.graph_static_tariff.max(0.0);
    let compute_energy = remaining_energy.min(requested_cost);
    let remaining = remaining_energy - compute_energy;
    let exhausted_energy = remaining <= f32::EPSILON;

    let emitted = request
        .output_definitions
        .iter()
        .map(emitted_output_from_definition)
        .collect::<Vec<_>>();

    BackendExecution {
        emitted,
        compute_energy,
        exhausted_energy,
    }
}

pub fn execute_vm_backend(request: &BackendExecutionRequest) -> BackendExecution {
    let vm_metering = meter_vm_ops(
        request.remaining_energy,
        request.energy_budget.vm_per_op_cost,
        request.vm_requested_ops,
    );
    BackendExecution {
        emitted: Vec::new(),
        compute_energy: vm_metering.charged_energy,
        exhausted_energy: vm_metering.exhausted,
    }
}

fn emitted_output_from_definition(definition: &OutputDefinition) -> EmittedOutput {
    match definition {
        OutputDefinition::InternalTarget(target) => EmittedOutput::InternalTarget(target.clone()),
        OutputDefinition::WorldAction(action) => EmittedOutput::WorldAction(action.clone()),
    }
}
