//! Server run telemetry (T21.F01): run identity across `startup`, the
//! lifecycle records, the bounded shutdown flush, and neutrality of the
//! simulation payloads with telemetry off, on, and on with a closed port.
#![cfg(feature = "telemetry")]

use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;
use v3_core::config::SimulationConfig;
use v3_server::{router, state::AppState};
use v3_telemetry::testing::{closed_endpoint, parse_run_line, ReceivedRecord, Receiver};
use v3_telemetry::{Limits, Options, ReportSink, Service, Telemetry};

fn test_config() -> SimulationConfig {
    let mut config = SimulationConfig::default();
    config.world.width = 64;
    config.world.height = 64;
    config.population.initial_creatures = 64;
    config.population.max_creatures = 512;
    config
}

fn telemetry(endpoint: &str) -> (Telemetry, ReportSink) {
    let reports = ReportSink::capture();
    let telemetry = Telemetry::start_with(Options {
        service: Service::Server,
        endpoint: endpoint.to_owned(),
        limits: Limits::default(),
        reports: reports.clone(),
    });
    (telemetry, reports)
}

fn request(method: &str, uri: &str, body: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(body.to_owned()))
        .unwrap()
}

async fn call(app: &axum::Router, method: &str, uri: &str, body: &str) -> serde_json::Value {
    let response = app
        .clone()
        .oneshot(request(method, uri, body))
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let value = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
    assert_eq!(status, StatusCode::OK, "{method} {uri}: {value}");
    value
}

const GRAZING_PATCH: &str = r#"{"world":{"food":{"shared":{"grazing":{"enabled":false,"factor":0.25,"floor":0.1,"recovery_ticks":250}}}}}"#;

fn run_lines(reports: &ReportSink) -> BTreeMap<String, BTreeMap<String, String>> {
    reports
        .lines()
        .iter()
        .filter_map(|line| parse_run_line(line))
        .map(|fields| (fields["run"].clone(), fields))
        .collect()
}

/// Each run's record names, in the order the run emitted them.
fn by_run(records: &[ReceivedRecord]) -> Vec<(String, Vec<String>)> {
    let mut runs: Vec<(String, Vec<String>)> = Vec::new();
    for record in records {
        let id = record.attribute("petri.run_id").unwrap().to_owned();
        let name = match record.event_name.as_str() {
            "run.state" => format!("run.state:{}", record.attribute("petri.state").unwrap()),
            "run.ended" => format!("run.ended:{}", record.attribute("petri.status").unwrap()),
            other => other.to_owned(),
        };
        match runs.iter_mut().find(|(run, _)| *run == id) {
            Some((_, names)) => names.push(name),
            None => runs.push((id, vec![name])),
        }
    }
    runs
}

#[tokio::test]
async fn each_seeding_is_a_run_and_each_reset_ends_the_previous_one() {
    let receiver = Receiver::start();
    let (telemetry, reports) = telemetry(receiver.endpoint());
    let state = AppState::from_config_with_telemetry(test_config(), 0, telemetry);
    let app = router(state.clone());

    call(&app, "POST", "/v3/simulation/startup", r#"{"seed":1}"#).await;
    call(&app, "POST", "/v3/simulation/start", "").await;
    call(&app, "POST", "/v3/simulation/pause", "").await;
    call(&app, "PATCH", "/v3/simulation/config", GRAZING_PATCH).await;
    call(&app, "POST", "/v3/simulation/startup", r#"{"seed":2}"#).await;
    state.shutdown_telemetry().await;

    let records = receiver.records();
    let runs = by_run(&records);
    let expected: [&[&str]; 3] = [
        &["run.started", "run.ended:reset"],
        &[
            "run.started",
            "run.state:running",
            "run.state:paused",
            "run.config",
            "run.ended:reset",
        ],
        &["run.started", "run.state:idle", "run.ended:shutdown"],
    ];
    assert_eq!(runs.len(), 3, "{runs:?}");
    for ((_, names), expected) in runs.iter().zip(expected) {
        assert_eq!(names, expected);
    }
    let seeds: Vec<&str> = records
        .iter()
        .filter(|record| record.event_name == "run.started")
        .map(|record| record.attribute("petri.seed").unwrap())
        .collect();
    assert_eq!(seeds, ["0", "1", "2"]);
    assert!(records
        .iter()
        .all(|record| record.resource["service.name"] == "v3-server"));
    let config = records
        .iter()
        .find(|record| record.event_name == "run.config")
        .unwrap();
    let patched: SimulationConfig =
        serde_json::from_str(config.body.as_deref().unwrap()).expect("body is the config");
    assert!(!patched.world.food.shared.grazing.enabled);
    assert_eq!(
        config.attribute("petri.config_digest"),
        Some(v3_core::config::config_digest(&patched).as_str())
    );

    let lines = run_lines(&reports);
    for ((run, names), _) in runs.iter().zip(expected) {
        let line = &lines[run];
        assert_eq!(line["exported"], names.len().to_string(), "{line:?}");
        assert_eq!(line["failed"], "0");
    }
}

#[tokio::test]
async fn shutdown_with_pending_records_ends_within_10_s_with_exact_counts() {
    let receiver = Receiver::hanging();
    let (telemetry, reports) = telemetry(receiver.endpoint());
    let state = AppState::from_config_with_telemetry(test_config(), 0, telemetry);
    let app = router(state.clone());
    for seed in 1..=3 {
        call(
            &app,
            "POST",
            "/v3/simulation/startup",
            &format!(r#"{{"seed":{seed}}}"#),
        )
        .await;
    }
    let began = Instant::now();
    state.shutdown_telemetry().await;
    let elapsed = began.elapsed();
    assert!(elapsed <= Duration::from_millis(10_500), "{elapsed:?}");

    let lines = run_lines(&reports);
    assert_eq!(lines.len(), 4);
    // Every record is pending when shutdown starts (the receiver never
    // answers); each ends failed (request timeout) or abandoned (flush bound).
    for line in lines.values() {
        let count = |key: &str| line[key].parse::<u64>().unwrap();
        assert_eq!(count("exported"), 0);
        assert_eq!(count("dropped"), 0);
        assert_eq!(count("failed") + count("abandoned"), 2, "{line:?}");
    }
}

/// Status, snapshot and creature payloads after `steps` single steps from a
/// seeded `startup`, keys sorted and `perf` timings removed.
async fn payloads(state: AppState, steps: usize) -> Vec<String> {
    fn strip_perf(value: &mut serde_json::Value) {
        match value {
            serde_json::Value::Object(map) => {
                map.remove("perf");
                map.values_mut().for_each(strip_perf);
            }
            serde_json::Value::Array(items) => items.iter_mut().for_each(strip_perf),
            _ => {}
        }
    }
    let app = router(state.clone());
    call(&app, "POST", "/v3/simulation/startup", r#"{"seed":5}"#).await;
    call(&app, "POST", "/v3/simulation/start", "").await;
    let paused = call(&app, "POST", "/v3/simulation/pause", "").await;
    assert_eq!(paused["tick"], 0, "the run loop ticked before the pause");
    let mut out = Vec::new();
    for _ in 0..steps {
        call(&app, "POST", "/v3/simulation/step", r#"{"steps":1}"#).await;
        let mut values = vec![
            call(&app, "GET", "/v3/simulation/status", "").await,
            call(&app, "GET", "/v3/simulation/snapshot?zoom_tier=detail", "").await,
        ];
        let ids: Vec<u64> = values[1]["view"]["creatures"]
            .as_array()
            .unwrap()
            .iter()
            .take(3)
            .map(|creature| creature["id"].as_u64().unwrap())
            .collect();
        for id in ids {
            values.push(call(&app, "GET", &format!("/v3/simulation/creature/{id}"), "").await);
        }
        for mut value in values {
            strip_perf(&mut value);
            out.push(serde_json::to_string(&value).unwrap());
        }
    }
    state.shutdown_telemetry().await;
    out
}

#[tokio::test]
async fn simulation_payloads_are_identical_off_on_and_on_with_a_closed_port() {
    const STEPS: usize = 5;
    let off = payloads(AppState::from_config(test_config(), 0), STEPS).await;

    let receiver = Receiver::start();
    let (on_telemetry, _) = telemetry(receiver.endpoint());
    let on = payloads(
        AppState::from_config_with_telemetry(test_config(), 0, on_telemetry),
        STEPS,
    )
    .await;
    assert!(receiver
        .records()
        .iter()
        .any(|record| record.event_name == "run.state"));

    let (closed_telemetry, _) = telemetry(&closed_endpoint());
    let closed = payloads(
        AppState::from_config_with_telemetry(test_config(), 0, closed_telemetry),
        STEPS,
    )
    .await;

    assert_eq!(on, off);
    assert_eq!(closed, off);
}
