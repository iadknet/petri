//! Eligibility trace management for reward-modulated plasticity.
//!
//! Eligibility traces record a decaying memory of recent Hebbian activity.
//! During Phase 2.5 (reward learning pass), traces are multiplied by the
//! outcome signal to produce three-factor weight updates:
//! `dw = learning_rate * outcome_signal * trace`.
//!
//! Traces are parallel to `plasticity_weights` in shape:
//! `[mesh_node_idx][internal_node_idx][edge_idx]`.

use crate::creature::genome::{GraphBackendDef, HebbianRule};

/// Ensure `eligibility_traces` is properly sized for the given mesh node.
///
/// Mirrors [`super::hebbian::ensure_hebbian_weights`]: lazily initializes
/// trace storage for reward-modulated nodes. Non-modulated nodes get empty
/// `Box<[f32]>` sentinels.
pub(crate) fn ensure_eligibility_traces(
    def: &GraphBackendDef,
    node_idx: usize,
    eligibility_traces: &mut Vec<Vec<Box<[f32]>>>,
) {
    // Ensure outer vec covers this mesh node index.
    if eligibility_traces.len() <= node_idx {
        eligibility_traces.resize_with(node_idx + 1, Vec::new);
    }

    let node_traces = &mut eligibility_traces[node_idx];
    let node_count = def.internal_nodes.len();

    // Ensure inner vec covers all internal nodes.
    if node_traces.len() < node_count {
        node_traces.resize_with(node_count, || Box::new([]) as Box<[f32]>);
    }

    // Lazily initialize traces for reward-modulated nodes (all zeros).
    for (i, inode) in def.internal_nodes.iter().enumerate() {
        let is_reward_modulated = inode
            .plasticity
            .as_ref()
            .is_some_and(|p| p.modulation.is_some());
        if is_reward_modulated && node_traces[i].is_empty() && !inode.inputs.is_empty() {
            node_traces[i] = vec![0.0f32; inode.inputs.len()].into_boxed_slice();
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
pub(crate) fn update_eligibility_traces(
    def: &GraphBackendDef,
    node_idx: usize,
    eligibility_traces: &mut [Vec<Box<[f32]>>],
    plasticity_weights: &[Vec<Box<[f32]>>],
    final_outputs: &[f32],
) {
    let node_count = def.internal_nodes.len();

    for (i, inode) in def.internal_nodes.iter().enumerate() {
        let cfg = match &inode.plasticity {
            Some(c) => c,
            None => continue,
        };

        // Only update traces for reward-modulated nodes.
        let modulation = match &cfg.modulation {
            Some(m) => m,
            None => continue,
        };

        if inode.inputs.is_empty() {
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

        for (edge_idx, input) in inode.inputs.iter().enumerate() {
            if edge_idx >= traces.len() {
                break;
            }

            let src = input.source_idx as usize;
            let pre = if src < node_count && src < final_outputs.len() {
                final_outputs[src]
            } else {
                0.0
            };

            let w = if edge_idx < weights.len() {
                weights[edge_idx]
            } else {
                input.weight // fall back to genome weight
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
pub(crate) fn has_any_reward_modulated(def: &GraphBackendDef) -> bool {
    def.internal_nodes.iter().any(|n| {
        n.plasticity
            .as_ref()
            .is_some_and(|p| p.modulation.is_some())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::creature::genome::{
        GraphInput, GraphInternalNode, GraphNodeKind, HebbianRule, OutcomeChannel,
        PlasticityConfig, RewardModulationConfig,
    };

    fn make_reward_modulated_def(rule: HebbianRule, rate: f32, decay: f32) -> GraphBackendDef {
        GraphBackendDef {
            internal_nodes: vec![
                // Node 0: constant input source
                GraphInternalNode {
                    kind: GraphNodeKind::Constant(1.0),
                    inputs: vec![],
                    plasticity: None,
                },
                // Node 1: reward-modulated node with one input from node 0
                GraphInternalNode {
                    kind: GraphNodeKind::Add,
                    inputs: vec![GraphInput {
                        source_idx: 0,
                        weight: 0.5,
                    }],
                    plasticity: Some(PlasticityConfig {
                        rule,
                        learning_rate: rate,
                        weight_clamp: 5.0,
                        lamarckian: false,
                        modulation: Some(RewardModulationConfig {
                            reward_source: OutcomeChannel::EnergyDelta,
                            trace_decay: decay,
                        }),
                    }),
                },
            ],
        }
    }

    #[test]
    fn ensure_traces_lazy_init() {
        let def = make_reward_modulated_def(HebbianRule::Classic, 0.1, 0.9);
        let mut traces: Vec<Vec<Box<[f32]>>> = Vec::new();

        ensure_eligibility_traces(&def, 0, &mut traces);

        // Node 0 (no modulation) should have empty traces.
        assert!(traces[0][0].is_empty());
        // Node 1 (reward-modulated) should have zero-initialized traces.
        assert_eq!(traces[0][1].len(), 1);
        assert!((traces[0][1][0] - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn ensure_traces_idempotent() {
        let def = make_reward_modulated_def(HebbianRule::Classic, 0.1, 0.9);
        let mut traces: Vec<Vec<Box<[f32]>>> = Vec::new();

        ensure_eligibility_traces(&def, 0, &mut traces);
        // Modify the trace.
        traces[0][1][0] = 0.42;
        // Call again — should NOT reset.
        ensure_eligibility_traces(&def, 0, &mut traces);
        assert!((traces[0][1][0] - 0.42).abs() < f32::EPSILON);
    }

    #[test]
    fn trace_accumulates_with_decay() {
        let def = make_reward_modulated_def(HebbianRule::Classic, 0.1, 0.9);
        let mut traces: Vec<Vec<Box<[f32]>>> = Vec::new();
        ensure_eligibility_traces(&def, 0, &mut traces);

        // Set up weights (needed for the delta computation).
        let weights: Vec<Vec<Box<[f32]>>> = vec![vec![
            Box::new([]) as Box<[f32]>,
            vec![0.5f32].into_boxed_slice(),
        ]];

        // pre=1.0 (node 0 output), post=0.5 (node 1 output)
        let outputs = vec![1.0, 0.5];

        // First update: trace = 0.9 * 0.0 + (0.1 * 1.0 * 0.5) = 0.05
        update_eligibility_traces(&def, 0, &mut traces, &weights, &outputs);
        assert!((traces[0][1][0] - 0.05).abs() < 1e-6);

        // Second update: trace = 0.9 * 0.05 + 0.05 = 0.095
        update_eligibility_traces(&def, 0, &mut traces, &weights, &outputs);
        assert!((traces[0][1][0] - 0.095).abs() < 1e-6);
    }

    #[test]
    fn trace_resets_with_zero_decay() {
        let def = make_reward_modulated_def(HebbianRule::Classic, 0.1, 0.0);
        let mut traces: Vec<Vec<Box<[f32]>>> = Vec::new();
        ensure_eligibility_traces(&def, 0, &mut traces);

        let weights: Vec<Vec<Box<[f32]>>> = vec![vec![
            Box::new([]) as Box<[f32]>,
            vec![0.5f32].into_boxed_slice(),
        ]];
        let outputs = vec![1.0, 0.5];

        // First update: trace = 0.0 * 0.0 + 0.05 = 0.05
        update_eligibility_traces(&def, 0, &mut traces, &weights, &outputs);
        assert!((traces[0][1][0] - 0.05).abs() < 1e-6);

        // Second update: trace = 0.0 * 0.05 + 0.05 = 0.05 (no memory)
        update_eligibility_traces(&def, 0, &mut traces, &weights, &outputs);
        assert!((traces[0][1][0] - 0.05).abs() < 1e-6);
    }

    #[test]
    fn pure_hebbian_nodes_skipped() {
        let def = GraphBackendDef {
            internal_nodes: vec![GraphInternalNode {
                kind: GraphNodeKind::Add,
                inputs: vec![GraphInput {
                    source_idx: 0,
                    weight: 1.0,
                }],
                plasticity: Some(PlasticityConfig {
                    rule: HebbianRule::Classic,
                    learning_rate: 0.1,
                    weight_clamp: 5.0,
                    lamarckian: false,
                    modulation: None, // Pure Hebbian — no modulation.
                }),
            }],
        };
        let mut traces: Vec<Vec<Box<[f32]>>> = Vec::new();
        ensure_eligibility_traces(&def, 0, &mut traces);

        // Pure Hebbian node should not get traces initialized.
        assert!(traces[0][0].is_empty());
    }

    #[test]
    fn has_any_reward_modulated_detects_presence() {
        let def_with = make_reward_modulated_def(HebbianRule::Classic, 0.1, 0.9);
        assert!(has_any_reward_modulated(&def_with));

        let def_without = GraphBackendDef {
            internal_nodes: vec![GraphInternalNode {
                kind: GraphNodeKind::Add,
                inputs: vec![],
                plasticity: Some(PlasticityConfig {
                    rule: HebbianRule::Classic,
                    learning_rate: 0.1,
                    weight_clamp: 5.0,
                    lamarckian: false,
                    modulation: None,
                }),
            }],
        };
        assert!(!has_any_reward_modulated(&def_without));
    }

    #[test]
    fn update_on_missing_traces_is_noop() {
        let def = make_reward_modulated_def(HebbianRule::Classic, 0.1, 0.9);
        let mut traces: Vec<Vec<Box<[f32]>>> = Vec::new();
        // Don't call ensure — traces are empty.
        let weights: Vec<Vec<Box<[f32]>>> = Vec::new();
        let outputs = vec![1.0, 0.5];

        // Should not panic.
        update_eligibility_traces(&def, 0, &mut traces, &weights, &outputs);
    }
}
