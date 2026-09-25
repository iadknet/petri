use super::*;
use crate::bench::profiles::{goal_recipe_seeds, GOAL_WORLD_SET};
use crate::bench::run::run_deterministic;
use crate::bench::tests::small_profile;

/// Every defined row keeps the funnel's order and names a stage; shared
/// memory rows carry no declaration or causal stage.
fn assert_rows(cohort: &CohortBlock) {
    assert!(cohort.parents_evaluated <= cohort.parents_requested);
    assert!(cohort.retention_children_sampled <= cohort.retention_children_requested);
    for text in &cohort.rows {
        let row = RowBlock::decode(text).expect("a row string");
        assert_eq!(row.encode(), *text);
        let at = |field: &str| row.counts[ROW_FIELDS.iter().position(|f| *f == field).unwrap()];
        let shared = row.family.starts_with("SharedMemory");
        assert_eq!(at("declared").is_none(), shared, "{row:?}");
        assert_eq!(at("causal").is_none(), shared, "{row:?}");
        if !shared {
            let declared = at("declared").unwrap();
            assert!(declared >= at("connected").unwrap(), "{row:?}");
            assert!(declared >= at("executed").unwrap(), "{row:?}");
            assert!(at("executed").unwrap() >= at("causal").unwrap(), "{row:?}");
            assert!(
                at("causal").unwrap() >= at("causal_original").unwrap(),
                "{row:?}"
            );
        }
        assert!(
            row.counts.iter().any(|count| count.unwrap_or(0) > 0),
            "{row:?}"
        );
    }
}

#[test]
fn world_set_cases_carry_the_input_use_block_and_the_gate_does_not() {
    let mut params = small_profile(GOAL_WORLD_SET);
    params.seeds = goal_recipe_seeds();
    let (report, timings) = run_deterministic(&params).expect("a valid profile");
    assert_eq!(timings.input_use_wall_clock_ms_per_seed.len(), 3);
    for case in &report.goal_indicators.cases {
        let block = case.input_use.defined().expect("defined on the world set");
        assert_eq!(block.version, "input-use-v1");
        assert_eq!(block.row_format, ROW_FORMAT);
        assert_eq!(block.scenes.original_executions, 80);
        assert_eq!(block.cohorts.len(), 3);
        let founder = block.cohorts[0].defined().expect("the founder cohort");
        assert_eq!(founder.consistency_violations, 0);
        assert!(founder
            .rows
            .iter()
            .filter_map(|text| RowBlock::decode(text))
            .any(|row| row.family == "FoodHere:0" && row.counts[3] == Some(1)));
        for cohort in block.cohorts.iter().filter_map(Indicator::defined) {
            assert_rows(cohort);
        }
    }
    let (gate, gate_timings) = run_deterministic(&small_profile("gate")).expect("a valid profile");
    assert!(gate.goal_indicators.cases.is_empty());
    assert!(gate_timings.input_use_wall_clock_ms_per_seed.is_empty());
}

#[test]
fn a_report_without_the_block_reads_back_unmeasured() {
    let mut params = small_profile(GOAL_WORLD_SET);
    params.seeds = goal_recipe_seeds();
    let (report, _) = run_deterministic(&params).expect("a valid profile");
    let mut case = serde_json::to_value(&report.goal_indicators.cases[0]).unwrap();
    case.as_object_mut().unwrap().remove("input_use");
    let historical: crate::bench::schema::GoalCaseObservation =
        serde_json::from_value(case).unwrap();
    assert!(historical.input_use.defined().is_none());
    let encoded = serde_json::to_value(&report.goal_indicators.cases[0].input_use).unwrap();
    let decoded: Indicator<InputUse> = serde_json::from_value(encoded).unwrap();
    assert_eq!(decoded, report.goal_indicators.cases[0].input_use);
}

/// Times the block at production sizes on reduced goal worlds. Run with
/// `cargo test -p v3-cli --release --lib input_use_cost_probe -- --ignored --nocapture`.
#[test]
#[ignore = "cost probe"]
fn input_use_cost_probe() {
    let mut params = crate::bench::profiles::goal_profile_params();
    params.width = 320;
    params.height = 320;
    params.founders = 1_000;
    params.ticks = 600;
    let (report, timings) = run_deterministic(&params).expect("a valid profile");
    for (case, timing) in report
        .goal_indicators
        .cases
        .iter()
        .zip(&timings.input_use_wall_clock_ms_per_seed)
    {
        let block = case.input_use.defined().expect("defined");
        let rows: usize = block
            .cohorts
            .iter()
            .filter_map(Indicator::defined)
            .map(|cohort| cohort.rows.len())
            .sum();
        let nested = serde_json::json!({"deterministic": {"goal_indicators": {"cases": [{"input_use": block}]}}});
        let bytes = serde_json::to_vec_pretty(&nested).unwrap().len();
        println!(
            "{}: input_use {:.0} ms, rows {rows}, {bytes} bytes",
            case.case.name, timing.wall_clock_ms
        );
    }
    println!(
        "mutation_effects {:?}",
        timings.mutation_effects_wall_clock_ms_per_seed
    );
}

#[test]
fn row_strings_round_trip_and_reject_malformed_text() {
    let row = RowBlock {
        family: "ActionQueue".to_string(),
        channel: 14,
        beyond_draw_width: true,
        counts: vec![
            Some(3),
            Some(1),
            Some(1),
            Some(0),
            Some(0),
            Some(0),
            Some(0),
            Some(0),
            None,
        ],
    };
    assert_eq!(row.encode(), "ActionQueue 14* 3 1 1 0 0 0 0 0 -");
    assert_eq!(RowBlock::decode(&row.encode()), Some(row));
    assert_eq!(RowBlock::decode("SharedMemory 3 - 1"), None);
    assert_eq!(RowBlock::decode("x y 1 1 1 1 1 1 1 1 1"), None);
}

#[test]
fn retained_share_is_undefined_without_a_retention_pair() {
    let reading = CohortUse {
        cohort: effects::Cohort::Drift,
        parents_requested: 0,
        parents_evaluated: 0,
        consistency_violations: 0,
        out_of_width_consumers: Default::default(),
        retention_parents: 0,
        retention_children_requested: 0,
        retention_children_sampled: 0,
        rows: Vec::new(),
    };
    assert!(cohort_block(&reading).retained_share.defined().is_none());
}
