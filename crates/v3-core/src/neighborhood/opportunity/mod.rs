//! Ecological opportunity assay (T20.F01, `input-opportunity-v1`): does
//! competent authored use of a ring, a heterogeneous vector and a scalar
//! input change actual offspring against a matched ablation, in the three
//! goal worlds at native costs?
//!
//! Head-to-head competition (Lenski et al. 1991): eight arms share one world
//! and fitness is read relative to a matched competitor within a replicate.
//! Mutation is off (`per_unit_rate = 0`), so arm identity is fixed and the
//! assay reads opportunity, not heritability.
//!
//! Authored controllers are instruments, never production genomes, mutation
//! targets or fitness signals.

pub mod assay;
pub mod controllers;
pub mod discovery;
pub mod fixtures;
#[cfg(test)]
mod tests;
pub mod verdict;

use crate::config::SimulationConfig;

pub use assay::{Arm, ArmGenomes, ReplicateRun};
pub use controllers::Family;
pub use verdict::{Exposure, Verdict, WorldVerdict};

pub const VERSION: &str = "input-opportunity-v1";
pub const REPLICATES: u32 = 8;
pub const HORIZON: u64 = 1_000;
/// World indices, in `goal-worlds-v1` recipe order.
pub const ORCHARDS: usize = 0;
pub const CANYON: usize = 1;

/// Run seed of replicate `replicate` in world `world_index`.
#[must_use]
pub fn run_seed(world_index: usize, replicate: u32) -> u64 {
    assay::RUN_SEED_BASE + 1_000 * world_index as u64 + u64::from(replicate)
}

/// Whether `family` has anything to sense in world `world_index`: the ring
/// in Orchards (no barriers) and the scalar in Canyon (one food type) are
/// predeclared null references.
#[must_use]
pub fn applicable(family: Family, world_index: usize) -> bool {
    !matches!(
        (family, world_index),
        (Family::Ring, ORCHARDS) | (Family::Scalar, CANYON)
    )
}

/// A world's replicate config: its production config with mutation off.
#[must_use]
pub fn replicate_config(world: &SimulationConfig) -> SimulationConfig {
    let mut config = world.clone();
    config.mutation.per_unit_rate = 0.0;
    config
}

/// Each family's verdict in one world from its replicates.
#[must_use]
pub fn world_verdicts(world_index: usize, replicates: &[ReplicateRun]) -> Vec<WorldVerdict> {
    Family::ALL
        .into_iter()
        .map(|family| {
            let (authored, inert) = (Arm::Authored(family).index(), Arm::Inert(family).index());
            let exposure = replicates.iter().fold(Exposure::default(), |sum, run| {
                sum.merge(run.arms[authored].exposure)
            });
            let pairs: Vec<(u64, u64)> = replicates
                .iter()
                .map(|run| (run.arms[authored].births, run.arms[inert].births))
                .collect();
            verdict::world_verdict(applicable(family, world_index), exposure, &pairs)
        })
        .collect()
}
