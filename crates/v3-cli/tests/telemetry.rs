//! Telemetry neutrality (T21.F01): one seeded run's canonical NDJSON is the same
//! with telemetry off, on with a receiver listening, and on with a closed port;
//! and the run's snapshots (T21.F02) agree with its final `tick_sample`.
#![cfg(feature = "telemetry")]

use std::process::Command;

use v3_telemetry::testing::{closed_endpoint, parse_run_line, PointValue, Receiver};

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

/// A run's shape: its recipe and its tick and sample counts.
#[derive(Clone, Copy)]
struct Shape {
    name: &'static str,
    recipe: &'static str,
    ticks: u64,
    sample_every: u64,
}

const SMALL: Shape = Shape {
    name: "small",
    recipe: r#"{"world":{"width":32,"height":32},"population":{"initial_creatures":12}}"#,
    ticks: 30,
    sample_every: 10,
};

/// A world whose population is still alive at its last tick.
const LIVING: Shape = Shape {
    name: "living",
    recipe: r#"{"world":{"width":64,"height":64},"population":{"initial_creatures":64}}"#,
    ticks: 100,
    sample_every: 50,
};

/// One `v3-cli run` of `shape` with the telemetry `switch`, the OTLP
/// `endpoint` and the metrics `interval`.
fn command(
    switch: &str,
    endpoint: Option<&str>,
    interval: Option<&str>,
    shape: Shape,
) -> std::process::Output {
    let recipe =
        std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("t21-{}.json", shape.name));
    std::fs::write(&recipe, shape.recipe).expect("write recipe");
    let (ticks, sample_every) = (shape.ticks, shape.sample_every);
    let mut command = Command::new(env!("CARGO_BIN_EXE_v3-cli"));
    command
        .args(["--telemetry", switch, "run", "--seed", "7"])
        .args(["--ticks", &ticks.to_string()])
        .args(["--sample-every", &sample_every.to_string(), "--config"])
        .arg(&recipe)
        .env_remove(v3_telemetry::SWITCH_ENV)
        .env_remove(v3_telemetry::ENDPOINT_ENV)
        .env_remove(v3_telemetry::METRICS_INTERVAL_ENV);
    if let Some(endpoint) = endpoint {
        command.env(v3_telemetry::ENDPOINT_ENV, endpoint);
    }
    if let Some(interval) = interval {
        command.env(v3_telemetry::METRICS_INTERVAL_ENV, interval);
    }
    command.output().expect("v3-cli runs")
}

fn run_with(
    switch: &str,
    endpoint: Option<&str>,
    interval: Option<&str>,
    shape: Shape,
) -> RunOutput {
    let output = command(switch, endpoint, interval, shape);
    assert!(output.status.success(), "{output:?}");
    RunOutput {
        canonical: canonical(&output.stdout),
        stderr: String::from_utf8(output.stderr).expect("stderr is UTF-8"),
    }
}

fn run(switch: &str, endpoint: Option<&str>) -> RunOutput {
    run_with(switch, endpoint, None, SMALL)
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
    assert_eq!(
        records[0].attribute("petri.metrics_interval_ms"),
        Some("1000")
    );
    assert_eq!(records[1].attribute("petri.status"), Some("completed"));
    assert_eq!(records[1].attribute("petri.tick"), Some("30"));
    // Thirty ticks fall inside one 1,000 ms interval: the run-end snapshot is
    // the only one.
    let ticks: Vec<Option<u64>> = receiver
        .snapshots()
        .iter()
        .filter(|snapshot| snapshot.run_id() == Some(run_id.as_str()))
        .map(|snapshot| snapshot.tick())
        .collect();
    assert_eq!(ticks, [Some(30)]);
    let line = run_line(&on.stderr);
    assert_eq!(line["run"], run_id);
    assert_eq!(line["exported"], "3");
    assert_eq!(line["snapshots"], "1");

    let closed = run("on", Some(&closed_endpoint()));
    let line = run_line(&closed.stderr);
    assert_eq!(line["exported"], "0");
    assert_eq!(line["failed"], "3");

    assert_eq!(on.canonical, off.canonical);
    assert_eq!(closed.canonical, off.canonical);
}

#[test]
fn snapshots_every_10_ms_end_at_the_final_tick_sample() {
    const TICKS: u64 = LIVING.ticks;
    let off = run_with("off", None, None, LIVING);
    let receiver = Receiver::start();
    let on = run_with("on", Some(receiver.endpoint()), Some("10"), LIVING);
    assert_eq!(on.canonical, off.canonical);
    let run_id = announced_run(&on.stderr);
    let snapshots: Vec<_> = receiver
        .snapshots()
        .into_iter()
        .filter(|snapshot| snapshot.run_id() == Some(run_id.as_str()))
        .collect();
    assert!(snapshots.len() >= 2, "{} snapshots", snapshots.len());
    assert_eq!(
        run_line(&on.stderr)["snapshots"],
        snapshots.len().to_string()
    );
    for pair in snapshots.windows(2) {
        assert!(pair[0].tick() < pair[1].tick());
        let (earlier, later) = (
            pair[0].time_unix_nano().unwrap(),
            pair[1].time_unix_nano().unwrap(),
        );
        assert!(later >= earlier + 1_000_000, "{earlier} then {later}");
    }

    let last = snapshots.last().unwrap();
    assert_eq!(last.tick(), Some(TICKS));
    let sample: serde_json::Value = serde_json::from_str(
        off.canonical
            .iter()
            .rev()
            .nth(1)
            .expect("final tick_sample"),
    )
    .expect("JSON");
    assert_eq!(sample["event_type"], "tick_sample");
    assert_eq!(sample["tick"], TICKS);
    let gauge = |name: &str| last.value(name, &[]).unwrap().as_f64();
    assert!(
        sample["population"].as_u64().unwrap() > 0,
        "the census is not empty"
    );
    assert_eq!(
        gauge("petri.tick.population"),
        sample["population"].as_f64().unwrap()
    );
    assert_eq!(
        gauge("petri.tick.mean_energy") as f32,
        sample["mean_energy"].as_f64().unwrap() as f32
    );
    for field in ["mean_genome_size", "mean_mesh_nodes", "mean_generation"] {
        assert_eq!(
            gauge(&format!("petri.tick.{field}")),
            sample[field].as_f64().unwrap(),
            "{field}"
        );
    }
    for field in [
        "reproduction_actions_attempted",
        "reproduction_actions_spawned",
        "reproduction_actions_rejected",
        "mutation_events_attempted",
        "mutation_events_applied",
        "mutation_events_skipped",
    ] {
        assert_eq!(
            last.value(&format!("petri.run.{field}"), &[]),
            Some(PointValue::Int(
                sample[format!("{field}_total")].as_i64().unwrap()
            )),
            "{field}"
        );
    }
    for (family, key, attribute) in [
        (
            "mutation_events_attempted_by_domain",
            "mutation_events_attempted_total_by_domain",
            "petri.domain",
        ),
        (
            "mutation_events_applied_by_domain",
            "mutation_events_applied_total_by_domain",
            "petri.domain",
        ),
        (
            "mutation_events_attempted_by_operator",
            "mutation_events_attempted_total_by_operator",
            "petri.operator",
        ),
        (
            "mutation_events_applied_by_operator",
            "mutation_events_applied_total_by_operator",
            "petri.operator",
        ),
    ] {
        let exported: std::collections::BTreeMap<String, i64> = last
            .family(&format!("petri.run.{family}"))
            .map(|point| {
                let value = match point.value {
                    PointValue::Int(count) => count,
                    PointValue::Double(_) => panic!("{family} is an integer sum"),
                };
                (point.attribute(attribute).unwrap().to_owned(), value)
            })
            .collect();
        let sampled: std::collections::BTreeMap<String, i64> = sample[key]
            .as_object()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.clone(), v.as_i64().unwrap()))
            .collect();
        assert_eq!(exported, sampled, "{family}");
    }
}

#[test]
fn an_invalid_metrics_interval_refuses_to_start_a_run_with_telemetry_on() {
    for invalid in ["0", "9", "3600001", "soon"] {
        let output = command("on", Some(&closed_endpoint()), Some(invalid), SMALL);
        assert!(!output.status.success(), "{invalid}");
        assert!(output.stdout.is_empty(), "{invalid}");
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(
            stderr.contains(v3_telemetry::METRICS_INTERVAL_ENV),
            "{stderr}"
        );
    }
}
