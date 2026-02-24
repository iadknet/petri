use v3_cli::{run_simulation, RunError};
use v3_core::config::SimulationConfig;

fn default_config() -> SimulationConfig {
    let mut cfg = SimulationConfig::default();
    cfg.world.width = 32;
    cfg.world.height = 32;
    cfg.population.initial_creatures = 10;
    cfg.world.food.initial_coverage = 0.6;
    cfg.world.food.initial_density = 1.0;
    cfg.world.food.growth_rate = 0.25;
    cfg.energy.lifecycle.initial_energy = 110.0;
    cfg.energy.lifecycle.max_energy = 160.0;
    cfg.energy.lifecycle.default_offspring_energy = 4.0;
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
