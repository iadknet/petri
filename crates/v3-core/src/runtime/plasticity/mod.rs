//! Plasticity system — weight adaptation for graph nodes at runtime.
//!
//! Submodules:
//! - `hebbian`: pure Hebbian learning (unsupervised, 4 rules)
//! - `traces`: eligibility trace management (reward-modulated nodes)
//! - `reward`: reward-modulated learning pass (three-factor plasticity)

pub(crate) mod hebbian;
pub(crate) mod reward;
pub(crate) mod traces;

use crate::creature::genome::OUTCOME_CHANNEL_COUNT;

// Re-export genome-layer types used by reward.rs and traces.rs tests.
#[allow(unused_imports)]
pub(crate) use crate::creature::genome::OutcomeChannel;

/// Outcome signals computed per-creature per-tick for reward-modulated plasticity.
///
/// Each channel holds a signed f32: positive = beneficial, negative = harmful.
/// Computed on-the-fly by `simulation::outcomes::compute_signal_bank()` — not
/// stored in `CreatureState`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(crate) struct OutcomeSignalBank {
    pub signals: [f32; OUTCOME_CHANNEL_COUNT],
}

#[cfg(test)]
mod tests;
