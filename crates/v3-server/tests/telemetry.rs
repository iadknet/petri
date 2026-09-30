//! Server run telemetry (T21.F01): run identity across `startup`, the
//! lifecycle records and their snapshots (T21.F02), the bounded shutdown
//! flush, and neutrality of the simulation payloads with telemetry off, on,
//! and on with a closed port.
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
        // Longer than any test: only transition and run-end snapshots.
        metrics_interval: Duration::from_secs(3_600),
        tick_traces: v3_telemetry::Switch::On,
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

    // The loop never ticks before the pause, so every run stays at tick 0:
    // the first run's snapshot is its run-end one, the second's is taken at
    // `running` (none at `paused` or at reset, the tick has one), the third's
    // at `idle` (none at shutdown).
    let snapshots = receiver.snapshots();
    let lines = run_lines(&reports);
    for (run, names) in &runs {
        let ticks: Vec<Option<u64>> = snapshots
            .iter()
            .filter(|snapshot| snapshot.run_id() == Some(run.as_str()))
            .map(|snapshot| snapshot.tick())
            .collect();
        assert_eq!(ticks, [Some(0)], "{run}");
        let line = &lines[run];
        assert_eq!(line["exported"], (names.len() + 1).to_string(), "{line:?}");
        assert_eq!(line["snapshots"], "1", "{line:?}");
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
        // `run.started`, `run.ended` and the run-end snapshot.
        assert_eq!(count("failed") + count("abandoned"), 3, "{line:?}");
        assert_eq!(count("snapshots"), 1, "{line:?}");
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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn shutdown_stops_the_running_simulation_at_the_tick_run_ended_reports() {
    let receiver = Receiver::start();
    let (telemetry, _) = telemetry(receiver.endpoint());
    let state = AppState::from_config_with_telemetry(test_config(), 0, telemetry);
    let app = router(state.clone());
    call(&app, "POST", "/v3/simulation/start", "").await;
    while state.sim.lock().await.sim.tick == 0 {
        tokio::task::yield_now().await;
    }

    state.shutdown_telemetry().await;
    // A loop still running would move the tick past the one reported.
    tokio::time::sleep(Duration::from_millis(100)).await;

    let ended = receiver
        .records()
        .into_iter()
        .find(|record| record.event_name == "run.ended")
        .expect("run.ended was exported");
    let handle = state.sim.lock().await;
    assert_ne!(handle.status, v3_server::state::SimulationStatus::Running);
    assert_eq!(
        ended.attribute("petri.tick"),
        Some(handle.sim.tick.to_string().as_str())
    );
}

/// The tick of a `step` of one tick.
async fn step(app: &axum::Router) -> u64 {
    call(app, "POST", "/v3/simulation/step", r#"{"steps":1}"#).await["tick"]
        .as_u64()
        .unwrap()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn interval_snapshots_carry_tick_traces_and_lifecycle_snapshots_do_not() {
    const INTERVAL: Duration = Duration::from_millis(400);
    let receiver = Receiver::start();
    let telemetry = Telemetry::start_with(Options {
        service: Service::Server,
        endpoint: receiver.endpoint().to_owned(),
        limits: Limits::default(),
        reports: ReportSink::capture(),
        metrics_interval: INTERVAL,
        tick_traces: v3_telemetry::Switch::On,
    });
    let mut config = test_config();
    config.world.width = 32;
    config.world.height = 32;
    config.population.initial_creatures = 16;
    let state = AppState::from_config_with_telemetry(config, 0, telemetry);
    let app = router(state.clone());

    call(&app, "POST", "/v3/simulation/startup", r#"{"seed":1}"#).await;
    call(&app, "POST", "/v3/simulation/start", "").await;
    tokio::time::sleep(Duration::from_millis(15)).await;
    call(&app, "POST", "/v3/simulation/pause", "").await;
    // A step once the interval has passed: an interval snapshot and its trace.
    tokio::time::sleep(INTERVAL + Duration::from_millis(50)).await;
    let sampled = step(&app).await;
    // A step inside the interval, a patch and a resume: the transition
    // snapshot at the unsampled tick takes no trace.
    let unsampled = step(&app).await;
    call(&app, "PATCH", "/v3/simulation/config", GRAZING_PATCH).await;
    tokio::time::sleep(Duration::from_millis(15)).await;
    call(&app, "POST", "/v3/simulation/start", "").await;
    tokio::time::sleep(INTERVAL * 2 + Duration::from_millis(100)).await;
    call(&app, "POST", "/v3/simulation/pause", "").await;
    // A step inside the interval of the pause, then a reset at its tick.
    let reset_tick = step(&app).await;
    call(&app, "POST", "/v3/simulation/startup", r#"{"seed":2}"#).await;
    call(&app, "POST", "/v3/simulation/start", "").await;
    tokio::time::sleep(Duration::from_millis(15)).await;
    call(&app, "POST", "/v3/simulation/pause", "").await;
    let shutdown_tick = step(&app).await;
    state.shutdown_telemetry().await;

    let records = receiver.records();
    let runs: Vec<&str> = records
        .iter()
        .filter(|record| record.event_name == "run.started")
        .map(|record| record.attribute("petri.run_id").unwrap())
        .collect();
    assert_eq!(runs.len(), 3);
    let (first, second) = (runs[1], runs[2]);
    let snapshot_ticks = |run: &str| -> Vec<u64> {
        receiver
            .snapshots()
            .iter()
            .filter(|snapshot| snapshot.run_id() == Some(run))
            .map(|snapshot| snapshot.tick().unwrap())
            .collect()
    };
    let traces = receiver.traces();
    let traces_of = |run: &str| -> Vec<&v3_telemetry::testing::ReceivedTrace> {
        traces
            .iter()
            .filter(|trace| trace.run_id() == Some(run))
            .collect()
    };
    let trace_ticks = |run: &str| -> Vec<u64> {
        traces_of(run)
            .iter()
            .map(|trace| trace.tick().unwrap())
            .collect()
    };

    let first_snapshots = snapshot_ticks(first);
    let first_traces = trace_ticks(first);
    assert!(first_traces.contains(&sampled), "{first_traces:?}");
    for tick in [unsampled, reset_tick] {
        assert!(
            first_snapshots.contains(&tick),
            "{tick}: {first_snapshots:?}"
        );
        assert!(!first_traces.contains(&tick), "{tick}: {first_traces:?}");
    }
    assert!(first_traces
        .iter()
        .all(|tick| first_snapshots.contains(tick)));
    let patched = records
        .iter()
        .find(|record| record.event_name == "run.config")
        .and_then(|record| record.attribute("petri.config_digest"))
        .unwrap();
    let running: Vec<_> = traces_of(first)
        .into_iter()
        .filter(|trace| trace.tick().unwrap() > unsampled)
        .collect();
    assert!(!running.is_empty(), "{first_traces:?}");
    for trace in running {
        for span in &trace.spans {
            assert_eq!(span.attribute("petri.config_digest"), Some(patched));
            assert_eq!(span.attribute("petri.sample_policy"), Some("interval"));
        }
    }
    let before = &traces_of(first)[0];
    assert_ne!(
        before.spans[0].attribute("petri.config_digest"),
        Some(patched)
    );

    assert!(snapshot_ticks(second).contains(&shutdown_tick));
    assert!(trace_ticks(second).is_empty(), "{:?}", trace_ticks(second));
    assert!(trace_ticks(runs[0]).is_empty());
}
