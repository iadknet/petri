//! Indicator computations over a finished simulation.

use super::profiles::{is_goal_world_set, NeighborhoodSizes, ProfileParams};
use super::run::millis;
use super::schema::{
    cohort_ladder, undefined_drift_depth, undefined_evolved_neighborhood,
    undefined_generation_distribution, undefined_lineage_diversity, undefined_memory_sensitivity,
    undefined_mutational_neighborhood, undefined_temporal_memory_sensitivity, CohortLineageRow,
    CycleClasses, DriftDepth, DriftDepthCheckpoint, EvolvedNeighborhoodHalf, FounderRow,
    GenerationDistribution, GoalCaseObservation, GoalIndicators, Indicator, LineageDiversity,
    LineageDiversitySeed, LineageOpportunityRow, MemorySensitivity, MemorySensitivitySeed,
    MeshExecution, ModuleRecruitment, MutationOpportunities, MutationalNeighborhood,
    NeighborhoodBattery, NeighborhoodBirthBucket, NeighborhoodBirthCyclePartition,
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
use v3_core::genome::{structural_companions, StructuralCompanions};
use v3_core::neighborhood::clades::CladeCensus;
use v3_core::neighborhood::final_census::{CompanionCounts, DifferingCounts};
use v3_core::neighborhood::steering::{self, SteeringBattery};
use v3_core::neighborhood::{
    self, evaluate_genome, Battery, BirthResult, EvalContext, GenomeEvaluation, OperatorRow, Tally,
    BATTERY_VERSION, READ_GENOME_MULTIPLIER, READ_SEED_BASE,
};
use v3_core::neighborhood::{distributions, population_sample};
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
        route_varying_within_snapshot_lineages: mesh.route_varying_within_snapshot_lineages,
        route_destination_varying_within_snapshot_lineages: mesh
            .route_destination_varying_within_snapshot_lineages,
        battery_executions,
        pass_cap_hits: mesh.pass_cap_hits,
        tick_reasons: mesh.tick_reasons,
        passes: mesh.passes,
        decided_passes: mesh.decided_passes,
        pass_cap_fraction: fraction_or_undefined(mesh.pass_cap_hits, mesh.passes),
        cycle_classes: Some(CycleClasses {
            cycle_carrying_lineages: mesh.cycle_carrying_lineages,
            cycle_carrying_fraction: fraction_or_undefined(
                u64::from(mesh.cycle_carrying_lineages),
                denominator,
            ),
            revisiting_lineages: mesh.revisiting_lineages,
            revisiting_fraction: fraction_or_undefined(
                u64::from(mesh.revisiting_lineages),
                denominator,
            ),
            productive_cycle_lineages: mesh.productive_cycle_lineages,
            productive_cycle_fraction: fraction_or_undefined(
                u64::from(mesh.productive_cycle_lineages),
                denominator,
            ),
        }),
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

/// What the drift walk hands the mutation-effects reading (T11.F26): each
/// checkpoint's exposure strata and the birth lineages at the last one.
pub(super) struct DriftCohortInputs {
    pub exposure: Vec<(u64, neighborhood::BirthExposure)>,
    pub parents: Vec<neighborhood::mutation_effects::CohortParent>,
}

pub(super) fn timed_drift_depth(
    params: &ProfileParams,
    config: &SimulationConfig,
    battery: Option<&Battery>,
) -> (
    Indicator<DriftDepth>,
    Option<f64>,
    Option<DriftCohortInputs>,
) {
    if params.name != "goal" && !is_goal_world_set(&params.name) {
        return (undefined_drift_depth(), None, None);
    }
    let started = Instant::now();
    let (reading, cohort) =
        compute_drift_depth(config, battery.expect("goal battery"), params.drift);
    (
        Indicator::Defined(reading),
        Some(millis(started.elapsed())),
        Some(cohort),
    )
}

fn compute_drift_depth(
    config: &SimulationConfig,
    battery: &Battery,
    sizes: neighborhood::drift::DriftSizes,
) -> (DriftDepth, DriftCohortInputs) {
    use neighborhood::{battery as fixed_battery, drift, mesh_execution, recruitment};
    // Founder 0's body under the config; a report-level reading has no
    // world seed, so its body takes startup seed 0 (T29.F02).
    let founder = founder_genome(
        v3_core::config::FounderProfile::V3Alpha1,
        v3_core::creature::founder::founder_body(0, 0, config),
    );
    let readings = drift::observe(
        &founder,
        battery,
        &config.mutation,
        &EvalContext::from_config(config),
        sizes,
    );
    let last_depth = readings.checkpoints.last().map_or(0, |row| row.depth);
    let cohort = DriftCohortInputs {
        exposure: readings
            .checkpoints
            .iter()
            .map(|row| (row.depth, row.exposure))
            .collect(),
        parents: readings
            .final_birth_genomes
            .into_iter()
            .enumerate()
            .map(
                |(lineage, genome)| neighborhood::mutation_effects::CohortParent {
                    index: lineage as u64,
                    depth_or_generation: last_depth,
                    genome,
                },
            )
            .collect(),
    };
    let depth = DriftDepth {
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
    };
    (depth, cohort)
}

pub(super) fn generation_distribution(generations: Vec<u64>) -> Indicator<GenerationDistribution> {
    match distributions::generation_distribution(generations) {
        Some(distribution) => Indicator::Defined(GenerationDistribution {
            median: distribution.median,
            max: distribution.max,
        }),
        None => undefined_generation_distribution(),
    }
}

/// One genome's `mesh_execution` block and its `steering-v1` block from the
/// readers' single observed battery pass.
pub(super) fn mesh_execution_and_steering(
    battery: &Battery,
    steering_battery: &SteeringBattery,
    genome: &v3_core::genome::CreatureGenome,
    context: &EvalContext,
) -> (MeshExecution, Steering) {
    let (reading, steering) =
        population_sample::mesh_execution_and_steering(battery, steering_battery, genome, context);
    (mesh_execution_block(reading), steering_block(steering))
}

fn mesh_execution_block(
    reading: v3_core::neighborhood::mesh_execution::MeshExecutionReading,
) -> MeshExecution {
    use v3_core::neighborhood::mesh_execution::{KNOCKOUT_METHOD, MESH_EXECUTION_VERSION};
    MeshExecution {
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
        route_destination_varies: reading.route_destination_varies,
        route_varies_within_snapshot: reading.route_varies_within_snapshot,
        route_destination_varies_within_snapshot: reading.route_destination_varies_within_snapshot,
        pass_cap_hits: reading.pass_cap_hits as u64,
        tick_reasons: reading.tick_reasons,
        passes: reading.passes as u64,
        decided_passes: reading.decided_passes as u64,
        cycle_carrying: reading.cycle_carrying,
        revisiting: reading.revisiting,
        productive_cycle: reading.productive_cycle,
    }
}

fn steering_block(reading: steering::SteeringReading) -> Steering {
    Steering {
        version: steering::STEERING_VERSION.to_string(),
        seed: steering::STEERING_SEED,
        base_count: steering::STEERING_BASE_COUNT as u32,
        reading,
    }
}

/// The pooled `steering-v1` block for one seed's sample.
pub(super) fn steering_pooled(pooled: steering::SteeringPooled) -> Indicator<SteeringPooled> {
    Indicator::Defined(SteeringPooled {
        version: steering::STEERING_VERSION.to_string(),
        sums: pooled,
        exact_hit_fraction: fraction_or_undefined(pooled.exact_hits, pooled.moves),
        within_45_fraction: fraction_or_undefined(pooled.within_45, pooled.moves),
        avoidance_fraction: fraction_or_undefined(pooled.avoided, pooled.avoidance_trials),
        move_voted_fraction: fraction_or_undefined(pooled.move_voted, pooled.genomes),
        chance: SteeringChance {
            exact: steering::CHANCE_EXACT,
            within_45: steering::CHANCE_WITHIN_45,
        },
    })
}

/// Surviving founder clade count and the Shannon entropy of their size
/// distribution in nats, from the readers' clade census; the entropy is
/// `UNDEFINED` for an empty population.
pub(super) fn clade_diversity(lineage_ids: impl IntoIterator<Item = u32>) -> (u64, String) {
    let census = CladeCensus::of(lineage_ids);
    let entropy = census
        .entropy_nats()
        .map_or_else(|| UNDEFINED.to_string(), six);
    (census.surviving(), entropy)
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
    genomes: impl Iterator<Item = &'a v3_core::genome::CreatureGenome>,
) -> StructuralCompanionsSeed {
    let counts = CompanionCounts::of(genomes);
    StructuralCompanionsSeed {
        seed,
        final_creature_count: counts.creatures,
        reads_shared_memory: counts.reads_shared_memory,
        writes_shared_memory: counts.writes_shared_memory,
        has_stateful_compute_node: counts.has_stateful_compute_node,
        has_plasticity: counts.has_plasticity,
    }
}

fn memory_sensitivity_seed(seed: u64, counts: DifferingCounts) -> MemorySensitivitySeed {
    MemorySensitivitySeed {
        seed,
        final_creature_count: counts.creatures,
        different_from_zeroed_count: counts.from_zeroed,
        different_from_scrambled_count: counts.from_scrambled,
        different_from_either_count: counts.from_either,
        different_from_zeroed_fraction: fraction_or_undefined(counts.from_zeroed, counts.creatures),
        different_from_scrambled_fraction: fraction_or_undefined(
            counts.from_scrambled,
            counts.creatures,
        ),
        different_from_either_fraction: fraction_or_undefined(counts.from_either, counts.creatures),
    }
}

pub(super) fn memory_sensitivity(
    seed: u64,
    observations: &[FinalActionObservation],
) -> MemorySensitivitySeed {
    memory_sensitivity_seed(seed, DifferingCounts::of(observations))
}

pub(super) fn temporal_memory_sensitivity(
    seed: u64,
    sim: &v3_core::simulation::Simulation,
) -> TemporalMemorySensitivitySeed {
    use v3_core::simulation::{observe_temporal_actions, TemporalMemorySubstrate};
    let observations = observe_temporal_actions(sim);
    let component = |substrate| {
        memory_sensitivity_seed(
            seed,
            DifferingCounts::of_substrate(&observations, substrate),
        )
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
        reordered: tally.reordered,
        recount: tally.recount,
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
        by_cycle_carrying: Some(NeighborhoodBirthCyclePartition {
            cycle_carrying: to_neighborhood_tally(&result.cycle_carrying),
            acyclic: to_neighborhood_tally(&result.acyclic),
        }),
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
    // Founder 0's body under the config; a report-level reading has no
    // world seed, so its body takes startup seed 0 (T29.F02).
    let subject = founder_genome(
        config.population.founder_profile,
        v3_core::creature::founder::founder_body(0, 0, config),
    );
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
    let steering_battery = SteeringBattery::generate(context.sensors.food_type_count);
    let (mesh_execution, steering) =
        mesh_execution_and_steering(battery, &steering_battery, &subject, &context);
    NeighborhoodFounderHalf {
        generation: Some(0),
        mesh_execution: Indicator::Defined(mesh_execution),
        steering: Indicator::Defined(steering),
        reachable_node_count: structural_companions(&subject).reachable_node_count as u64,
        operator_rows: to_neighborhood_operator_rows(&evaluation.operator_rows),
        births: to_neighborhood_births(&evaluation.births),
    }
}

/// The evolved half for one seed's final living population, from the
/// readers' evolved observation: each sampled genome's full reading and
/// structural companions, and the pooled per-operator and per-birth tallies
/// across the sample.
pub(super) fn evolved_neighborhood_for_seed(
    seed: u64,
    sim: &v3_core::simulation::Simulation,
    battery: &Battery,
    mutation_config: &MutationConfig,
    context: &EvalContext,
    sizes: NeighborhoodSizes,
) -> NeighborhoodEvolvedSeed {
    let observation = population_sample::evolved_observation(
        sim,
        battery,
        mutation_config,
        context,
        population_sample::EvaluationSizes {
            operator_trials: sizes.evolved_operator_trials,
            births: sizes.evolved_births,
        },
    );
    let sampled_genomes = observation
        .genomes
        .into_iter()
        .map(|genome| NeighborhoodSampledGenome {
            generation: Some(genome.generation),
            mesh_execution: Indicator::Defined(mesh_execution_block(genome.mesh_execution)),
            steering: Indicator::Defined(steering_block(genome.steering)),
            rank: genome.rank as u64,
            creature_id: format!("{:?}", genome.creature_id),
            operator_rows: to_neighborhood_operator_rows(&genome.evaluation.operator_rows),
            births: to_neighborhood_births(&genome.evaluation.births),
            companions: to_neighborhood_companions(&genome.companions),
        })
        .collect();
    let pooled_operator_rows = v3_core::neighborhood::operator_catalog()
        .iter()
        .zip(observation.pooled_operator_tallies.iter())
        .map(|((family, name), tally)| NeighborhoodOperatorRow {
            family: (*family).to_string(),
            operator: name.clone(),
            tally: to_neighborhood_tally(tally),
        })
        .collect();

    NeighborhoodEvolvedSeed {
        generation_distribution: generation_distribution(observation.generations),
        seed,
        final_population_size: observation.population_size as u64,
        sampled_genomes,
        pooled_operator_rows,
        pooled_births: to_neighborhood_births(&observation.pooled_births),
        steering_pooled: steering_pooled(observation.pooled_steering),
    }
}

/// The neighborhood read (T14.F12) for one world's final living population,
/// from the readers' read observation, and beside it the exposure strata of
/// the same births (T11.F26).
pub(super) fn neighborhood_read_for_seed(
    seed: u64,
    sim: &v3_core::simulation::Simulation,
    battery: &Battery,
    mutation_config: &MutationConfig,
    context: &EvalContext,
    sizes: NeighborhoodSizes,
) -> (NeighborhoodRead, neighborhood::BirthExposure) {
    let observation = population_sample::read_observation(
        seed,
        sim,
        battery,
        mutation_config,
        context,
        sizes.read_sample as usize,
        sizes.read_births,
    );
    let pooled = &observation.pooled;
    let sample_size = observation.genomes.len() as u64;
    let births_total = u64::from(pooled.births_total);
    let read = NeighborhoodRead {
        version: NEIGHBORHOOD_READ_VERSION.to_string(),
        battery_version: BATTERY_VERSION.to_string(),
        sample_seed_formula: format!("{READ_SEED_BASE} + world_seed"),
        birth_seed_formula: format!(
            "{READ_SEED_BASE} + {READ_GENOME_MULTIPLIER} * (sample_index + 1) + {} + birth_index",
            neighborhood::births::BIRTH_SEED_BASE
        ),
        population_size: observation.population_size as u64,
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
        births: to_neighborhood_births(pooled),
        generation_sum: observation.generation_sum,
        mean_generation: fraction_or_undefined(observation.generation_sum, sample_size),
        genome_size_sum: observation.genome_size_sum,
        mean_genome_size: fraction_or_undefined(observation.genome_size_sum, sample_size),
        total_nodes: observation.total_nodes,
        mean_total_nodes: fraction_or_undefined(observation.total_nodes, sample_size),
        reachable_nodes: observation.reachable_nodes,
        mean_reachable_nodes: fraction_or_undefined(observation.reachable_nodes, sample_size),
        executed_nodes: observation.executed_nodes,
        mean_executed_nodes: fraction_or_undefined(observation.executed_nodes, sample_size),
        genomes: observation
            .genomes
            .iter()
            .map(|genome| NeighborhoodReadGenome {
                rank: genome.rank as u64,
                creature_id: format!("{:?}", genome.creature_id),
                lineage_id: genome.lineage_id,
                generation: genome.generation,
                genome_size: genome.genome_size,
                total_nodes: genome.total_nodes,
                reachable_nodes: genome.reachable_nodes,
                executed_nodes: genome.executed_nodes,
                births_total: genome.births.births_total,
                zero_event_births: genome.births.zero_event_births,
                silent: genome.births.any_events.silent,
                changed: genome.births.any_events.changed,
                dead: genome.births.any_events.dead,
            })
            .collect(),
    };
    (read, observation.exposure)
}

pub(super) fn structure_size_distribution(pooled: Vec<u32>) -> StructureSizeDistribution {
    let sizes = distributions::structure_sizes(pooled);
    StructureSizeDistribution {
        version: Some(REACHABLE_STRUCTURE_VERSION.to_string()),
        min: sizes.min,
        p25: sizes.p25,
        median: sizes.median,
        p75: sizes.p75,
        max: sizes.max,
        mean: six(sizes.mean),
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
    let world_set = is_goal_world_set(&params.name);
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
