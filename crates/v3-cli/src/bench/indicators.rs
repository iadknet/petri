//! Indicator computations over a finished simulation.

use super::profiles::{NeighborhoodSizes, ProfileParams, GOAL_WORLD_SET};
use super::run::millis;
use super::schema::{
    cohort_ladder, undefined_drift_depth, undefined_evolved_neighborhood,
    undefined_generation_distribution, undefined_lineage_diversity, undefined_memory_sensitivity,
    undefined_mutational_neighborhood, undefined_recruitment_paths,
    undefined_temporal_memory_sensitivity, CohortLineageRow, DriftDepth, DriftDepthCheckpoint,
    EvolvedNeighborhoodHalf, FounderRow, GenerationDistribution, GoalCaseObservation,
    GoalIndicators, Indicator, LineageDiversity, LineageDiversitySeed, LineageOpportunityRow,
    MemorySensitivity, MemorySensitivitySeed, MeshExecution, ModuleRecruitment,
    MutationOpportunities, MutationalNeighborhood, NeighborhoodBattery, NeighborhoodBirthBucket,
    NeighborhoodBirths, NeighborhoodCompanions, NeighborhoodEvolvedSeed, NeighborhoodFounderHalf,
    NeighborhoodOperatorRow, NeighborhoodRead, NeighborhoodReadGenome,
    NeighborhoodRequestedBirthBucket, NeighborhoodSampledGenome, NeighborhoodTally,
    OperatorOpportunityRow, PopulationPersistence, PopulationPersistenceSeed, RetentionRow,
    Steering, SteeringChance, SteeringPooled, StructuralCompanionsCensus, StructuralCompanionsSeed,
    StructureSizeDistribution, TemporalMemorySensitivity, TemporalMemorySensitivitySeed,
    TimeToFirstRow, Totals, LINEAGE_DIVERSITY_VERSION, MEMORY_SENSITIVITY_VERSION,
    NEIGHBORHOOD_READ_VERSION, REACHABLE_STRUCTURE_VERSION,
};
use crate::{fraction_or_undefined, six, UNDEFINED};
use std::collections::BTreeMap;
use std::time::Instant;
use v3_core::config::{MutationConfig, SimulationConfig};
use v3_core::creature::founder::founder_genome;
use v3_core::creature::genome::analysis::mesh_reachable_nodes;
use v3_core::neighborhood::steering::{self, SteeringBattery};
use v3_core::neighborhood::{
    self, evaluate_genome, evolved_sample_ranks, read_sample_ranks, structural_companions, Battery,
    BirthResult, EvalContext, GenomeEvaluation, OperatorRow, StructuralCompanions, Tally,
    BATTERY_VERSION, READ_GENOME_MULTIPLIER, READ_SEED_BASE,
};
use v3_core::simulation::FinalActionObservation;

fn domain_keys(counts: &BTreeMap<v3_core::mutation::MutationDomain, u64>) -> BTreeMap<String, u64> {
    counts
        .iter()
        .map(|(domain, &count)| (domain.as_key().to_string(), count))
        .collect()
}

fn operator_keys(
    counts: &BTreeMap<v3_core::mutation::MutationOperator, u64>,
) -> BTreeMap<String, u64> {
    counts
        .iter()
        .map(|(operator, &count)| (operator.as_key().to_string(), count))
        .collect()
}

fn module_recruitment(
    reading: &neighborhood::recruitment::RecruitmentCheckpoint,
) -> ModuleRecruitment {
    use neighborhood::recruitment::CohortFact;
    let created = reading.cohort.created;
    ModuleRecruitment {
        cohort: cohort_ladder(reading.cohort),
        graph: cohort_ladder(reading.graph),
        vm: cohort_ladder(reading.vm),
        founders: FounderRow {
            created: reading.founders.created,
            deleted: reading.founders.deleted,
            present: reading.founders.present,
            dispatched: reading.founders.dispatched,
            contributing: reading.founders.contributing,
            contributing_fraction: fraction_or_undefined(
                reading.founders.contributing,
                reading.founders.present,
            ),
        },
        time_to_first: CohortFact::ALL
            .into_iter()
            .map(|fact| {
                let time = reading.time_to_first(fact);
                TimeToFirstRow {
                    fact: fact.as_key().to_string(),
                    reached: time.reached,
                    reached_fraction: fraction_or_undefined(time.reached, created),
                    median_generations: time.median_generations,
                    censored_deleted: time.censored_deleted,
                    censored_present: time.censored_present,
                }
            })
            .collect(),
        retention: reading.retention.map(|retention| RetentionRow {
            from_depth: retention.from_depth,
            contributing_before: retention.contributing_before,
            still_contributing: retention.still_contributing,
            present_not_contributing: retention.present_not_contributing,
            deleted: retention.deleted,
            retained_fraction: fraction_or_undefined(
                retention.still_contributing,
                retention.contributing_before,
            ),
        }),
        lineages: reading
            .lineage_rows
            .iter()
            .map(|row| CohortLineageRow {
                lineage: row.lineage,
                created: row.created,
                present: row.present,
                dispatched: row.dispatched,
                contributing: row.contributing,
            })
            .collect(),
    }
}

fn mutation_opportunities(
    reading: &neighborhood::recruitment::RecruitmentCheckpoint,
) -> MutationOpportunities {
    let pooled = &reading.opportunities;
    MutationOpportunities {
        births: pooled.births,
        zero_event_births: pooled.zero_event_births,
        zero_event_fraction: fraction_or_undefined(pooled.zero_event_births, pooled.births),
        attempted: pooled.attempted,
        applied: pooled.applied,
        skipped: pooled.skipped,
        applied_fraction: fraction_or_undefined(pooled.applied, pooled.attempted),
        reachable_target_events: pooled.reachable_target_events,
        unreachable_target_events: pooled.unreachable_target_events,
        executed_target_events: pooled.executed_target_events,
        executed_target_fraction: fraction_or_undefined(
            pooled.executed_target_events,
            pooled.applied,
        ),
        attempted_by_domain: domain_keys(&pooled.attempted_by_domain),
        applied_by_domain: domain_keys(&pooled.applied_by_domain),
        selected_inapplicable_by_domain: domain_keys(&pooled.selected_inapplicable_by_domain),
        no_eligible_node_by_domain: domain_keys(&pooled.no_eligible_node_by_domain),
        discarded_selected_inapplicable_by_operator: operator_keys(
            &pooled.discarded_selected_inapplicable_by_operator,
        ),
        discarded_no_eligible_node_by_operator: operator_keys(
            &pooled.discarded_no_eligible_node_by_operator,
        ),
        operators: pooled
            .attempted_by_operator
            .iter()
            .map(|(operator, &attempted)| {
                let applied = pooled
                    .applied_by_operator
                    .get(operator)
                    .copied()
                    .unwrap_or_default();
                OperatorOpportunityRow {
                    operator: operator.as_key().to_string(),
                    attempted,
                    applied,
                    applied_fraction: fraction_or_undefined(applied, attempted),
                    skipped_by_reason: pooled
                        .skipped_by_operator_reason
                        .get(operator)
                        .into_iter()
                        .flatten()
                        .map(|(reason, &count)| (reason.as_key().to_string(), count))
                        .collect(),
                }
            })
            .collect(),
        lineages: reading
            .lineage_opportunities
            .iter()
            .enumerate()
            .map(|(lineage, row)| LineageOpportunityRow {
                lineage: lineage as u32,
                births: row.births,
                attempted: row.attempted,
                applied: row.applied,
                skipped: row.skipped,
                discarded_selected_inapplicable: row
                    .discarded_selected_inapplicable_by_operator
                    .values()
                    .sum(),
                applied_by_domain: domain_keys(&row.applied_by_domain),
            })
            .collect(),
    }
}

fn drift_checkpoint(
    row: neighborhood::drift::Checkpoint,
    recruitment: &neighborhood::recruitment::RecruitmentCheckpoint,
) -> DriftDepthCheckpoint {
    let mesh = row.mesh;
    let denominator = u64::from(mesh.lineages);
    let battery_executions = denominator * u64::from(neighborhood_battery_execution_count());
    DriftDepthCheckpoint {
        backends: Some(mesh.backends),
        recruitment: Some(module_recruitment(recruitment)),
        opportunities: Some(mutation_opportunities(recruitment)),
        depth: row.depth,
        lineages: mesh.lineages,
        total_nodes: mesh.total_nodes,
        reachable_nodes: mesh.reachable_nodes,
        executed_nodes: mesh.executed_nodes,
        knockout_nodes: mesh.knockout_nodes,
        mean_total_nodes: fraction_or_undefined(mesh.total_nodes, denominator),
        mean_reachable_nodes: fraction_or_undefined(mesh.reachable_nodes, denominator),
        mean_executed_nodes: fraction_or_undefined(mesh.executed_nodes, denominator),
        mean_knockout_nodes: fraction_or_undefined(mesh.knockout_nodes, denominator),
        route_varying_lineages: mesh.route_varying_lineages,
        route_varying_fraction: fraction_or_undefined(
            u64::from(mesh.route_varying_lineages),
            denominator,
        ),
        hop_cap_hits: mesh.hop_cap_hits,
        battery_executions,
        hop_cap_fraction: fraction_or_undefined(mesh.hop_cap_hits, battery_executions),
        silent_per_all_births: fraction_or_undefined(
            u64::from(row.births.any_events.silent),
            u64::from(row.births.births_total),
        ),
        changed_per_all_births: fraction_or_undefined(
            u64::from(row.births.any_events.changed),
            u64::from(row.births.births_total),
        ),
        dead_per_all_births: fraction_or_undefined(
            u64::from(row.births.any_events.dead),
            u64::from(row.births.births_total),
        ),
        births: to_neighborhood_births(&row.births),
    }
}

pub(super) fn timed_drift_depth(
    params: &ProfileParams,
    config: &SimulationConfig,
    battery: Option<&Battery>,
) -> (Indicator<DriftDepth>, Option<f64>) {
    if params.name != "goal" && params.name != GOAL_WORLD_SET {
        return (undefined_drift_depth(), None);
    }
    let started = Instant::now();
    let reading = compute_drift_depth(config, battery.expect("goal battery"), params.drift);
    (Indicator::Defined(reading), Some(millis(started.elapsed())))
}

fn compute_drift_depth(
    config: &SimulationConfig,
    battery: &Battery,
    sizes: neighborhood::drift::DriftSizes,
) -> DriftDepth {
    use neighborhood::{battery as fixed_battery, drift, mesh_execution, recruitment};
    let founder = founder_genome(v3_core::config::FounderProfile::V3Alpha1);
    let readings = drift::observe(
        &founder,
        battery,
        &config.mutation,
        &EvalContext::from_config(config),
        sizes,
    );
    DriftDepth {
        version: drift::VERSION.to_string(),
        founder: "V3Alpha1".to_string(),
        lineages: sizes.lineages,
        birth_lineages: sizes.birth_lineages,
        birth_subset: "first lineage indices in ascending order".to_string(),
        birth_trials: sizes.births,
        checkpoints: sizes.checkpoints.to_vec(),
        walk_seed_formula: format!("{} + lineage_index", drift::WALK_SEED_BASE),
        birth_seed_formula: format!(
            "{} + {} * (lineage_index + 1) + checkpoint + {} + trial_index",
            drift::BIRTH_OFFSET_BASE,
            drift::BIRTH_LINEAGE_MULTIPLIER,
            neighborhood::births::BIRTH_SEED_BASE
        ),
        battery_version: BATTERY_VERSION.to_string(),
        mesh_version: mesh_execution::MESH_EXECUTION_VERSION.to_string(),
        knockout_method: mesh_execution::KNOCKOUT_METHOD.to_string(),
        executed_source: drift::EXECUTED_SOURCE.to_string(),
        executed_refresh: drift::EXECUTED_REFRESH.to_string(),
        recruitment_version: recruitment::RECRUITMENT_VERSION.to_string(),
        module_identity: recruitment::MODULE_IDENTITY.to_string(),
        provenance_rule: recruitment::PROVENANCE_RULE.to_string(),
        supply_rule: drift::supply_rule(&config.mutation),
        executions_per_genome: neighborhood_battery_execution_count(),
        snapshot_count: fixed_battery::SNAPSHOT_COUNT as u32,
        sequence_count: fixed_battery::SEQUENCE_COUNT as u32,
        sequence_len: fixed_battery::SEQUENCE_LEN as u32,
        readings: readings
            .checkpoints
            .into_iter()
            .zip(&readings.recruitment)
            .map(|(row, recruitment)| drift_checkpoint(row, recruitment))
            .collect(),
    }
}

pub(super) fn generation_distribution(
    mut generations: Vec<u64>,
) -> Indicator<GenerationDistribution> {
    if generations.is_empty() {
        return undefined_generation_distribution();
    }
    generations.sort_unstable();
    Indicator::Defined(GenerationDistribution {
        median: generations[generations.len() / 2],
        max: generations[generations.len() - 1],
    })
}

/// One genome's `mesh_execution` block and its `steering-v1` reading from a
/// single observed battery pass: the T11.F14 executed set feeds the
/// steering `bank_written` flag, so the knockout pass runs once.
pub(super) fn mesh_execution_and_steering(
    battery: &Battery,
    steering_battery: &SteeringBattery,
    genome: &v3_core::creature::genome::CreatureGenome,
    context: &EvalContext,
) -> (Indicator<MeshExecution>, Indicator<Steering>) {
    use v3_core::neighborhood::mesh_execution::{KNOCKOUT_METHOD, MESH_EXECUTION_VERSION};
    let sets =
        battery.mesh_execution_sets(genome, context.runtime, context.shared_memory_decay_rate);
    let reading = sets.reading;
    let mesh = MeshExecution {
        backends: Some(reading.backends),
        version: MESH_EXECUTION_VERSION.to_string(),
        executions_per_genome: neighborhood_battery_execution_count(),
        snapshot_route_probes: neighborhood::battery::SNAPSHOT_COUNT as u32,
        knockout_method: KNOCKOUT_METHOD.to_string(),
        total_node_count: reading.total_node_count as u64,
        reachable_node_count: reading.reachable_node_count as u64,
        executed_node_count: reading.executed_node_count as u64,
        knockout_count: reading.knockout_count as u64,
        route_varies_with_input: reading.route_varies_with_input,
        hop_cap_hits: reading.hop_cap_hits as u64,
    };
    let steering = Steering {
        version: steering::STEERING_VERSION.to_string(),
        seed: steering::STEERING_SEED,
        base_count: steering::STEERING_BASE_COUNT as u32,
        reading: steering_battery.read(genome, context.runtime, &sets.executed),
    };
    (Indicator::Defined(mesh), Indicator::Defined(steering))
}

/// The pooled `steering-v1` block for one seed's sample.
pub(super) fn steering_pooled(pooled: steering::SteeringPooled) -> Indicator<SteeringPooled> {
    Indicator::Defined(SteeringPooled {
        version: steering::STEERING_VERSION.to_string(),
        genomes: pooled.genomes,
        scenarios: pooled.scenarios,
        moves: pooled.moves,
        exact_hits: pooled.exact_hits,
        within_45: pooled.within_45,
        avoidance_trials: pooled.avoidance_trials,
        avoided: pooled.avoided,
        bank_written: pooled.bank_written,
        exact_hit_fraction: fraction_or_undefined(pooled.exact_hits, pooled.moves),
        within_45_fraction: fraction_or_undefined(pooled.within_45, pooled.moves),
        avoidance_fraction: fraction_or_undefined(pooled.avoided, pooled.avoidance_trials),
        bank_written_fraction: fraction_or_undefined(pooled.bank_written, pooled.genomes),
        chance: SteeringChance {
            exact: steering::CHANCE_EXACT,
            within_45: steering::CHANCE_WITHIN_45,
        },
    })
}

/// Surviving founder clade count and the Shannon entropy of their size
/// distribution, in nats. Counting is keyed in a `BTreeMap`, so neither the
/// count nor the fixed summation order of the entropy depends on how the
/// caller's population was iterated.
pub(super) fn clade_diversity(lineage_ids: impl IntoIterator<Item = u32>) -> (u64, String) {
    let mut counts = BTreeMap::<u32, u64>::new();
    for lineage_id in lineage_ids {
        *counts.entry(lineage_id).or_default() += 1;
    }
    let total: u64 = counts.values().sum();
    let shannon_entropy_nats = if total == 0 {
        UNDEFINED.to_string()
    } else {
        let total = total as f64;
        let entropy = counts.values().fold(0.0, |acc, &count| {
            let probability = count as f64 / total;
            acc - probability * probability.ln()
        });
        six(entropy)
    };
    (counts.len() as u64, shannon_entropy_nats)
}

pub(super) fn lineage_diversity(
    seed: u64,
    lineage_ids: impl IntoIterator<Item = u32>,
) -> LineageDiversitySeed {
    let (surviving_founder_clade_count, shannon_entropy_nats) = clade_diversity(lineage_ids);
    LineageDiversitySeed {
        seed,
        surviving_founder_clade_count,
        shannon_entropy_nats,
    }
}

pub(super) fn structural_companions_census<'a>(
    seed: u64,
    genomes: impl Iterator<Item = &'a v3_core::creature::genome::CreatureGenome>,
) -> StructuralCompanionsSeed {
    let mut census = StructuralCompanionsSeed {
        seed,
        ..Default::default()
    };
    for genome in genomes {
        let companions = structural_companions(genome);
        census.final_creature_count += 1;
        census.reads_shared_memory += u64::from(companions.reads_shared_memory);
        census.writes_shared_memory += u64::from(companions.writes_shared_memory);
        census.has_stateful_compute_node += u64::from(companions.has_stateful_compute_node);
        census.has_plasticity += u64::from(companions.has_plasticity);
    }
    census
}

pub(super) fn memory_sensitivity(
    seed: u64,
    observations: &[FinalActionObservation],
) -> MemorySensitivitySeed {
    let final_creature_count = observations.len() as u64;
    let (different_from_zeroed_count, different_from_scrambled_count, different_from_either_count) =
        observations.iter().fold((0, 0, 0), |counts, observation| {
            let different_from_zeroed = observation.intact != observation.zeroed;
            let different_from_scrambled = observation.intact != observation.scrambled;
            (
                counts.0 + u64::from(different_from_zeroed),
                counts.1 + u64::from(different_from_scrambled),
                counts.2 + u64::from(different_from_zeroed || different_from_scrambled),
            )
        });
    MemorySensitivitySeed {
        seed,
        final_creature_count,
        different_from_zeroed_count,
        different_from_scrambled_count,
        different_from_either_count,
        different_from_zeroed_fraction: fraction_or_undefined(
            different_from_zeroed_count,
            final_creature_count,
        ),
        different_from_scrambled_fraction: fraction_or_undefined(
            different_from_scrambled_count,
            final_creature_count,
        ),
        different_from_either_fraction: fraction_or_undefined(
            different_from_either_count,
            final_creature_count,
        ),
    }
}

pub(super) fn temporal_memory_sensitivity(
    seed: u64,
    sim: &v3_core::simulation::Simulation,
) -> TemporalMemorySensitivitySeed {
    use v3_core::simulation::{observe_temporal_actions, TemporalMemorySubstrate};
    let observations = observe_temporal_actions(sim);
    let component = |substrate| {
        let actions: Vec<_> = observations
            .iter()
            .filter(|observation| observation.substrate == substrate)
            .map(|observation| observation.actions.clone())
            .collect();
        memory_sensitivity(seed, &actions)
    };
    TemporalMemorySensitivitySeed {
        seed,
        previous_slots: component(TemporalMemorySubstrate::PreviousSlots),
        persisted_outputs: component(TemporalMemorySubstrate::PersistedOutputs),
        operator_state: component(TemporalMemorySubstrate::OperatorState),
    }
}

// ── Mutational neighborhood (T11.F01) ───────────────────────────────────────

/// The battery's total executions per genome: every single-tick snapshot
/// plus every sequence's ticks.
fn neighborhood_battery_execution_count() -> u32 {
    (neighborhood::battery::SNAPSHOT_COUNT
        + neighborhood::battery::SEQUENCE_COUNT * neighborhood::battery::SEQUENCE_LEN) as u32
}

fn to_neighborhood_tally(tally: &Tally) -> NeighborhoodTally {
    let applied = tally.applied();
    NeighborhoodTally {
        trials: tally.trials,
        skipped: tally.skipped,
        applied,
        silent: tally.silent,
        changed: tally.changed,
        dead: tally.dead,
        changed_only_in_sequences: tally.changed_only_in_sequences,
        silent_fraction: fraction_or_undefined(tally.silent.into(), applied.into()),
        changed_fraction: fraction_or_undefined(tally.changed.into(), applied.into()),
        dead_fraction: fraction_or_undefined(tally.dead.into(), applied.into()),
        mean_fraction_differing: six(tally.mean_fraction_differing()),
    }
}

fn to_neighborhood_operator_rows(rows: &[OperatorRow]) -> Vec<NeighborhoodOperatorRow> {
    rows.iter()
        .map(|row| NeighborhoodOperatorRow {
            family: row.family.to_string(),
            operator: row.operator.clone(),
            tally: to_neighborhood_tally(&row.tally),
        })
        .collect()
}

fn to_neighborhood_births(result: &BirthResult) -> NeighborhoodBirths {
    NeighborhoodBirths {
        births_total: result.births_total,
        by_requested_events: result
            .by_requested_events
            .iter()
            .map(
                |(&requested_events, &births)| NeighborhoodRequestedBirthBucket {
                    requested_events,
                    births,
                },
            )
            .collect(),
        zero_event_births: result.zero_event_births,
        any_events: to_neighborhood_tally(&result.any_events),
        by_events: result
            .by_events
            .iter()
            .map(|(&applied_events, tally)| NeighborhoodBirthBucket {
                applied_events,
                tally: to_neighborhood_tally(tally),
            })
            .collect(),
    }
}

fn to_neighborhood_companions(companions: &StructuralCompanions) -> NeighborhoodCompanions {
    NeighborhoodCompanions {
        functional_complexity: companions.functional_complexity,
        reachable_node_count: companions.reachable_node_count as u64,
        reads_shared_memory: companions.reads_shared_memory,
        writes_shared_memory: companions.writes_shared_memory,
        has_stateful_compute_node: companions.has_stateful_compute_node,
        has_plasticity: companions.has_plasticity,
    }
}

/// The founder half: always computed when `mutational_neighborhood` is
/// defined (gate and goal), once per report — outside the per-seed loop,
/// since it depends only on the founder genome and the production mutation
/// config, never on a world trajectory.
pub(super) fn compute_founder_neighborhood(
    config: &SimulationConfig,
    battery: &Battery,
    sizes: NeighborhoodSizes,
) -> NeighborhoodFounderHalf {
    let subject = founder_genome(config.population.founder_profile);
    let context = EvalContext::from_config(config);
    let evaluation: GenomeEvaluation = evaluate_genome(
        &subject,
        battery,
        &config.mutation,
        &context,
        sizes.founder_operator_trials,
        sizes.founder_births,
        0,
    );
    let steering_battery = SteeringBattery::generate(context.food_type_count);
    let (mesh_execution, steering) =
        mesh_execution_and_steering(battery, &steering_battery, &subject, &context);
    NeighborhoodFounderHalf {
        generation: Some(0),
        mesh_execution,
        steering,
        reachable_node_count: structural_companions(&subject).reachable_node_count as u64,
        operator_rows: to_neighborhood_operator_rows(&evaluation.operator_rows),
        births: to_neighborhood_births(&evaluation.births),
    }
}

/// The living population's ids in ascending order: the rank space every
/// final-population sample is drawn from.
fn sorted_creature_ids(
    sim: &v3_core::simulation::Simulation,
) -> Vec<v3_core::contracts::CreatureId> {
    let mut creature_ids: Vec<_> = sim.creatures.keys().collect();
    creature_ids.sort();
    creature_ids
}

/// The evolved half for one seed's final living population: the predeclared
/// rank sample, each sampled genome's full reading and structural
/// companions, and the pooled per-operator and per-birth tallies across the
/// sample.
pub(super) fn evolved_neighborhood_for_seed(
    seed: u64,
    sim: &v3_core::simulation::Simulation,
    battery: &Battery,
    mutation_config: &MutationConfig,
    context: &EvalContext,
    sizes: NeighborhoodSizes,
) -> NeighborhoodEvolvedSeed {
    let creature_ids = sorted_creature_ids(sim);
    let population_size = creature_ids.len();
    let ranks = evolved_sample_ranks(population_size);
    let catalog = v3_core::neighborhood::operator_catalog();

    let mut sampled_genomes = Vec::with_capacity(ranks.len());
    let mut pooled_operator_tallies: Vec<Tally> = vec![Tally::default(); catalog.len()];
    let mut pooled_births = BirthResult::default();
    let steering_battery = SteeringBattery::generate(context.food_type_count);
    let mut pooled_steering = steering::SteeringPooled::default();

    for (genome_index, &rank) in ranks.iter().enumerate() {
        let creature_id = creature_ids[rank];
        let creature = &sim.creatures[creature_id];
        let seed_offset =
            v3_core::neighborhood::EVOLVED_SEED_MULTIPLIER * (genome_index as u64 + 1);
        let evaluation = evaluate_genome(
            &creature.genome,
            battery,
            mutation_config,
            context,
            sizes.evolved_operator_trials,
            sizes.evolved_births,
            seed_offset,
        );
        let companions = structural_companions(&creature.genome);

        for (pooled, row) in pooled_operator_tallies
            .iter_mut()
            .zip(&evaluation.operator_rows)
        {
            *pooled = pooled.merge(row.tally);
        }
        pooled_births = pooled_births.merge(&evaluation.births);
        let (mesh_execution, steering) =
            mesh_execution_and_steering(battery, &steering_battery, &creature.genome, context);
        if let Indicator::Defined(steering) = &steering {
            pooled_steering = pooled_steering.merge(steering.reading);
        }

        sampled_genomes.push(NeighborhoodSampledGenome {
            generation: Some(creature.generation),
            mesh_execution,
            steering,
            rank: rank as u64,
            creature_id: format!("{creature_id:?}"),
            operator_rows: to_neighborhood_operator_rows(&evaluation.operator_rows),
            births: to_neighborhood_births(&evaluation.births),
            companions: to_neighborhood_companions(&companions),
        });
    }

    let pooled_operator_rows = catalog
        .iter()
        .zip(pooled_operator_tallies.iter())
        .map(|((family, name), tally)| NeighborhoodOperatorRow {
            family: (*family).to_string(),
            operator: name.clone(),
            tally: to_neighborhood_tally(tally),
        })
        .collect();

    NeighborhoodEvolvedSeed {
        generation_distribution: generation_distribution(
            sim.creatures
                .values()
                .map(|creature| creature.generation)
                .collect(),
        ),
        seed,
        final_population_size: population_size as u64,
        sampled_genomes,
        pooled_operator_rows,
        pooled_births: to_neighborhood_births(&pooled_births),
        steering_pooled: steering_pooled(pooled_steering),
    }
}

/// The neighborhood read (T14.F12) for one world's final living population:
/// a seeded uniform sample of the id-sorted population, each genome's
/// production births through the unchanged `per_birth_result`, pooled by
/// integer merge in sample order. Reads the terminal population only; the
/// evolved half's reading is untouched.
pub(super) fn neighborhood_read_for_seed(
    seed: u64,
    sim: &v3_core::simulation::Simulation,
    battery: &Battery,
    mutation_config: &MutationConfig,
    context: &EvalContext,
    sizes: NeighborhoodSizes,
) -> NeighborhoodRead {
    let creature_ids = sorted_creature_ids(sim);
    let population_size = creature_ids.len();
    let ranks = read_sample_ranks(population_size, sizes.read_sample as usize, seed);

    let mut genomes = Vec::with_capacity(ranks.len());
    let mut pooled = BirthResult::default();
    let mut generation_sum = 0u64;
    let mut genome_size_sum = 0u64;
    let mut total_nodes = 0u64;
    let mut reachable_nodes = 0u64;
    let mut executed_nodes = 0u64;

    for (genome_index, &rank) in ranks.iter().enumerate() {
        let creature_id = creature_ids[rank];
        let creature = &sim.creatures[creature_id];
        let genome = &creature.genome;
        let seed_offset = READ_SEED_BASE + READ_GENOME_MULTIPLIER * (genome_index as u64 + 1);
        let base = battery.signature(genome, context.runtime, context.shared_memory_decay_rate);
        let births = neighborhood::births::per_birth_result(
            genome,
            &base,
            battery,
            mutation_config,
            context,
            sizes.read_births,
            seed_offset,
        );
        let executed = battery
            .executed_indices(genome, context.runtime, context.shared_memory_decay_rate)
            .len() as u64;
        let reachable = mesh_reachable_nodes(genome).len() as u64;
        let nodes = genome.nodes.len() as u64;
        let genome_size = genome.genome_size();

        generation_sum += creature.generation;
        genome_size_sum += u64::from(genome_size);
        total_nodes += nodes;
        reachable_nodes += reachable;
        executed_nodes += executed;
        pooled = pooled.merge(&births);

        genomes.push(NeighborhoodReadGenome {
            rank: rank as u64,
            creature_id: format!("{creature_id:?}"),
            lineage_id: creature.identity.lineage_id,
            generation: creature.generation,
            genome_size,
            total_nodes: nodes,
            reachable_nodes: reachable,
            executed_nodes: executed,
            births_total: births.births_total,
            zero_event_births: births.zero_event_births,
            silent: births.any_events.silent,
            changed: births.any_events.changed,
            dead: births.any_events.dead,
        });
    }

    let sample_size = ranks.len() as u64;
    let births_total = u64::from(pooled.births_total);
    NeighborhoodRead {
        version: NEIGHBORHOOD_READ_VERSION.to_string(),
        battery_version: BATTERY_VERSION.to_string(),
        sample_seed_formula: format!("{READ_SEED_BASE} + world_seed"),
        birth_seed_formula: format!(
            "{READ_SEED_BASE} + {READ_GENOME_MULTIPLIER} * (sample_index + 1) + {} + birth_index",
            neighborhood::births::BIRTH_SEED_BASE
        ),
        population_size: population_size as u64,
        sample_size_requested: sizes.read_sample,
        sample_size: sample_size as u32,
        birth_trials: sizes.read_births,
        silent_per_all_births: fraction_or_undefined(
            u64::from(pooled.any_events.silent),
            births_total,
        ),
        changed_per_all_births: fraction_or_undefined(
            u64::from(pooled.any_events.changed),
            births_total,
        ),
        dead_per_all_births: fraction_or_undefined(u64::from(pooled.any_events.dead), births_total),
        births: to_neighborhood_births(&pooled),
        generation_sum,
        mean_generation: fraction_or_undefined(generation_sum, sample_size),
        genome_size_sum,
        mean_genome_size: fraction_or_undefined(genome_size_sum, sample_size),
        total_nodes,
        mean_total_nodes: fraction_or_undefined(total_nodes, sample_size),
        reachable_nodes,
        mean_reachable_nodes: fraction_or_undefined(reachable_nodes, sample_size),
        executed_nodes,
        mean_executed_nodes: fraction_or_undefined(executed_nodes, sample_size),
        genomes,
    }
}

fn percentile(sorted: &[u32], p: f64) -> u32 {
    if sorted.is_empty() {
        return 0;
    }
    let idx = (p / 100.0 * (sorted.len() - 1) as f64).round();
    let idx = idx.clamp(0.0, (sorted.len() - 1) as f64) as usize;
    sorted[idx]
}

pub(super) fn structure_size_distribution(mut pooled: Vec<u32>) -> StructureSizeDistribution {
    if pooled.is_empty() {
        return StructureSizeDistribution {
            version: Some(REACHABLE_STRUCTURE_VERSION.to_string()),
            min: 0,
            p25: 0,
            median: 0,
            p75: 0,
            max: 0,
            mean: six(0.0),
        };
    }
    pooled.sort_unstable();
    let sum: u64 = pooled.iter().map(|&v| u64::from(v)).sum();
    let mean = sum as f64 / pooled.len() as f64;
    StructureSizeDistribution {
        version: Some(REACHABLE_STRUCTURE_VERSION.to_string()),
        min: pooled[0],
        p25: percentile(&pooled, 25.0),
        median: percentile(&pooled, 50.0),
        p75: percentile(&pooled, 75.0),
        max: pooled[pooled.len() - 1],
        mean: six(mean),
    }
}

/// Assemble the report's `mutational_neighborhood` indicator (T11.F01):
/// `Undefined` when the founder half did not run (the sweep and synthetic
/// profiles), otherwise the battery block, the founder half, and the evolved
/// half (only defined for the goal profile).
pub(super) fn build_mutational_neighborhood_indicator(
    neighborhood_founder: Option<NeighborhoodFounderHalf>,
    params: &ProfileParams,
    observe_goal_indicators: bool,
    evolved_neighborhood_per_seed: Vec<NeighborhoodEvolvedSeed>,
) -> Indicator<MutationalNeighborhood> {
    let Some(founder) = neighborhood_founder else {
        return undefined_mutational_neighborhood();
    };
    Indicator::Defined(MutationalNeighborhood {
        battery: NeighborhoodBattery {
            version: BATTERY_VERSION.to_string(),
            snapshot_seed: neighborhood::battery::SNAPSHOT_SEED,
            sequence_seed: neighborhood::battery::SEQUENCE_SEED,
            snapshot_count: neighborhood::battery::SNAPSHOT_COUNT as u32,
            sequence_count: neighborhood::battery::SEQUENCE_COUNT as u32,
            sequence_len: neighborhood::battery::SEQUENCE_LEN as u32,
            executions_per_genome: neighborhood_battery_execution_count(),
            founder_operator_trials: params.neighborhood.founder_operator_trials,
            founder_birth_count: params.neighborhood.founder_births,
            evolved_operator_trials: params.neighborhood.evolved_operator_trials,
            evolved_birth_count: params.neighborhood.evolved_births,
            evolved_sample_size: neighborhood::SAMPLE_SIZE as u32,
        },
        founder,
        evolved: if observe_goal_indicators {
            Indicator::Defined(EvolvedNeighborhoodHalf {
                per_seed: evolved_neighborhood_per_seed,
            })
        } else {
            undefined_evolved_neighborhood()
        },
    })
}

pub(super) struct GoalIndicatorInputs {
    pub(super) population_persistence_per_seed: Vec<PopulationPersistenceSeed>,
    pub(super) lineage_diversity_per_seed: Vec<LineageDiversitySeed>,
    pub(super) memory_sensitivity_per_seed: Vec<MemorySensitivitySeed>,
    pub(super) structural_companions_per_seed: Vec<StructuralCompanionsSeed>,
    pub(super) temporal_memory_sensitivity_per_seed: Vec<TemporalMemorySensitivitySeed>,
    pub(super) evolved_neighborhood_per_seed: Vec<NeighborhoodEvolvedSeed>,
    pub(super) pooled_complexities: Vec<u32>,
    pub(super) neighborhood_founder: Option<NeighborhoodFounderHalf>,
    pub(super) drift_depth: Indicator<DriftDepth>,
    pub(super) case_observations: Vec<GoalCaseObservation>,
}

pub(super) fn assemble_goal_indicators(
    params: &ProfileParams,
    totals: &Totals,
    inputs: GoalIndicatorInputs,
) -> GoalIndicators {
    let GoalIndicatorInputs {
        population_persistence_per_seed,
        lineage_diversity_per_seed,
        memory_sensitivity_per_seed,
        structural_companions_per_seed,
        temporal_memory_sensitivity_per_seed,
        evolved_neighborhood_per_seed,
        pooled_complexities,
        neighborhood_founder,
        drift_depth,
        case_observations,
    } = inputs;
    let world_set = params.name == GOAL_WORLD_SET;
    let observe_goal_indicators = params.name == "goal" || world_set;
    let population_persistence = PopulationPersistence {
        per_seed: population_persistence_per_seed,
    };

    let births_per_100_ticks = if totals.ticks == 0 {
        six(0.0)
    } else {
        six(totals.births as f64 / totals.ticks as f64 * 100.0)
    };

    GoalIndicators {
        cases: case_observations,
        population_persistence,
        births_per_100_ticks,
        reachable_structure_size_distribution: structure_size_distribution(pooled_complexities),
        lineage_diversity: if observe_goal_indicators {
            Indicator::Defined(LineageDiversity {
                version: Some(LINEAGE_DIVERSITY_VERSION.to_string()),
                per_seed: lineage_diversity_per_seed,
            })
        } else {
            undefined_lineage_diversity()
        },
        memory_sensitivity: if observe_goal_indicators {
            Indicator::Defined(MemorySensitivity {
                version: Some(MEMORY_SENSITIVITY_VERSION.to_string()),
                snapshot_timing: "after the final executed tick, before any observation action"
                    .to_string(),
                scramble_algorithm: "rotate_left(1) across 16 shared-memory slots".to_string(),
                per_seed: memory_sensitivity_per_seed,
            })
        } else {
            undefined_memory_sensitivity()
        },
        structural_companions: observe_goal_indicators.then_some(StructuralCompanionsCensus {
            per_seed: structural_companions_per_seed,
        }),
        temporal_memory_sensitivity: if observe_goal_indicators {
            Indicator::Defined(TemporalMemorySensitivity {
                version: "temporal-memory-v1".to_string(),
                snapshot_timing: "hypothetical cognition from final committed graph state, with final sensors and current/previous shared-memory slots held fixed; no world tick or shared-memory snapshot/decay".to_string(),
                scramble_algorithm: "rotate_left(1) independently within each named slot vector, after graph tick preparation".to_string(),
                per_seed: temporal_memory_sensitivity_per_seed,
            })
        } else {
            undefined_temporal_memory_sensitivity()
        },
        mutational_neighborhood: if world_set {
            Indicator::Undefined("reported per case".to_string())
        } else {
            build_mutational_neighborhood_indicator(
                neighborhood_founder,
                params,
                observe_goal_indicators,
                evolved_neighborhood_per_seed,
            )
        },
        drift_depth,
        recruitment_paths: undefined_recruitment_paths(),
        strategy_count: UNDEFINED.to_string(),
        strategy_causal_distinctness: UNDEFINED.to_string(),
        evolutionary_activity: UNDEFINED.to_string(),
        adaptive_novelty: UNDEFINED.to_string(),
        memory_dependence: UNDEFINED.to_string(),
        learning_dependence: UNDEFINED.to_string(),
        prediction_dependence: UNDEFINED.to_string(),
        information_integration: UNDEFINED.to_string(),
        reciprocal_interaction: UNDEFINED.to_string(),
    }
}

#[cfg(test)]
mod tests;
