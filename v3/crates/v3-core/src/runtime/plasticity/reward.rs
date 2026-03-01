//! Reward-modulated learning pass (three-factor plasticity).
//!
//! After Phase 2 (action execution), the tick loop calls
//! [`apply_reward_modulated_updates`] for each creature that has
//! reward-modulated nodes. The three-factor update rule:
//!
//! ```text
//! dw = learning_rate * outcome_signal[channel] * trace[edge]
//! new_w = clamp(w + dw, -weight_clamp, weight_clamp)
//! ```
//!
//! This couples the eligibility trace (which records recent Hebbian activity)
//! with an outcome signal (which describes what happened to the creature) to
//! produce biologically-plausible credit assignment.

use crate::creature::genome::GraphBackendDef;
use crate::runtime::plasticity::OutcomeSignalBank;

/// Apply reward-modulated weight updates for a single mesh node.
///
/// For each reward-modulated internal node:
/// 1. Look up the outcome signal from the node's `reward_source` channel.
/// 2. For each edge, compute `dw = learning_rate * signal * trace`.
/// 3. Update the learned weight with clamping.
///
/// Returns total energy cost of the updates.
pub(crate) fn apply_reward_modulated_updates(
    def: &GraphBackendDef,
    node_idx: usize,
    plasticity_weights: &mut [Vec<Box<[f32]>>],
    eligibility_traces: &[Vec<Box<[f32]>>],
    signals: &OutcomeSignalBank,
    cost_per_update: f32,
) -> f32 {
    let mut total_cost: f32 = 0.0;

    for (i, inode) in def.internal_nodes.iter().enumerate() {
        let cfg = match &inode.plasticity {
            Some(c) => c,
            None => continue,
        };

        let modulation = match &cfg.modulation {
            Some(m) => m,
            None => continue,
        };

        if inode.inputs.is_empty() {
            continue;
        }

        let eta = cfg.learning_rate.clamp(0.0, 1.0);
        let w_clamp = cfg.weight_clamp.clamp(0.01, 10.0);
        let signal = signals.signals[modulation.reward_source as usize];

        // Bounds check for weights and traces.
        if plasticity_weights.len() <= node_idx || plasticity_weights[node_idx].len() <= i {
            continue;
        }
        if eligibility_traces.len() <= node_idx || eligibility_traces[node_idx].len() <= i {
            continue;
        }
        let weights = &mut plasticity_weights[node_idx][i];
        let traces = &eligibility_traces[node_idx][i];

        if weights.is_empty() || traces.is_empty() {
            continue;
        }

        for edge_idx in 0..inode.inputs.len() {
            if edge_idx >= weights.len() || edge_idx >= traces.len() {
                break;
            }

            let dw = eta * signal * traces[edge_idx];
            weights[edge_idx] = (weights[edge_idx] + dw).clamp(-w_clamp, w_clamp);
            total_cost += cost_per_update;
        }
    }

    total_cost
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::creature::genome::{
        GraphInput, GraphInternalNode, GraphNodeKind, HebbianRule, OutcomeChannel,
        PlasticityConfig, RewardModulationConfig, OUTCOME_CHANNEL_COUNT,
    };
    use crate::runtime::plasticity::OutcomeSignalBank;

    fn make_reward_def() -> GraphBackendDef {
        GraphBackendDef {
            internal_nodes: vec![
                // Node 0: input source
                GraphInternalNode {
                    kind: GraphNodeKind::Constant(1.0),
                    inputs: vec![],
                    plasticity: None,
                },
                // Node 1: reward-modulated, listens to EnergyDelta
                GraphInternalNode {
                    kind: GraphNodeKind::Add,
                    inputs: vec![GraphInput {
                        source_idx: 0,
                        weight: 0.5,
                    }],
                    plasticity: Some(PlasticityConfig {
                        rule: HebbianRule::Classic,
                        learning_rate: 0.1,
                        weight_clamp: 5.0,
                        lamarckian: false,
                        modulation: Some(RewardModulationConfig {
                            reward_source: OutcomeChannel::EnergyDelta,
                            trace_decay: 0.9,
                        }),
                    }),
                },
            ],
        }
    }

    fn make_signals(energy_delta: f32) -> OutcomeSignalBank {
        let mut signals = [0.0f32; OUTCOME_CHANNEL_COUNT];
        signals[OutcomeChannel::EnergyDelta as usize] = energy_delta;
        OutcomeSignalBank { signals }
    }

    #[test]
    fn positive_signal_reinforces_weight() {
        let def = make_reward_def();
        let mut weights: Vec<Vec<Box<[f32]>>> = vec![vec![
            Box::new([]) as Box<[f32]>,
            vec![0.5f32].into_boxed_slice(),
        ]];
        let traces: Vec<Vec<Box<[f32]>>> = vec![vec![
            Box::new([]) as Box<[f32]>,
            vec![0.3f32].into_boxed_slice(), // trace = 0.3
        ]];

        let signals = make_signals(10.0); // positive outcome
        let cost = apply_reward_modulated_updates(&def, 0, &mut weights, &traces, &signals, 0.0);
        assert!((cost - 0.0).abs() < f32::EPSILON);

        // dw = 0.1 * 10.0 * 0.3 = 0.3
        // new_w = 0.5 + 0.3 = 0.8
        assert!((weights[0][1][0] - 0.8).abs() < 1e-6);
    }

    #[test]
    fn negative_signal_weakens_weight() {
        let def = make_reward_def();
        let mut weights: Vec<Vec<Box<[f32]>>> = vec![vec![
            Box::new([]) as Box<[f32]>,
            vec![0.5f32].into_boxed_slice(),
        ]];
        let traces: Vec<Vec<Box<[f32]>>> = vec![vec![
            Box::new([]) as Box<[f32]>,
            vec![0.3f32].into_boxed_slice(),
        ]];

        let signals = make_signals(-5.0); // negative outcome
        apply_reward_modulated_updates(&def, 0, &mut weights, &traces, &signals, 0.0);

        // dw = 0.1 * (-5.0) * 0.3 = -0.15
        // new_w = 0.5 - 0.15 = 0.35
        assert!((weights[0][1][0] - 0.35).abs() < 1e-6);
    }

    #[test]
    fn zero_signal_no_change() {
        let def = make_reward_def();
        let mut weights: Vec<Vec<Box<[f32]>>> = vec![vec![
            Box::new([]) as Box<[f32]>,
            vec![0.5f32].into_boxed_slice(),
        ]];
        let traces: Vec<Vec<Box<[f32]>>> = vec![vec![
            Box::new([]) as Box<[f32]>,
            vec![0.3f32].into_boxed_slice(),
        ]];

        let signals = make_signals(0.0);
        apply_reward_modulated_updates(&def, 0, &mut weights, &traces, &signals, 0.0);

        assert!((weights[0][1][0] - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn weight_clamping_enforced() {
        let def = make_reward_def();
        let mut weights: Vec<Vec<Box<[f32]>>> = vec![vec![
            Box::new([]) as Box<[f32]>,
            vec![4.9f32].into_boxed_slice(), // near upper clamp
        ]];
        let traces: Vec<Vec<Box<[f32]>>> = vec![vec![
            Box::new([]) as Box<[f32]>,
            vec![1.0f32].into_boxed_slice(),
        ]];

        let signals = make_signals(100.0); // huge signal
        apply_reward_modulated_updates(&def, 0, &mut weights, &traces, &signals, 0.0);

        // dw = 0.1 * 100.0 * 1.0 = 10.0 → 4.9 + 10.0 = 14.9, clamped to 5.0
        assert!((weights[0][1][0] - 5.0).abs() < 1e-6);
    }

    #[test]
    fn energy_cost_accumulated() {
        let def = make_reward_def();
        let mut weights: Vec<Vec<Box<[f32]>>> = vec![vec![
            Box::new([]) as Box<[f32]>,
            vec![0.5f32].into_boxed_slice(),
        ]];
        let traces: Vec<Vec<Box<[f32]>>> = vec![vec![
            Box::new([]) as Box<[f32]>,
            vec![0.3f32].into_boxed_slice(),
        ]];

        let signals = make_signals(1.0);
        let cost = apply_reward_modulated_updates(&def, 0, &mut weights, &traces, &signals, 0.01);

        // 1 reward-modulated node with 1 edge = 1 update = 0.01
        assert!((cost - 0.01).abs() < 1e-9);
    }

    #[test]
    fn pure_hebbian_nodes_skipped() {
        let def = GraphBackendDef {
            internal_nodes: vec![GraphInternalNode {
                kind: GraphNodeKind::Add,
                inputs: vec![GraphInput {
                    source_idx: 0,
                    weight: 0.5,
                }],
                plasticity: Some(PlasticityConfig {
                    rule: HebbianRule::Classic,
                    learning_rate: 0.1,
                    weight_clamp: 5.0,
                    lamarckian: false,
                    modulation: None, // Pure Hebbian — no reward modulation.
                }),
            }],
        };
        let mut weights: Vec<Vec<Box<[f32]>>> = vec![vec![vec![0.5f32].into_boxed_slice()]];
        let traces: Vec<Vec<Box<[f32]>>> = vec![vec![vec![0.3f32].into_boxed_slice()]];

        let signals = make_signals(10.0);
        let cost = apply_reward_modulated_updates(&def, 0, &mut weights, &traces, &signals, 0.01);

        // Pure Hebbian node should be skipped — no cost, no weight change.
        assert!((cost - 0.0).abs() < f32::EPSILON);
        assert!((weights[0][0][0] - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn missing_weights_or_traces_is_noop() {
        let def = make_reward_def();
        let mut weights: Vec<Vec<Box<[f32]>>> = Vec::new();
        let traces: Vec<Vec<Box<[f32]>>> = Vec::new();
        let signals = make_signals(10.0);

        // Should not panic.
        let cost = apply_reward_modulated_updates(&def, 0, &mut weights, &traces, &signals, 0.0);
        assert!((cost - 0.0).abs() < f32::EPSILON);
    }
}
