use crate::energy::meter_vm_ops;
use crate::mesh::{
    EmittedOutput, GraphBackendDef, GraphOperator, OutputDefinition, emitted_output_from_definition,
};

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

#[must_use]
pub fn graph_operator_cost_multiplier(operator: GraphOperator) -> f32 {
    match operator {
        GraphOperator::Passthrough => 0.7,
        GraphOperator::WeightedSum => 1.0,
        GraphOperator::Threshold { .. } => 0.9,
        GraphOperator::Clamp01 => 0.8,
        GraphOperator::DecayIntegrator { .. } => 1.2,
        GraphOperator::Momentum { .. } => 1.3,
        GraphOperator::Oscillator { .. } => 1.4,
        GraphOperator::SumPool => 1.0,
        GraphOperator::MeanPool => 1.1,
        GraphOperator::MaxPool => 1.2,
        GraphOperator::AdaptiveGain { .. } => 1.3,
    }
}

#[must_use]
pub fn evaluate_graph_operator(
    graph: &GraphBackendDef,
    inputs: &[f32],
    state_slots: &mut [f32],
) -> f32 {
    let first = inputs.first().copied().unwrap_or(0.0);
    let second = inputs.get(1).copied().unwrap_or(0.0);
    match graph.operator {
        GraphOperator::Passthrough => first + graph.bias,
        GraphOperator::WeightedSum => {
            let weighted = inputs
                .iter()
                .enumerate()
                .map(|(index, value)| {
                    let coefficient = graph.coefficients.get(index).copied().unwrap_or(1.0);
                    coefficient * value
                })
                .sum::<f32>();
            weighted + graph.bias
        }
        GraphOperator::Threshold { threshold } => {
            if first >= threshold {
                1.0
            } else {
                0.0
            }
        }
        GraphOperator::Clamp01 => (first + graph.bias).clamp(0.0, 1.0),
        GraphOperator::DecayIntegrator { state_slot, alpha } => {
            let slot = usize::from(state_slot);
            if let Some(state) = state_slots.get_mut(slot) {
                let alpha = alpha.clamp(0.0, 1.0);
                *state = (1.0 - alpha) * *state + alpha * first;
                *state
            } else {
                0.0
            }
        }
        GraphOperator::Momentum { state_slot, beta } => {
            let slot = usize::from(state_slot);
            if let Some(state) = state_slots.get_mut(slot) {
                let beta = beta.clamp(0.0, 1.0);
                let delta = first - second;
                *state = beta * *state + (1.0 - beta) * delta;
                *state
            } else {
                0.0
            }
        }
        GraphOperator::Oscillator {
            phase_slot,
            frequency,
            amplitude,
            bias,
        } => {
            let slot = usize::from(phase_slot);
            if let Some(phase) = state_slots.get_mut(slot) {
                let frequency = frequency.clamp(0.0, 8.0);
                *phase = (*phase + frequency).fract();
                bias + amplitude.clamp(0.0, 10.0) * (2.0 * std::f32::consts::PI * *phase).sin()
            } else {
                bias
            }
        }
        GraphOperator::SumPool => inputs.iter().copied().sum::<f32>() + graph.bias,
        GraphOperator::MeanPool => {
            if inputs.is_empty() {
                graph.bias
            } else {
                (inputs.iter().copied().sum::<f32>() / inputs.len() as f32) + graph.bias
            }
        }
        GraphOperator::MaxPool => {
            let base = inputs.iter().copied().reduce(f32::max).unwrap_or(0.0);
            base + graph.bias
        }
        GraphOperator::AdaptiveGain {
            gain_slot,
            learning_rate,
            min_gain,
            max_gain,
        } => {
            let slot = usize::from(gain_slot);
            if let Some(gain) = state_slots.get_mut(slot) {
                let learning_rate = learning_rate.clamp(0.0, 0.1);
                *gain = (*gain + learning_rate * second).clamp(min_gain, max_gain);
                *gain * first
            } else {
                0.0
            }
        }
    }
}
