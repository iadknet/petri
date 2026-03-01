//! Hebbian learning — weight adaptation for graph nodes at runtime.
//!
//! Called by [`super::graph::execute_graph_node`] and
//! [`super::traced_graph::execute_graph_node_traced`] after the relaxation loop
//! converges.
//!
//! **Module boundary:** This module owns the learning math (weight init,
//! update rules, clamping). Graph relaxation lives in `graph.rs`.

use crate::creature::genome::{GraphBackendDef, HebbianRule};

/// Ensure `hebbian_weights` is properly sized and initialized for the given mesh node.
///
/// For internal nodes with `hebbian: Some(_)`, copies genome weights into learned
/// weights on first call (lazy init). For nodes without Hebbian config, the inner
/// vec stays empty (sentinel: use genome weights directly).
pub(crate) fn ensure_hebbian_weights(
    def: &GraphBackendDef,
    node_idx: usize,
    hebbian_weights: &mut Vec<Vec<Box<[f32]>>>,
) {
    // Ensure outer vec covers this mesh node index.
    if hebbian_weights.len() <= node_idx {
        hebbian_weights.resize_with(node_idx + 1, Vec::new);
    }

    let node_weights = &mut hebbian_weights[node_idx];
    let node_count = def.internal_nodes.len();

    // Ensure inner vec covers all internal nodes.
    if node_weights.len() < node_count {
        node_weights.resize_with(node_count, || Box::new([]) as Box<[f32]>);
    }

    // Lazily initialize weights for Hebbian nodes from genome.
    for (i, inode) in def.internal_nodes.iter().enumerate() {
        if inode.plasticity.is_some() && node_weights[i].is_empty() && !inode.inputs.is_empty() {
            let weights: Vec<f32> = inode.inputs.iter().map(|inp| inp.weight).collect();
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
    def: &GraphBackendDef,
    node_idx: usize,
    hebbian_weights: &mut [Vec<Box<[f32]>>],
    final_outputs: &[f32],
    cost_per_update: f32,
) -> f32 {
    let node_count = def.internal_nodes.len();
    let mut total_cost: f32 = 0.0;

    for (i, inode) in def.internal_nodes.iter().enumerate() {
        let cfg = match &inode.plasticity {
            Some(c) => c,
            None => continue,
        };

        if inode.inputs.is_empty() {
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

        for (edge_idx, input) in inode.inputs.iter().enumerate() {
            if edge_idx >= weights.len() {
                break;
            }

            let src = input.source_idx as usize;
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
#[inline]
pub(crate) fn collect_weighted_inputs_hebbian(
    node: &crate::creature::genome::GraphInternalNode,
    current_idx: usize,
    node_count: usize,
    prev_outputs: &[f32],
    curr_outputs: &[f32],
    learned_weights: &[f32],
    buf: &mut Vec<f32>,
) {
    buf.clear();
    buf.extend(node.inputs.iter().enumerate().map(|(edge_idx, input)| {
        let src = input.source_idx as usize;
        let source_value = if src >= node_count {
            0.0
        } else if src < current_idx {
            curr_outputs[src]
        } else {
            prev_outputs[src]
        };
        source_value * effective_weight(input.weight, learned_weights, edge_idx)
    }));
}

/// Returns `true` if any internal node in this graph def has Hebbian learning enabled.
#[inline]
pub(crate) fn has_any_hebbian(def: &GraphBackendDef) -> bool {
    def.internal_nodes.iter().any(|n| n.plasticity.is_some())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::creature::genome::{
        GraphBackendDef, GraphInput, GraphInternalNode, GraphNodeKind, HebbianRule,
        PlasticityConfig,
    };

    fn make_hebbian_def(rule: HebbianRule, rate: f32, clamp: f32) -> GraphBackendDef {
        GraphBackendDef {
            internal_nodes: vec![
                // Node 0: constant input source
                GraphInternalNode {
                    kind: GraphNodeKind::Constant(1.0),
                    inputs: vec![],
                    plasticity: None,
                },
                // Node 1: Hebbian node with one input from node 0
                GraphInternalNode {
                    kind: GraphNodeKind::Add,
                    inputs: vec![GraphInput {
                        source_idx: 0,
                        weight: 0.5,
                    }],
                    plasticity: Some(PlasticityConfig {
                        rule,
                        learning_rate: rate,
                        weight_clamp: clamp,
                        lamarckian: false,
                        modulation: None,
                    }),
                },
            ],
        }
    }

    #[test]
    fn effective_weight_uses_learned_when_available() {
        assert!((effective_weight(1.0, &[0.3, 0.7], 0) - 0.3).abs() < 1e-6);
        assert!((effective_weight(1.0, &[0.3, 0.7], 1) - 0.7).abs() < 1e-6);
    }

    #[test]
    fn effective_weight_falls_back_to_genome() {
        // Edge index beyond learned weights falls back
        assert!((effective_weight(1.0, &[0.3], 1) - 1.0).abs() < 1e-6);
        // Empty learned weights falls back
        assert!((effective_weight(2.0, &[], 0) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn ensure_hebbian_weights_lazy_init() {
        let def = make_hebbian_def(HebbianRule::Classic, 0.1, 5.0);
        let mut hw: Vec<Vec<Box<[f32]>>> = Vec::new();

        ensure_hebbian_weights(&def, 0, &mut hw);

        // Node 0 (no hebbian) should have empty weights
        assert!(hw[0][0].is_empty());
        // Node 1 (hebbian) should have weights copied from genome
        assert_eq!(hw[0][1].len(), 1);
        assert!((hw[0][1][0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn ensure_hebbian_weights_idempotent() {
        let def = make_hebbian_def(HebbianRule::Classic, 0.1, 5.0);
        let mut hw: Vec<Vec<Box<[f32]>>> = Vec::new();

        ensure_hebbian_weights(&def, 0, &mut hw);
        // Modify the weight
        hw[0][1][0] = 0.9;
        // Call again — should NOT reset
        ensure_hebbian_weights(&def, 0, &mut hw);
        assert!((hw[0][1][0] - 0.9).abs() < 1e-6);
    }

    #[test]
    fn classic_hebbian_dw() {
        let def = make_hebbian_def(HebbianRule::Classic, 0.1, 5.0);
        let mut hw: Vec<Vec<Box<[f32]>>> = Vec::new();
        ensure_hebbian_weights(&def, 0, &mut hw);

        // pre=1.0 (node 0 output), post=0.5 (node 1 output)
        let outputs = vec![1.0, 0.5];
        let cost = apply_hebbian_updates(&def, 0, &mut hw, &outputs, 0.0);
        assert!((cost - 0.0).abs() < 1e-9);

        // dw = eta * pre * post = 0.1 * 1.0 * 0.5 = 0.05
        // new_w = 0.5 + 0.05 = 0.55
        assert!((hw[0][1][0] - 0.55).abs() < 1e-6);
    }

    #[test]
    fn oja_hebbian_dw() {
        let def = make_hebbian_def(HebbianRule::Oja, 0.1, 5.0);
        let mut hw: Vec<Vec<Box<[f32]>>> = Vec::new();
        ensure_hebbian_weights(&def, 0, &mut hw);

        let outputs = vec![1.0, 0.5];
        apply_hebbian_updates(&def, 0, &mut hw, &outputs, 0.0);

        // dw = eta * post * (pre - w * post) = 0.1 * 0.5 * (1.0 - 0.5 * 0.5)
        //    = 0.1 * 0.5 * 0.75 = 0.0375
        // new_w = 0.5 + 0.0375 = 0.5375
        assert!((hw[0][1][0] - 0.5375).abs() < 1e-6);
    }

    #[test]
    fn antihebb_dw() {
        let def = make_hebbian_def(HebbianRule::AntiHebb, 0.1, 5.0);
        let mut hw: Vec<Vec<Box<[f32]>>> = Vec::new();
        ensure_hebbian_weights(&def, 0, &mut hw);

        let outputs = vec![1.0, 0.5];
        apply_hebbian_updates(&def, 0, &mut hw, &outputs, 0.0);

        // dw = -eta * pre * post = -0.1 * 1.0 * 0.5 = -0.05
        // new_w = 0.5 - 0.05 = 0.45
        assert!((hw[0][1][0] - 0.45).abs() < 1e-6);
    }

    #[test]
    fn covariance_hebbian_dw() {
        let def = make_hebbian_def(HebbianRule::Covariance, 0.1, 5.0);
        let mut hw: Vec<Vec<Box<[f32]>>> = Vec::new();
        ensure_hebbian_weights(&def, 0, &mut hw);

        let outputs = vec![1.0, 0.5];
        apply_hebbian_updates(&def, 0, &mut hw, &outputs, 0.0);

        // dw = eta * (pre - 0.5) * (post - 0.5) = 0.1 * 0.5 * 0.0 = 0.0
        // new_w = 0.5 + 0.0 = 0.5
        assert!((hw[0][1][0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn weight_clamping_enforced() {
        let def = make_hebbian_def(HebbianRule::Classic, 1.0, 0.6);
        let mut hw: Vec<Vec<Box<[f32]>>> = Vec::new();
        ensure_hebbian_weights(&def, 0, &mut hw);

        // Large activations should push weight past clamp
        let outputs = vec![1.0, 1.0];
        apply_hebbian_updates(&def, 0, &mut hw, &outputs, 0.0);

        // dw = 1.0 * 1.0 * 1.0 = 1.0, new_w = 0.5 + 1.0 = 1.5
        // Clamped to 0.6
        assert!((hw[0][1][0] - 0.6).abs() < 1e-6);
    }

    #[test]
    fn energy_cost_accumulated() {
        let def = make_hebbian_def(HebbianRule::Classic, 0.1, 5.0);
        let mut hw: Vec<Vec<Box<[f32]>>> = Vec::new();
        ensure_hebbian_weights(&def, 0, &mut hw);

        let outputs = vec![1.0, 0.5];
        let cost = apply_hebbian_updates(&def, 0, &mut hw, &outputs, 0.01);

        // 1 Hebbian node with 1 edge = 1 update = 0.01
        assert!((cost - 0.01).abs() < 1e-9);
    }

    #[test]
    fn non_hebbian_nodes_skipped() {
        let def = GraphBackendDef {
            internal_nodes: vec![GraphInternalNode {
                kind: GraphNodeKind::Add,
                inputs: vec![GraphInput {
                    source_idx: 0,
                    weight: 1.0,
                }],
                plasticity: None,
            }],
        };
        let mut hw: Vec<Vec<Box<[f32]>>> = Vec::new();
        ensure_hebbian_weights(&def, 0, &mut hw);

        let outputs = vec![1.0];
        let cost = apply_hebbian_updates(&def, 0, &mut hw, &outputs, 0.01);

        assert!((cost - 0.0).abs() < 1e-9);
        // Weights should remain empty for non-hebbian nodes
        assert!(hw[0][0].is_empty());
    }

    #[test]
    fn has_any_hebbian_detects_presence() {
        let def_with = make_hebbian_def(HebbianRule::Classic, 0.1, 5.0);
        assert!(has_any_hebbian(&def_with));

        let def_without = GraphBackendDef {
            internal_nodes: vec![GraphInternalNode {
                kind: GraphNodeKind::Add,
                inputs: vec![],
                plasticity: None,
            }],
        };
        assert!(!has_any_hebbian(&def_without));
    }

    #[test]
    fn collect_weighted_inputs_hebbian_uses_learned() {
        let node = GraphInternalNode {
            kind: GraphNodeKind::Add,
            inputs: vec![
                GraphInput {
                    source_idx: 0,
                    weight: 1.0,
                },
                GraphInput {
                    source_idx: 1,
                    weight: 2.0,
                },
            ],
            plasticity: Some(PlasticityConfig {
                rule: HebbianRule::Classic,
                learning_rate: 0.1,
                weight_clamp: 5.0,
                lamarckian: false,
                modulation: None,
            }),
        };

        let prev = vec![0.5, 0.8, 0.0];
        let curr = vec![0.0, 0.0, 0.0];
        let learned = [0.3, 0.7]; // overrides genome weights 1.0, 2.0

        let mut buf = Vec::new();
        collect_weighted_inputs_hebbian(&node, 2, 3, &prev, &curr, &learned, &mut buf);

        // src=0 < current_idx=2, so use curr[0]=0.0 * 0.3 = 0.0
        // src=1 < current_idx=2, so use curr[1]=0.0 * 0.7 = 0.0
        // Wait, curr is all zeros. Let me use prev_outputs for src >= current_idx
        // Actually: src=0 < current_idx=2 → curr[0]=0.0. src=1 < 2 → curr[1]=0.0
        // Both use curr which is 0.0, so result is [0.0, 0.0]
        assert_eq!(buf.len(), 2);
        assert!((buf[0] - 0.0).abs() < 1e-6);
        assert!((buf[1] - 0.0).abs() < 1e-6);

        // Test with populated curr
        let curr2 = vec![0.5, 0.8, 0.0];
        collect_weighted_inputs_hebbian(&node, 2, 3, &prev, &curr2, &learned, &mut buf);
        // src=0 < 2 → curr2[0]=0.5 * 0.3 = 0.15
        // src=1 < 2 → curr2[1]=0.8 * 0.7 = 0.56
        assert!((buf[0] - 0.15).abs() < 1e-6);
        assert!((buf[1] - 0.56).abs() < 1e-6);
    }
}
