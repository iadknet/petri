//! End-to-end runs at tiny sizes: determinism across thread counts and
//! processes, the `uncalibrated` exit, the byte-cap stop, the summary
//! keep-list, and barrier navigation on both built-in arenas and layouts.

use std::path::{Path, PathBuf};
use std::process::Command;

use v3_lab::arena::{resolve_arm, Policy, Role};
use v3_lab::calibration::draw_scenes;
use v3_lab::campaign::{run_campaign, Arm, ArmKind, Plan};
use v3_lab::cli::read_summary;
use v3_lab::eval::{evaluate_genome, Setup};
use v3_lab::output::{Budget, GitProvenance, LabRoot, RunDir, SUMMARY_RESERVE};
use v3_lab::readings::SignatureArms;
use v3_lab::run::{run, RunOutcome, RunParams, UserArm, EXIT_UNCALIBRATED};
use v3_lab::scene::{ArenaId, Assay, SceneSpec};
use v3_lab::summary::{
    render_report, Incomplete, LifetimeLearning, StoppedBy, GENOME_FORMAT, SUMMARY_KIND,
    SUMMARY_VERSION,
};
use v3_lab::{sha256_hex, GenomeFile, LabError};

/// A temporary lab root outside any checkout, with `lab` =
/// `<root>/.bench-artifacts/lab`; removed on drop.
struct Scratch {
    root: PathBuf,
    lab: PathBuf,
}

impl Scratch {
    fn new(name: &str) -> Self {
        let root =
            std::env::temp_dir().join(format!("petri-lab-test-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let lab = root.join(".bench-artifacts/lab");
        std::fs::create_dir_all(&lab).unwrap();
        Self { root, lab }
    }

    /// A library run on the injected root, which is not a git checkout.
    fn run(&self, params: &RunParams) -> Result<RunOutcome, LabError> {
        run(params, &LabRoot::without_git(self.root.clone()))
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn tiny(out: &Path) -> RunParams {
    RunParams {
        assay: Assay::FoodSeeking,
        arena: None,
        layout: None,
        seed: 7,
        replicates: 1,
        generations: 2,
        population: 2,
        elite_fraction: 0.5,
        scenes: 1,
        validation_scenes: 4,
        lifetime: Some(100),
        arena_size: Some(48),
        food_fraction: Some(0.08),
        scale: None,
        start_energy: 100.0,
        calibration_fractions: None,
        calibration_scales: None,
        calibration_lifetimes: vec![100],
        blocked_weight: None,
        efficiency_weight: None,
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
        mutants: 2,
        signature_arms: SignatureArms::Changing,
    }
}

const CHILD_ROOT: &str = "PETRI_LAB_TEST_CHILD_ROOT";
const CHILD_OVERLAY: &str = "PETRI_LAB_TEST_CHILD_OVERLAY";

/// Child half of the determinism test: `tiny` with the `hot` arm at 2
/// threads into `<root>/.bench-artifacts/lab/two`.
#[test]
#[ignore = "spawned by same_seed_rows_are_byte_identical_across_threads_and_processes"]
fn child_run_for_the_determinism_test() {
    let root = PathBuf::from(std::env::var_os(CHILD_ROOT).expect("child root"));
    let overlay = PathBuf::from(std::env::var_os(CHILD_OVERLAY).expect("child overlay"));
    let mut params = tiny(&root.join(".bench-artifacts/lab/two"));
    params.threads = 2;
    params.mutants = 4;
    params.signature_arms = SignatureArms::All;
    params.arms = vec![UserArm {
        name: "hot".into(),
        overlay,
        genome: None,
    }];
    let outcome = run(&params, &LabRoot::without_git(root)).unwrap();
    assert_eq!(outcome.exit_code, 0);
}

#[test]
fn same_seed_rows_are_byte_identical_across_threads_and_processes() {
    let scratch = Scratch::new("determinism");
    let overlay = scratch.lab.join("hot.json");
    std::fs::write(&overlay, r#"{"mutation": {"per_unit_rate": 0.02}}"#).unwrap();
    let mut params = tiny(&scratch.lab.join("one"));
    params.mutants = 4;
    params.signature_arms = SignatureArms::All;
    params.arms = vec![UserArm {
        name: "hot".into(),
        overlay: overlay.clone(),
        genome: None,
    }];
    let outcome = scratch.run(&params).unwrap();
    assert_eq!(outcome.exit_code, 0, "{}", render_report(&outcome.summary));

    // The same run in a child process at 2 threads. The CLI refuses outside
    // a checkout, so the child is this test binary's ignored helper.
    let status = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "child_run_for_the_determinism_test", "--ignored"])
        .env(CHILD_ROOT, &scratch.root)
        .env(CHILD_OVERLAY, &overlay)
        .output()
        .unwrap();
    assert!(
        status.status.success(),
        "{}",
        String::from_utf8_lossy(&status.stdout)
    );

    let one = std::fs::read(scratch.lab.join("one/rows.ndjson")).unwrap();
    let two = std::fs::read(scratch.lab.join("two/rows.ndjson")).unwrap();
    assert!(!one.is_empty());
    assert_eq!(
        one, two,
        "rows differ between threads 1 in-process and 2 in a child"
    );

    // Labels: the user arm raised the mutation rate, so it is a policy
    // deviation like mutation-off: its reach is tested but not reported as
    // native reachability.
    let summary = read_summary(&scratch.lab.join("two/summary.json")).unwrap();
    let hot = summary.arms.iter().find(|a| a.name == "hot").unwrap();
    assert_eq!(
        (hot.role, hot.policy),
        (Role::User, Policy::PolicyDeviation)
    );
    assert!(!hot.reach_reported && hot.reached_fraction.is_some());
    let off = summary
        .arms
        .iter()
        .find(|a| a.name == "mutation-off")
        .unwrap();
    assert_eq!(
        (off.role, off.policy),
        (Role::Control, Policy::PolicyDeviation)
    );
    assert!(!off.reach_reported && off.wilson_95.is_some());
    let native = summary.arms.iter().find(|a| a.name == "native").unwrap();
    assert!(native.reach_reported);
    assert!(native.reached_fraction.is_some() && native.wilson_95.is_some());
    for arm in &summary.arms {
        let tested = arm.role != Role::Instrument;
        for replicate in &arm.replicates {
            assert!(!replicate.incomplete);
            assert_eq!(replicate.reached.is_some(), tested, "{}", arm.name);
            if tested {
                assert_eq!(replicate.censored, Some(replicate.reached == Some(false)));
            }
        }
    }

    // The report prints deviation arms' reach under a diagnostic label, apart
    // from the native reachability table.
    let report = render_report(&summary);
    let (native_part, diagnostic_part) = report
        .split_once("## Diagnostic reach")
        .expect("diagnostic block");
    for name in ["hot", "mutation-off"] {
        let line = format!("| {name} |");
        assert!(!native_part.contains(&line), "{name} in the native table");
        assert!(diagnostic_part.contains(&line), "{name} not diagnostic");
    }
    assert!(native_part.contains("| native |"));
    let rows = String::from_utf8(one).unwrap();
    assert!(rows
        .lines()
        .any(|l| l.contains(r#""arm":"hot","role":"user","policy":"policy-deviation""#)));

    // Under `all`, every genome arm's rows carry four mutants; scripted
    // arms carry no readings.
    for line in rows.lines() {
        let row: serde_json::Value = serde_json::from_str(line).unwrap();
        assert_eq!(row["row_version"], 2);
        if row["arm"] == "random-walk" {
            assert!(row["readings"].is_null());
        } else {
            assert!(row["readings"]["shape"].is_object(), "{}", row["arm"]);
            assert_eq!(row["readings"]["signature"]["mutants"]["n"], 4);
        }
    }
    assert_eq!(summary.provenance.sizes.mutants, Some(4));
    assert_eq!(
        summary.provenance.sizes.signature_arms,
        Some(SignatureArms::All)
    );
}

#[test]
fn an_unmet_gate_is_uncalibrated_with_exit_2_and_no_campaign() {
    let scratch = Scratch::new("uncalibrated");
    let mut params = tiny(&scratch.lab.join("run"));
    params.calibration_margin = 1e9;
    let outcome = scratch.run(&params).unwrap();
    assert_eq!(outcome.exit_code, EXIT_UNCALIBRATED);
    let summary = read_summary(&outcome.dir.join("summary.json")).unwrap();
    assert_eq!(summary.incomplete, Some(Incomplete::Uncalibrated));
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
    let mut params = tiny(&scratch.lab.join("run"));
    params.calibrate_only = true;
    let outcome = scratch.run(&params).unwrap();
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
    for key in ["mutants", "signature_arms"] {
        assert!(
            !value["provenance"]["sizes"][key].is_null(),
            "sizes.{key} in every summary"
        );
    }
    assert_eq!(value["timing"]["readings_creature_ticks"], 0);
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
    assert!(report.contains("| fraction | lifetime |"), "{report}");

    // `--calibrate-only` stops after a passing gate: no arms, rows or fidelity.
    assert_eq!(outcome.exit_code, 0);
    assert!(outcome.summary.calibration.selected.is_some());
    assert!(outcome.summary.arms.is_empty() && outcome.summary.fidelity.is_none());
    assert!(std::fs::read(outcome.dir.join("rows.ndjson"))
        .unwrap()
        .is_empty());

    // A foreign kind or version alone is refused, F01's v1 included; v2
    // (F02) is read.
    for (key, foreign) in [
        ("kind", serde_json::json!("other-summary")),
        ("summary_version", serde_json::json!(1)),
        ("summary_version", serde_json::json!(SUMMARY_VERSION + 1)),
    ] {
        let mut changed = value.clone();
        changed[key] = foreign;
        let path = scratch.lab.join(format!("foreign-{key}.json"));
        std::fs::write(&path, serde_json::to_vec(&changed).unwrap()).unwrap();
        assert!(read_summary(&path).is_err(), "foreign {key} accepted");
    }
}

#[test]
fn timing_counts_gate_and_campaign_creature_ticks() {
    let scratch = Scratch::new("timing");
    let mut gate_only = tiny(&scratch.lab.join("gate"));
    gate_only.calibrate_only = true;
    let gate = scratch.run(&gate_only).unwrap().summary.timing;
    let full = scratch
        .run(&tiny(&scratch.lab.join("full")))
        .unwrap()
        .summary
        .timing;
    for timing in [&gate, &full] {
        let ms = timing.per_creature_tick_ms.expect("ticks ran");
        let expected = timing.wall_seconds * 1_000.0 / timing.creature_ticks as f64;
        assert!(
            (ms - expected).abs() <= 1e-9 * expected,
            "{ms} vs {expected}"
        );
    }
    // The campaign adds at most every evaluation's full lifetime: 5 genome
    // arms × population × scenes per generation, plus one validation pass
    // per reach-tested arm and generation. Readings add, per generation and
    // `changing` arm (native, shuffled-score), at most one scene pass per
    // ablated family (fewer than 22) and mutant.
    let p = tiny(&scratch.lab.join("unused"));
    let lifetime = u64::from(p.lifetime.unwrap());
    let generations = u64::from(p.replicates * p.generations);
    let bound = lifetime
        * generations
        * (5 * u64::from(p.population * p.scenes) + 4 * u64::from(p.validation_scenes));
    let readings_bound =
        lifetime * generations * 2 * (22 + u64::from(p.mutants)) * u64::from(p.scenes);
    assert_eq!(gate.readings_creature_ticks, 0);
    assert!(full.readings_creature_ticks > 0);
    assert!(full.readings_creature_ticks <= readings_bound);
    assert!(full.creature_ticks > gate.creature_ticks + full.readings_creature_ticks);
    assert!(
        full.creature_ticks <= gate.creature_ticks + full.readings_creature_ticks + bound,
        "{} > {} + {} + {bound}",
        full.creature_ticks,
        gate.creature_ticks,
        full.readings_creature_ticks
    );
}

#[test]
fn an_infeasible_grid_scores_nothing_and_reports_no_per_tick_cost() {
    let scratch = Scratch::new("infeasible");
    let mut params = tiny(&scratch.lab.join("run"));
    // round(0.999 × 48²) cells exceed the cells at distance ≥ 2.
    params.food_fraction = Some(0.999);
    let outcome = scratch.run(&params).unwrap();
    assert_eq!(outcome.exit_code, EXIT_UNCALIBRATED);
    assert!(outcome
        .summary
        .calibration
        .points
        .iter()
        .all(|p| !p.exposure));
    assert_eq!(outcome.summary.timing.creature_ticks, 0);
    assert_eq!(outcome.summary.timing.per_creature_tick_ms, None);
}

#[test]
fn overlay_bytes_join_the_summary_reserve() {
    let scratch = Scratch::new("reserve");
    let overlay = scratch.lab.join("hot.json");
    let text = r#"{"mutation": {"per_unit_rate": 0.02}}"#;
    std::fs::write(&overlay, text).unwrap();
    let reserve = SUMMARY_RESERVE + text.len() as u64;
    for (name, byte_cap, accepted) in [("below", 2 * reserve - 1, false), ("at", 2 * reserve, true)]
    {
        let out = scratch.lab.join(name);
        let mut params = tiny(&out);
        params.calibrate_only = true;
        params.byte_cap = byte_cap;
        params.arms = vec![UserArm {
            name: "hot".into(),
            overlay: overlay.clone(),
            genome: None,
        }];
        assert_eq!(scratch.run(&params).is_ok(), accepted, "cap {byte_cap}");
        assert_eq!(out.exists(), accepted);
    }
}

#[test]
fn out_outside_bench_artifacts_is_refused() {
    let scratch = Scratch::new("outside");
    let mut params = tiny(&scratch.root.join("elsewhere"));
    params.calibrate_only = true;
    assert!(scratch.run(&params).is_err());
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
        spec: SceneSpec::sparse(48, 0.06, 5),
        threshold: 1e9,
        mutants: 2,
        signature_arms: SignatureArms::Changing,
    };
    let validation = draw_scenes(11, 2, &SceneSpec::sparse(48, 0.06, 5)).unwrap();
    // Room for a few rows only: rows and elites get cap − reserve bytes.
    let mut dir =
        RunDir::create(scratch.lab.join("run"), Budget::new(12_000, 4_000).unwrap()).unwrap();
    let (arms, totals) = run_campaign(&arms, &plan, &validation, &mut dir).unwrap();
    assert!(totals.byte_cap_hit);
    let rows = std::fs::read(scratch.lab.join("run/rows.ndjson")).unwrap();
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

    // Projections come from written rows only: each replicate's `first` and
    // `last` are its first and last written rows' readings, a replicate
    // with none written (the refused row's included) has null.
    let rows: Vec<serde_json::Value> = String::from_utf8(rows)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    let mut nulls = 0;
    for arm in &arms {
        for replicate in &arm.replicates {
            let written: Vec<&serde_json::Value> = rows
                .iter()
                .filter(|row| {
                    row["arm"] == arm.name.as_str() && row["replicate"] == replicate.replicate
                })
                .collect();
            let projections = serde_json::to_value(&replicate.readings).unwrap();
            match (written.first(), written.last()) {
                (Some(first), Some(last)) => {
                    let shape = &first["readings"]["shape"];
                    assert_eq!(projections["first"]["genome_size"], shape["genome_size"]);
                    assert_eq!(projections["first"]["births"], shape["ancestry"]["births"]);
                    assert_eq!(projections["last"]["shape"], last["readings"]["shape"]);
                }
                _ => {
                    nulls += 1;
                    assert!(projections["first"].is_null() && projections["last"].is_null());
                }
            }
        }
    }
    assert!(nulls > 0, "some arm and replicate wrote no row");
}

#[test]
fn validation_evaluations_count_toward_creature_ticks() {
    let scratch = Scratch::new("validation-ticks");
    let reference = resolve_arm(None, 48, 100.0).unwrap();
    let founder = v3_core::creature::founder::founder_genome_with_age_gate(
        reference.population.founder_profile,
        &reference.energy.lifecycle,
    );
    let setup = Setup::new(reference, 100.0, 60);
    let arms = vec![Arm {
        name: "founder-only".into(),
        role: Role::Control,
        policy: Policy::Native,
        setup: setup.clone(),
        kind: ArmKind::Fixed(founder.clone()),
    }];
    let validation = draw_scenes(11, 3, &SceneSpec::sparse(48, 0.06, 5)).unwrap();
    let campaign = |threshold: f64, name: &str| {
        let plan = Plan {
            seed: 3,
            replicates: 1,
            generations: 1,
            population: 2,
            elite_fraction: 0.5,
            scenes: 2,
            spec: SceneSpec::sparse(48, 0.06, 5),
            threshold,
            mutants: 2,
            signature_arms: SignatureArms::Changing,
        };
        let mut dir = RunDir::create(
            scratch.lab.join(name),
            Budget::new(64 << 20, 1 << 20).unwrap(),
        )
        .unwrap();
        run_campaign(&arms, &plan, &validation, &mut dir).unwrap().1
    };
    // Threshold 0 tests reach once (one generation, one reach-tested arm);
    // an unreachable threshold never runs the validation scenes.
    let tested = campaign(0.0, "tested");
    let untested = campaign(1e9, "untested");
    let validation_ticks: u64 = validation
        .iter()
        .map(|scene| u64::from(evaluate_genome(&setup, &founder, scene).0.ticks))
        .sum();
    assert!(validation_ticks > 0);
    assert!(untested.creature_ticks > 0, "training ticks are counted");
    assert_eq!(
        tested.creature_ticks,
        untested.creature_ticks + validation_ticks
    );
}

#[test]
fn a_non_finite_reach_threshold_is_refused_before_any_output() {
    let scratch = Scratch::new("reach-threshold");
    for (index, value) in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY]
        .into_iter()
        .enumerate()
    {
        let out = scratch.lab.join(format!("run-{index}"));
        let mut params = tiny(&out);
        params.reach_threshold = Some(value);
        let error = scratch
            .run(&params)
            .expect_err("non-finite threshold accepted");
        assert!(error.to_string().contains("--reach-threshold"), "{error}");
        assert!(!out.exists(), "{value} created {}", out.display());
    }
}

#[test]
fn every_arm_genome_is_recorded_with_its_hash_and_format() {
    let scratch = Scratch::new("arm-genome");
    let overlay = scratch.lab.join("hot.json");
    std::fs::write(&overlay, r#"{"mutation": {"per_unit_rate": 0.02}}"#).unwrap();
    let reference = resolve_arm(None, 48, 100.0).unwrap();
    let founder = v3_core::creature::founder::founder_genome_with_age_gate(
        reference.population.founder_profile,
        &reference.energy.lifecycle,
    );
    let (genome, _) = v3_core::neighborhood::opportunity::controllers::controller(
        &founder,
        v3_core::neighborhood::opportunity::Family::Vector,
        false,
    );
    let genome_path = scratch.lab.join("hot-genome.json");
    std::fs::write(
        &genome_path,
        serde_json::to_vec(&GenomeFile::new(genome.clone())).unwrap(),
    )
    .unwrap();
    let mut params = tiny(&scratch.lab.join("run"));
    params.calibrate_only = true;
    params.arms = vec![
        UserArm {
            name: "hot".into(),
            overlay: overlay.clone(),
            genome: Some(genome_path),
        },
        UserArm {
            name: "plain".into(),
            overlay,
            genome: None,
        },
    ];
    let outcome = scratch.run(&params).unwrap();
    let genomes = &outcome.summary.provenance.genomes;
    let record = genomes
        .iter()
        .find(|g| g.name == "arm:hot")
        .expect("the arm genome is recorded under its arm");
    assert_eq!(
        record.sha256,
        sha256_hex(&serde_json::to_vec(&genome).unwrap())
    );
    assert_eq!(record.genome_format, GENOME_FORMAT);
    assert_eq!(record.v3_core_version, env!("CARGO_PKG_VERSION"));
    let names: Vec<&str> = genomes.iter().map(|g| g.name.as_str()).collect();
    assert_eq!(names, ["start", "comparator", "arm:hot"]);
}

#[test]
fn an_injected_root_without_a_checkout_records_git_unavailable() {
    let scratch = Scratch::new("git-unavailable");
    let mut params = tiny(&scratch.lab.join("run"));
    params.calibrate_only = true;
    let outcome = scratch.run(&params).unwrap();
    let text = std::fs::read_to_string(outcome.dir.join("summary.json")).unwrap();
    let value: serde_json::Value = serde_json::from_str(&text).unwrap();
    let provenance = &value["provenance"];
    assert_eq!(provenance["git"], "unavailable");
    assert!(provenance["git_revision"].is_null() && provenance["dirty"].is_null());
    // A checkout's provenance (the CLI's case) carries no `git` key.
    let mut summary = read_summary(&outcome.dir.join("summary.json")).unwrap();
    (
        summary.provenance.git_revision,
        summary.provenance.dirty,
        summary.provenance.git,
    ) = GitProvenance::Checkout {
        revision: "abc".into(),
        dirty: false,
    }
    .fields();
    let value = serde_json::to_value(&summary).unwrap();
    assert!(value["provenance"].get("git").is_none());
    assert_eq!(value["provenance"]["git_revision"], "abc");
}

#[test]
fn the_cli_refuses_to_run_outside_a_checkout() {
    let scratch = Scratch::new("cli-no-checkout");
    let output = Command::new(env!("CARGO_BIN_EXE_v3-lab"))
        .args(["run", "--quick", "--calibrate-only"])
        .current_dir(&scratch.root)
        // Stop git's discovery at the temp directory even if it sits in a repo.
        .env("GIT_CEILING_DIRECTORIES", std::env::temp_dir())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("not inside a git checkout"));
    assert_eq!(std::fs::read_dir(&scratch.lab).unwrap().count(), 0);
}

/// `tiny` on a built-in barrier arena at scale 1.
fn tiny_barrier(out: &Path, arena: ArenaId) -> RunParams {
    RunParams {
        assay: Assay::BarrierNavigation,
        arena: Some(arena),
        food_fraction: None,
        scale: Some(1),
        // Short enough that the half-seeker trails the oracle on the 3 × 3
        // food block (both exhaust it by about 60 ticks).
        lifetime: Some(15),
        ..tiny(out)
    }
}

/// The summary fields every barrier-navigation run carries.
fn assert_barrier_summary(outcome: &RunOutcome, arena: &str) {
    let summary = &outcome.summary;
    assert_eq!(summary.summary_version, SUMMARY_VERSION);
    assert_eq!(summary.assay, Assay::BarrierNavigation);
    assert_eq!(summary.provenance.sizes.blocked_weight, 1.0);
    assert_eq!(summary.provenance.sizes.efficiency_weight, 1.0);
    assert_eq!(summary.provenance.arena.spec["id"], arena);
    assert_eq!(summary.provenance.arena.spec["arena_version"], 1);
    let report = render_report(&read_summary(&outcome.dir.join("summary.json")).unwrap());
    assert!(
        report.contains("blocked weight 1, efficiency weight 1"),
        "{report}"
    );
    assert!(report.contains(
        "| floor blocked | comparator blocked | floor efficiency | half efficiency | oracle efficiency | comparator efficiency |"
    ));
}

#[test]
fn a_wall_run_calibrates_evolves_and_is_byte_identical_across_threads() {
    let scratch = Scratch::new("barrier-wall");
    let one = scratch
        .run(&tiny_barrier(&scratch.lab.join("one"), ArenaId::WallV1))
        .unwrap();
    assert_eq!(one.exit_code, 0, "{}", render_report(&one.summary));
    assert_barrier_summary(&one, "wall-v1");
    let summary = &one.summary;
    let selected = summary.calibration.selected.as_ref().unwrap();
    assert_eq!((selected.food_fraction, selected.scale), (None, Some(1)));
    let sizes = &summary.provenance.sizes;
    assert_eq!((sizes.scale, sizes.food_fraction), (Some(1), None));
    assert_eq!(summary.provenance.arena.spec["scale"], 1);
    assert_eq!(
        summary.fidelity.as_ref().unwrap().lifetime_learning,
        LifetimeLearning::Masked
    );
    assert!(render_report(summary).contains("| scale | lifetime |"));

    let mut params = tiny_barrier(&scratch.lab.join("three"), ArenaId::WallV1);
    params.threads = 3;
    let three = scratch.run(&params).unwrap();
    let rows = |outcome: &RunOutcome| std::fs::read(outcome.dir.join("rows.ndjson")).unwrap();
    assert!(!rows(&one).is_empty());
    assert_eq!(
        rows(&one),
        rows(&three),
        "rows differ between 1 and 3 threads"
    );
}

#[test]
fn a_ring_run_records_its_gate_on_the_scale_axis() {
    let scratch = Scratch::new("barrier-ring");
    let mut params = tiny_barrier(&scratch.lab.join("run"), ArenaId::RingV1);
    params.calibrate_only = true;
    let outcome = scratch.run(&params).unwrap();
    // The composed comparator's competence on the ring is the gate's
    // finding, not the fixture's: either verdict is a complete run.
    assert!([0, EXIT_UNCALIBRATED].contains(&outcome.exit_code));
    assert_barrier_summary(&outcome, "ring-v1");
    let [point] = &outcome.summary.calibration.points[..] else {
        panic!("one scale × one lifetime");
    };
    assert_eq!((point.food_fraction, point.scale), (None, Some(1)));
    assert!(point.exposure && point.means.is_some());
}

/// Write a 16-row layout of `.` with `cells` (x, y, char) set.
fn write_layout(path: &Path, cells: &[(usize, usize, char)]) {
    let mut grid = vec![vec!['.'; 16]; 16];
    for &(x, y, c) in cells {
        grid[y][x] = c;
    }
    let rows: Vec<String> = grid.into_iter().map(String::from_iter).collect();
    let file = serde_json::json!({"arena_format": 1, "rows": rows});
    std::fs::write(path, serde_json::to_vec(&file).unwrap()).unwrap();
}

fn layout_params(out: &Path, layout: &Path) -> RunParams {
    RunParams {
        layout: Some(layout.to_path_buf()),
        arena: None,
        arena_size: None,
        scale: None,
        ..tiny_barrier(out, ArenaId::WallV1)
    }
}

#[test]
fn a_layout_file_is_the_arena_and_its_descriptor() {
    let scratch = Scratch::new("barrier-layout");
    let path = scratch.lab.join("wall.json");
    // A wall at x = 6 between the start (4, 8) and food (8, 8).
    let cells: Vec<(usize, usize, char)> = (5..=11)
        .map(|y| (6, y, '#'))
        .chain([(4, 8, 'S'), (8, 8, 'F'), (8, 9, 'F')])
        .collect();
    write_layout(&path, &cells);
    let mut params = layout_params(&scratch.lab.join("run"), &path);
    params.calibrate_only = true;
    let outcome = scratch.run(&params).unwrap();
    assert!([0, EXIT_UNCALIBRATED].contains(&outcome.exit_code));
    assert_barrier_summary(&outcome, "layout");
    let spec = &outcome.summary.provenance.arena.spec;
    assert_eq!(spec["arena_format"], 1);
    assert_eq!(spec["size"], 16);
    assert_eq!(spec["path"], path.display().to_string());
    assert_eq!(spec["rows"][8], "....S.#.F.......");
    assert_eq!(outcome.summary.provenance.sizes.arena_size, 16);
    let [point] = &outcome.summary.calibration.points[..] else {
        panic!("one layout point × one lifetime");
    };
    assert_eq!((point.food_fraction, point.scale), (None, None));
    assert!(point.exposure && point.means.is_some());
    assert!(render_report(&outcome.summary).contains("| point | lifetime |"));

    // `--arena-size` with a layout is a config error.
    params.arena_size = Some(48);
    assert!(matches!(scratch.run(&params), Err(LabError::Config(_))));
}

#[test]
fn a_layout_with_unreachable_food_is_not_exposed() {
    let scratch = Scratch::new("barrier-enclosed");
    let path = scratch.lab.join("enclosed.json");
    // Food at (8, 8) inside a closed ring of barriers.
    let cells: Vec<(usize, usize, char)> = (7..=9)
        .flat_map(|y| (7..=9).map(move |x| (x, y)))
        .filter(|&(x, y)| (x, y) != (8, 8))
        .map(|(x, y)| (x, y, '#'))
        .chain([(4, 8, 'S'), (8, 8, 'F')])
        .collect();
    write_layout(&path, &cells);
    let outcome = scratch
        .run(&layout_params(&scratch.lab.join("run"), &path))
        .unwrap();
    assert_eq!(outcome.exit_code, EXIT_UNCALIBRATED);
    let points = &outcome.summary.calibration.points;
    assert!(!points.is_empty());
    assert!(points.iter().all(|p| !p.exposure && p.means.is_none()));
    assert_eq!(points[0].redraws, [0], "a layout has no redraw");
    assert!(outcome.summary.arms.is_empty());
    assert_eq!(outcome.summary.timing.creature_ticks, 0);
}

#[test]
fn a_v2_summary_renders_without_readings_and_a_v3_summary_renders_them() {
    // The T22.F02 wall summary as committed at df2ddbe3 (summary_version 2).
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/summary-v2-wall.json");
    let v2 = read_summary(&fixture).unwrap();
    assert_eq!(v2.summary_version, 2);
    let report = render_report(&v2);
    assert!(report.contains("## Arms") && !report.contains("## Readings"));

    let scratch = Scratch::new("v3-report");
    let outcome = scratch.run(&tiny(&scratch.lab.join("run"))).unwrap();
    let v3 = read_summary(&outcome.dir.join("summary.json")).unwrap();
    assert_eq!(v3.summary_version, SUMMARY_VERSION);
    let report = render_report(&v3);
    assert!(report.contains("## Readings"), "{report}");
    assert!(report.contains("### native replicate 0"));
    assert!(report.contains("| FoodHere:0 |"));
    assert!(report.contains("signature: 2 mutants"));
    // founder-only is outside `changing`; random-walk has no readings.
    let founder_only = report.split("### founder-only replicate 0").nth(1).unwrap();
    assert!(founder_only.contains("signature: not computed"));
    assert!(report.contains("| random-walk | 0 | - |"));
}
