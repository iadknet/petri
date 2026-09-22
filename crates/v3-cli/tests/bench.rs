//! Fast benchmark-harness tests (T10.F10), run by `make rust-test-cli` and
//! therefore `make check`.
//!
//! The two tests required by the spec (`gate_profile_deterministic_block_is_
//! byte_identical_across_two_runs` and `gate_profile_has_no_severe_
//! regression_against_series_references`) run the gate profile in-process
//! (no subprocess, no release build) and are bounded by the gate profile's
//! timing budget: two full gate runs plus one comparison run must finish
//! well under 30 seconds combined in a debug build. The remaining tests
//! exercise `compare_against`'s regression logic directly against a tiny
//! synthetic report and add negligible time.

use std::num::NonZeroUsize;
use std::path::PathBuf;

use proptest::prelude::*;
use v3_cli::bench;

/// Repository root, derived from `CARGO_MANIFEST_DIR` (`crates/v3-cli`) so the
/// test does not depend on the working directory `cargo test` was invoked
/// from.
fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/ parent")
        .parent()
        .expect("repo root")
        .to_path_buf()
}

fn docs_progress_dir() -> PathBuf {
    repo_root().join("docs/progress")
}

/// A minimal, fast report for the synthetic comparison tests below, which
/// only need a well-formed `Report` shape to mutate — not gate-scale data.
fn tiny_report() -> bench::Report {
    let params = bench::ProfileParams {
        recipe: None,
        name: "synthetic".to_string(),
        width: 8,
        height: 8,
        founders: 4,
        seeds: vec![1],
        ticks: 3,
        food_coverage: Some(1.0),
        neighborhood: bench::NeighborhoodSizes::default(),
        drift: Default::default(),
        recruitment: v3_core::neighborhood::recruitment_paths::Sizes::TEST,
    };
    bench::build_report(&params, "t10-f10-synthetic-check").expect("a valid profile")
}

/// The tiny sweep profile used by the T01.F11 persistence-field tests: small
/// enough to run twice in a debug build, long enough to exercise the
/// accumulator against a real simulation at production food coverage.
fn tiny_sweep_params() -> bench::ProfileParams {
    bench::ProfileParams {
        recipe: None,
        name: "sweep".to_string(),
        width: 32,
        height: 32,
        founders: 8,
        seeds: vec![11],
        ticks: 30,
        food_coverage: None,
        neighborhood: bench::NeighborhoodSizes::default(),
        drift: Default::default(),
        recruitment: v3_core::neighborhood::recruitment_paths::Sizes::TEST,
    }
}

/// Small mutational-neighborhood sizes (T11.F01), distinct from the
/// production `NeighborhoodSizes::PRODUCTION` reading, so this fixture
/// exercises the same goal-profile code path as `make bench PROFILE=goal`
/// without paying production cost in a debug-build test.
fn tiny_goal_params() -> bench::ProfileParams {
    bench::ProfileParams {
        recipe: None,
        name: "goal".to_string(),
        width: 32,
        height: 32,
        founders: 8,
        seeds: vec![11],
        ticks: 30,
        food_coverage: None,
        neighborhood: bench::NeighborhoodSizes::default(),
        drift: Default::default(),
        recruitment: v3_core::neighborhood::recruitment_paths::Sizes::TEST,
    }
}

/// The gate profile's `deterministic` block is byte-identical across two
/// independent runs at the same commit and inputs.
#[test]
fn gate_profile_deterministic_block_is_byte_identical_across_two_runs() {
    // Runs the gate profile at the small fixture `NeighborhoodSizes::default()`
    // rather than `NeighborhoodSizes::PRODUCTION` (T11.F01 spec, "Compute
    // limits"): a debug-build timing measurement at production sizes found
    // this test alone taking ~41s and the regression test below ~15s, against
    // a pre-feature combined total of ~30s for both, well past the 10-second
    // growth budget. In-process byte-identity does not depend on trial count.
    // `make bench` alone produces the production reading.
    let params = bench::ProfileParams {
        neighborhood: bench::NeighborhoodSizes::default(),
        drift: Default::default(),
        ..bench::gate_profile_params()
    };

    let report_a =
        bench::build_report(&params, "t10-f10-determinism-check").expect("a valid profile");
    let report_b =
        bench::build_report(&params, "t10-f10-determinism-check").expect("a valid profile");

    let block_a = bench::deterministic_block_json(&report_a);
    let block_b = bench::deterministic_block_json(&report_b);

    assert_eq!(
        block_a, block_b,
        "gate profile's deterministic block must be byte-identical across two runs"
    );
}

/// The gate profile, compared against the series' epoch baseline and last
/// closed report, never reports a severe work-counter regression at the
/// commit that produced those references. Wall-clock is recorded but must
/// never contribute to `severe`.
#[test]
fn gate_profile_has_no_severe_regression_against_series_references() {
    let series_path = docs_progress_dir().join("benchmark-series.json");
    assert!(
        series_path.exists(),
        "docs/progress/benchmark-series.json must exist before this test can run; \
         generate it with `make bench PROFILE=gate FEATURE=<slug>` first"
    );

    let selection = bench::default_gate_references(&series_path)
        .expect("series index must parse and reference existing reports");
    assert!(
        !selection.paths.is_empty(),
        "series index must declare at least one reference report"
    );

    let resolved = bench::ReferenceSelection {
        paths: selection
            .paths
            .iter()
            .map(|p| repo_root().join(p))
            .collect(),
        absence: None,
    };
    // An output path that is provably not a reference, so the self-reference
    // filter can never empty the selection here.
    let out_path = std::env::temp_dir().join(format!(
        "t10-f10-regression-check-{}.json",
        std::process::id()
    ));

    // Same reduced-fixture rationale as the byte-identity test above: the
    // work-counter and profile comparisons below do not depend on
    // neighborhood trial count, and the production reading only needs to
    // exist once, produced by `make bench`.
    let params = bench::ProfileParams {
        neighborhood: bench::NeighborhoodSizes::default(),
        drift: Default::default(),
        ..bench::gate_profile_params()
    };
    let mut report =
        bench::build_report(&params, "t10-f10-regression-check").expect("a valid profile");

    let severe = bench::apply_comparisons(&mut report, &resolved, &out_path)
        .expect("every declared reference report must exist and parse");

    assert_eq!(
        report.comparison.references.len(),
        resolved.paths.len(),
        "every series reference is compared; none is mistaken for the output path"
    );
    assert_eq!(report.comparison.reference_absence, None);
    for reference in &report.comparison.references {
        for counter in &reference.counters {
            assert_ne!(
                counter.level,
                bench::ComparisonLevel::Severe,
                "counter {} regressed severely against {}: current={} reference={:?} delta%={:?}",
                counter.name,
                reference.path,
                counter.current,
                counter.reference,
                counter.percent_delta
            );
        }
    }
    assert!(
        !severe,
        "gate profile must not report a severe work-counter regression against its own series references"
    );
}

/// A synthetic severe work-counter regression is detected and marks both the
/// per-counter level and the overall comparison `severe`.
#[test]
fn compare_against_flags_severe_work_counter_regression() {
    let current = tiny_report();
    let mut reference = current.clone();

    // Reference had far fewer mesh hops per creature-tick: current regresses
    // well past the +50% severe threshold.
    reference.deterministic.per_creature_tick.mesh_hops = Some("0.001000".to_string());

    let comparison = bench::compare_against(
        &current,
        std::path::Path::new("synthetic-ref.json"),
        &reference,
    );

    let mesh_hops = comparison
        .counters
        .iter()
        .find(|c| c.name == "mesh_hops")
        .expect("mesh_hops counter must be present");
    assert_eq!(mesh_hops.level, bench::ComparisonLevel::Severe);
    assert!(
        comparison.severe,
        "a severe counter must mark the reference comparison severe"
    );
}

/// A counter absent from the reference report (older schema) is labeled
/// `new`, not treated as a zero-value regression, and does not fail the
/// comparison.
#[test]
fn compare_against_labels_missing_reference_counter_as_new() {
    let current = tiny_report();
    let mut reference = current.clone();

    reference.deterministic.per_creature_tick.vm_steps = None;

    let comparison = bench::compare_against(
        &current,
        std::path::Path::new("synthetic-ref.json"),
        &reference,
    );

    let vm_steps = comparison
        .counters
        .iter()
        .find(|c| c.name == "vm_steps")
        .expect("vm_steps counter must be present");
    assert_eq!(vm_steps.level, bench::ComparisonLevel::New);
    assert!(vm_steps.reference.is_none());
    assert!(vm_steps.percent_delta.is_none());
    assert!(
        !comparison.severe,
        "a `new` counter must never mark the comparison severe"
    );
}

/// A severe wall-clock delta on a host-matching reference is reported on the
/// `wall_clock` field but never contributes to the overall `severe` flag —
/// only integer work counters may fail a comparison.
#[test]
fn compare_against_reports_severe_wall_clock_without_marking_comparison_severe() {
    let current = tiny_report();
    let mut reference = current.clone();

    // Same host identity, but the reference ran far faster per creature-tick.
    reference.environment.wall_clock_ms_per_creature_tick =
        current.environment.wall_clock_ms_per_creature_tick / 10.0;

    let comparison = bench::compare_against(
        &current,
        std::path::Path::new("synthetic-ref.json"),
        &reference,
    );

    let wall_clock = comparison
        .wall_clock
        .expect("host identity matches, so wall_clock must be populated");
    assert_eq!(wall_clock.level, bench::ComparisonLevel::Severe);
    assert!(
        !comparison.severe,
        "wall-clock must never mark the reference comparison severe, even when severe itself"
    );
}

/// The wall-clock thresholds are strict: a synthetic delta of exactly +25
/// percent is `ok` and exactly +100 percent is `flag`, not the next level up.
/// The values are assigned, not timed, so no timing is asserted.
#[test]
fn wall_clock_levels_are_strict_at_their_thresholds() {
    // One synthetic report, cloned per case: the comparison reads only the
    // two wall-clock fields the case assigns.
    let base = tiny_report();
    let level_for = |current_ms: f64, reference_ms: f64| {
        let mut current = base.clone();
        let mut reference = base.clone();
        current.environment.wall_clock_ms_per_creature_tick = current_ms;
        reference.environment.wall_clock_ms_per_creature_tick = reference_ms;
        let comparison = bench::compare_against(
            &current,
            std::path::Path::new("synthetic-ref.json"),
            &reference,
        );
        let wall_clock = comparison
            .wall_clock
            .expect("host identity matches, so wall_clock must be populated");
        assert!(
            !comparison.severe,
            "wall-clock never marks a comparison severe"
        );
        (wall_clock.level, wall_clock.percent_delta)
    };

    assert_eq!(level_for(1.0, 1.0), (bench::ComparisonLevel::Ok, 0.0));
    assert_eq!(level_for(0.5, 1.0), (bench::ComparisonLevel::Ok, -50.0));
    assert_eq!(
        level_for(1.25, 1.0),
        (bench::ComparisonLevel::Ok, 25.0),
        "exactly +25 percent is inside the flag threshold"
    );
    assert_eq!(level_for(1.5, 1.0), (bench::ComparisonLevel::Flag, 50.0));
    assert_eq!(
        level_for(2.0, 1.0),
        (bench::ComparisonLevel::Flag, 100.0),
        "exactly +100 percent is inside the severe threshold"
    );
    assert_eq!(level_for(3.0, 1.0), (bench::ComparisonLevel::Severe, 200.0));
}

/// The `bench` subcommand is wired end to end: it writes the report the
/// `--out` path names on the thread count `--threads` asked for, and it
/// rejects `--threads 0` before running anything.
#[test]
fn bench_subcommand_writes_a_report_and_rejects_zero_threads() {
    let sweep_args = |threads: &str, out: &std::path::Path| {
        let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_v3-cli"));
        command.args([
            "bench",
            "--profile",
            "sweep",
            "--width",
            "16",
            "--height",
            "16",
            "--founders",
            "4",
            "--seeds",
            "11",
            "--ticks",
            "5",
            "--threads",
            threads,
            "--feature",
            "t10-f09-bench-cli-check",
            "--out",
        ]);
        command
            .arg(out)
            .arg("--summary-out")
            .arg(out.with_extension("summary.json"));
        command.output().expect("the v3-cli binary must run")
    };

    let out = std::env::temp_dir().join(format!("t10-f09-bench-cli-{}.json", std::process::id()));
    let _ = std::fs::remove_file(&out);

    let accepted = sweep_args("1", &out);
    assert!(
        accepted.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&accepted.stderr)
    );
    let written = std::fs::read_to_string(&out).expect("--out must name a written report");
    let _ = std::fs::remove_file(&out);
    let _ = std::fs::remove_file(out.with_extension("summary.json"));
    let report: bench::Report = serde_json::from_str(&written).expect("the report must parse");
    assert_eq!(report.environment.threads, Some(1));
    assert_eq!(report.deterministic.profile.ticks, 5);
    assert_eq!(report.feature, "t10-f09-bench-cli-check");

    let rejected = sweep_args("0", &out);
    assert!(!rejected.status.success(), "--threads 0 must be rejected");
    assert!(
        String::from_utf8_lossy(&rejected.stderr).contains("--threads must be >= 1"),
        "stderr: {}",
        String::from_utf8_lossy(&rejected.stderr)
    );
    assert!(!out.exists(), "a rejected argument must not write a report");
}

/// Sweep output must be explicit, and an explicit sweep output must not select
/// goal references just because the goal series has a baseline. This keeps the
/// fixed goal profile separate from user-configured sweep runs.
#[test]
fn sweep_output_and_reference_selection_stay_separate_from_goal() {
    let base_args = [
        "bench",
        "--profile",
        "sweep",
        "--width",
        "16",
        "--height",
        "16",
        "--founders",
        "4",
        "--seeds",
        "11",
        "--ticks",
        "5",
        "--feature",
        "t01-f12-sweep-output-check",
    ];

    let missing_out_dir =
        std::env::temp_dir().join(format!("t01-f12-sweep-missing-out-{}", std::process::id()));
    std::fs::create_dir_all(&missing_out_dir).expect("create isolated working directory");
    let rejected = std::process::Command::new(env!("CARGO_BIN_EXE_v3-cli"))
        .args(base_args)
        .current_dir(&missing_out_dir)
        .output()
        .expect("the v3-cli binary must run");
    let _ = std::fs::remove_dir_all(&missing_out_dir);
    assert!(
        !rejected.status.success(),
        "a sweep without --out must fail"
    );
    assert!(
        String::from_utf8_lossy(&rejected.stderr).contains("usable Git checkout required"),
        "stderr: {}",
        String::from_utf8_lossy(&rejected.stderr)
    );

    let out = std::env::temp_dir().join(format!(
        "t01-f12-sweep-reference-selection-{}.json",
        std::process::id()
    ));
    let accepted = std::process::Command::new(env!("CARGO_BIN_EXE_v3-cli"))
        .args(base_args)
        .args(["--out", out.to_str().expect("UTF-8 output path")])
        .arg("--summary-out")
        .arg(out.with_extension("summary.json"))
        .current_dir(repo_root())
        .output()
        .expect("the v3-cli binary must run");
    assert!(
        accepted.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&accepted.stderr)
    );
    let report: bench::Report = serde_json::from_str(
        &std::fs::read_to_string(&out).expect("the explicit sweep output must be written"),
    )
    .expect("the sweep report must parse");
    let _ = std::fs::remove_file(&out);
    let _ = std::fs::remove_file(out.with_extension("summary.json"));
    assert!(
        report.comparison.references.is_empty(),
        "a sweep without --baseline/--compare must not auto-select goal references"
    );
    assert_eq!(
        report.comparison.reference_absence.as_deref(),
        Some("no reference paths were given")
    );
    let stdout = String::from_utf8_lossy(&accepted.stdout);
    assert!(
        stdout.contains("no comparison reference: no reference paths were given"),
        "the absence cause is printed after the report is written; stdout: {stdout}"
    );
}

/// A counter that was exactly zero in the reference but positive in the
/// current run has an unbounded (division-by-zero) ratio. It must be
/// reported as `severe`, not silently `ok` with a null delta — otherwise the
/// first feature to introduce nonzero work for that counter (e.g. the first
/// plastic founder genome) could never trip a regression.
#[test]
fn compare_against_treats_reference_zero_current_positive_as_severe() {
    let mut current = tiny_report();
    let mut reference = current.clone();

    // The gate founder genome has no plastic CGP nodes, so plasticity_updates
    // is 0 in both reports by construction; force the current side nonzero
    // to exercise the reference==0/current>0 (division-by-zero) path
    // directly, independent of genome data.
    current.deterministic.per_creature_tick.plasticity_updates = Some("0.500000".to_string());
    reference.deterministic.per_creature_tick.plasticity_updates = Some("0.000000".to_string());

    let comparison = bench::compare_against(
        &current,
        std::path::Path::new("synthetic-ref.json"),
        &reference,
    );

    let plasticity_updates = comparison
        .counters
        .iter()
        .find(|c| c.name == "plasticity_updates")
        .expect("plasticity_updates counter must be present");
    assert_eq!(plasticity_updates.level, bench::ComparisonLevel::Severe);
    assert!(comparison.severe);
}

/// `compare_against_path` hard-fails when the reference report was generated
/// with a different profile (world size, founders, seeds, ticks, or food
/// coverage): a work-counter comparison across different profiles is
/// meaningless, and this is the live failure mode since the gate horizon
/// already changed once during this feature's own implementation.
#[test]
fn compare_against_path_errors_on_profile_mismatch() {
    let current = tiny_report();

    let mut reference = current.clone();
    reference.deterministic.profile.ticks += 1;

    let scratch_path = std::env::temp_dir().join(format!(
        "t10-f10-bench-profile-mismatch-{}.json",
        std::process::id()
    ));
    std::fs::write(&scratch_path, bench::report_json_pretty(&reference))
        .expect("failed to write scratch reference report");

    let result = bench::compare_against_path(&current, &scratch_path);
    let _ = std::fs::remove_file(&scratch_path);

    let err = result.expect_err("a profile mismatch must be a hard error, not a silent skip");
    assert!(
        err.contains("different profile"),
        "error message should explain the profile mismatch, got: {err}"
    );
}

/// A sweep at production food coverage populates every T01.F11 persistence
/// field, and the whole `deterministic` block — samples included — is
/// byte-identical across two runs.
#[test]
fn tiny_sweep_records_persistence_fields_identically_across_two_runs() {
    let params = tiny_sweep_params();

    let report_a =
        bench::build_report(&params, "t01-f11-persistence-check").expect("a valid profile");
    let report_b =
        bench::build_report(&params, "t01-f11-persistence-check").expect("a valid profile");

    assert_eq!(
        report_a.deterministic.profile.food_coverage, "default",
        "a sweep without --food-coverage records production coverage as `default`"
    );

    let seeds = &report_a
        .deterministic
        .goal_indicators
        .population_persistence
        .per_seed;
    assert_eq!(seeds.len(), 1);
    let seed = &seeds[0];
    assert!(
        seed.peak_population >= params.founders.into(),
        "the peak must include the founder population, got {}",
        seed.peak_population
    );
    assert!(seed.peak_tick <= params.ticks);
    let last = seed
        .samples
        .last()
        .expect("the last executed tick is always sampled");
    assert_eq!(
        last.tick,
        seed.extinction_tick.unwrap_or(params.ticks),
        "the final sample is the last executed tick"
    );
    assert_eq!(last.population, seed.final_population);
    assert_eq!(last.mean_energy, seed.mean_energy);
    assert_eq!(
        seed.mean_energy.is_none(),
        seed.final_population == 0,
        "mean energy is null exactly when the population is zero"
    );
    assert_eq!(
        seed.plateau_population.is_none(),
        seed.extinction_tick
            .is_some_and(|tick| tick * 4 <= params.ticks * 3),
        "the plateau is null exactly when the run ended before its window"
    );

    assert_eq!(
        bench::deterministic_block_json(&report_a),
        bench::deterministic_block_json(&report_b),
        "the persistence fields must be byte-identical across two runs"
    );
}

/// Goal observations use the same read-only path at tiny scale in tests. Their
/// deterministic block is stable across independent runs and thread counts,
/// while the new probe timing stays in the environment block.
#[test]
fn tiny_goal_profile_observations_are_deterministic_and_goal_only() {
    let params = tiny_goal_params();
    let one_thread =
        bench::build_report_with_threads(&params, "t01-f12-tiny-goal-check", NonZeroUsize::new(1))
            .expect("a valid profile");
    let default_pool =
        bench::build_report(&params, "t01-f12-tiny-goal-check").expect("a valid profile");

    assert_eq!(
        bench::deterministic_block_json(&one_thread),
        bench::deterministic_block_json(&default_pool),
        "goal readings must not depend on the cognition pool"
    );
    assert!(matches!(
        one_thread.deterministic.goal_indicators.lineage_diversity,
        bench::Indicator::Defined(_)
    ));
    assert!(matches!(
        one_thread.deterministic.goal_indicators.memory_sensitivity,
        bench::Indicator::Defined(_)
    ));
    match &one_thread
        .deterministic
        .goal_indicators
        .mutational_neighborhood
    {
        bench::Indicator::Defined(neighborhood) => assert!(
            matches!(neighborhood.evolved, bench::Indicator::Defined(_)),
            "the evolved half must be defined in the goal profile"
        ),
        bench::Indicator::Undefined(_) => {
            panic!("mutational_neighborhood must be defined in the goal profile")
        }
    }
    assert_eq!(
        one_thread
            .environment
            .final_state_observation_ms_per_seed
            .len(),
        params.seeds.len()
    );
    // Integration tests link the non-test library build, so the reduced
    // experiment must come from the params, not from `cfg(test)`.
    let recruitment = one_thread
        .deterministic
        .goal_indicators
        .recruitment_paths
        .defined()
        .expect("recruitment paths are measured in the goal profile");
    assert_eq!(
        recruitment.sizes,
        v3_core::neighborhood::recruitment_paths::Sizes::TEST,
        "the tiny goal fixture must run the reduced recruitment-paths experiment"
    );

    let sweep = bench::build_report(&tiny_sweep_params(), "t01-f12-sweep-undefined-check")
        .expect("a valid profile");
    assert!(matches!(
        sweep.deterministic.goal_indicators.lineage_diversity,
        bench::Indicator::Undefined(ref value) if value == "Undefined"
    ));
    assert!(matches!(
        sweep.deterministic.goal_indicators.memory_sensitivity,
        bench::Indicator::Undefined(ref value) if value == "Undefined"
    ));
    assert!(matches!(
        sweep.deterministic.goal_indicators.mutational_neighborhood,
        bench::Indicator::Undefined(ref value) if value == "Undefined"
    ));
    assert!(sweep
        .environment
        .final_state_observation_ms_per_seed
        .is_empty());
}

#[test]
fn gate_and_goal_series_select_only_their_own_references() {
    let scratch_dir =
        std::env::temp_dir().join(format!("t01-f12-series-index-{}", std::process::id()));
    std::fs::create_dir_all(&scratch_dir).expect("create scratch directory");
    let gate_epoch = scratch_dir.join("gate-epoch.json");
    let gate_latest = scratch_dir.join("gate-latest.json");
    let goal_epoch = scratch_dir.join("goal-epoch.json");
    for path in [&gate_epoch, &gate_latest, &goal_epoch] {
        std::fs::write(path, "{}").expect("create reference placeholder");
    }
    let index = bench::BenchmarkSeriesIndex {
        goal_worlds: Some(bench::SeriesIndex {
            series: "goal-worlds-v1".to_string(),
            epoch_baseline: goal_epoch.display().to_string(),
            closed: vec![],
        }),
        gate: bench::SeriesIndex {
            series: "gate-v1".to_string(),
            epoch_baseline: gate_epoch.display().to_string(),
            closed: vec![
                gate_epoch.display().to_string(),
                gate_latest.display().to_string(),
            ],
        },
        goal: bench::SeriesIndex {
            series: "goal-v1".to_string(),
            epoch_baseline: "historical-goal-reference-must-not-be-selected".to_string(),
            closed: vec![],
        },
    };
    let index_path = scratch_dir.join("benchmark-series.json");
    std::fs::write(
        &index_path,
        serde_json::to_string(&index).expect("serialize series index"),
    )
    .expect("write series index");

    assert_eq!(
        bench::default_gate_references(&index_path).expect("read gate references"),
        bench::ReferenceSelection {
            paths: vec![gate_epoch.clone(), gate_latest],
            absence: None,
        }
    );
    assert_eq!(
        bench::default_goal_references(&index_path).expect("read goal references"),
        bench::ReferenceSelection {
            paths: vec![goal_epoch],
            absence: None,
        }
    );
    std::fs::remove_dir_all(&scratch_dir).expect("remove scratch directory");
}

/// Each way the series index can yield no reference names its own cause, and
/// the cause travels through `apply_comparisons` into the report instead of a
/// silent empty comparison. The third cause — the index naming the output path
/// itself — is the T12.F04 defect.
#[test]
fn series_index_absence_causes_are_each_recorded() {
    let scratch_dir =
        std::env::temp_dir().join(format!("t14-f01-series-absence-{}", std::process::id()));
    std::fs::create_dir_all(&scratch_dir).expect("create scratch directory");
    let out_path = scratch_dir.join("this-report-goal.json");
    let mut report = tiny_report();

    // 1. No series index at all.
    let missing_index = scratch_dir.join("benchmark-series.json");
    let expected_missing = format!("no series index at {}", missing_index.display());
    let gate =
        bench::default_gate_references(&missing_index).expect("absent index is not an error");
    assert_eq!(
        gate,
        bench::ReferenceSelection {
            paths: vec![],
            absence: Some(expected_missing.clone()),
        }
    );
    let goal =
        bench::default_goal_references(&missing_index).expect("absent index is not an error");
    assert_eq!(goal.absence.as_deref(), Some(expected_missing.as_str()));
    bench::apply_comparisons(&mut report, &gate, &out_path).expect("nothing to compare");
    assert!(report.comparison.references.is_empty());
    assert_eq!(
        report.comparison.reference_absence.as_deref(),
        Some(expected_missing.as_str())
    );
    assert!(!report.comparison.severe);

    // 2. The goal-worlds series has no stored epoch baseline yet: either the
    //    series is missing from the index or its baseline file does not exist.
    let write_index = |goal_worlds: Option<bench::SeriesIndex>| {
        let index = bench::BenchmarkSeriesIndex {
            gate: bench::SeriesIndex {
                series: "gate-v1".to_string(),
                epoch_baseline: "unused".to_string(),
                closed: vec![],
            },
            goal: bench::SeriesIndex {
                series: "goal-v1".to_string(),
                epoch_baseline: "unused".to_string(),
                closed: vec![],
            },
            goal_worlds,
        };
        std::fs::write(
            &missing_index,
            serde_json::to_string(&index).expect("serialize series index"),
        )
        .expect("write series index");
    };
    let expected_no_baseline = "series goal-worlds-v1 has no stored epoch baseline yet";
    write_index(None);
    let goal = bench::default_goal_references(&missing_index).expect("index parses");
    assert_eq!(goal.paths, Vec::<PathBuf>::new());
    assert_eq!(goal.absence.as_deref(), Some(expected_no_baseline));
    write_index(Some(bench::SeriesIndex {
        series: "goal-worlds-v1".to_string(),
        epoch_baseline: scratch_dir.join("never-written.json").display().to_string(),
        closed: vec![],
    }));
    let goal = bench::default_goal_references(&missing_index).expect("index parses");
    assert_eq!(goal.paths, Vec::<PathBuf>::new());
    assert_eq!(goal.absence.as_deref(), Some(expected_no_baseline));
    bench::apply_comparisons(&mut report, &goal, &out_path).expect("nothing to compare");
    assert!(report.comparison.references.is_empty());
    assert_eq!(
        report.comparison.reference_absence.as_deref(),
        Some(expected_no_baseline)
    );

    // 3. The index names this report's own output path (written by an earlier
    //    run), so the selection carries it and the comparison drops it.
    std::fs::write(&out_path, bench::report_json_pretty(&report)).expect("write earlier report");
    write_index(Some(bench::SeriesIndex {
        series: "goal-worlds-v1".to_string(),
        epoch_baseline: out_path.display().to_string(),
        closed: vec![out_path.display().to_string()],
    }));
    let goal = bench::default_goal_references(&missing_index).expect("index parses");
    assert_eq!(
        goal,
        bench::ReferenceSelection {
            paths: vec![out_path.clone()],
            absence: None,
        }
    );
    let severe = bench::apply_comparisons(&mut report, &goal, &out_path)
        .expect("a skipped self-reference is not an error");
    assert!(!severe);
    assert!(report.comparison.references.is_empty());
    assert_eq!(
        report.comparison.reference_absence.as_deref(),
        Some(
            format!(
                "the only candidate reference is this report's own output path {}",
                out_path.display()
            )
            .as_str()
        )
    );

    std::fs::remove_dir_all(&scratch_dir).expect("remove scratch directory");
}

/// A profile that leaves production food coverage in place round-trips its
/// `default` coverage string through the profile-mismatch comparison: it
/// matches itself and is rejected against an otherwise identical profile that
/// forced a numeric coverage.
#[test]
fn default_food_coverage_round_trips_through_the_profile_comparison() {
    let current = bench::build_report(&tiny_sweep_params(), "t01-f11-default-coverage-check")
        .expect("a valid profile");
    let forced = bench::build_report(
        &bench::ProfileParams {
            food_coverage: Some(1.0),
            ..tiny_sweep_params()
        },
        "t01-f11-forced-coverage-check",
    )
    .expect("a valid profile");

    let scratch_path = std::env::temp_dir().join(format!(
        "t01-f11-bench-default-coverage-{}.json",
        std::process::id()
    ));

    std::fs::write(&scratch_path, bench::report_json_pretty(&current))
        .expect("failed to write scratch reference report");
    let same = bench::compare_against_path(&current, &scratch_path);

    std::fs::write(&scratch_path, bench::report_json_pretty(&forced))
        .expect("failed to write scratch reference report");
    let different = bench::compare_against_path(&current, &scratch_path);

    let _ = std::fs::remove_file(&scratch_path);

    same.expect("a `default` coverage profile must match itself after a JSON round-trip");
    let err = different.expect_err("`default` and `1.000000` are different profiles");
    assert!(
        err.contains("different profile"),
        "error message should explain the profile mismatch, got: {err}"
    );
}

/// The thread count changes only the `environment` block (T10.F09): the same
/// sweep run on a one-thread pool and on the rayon global pool produces a
/// byte-identical `deterministic` block, and each report records the thread
/// count of the pool that actually ran it.
#[test]
fn thread_count_changes_the_environment_but_not_the_deterministic_block() {
    let params = tiny_sweep_params();
    let global_threads = rayon::current_num_threads();

    let one_thread = bench::build_report_with_threads(
        &params,
        "t10-f09-thread-independence-check",
        NonZeroUsize::new(1),
    )
    .expect("a valid profile");
    let default_pool =
        bench::build_report(&params, "t10-f09-thread-independence-check").expect("a valid profile");

    assert_eq!(
        bench::deterministic_block_json(&one_thread),
        bench::deterministic_block_json(&default_pool),
        "the thread count must never change simulation results"
    );
    assert_eq!(one_thread.environment.threads, Some(1));
    assert_eq!(
        default_pool.environment.threads,
        Some(global_threads),
        "without --threads the report records the global pool's thread count"
    );

    for report in [&one_thread, &default_pool] {
        let phases = &report.environment.phase_wall_clock_ms_per_seed;
        assert_eq!(phases.len(), params.seeds.len());
        assert_eq!(phases[0].seed, params.seeds[0]);
        let phase_total = phases[0].world_update_ms
            + phases[0].sensor_assembly_ms
            + phases[0].cognition_ms
            + phases[0].actions_ms
            + phases[0].reward_learning_ms;
        assert!(
            phase_total > 0.0 && phase_total <= report.environment.wall_clock_ms_total,
            "the timed phases must be a positive part of the run's wall-clock: \
             {phase_total} vs {}",
            report.environment.wall_clock_ms_total
        );
        assert_eq!(
            report.environment.throughput.per_seed.len(),
            params.seeds.len()
        );
        assert!(report.environment.throughput.total.ticks_per_second > 0.0);
    }
}

proptest! {
    /// Every rate multiplied by the elapsed time it was derived from recovers
    /// its own counter. Relative tolerance, because a u64 counter beyond 2^53
    /// is not exactly representable as an `f64`.
    #[test]
    fn throughput_rates_recover_their_counters(
        ticks in 0_u64..1_000_000_000,
        creature_ticks in 0_u64..1_000_000_000_000,
        births in 0_u64..1_000_000_000,
        wall_clock_ms in 0.001_f64..1_000_000_000.0,
    ) {
        let rates = bench::throughput_rates(ticks, creature_ticks, births, wall_clock_ms);
        let seconds = wall_clock_ms / 1000.0;
        let hours = seconds / 3600.0;

        let close = |recovered: f64, counter: u64| {
            let counter = counter as f64;
            (recovered - counter).abs() <= 1e-6 * counter.max(1.0)
        };

        prop_assert!(close(rates.ticks_per_second * seconds, ticks));
        prop_assert!(close(rates.creature_ticks_per_second * seconds, creature_ticks));
        prop_assert!(close(rates.ticks_per_hour * hours, ticks));
        prop_assert!(close(rates.births_per_hour * hours, births));
    }

    /// A run that recorded no elapsed time reports zero rates rather than
    /// infinity: a report must always serialize, and `Infinity` is not JSON.
    #[test]
    fn a_zero_wall_clock_yields_zero_rates(
        ticks in 0_u64..1_000_000_000,
        creature_ticks in 0_u64..1_000_000_000_000,
        births in 0_u64..1_000_000_000,
    ) {
        let rates = bench::throughput_rates(ticks, creature_ticks, births, 0.0);

        prop_assert_eq!(rates, bench::ThroughputRates::default());
        prop_assert!(serde_json::to_string(&rates).is_ok());
    }
}

/// `world inspect` resolves a saved recipe, prints its readings, and writes the
/// preview where `--png` asks for it. A recipe it cannot read is a validation
/// error that exits 1 and writes nothing.
#[test]
fn world_inspect_prints_readings_writes_a_preview_and_rejects_a_missing_recipe() {
    let dir = std::env::temp_dir().join(format!("t12-f04-inspect-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create the isolated working directory");
    let recipe = dir.join("small.json");
    std::fs::write(
        &recipe,
        r#"{"world": {"width": 24, "height": 16, "world_seed": 5},
            "population": {"initial_creatures": 3}}"#,
    )
    .expect("write the recipe");
    let png = dir.join("small.png");

    let inspect = |args: &[&std::path::Path]| {
        let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_v3-cli"));
        command.args(["world", "inspect", "--seed", "11", "--config"]);
        command.args(args);
        command.output().expect("the v3-cli binary must run")
    };

    let accepted = inspect(&[&recipe, std::path::Path::new("--png"), &png]);
    assert!(
        accepted.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&accepted.stderr)
    );
    let reading: serde_json::Value =
        serde_json::from_slice(&accepted.stdout).expect("stdout must be one JSON object");
    assert_eq!(reading["seed"], 11);
    assert_eq!(reading["world_seed"], 5);
    assert_eq!(reading["world_width"], 24);
    assert_eq!(reading["world_height"], 16);
    assert_eq!(reading["founders_placed"], 3);
    assert_eq!(
        reading["passable_connectivity"]["total_cells"],
        24 * 16,
        "the connectivity block is the world's own reading"
    );
    assert!(reading["food_types"][0]["food_cells"].as_u64().unwrap() > 0);
    let preview = std::fs::read(&png).expect("--png must write the preview");
    assert_eq!(
        &preview[..8],
        b"\x89PNG\r\n\x1a\n",
        "the preview must be a PNG"
    );

    let missing = dir.join("absent.json");
    let rejected = inspect(&[&missing]);
    assert_eq!(
        rejected.status.code(),
        Some(1),
        "a missing recipe is a validation error"
    );
    assert!(
        String::from_utf8_lossy(&rejected.stderr).contains("failed to read config"),
        "stderr: {}",
        String::from_utf8_lossy(&rejected.stderr)
    );
    assert!(
        rejected.stdout.is_empty(),
        "a rejected recipe prints nothing"
    );

    let unparseable = dir.join("bad.json");
    std::fs::write(&unparseable, "{ not json").expect("write the broken recipe");
    let rejected = inspect(&[&unparseable]);
    assert_eq!(rejected.status.code(), Some(1));
    assert!(
        String::from_utf8_lossy(&rejected.stderr).contains("invalid config"),
        "stderr: {}",
        String::from_utf8_lossy(&rejected.stderr)
    );

    // A seed that is not a number is rejected by argument parsing, before any
    // world is built. clap exits 2 for a usage error rather than 1.
    let bad_seed = std::process::Command::new(env!("CARGO_BIN_EXE_v3-cli"))
        .args(["world", "inspect", "--seed", "later", "--config"])
        .arg(&recipe)
        .output()
        .expect("the v3-cli binary must run");
    assert_eq!(bad_seed.status.code(), Some(2), "clap usage errors exit 2");
    assert!(bad_seed.stdout.is_empty());

    let _ = std::fs::remove_dir_all(&dir);
}

/// An explicit `--compare` path is the reference even for a profile whose
/// default selection is empty: reference selection reads the explicit paths
/// first and only falls back to the profile default when none were given.
#[test]
fn an_explicit_compare_path_is_preferred_over_the_profile_default_selection() {
    let sweep = |out: &std::path::Path, compare: Option<&std::path::Path>| {
        let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_v3-cli"));
        command
            .args([
                "bench",
                "--profile",
                "sweep",
                "--width",
                "16",
                "--height",
                "16",
                "--founders",
                "4",
                "--seeds",
                "11",
                "--ticks",
                "5",
                "--feature",
                "t14-f01-explicit-compare-check",
                "--out",
            ])
            .arg(out)
            .arg("--summary-out")
            .arg(out.with_extension("summary.json"));
        if let Some(compare) = compare {
            command.arg("--compare").arg(compare);
        }
        command
            .current_dir(repo_root())
            .output()
            .expect("the v3-cli binary must run")
    };

    let reference_out = std::env::temp_dir().join(format!(
        "t14-f01-explicit-compare-reference-{}.json",
        std::process::id()
    ));
    let current_out = std::env::temp_dir().join(format!(
        "t14-f01-explicit-compare-current-{}.json",
        std::process::id()
    ));
    let reference_run = sweep(&reference_out, None);
    assert!(
        reference_run.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&reference_run.stderr)
    );

    let compared = sweep(&current_out, Some(&reference_out));
    assert!(
        compared.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&compared.stderr)
    );
    let report: bench::Report = serde_json::from_str(
        &std::fs::read_to_string(&current_out).expect("the compared run must be written"),
    )
    .expect("the sweep report must parse");
    let _ = std::fs::remove_file(&reference_out);
    let _ = std::fs::remove_file(&current_out);
    let _ = std::fs::remove_file(reference_out.with_extension("summary.json"));
    let _ = std::fs::remove_file(current_out.with_extension("summary.json"));

    assert_eq!(
        report
            .comparison
            .references
            .iter()
            .map(|reference| reference.path.as_str())
            .collect::<Vec<_>>(),
        vec![reference_out.to_str().expect("UTF-8 reference path")],
        "the explicit --compare path must be the only reference"
    );
    assert_eq!(
        report.comparison.reference_absence, None,
        "an explicit reference leaves no absence cause"
    );
    let stdout = String::from_utf8_lossy(&compared.stdout);
    assert!(
        !stdout.contains("no comparison reference:"),
        "an explicit reference prints no absence line; stdout: {stdout}"
    );
}
