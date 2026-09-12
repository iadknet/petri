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

use crate::creature::genome::cgp::CgpGraphBackendDef;
use crate::runtime::plasticity::OutcomeSignalBank;

/// Apply reward-modulated weight updates for a single mesh node.
///
/// For each reward-modulated internal node:
/// 1. Look up the outcome signal from the node's `reward_source` channel.
/// 2. For each edge, compute `dw = learning_rate * signal * trace`.
/// 3. Update the learned weight with clamping.
///
/// Returns energy cost, edge assignments, and assignments that change a weight.
pub(crate) fn apply_reward_modulated_updates(
    def: &CgpGraphBackendDef,
    node_idx: usize,
    plasticity_weights: &mut [Vec<Box<[f32]>>],
    eligibility_traces: &[Vec<Box<[f32]>>],
    signals: &OutcomeSignalBank,
    cost_per_update: f32,
) -> (f32, u32, u32) {
    let mut total_cost: f32 = 0.0;
    let mut update_count: u32 = 0;
    let mut changed_count: u32 = 0;

    for (i, cnode) in def.compute_nodes.iter().enumerate() {
        let cfg = match &cnode.plasticity {
            Some(c) => c,
            None => continue,
        };

        let modulation = match &cfg.modulation {
            Some(m) => m,
            None => continue,
        };

        if cnode.inputs.is_empty() {
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

        for edge_idx in 0..cnode.inputs.len() {
            if edge_idx >= weights.len() || edge_idx >= traces.len() {
                break;
            }

            let dw = eta * signal * traces[edge_idx];
            let old_weight = weights[edge_idx];
            weights[edge_idx] = (weights[edge_idx] + dw).clamp(-w_clamp, w_clamp);
            // Compare the final stored f32, including clamping and rounding.
            changed_count += u32::from(weights[edge_idx] != old_weight);
            total_cost += cost_per_update;
            update_count += 1;
        }
    }

    (total_cost, update_count, changed_count)
}

// Old reward-modulated plasticity tests removed — production code ported to CGP types.
