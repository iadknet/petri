//! CGP-style graph backend evaluation.
//!
//! Relaxation loop over `compute_nodes` only. Fixed structural outputs
//! (output sinks, action bank, execute gate) are processed in a separate
//! post-convergence effects pass (see `cgp_graph_effects`).

use crate::contracts::InputReference;
use crate::creature::genome::cgp::{ComputeNodeKind, GraphSource};
use crate::runtime::inputs::{resolve_input, ResolveCtx};
use crate::runtime::types::sanitize_f32;

// ─── Source resolution ──────────────────────────────────────────────────────

/// Resolve a `GraphSource` to its scalar value during graph relaxation.
///
/// For `ComputeNode` sources, uses Gauss-Seidel order: sources already
/// evaluated this pass (`idx < current_idx`) read from `curr_outputs`;
/// not-yet-evaluated sources (including self-loops) read from `prev_outputs`.
#[inline]
#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_source(
    source: &GraphSource,
    current_idx: usize,
    node_count: usize,
    prev_outputs: &[f32],
    curr_outputs: &[f32],
    input_refs: &[InputReference],
    resolve_ctx: &ResolveCtx<'_>,
    shared_memory: &[f32; 16],
    prev_shared_memory: &[f32; 16],
) -> f32 {
    match source {
        GraphSource::InputLeaf { ref_idx, sub_idx } => {
            let ri = *ref_idx as usize;
            if ri < input_refs.len() {
                resolve_input(&input_refs[ri], *sub_idx, resolve_ctx)
            } else {
                0.0
            }
        }
        GraphSource::SharedMemory { slot, previous } => {
            let idx = (*slot as usize) % 16;
            if *previous {
                prev_shared_memory[idx]
            } else {
                shared_memory[idx]
            }
        }
        GraphSource::ComputeNode(idx) => {
            let i = *idx as usize;
            if i >= node_count {
                0.0
            } else if i < current_idx {
                curr_outputs[i]
            } else {
                prev_outputs[i]
            }
        }
    }
}

// ─── Weighted input collection ──────────────────────────────────────────────

/// Collect weighted input values for a compute node into `buf`.
///
/// Clears `buf` and fills it with one entry per edge: `resolve(source) * weight`.
/// The caller allocates `buf` once and reuses it across nodes/passes.
#[inline]
#[allow(clippy::too_many_arguments)]
pub(crate) fn collect_cgp_weighted_inputs(
    edges: &[crate::creature::genome::cgp::GraphEdge],
    current_idx: usize,
    node_count: usize,
    prev_outputs: &[f32],
    curr_outputs: &[f32],
    input_refs: &[InputReference],
    resolve_ctx: &ResolveCtx<'_>,
    shared_memory: &[f32; 16],
    prev_shared_memory: &[f32; 16],
    buf: &mut Vec<f32>,
) {
    buf.clear();
    buf.extend(edges.iter().map(|edge| {
        let source_value = resolve_source(
            &edge.source,
            current_idx,
            node_count,
            prev_outputs,
            curr_outputs,
            input_refs,
            resolve_ctx,
            shared_memory,
            prev_shared_memory,
        );
        source_value * edge.weight
    }));
}

// ─── Compute node evaluation ────────────────────────────────────────────────

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

#[cfg(test)]
mod tests {
    use super::*;

    // ── evaluate_compute_kind tests ─────────────────────────────────────────

    #[test]
    fn add_sums_weighted_inputs() {
        let mut s = 0.0;
        assert_eq!(
            evaluate_compute_kind(&ComputeNodeKind::Add, &[1.0, 2.0, 3.0], 6.0, &mut s),
            6.0
        );
    }

    #[test]
    fn multiply_products_weighted_inputs() {
        let mut s = 0.0;
        assert_eq!(
            evaluate_compute_kind(&ComputeNodeKind::Multiply, &[2.0, 3.0, 4.0], 9.0, &mut s),
            24.0
        );
    }

    #[test]
    fn multiply_empty_inputs_returns_one() {
        let mut s = 0.0;
        // empty product = 1.0 (f32 iterator product identity)
        assert_eq!(
            evaluate_compute_kind(&ComputeNodeKind::Multiply, &[], 0.0, &mut s),
            1.0
        );
    }

    #[test]
    fn threshold_gate() {
        let mut s = 0.0;
        assert_eq!(
            evaluate_compute_kind(&ComputeNodeKind::Threshold(5.0), &[], 6.0, &mut s),
            1.0
        );
        assert_eq!(
            evaluate_compute_kind(&ComputeNodeKind::Threshold(5.0), &[], 4.0, &mut s),
            0.0
        );
        assert_eq!(
            evaluate_compute_kind(&ComputeNodeKind::Threshold(5.0), &[], 5.0, &mut s),
            0.0
        );
    }

    #[test]
    fn greater_than_two_inputs() {
        let mut s = 0.0;
        assert_eq!(
            evaluate_compute_kind(&ComputeNodeKind::GreaterThan, &[3.0, 2.0], 5.0, &mut s),
            1.0
        );
        assert_eq!(
            evaluate_compute_kind(&ComputeNodeKind::GreaterThan, &[2.0, 3.0], 5.0, &mut s),
            0.0
        );
    }

    #[test]
    fn sigmoid_at_zero() {
        let mut s = 0.0;
        let result = evaluate_compute_kind(&ComputeNodeKind::Sigmoid, &[], 0.0, &mut s);
        assert!((result - 0.5).abs() < 1e-6);
    }

    #[test]
    fn constant_ignores_inputs() {
        let mut s = 0.0;
        assert_eq!(
            evaluate_compute_kind(&ComputeNodeKind::Constant(42.0), &[1.0, 2.0], 3.0, &mut s),
            42.0
        );
    }

    #[test]
    fn select_chooses_by_condition() {
        let mut s = 0.0;
        // cond >= 0.5 → pick input[1]
        assert_eq!(
            evaluate_compute_kind(&ComputeNodeKind::Select, &[1.0, 10.0, 20.0], 31.0, &mut s),
            10.0
        );
        // cond < 0.5 → pick input[2]
        assert_eq!(
            evaluate_compute_kind(&ComputeNodeKind::Select, &[0.0, 10.0, 20.0], 30.0, &mut s),
            20.0
        );
    }

    #[test]
    fn decay_integrator_accumulates() {
        let mut s = 0.0;
        // a=0.5: state = 0.5*0 + 0.5*10 = 5.0
        let out = evaluate_compute_kind(&ComputeNodeKind::DecayIntegrator(0.5), &[], 10.0, &mut s);
        assert!((out - 5.0).abs() < 1e-6);
        assert!((s - 5.0).abs() < 1e-6);
        // state = 0.5*5 + 0.5*10 = 7.5
        let out = evaluate_compute_kind(&ComputeNodeKind::DecayIntegrator(0.5), &[], 10.0, &mut s);
        assert!((out - 7.5).abs() < 1e-6);
    }

    #[test]
    fn oscillator_advances_phase() {
        let mut s = 0.0;
        // f=0.25: state = (0 + 0.25).fract() = 0.25
        let out = evaluate_compute_kind(&ComputeNodeKind::Oscillator(0.25), &[], 0.0, &mut s);
        // sin(2π * 0.25) = sin(π/2) = 1.0
        assert!((out - 1.0).abs() < 1e-5);
    }

    // ── resolve_source tests ────────────────────────────────────────────────

    #[test]
    fn resolve_compute_node_gauss_seidel() {
        let prev = [1.0, 2.0, 3.0];
        let curr = [10.0, 20.0, 30.0];

        // Source before current_idx → curr_outputs
        let v = resolve_source(
            &GraphSource::ComputeNode(0),
            2,
            3,
            &prev,
            &curr,
            &[],
            &ResolveCtx::dummy(),
            &[0.0; 16],
            &[0.0; 16],
        );
        assert_eq!(v, 10.0);

        // Source at current_idx (self-loop) → prev_outputs
        let v = resolve_source(
            &GraphSource::ComputeNode(2),
            2,
            3,
            &prev,
            &curr,
            &[],
            &ResolveCtx::dummy(),
            &[0.0; 16],
            &[0.0; 16],
        );
        assert_eq!(v, 3.0);

        // Source after current_idx → prev_outputs
        let v = resolve_source(
            &GraphSource::ComputeNode(2),
            1,
            3,
            &prev,
            &curr,
            &[],
            &ResolveCtx::dummy(),
            &[0.0; 16],
            &[0.0; 16],
        );
        assert_eq!(v, 3.0);
    }

    #[test]
    fn resolve_out_of_range_compute_node_returns_zero() {
        let v = resolve_source(
            &GraphSource::ComputeNode(99),
            0,
            3,
            &[1.0, 2.0, 3.0],
            &[10.0, 20.0, 30.0],
            &[],
            &ResolveCtx::dummy(),
            &[0.0; 16],
            &[0.0; 16],
        );
        assert_eq!(v, 0.0);
    }

    #[test]
    fn resolve_shared_memory_current_and_previous() {
        let mut mem = [0.0f32; 16];
        mem[3] = 42.0;
        let mut prev_mem = [0.0f32; 16];
        prev_mem[3] = 7.0;

        let v = resolve_source(
            &GraphSource::SharedMemory {
                slot: 3,
                previous: false,
            },
            0,
            0,
            &[],
            &[],
            &[],
            &ResolveCtx::dummy(),
            &mem,
            &prev_mem,
        );
        assert_eq!(v, 42.0);

        let v = resolve_source(
            &GraphSource::SharedMemory {
                slot: 3,
                previous: true,
            },
            0,
            0,
            &[],
            &[],
            &[],
            &ResolveCtx::dummy(),
            &mem,
            &prev_mem,
        );
        assert_eq!(v, 7.0);
    }

    #[test]
    fn resolve_shared_memory_wraps_slot() {
        let mut mem = [0.0f32; 16];
        mem[3] = 99.0;

        let v = resolve_source(
            &GraphSource::SharedMemory {
                slot: 19,
                previous: false,
            }, // 19 % 16 = 3
            0,
            0,
            &[],
            &[],
            &[],
            &ResolveCtx::dummy(),
            &mem,
            &[0.0; 16],
        );
        assert_eq!(v, 99.0);
    }
}
