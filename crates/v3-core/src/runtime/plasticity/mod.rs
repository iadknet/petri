//! Plasticity system — per-edge weight adaptation at runtime (T25.F06).
//!
//! Submodules:
//! - `rules`: the four Hebbian laws and the three-factor update
//! - `kind`: the plastic-edge kind descriptor
//! - `learned`: the learned state built at birth, the visit's learner and
//!   the read-only view

mod kind;
mod learned;
pub(crate) mod rules;

pub use kind::{PlasticityKind, RULES};
pub(crate) use learned::Learner;
pub use learned::{LearnedEdge, LearnedScale, LearnedState};

use crate::creature::genome::OUTCOME_CHANNEL_COUNT;

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
mod group_tests;
#[cfg(test)]
mod tests;
