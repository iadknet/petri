//! Telemetry neutrality (T21.F01): one seeded run's canonical NDJSON is the same
//! with telemetry off, on with a receiver listening, and on with a closed port.
#![cfg(feature = "telemetry")]

use std::process::Command;

use v3_telemetry::testing::{closed_endpoint, parse_run_line, Receiver};

struct RunOutput {
    canonical: Vec<String>,
    stderr: String,
}

/// Each NDJSON line re-serialized with keys sorted (`serde_json::Value`
/// objects are ordered maps) and values untouched.
fn canonical(stdout: &[u8]) -> Vec<String> {
    String::from_utf8(stdout.to_vec())
        .expect("stdout is UTF-8")
        .lines()
        .map(|line| {
            let value: serde_json::Value = serde_json::from_str(line).expect("NDJSON line");
            serde_json::to_string(&value).expect("value serializes")
        })
        .collect()
}

fn run(switch: &str, endpoint: Option<&str>) -> RunOutput {
    let recipe = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("t21-neutrality.json");
    std::fs::write(
        &recipe,
        r#"{"world":{"width":32,"height":32},"population":{"initial_creatures":12}}"#,
    )
    .expect("write recipe");
    let mut command = Command::new(env!("CARGO_BIN_EXE_v3-cli"));
    command
        .args(["--telemetry", switch, "run", "--seed", "7", "--ticks", "30"])
        .args(["--sample-every", "10", "--config"])
        .arg(&recipe)
        .env_remove(v3_telemetry::SWITCH_ENV)
        .env_remove(v3_telemetry::ENDPOINT_ENV);
    if let Some(endpoint) = endpoint {
        command.env(v3_telemetry::ENDPOINT_ENV, endpoint);
    }
    let output = command.output().expect("v3-cli runs");
    assert!(output.status.success(), "{output:?}");
    RunOutput {
        canonical: canonical(&output.stdout),
        stderr: String::from_utf8(output.stderr).expect("stderr is UTF-8"),
    }
}

/// The run ID from the `telemetry: on … run=<id>` line.
fn announced_run(stderr: &str) -> String {
    let line = stderr
        .lines()
        .find(|line| line.starts_with("telemetry: on "))
        .expect("the start line");
    line.rsplit_once(" run=").expect("run field").1.to_owned()
}

fn run_line(stderr: &str) -> std::collections::BTreeMap<String, String> {
    stderr
        .lines()
        .find_map(parse_run_line)
        .expect("the run's line")
}

#[test]
fn canonical_output_is_identical_off_on_and_on_with_a_closed_port() {
    let off = run("off", None);
    assert!(!off.stderr.contains("telemetry:"), "{}", off.stderr);
    assert!(off.canonical.len() >= 3);

    let receiver = Receiver::start();
    let on = run("on", Some(receiver.endpoint()));
    let run_id = announced_run(&on.stderr);
    let records = receiver.records();
    let names: Vec<&str> = records
        .iter()
        .filter(|record| record.attribute("petri.run_id") == Some(run_id.as_str()))
        .map(|record| record.event_name.as_str())
        .collect();
    assert_eq!(names, ["run.started", "run.ended"]);
    assert_eq!(records[0].resource["service.name"], "v3-cli");
    assert_eq!(records[0].attribute("petri.ticks_requested"), Some("30"));
    assert_eq!(records[1].attribute("petri.status"), Some("completed"));
    assert_eq!(records[1].attribute("petri.tick"), Some("30"));
    let line = run_line(&on.stderr);
    assert_eq!(line["run"], run_id);
    assert_eq!(line["exported"], "2");

    let closed = run("on", Some(&closed_endpoint()));
    let line = run_line(&closed.stderr);
    assert_eq!(line["exported"], "0");
    assert_eq!(line["failed"], "2");

    assert_eq!(on.canonical, off.canonical);
    assert_eq!(closed.canonical, off.canonical);
}
