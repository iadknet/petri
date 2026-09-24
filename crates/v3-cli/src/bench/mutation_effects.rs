//! The per-world `mutation_effects` block (T11.F26): exposure strata beside
//! the drift and selected readings, single-event attribution cohorts, and the
//! `neighborhood-coverage-v1` extension, projected from
//! `v3_core::neighborhood::mutation_effects`.
//!
//! Count rows are fixed-order integer arrays named once by `count_fields`,
//! keeping the committed summary inside its storage budget.

use serde::{Deserialize, Serialize};
use v3_core::creature::founder::FOUNDER_GENOME_SIZE_UNITS;
use v3_core::neighborhood::births::{CapabilitySplit, QUEUE_BUCKETS};
use v3_core::neighborhood::mutation_effects::{
    self as effects, contexts, Category, CohortReading, EffectCounts, Group, TargetClass,
};
use v3_core::neighborhood::{BirthExposure, BATTERY_VERSION};

use super::schema::Indicator;
use crate::UNDEFINED;

#[cfg(test)]
mod tests;

pub(super) fn undefined_mutation_effects() -> Indicator<MutationEffects> {
    Indicator::Undefined(UNDEFINED.to_string())
}

/// The names of a count row's positions, in order.
#[must_use]
pub fn count_fields() -> Vec<String> {
    ["proposals", "skipped"]
        .into_iter()
        .chain(Category::ALL.map(Category::as_key))
        .chain(["silent_with_state_or_cost", "consistency_violations"])
        .map(str::to_string)
        .collect()
}

fn count_row(counts: &EffectCounts) -> Vec<u32> {
    [counts.proposals, counts.skipped]
        .into_iter()
        .chain(counts.categories)
        .chain([
            counts.silent_with_state_or_cost,
            counts.consistency_violations,
        ])
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityBirths {
    pub silent: u32,
    pub changed: u32,
    pub dead: u32,
    pub zero_applied: u32,
}

impl From<CapabilitySplit> for CapabilityBirths {
    fn from(split: CapabilitySplit) -> Self {
        Self {
            silent: split.silent,
            changed: split.changed,
            dead: split.dead,
            zero_applied: split.zero_applied,
        }
    }
}

/// One exposure stratum: a drift checkpoint or the selected read.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExposureRow {
    pub panel: String,
    pub supply: String,
    pub parents: u32,
    pub parents_all_noop: u32,
    pub parents_one_queue: u32,
    /// Parents by distinct-queue count, bucketed by `queue_buckets`.
    pub distinct_queues_histogram: Vec<u32>,
    pub births_total: u32,
    pub zero_requested: u32,
    pub requested_all_skipped: u32,
    pub event_bearing: u32,
    pub requested_events_total: u64,
    pub applied_events_total: u64,
    pub genome_identical: u32,
    pub from_actionless: CapabilityBirths,
    pub from_acting: CapabilityBirths,
}

fn exposure_row(panel: String, supply: String, exposure: &BirthExposure) -> ExposureRow {
    ExposureRow {
        panel,
        supply,
        parents: exposure.parents,
        parents_all_noop: exposure.parents_all_noop,
        parents_one_queue: exposure.parents_one_queue,
        distinct_queues_histogram: exposure.distinct_queues_histogram.to_vec(),
        births_total: exposure.births_total,
        zero_requested: exposure.zero_requested,
        requested_all_skipped: exposure.requested_all_skipped,
        event_bearing: exposure.event_bearing,
        requested_events_total: exposure.requested_events_total,
        applied_events_total: exposure.applied_events_total,
        genome_identical: exposure.genome_identical,
        from_actionless: exposure.from_actionless.into(),
        from_acting: exposure.from_acting.into(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParentRowBlock {
    pub index: u64,
    pub depth_or_generation: u64,
    pub genome_size: u32,
    pub total_nodes: u32,
    pub reachable_nodes: u32,
    pub executed_nodes: u32,
    pub contributing_nodes: u32,
    pub all_noop: bool,
    pub distinct_queues: u32,
    pub counts: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyedCounts {
    pub key: String,
    pub counts: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CohortCoverage {
    pub parents_evaluated: u32,
    pub pairs_requested: u32,
    pub pairs_sampled: u32,
    /// `null` when the world recorded no context.
    pub differ_recorded: Option<u32>,
    pub differ_authored: u32,
    pub differ_sequence_ticks_1_4: u32,
    pub differ_sequence_ticks_5_32: u32,
    /// Pairs whose actions differ in some group.
    pub differ_any: u32,
    /// Pairs with identical actions in every group and a differing
    /// state-and-cost record.
    pub state_or_cost_only: u32,
    pub all_noop_parents: u32,
    pub all_noop_parents_acting: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cohort {
    pub cohort: String,
    pub identity: String,
    pub parents_requested: u32,
    pub parents_evaluated: u32,
    pub totals: Vec<u32>,
    pub parents: Vec<ParentRowBlock>,
    /// Rows only for operators drawn; `<Domain>.domain_exhausted` rows hold
    /// events no operator accepted.
    pub operators: Vec<KeyedCounts>,
    /// Rows by first-recorded-target class, a stratum label.
    pub targets: Vec<KeyedCounts>,
    pub coverage: CohortCoverage,
}

fn cohort_block(reading: &CohortReading, identity: String, recorded: bool) -> Cohort {
    let coverage = reading.coverage;
    let group = |group: Group| coverage.differ_by_group[group.index()];
    Cohort {
        cohort: reading.cohort.as_key().to_string(),
        identity,
        parents_requested: reading.parents_requested,
        parents_evaluated: reading.parents.len() as u32,
        totals: count_row(&reading.totals),
        parents: reading
            .parents
            .iter()
            .map(|row| ParentRowBlock {
                index: row.index,
                depth_or_generation: row.depth_or_generation,
                genome_size: row.genome_size,
                total_nodes: row.total_nodes,
                reachable_nodes: row.reachable_nodes,
                executed_nodes: row.executed_nodes,
                contributing_nodes: row.contributing_nodes,
                all_noop: row.all_noop,
                distinct_queues: row.distinct_queues,
                counts: count_row(&row.counts),
            })
            .collect(),
        operators: reading
            .operators
            .iter()
            .map(|(operator, counts)| KeyedCounts {
                key: operator.as_key(),
                counts: count_row(counts),
            })
            .collect(),
        targets: TargetClass::ALL
            .into_iter()
            .map(|class| KeyedCounts {
                key: class.as_key().to_string(),
                counts: count_row(&reading.targets[class.index()]),
            })
            .collect(),
        coverage: CohortCoverage {
            parents_evaluated: coverage.parents_evaluated,
            pairs_requested: coverage.pairs_requested,
            pairs_sampled: coverage.pairs_sampled,
            differ_recorded: recorded.then(|| group(Group::Recorded)),
            differ_authored: group(Group::Authored),
            differ_sequence_ticks_1_4: group(Group::SequenceEarly),
            differ_sequence_ticks_5_32: group(Group::SequenceLate),
            differ_any: coverage.differ_any,
            state_or_cost_only: coverage.state_or_cost_only,
            all_noop_parents: coverage.all_noop_parents,
            all_noop_parents_acting: coverage.all_noop_parents_acting,
        },
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordedGroup {
    pub requested: u32,
    pub actual: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlBlock {
    pub name: String,
    pub expectation: String,
    pub silent_on_original: bool,
    /// Groups whose actions differ between the control's parent and child.
    pub differing_groups: Vec<String>,
    pub state_differs: bool,
    pub passed: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Coverage {
    pub version: String,
    pub pair_rule: String,
    pub recorded_rule: String,
    pub authored_rule: String,
    pub sequence_rule: String,
    pub recorded: Indicator<RecordedGroup>,
    pub authored_contexts: u32,
    pub channel_groups: Vec<String>,
    pub sequences: u32,
    pub sequence_ticks: u32,
    /// `recorded`, or `authored` when fewer than four contexts were recorded.
    pub sequence_source: String,
    pub controls: Vec<ControlBlock>,
}

/// One world's `mutation_effects` block.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MutationEffects {
    pub version: String,
    pub battery_version: String,
    pub proposal_rule: String,
    pub proposal_seed_formula: String,
    pub selected_rule: String,
    pub genome_identity: String,
    pub count_fields: Vec<String>,
    pub queue_buckets: Vec<String>,
    pub proposals_per_parent: u32,
    pub exposure: Vec<ExposureRow>,
    pub cohorts: Vec<Indicator<Cohort>>,
    pub coverage: Coverage,
}

/// The existing readings' exposure inputs: each drift checkpoint's depth and
/// strata, and the selected read's strata when the world was read.
pub struct ExposureInputs<'a> {
    pub drift: &'a [(u64, BirthExposure)],
    pub read: Option<&'a BirthExposure>,
}

#[must_use]
pub fn project(
    reading: &effects::Reading,
    exposure: &ExposureInputs<'_>,
    sizes: effects::Sizes,
) -> MutationEffects {
    let pinned = format!("pinned {FOUNDER_GENOME_SIZE_UNITS} units (drift chart)");
    let mut rows: Vec<ExposureRow> = exposure
        .drift
        .iter()
        .map(|(depth, strata)| exposure_row(format!("drift@{depth}"), pinned.clone(), strata))
        .collect();
    rows.extend(exposure.read.map(|strata| {
        exposure_row(
            "selected-read".to_string(),
            "own genome_size (production)".to_string(),
            strata,
        )
    }));
    let recorded = reading.recorded_contexts.is_ok();
    let drift_depth = reading
        .drift
        .parents
        .first()
        .map_or(0, |row| row.depth_or_generation);
    let cohorts = vec![
        Indicator::Defined(cohort_block(
            &reading.founder,
            "canonical V3Alpha1 founder, reference only".to_string(),
            recorded,
        )),
        Indicator::Defined(cohort_block(
            &reading.drift,
            format!("drift walk birth lineages at depth {drift_depth} (pinned supply walk)"),
            recorded,
        )),
        match &reading.selected {
            Ok(selected) => Indicator::Defined(cohort_block(
                selected,
                "selected genomes: T14.F12 sample positions, terminal population".to_string(),
                recorded,
            )),
            Err(reason) => Indicator::Undefined(reason.clone()),
        },
    ];
    MutationEffects {
        version: effects::VERSION.to_string(),
        battery_version: BATTERY_VERSION.to_string(),
        proposal_rule: "one requested event per proposal: apply_mutations_on_units with units 1 \
                        and per_unit_rate 1.0, the production operator mix, and the parent's \
                        battery-executed set; skipped proposals stay in denominators, no retry"
            .to_string(),
        proposal_seed_formula: format!(
            "{} + {} * cohort (founder 0, drift 1, selected 2) + {} * (parent_index + 1) + proposal_index",
            effects::PROPOSAL_SEED_BASE,
            effects::COHORT_SEED_MULTIPLIER,
            effects::PARENT_SEED_MULTIPLIER
        ),
        selected_rule: format!(
            "min({}, s) of the T14.F12 sample positions, uniform without replacement from \
             SmallRng::seed_from_u64({} + world_seed), ascending",
            sizes.parents,
            effects::SELECTED_SEED_BASE
        ),
        genome_identity: "equal Debug rendering: distinguishes -0.0, ignores NaN payloads, \
                          omits birth_weights"
            .to_string(),
        count_fields: count_fields(),
        queue_buckets: QUEUE_BUCKETS.map(str::to_string).to_vec(),
        proposals_per_parent: sizes.proposals,
        exposure: rows,
        cohorts,
        coverage: Coverage {
            version: effects::COVERAGE_VERSION.to_string(),
            pair_rule: format!(
                "per parent, the first {} proposals in proposal order that are applied, not \
                 genome-identical, and action-silent on {BATTERY_VERSION}",
                sizes.pairs_per_parent
            ),
            recorded_rule: format!(
                "up to {} living creatures sampled without replacement from the id-sorted \
                 terminal population with SmallRng::seed_from_u64({} + world_seed), in sample \
                 order; production assemblers with typed local food and extended perception \
                 assembled unconditionally; the creature's own energy",
                sizes.recorded_contexts,
                contexts::RECORDED_SEED_BASE
            ),
            authored_rule: format!(
                "context i copies {BATTERY_VERSION} snapshot i and sets channel group i mod 8 \
                 inside production ranges from SmallRng::seed_from_u64({} + i); not guaranteed \
                 realizable",
                contexts::AUTHORED_SEED_BASE
            ),
            sequence_rule: "sequence s tick t uses recorded context (8s + t) mod r (authored \
                            contexts when r < 4); production shared-memory bookkeeping, energy \
                            reset each tick; ticks 1-4 and 5-32 reported apart"
                .to_string(),
            recorded: match &reading.recorded_contexts {
                Ok(actual) => Indicator::Defined(RecordedGroup {
                    requested: reading.recorded_requested,
                    actual: *actual,
                }),
                Err(reason) => Indicator::Undefined(reason.clone()),
            },
            authored_contexts: contexts::AUTHORED_CONTEXTS as u32,
            channel_groups: contexts::CHANNEL_GROUPS.map(str::to_string).to_vec(),
            sequences: contexts::SEQUENCES as u32,
            sequence_ticks: contexts::SEQUENCE_TICKS as u32,
            sequence_source: reading.sequence_source.to_string(),
            controls: reading
                .controls
                .iter()
                .map(|control| ControlBlock {
                    name: control.name.to_string(),
                    expectation: control.expectation.to_string(),
                    silent_on_original: control.silent_on_original,
                    differing_groups: Group::ALL
                        .into_iter()
                        .filter(|group| control.differs_by_group[group.index()])
                        .map(|group| group.as_key().to_string())
                        .collect(),
                    state_differs: control.state_differs,
                    passed: control.passed,
                })
                .collect(),
        },
    }
}

/// Everything one world's reading needs beside its terminal state.
#[derive(Clone, Copy)]
pub(super) struct WorldObservation<'a> {
    pub battery: &'a v3_core::neighborhood::Battery,
    pub config: &'a v3_core::config::SimulationConfig,
    pub read_sample: u32,
    pub drift: &'a super::indicators::DriftCohortInputs,
    pub read_exposure: Option<&'a BirthExposure>,
    pub sizes: effects::Sizes,
}

/// One world's block: the selected cohort drawn over the T14.F12 read's
/// sample positions, joined to the read's rows by position.
pub(super) fn observe_world(
    seed: u64,
    sim: &v3_core::simulation::Simulation,
    observation: WorldObservation<'_>,
) -> MutationEffects {
    let ids = super::indicators::sorted_creature_ids(sim);
    let ranks =
        v3_core::neighborhood::read_sample_ranks(ids.len(), observation.read_sample as usize, seed);
    let parents: Vec<effects::CohortParent> =
        effects::selected_positions(ranks.len(), observation.sizes.parents as usize, seed)
            .into_iter()
            .map(|position| {
                let creature = &sim.creatures[ids[ranks[position]]];
                effects::CohortParent {
                    index: position as u64,
                    depth_or_generation: creature.generation,
                    genome: creature.genome.clone(),
                }
            })
            .collect();
    let selected = if parents.is_empty() {
        Err("extinct: no living creature at the terminal tick")
    } else {
        Ok(parents.as_slice())
    };
    let founder =
        v3_core::creature::founder::founder_genome(v3_core::config::FounderProfile::V3Alpha1);
    let context = v3_core::neighborhood::EvalContext::from_config(observation.config);
    let reading = effects::observe(
        effects::WorldInputs {
            founder: &founder,
            drift: &observation.drift.parents,
            selected,
            sim,
            world_seed: seed,
        },
        observation.battery,
        &observation.config.mutation,
        &context,
        observation.sizes,
    );
    project(
        &reading,
        &ExposureInputs {
            drift: &observation.drift.exposure,
            read: observation.read_exposure,
        },
        observation.sizes,
    )
}
