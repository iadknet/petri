use super::*;
use crate::bench::profiles::{goal_recipe_seeds, GOAL_WORLD_SET};
use crate::bench::run::run_deterministic;
use crate::bench::schema::NeighborhoodBirths;
use crate::bench::tests::small_profile;

/// An exposure row's partitions sum to its births, and its capability split
/// reconciles exactly with the tallies of the reading it describes.
fn assert_reconciles(row: &ExposureRow, births: &NeighborhoodBirths) {
    assert_eq!(row.births_total, births.births_total);
    assert_eq!(
        row.zero_requested + row.requested_all_skipped + row.event_bearing,
        row.births_total
    );
    let split =
        |pick: fn(&CapabilityBirths) -> u32| pick(&row.from_actionless) + pick(&row.from_acting);
    assert_eq!(split(|side| side.silent), births.any_events.silent);
    assert_eq!(split(|side| side.changed), births.any_events.changed);
    assert_eq!(split(|side| side.dead), births.any_events.dead);
    assert_eq!(split(|side| side.zero_applied), births.zero_event_births);
    assert_eq!(row.event_bearing, births.any_events.trials);
    assert_eq!(
        row.distinct_queues_histogram.iter().sum::<u32>(),
        row.parents
    );
    assert!(row.genome_identical <= row.event_bearing);
}

fn assert_cohort_partitions(cohort: &Cohort) {
    let applied = |counts: &[u32]| counts[0] - counts[1];
    let categories = |counts: &[u32]| counts[2..9].iter().sum::<u32>();
    assert_eq!(categories(&cohort.totals), applied(&cohort.totals));
    for rows in [&cohort.operators, &cohort.targets] {
        let summed: Vec<u32> = (0..cohort.totals.len())
            .map(|field| rows.iter().map(|row| row.counts[field]).sum())
            .collect();
        assert_eq!(summed, cohort.totals);
    }
    let parents: Vec<u32> = (0..cohort.totals.len())
        .map(|field| cohort.parents.iter().map(|row| row.counts[field]).sum())
        .collect();
    assert_eq!(parents, cohort.totals);
    assert_eq!(cohort.parents_evaluated as usize, cohort.parents.len());
    assert!(cohort.coverage.pairs_sampled <= cohort.coverage.pairs_requested);
}

#[test]
fn count_fields_name_every_position_of_a_count_row() {
    let fields = count_fields();
    assert_eq!(fields.len(), count_row(&EffectCounts::default()).len());
    assert_eq!(fields[2], "action_changed");
    assert_eq!(fields[8], "unresolved");
    assert_eq!(fields[10], "consistency_violations");
}

#[test]
fn world_set_cases_carry_reconciled_mutation_effects_on_the_existing_readings() {
    let mut params = small_profile(GOAL_WORLD_SET);
    params.seeds = goal_recipe_seeds();
    let (report, timings) = run_deterministic(&params).expect("a valid profile");
    assert_eq!(timings.mutation_effects_wall_clock_ms_per_seed.len(), 3);
    for case in &report.goal_indicators.cases {
        let effects = case
            .mutation_effects
            .defined()
            .expect("defined on the world set");
        let drift = case
            .drift_depth
            .defined()
            .expect("drift runs on the world set");
        assert_eq!(effects.version, "mutation-effects-v1");
        assert_eq!(effects.coverage.version, "neighborhood-coverage-v1");
        for (row, checkpoint) in effects.exposure.iter().zip(&drift.readings) {
            assert_eq!(row.panel, format!("drift@{}", checkpoint.depth));
            assert_eq!(row.supply, "pinned 97 units (drift chart)");
            assert_reconciles(row, &checkpoint.births);
        }
        match case.neighborhood_read.defined() {
            Some(read) => {
                let row = effects.exposure.last().expect("a read row");
                assert_eq!(row.panel, "selected-read");
                assert_reconciles(row, &read.births);
                assert_eq!(effects.exposure.len(), drift.readings.len() + 1);
            }
            None => assert_eq!(effects.exposure.len(), drift.readings.len()),
        }
        assert_eq!(effects.cohorts.len(), 3);
        for cohort in effects.cohorts.iter().filter_map(Indicator::defined) {
            assert_cohort_partitions(cohort);
        }
        assert!(effects
            .coverage
            .controls
            .iter()
            .all(|control| control.passed));
    }
    let (gate, _) = run_deterministic(&small_profile("gate")).expect("a valid profile");
    assert!(gate.goal_indicators.cases.is_empty());
}

#[test]
fn a_report_without_the_block_reads_back_unmeasured() {
    let mut params = small_profile(GOAL_WORLD_SET);
    params.seeds = goal_recipe_seeds();
    let (report, _) = run_deterministic(&params).expect("a valid profile");
    let mut case = serde_json::to_value(&report.goal_indicators.cases[0]).unwrap();
    case.as_object_mut().unwrap().remove("mutation_effects");
    let historical: crate::bench::schema::GoalCaseObservation =
        serde_json::from_value(case).unwrap();
    assert!(historical.mutation_effects.defined().is_none());
    let encoded = serde_json::to_value(&report.goal_indicators.cases[0].mutation_effects).unwrap();
    let decoded: Indicator<MutationEffects> = serde_json::from_value(encoded).unwrap();
    assert_eq!(decoded, report.goal_indicators.cases[0].mutation_effects);
}

#[test]
fn world_set_blocks_are_byte_identical_across_thread_counts() {
    let mut params = small_profile(GOAL_WORLD_SET);
    params.seeds = goal_recipe_seeds();
    let run = |threads| {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .unwrap()
            .install(|| {
                let (report, _) = run_deterministic(&params).expect("a valid profile");
                serde_json::to_vec(&report.goal_indicators.cases).unwrap()
            })
    };
    assert_eq!(run(1), run(3));
}
