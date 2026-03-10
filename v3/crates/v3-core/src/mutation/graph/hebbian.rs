//! Plasticity mutation operators for graph internal nodes.
//!
//! Nine operators that evolve per-node plasticity learning parameters:
//! - EnableHebbian: add PlasticityConfig to a non-plasticity node
//! - DisableHebbian: remove PlasticityConfig from a plasticity node
//! - MutateHebbianRule: change the learning rule variant
//! - MutateHebbianRate: perturb the learning rate
//! - ToggleHebbianLamarckian: flip the inheritance flag
//! - EnableRewardModulation: add reward modulation to a pure Hebbian node
//! - DisableRewardModulation: remove reward modulation from a modulated node
//! - MutateRewardSource: change the outcome channel a modulated node listens to
//! - MutateTraceDecay: perturb the trace decay rate on a modulated node

use rand::Rng;

use crate::creature::genome::{BackendDef, CreatureGenome};
use crate::mutation::types::MutationSkipReason;

// --- Old Hebbian operators: temporarily stubbed as no-ops for Graph backend ---
// CGP replacement lives in cgp_hebbian.rs. These stubs exist only to keep
// the dispatch in graph/mod.rs compiling until it is rewired to CGP operators.

/// Add PlasticityConfig to a random non-plasticity internal node.
pub fn enable_hebbian(
    genome: &mut CreatureGenome,
    node_idx: usize,
    _rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if let BackendDef::Graph(_) = genome.nodes[node_idx].backend_def {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    Ok(())
}

/// Remove PlasticityConfig from a random plasticity internal node.
pub fn disable_hebbian(
    genome: &mut CreatureGenome,
    node_idx: usize,
    _rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if let BackendDef::Graph(_) = genome.nodes[node_idx].backend_def {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    Ok(())
}

/// Change the HebbianRule variant on a random Hebbian internal node.
pub fn mutate_hebbian_rule(
    genome: &mut CreatureGenome,
    node_idx: usize,
    _rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if let BackendDef::Graph(_) = genome.nodes[node_idx].backend_def {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    Ok(())
}

/// Perturb the learning_rate on a random Hebbian internal node.
pub fn mutate_hebbian_rate(
    genome: &mut CreatureGenome,
    node_idx: usize,
    _rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if let BackendDef::Graph(_) = genome.nodes[node_idx].backend_def {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    Ok(())
}

/// Flip the `lamarckian` flag on a random Hebbian internal node.
pub fn toggle_hebbian_lamarckian(
    genome: &mut CreatureGenome,
    node_idx: usize,
    _rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if let BackendDef::Graph(_) = genome.nodes[node_idx].backend_def {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    Ok(())
}

/// Add reward modulation to a random plasticity node that has `modulation=None`.
pub fn enable_reward_modulation(
    genome: &mut CreatureGenome,
    node_idx: usize,
    _rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if let BackendDef::Graph(_) = genome.nodes[node_idx].backend_def {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    Ok(())
}

/// Remove reward modulation from a random reward-modulated node.
pub fn disable_reward_modulation(
    genome: &mut CreatureGenome,
    node_idx: usize,
    _rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if let BackendDef::Graph(_) = genome.nodes[node_idx].backend_def {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    Ok(())
}

/// Switch the `reward_source` channel on a random reward-modulated node.
pub fn mutate_reward_source(
    genome: &mut CreatureGenome,
    node_idx: usize,
    _rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if let BackendDef::Graph(_) = genome.nodes[node_idx].backend_def {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    Ok(())
}

/// Perturb the `trace_decay` on a random reward-modulated node.
pub fn mutate_trace_decay(
    genome: &mut CreatureGenome,
    node_idx: usize,
    _rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if let BackendDef::Graph(_) = genome.nodes[node_idx].backend_def {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    Ok(())
}

// Old graph hebbian tests deleted.
// CGP replacements live in cgp_hebbian.rs with their own test suite.
