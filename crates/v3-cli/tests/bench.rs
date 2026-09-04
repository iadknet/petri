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

use std::path::PathBuf;

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
        name: "synthetic".to_string(),
        width: 8,
        height: 8,
        founders: 4,
        seeds: vec![1],
        ticks: 3,
        food_coverage: 1.0,
    };
    bench::build_report(&params, "t10-f10-synthetic-check")
}

/// The gate profile's `deterministic` block is byte-identical across two
/// independent runs at the same commit and inputs.
#[test]
fn gate_profile_deterministic_block_is_byte_identical_across_two_runs() {
    let params = bench::gate_profile_params();

    let report_a = bench::build_report(&params, "t10-f10-determinism-check");
    let report_b = bench::build_report(&params, "t10-f10-determinism-check");

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

    let reference_paths = bench::default_gate_references(&series_path)
        .expect("series index must parse and reference existing reports");
    assert!(
        !reference_paths.is_empty(),
        "series index must declare at least one reference report"
    );

    let resolved_paths: Vec<PathBuf> = reference_paths
        .iter()
        .map(|p| repo_root().join(p))
        .collect();

    let params = bench::gate_profile_params();
    let mut report = bench::build_report(&params, "t10-f10-regression-check");

    let severe = bench::apply_comparisons(&mut report, &resolved_paths)
        .expect("every declared reference report must exist and parse");

    for reference in &report.comparison.references {
        for counter in &reference.counters {
            assert_ne!(
                counter.level,
                "severe",
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

    // Reference had far fewer vm_steps per creature-tick: current regresses
    // well past the +50% severe threshold.
    reference.deterministic.per_creature_tick.vm_steps = Some("1.000000".to_string());

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
    assert_eq!(vm_steps.level, "severe");
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
    assert_eq!(vm_steps.level, "new");
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
    assert_eq!(wall_clock.level, "severe");
    assert!(
        !comparison.severe,
        "wall-clock must never mark the reference comparison severe, even when severe itself"
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
    assert_eq!(plasticity_updates.level, "severe");
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
