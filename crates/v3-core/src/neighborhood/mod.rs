//! Mutational-neighborhood indicator (T11.F01): how often a single mutation
//! leaves a brain acting like its parent (silent), acting differently
//! (changed), or not acting at all (dead), per operator at one event and per
//! birth through the production mutation engine, on the fixed `neighborhood-v1`
//! sensor battery.
//!
//! This module is a top-level sibling of `mutation`, `runtime`, and
//! `simulation` rather than nested inside any of them: it depends on all
//! three (a subject's cognition path from `runtime`, mutation operators from
//! `mutation`, and the shared-memory bookkeeping seam from `simulation`), and
//! none of those three may depend back on it.
//!
//! Observation only: nothing here changes a genome, mutation probability, or
//! runtime semantics. Every function is pure over its explicit inputs (a
//! genome, a battery, a config, a seed) so the whole module is unit- and
//! property-testable without a `Simulation`.

pub mod battery;
pub mod births;
pub mod classify;
pub mod companions;
pub mod drift;
pub mod mesh_execution;
pub mod operators;
pub mod recruitment;
pub mod recruitment_paths;
pub mod sample;

pub use battery::{Battery, Signature};
pub use births::BirthResult;
pub use classify::{classify, Class, Classification, Tally};
pub use companions::{structural_companions, StructuralCompanions};
pub use operators::{operator_catalog, OperatorRow};
pub use sample::{evolved_sample_ranks, SAMPLE_SIZE};

use crate::config::{RuntimeConfig, SimulationConfig};

/// Multiplier for an evolved genome's seed offset:
/// `EVOLVED_SEED_MULTIPLIER * (genome_index + 1)`, `genome_index` running
/// over the sampled genomes of one seed in rank order.
pub const EVOLVED_SEED_MULTIPLIER: u64 = 100_000;

/// Predeclared founder-half battery version and sizes (T11.F01 Battery,
/// "Predeclared sizes and seeds"). Halved once, uniformly, if a compute limit
/// is exceeded — never chosen for better readings.
pub const BATTERY_VERSION: &str = "neighborhood-v1";

/// The pieces of a [`SimulationConfig`] a battery evaluation needs beyond the
/// mutation config, bundled so call sites do not thread three separate values
/// through every function.
#[derive(Debug, Clone, Copy)]
pub struct EvalContext<'a> {
    pub runtime: &'a RuntimeConfig,
    pub shared_memory_decay_rate: f32,
    pub food_type_count: usize,
}

impl<'a> EvalContext<'a> {
    #[must_use]
    pub fn from_config(config: &'a SimulationConfig) -> Self {
        Self {
            runtime: &config.runtime,
            shared_memory_decay_rate: config.shared_memory.decay_rate,
            food_type_count: config.world.food.types.len(),
        }
    }
}

/// One genome's complete neighborhood reading: the per-operator rows (four
/// families' `ALL` lists, in order) and the per-birth bucketed result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenomeEvaluation {
    pub operator_rows: Vec<OperatorRow>,
    pub births: BirthResult,
}

/// Evaluate one genome's mutational neighborhood: its own base signature,
/// every operator applied once per trial, and every mutated birth — sharing
/// the same battery, mutation config, and evaluation context. `seed_offset`
/// is `0` for the founder half and
/// `EVOLVED_SEED_MULTIPLIER * (genome_index + 1)` for an evolved sample
/// genome, so founder and evolved readings never draw the same seeds.
#[must_use]
pub fn evaluate_genome(
    genome: &crate::creature::genome::CreatureGenome,
    battery: &Battery,
    mutation_config: &crate::config::MutationConfig,
    context: &EvalContext,
    operator_trials: u32,
    birth_count: u32,
    seed_offset: u64,
) -> GenomeEvaluation {
    let base = battery.signature(genome, context.runtime, context.shared_memory_decay_rate);
    let operator_rows = operators::per_operator_rows(
        genome,
        &base,
        battery,
        mutation_config,
        context,
        operator_trials,
        seed_offset,
    );
    let births = births::per_birth_result(
        genome,
        &base,
        battery,
        mutation_config,
        context,
        birth_count,
        seed_offset,
    );
    GenomeEvaluation {
        operator_rows,
        births,
    }
}
