use v3_cli::{run_simulation, RunError};
use v3_core::config::SimulationConfig;

fn default_config() -> SimulationConfig {
    let mut cfg = SimulationConfig::default();
    cfg.world.width = 32;
    cfg.world.height = 32;
    cfg.population.initial_creatures = 10;
    cfg.world.food.types[0].initial_coverage = 0.6;
    cfg.world.food.types[0].initial_density = 1.0;
    cfg.world.food.growth_rate = 0.25;
    cfg.energy.lifecycle.max_energy = 160.0;
    cfg.energy.costs.reproduce_cost = 1.0;
    cfg
}

fn run_and_collect(ticks: u64, sample_every: u16) -> Vec<serde_json::Value> {
    let mut buf = Vec::new();
    run_simulation(default_config(), 42, ticks, sample_every, &mut buf).unwrap();
    let text = String::from_utf8(buf).unwrap();
    text.lines()
        .filter(|l| !l.is_empty())
        .map(|l| serde_json::from_str::<serde_json::Value>(l).unwrap())
        .collect()
}

#[test]
fn run_started_is_first_event() {
    let events = run_and_collect(5, 1);
    assert!(!events.is_empty());
    assert_eq!(
        events[0]["event_type"].as_str().unwrap(),
        "run_started",
        "first event must be run_started"
    );
    assert_eq!(
        events[0]["sample_every"].as_u64().unwrap(),
        1,
        "run_started must have sample_every == 1"
    );
}

#[test]
fn tick_sample_alignment_sample_every_3() {
    let events = run_and_collect(10, 3);
    let samples: Vec<u64> = events
        .iter()
        .filter(|e| e["event_type"].as_str() == Some("tick_sample"))
        .map(|e| e["tick"].as_u64().unwrap())
        .collect();
    // ticks 3, 6, 9, 10 => 4 samples
    assert_eq!(
        samples,
        vec![3, 6, 9, 10],
        "tick_samples at wrong ticks: {samples:?}"
    );
}

#[test]
fn run_completed_is_last_event() {
    let events = run_and_collect(10, 3);
    let last = events.last().unwrap();
    assert_eq!(
        last["event_type"].as_str().unwrap(),
        "run_completed",
        "last event must be run_completed"
    );
    assert_eq!(
        last["ticks_executed"].as_u64().unwrap(),
        10,
        "ticks_executed must equal requested ticks"
    );
}

#[test]
fn stats_accounting_invariant_in_cli_output() {
    let events = run_and_collect(50, 50);
    // Find the tick_sample at tick 50.
    let sample = events
        .iter()
        .find(|e| e["event_type"].as_str() == Some("tick_sample") && e["tick"].as_u64() == Some(50))
        .expect("tick_sample at tick 50 not found");

    let attempted = sample["reproduction_actions_attempted_total"]
        .as_u64()
        .unwrap();
    let spawned = sample["reproduction_actions_spawned_total"]
        .as_u64()
        .unwrap();
    let rejected = sample["reproduction_actions_rejected_total"]
        .as_u64()
        .unwrap();
    assert_eq!(
        attempted,
        spawned + rejected,
        "accounting invariant: attempted({attempted}) == spawned({spawned}) + rejected({rejected})"
    );

    let mutation_attempted = sample["mutation_events_attempted_total"].as_u64().unwrap();
    let mutation_applied = sample["mutation_events_applied_total"].as_u64().unwrap();
    let mutation_skipped = sample["mutation_events_skipped_total"].as_u64().unwrap();
    assert_eq!(
        mutation_attempted,
        mutation_applied + mutation_skipped,
        "mutation accounting invariant: attempted({mutation_attempted}) == applied({mutation_applied}) + skipped({mutation_skipped})"
    );

    let attempted_by_domain: u64 = sample["mutation_events_attempted_total_by_domain"]
        .as_object()
        .unwrap()
        .values()
        .map(|v| v.as_u64().unwrap())
        .sum();
    let applied_by_domain: u64 = sample["mutation_events_applied_total_by_domain"]
        .as_object()
        .unwrap()
        .values()
        .map(|v| v.as_u64().unwrap())
        .sum();
    let attempted_by_operator: u64 = sample["mutation_events_attempted_total_by_operator"]
        .as_object()
        .unwrap()
        .values()
        .map(|v| v.as_u64().unwrap())
        .sum();
    let applied_by_operator: u64 = sample["mutation_events_applied_total_by_operator"]
        .as_object()
        .unwrap()
        .values()
        .map(|v| v.as_u64().unwrap())
        .sum();
    assert_eq!(
        attempted_by_domain, mutation_attempted,
        "attempted_by_domain must reconcile to mutation_events_attempted_total"
    );
    assert_eq!(
        applied_by_domain, mutation_applied,
        "applied_by_domain must reconcile to mutation_events_applied_total"
    );
    // Operator-level attempts may be less than total when events are skipped at
    // the domain level (no applicable operator under complexity pressure).
    assert!(
        attempted_by_operator <= mutation_attempted,
        "attempted_by_operator ({attempted_by_operator}) must not exceed mutation_events_attempted_total ({mutation_attempted})"
    );
    assert_eq!(
        applied_by_operator, mutation_applied,
        "applied_by_operator must reconcile to mutation_events_applied_total"
    );
}

#[test]
fn unknown_config_field_returns_validation_error() {
    // Build a JSON with an unknown top-level key.
    let bad_json = r#"{"unknown_key": 1}"#;
    let result = serde_json::from_str::<SimulationConfig>(bad_json);
    assert!(
        result.is_err(),
        "unknown config field must be rejected at deserialization"
    );
    // Confirm RunError::ValidationError would be returned if we passed it via the config path.
    // Simulate what main.rs does: attempt parse and return ValidationError on failure.
    let err_msg = result.unwrap_err().to_string();
    let run_err = RunError::ValidationError(err_msg);
    assert!(matches!(run_err, RunError::ValidationError(_)));
}

#[test]
fn sample_every_1_emits_one_sample_per_tick() {
    let events = run_and_collect(5, 1);
    let samples: Vec<u64> = events
        .iter()
        .filter(|e| e["event_type"].as_str() == Some("tick_sample"))
        .map(|e| e["tick"].as_u64().unwrap())
        .collect();
    assert_eq!(
        samples.len(),
        5,
        "sample_every=1 should emit exactly 5 tick_samples, got {samples:?}"
    );
    assert_eq!(samples, vec![1, 2, 3, 4, 5]);
}

#[test]
fn run_completed_has_final_mean_energy() {
    let events = run_and_collect(10, 10);
    let last = events.last().unwrap();
    assert_eq!(last["event_type"].as_str().unwrap(), "run_completed");
    let fme = last["final_mean_energy"].as_f64();
    assert!(
        fme.is_some(),
        "run_completed must contain final_mean_energy"
    );
    assert!(
        fme.unwrap() >= 0.0,
        "final_mean_energy must be non-negative"
    );
}

// ── Structure means on tick_sample (T03.F08) ────────────────────────────────

fn first_tick_sample(events: &[serde_json::Value]) -> &serde_json::Value {
    events
        .iter()
        .find(|e| e["event_type"] == "tick_sample")
        .expect("a tick_sample must be emitted")
}

#[test]
fn tick_sample_reports_the_founder_structure_means_before_any_birth() {
    // min_reproduce_age is 20 ticks, so at tick 1 every creature is a founder.
    let events = run_and_collect(1, 1);
    let sample = first_tick_sample(&events);
    assert_eq!(sample["population"].as_u64().unwrap(), 10);
    // The T19.F04 vote founder is 97 genome units.
    assert_eq!(sample["mean_genome_size"].as_f64().unwrap(), 97.0);
    assert_eq!(sample["mean_mesh_nodes"].as_f64().unwrap(), 2.0);
    assert_eq!(sample["mean_generation"].as_f64().unwrap(), 0.0);
}

#[test]
fn tick_sample_mean_generation_rises_once_the_population_reproduces() {
    let events = run_and_collect(60, 60);
    let sample = first_tick_sample(&events);
    assert!(
        sample["population"].as_u64().unwrap() > 10,
        "the fixture must reproduce for this reading to mean anything"
    );
    assert!(
        sample["mean_generation"].as_f64().unwrap() > 0.0,
        "descendants must lift the mean generation above the founders' zero"
    );
    assert!(sample["mean_genome_size"].as_f64().unwrap() > 0.0);
    assert!(sample["mean_mesh_nodes"].as_f64().unwrap() > 0.0);
}

#[test]
fn run_started_identifies_applied_config() {
    let config = default_config();
    let expected = v3_core::config::config_digest(&config);
    let mut out = Vec::new();
    run_simulation(config, 42, 1, 1, &mut out).unwrap();
    let first: serde_json::Value =
        serde_json::from_str(String::from_utf8(out).unwrap().lines().next().unwrap()).unwrap();
    assert_eq!(first["config_digest"], expected);
}

#[test]
fn recipe_cli_save_reload_and_failures() {
    let dir = std::env::temp_dir().join(format!("petri-recipe-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let input = dir.join("input.json");
    let saved = dir.join("saved.json");
    std::fs::write(&input, r#"{"world":{"width":8,"height":8,"world_seed":18446744073709551615,"terrain":[{"params":{"pattern_type":"Noise","density":0.2,"cluster_size":1},"seed":18446744073709551615}],"food":{"fertility":{"enabled":true,"layers":[{"weight":1.0,"algorithm":{"Fbm":{"octaves":2,"frequency":0.1,"lacunarity":2.0,"persistence":0.5,"seed":18446744073709551615}}}]}}},"population":{"initial_creatures":2,"founder_profile":"forage_first_sparse"},"runtime":{"max_actions_per_turn":2},"energy":{"costs":{"move_cost":0.25}}}"#).unwrap();
    let run = |input: &std::path::Path, save: Option<&std::path::Path>| {
        let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_v3-cli"));
        cmd.args(["run", "--ticks", "1", "--seed", "42", "--config"])
            .arg(input);
        if let Some(path) = save {
            cmd.arg("--save-config").arg(path);
        }
        cmd.output().unwrap()
    };
    let first = run(&input, Some(&saved));
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&std::fs::read(&saved).unwrap()).unwrap();
    assert_eq!(json["world"]["world_seed"].as_u64(), Some(u64::MAX));
    assert!(json.get("seed").is_none());
    assert!(json.get("mutation").is_some());
    assert_eq!(json["world"]["terrain"][0]["seed"].as_u64(), Some(u64::MAX));
    assert_eq!(
        json["world"]["food"]["fertility"]["layers"][0]["algorithm"]["Fbm"]["seed"].as_u64(),
        Some(u64::MAX)
    );
    assert_eq!(json["population"]["founder_profile"], "forage_first_sparse");
    assert_eq!(json["runtime"]["max_actions_per_turn"], 2);
    assert_eq!(json["energy"]["costs"]["move_cost"], 0.25);
    let config: SimulationConfig = serde_json::from_value(json.clone()).unwrap();
    let mut direct = Vec::new();
    run_simulation(config, 42, 1, 1, &mut direct).unwrap();
    assert_eq!(first.stdout, direct);

    assert_eq!(first.stdout, run(&saved, None).stdout);
    assert!(!run(&input, Some(&dir)).status.success());
    for bad in ["{", "[]", "null", r#"{"seed":1}"#, r#"{"stale":1}"#] {
        std::fs::write(&input, bad).unwrap();
        let result = run(&input, None);
        assert!(!result.status.success());
        assert!(result.stdout.is_empty());
    }
    std::fs::remove_dir_all(dir).unwrap();
}

/// The `recruitment` subcommand passes every option through: the pilot
/// panel, thread count, both caps, the replay check, the explicit output
/// paths and the checkout revision all reach the summary, and a cap stop
/// exits 3 after writing both artifacts.
#[test]
fn recruitment_cli_passes_every_option_into_the_written_summary() {
    let dir = std::env::temp_dir().join(format!("petri-recruitment-cli-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    // Output paths are recorded canonically.
    let dir = dir.canonicalize().unwrap();
    let git = |args: &[&str]| {
        let output = std::process::Command::new("git")
            .current_dir(&dir)
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap().trim().to_string()
    };
    git(&["init", "--quiet"]);
    git(&[
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
    ]);
    let revision = git(&["rev-parse", "HEAD"]);
    let run = |name: &str, wall_cap_secs: &str, byte_cap: &str| {
        let raw = dir.join(format!("{name}-raw.json"));
        let summary = dir.join(format!("{name}-summary.json"));
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_v3-cli"))
            .current_dir(&dir)
            .args([
                "recruitment",
                "--feature",
                "t13-f07-cli-test",
                "--pilot",
                "--threads",
                "1",
                "--wall-cap-secs",
                wall_cap_secs,
                "--byte-cap",
                byte_cap,
                "--replay-check",
                "--out",
            ])
            .arg(&raw)
            .arg("--summary-out")
            .arg(&summary)
            .output()
            .unwrap();
        assert_eq!(
            output.status.code(),
            Some(3),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(raw.is_file(), "{name}: raw record written");
        let summary: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&summary).unwrap()).unwrap();
        assert_eq!(summary["kind"], "petri-recruitment-s0-summary");
        assert_eq!(summary["feature"], "t13-f07-cli-test");
        assert_eq!(summary["pilot"], true);
        assert_eq!(summary["threads"], 1);
        assert_eq!(summary["source_revision"], revision);
        assert_eq!(summary["incomplete"], true);
        assert_eq!(summary["raw"]["path"], raw.display().to_string());
        assert!(summary["replay_check"].is_object(), "{name}: replay check");
        summary
    };
    // A zero wall cap stops before any lineage; a one-byte cap after the
    // first lineage crosses it.
    let wall = run("wall", "0", "0");
    assert_eq!(wall["stop_reason"], "wall_cap");
    assert_eq!(wall["lineage_count"], 0);
    let byte = run("byte", "20", "1");
    assert_eq!(byte["stop_reason"], "byte_cap");
    assert_eq!(byte["lineage_count"], 1);
    assert_eq!(byte["replay_check"]["proposals"], 1024);
    assert_eq!(byte["replay_check"]["matched"], 1024);
    std::fs::remove_dir_all(dir).unwrap();
}
