//! Plasticity system — weight adaptation for graph nodes at runtime.
//!
//! Submodules:
//! - `hebbian`: pure Hebbian learning (unsupervised, 4 rules)
//! - `traces`: eligibility trace management (reward-modulated nodes)
//! - `reward`: reward-modulated learning pass (three-factor plasticity)

pub(crate) mod hebbian;
pub(crate) mod reward;
pub(crate) mod traces;

use crate::creature::genome::{GraphBackendDef, OUTCOME_CHANNEL_COUNT};

// Re-export genome-layer types for Stage 2+ consumers (reward.rs, traces.rs).
#[allow(unused_imports)] // Wired in Stage 2
pub(crate) use crate::creature::genome::OutcomeChannel;

/// Returns `true` if any internal node has plasticity config enabled.
///
/// Used by Stage 2+ code paths; `hebbian::has_any_hebbian()` is the
/// current production entry point (identical logic).
#[inline]
#[allow(dead_code)] // Wired in Stage 2
pub(crate) fn has_any_plasticity(def: &GraphBackendDef) -> bool {
    def.internal_nodes.iter().any(|n| n.plasticity.is_some())
}

/// Outcome signals computed per-creature per-tick for reward-modulated plasticity.
///
/// Each channel holds a signed f32: positive = beneficial, negative = harmful.
/// Computed on-the-fly by `simulation::outcomes::compute_signal_bank()` — not
/// stored in `CreatureState`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[allow(dead_code)] // Wired in Stage 2
pub(crate) struct OutcomeSignalBank {
    pub signals: [f32; OUTCOME_CHANNEL_COUNT],
}
