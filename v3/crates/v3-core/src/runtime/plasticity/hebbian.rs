//! Hebbian learning — weight adaptation for graph nodes at runtime.
//!
//! Called by [`super::graph::execute_graph_node`] and
//! [`super::traced_graph::execute_graph_node_traced`] after the relaxation loop
//! converges.
//!
//! **Module boundary:** This module owns the learning math (weight init,
//! update rules, clamping). Graph relaxation lives in `graph.rs`.

use crate::creature::genome::cgp::{CgpGraphBackendDef, ComputeNode, GraphSource};
use crate::creature::genome::HebbianRule;

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

/// Apply Hebbian weight updates after graph relaxation converges.
///
/// Uses `final_outputs` for pre/post activations: for each Hebbian node, the
/// node's own output is the "post" activation, and each input source's output
/// is the "pre" activation.
///
/// Returns total energy cost of the updates.
pub(crate) fn apply_hebbian_updates(
    def: &CgpGraphBackendDef,
    node_idx: usize,
    hebbian_weights: &mut [Vec<Box<[f32]>>],
    final_outputs: &[f32],
    cost_per_update: f32,
) -> f32 {
    let node_count = def.compute_nodes.len();
    let mut total_cost: f32 = 0.0;

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

            let src = match edge.source {
                GraphSource::ComputeNode(idx) => idx as usize,
                _ => usize::MAX,
            };
            let pre = if src < node_count && src < final_outputs.len() {
                final_outputs[src]
            } else {
                0.0
            };

            let w = weights[edge_idx];

            let dw = match cfg.rule {
                HebbianRule::Classic => eta * pre * post,
                HebbianRule::Oja => eta * post * (pre - w * post),
                HebbianRule::AntiHebb => -eta * pre * post,
                HebbianRule::Covariance => eta * (pre - 0.5) * (post - 0.5),
            };

            weights[edge_idx] = (w + dw).clamp(-w_clamp, w_clamp);
            total_cost += cost_per_update;
        }
    }

    total_cost
}

/// Collect weighted inputs using effective weights (learned if available).
///
/// Like [`super::graph::collect_weighted_inputs`] but substitutes learned
/// Hebbian weights for genome weights when available.
///
/// Only resolves `GraphSource::ComputeNode` sources using Gauss-Seidel order.
/// Other source types (InputLeaf, SharedMemory) are not resolved here — they
/// require a full `ResolveCtx` which is not available in this context. Non-compute
/// sources evaluate to 0.0.
#[inline]
pub(crate) fn collect_weighted_inputs_hebbian(
    node: &ComputeNode,
    current_idx: usize,
    node_count: usize,
    prev_outputs: &[f32],
    curr_outputs: &[f32],
    learned_weights: &[f32],
    buf: &mut Vec<f32>,
) {
    buf.clear();
    buf.extend(node.inputs.iter().enumerate().map(|(edge_idx, edge)| {
        let source_value = match edge.source {
            GraphSource::ComputeNode(idx) => {
                let src = idx as usize;
                if src >= node_count {
                    0.0
                } else if src < current_idx {
                    curr_outputs[src]
                } else {
                    prev_outputs[src]
                }
            }
            _ => 0.0, // non-compute sources not resolved in Hebbian context
        };
        source_value * effective_weight(edge.weight, learned_weights, edge_idx)
    }));
}

/// Returns `true` if any internal node in this graph def has Hebbian learning enabled.
#[inline]
pub(crate) fn has_any_hebbian(def: &CgpGraphBackendDef) -> bool {
    def.compute_nodes.iter().any(|n| n.plasticity.is_some())
}

// Old Hebbian tests removed — CGP Hebbian tests live in mutation/graph/cgp_hebbian.rs.
