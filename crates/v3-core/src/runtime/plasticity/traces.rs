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
use crate::creature::genome::{BackendDef, HebbianRule, NodeGenome};
use crate::runtime::cgp::sources::resolve_source;
use crate::runtime::inputs::ResolveCtx;

/// Advance initialized credit once per world tick without initializing weights or traces.
pub(crate) fn decay_eligibility_traces(
    nodes: &[NodeGenome],
    eligibility_traces: &mut [Vec<Box<[f32]>>],
) {
    for (node, module_traces) in nodes.iter().zip(eligibility_traces) {
        let BackendDef::Graph(def) = &node.backend_def else {
            continue;
        };
        for (compute, traces) in def.compute_nodes.iter().zip(module_traces) {
            if let Some(modulation) = compute
                .plasticity
                .as_ref()
                .and_then(|p| p.modulation.as_ref())
            {
                let decay = modulation.trace_decay.clamp(0.0, 1.0);
                for trace in traces {
                    *trace *= decay;
                }
            }
        }
    }
}

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

/// Commit the last successful visit's activity from the frozen, decayed tick base.
/// Activity has no learning-rate factor; eta is applied once when reward arrives.
/// Source replay uses the evaluation context and ordered temporal read bases.
#[allow(clippy::too_many_arguments)]
pub(crate) fn update_eligibility_traces(
    def: &CgpGraphBackendDef,
    node_idx: usize,
    eligibility_traces: &mut [Vec<Box<[f32]>>],
    tick_start_traces: &[Vec<Box<[f32]>>],
    plasticity_weights: &[Vec<Box<[f32]>>],
    prev_outputs: &[f32],
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
        if cfg.modulation.is_none() {
            continue;
        }

        if cnode.inputs.is_empty() {
            continue;
        }

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

        let base_traces = tick_start_traces
            .get(node_idx)
            .and_then(|module| module.get(i));
        for (edge_idx, edge) in cnode.inputs.iter().enumerate() {
            if edge_idx >= traces.len() {
                break;
            }

            let pre = resolve_source(
                &edge.source,
                i,
                compute_count,
                prev_outputs,
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

            // Compute activity only; reward applies the learning rate.
            let activity = match cfg.rule {
                HebbianRule::Classic => pre * post,
                HebbianRule::Oja => post * (pre - w * post),
                HebbianRule::AntiHebb => -pre * post,
                HebbianRule::Covariance => (pre - 0.5) * (post - 0.5),
            };

            // A module first initialized this tick has no prior credit, even on a revisit.
            let base = base_traces
                .and_then(|edges| edges.get(edge_idx))
                .copied()
                .unwrap_or(0.0);
            traces[edge_idx] = base + activity;
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
