//! Readings over a sample of a run's final living population: the evolved
//! half of the mutational neighborhood (T11.F01), the neighborhood read
//! (T14.F12) and the mutation-effects cohort (T11.F26). Each samples the
//! id-sorted population by the ranks `neighborhood::sample` draws, evaluates
//! every sampled genome at its predeclared seed, and pools the results in
//! sample order by integer merge.
//!
//! Observation only: every evaluation runs on a copy of a genome.

use crate::config::MutationConfig;
use crate::contracts::CreatureId;
use crate::genome::analysis::mesh_reachable_nodes;
use crate::genome::{structural_companions, CreatureGenome, StructuralCompanions};
use crate::neighborhood::battery::Battery;
use crate::neighborhood::births::{per_birth_reading, BirthExposure, BirthResult};
use crate::neighborhood::classify::Tally;
use crate::neighborhood::mesh_execution::MeshExecutionReading;
use crate::neighborhood::mutation_effects::{self as effects, CohortParent};
use crate::neighborhood::sample::{evolved_sample_ranks, read_sample_ranks};
use crate::neighborhood::steering::{SteeringBattery, SteeringPooled, SteeringReading};
use crate::neighborhood::{
    evaluate_genome, operator_catalog, EvalContext, GenomeEvaluation, EVOLVED_SEED_MULTIPLIER,
    READ_GENOME_MULTIPLIER, READ_SEED_BASE,
};
use crate::simulation::Simulation;

/// The living population's ids in ascending order: the rank space every
/// final-population sample is drawn from.
#[must_use]
pub fn sorted_creature_ids(sim: &Simulation) -> Vec<CreatureId> {
    let mut creature_ids: Vec<_> = sim.creatures.keys().collect();
    creature_ids.sort();
    creature_ids
}

/// One genome's `mesh_execution` reading and its `steering-v1` reading from
/// a single observed battery pass: the T11.F14 executed set feeds the
/// steering `move_voted` flag, so the knockout pass runs once.
#[must_use]
pub fn mesh_execution_and_steering(
    battery: &Battery,
    steering_battery: &SteeringBattery,
    genome: &CreatureGenome,
    context: &EvalContext,
) -> (MeshExecutionReading, SteeringReading) {
    let sets =
        battery.mesh_execution_sets(genome, context.runtime, context.shared_memory_decay_rate);
    let steering = steering_battery.read(genome, context.runtime, &sets.executed);
    (sets.reading, steering)
}

/// The per-genome evaluation sizes of a sample.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EvaluationSizes {
    pub operator_trials: u32,
    pub births: u32,
}

/// One sampled genome of the evolved half.
#[derive(Debug, Clone)]
pub struct EvolvedGenome {
    pub rank: usize,
    pub creature_id: CreatureId,
    pub generation: u64,
    pub evaluation: GenomeEvaluation,
    pub companions: StructuralCompanions,
    pub mesh_execution: MeshExecutionReading,
    pub steering: SteeringReading,
}

/// The evolved half for one final population.
#[derive(Debug, Clone)]
pub struct EvolvedObservation {
    pub population_size: usize,
    /// Every living creature's generation, in population order.
    pub generations: Vec<u64>,
    pub genomes: Vec<EvolvedGenome>,
    /// One tally per `operator_catalog()` row, merged in sample order.
    pub pooled_operator_tallies: Vec<Tally>,
    pub pooled_births: BirthResult,
    pub pooled_steering: SteeringPooled,
}

/// The evolved half: the predeclared rank sample of the final population,
/// each sampled genome evaluated at seed offset
/// `EVOLVED_SEED_MULTIPLIER × (genome_index + 1)`, and the pooled
/// per-operator, per-birth and steering readings across the sample.
#[must_use]
pub fn evolved_observation(
    sim: &Simulation,
    battery: &Battery,
    mutation_config: &MutationConfig,
    context: &EvalContext,
    sizes: EvaluationSizes,
) -> EvolvedObservation {
    let creature_ids = sorted_creature_ids(sim);
    let population_size = creature_ids.len();
    let ranks = evolved_sample_ranks(population_size);
    let catalog_len = operator_catalog().len();

    let mut genomes = Vec::with_capacity(ranks.len());
    let mut pooled_operator_tallies: Vec<Tally> = vec![Tally::default(); catalog_len];
    let mut pooled_births = BirthResult::default();
    let steering_battery = SteeringBattery::generate(context.sensors.food_type_count);
    let mut pooled_steering = SteeringPooled::default();

    for (genome_index, &rank) in ranks.iter().enumerate() {
        let creature_id = creature_ids[rank];
        let creature = &sim.creatures[creature_id];
        let seed_offset = EVOLVED_SEED_MULTIPLIER * (genome_index as u64 + 1);
        let evaluation = evaluate_genome(
            creature.genome(),
            battery,
            mutation_config,
            context,
            sizes.operator_trials,
            sizes.births,
            seed_offset,
        );
        let companions = structural_companions(creature.genome());

        for (pooled, row) in pooled_operator_tallies
            .iter_mut()
            .zip(&evaluation.operator_rows)
        {
            *pooled = pooled.merge(row.tally);
        }
        pooled_births = pooled_births.merge(&evaluation.births);
        let (mesh_execution, steering) =
            mesh_execution_and_steering(battery, &steering_battery, creature.genome(), context);
        pooled_steering = pooled_steering.merge(steering);

        genomes.push(EvolvedGenome {
            rank,
            creature_id,
            generation: creature.generation,
            evaluation,
            companions,
            mesh_execution,
            steering,
        });
    }

    EvolvedObservation {
        population_size,
        generations: sim
            .creatures
            .values()
            .map(|creature| creature.generation)
            .collect(),
        genomes,
        pooled_operator_tallies,
        pooled_births,
        pooled_steering,
    }
}

/// One sampled genome of the neighborhood read.
#[derive(Debug, Clone)]
pub struct ReadGenome {
    pub rank: usize,
    pub creature_id: CreatureId,
    pub lineage_id: u32,
    pub generation: u64,
    pub genome_size: u32,
    pub total_nodes: u64,
    pub reachable_nodes: u64,
    pub executed_nodes: u64,
    pub births: BirthResult,
}

/// The neighborhood read for one final population.
#[derive(Debug, Clone, Default)]
pub struct ReadObservation {
    pub population_size: usize,
    pub genomes: Vec<ReadGenome>,
    /// The sampled genomes' production births, merged in sample order.
    pub pooled: BirthResult,
    /// The exposure strata of the same births, merged in sample order.
    pub exposure: BirthExposure,
    pub generation_sum: u64,
    pub genome_size_sum: u64,
    pub total_nodes: u64,
    pub reachable_nodes: u64,
    pub executed_nodes: u64,
}

/// The neighborhood read (T14.F12): a seeded uniform sample of `read_sample`
/// genomes of the id-sorted population, each genome's `births` production
/// births at seed offset `READ_SEED_BASE + READ_GENOME_MULTIPLIER ×
/// (genome_index + 1)`, pooled by integer merge in sample order.
#[must_use]
pub fn read_observation(
    seed: u64,
    sim: &Simulation,
    battery: &Battery,
    mutation_config: &MutationConfig,
    context: &EvalContext,
    read_sample: usize,
    births: u32,
) -> ReadObservation {
    let creature_ids = sorted_creature_ids(sim);
    let population_size = creature_ids.len();
    let ranks = read_sample_ranks(population_size, read_sample, seed);
    let mut read = ReadObservation {
        population_size,
        genomes: Vec::with_capacity(ranks.len()),
        ..ReadObservation::default()
    };

    for (genome_index, &rank) in ranks.iter().enumerate() {
        let creature_id = creature_ids[rank];
        let creature = &sim.creatures[creature_id];
        let genome = creature.genome();
        let seed_offset = READ_SEED_BASE + READ_GENOME_MULTIPLIER * (genome_index as u64 + 1);
        let base = battery.signature(genome, context.runtime, context.shared_memory_decay_rate);
        let (genome_births, genome_exposure) = per_birth_reading(
            genome,
            &base,
            battery,
            mutation_config,
            context,
            births,
            seed_offset,
        );
        read.exposure = read.exposure.merge(&genome_exposure);
        let executed = battery
            .executed_indices(genome, context.runtime, context.shared_memory_decay_rate)
            .len() as u64;
        let reachable = mesh_reachable_nodes(genome).len() as u64;
        let nodes = genome.nodes.len() as u64;
        let genome_size = genome.genome_size();

        read.generation_sum += creature.generation;
        read.genome_size_sum += u64::from(genome_size);
        read.total_nodes += nodes;
        read.reachable_nodes += reachable;
        read.executed_nodes += executed;
        read.pooled = read.pooled.merge(&genome_births);

        read.genomes.push(ReadGenome {
            rank,
            creature_id,
            lineage_id: creature.lineage_id,
            generation: creature.generation,
            genome_size,
            total_nodes: nodes,
            reachable_nodes: reachable,
            executed_nodes: executed,
            births: genome_births,
        });
    }
    read
}

/// The selected cohort (T11.F26): `min(parents, s)` of the neighborhood
/// read's sample positions, each carrying its position as its index.
#[must_use]
pub fn selected_cohort(
    seed: u64,
    sim: &Simulation,
    read_sample: u32,
    sizes: effects::Sizes,
) -> Vec<CohortParent> {
    let ids = sorted_creature_ids(sim);
    let ranks = read_sample_ranks(ids.len(), read_sample as usize, seed);
    effects::selected_positions(ranks.len(), sizes.parents as usize, seed)
        .into_iter()
        .map(|position| {
            let creature = &sim.creatures[ids[ranks[position]]];
            CohortParent {
                index: position as u64,
                depth_or_generation: creature.generation,
                genome: creature.genome().clone(),
            }
        })
        .collect()
}
