//! Eligibility trace management for reward-modulated plasticity.
//!
//! Eligibility traces record a decaying memory of recent Hebbian activity.
//! During Phase 2.5 (reward learning pass), traces are multiplied by the
//! outcome signal to produce three-factor weight updates:
//! `dw = learning_rate * outcome_signal * trace`.
//!
//! Traces are parallel to `plasticity_weights` in shape:
//! `[mesh_node_idx][internal_node_idx][edge_idx]`.

use crate::contracts::InputReference;
use crate::creature::genome::cgp::CgpGraphBackendDef;
use crate::creature::genome::HebbianRule;
use crate::runtime::cgp::sources::resolve_source_post_convergence;
use crate::runtime::inputs::ResolveCtx;

/// Ensure `eligibility_traces` is properly sized for the given mesh node.
///
/// Mirrors [`super::hebbian::ensure_hebbian_weights`]: lazily initializes
/// trace storage for reward-modulated nodes. Non-modulated nodes get empty
/// `Box<[f32]>` sentinels.
pub(crate) fn ensure_eligibility_traces(
    def: &CgpGraphBackendDef,
    node_idx: usize,
    eligibility_traces: &mut Vec<Vec<Box<[f32]>>>,
) {
    // Ensure outer vec covers this mesh node index.
    if eligibility_traces.len() <= node_idx {
        eligibility_traces.resize_with(node_idx + 1, Vec::new);
    }

    let node_traces = &mut eligibility_traces[node_idx];
    let node_count = def.compute_nodes.len();

    // Ensure inner vec covers all compute nodes.
    if node_traces.len() < node_count {
        node_traces.resize_with(node_count, || Box::new([]) as Box<[f32]>);
    }

    // Lazily initialize traces for reward-modulated nodes (all zeros).
    for (i, cnode) in def.compute_nodes.iter().enumerate() {
        let is_reward_modulated = cnode
            .plasticity
            .as_ref()
            .is_some_and(|p| p.modulation.is_some());
        if is_reward_modulated && node_traces[i].is_empty() && !cnode.inputs.is_empty() {
            node_traces[i] = vec![0.0f32; cnode.inputs.len()].into_boxed_slice();
        }
    }
}

/// Update eligibility traces after graph convergence.
///
/// For each reward-modulated node, compute the Hebbian delta using the same
/// rule as pure Hebbian learning, then accumulate into the trace with decay:
///
/// ```text
/// trace[edge] = decay * old_trace + hebbian_delta(pre, post, w)
/// ```
///
/// Pure Hebbian nodes (no modulation) are skipped — they are updated
/// directly by [`super::hebbian::apply_hebbian_updates`].
#[allow(clippy::too_many_arguments)]
pub(crate) fn update_eligibility_traces(
    def: &CgpGraphBackendDef,
    node_idx: usize,
    eligibility_traces: &mut [Vec<Box<[f32]>>],
    plasticity_weights: &[Vec<Box<[f32]>>],
    final_outputs: &[f32],
    input_refs: &[InputReference],
    resolve_ctx: &ResolveCtx<'_>,
    shared_memory: &[f32; 16],
    prev_shared_memory: &[f32; 16],
) {
    let compute_count = def.compute_nodes.len();

    for (i, cnode) in def.compute_nodes.iter().enumerate() {
        let cfg = match &cnode.plasticity {
            Some(c) => c,
            None => continue,
        };

        // Only update traces for reward-modulated nodes.
        let modulation = match &cfg.modulation {
            Some(m) => m,
            None => continue,
        };

        if cnode.inputs.is_empty() {
            continue;
        }

        // Clamp decay to [0.0, 1.0] at runtime.
        let decay = modulation.trace_decay.clamp(0.0, 1.0);
        let eta = cfg.learning_rate.clamp(0.0, 1.0);

        let post = if i < final_outputs.len() {
            final_outputs[i]
        } else {
            0.0
        };

        // Bounds check for traces and weights.
        if eligibility_traces.len() <= node_idx || eligibility_traces[node_idx].len() <= i {
            continue;
        }
        let traces = &mut eligibility_traces[node_idx][i];
        if traces.is_empty() {
            continue;
        }

        // Get learned weights for computing Hebbian delta (need current w for Oja rule).
        let weights =
            if plasticity_weights.len() > node_idx && plasticity_weights[node_idx].len() > i {
                &plasticity_weights[node_idx][i][..]
            } else {
                &[][..]
            };

        for (edge_idx, edge) in cnode.inputs.iter().enumerate() {
            if edge_idx >= traces.len() {
                break;
            }

            let pre = resolve_source_post_convergence(
                &edge.source,
                compute_count,
                final_outputs,
                input_refs,
                resolve_ctx,
                shared_memory,
                prev_shared_memory,
            );

            let w = if edge_idx < weights.len() {
                weights[edge_idx]
            } else {
                edge.weight // fall back to genome weight
            };

            // Compute the Hebbian delta (same rules as pure Hebbian).
            let hebbian_delta = match cfg.rule {
                HebbianRule::Classic => eta * pre * post,
                HebbianRule::Oja => eta * post * (pre - w * post),
                HebbianRule::AntiHebb => -eta * pre * post,
                HebbianRule::Covariance => eta * (pre - 0.5) * (post - 0.5),
            };

            // Accumulate: trace = decay * old_trace + hebbian_delta
            traces[edge_idx] = decay * traces[edge_idx] + hebbian_delta;
        }
    }
}

/// Returns `true` if any internal node has reward-modulated plasticity.
#[inline]
pub(crate) fn has_any_reward_modulated(def: &CgpGraphBackendDef) -> bool {
    def.compute_nodes.iter().any(|n| {
        n.plasticity
            .as_ref()
            .is_some_and(|p| p.modulation.is_some())
    })
}

// Old eligibility trace tests removed — production code ported to CGP types.
