//! Plasticity system — weight adaptation for graph nodes at runtime.
//!
//! Submodules:
//! - `hebbian`: pure Hebbian learning (unsupervised, 4 rules)
//! - `traces`: eligibility trace management (reward-modulated nodes)
//! - `reward`: reward-modulated learning pass (three-factor plasticity)

pub(crate) mod hebbian;
pub(crate) mod reward;
pub(crate) mod traces;

use crate::creature::genome::GraphBackendDef;

/// Returns `true` if any internal node has plasticity config enabled.
#[inline]
pub(crate) fn has_any_plasticity(def: &GraphBackendDef) -> bool {
    def.internal_nodes.iter().any(|n| n.plasticity.is_some())
}
