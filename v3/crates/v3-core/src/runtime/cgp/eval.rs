use crate::creature::genome::cgp::ComputeNodeKind;
use crate::runtime::types::sanitize_f32;

/// Evaluate one compute node kind, returning the scalar output.
///
/// `w_inputs` holds per-edge weighted values.
/// `wsum` is `w_inputs.iter().sum()`.
/// `state` is the node's mutable persistent scalar state (for stateful operators).
#[inline]
pub(crate) fn evaluate_compute_kind(
    kind: &ComputeNodeKind,
    w_inputs: &[f32],
    wsum: f32,
    state: &mut f32,
) -> f32 {
    match kind {
        ComputeNodeKind::Add | ComputeNodeKind::WeightedSum => wsum,
        ComputeNodeKind::Multiply => w_inputs.iter().copied().product::<f32>(),
        ComputeNodeKind::Negate => -wsum,
        ComputeNodeKind::Abs => wsum.abs(),
        ComputeNodeKind::Min => w_inputs.iter().copied().reduce(f32::min).unwrap_or(0.0),
        ComputeNodeKind::Max => w_inputs.iter().copied().reduce(f32::max).unwrap_or(0.0),
        ComputeNodeKind::Threshold(t) => {
            if wsum > *t {
                1.0
            } else {
                0.0
            }
        }
        ComputeNodeKind::GreaterThan => {
            let a = w_inputs.first().copied().unwrap_or(0.0);
            let b = w_inputs.get(1).copied().unwrap_or(0.0);
            if a > b {
                1.0
            } else {
                0.0
            }
        }
        ComputeNodeKind::Sigmoid => 1.0 / (1.0 + (-wsum).exp()),
        ComputeNodeKind::Tanh => wsum.tanh(),
        ComputeNodeKind::Relu => wsum.max(0.0),
        ComputeNodeKind::Clamp01 => wsum.clamp(0.0, 1.0),
        ComputeNodeKind::Select => {
            let cond = w_inputs.first().copied().unwrap_or(0.0);
            if cond >= 0.5 {
                w_inputs.get(1).copied().unwrap_or(0.0)
            } else {
                w_inputs.get(2).copied().unwrap_or(0.0)
            }
        }
        ComputeNodeKind::Constant(f) => *f,
        ComputeNodeKind::DecayIntegrator(a) => {
            let a_c = a.clamp(0.0, 1.0);
            *state = (1.0 - a_c) * *state + a_c * wsum;
            *state
        }
        ComputeNodeKind::Momentum(b) => {
            let b_c = b.clamp(0.0, 1.0);
            *state = b_c * *state + (1.0 - b_c) * wsum;
            *state
        }
        ComputeNodeKind::Oscillator(f) => {
            let f_c = f.clamp(0.0, 8.0);
            *state = (*state + f_c).fract();
            (2.0 * std::f32::consts::PI * *state).sin()
        }
        ComputeNodeKind::AdaptiveGain => {
            *state = (*state + 0.01 * wsum).clamp(0.1, 2.0);
            *state * wsum
        }
    }
}

/// Sanitize the output of a compute node.
#[inline]
pub(crate) fn sanitize_output(v: f32) -> f32 {
    sanitize_f32(v)
}
