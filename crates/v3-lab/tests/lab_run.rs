//! End-to-end runs at tiny sizes: determinism across thread counts and
//! processes, the `uncalibrated` exit, the byte-cap stop, and the summary
//! keep-list.

use std::path::{Path, PathBuf};
use std::process::Command;

use v3_lab::arena::{resolve_arm, Policy, Role};
use v3_lab::calibration::draw_scenes;
use v3_lab::campaign::{run_campaign, Arm, ArmKind, Plan};
use v3_lab::cli::read_summary;
use v3_lab::eval::Setup;
use v3_lab::output::{checkout_root, Budget, RunDir};
use v3_lab::run::{run, RunParams, UserArm, EXIT_UNCALIBRATED};
use v3_lab::summary::{render_report, StoppedBy, SUMMARY_KIND, SUMMARY_VERSION};

/// A scratch directory under the checkout's `.bench-artifacts/lab/`,
/// removed on drop.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let path = checkout_root()
            .unwrap()
            .join(".bench-artifacts/lab")
            .join(format!("test-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn tiny(out: &Path) -> RunParams {
    RunParams {
        seed: 7,
        replicates: 1,
        generations: 2,
        population: 2,
        elite_fraction: 0.5,
        scenes: 1,
        validation_scenes: 4,
        lifetime: Some(100),
        arena_size: 48,
        food_fraction: Some(0.08),
        start_energy: 100.0,
        calibration_fractions: vec![0.08],
        calibration_lifetimes: vec![100],
        calibration_scenes: 4,
        calibration_margin: 1.0,
        reach_threshold: None,
        arms: Vec::new(),
        genome: None,
        comparator: None,
        threads: 1,
        byte_cap: 64 << 20,
        out: Some(out.to_path_buf()),
        calibrate_only: false,
        quick: false,
    }
}

#[test]
fn same_seed_rows_are_byte_identical_across_threads_and_processes() {
    let scratch = Scratch::new("determinism");
    let overlay = scratch.0.join("hot.json");
    std::fs::write(&overlay, r#"{"mutation": {"per_unit_rate": 0.02}}"#).unwrap();
    let mut params = tiny(&scratch.0.join("one"));
    params.arms = vec![UserArm {
        name: "hot".into(),
        overlay: overlay.clone(),
        genome: None,
    }];
    let outcome = run(&params).unwrap();
    assert_eq!(outcome.exit_code, 0, "{}", render_report(&outcome.summary));

    let status = Command::new(env!("CARGO_BIN_EXE_v3-lab"))
        .args([
            "run",
            "--seed",
            "7",
            "--replicates",
            "1",
            "--generations",
            "2",
        ])
        .args([
            "--population",
            "2",
            "--elite-fraction",
            "0.5",
            "--scenes",
            "1",
        ])
        .args([
            "--validation-scenes",
            "4",
            "--lifetime",
            "100",
            "--arena-size",
            "48",
        ])
        .args([
            "--food-fraction",
            "0.08",
            "--calibration-scenes",
            "4",
            "--threads",
            "2",
        ])
        .arg("--arm")
        .arg(format!("hot={}", overlay.display()))
        .arg("--out")
        .arg(scratch.0.join("two"))
        .output()
        .unwrap();
    assert!(
        status.status.success(),
        "{}",
        String::from_utf8_lossy(&status.stderr)
    );

    let one = std::fs::read(scratch.0.join("one/rows.ndjson")).unwrap();
    let two = std::fs::read(scratch.0.join("two/rows.ndjson")).unwrap();
    assert!(!one.is_empty());
    assert_eq!(
        one, two,
        "rows differ between threads 1 in-process and 2 in a child"
    );

    // Labels: the user arm raised the mutation rate, so it is a policy
    // deviation with no reachability, like mutation-off.
    let summary = read_summary(&scratch.0.join("two/summary.json")).unwrap();
    let hot = summary.arms.iter().find(|a| a.name == "hot").unwrap();
    assert_eq!(
        (hot.role, hot.policy),
        (Role::User, Policy::PolicyDeviation)
    );
    assert!(!hot.reach_reported && hot.reached_fraction.is_none());
    let off = summary
        .arms
        .iter()
        .find(|a| a.name == "mutation-off")
        .unwrap();
    assert_eq!(
        (off.role, off.policy),
        (Role::Control, Policy::PolicyDeviation)
    );
    let native = summary.arms.iter().find(|a| a.name == "native").unwrap();
    assert!(native.reached_fraction.is_some() && native.wilson_95.is_some());
    for arm in &summary.arms {
        for replicate in &arm.replicates {
            assert!(!replicate.incomplete);
            if arm.reach_reported {
                assert_eq!(replicate.censored, Some(replicate.reached == Some(false)));
            }
        }
    }
    let rows = String::from_utf8(one).unwrap();
    assert!(rows
        .lines()
        .any(|l| l.contains(r#""arm":"hot","role":"user","policy":"policy-deviation""#)));
}

#[test]
fn an_unmet_gate_is_uncalibrated_with_exit_2_and_no_campaign() {
    let scratch = Scratch::new("uncalibrated");
    let mut params = tiny(&scratch.0.join("run"));
    params.calibration_margin = 1e9;
    let outcome = run(&params).unwrap();
    assert_eq!(outcome.exit_code, EXIT_UNCALIBRATED);
    let summary = read_summary(&outcome.dir.join("summary.json")).unwrap();
    assert_eq!(summary.incomplete.as_deref(), Some("uncalibrated"));
    assert!(summary.calibration.selected.is_none());
    assert!(summary.arms.is_empty());
    assert!(summary.calibration.points.iter().all(|p| p.means.is_some()));
    assert!(std::fs::read(outcome.dir.join("rows.ndjson"))
        .unwrap()
        .is_empty());
}

#[test]
fn the_summary_carries_the_versioned_keep_list_and_reports_alone() {
    let scratch = Scratch::new("keep-list");
    let mut params = tiny(&scratch.0.join("run"));
    params.calibrate_only = true;
    let outcome = run(&params).unwrap();
    let text = std::fs::read_to_string(outcome.dir.join("summary.json")).unwrap();
    let value: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(value["kind"], SUMMARY_KIND);
    assert_eq!(value["summary_version"], SUMMARY_VERSION);
    for key in [
        "provenance",
        "calibration",
        "arms",
        "fidelity",
        "incomplete",
        "exit_code",
        "timing",
    ] {
        assert!(value.get(key).is_some(), "missing {key}");
    }
    for key in [
        "git_revision",
        "dirty",
        "config_digest",
        "overlays",
        "genomes",
        "arena",
        "seeds",
        "threads",
        "sizes",
    ] {
        assert!(
            value["provenance"].get(key).is_some(),
            "missing provenance.{key}"
        );
    }
    let report = render_report(&read_summary(&outcome.dir.join("summary.json")).unwrap());
    assert!(report.contains("Calibration") && report.contains("| 0.08 | 100 |"));
}

#[test]
fn out_outside_bench_artifacts_is_refused() {
    let mut params = tiny(Path::new("/tmp/petri-lab-outside"));
    params.calibrate_only = true;
    assert!(run(&params).is_err());
}

#[test]
fn the_byte_cap_stops_the_run_and_marks_replicates_incomplete() {
    let scratch = Scratch::new("byte-cap");
    let reference = resolve_arm(None, 48, 100.0).unwrap();
    let founder = v3_core::creature::founder::founder_genome_with_age_gate(
        reference.population.founder_profile,
        &reference.energy.lifecycle,
    );
    let setup = Setup::new(reference, 100.0, 60);
    let arms = vec![
        Arm {
            name: "native".into(),
            role: Role::Reference,
            policy: Policy::Native,
            setup: setup.clone(),
            kind: ArmKind::Evolving {
                start: founder.clone(),
                shuffled: false,
            },
        },
        Arm {
            name: "founder-only".into(),
            role: Role::Control,
            policy: Policy::Native,
            setup,
            kind: ArmKind::Fixed(founder),
        },
    ];
    let plan = Plan {
        seed: 3,
        replicates: 3,
        generations: 4,
        population: 4,
        elite_fraction: 0.5,
        scenes: 2,
        food_fraction: 0.06,
        threshold: 1e9,
    };
    let validation = draw_scenes(11, 2, 48, 0.06, 5).unwrap();
    // Room for a few rows only: rows and elites get cap − reserve bytes.
    let mut dir =
        RunDir::create(scratch.0.join("run"), Budget::new(12_000, 4_000).unwrap()).unwrap();
    let (arms, totals) = run_campaign(&arms, &plan, &validation, &mut dir).unwrap();
    assert!(totals.byte_cap_hit);
    let rows = std::fs::read(scratch.0.join("run/rows.ndjson")).unwrap();
    assert!(!rows.is_empty() && rows.len() <= 8_000);
    for arm in &arms {
        assert!(arm.incomplete_replicates > 0);
        assert_eq!(arm.reached_fraction, None);
        assert_eq!(arm.wilson_95, None);
        let last = arm.replicates.last().unwrap();
        assert!(last.incomplete && last.reached.is_none() && last.censored.is_none());
        assert!(matches!(
            last.stopped_by,
            StoppedBy::ByteCap | StoppedBy::NotStarted
        ));
    }
}
