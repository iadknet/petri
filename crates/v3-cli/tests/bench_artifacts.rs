//! Storage and conversion tests use tiny sweeps, never measured baselines.
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use proptest::prelude::*;
use serde_json::{json, Value};
use v3_cli::bench::{self, artifacts};

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "petri artifact test {} {}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}

fn synthetic_full_report() -> Vec<u8> {
    // Test-only schema fixture, not an ecological run. Literal decimal inputs
    // exercise historical JSON fidelity; omitted fields must stay unmeasured.
    include_bytes!("fixtures/synthetic-full-benchmark-v1.json").to_vec()
}

fn provenance() -> artifacts::ConversionProvenance {
    serde_json::from_value(json!({
        "verified_at": "2026-09-13T10:00:00Z",
        "converter": {"executable":"v3-cli", "arguments":["bench-summarize"],
                      "working_directory":"/tmp/converter"},
        "supplied_evidence": null
    }))
    .unwrap()
}

fn world_source() -> Value {
    let mut raw: Value = serde_json::from_slice(&synthetic_full_report()).unwrap();
    let seed = raw["deterministic"]["per_seed"][0]["seed"].clone();
    let case = json!({"name":"test world", "seed":seed, "recipe_path":"test.json", "config_digest":"abc", "food_type_count":1});
    raw["deterministic"]["profile"]["name"] = json!("goal-worlds-v1");
    raw["deterministic"]["profile"]["cases"] = json!([case.clone()]);
    raw["deterministic"]["goal_indicators"]["cases"] = json!([{
        "case":case, "mutational_neighborhood":"Undefined", "drift_depth":"Undefined"
    }]);
    raw
}

#[test]
fn conversion_is_deterministic_hashes_exact_bytes_and_keeps_historical_unknowns() {
    assert_eq!(
        artifacts::sha256(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    let dir = Temp::new();
    let path = dir.0.join("raw.json");
    let raw = synthetic_full_report();
    std::fs::write(&path, &raw).unwrap();
    let first = artifacts::summarize(&raw, &path, &provenance()).unwrap();
    let second = artifacts::summarize(&raw, &path, &provenance()).unwrap();
    assert_eq!(
        artifacts::summary_bytes(&first).unwrap(),
        artifacts::summary_bytes(&second).unwrap()
    );
    let source: Value = serde_json::from_slice(&raw).unwrap();
    let summary = serde_json::to_value(&first).unwrap();
    assert_eq!(summary["kind"], "petri-benchmark-summary");
    assert_eq!(summary["summary_version"], 1);
    assert_eq!(summary["environment"], source["environment"]);
    assert_eq!(summary["comparison"], source["comparison"]);
    assert_eq!(summary["raw"]["bytes"], raw.len());
    assert_eq!(summary["raw"]["sha256"], artifacts::sha256(&raw));
    assert_eq!(
        summary["raw"]["path"],
        path.canonicalize().unwrap().to_str().unwrap()
    );
    assert!(summary["measurement_evidence"]["command"].is_null());
    assert!(summary["measurement_evidence"]["cli_exit"].is_null());
    assert!(summary["measurement_evidence"]["thresholds"].is_null());
    assert!(summary["measurement_evidence"]["inheritance"].is_null());
    for claim in summary["claims"].as_array().unwrap() {
        assert_eq!(
            source.pointer(claim["pointer"].as_str().unwrap()).unwrap(),
            &claim["value"]
        );
        assert!(!claim["value"].is_object() && !claim["value"].is_array());
    }
    assert!(summary["claims"].as_array().unwrap().len() <= artifacts::MAX_CLAIMS);
    let mut differently_formatted = raw.clone();
    differently_formatted.push(b'\n');
    assert_ne!(
        artifacts::sha256(&raw),
        artifacts::sha256(&differently_formatted)
    );
}

#[test]
fn historical_absence_and_measured_zero_survive_projection() {
    let dir = Temp::new();
    let path = dir.0.join("raw.json");
    let mut raw: Value = serde_json::from_slice(&synthetic_full_report()).unwrap();
    raw["deterministic"]["goal_indicators"]["population_persistence"]["per_seed"][0]
        .as_object_mut()
        .unwrap()
        .remove("peak_population");
    raw["deterministic"]["goal_indicators"]["memory_sensitivity"] = json!("Undefined");
    let bytes = serde_json::to_vec(&raw).unwrap();
    std::fs::write(&path, &bytes).unwrap();
    let summary = artifacts::summarize(&bytes, &path, &provenance()).unwrap();
    assert!(
        summary.deterministic["goal_indicators"]["population_persistence"]["per_seed"][0]
            .get("peak_population")
            .is_none()
    );
    assert_eq!(
        summary.deterministic["goal_indicators"]["memory_sensitivity"],
        "Undefined"
    );
    assert_eq!(
        summary.deterministic["per_seed"],
        raw["deterministic"]["per_seed"]
    );
    assert_eq!(summary.deterministic["per_seed"][0]["final_population"], 0);
    assert_eq!(
        summary.deterministic["goal_indicators"]["population_persistence"]["per_seed"][0]
            ["final_population"],
        0
    );
    assert!(summary.environment.get("threads").is_none());
    assert!(
        summary.deterministic["goal_indicators"]["reachable_structure_size_distribution"]
            .get("version")
            .is_none()
    );
}

#[test]
fn retained_float_readings_preserve_historical_numeric_values() {
    let dir = Temp::new();
    let raw_path = dir.0.join("raw.json");
    let raw = synthetic_full_report();
    std::fs::write(&raw_path, &raw).unwrap();
    let summary = artifacts::summarize(&raw, &raw_path, &provenance()).unwrap();
    let json = artifacts::summary_bytes(&summary).unwrap();
    let roundtrip: artifacts::Summary = serde_json::from_slice(&json).unwrap();
    assert_eq!(
        roundtrip.deterministic["per_seed"][0]["tick_zero_connectivity"]
            ["largest_component_fraction_of_passable"]
            .as_f64(),
        Some(0.9805590711984373)
    );
    assert_eq!(
        roundtrip.deterministic["per_seed"][1]["tick_zero_connectivity"]
            ["largest_component_fraction_of_passable"]
            .as_f64(),
        Some(0.9953555281754939)
    );
}

#[test]
fn checked_in_goal_recipe_identities_are_unchanged_by_json_precision() {
    for (name, expected) in [
        (
            "orchards-in-grassland",
            "sha256:8141056a33bc30445fd29340fa40498372835e08459b7c04c347f9724b445423",
        ),
        (
            "canyon-country",
            "sha256:ee72c5532cb947fad7349a3a4d3c5a5b5bef2c501ebe5b299b5de12ad26bd99d",
        ),
        (
            "confluence",
            "sha256:25fb4d0baf34719c0f1e657c98b4e8a7932216510d69651c6fd58d4f6d318676",
        ),
    ] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../../experiments/worlds/{name}.json"));
        let recipe = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        let config =
            v3_cli::inspect::resolve_baseline_world(recipe, path.to_str().unwrap()).unwrap();
        let digest = v3_core::config::config_digest(&config);
        println!("{name}: {digest}");
        assert_eq!(digest, expected);
    }
}

#[test]
fn stored_summary_compares_without_raw_and_rejects_bad_versions() {
    let dir = Temp::new();
    let raw_path = dir.0.join("raw.json");
    let summary_path = dir.0.join("summary.json");
    let raw = synthetic_full_report();
    std::fs::write(&raw_path, &raw).unwrap();
    let current: bench::Report = serde_json::from_slice(&raw).unwrap();
    let expected = bench::compare_against_path(&current, &raw_path).unwrap();
    let summary = artifacts::summarize(&raw, &raw_path, &provenance()).unwrap();
    std::fs::write(&summary_path, artifacts::summary_bytes(&summary).unwrap()).unwrap();
    std::fs::remove_file(&raw_path).unwrap();
    let actual = bench::compare_against_path(&current, &summary_path).unwrap();
    let mut expected = serde_json::to_value(expected).unwrap();
    expected["path"] = json!(summary_path);
    assert_eq!(serde_json::to_value(actual).unwrap(), expected);
    let mut invalid = serde_json::to_value(summary).unwrap();
    invalid["summary_version"] = json!(999);
    std::fs::write(&summary_path, serde_json::to_vec(&invalid).unwrap()).unwrap();
    assert!(bench::compare_against_path(&current, &summary_path)
        .unwrap_err()
        .contains("summary version"));
    std::fs::write(&summary_path, b"{}").unwrap();
    assert!(bench::compare_against_path(&current, &summary_path).is_err());
}

#[test]
fn summary_rejects_inconsistent_measured_metadata() {
    let dir = Temp::new();
    let path = dir.0.join("raw.json");
    let raw = synthetic_full_report();
    std::fs::write(&path, &raw).unwrap();
    let report: bench::Report = serde_json::from_slice(&raw).unwrap();
    let original =
        serde_json::to_value(artifacts::summarize(&raw, &path, &provenance()).unwrap()).unwrap();
    let summary_path = dir.0.join("summary.json");
    for (pointer, replacement) in [
        ("/source_schema_version", json!(999)),
        ("/feature", json!("wrong feature")),
        ("/environment/git_revision", json!("wrong revision")),
        ("/environment/generated_at", json!("wrong time")),
        ("/environment/host/hostname", json!("wrong host")),
        ("/environment/wall_clock_ms_per_creature_tick", json!(123.0)),
        (
            "/deterministic/per_creature_tick/vm_steps",
            json!("999.000000"),
        ),
        (
            "/comparison_inputs/per_creature_tick/vm_steps",
            json!("not a reading"),
        ),
    ] {
        let mut invalid = original.clone();
        *invalid.pointer_mut(pointer).unwrap() = replacement;
        std::fs::write(&summary_path, serde_json::to_vec(&invalid).unwrap()).unwrap();
        assert!(
            bench::compare_against_path(&report, &summary_path).is_err(),
            "accepted inconsistent {pointer}"
        );
    }

    let mut invalid_reading = original;
    invalid_reading["comparison_inputs"]["case_readings"]["undeclared"] =
        json!([["reading", "not a number"]]);
    std::fs::write(&summary_path, serde_json::to_vec(&invalid_reading).unwrap()).unwrap();
    assert!(
        bench::compare_against_path(&report, &summary_path)
            .unwrap_err()
            .contains("invalid lossless comparison reading"),
        "every stored comparison reading is validated, including undeclared case keys"
    );
}

#[test]
fn raw_verification_rejects_changed_shorter_and_longer_files() {
    let dir = Temp::new();
    let path = dir.0.join("raw.json");
    let raw = synthetic_full_report();
    let mut changed = raw.clone();
    changed[0] = b' ';
    let mut longer = raw.clone();
    longer.push(b'\n');
    for stored in [&changed[..], &raw[..raw.len() - 1], &longer[..]] {
        std::fs::write(&path, stored).unwrap();
        assert!(artifacts::summarize(&raw, &path, &provenance())
            .unwrap_err()
            .contains("raw bytes differ"));
    }
}

#[test]
fn hardlink_aliases_are_rejected_before_any_file_is_truncated() {
    let dir = Temp::new();
    let raw_path = dir.0.join("raw.json");
    let alias_path = dir.0.join("alias.json");
    let raw = synthetic_full_report();
    std::fs::write(&raw_path, &raw).unwrap();
    std::fs::hard_link(&raw_path, &alias_path).unwrap();
    assert!(artifacts::convert(&raw_path, &alias_path, &provenance()).is_err());
    assert_eq!(std::fs::read(&raw_path).unwrap(), raw);
    let pair = sweep(
        &dir.0,
        &["--out", "raw.json", "--summary-out", "alias.json"],
    );
    assert!(!pair.status.success());
    assert!(String::from_utf8_lossy(&pair.stderr).contains("distinct"));
    assert_eq!(std::fs::read(&raw_path).unwrap(), raw);
    let reference = sweep(
        &dir.0,
        &[
            "--out",
            "raw.json",
            "--summary-out",
            "summary.json",
            "--compare",
            "alias.json",
        ],
    );
    assert!(!reference.status.success());
    assert!(String::from_utf8_lossy(&reference.stderr).contains("overwrite comparison reference"));
    assert_eq!(std::fs::read(&raw_path).unwrap(), raw);
    assert!(!dir.0.join("summary.json").exists());
}

#[cfg(unix)]
#[test]
fn self_reference_resolution_errors_are_not_ignored() {
    use std::os::unix::fs::PermissionsExt;

    let dir = Temp::new();
    let raw_path = dir.0.join("raw.json");
    std::fs::write(&raw_path, synthetic_full_report()).unwrap();
    let loop_path = dir.0.join("loop");
    std::os::unix::fs::symlink(&loop_path, &loop_path).unwrap();
    let mut report: bench::Report = serde_json::from_slice(&synthetic_full_report()).unwrap();
    let selection = bench::ReferenceSelection {
        paths: vec![raw_path],
        absence: None,
    };
    assert!(
        bench::apply_comparisons_for_outputs(&mut report, &selection, &[&loop_path])
            .unwrap_err()
            .contains("cannot resolve")
    );

    let blocked = dir.0.join("blocked");
    std::fs::create_dir(&blocked).unwrap();
    std::fs::set_permissions(&blocked, std::fs::Permissions::from_mode(0o000)).unwrap();
    let blocked_result = artifacts::resolved_path(&blocked.join("child"));
    std::fs::set_permissions(&blocked, std::fs::Permissions::from_mode(0o700)).unwrap();
    assert!(
        blocked_result.unwrap_err().contains("cannot resolve"),
        "non-NotFound resolution errors must not be treated as absent paths"
    );
}

#[test]
fn measurement_evidence_distinguishes_clean_and_dirty_git_worktrees() {
    let dir = Temp::new();
    let run_git = |args: &[&str]| {
        let output = Command::new("git")
            .current_dir(&dir.0)
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    run_git(&["init", "--quiet"]);
    std::fs::write(dir.0.join("tracked"), b"clean").unwrap();
    run_git(&["add", "tracked"]);
    run_git(&[
        "-c",
        "user.name=Test",
        "-c",
        "user.email=test@example.invalid",
        "-c",
        "core.hooksPath=/dev/null",
        "commit",
        "--quiet",
        "-m",
        "fixture",
    ]);
    let invocation = artifacts::Invocation {
        executable: "v3-cli".into(),
        arguments: Vec::new(),
        working_directory: dir.0.clone(),
    };

    assert_eq!(
        artifacts::measurement_evidence(&invocation, false)["dirty"],
        false
    );
    std::fs::write(dir.0.join("untracked"), b"dirty").unwrap();
    assert_eq!(
        artifacts::measurement_evidence(&invocation, false)["dirty"],
        true
    );
}

fn sweep(dir: &Path, extra: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_v3-cli"))
        .current_dir(dir)
        .args([
            "bench",
            "--profile",
            "sweep",
            "--width",
            "8",
            "--height",
            "8",
            "--founders",
            "2",
            "--seeds",
            "11",
            "--ticks",
            "1",
            "--threads",
            "1",
        ])
        .args(extra)
        .output()
        .unwrap()
}

#[test]
fn direct_sweep_writes_pair_and_conversion_never_overwrites_input() {
    let dir = Temp::new();
    let run = sweep(&dir.0, &["--out", "raw.json"]);
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    let raw = std::fs::read(dir.0.join("raw.json")).unwrap();
    let summary: Value =
        serde_json::from_slice(&std::fs::read(dir.0.join("raw.summary.json")).unwrap()).unwrap();
    assert_eq!(summary["raw"]["sha256"], artifacts::sha256(&raw));
    assert_eq!(summary["measurement_evidence"]["cli_exit"]["code"], 0);
    assert!(summary["measurement_evidence"]["outer_exit"].is_null());
    assert!(summary["measurement_evidence"]["effective_config_digest"].is_string());
    let invocation = &summary["measurement_evidence"]["command"];
    assert_eq!(
        invocation["working_directory"],
        dir.0.canonicalize().unwrap().to_str().unwrap()
    );
    assert!(invocation["arguments"]
        .as_array()
        .unwrap()
        .contains(&json!("--out")));
    std::fs::write(
        dir.0.join("provenance.json"),
        serde_json::to_vec(&provenance()).unwrap(),
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_v3-cli"))
        .current_dir(&dir.0)
        .args([
            "bench-summarize",
            "--input",
            "raw.json",
            "--out",
            "raw.json",
            "--provenance",
            "provenance.json",
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert_eq!(std::fs::read(dir.0.join("raw.json")).unwrap(), raw);
}

#[test]
fn write_failure_and_aliases_fail_before_announcing_pair() {
    let dir = Temp::new();
    std::fs::write(dir.0.join("blocked"), b"not a directory").unwrap();
    let failed = sweep(
        &dir.0,
        &["--out", "blocked/raw.json", "--summary-out", "summary.json"],
    );
    assert!(!failed.status.success());
    assert!(!dir.0.join("summary.json").exists());
    assert!(!String::from_utf8_lossy(&failed.stdout).contains("wrote"));
    std::fs::create_dir(dir.0.join("summary directory")).unwrap();
    let failed_summary = sweep(
        &dir.0,
        &[
            "--out",
            "complete-raw.json",
            "--summary-out",
            "summary directory",
        ],
    );
    assert!(!failed_summary.status.success());
    assert!(dir.0.join("complete-raw.json").is_file());
    assert!(!String::from_utf8_lossy(&failed_summary.stdout).contains("wrote"));
    let aliased = sweep(
        &dir.0,
        &["--out", "raw.json", "--summary-out", "./raw.json"],
    );
    assert!(!aliased.status.success());
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&dir.0, dir.0.join("alias")).unwrap();
        let aliased = sweep(
            &dir.0,
            &["--out", "raw.json", "--summary-out", "alias/raw.json"],
        );
        assert!(!aliased.status.success());
    }
}

proptest! {
    #[test]
    fn persistence_and_claims_have_fixed_bounds(sample_count in 0usize..100) {
        let dir = Temp::new();
        let path = dir.0.join("raw.json");
        let mut source = world_source();
        let row = &mut source["deterministic"]["goal_indicators"]["population_persistence"]["per_seed"][0];
        let sample = row["samples"][0].clone();
        row["samples"] = json!((0..sample_count).map(|i| {
            let mut sample = sample.clone(); sample["tick"] = json!(i); sample
        }).collect::<Vec<_>>());
        let raw = serde_json::to_vec(&source).unwrap();
        std::fs::write(&path, &raw).unwrap();
        let summary = artifacts::summarize(&raw, &path, &provenance()).unwrap();
        let samples = summary.deterministic["goal_indicators"]["population_persistence"]["per_seed"][0]["samples"].as_array().unwrap();
        prop_assert_eq!(samples.len(), sample_count.min(artifacts::MAX_PERSISTENCE_CHECKPOINTS));
        if sample_count > 0 {
            prop_assert_eq!(&samples[0]["tick"], &json!(0));
            prop_assert_eq!(&samples.last().unwrap()["tick"], &json!(sample_count - 1));
        }
        prop_assert!(summary.claims.len() <= artifacts::MAX_CLAIMS);
        for claim in summary.claims {
            prop_assert_eq!(source.pointer(&claim.pointer), Some(&claim.value));
        }
    }

    #[test]
    fn comparison_inputs_keep_unrounded_case_ratios(numerator in 0u64..100000, denominator in 1u64..100000) {
        let dir = Temp::new();
        let path = dir.0.join("raw.json");
        let mut source = world_source();
        source["deterministic"]["per_seed"][0]["births"] = json!(numerator);
        source["deterministic"]["per_seed"][0]["creature_ticks"] = json!(denominator);
        let raw = serde_json::to_vec(&source).unwrap();
        std::fs::write(&path, &raw).unwrap();
        let summary = artifacts::summarize(&raw, &path, &provenance()).unwrap();
        let summary_path = dir.0.join("summary.json");
        std::fs::write(&summary_path, artifacts::summary_bytes(&summary).unwrap()).unwrap();
        let mut current: bench::Report = serde_json::from_slice(&raw).unwrap();
        current.deterministic.per_seed[0].births += 1;
        let mut full = serde_json::to_value(bench::compare_against_path(&current, &path).unwrap()).unwrap();
        full["path"] = json!(summary_path);
        let compact = serde_json::to_value(bench::compare_against_path(&current, &summary_path).unwrap()).unwrap();
        prop_assert_eq!(compact, full);
    }
}

#[test]
fn both_output_identities_are_excluded_and_explicit_raw_cannot_overwrite_reference() {
    let dir = Temp::new();
    let raw_path = dir.0.join("raw.json");
    let summary_path = dir.0.join("summary.json");
    let mut report: bench::Report = serde_json::from_slice(&synthetic_full_report()).unwrap();
    let selection = bench::ReferenceSelection {
        paths: vec![raw_path.clone(), summary_path.clone()],
        absence: None,
    };
    assert!(!bench::apply_comparisons_for_outputs(
        &mut report,
        &selection,
        &[&raw_path, &summary_path]
    )
    .unwrap());
    assert!(report.comparison.references.is_empty());
    assert!(report
        .comparison
        .reference_absence
        .unwrap()
        .contains("own output path"));
    std::fs::write(&raw_path, b"untouched reference").unwrap();
    let failed = sweep(
        &dir.0,
        &[
            "--out",
            "raw.json",
            "--summary-out",
            "summary.json",
            "--compare",
            "./raw.json",
        ],
    );
    assert!(!failed.status.success());
    assert!(String::from_utf8_lossy(&failed.stderr).contains("overwrite comparison reference"));
    assert_eq!(std::fs::read(&raw_path).unwrap(), b"untouched reference");
    assert!(!summary_path.exists());
}

#[test]
fn summary_profile_errors_case_labels_and_host_rules_match_full() {
    let dir = Temp::new();
    let raw_path = dir.0.join("raw.json");
    let summary_path = dir.0.join("summary.json");
    let mut source = world_source();
    source["environment"]["host"]["hostname"] = json!("different host");
    let raw = serde_json::to_vec(&source).unwrap();
    std::fs::write(&raw_path, &raw).unwrap();
    let summary = artifacts::summarize(&raw, &raw_path, &provenance()).unwrap();
    std::fs::write(&summary_path, artifacts::summary_bytes(&summary).unwrap()).unwrap();
    let mut current: bench::Report = serde_json::from_value(world_source()).unwrap();
    current.deterministic.profile.cases[0].config_digest = "changed".into();
    current.deterministic.profile.cases.push(bench::GoalCase {
        name: "absent world".into(),
        ..current.deterministic.profile.cases[0].clone()
    });
    let compact = bench::compare_against_path(&current, &summary_path).unwrap();
    assert!(compact.wall_clock.is_none());
    assert!(compact.cases[0].inputs_changed);
    assert!(compact.cases[1].absent_in_reference);
    let extinction = compact.cases[0]
        .readings
        .iter()
        .find(|r| r.name == "extinction_tick")
        .unwrap();
    assert!(extinction.percent_delta.is_none());
    let mut full =
        serde_json::to_value(bench::compare_against_path(&current, &raw_path).unwrap()).unwrap();
    full["path"] = json!(summary_path);
    assert_eq!(serde_json::to_value(compact).unwrap(), full);
    current.deterministic.profile.ticks += 1;
    assert!(bench::compare_against_path(&current, &summary_path)
        .unwrap_err()
        .contains("different profile"));
    assert!(
        bench::compare_against_path(&current, &dir.0.join("missing.json"))
            .unwrap_err()
            .contains("failed to read reference")
    );
}

#[test]
fn severe_completed_sweep_retains_both_artifacts_and_status() {
    let dir = Temp::new();
    assert!(sweep(&dir.0, &["--out", "reference.json"]).status.success());
    let mut reference: Value =
        serde_json::from_slice(&std::fs::read(dir.0.join("reference.json")).unwrap()).unwrap();
    reference["deterministic"]["per_creature_tick"]["vm_steps"] = json!("0.000000");
    std::fs::write(
        dir.0.join("reference.json"),
        serde_json::to_vec(&reference).unwrap(),
    )
    .unwrap();
    let result = sweep(
        &dir.0,
        &["--out", "severe.json", "--compare", "reference.json"],
    );
    assert_eq!(result.status.code(), Some(3));
    let summary: Value =
        serde_json::from_slice(&std::fs::read(dir.0.join("severe.summary.json")).unwrap()).unwrap();
    assert_eq!(summary["comparison"]["severe"], true);
    assert_eq!(summary["measurement_evidence"]["cli_exit"]["code"], 3);
    assert_eq!(
        summary["raw"]["sha256"],
        artifacts::sha256(&std::fs::read(dir.0.join("severe.json")).unwrap())
    );
}

#[test]
fn output_defaults_resolve_main_and_linked_checkouts_with_spaces() {
    let dir = Temp::new();
    let main = dir.0.join("main checkout");
    let linked = dir.0.join("linked checkout");
    std::fs::create_dir(&main).unwrap();
    let git = |cwd: &Path, args: &[&str]| {
        let output = Command::new("git")
            .current_dir(cwd)
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    git(&main, &["init", "--quiet"]);
    git(
        &main,
        &[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.invalid",
            "-c",
            "core.hooksPath=/dev/null",
            "commit",
            "--quiet",
            "--allow-empty",
            "-m",
            "fixture",
        ],
    );
    git(
        &main,
        &[
            "worktree",
            "add",
            "--quiet",
            "--detach",
            linked.to_str().unwrap(),
        ],
    );
    for checkout in [&main, &linked] {
        for profile in ["gate", "goal", "sweep"] {
            let paths =
                artifacts::output_paths(checkout, profile, Some("test-feature"), None, None)
                    .unwrap();
            assert_eq!(
                paths.raw,
                main.canonicalize()
                    .unwrap()
                    .join(format!(".bench-artifacts/test-feature/{profile}.json"))
            );
            let suffix = if profile == "gate" {
                String::new()
            } else {
                format!("-{profile}")
            };
            assert_eq!(
                paths.summary,
                checkout
                    .canonicalize()
                    .unwrap()
                    .join(format!("docs/progress/features/test-feature{suffix}.json"))
            );
        }
        let result = sweep(checkout, &["--feature", "test-feature"]);
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(main
            .join(".bench-artifacts/test-feature/sweep.json")
            .is_file());
        assert!(checkout
            .join("docs/progress/features/test-feature-sweep.json")
            .is_file());
    }
    let overrides = artifacts::output_paths(
        &dir.0,
        "gate",
        Some("test-feature"),
        Some(Path::new("custom raw.json")),
        Some(Path::new("custom summary.json")),
    )
    .unwrap();
    assert_eq!(
        overrides.raw,
        dir.0.canonicalize().unwrap().join("custom raw.json")
    );
    assert!(artifacts::output_paths(&dir.0, "gate", Some("test-feature"), None, None).is_err());
    assert!(artifacts::output_paths(&main, "sweep", Some("../escape"), None, None).is_err());
    let bare = dir.0.join("bare repository");
    git(
        &dir.0,
        &[
            "clone",
            "--quiet",
            "--bare",
            main.to_str().unwrap(),
            bare.to_str().unwrap(),
        ],
    );
    let bare_linked = dir.0.join("bare linked checkout");
    git(
        &bare,
        &[
            "worktree",
            "add",
            "--quiet",
            "--detach",
            bare_linked.to_str().unwrap(),
        ],
    );
    assert!(
        artifacts::output_paths(&bare_linked, "gate", Some("test-feature"), None, None).is_err()
    );
    assert!(artifacts::output_paths(
        &bare_linked,
        "gate",
        Some("test-feature"),
        Some(Path::new("raw.json")),
        Some(Path::new("summary.json"))
    )
    .is_ok());
}

#[test]
fn recruitment_projection_keeps_estimates_counts_and_pairing_but_no_trace_payloads() {
    use v3_core::neighborhood::recruitment_paths as recruitment;
    let dir = Temp::new();
    let raw_path = dir.0.join("raw.json");
    let mut source = world_source();
    let mut experiment = recruitment::observe(recruitment::Sizes::TEST);
    // Known nonzero rows exercise pooling independently of which tiny mutations occur.
    use v3_core::mutation::MutationOperator;
    use v3_core::neighborhood::recruitment::ModuleBackend;
    for (index, proposal) in experiment.arms[0].lineages[0]
        .proposals
        .iter_mut()
        .enumerate()
    {
        proposal.selected_inapplicable_by_backend_operator.clear();
        proposal.selected_inapplicable_by_backend_operator.insert(
            ModuleBackend::Vm,
            [(MutationOperator::VmConstantMutation, 3)].into(),
        );
        proposal.selected_inapplicable_backend_unresolved = index as u64 + 1;
    }
    source["deterministic"]["goal_indicators"]["recruitment_paths"] =
        serde_json::to_value(&experiment).unwrap();
    let raw = serde_json::to_vec(&source).unwrap();
    std::fs::write(&raw_path, &raw).unwrap();
    let summary = artifacts::summarize(&raw, &raw_path, &provenance()).unwrap();
    let compact = &summary.deterministic["goal_indicators"]["recruitment_paths"];
    let proposal_count = experiment.arms[0].lineages[0].proposals.len() as u64;
    assert_eq!(
        compact["arms"][0]["selected_inapplicable_by_backend_operator"]["Vm"]["VmConstantMutation"],
        proposal_count * 3
    );
    assert_eq!(
        compact["arms"][0]["selected_inapplicable_backend_unresolved"],
        proposal_count * (proposal_count + 1) / 2
    );
    assert_eq!(compact["total_proposals"], experiment.total_proposals);
    assert_eq!(
        compact["pairs"],
        serde_json::to_value(&experiment.pairs).unwrap()
    );
    assert_eq!(
        compact["opportunities"],
        serde_json::to_value(&experiment.opportunities).unwrap()
    );
    let stage = &experiment.constructed[0].stages[0];
    assert_eq!(
        compact["constructed"][0]["stages"][0],
        json!({
            "name": stage.name,
            "edits": stage.edits,
            "seed": stage.seed,
            "task_summary": stage.task.summary(),
            "battery_class": stage.battery_class,
            "incumbent_actions_unchanged": stage.incumbent_actions_unchanged,
            "useful": stage.useful,
        })
    );
    let mut total = 0;
    for (index, arm) in experiment.arms.iter().enumerate() {
        let projected = &compact["arms"][index];
        let actual_count = arm
            .lineages
            .iter()
            .map(|lineage| lineage.proposals.len())
            .sum::<usize>();
        total += actual_count;
        assert_eq!(projected["proposal_count"], actual_count);
        assert_eq!(
            projected["summary"],
            serde_json::to_value(&arm.summary).unwrap()
        );
        assert_eq!(
            projected["batches"],
            serde_json::to_value(&arm.batches).unwrap()
        );
        assert_eq!(
            projected["outcome_counts"]["chosen"],
            arm.lineages
                .iter()
                .flat_map(|l| &l.proposals)
                .filter(|p| p.chosen)
                .count()
        );
        assert_eq!(
            projected["lineages"][0]["retention"],
            serde_json::to_value(&arm.lineages[0].retention).unwrap()
        );
    }
    assert_eq!(total as u64, experiment.total_proposals);
    let compact_bytes = serde_json::to_vec(compact).unwrap();
    let text = String::from_utf8(compact_bytes).unwrap();
    for forbidden in [
        "\"genome\":",
        "\"delta\":",
        "\"scenes\":",
        "\"proposals\":",
        "\"first_successful_path\":",
        "\"battery\":",
    ] {
        assert!(
            !text.contains(forbidden),
            "unexpected raw payload {forbidden}"
        );
    }
    let mut duplicated = source;
    let proposals = duplicated["deterministic"]["goal_indicators"]["recruitment_paths"]["arms"][0]
        ["lineages"][0]["proposals"]
        .as_array_mut()
        .unwrap();
    let rows = proposals.clone();
    for _ in 0..50 {
        proposals.extend(rows.clone());
    }
    let expanded = serde_json::to_vec(&duplicated).unwrap();
    std::fs::write(&raw_path, &expanded).unwrap();
    let expanded_summary = artifacts::summarize(&expanded, &raw_path, &provenance()).unwrap();
    assert_eq!(summary.claims.len(), expanded_summary.claims.len());
    assert!(
        artifacts::summary_bytes(&expanded_summary).unwrap().len()
            < artifacts::summary_bytes(&summary).unwrap().len() + 300
    );
}
