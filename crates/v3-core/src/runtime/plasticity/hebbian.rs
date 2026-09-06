//! Hebbian learning — weight adaptation for graph nodes at runtime.
//!
//! Called by [`crate::runtime::cgp::execute::execute_graph_node`] and
//! [`crate::runtime::cgp::traced::execute_graph_node_traced`] after ordered graph evaluation.
//!
//! **Module boundary:** This module owns the learning math (weight init,
//! update rules, clamping). Ordered graph evaluation lives in `runtime/cgp/execute.rs`.

use crate::contracts::InputReference;
use crate::creature::genome::cgp::{CgpGraphBackendDef, ComputeNode};
use crate::creature::genome::HebbianRule;
use crate::runtime::cgp::sources::{resolve_source, resolve_source_post_convergence};
use crate::runtime::inputs::ResolveCtx;

/// Ensure `hebbian_weights` is properly sized and initialized for the given mesh node.
///
/// For internal nodes with `hebbian: Some(_)`, copies genome weights into learned
/// weights on first call (lazy init). For nodes without Hebbian config, the inner
/// vec stays empty (sentinel: use genome weights directly).
pub(crate) fn ensure_hebbian_weights(
    def: &CgpGraphBackendDef,
    node_idx: usize,
    hebbian_weights: &mut Vec<Vec<Box<[f32]>>>,
) {
    // Ensure outer vec covers this mesh node index.
    if hebbian_weights.len() <= node_idx {
        hebbian_weights.resize_with(node_idx + 1, Vec::new);
    }

    let node_weights = &mut hebbian_weights[node_idx];
    let node_count = def.compute_nodes.len();

    // Ensure inner vec covers all compute nodes.
    if node_weights.len() < node_count {
        node_weights.resize_with(node_count, || Box::new([]) as Box<[f32]>);
    }

    // Lazily initialize weights for Hebbian nodes from genome.
    for (i, cnode) in def.compute_nodes.iter().enumerate() {
        if cnode.plasticity.is_some() && node_weights[i].is_empty() && !cnode.inputs.is_empty() {
            let weights: Vec<f32> = cnode.inputs.iter().map(|edge| edge.weight).collect();
            node_weights[i] = weights.into_boxed_slice();
        }
    }
}

/// Look up the effective weight for an edge: learned weight if available,
/// else genome weight.
#[inline]
pub(crate) fn effective_weight(
    genome_weight: f32,
    learned_weights: &[f32],
    edge_idx: usize,
) -> f32 {
    if edge_idx < learned_weights.len() {
        learned_weights[edge_idx]
    } else {
        genome_weight
    }
}

/// Apply Hebbian weight updates after ordered graph evaluation.
///
/// Uses `final_outputs` for pre/post activations: for each Hebbian node, the
/// node's own output is the "post" activation, and each input source's output
/// is the "pre" activation.
///
/// Returns total energy cost of the updates.
#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_hebbian_updates(
    def: &CgpGraphBackendDef,
    node_idx: usize,
    hebbian_weights: &mut [Vec<Box<[f32]>>],
    final_outputs: &[f32],
    input_refs: &[InputReference],
    resolve_ctx: &ResolveCtx<'_>,
    shared_memory: &[f32; 16],
    prev_shared_memory: &[f32; 16],
    cost_per_update: f32,
) -> (f32, u32) {
    let compute_count = def.compute_nodes.len();
    let mut total_cost: f32 = 0.0;
    let mut update_count: u32 = 0;

    for (i, cnode) in def.compute_nodes.iter().enumerate() {
        let cfg = match &cnode.plasticity {
            Some(c) => c,
            None => continue,
        };

        // Skip reward-modulated nodes — they are updated in Phase 2.5 via
        // reward::apply_reward_modulated_updates(), not here.
        if cfg.modulation.is_some() {
            continue;
        }

        if cnode.inputs.is_empty() {
            continue;
        }

        // Runtime clamps on learning parameters.
        let eta = cfg.learning_rate.clamp(0.0, 1.0);
        let w_clamp = cfg.weight_clamp.clamp(0.01, 10.0);

        let post = if i < final_outputs.len() {
            final_outputs[i]
        } else {
            0.0
        };

        // Ensure weights vec is initialized for this node.
        if hebbian_weights.len() <= node_idx || hebbian_weights[node_idx].len() <= i {
            continue;
        }
        let weights = &mut hebbian_weights[node_idx][i];
        if weights.is_empty() {
            continue;
        }

        for (edge_idx, edge) in cnode.inputs.iter().enumerate() {
            if edge_idx >= weights.len() {
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

            let w = weights[edge_idx];

            let dw = match cfg.rule {
                HebbianRule::Classic => eta * pre * post,
                HebbianRule::Oja => eta * post * (pre - w * post),
                HebbianRule::AntiHebb => -eta * pre * post,
                HebbianRule::Covariance => eta * (pre - 0.5) * (post - 0.5),
            };

            weights[edge_idx] = (w + dw).clamp(-w_clamp, w_clamp);
            total_cost += cost_per_update;
            update_count += 1;
        }
    }

    (total_cost, update_count)
}

/// Collect weighted inputs using effective weights (learned if available).
///
/// Like [`super::graph::collect_weighted_inputs`] but substitutes learned
/// Hebbian weights for genome weights when available.
///
/// Resolves all `GraphSource` variants through the shared CGP source resolver,
/// preserving the same current-visit/frozen-tick source semantics as the non-plastic path.
#[inline]
#[allow(clippy::too_many_arguments)]
pub(crate) fn collect_weighted_inputs_hebbian(
    node: &ComputeNode,
    current_idx: usize,
    node_count: usize,
    prev_outputs: &[f32],
    curr_outputs: &[f32],
    input_refs: &[InputReference],
    resolve_ctx: &ResolveCtx<'_>,
    shared_memory: &[f32; 16],
    prev_shared_memory: &[f32; 16],
    learned_weights: &[f32],
    buf: &mut Vec<f32>,
) {
    buf.clear();
    buf.extend(node.inputs.iter().enumerate().map(|(edge_idx, edge)| {
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
        source_value * effective_weight(edge.weight, learned_weights, edge_idx)
    }));
}

/// Returns `true` if any internal node in this graph def has Hebbian learning enabled.
#[inline]
pub(crate) fn has_any_hebbian(def: &CgpGraphBackendDef) -> bool {
    def.compute_nodes.iter().any(|n| n.plasticity.is_some())
}

// Old Hebbian tests removed — CGP Hebbian tests live in mutation/graph/hebbian.rs.
