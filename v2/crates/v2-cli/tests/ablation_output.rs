use serde_json::Value;
use v2_cli::commands::run_ablation;

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
fn ablation_events_follow_expected_sequence() {
    let lines = run_ablation(200, 7, vec!["baseline".to_string(), "no_vm".to_string()]);
    let records = parse(&lines);
    assert_eq!(records[0]["event_type"], "ablation_started");
    assert_eq!(records[1]["event_type"], "ablation_result");
    assert_eq!(records[2]["event_type"], "ablation_result");
    assert_eq!(records[3]["event_type"], "ablation_completed");

    for record in records {
        assert_eq!(record["protocol_version"], "v2alpha1");
    }
}

#[test]
fn ablation_result_has_canonical_fields_and_no_unknowns() {
    let lines = run_ablation(50, 1, vec!["single".to_string()]);
    let records = parse(&lines);
    let result = records
        .iter()
        .find(|record| record["event_type"] == "ablation_result")
        .expect("ablation_result");

    let object = result.as_object().expect("object");
    for key in [
        "protocol_version",
        "event_type",
        "preset",
        "score",
        "final_population",
        "final_mean_energy",
    ] {
        assert!(object.contains_key(key), "missing key {key}");
    }

    let line = lines
        .iter()
        .find(|line| line.contains("\"event_type\":\"ablation_result\""))
        .expect("ablation result line");
    assert_key_order(
        line,
        &[
            "protocol_version",
            "event_type",
            "preset",
            "score",
            "final_population",
            "final_mean_energy",
        ],
    );
}
