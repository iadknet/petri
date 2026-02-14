use serde_json::Value;
use v2_cli::commands::run_simulation;

fn parse(lines: &[String]) -> Vec<Value> {
    lines
        .iter()
        .map(|line| serde_json::from_str::<Value>(line).expect("valid json"))
        .collect()
}

fn assert_key_order(line: &str, keys: &[&str]) {
    let mut previous = 0;
    for key in keys {
        let token = format!("\"{key}\":");
        let index = line.find(&token).expect("key should exist");
        assert!(
            index >= previous,
            "key {key} appeared out of order in {line}"
        );
        previous = index;
    }
}

#[test]
fn run_command_emits_protocol_versioned_ndjson_events() {
    let lines = run_simulation(10, 4, 123);
    assert!(lines.len() >= 3);
    let records = parse(&lines);

    assert_eq!(records[0]["event_type"], "run_started");
    assert_eq!(records[records.len() - 1]["event_type"], "run_completed");

    for record in records {
        assert_eq!(record["protocol_version"], "v2alpha1");
    }
}

#[test]
fn tick_sample_schema_has_only_required_top_level_fields() {
    let lines = run_simulation(9, 3, 3);
    let records = parse(&lines);
    let tick = records
        .into_iter()
        .find(|record| record["event_type"] == "tick_sample")
        .expect("tick_sample present");

    let object = tick.as_object().expect("object");
    for key in [
        "protocol_version",
        "event_type",
        "tick",
        "population",
        "mean_energy",
        "births_last_window",
        "deaths_last_window",
        "last_action_counts",
    ] {
        assert!(object.contains_key(key), "missing key {key}");
    }

    let line = lines
        .iter()
        .find(|line| line.contains("\"event_type\":\"tick_sample\""))
        .expect("tick sample line");
    assert_key_order(
        line,
        &[
            "protocol_version",
            "event_type",
            "tick",
            "population",
            "mean_energy",
            "births_last_window",
            "deaths_last_window",
            "last_action_counts",
        ],
    );
}
