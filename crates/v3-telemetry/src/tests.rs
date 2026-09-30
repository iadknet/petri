use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use opentelemetry_sdk::error::OTelSdkResult;
use opentelemetry_sdk::logs::{LogBatch, LogExporter};
use v3_core::config::SimulationConfig;

use super::*;
use crate::queue::{Outcome, RunCounts};
use crate::testing::{closed_endpoint, parse_run_line, ReceivedTrace, Receiver};

/// Accepts every batch and snapshot at once.
#[derive(Debug)]
struct AcceptAll;

impl LogExporter for AcceptAll {
    async fn export(&self, _batch: LogBatch<'_>) -> OTelSdkResult {
        Ok(())
    }
}

impl PostEncoded for AcceptAll {
    fn post(&self, _signal: Signal, _body: Vec<u8>) -> Outcome {
        Outcome::Exported { rejected: 0 }
    }
}

/// Answers every snapshot or trace with a partial success rejecting two data
/// points or spans.
#[derive(Debug)]
struct RejectPoints;

impl PostEncoded for RejectPoints {
    fn post(&self, _signal: Signal, _body: Vec<u8>) -> Outcome {
        Outcome::Exported { rejected: 2 }
    }
}

fn options(endpoint: &str, limits: Limits) -> Options {
    Options {
        service: Service::Cli,
        endpoint: endpoint.to_owned(),
        limits,
        reports: ReportSink::capture(),
        metrics_interval: DEFAULT_METRICS_INTERVAL,
        tick_traces: Switch::On,
        windows: WindowSettings::default(),
    }
}

fn simulation(config: &SimulationConfig) -> Simulation {
    v3_core::simulation::seed_simulation(config.clone(), 7)
}

fn small_config() -> SimulationConfig {
    let mut config = SimulationConfig::default();
    config.world.width = 16;
    config.world.height = 8;
    config
}

fn start(config: &SimulationConfig) -> RunStart<'_> {
    RunStart {
        seed: 7,
        config,
        recipe: None,
        tick: 0,
        ticks_requested: Some(10),
        sample_every: Some(5),
    }
}

/// Telemetry whose worker takes nothing until released.
fn held() -> (Telemetry, ReportSink) {
    held_with(Limits::default(), AcceptAll)
}

fn held_with<M: PostEncoded + Send + 'static>(
    limits: Limits,
    metrics: M,
) -> (Telemetry, ReportSink) {
    let options = options("http://unused", limits);
    let reports = options.reports.clone();
    (
        Telemetry::with_exporter(options, AcceptAll, metrics, Arc::default(), true),
        reports,
    )
}

fn active(telemetry: &Telemetry) -> &Active {
    telemetry.active.as_deref().expect("telemetry is on")
}

fn counts(telemetry: &Telemetry, run: &RunHandle) -> RunCounts {
    active(telemetry)
        .shared
        .counts(run.key)
        .expect("run is still tracked")
}

/// Emits a test record with `body` for `run`.
fn emit_body(telemetry: &Telemetry, run: &RunHandle, body: String) {
    active(telemetry).emit(run, "run.test", 0, Vec::new(), Some(body));
}

fn run_line(reports: &ReportSink, run: &RunHandle) -> BTreeMap<String, String> {
    reports
        .lines()
        .iter()
        .filter_map(|line| parse_run_line(line))
        .find(|fields| fields.get("run").map(String::as_str) == Some(run.id()))
        .expect("the run's line was written")
}

fn wait_for_drain(telemetry: &Telemetry, run: &RunHandle) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while counts(telemetry, run).exported < counts(telemetry, run).accepted {
        assert!(Instant::now() < deadline, "queue did not drain");
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn switch_defaults_off_and_flag_beats_environment() {
    assert_eq!(resolve_switch(None, None), Ok(Switch::Off));
    assert_eq!(resolve_switch(None, Some("")), Ok(Switch::Off));
    assert_eq!(resolve_switch(None, Some("on")), Ok(Switch::On));
    assert_eq!(
        resolve_switch(Some(Switch::Off), Some("on")),
        Ok(Switch::Off)
    );
    assert_eq!(resolve_switch(Some(Switch::On), None), Ok(Switch::On));
    assert_eq!(
        resolve_switch(Some(Switch::On), Some("off")),
        Ok(Switch::On)
    );
    assert!(resolve_switch(None, Some("yes")).is_err());
}

#[test]
fn off_telemetry_draws_no_run() {
    let config = small_config();
    let telemetry = Telemetry::off();
    assert!(!telemetry.is_on());
    assert!(telemetry.begin_run(start(&config)).is_none());
}

#[test]
fn ids_are_32_lowercase_hex_and_distinct() {
    let (_, first) = entropy_id();
    let (_, second) = entropy_id();
    for id in [&first, &second] {
        assert_eq!(id.len(), 32);
        assert!(id
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
    }
    assert_ne!(first, second);
}

#[test]
fn build_revision_is_a_commit_or_unknown() {
    let commit = BUILD_REVISION
        .strip_suffix("-dirty")
        .unwrap_or(BUILD_REVISION);
    assert!(
        commit == "unknown"
            || (commit.len() == 40 && commit.chars().all(|c| c.is_ascii_hexdigit()))
            || std::env::var("PETRI_BUILD_REVISION").is_ok(),
        "unexpected build revision {BUILD_REVISION}"
    );
}

/// The build script's `rerun-if-changed` lines, read from the `output` file
/// Cargo writes beside its `OUT_DIR`.
fn build_script_watches() -> Vec<std::path::PathBuf> {
    let output = std::path::Path::new(env!("OUT_DIR"))
        .parent()
        .expect("OUT_DIR has a parent")
        .join("output");
    std::fs::read_to_string(&output)
        .unwrap_or_else(|error| panic!("{}: {error}", output.display()))
        .lines()
        .filter_map(|line| line.strip_prefix("cargo::rerun-if-changed="))
        .filter(|path| *path != "build.rs")
        .map(std::path::PathBuf::from)
        .collect()
}

#[test]
fn the_build_script_watches_head_by_an_absolute_path() {
    let overridden =
        option_env!("PETRI_BUILD_REVISION").is_some_and(|value| !value.trim().is_empty());
    if overridden || BUILD_REVISION == "unknown" {
        return;
    }
    let head = std::process::Command::new("git")
        .args(["rev-parse", "--path-format=absolute", "--git-path", "HEAD"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("git runs");
    let head = std::path::PathBuf::from(String::from_utf8(head.stdout).unwrap().trim());
    let watches = build_script_watches();
    assert!(
        watches.contains(&head),
        "{watches:?} lacks {}",
        head.display()
    );
    for path in &watches {
        assert!(path.is_absolute() && path.exists(), "{}", path.display());
    }
}

#[test]
fn a_queue_past_2048_records_reports_the_exact_dropped_count() {
    let config = small_config();
    let (telemetry, reports) = held();
    let run = telemetry.begin_run(start(&config)).unwrap();
    for tick in 0..2_100 {
        telemetry.run_state(&run, RunState::Running, tick);
    }
    let offered = 1 + 2_100;
    let queued = counts(&telemetry, &run);
    assert_eq!(queued.accepted, 2_048);
    assert_eq!(queued.dropped, offered - 2_048);

    active(&telemetry).shared.release();
    wait_for_drain(&telemetry, &run);
    telemetry.end_run(run.clone(), EndStatus::Completed, 10, Flush::Wait);
    let line = run_line(&reports, &run);
    assert_eq!(line["exported"], "2049");
    assert_eq!(line["dropped"], (offered - 2_048).to_string());
    assert_eq!(line["failed"], "0");
    assert_eq!(line["abandoned"], "0");
}

#[test]
fn a_queue_past_8_mib_drops_whole_records_and_counts_them() {
    let config = small_config();
    let (telemetry, _) = held();
    let run = telemetry.begin_run(start(&config)).unwrap();
    // Two 3 MiB bodies fit under 8 MiB; each further one would pass it.
    for _ in 0..5 {
        emit_body(&telemetry, &run, "x".repeat(3 * 1024 * 1024));
    }
    let queued = counts(&telemetry, &run);
    assert_eq!(queued.accepted, 3);
    assert_eq!(queued.dropped, 3);
}

#[test]
fn a_body_over_4_mib_is_dropped_whole_counted_and_named() {
    let config = small_config();
    let (telemetry, reports) = held();
    let run = telemetry.begin_run(start(&config)).unwrap();
    emit_body(&telemetry, &run, "x".repeat(4 * 1024 * 1024));
    emit_body(&telemetry, &run, "x".repeat(4 * 1024 * 1024 + 1));
    let queued = counts(&telemetry, &run);
    assert_eq!(queued.accepted, 2, "a body at the cap is kept");
    assert_eq!(queued.dropped, 1);
    let gap = format!(
        "telemetry: gap run={} record=run.test body_bytes={} cap={} dropped whole",
        run.id(),
        4 * 1024 * 1024 + 1,
        4 * 1024 * 1024
    );
    assert!(reports.lines().contains(&gap), "{:?}", reports.lines());
}

#[test]
fn records_reach_the_receiver_with_identity_and_resource() {
    let receiver = Receiver::start();
    let options = options(receiver.endpoint(), Limits::default());
    let reports = options.reports.clone();
    let telemetry = Telemetry::start_with(options);
    let config = small_config();
    let mut run = telemetry
        .begin_run(RunStart {
            recipe: Some("recipes/x.json"),
            ..start(&config)
        })
        .unwrap();
    telemetry.run_state(&run, RunState::Running, 3);
    let mut patched = config.clone();
    patched.world.food.growth_rate *= 0.5;
    telemetry.run_config(&mut run, &patched, 4);
    telemetry.end_run(run.clone(), EndStatus::Completed, 10, Flush::Wait);

    let records = receiver.records();
    let names: Vec<&str> = records.iter().map(|r| r.event_name.as_str()).collect();
    assert_eq!(
        names,
        ["run.started", "run.state", "run.config", "run.ended"]
    );
    let started = &records[0];
    assert_eq!(started.resource["service.name"], "v3-cli");
    assert_eq!(started.resource["petri.build_revision"], BUILD_REVISION);
    assert_eq!(started.resource["petri.invocation_id"].len(), 32);
    assert_eq!(
        started.resource["process.pid"],
        std::process::id().to_string()
    );
    assert_eq!(started.attribute("event.name"), Some("run.started"));
    assert_eq!(started.attribute("petri.run_id"), Some(run.id()));
    assert_eq!(started.attribute("petri.seed"), Some("7"));
    assert_eq!(started.attribute("petri.world"), Some("16x8"));
    assert_eq!(started.attribute("petri.recipe"), Some("recipes/x.json"));
    assert_eq!(started.attribute("petri.ticks_requested"), Some("10"));
    assert_eq!(started.attribute("petri.sample_every"), Some("5"));
    assert_eq!(started.attribute("petri.metrics_interval_ms"), Some("1000"));
    assert_eq!(
        started.attribute("petri.config_digest"),
        Some(v3_core::config::config_digest(&config).as_str())
    );
    assert_eq!(
        started.body.as_deref(),
        Some(serde_json::to_string(&config).unwrap().as_str())
    );
    assert_eq!(records[1].attribute("petri.state"), Some("running"));
    assert_eq!(records[1].attribute("petri.tick"), Some("3"));
    assert_eq!(
        records[2].attribute("petri.config_digest"),
        Some(v3_core::config::config_digest(&patched).as_str())
    );
    assert_eq!(records[3].attribute("petri.status"), Some("completed"));
    assert!(records[3].attribute("petri.wall_seconds").is_some());
    assert!(records[1].attribute("petri.recipe").is_some());

    let lines = reports.lines();
    assert_eq!(
        lines[0],
        format!(
            "telemetry: on endpoint={} invocation={} run={}",
            receiver.endpoint(),
            started.resource["petri.invocation_id"],
            run.id()
        )
    );
    let line = run_line(&reports, &run);
    assert_eq!(line["exported"], "4");
    assert_eq!(line["failed"], "0");
    assert!(line["bytes"].parse::<u64>().unwrap() > 0);
}

#[test]
fn an_unknown_recipe_is_omitted_not_empty() {
    let receiver = Receiver::start();
    let telemetry = Telemetry::start_with(options(receiver.endpoint(), Limits::default()));
    let config = small_config();
    let run = telemetry.begin_run(start(&config)).unwrap();
    telemetry.end_run(run, EndStatus::Completed, 0, Flush::Wait);
    assert!(receiver
        .records()
        .iter()
        .all(|record| record.attribute("petri.recipe").is_none()));
}

#[test]
fn records_a_partial_success_rejects_count_as_failed_for_their_run() {
    let receiver = Receiver::rejecting(1);
    let options = options(receiver.endpoint(), Limits::default());
    let reports = options.reports.clone();
    let telemetry = Telemetry::start_otlp(options, true);
    let config = small_config();
    // Both runs are queued before the worker takes anything, so one batch
    // could hold all six records.
    let four = telemetry.begin_run(start(&config)).unwrap();
    telemetry.run_state(&four, RunState::Running, 0);
    telemetry.run_state(&four, RunState::Paused, 1);
    telemetry.end_run(four.clone(), EndStatus::Reset, 1, Flush::Background);
    let two = telemetry.begin_run(start(&config)).unwrap();
    telemetry.end_run(two.clone(), EndStatus::Completed, 1, Flush::Background);
    active(&telemetry).shared.release();
    telemetry.shutdown();

    for (run, exported) in [(&four, "3"), (&two, "1")] {
        let line = run_line(&reports, run);
        assert_eq!(line["exported"], exported, "{line:?}");
        assert_eq!(line["failed"], "1", "{line:?}");
        assert_eq!(line["abandoned"], "0", "{line:?}");
        // Each run's one batch was partly rejected: no accepted batch bytes.
        assert_eq!(line["bytes"], "0", "{line:?}");
    }
    assert_eq!(receiver.records().len(), 4);
}

#[test]
fn a_closed_port_fails_records_without_blocking_the_flush() {
    let options = options(&closed_endpoint(), Limits::default());
    let reports = options.reports.clone();
    let telemetry = Telemetry::start_with(options);
    let config = small_config();
    let run = telemetry.begin_run(start(&config)).unwrap();
    let began = Instant::now();
    telemetry.end_run(run.clone(), EndStatus::Completed, 1, Flush::Wait);
    assert!(began.elapsed() < Duration::from_secs(5));
    let line = run_line(&reports, &run);
    assert_eq!(line["exported"], "0");
    assert_eq!(line["failed"], "2");
    assert_eq!(line["abandoned"], "0");
}

#[test]
fn records_answered_with_an_undecodable_success_count_as_failed() {
    let receiver = Receiver::garbled();
    let options = options(receiver.endpoint(), Limits::default());
    let reports = options.reports.clone();
    let telemetry = Telemetry::start_with(options);
    let config = small_config();
    let run = telemetry.begin_run(start(&config)).unwrap();
    telemetry.end_run(run.clone(), EndStatus::Completed, 1, Flush::Wait);
    let line = run_line(&reports, &run);
    assert_eq!(line["exported"], "0", "{line:?}");
    assert_eq!(line["failed"], "2", "{line:?}");
    assert_eq!(line["abandoned"], "0", "{line:?}");
    assert_eq!(line["bytes"], "0", "{line:?}");
}

#[test]
fn a_silent_receiver_bounds_the_run_flush_at_10_s_and_abandons_the_rest() {
    let receiver = Receiver::hanging();
    let limits = Limits::default();
    let options = options(receiver.endpoint(), limits);
    let reports = options.reports.clone();
    let telemetry = Telemetry::start_with(options);
    let config = small_config();
    let run = telemetry.begin_run(start(&config)).unwrap();
    for tick in 0..1_500 {
        telemetry.run_state(&run, RunState::Running, tick);
    }
    let began = Instant::now();
    telemetry.end_run(run.clone(), EndStatus::Completed, 1_500, Flush::Wait);
    let flush = began.elapsed();
    assert!(
        flush >= limits.flush_timeout,
        "flush ended early: {flush:?}"
    );
    assert!(
        flush < limits.flush_timeout + Duration::from_millis(500),
        "flush overran: {flush:?}"
    );

    let line = run_line(&reports, &run);
    let failed: u64 = line["failed"].parse().unwrap();
    let abandoned: u64 = line["abandoned"].parse().unwrap();
    assert_eq!(line["exported"], "0");
    assert_eq!(line["dropped"], "0");
    assert_eq!(failed + abandoned, 1_502);
    // At most two 512-record batches can time out in 10 s at 5 s each.
    assert!(abandoned >= 1_502 - 2 * 512, "abandoned {abandoned}");
}

#[test]
fn a_background_end_returns_at_once_and_the_line_follows() {
    let (telemetry, reports) = held();
    let config = small_config();
    let run = telemetry.begin_run(start(&config)).unwrap();
    telemetry.end_run(run.clone(), EndStatus::Reset, 5, Flush::Background);
    assert!(reports
        .lines()
        .iter()
        .all(|line| parse_run_line(line).is_none()));
    active(&telemetry).shared.release();
    let deadline = Instant::now() + Duration::from_secs(5);
    while !reports
        .lines()
        .iter()
        .any(|line| parse_run_line(line).is_some())
    {
        assert!(Instant::now() < deadline, "no run line");
        std::thread::sleep(Duration::from_millis(5));
    }
    let line = run_line(&reports, &run);
    assert_eq!(line["exported"], "2");
}

#[test]
fn shutdown_abandons_pending_records_of_every_run_within_the_bound() {
    let receiver = Receiver::hanging();
    let limits = Limits {
        request_timeout: Duration::from_millis(300),
        flush_timeout: Duration::from_millis(600),
        ..Limits::default()
    };
    let options = options(receiver.endpoint(), limits);
    let reports = options.reports.clone();
    let telemetry = Telemetry::start_with(options);
    let config = small_config();
    let first = telemetry.begin_run(start(&config)).unwrap();
    for tick in 0..1_000 {
        telemetry.run_state(&first, RunState::Running, tick);
    }
    telemetry.end_run(first.clone(), EndStatus::Reset, 1_000, Flush::Background);
    let second = telemetry.begin_run(start(&config)).unwrap();
    for tick in 0..1_000 {
        telemetry.run_state(&second, RunState::Running, tick);
    }
    let began = Instant::now();
    telemetry.end_run(
        second.clone(),
        EndStatus::Shutdown,
        1_000,
        Flush::Background,
    );
    telemetry.shutdown();
    assert!(began.elapsed() < limits.flush_timeout + Duration::from_millis(300));

    for run in [&first, &second] {
        let line = run_line(&reports, run);
        let total: u64 = ["exported", "failed", "dropped", "abandoned"]
            .iter()
            .map(|key| line[*key].parse::<u64>().unwrap())
            .sum();
        assert_eq!(total, 1_002, "{line:?}");
    }
    let abandoned: u64 = [&first, &second]
        .iter()
        .map(|run| run_line(&reports, run)["abandoned"].parse::<u64>().unwrap())
        .sum();
    assert!(abandoned > 0, "a silent receiver leaves records to abandon");
}

#[test]
fn the_metrics_interval_is_whole_milliseconds_from_10_to_3600000_defaulting_to_1000() {
    let default = Ok(Duration::from_millis(1_000));
    assert_eq!(resolve_metrics_interval(None), default);
    assert_eq!(resolve_metrics_interval(Some("")), default);
    assert_eq!(resolve_metrics_interval(Some(" ")), default);
    assert_eq!(
        resolve_metrics_interval(Some("10")),
        Ok(Duration::from_millis(10))
    );
    assert_eq!(
        resolve_metrics_interval(Some("3600000")),
        Ok(Duration::from_millis(3_600_000))
    );
    for invalid in ["0", "9", "3600001", "-10", "1.5", "1s", "ten"] {
        let error = resolve_metrics_interval(Some(invalid)).unwrap_err();
        assert!(error.contains(METRICS_INTERVAL_ENV), "{error}");
    }
}

#[test]
fn snapshots_past_the_queue_bound_are_dropped_and_counted_as_taken() {
    let config = small_config();
    let sim = simulation(&config);
    let limits = Limits {
        max_queue_records: 3,
        ..Limits::default()
    };
    let (telemetry, reports) = held_with(limits, AcceptAll);
    let mut run = telemetry.begin_run(start(&config)).unwrap();
    // `run.started` and two snapshots fill the queue; the third is dropped.
    let mut sim = sim;
    for _ in 0..3 {
        assert!(telemetry.snapshot(&mut run, &sim, Trigger::RunEnd));
        sim.tick += 1;
    }
    let queued = counts(&telemetry, &run);
    assert_eq!(
        (queued.accepted, queued.dropped, queued.snapshots),
        (3, 1, 3)
    );

    active(&telemetry).shared.release();
    wait_for_drain(&telemetry, &run);
    telemetry.end_run(run.clone(), EndStatus::Completed, sim.tick, Flush::Wait);
    let line = run_line(&reports, &run);
    assert_eq!(line["exported"], "4");
    assert_eq!(line["dropped"], "1");
    assert_eq!(line["snapshots"], "3");
}

#[test]
fn a_snapshot_with_rejected_data_points_counts_as_failed_with_no_bytes() {
    let config = small_config();
    let sim = simulation(&config);
    let bytes_of = |metrics_rejected: bool| {
        let (telemetry, reports) = if metrics_rejected {
            held_with(Limits::default(), RejectPoints)
        } else {
            held()
        };
        let mut run = telemetry.begin_run(start(&config)).unwrap();
        telemetry.snapshot(&mut run, &sim, Trigger::RunEnd);
        active(&telemetry).shared.release();
        telemetry.end_run(run.clone(), EndStatus::Completed, sim.tick, Flush::Wait);
        let line = run_line(&reports, &run);
        (
            line["exported"].clone(),
            line["failed"].clone(),
            line["bytes"].parse::<u64>().unwrap(),
        )
    };
    let (exported, failed, accepted_bytes) = bytes_of(false);
    assert_eq!((exported.as_str(), failed.as_str()), ("3", "0"));
    let (exported, failed, rejected_bytes) = bytes_of(true);
    assert_eq!((exported.as_str(), failed.as_str()), ("2", "1"));
    // The two records' bytes remain; the snapshot's encoded request, over
    // 1 KB for any simulation, does not.
    assert!(
        accepted_bytes > rejected_bytes + 1_024,
        "{accepted_bytes} vs {rejected_bytes}"
    );
}

#[test]
fn a_rejecting_collector_fails_the_snapshot_it_answers() {
    let receiver = Receiver::rejecting(1);
    let options = options(receiver.endpoint(), Limits::default());
    let reports = options.reports.clone();
    let telemetry = Telemetry::start_with(options);
    let config = small_config();
    let sim = simulation(&config);
    let mut run = telemetry.begin_run(start(&config)).unwrap();
    telemetry.snapshot(&mut run, &sim, Trigger::RunEnd);
    telemetry.end_run(run.clone(), EndStatus::Completed, sim.tick, Flush::Wait);
    let line = run_line(&reports, &run);
    assert_eq!(line["exported"], "0");
    assert_eq!(line["failed"], "3");
    assert_eq!(line["snapshots"], "1");
    assert!(receiver.snapshots().is_empty());
}

#[test]
fn a_snapshot_answered_with_an_undecodable_success_counts_as_failed() {
    let receiver = Receiver::garbled();
    let options = options(receiver.endpoint(), Limits::default());
    let reports = options.reports.clone();
    let telemetry = Telemetry::start_with(options);
    let config = small_config();
    let sim = simulation(&config);
    let mut run = telemetry.begin_run(start(&config)).unwrap();
    telemetry.snapshot(&mut run, &sim, Trigger::RunEnd);
    telemetry.end_run(run.clone(), EndStatus::Completed, sim.tick, Flush::Wait);
    let line = run_line(&reports, &run);
    assert_eq!(line["exported"], "0", "{line:?}");
    assert_eq!(line["failed"], "3", "{line:?}");
    assert_eq!(line["snapshots"], "1", "{line:?}");
    assert_eq!(line["bytes"], "0", "{line:?}");
}

#[test]
fn snapshots_and_records_of_one_run_stay_attributed_to_it_across_a_reset() {
    let receiver = Receiver::start();
    let options = options(receiver.endpoint(), Limits::default());
    let reports = options.reports.clone();
    let telemetry = Telemetry::start_otlp(options, true);
    let config = small_config();
    let mut sim = simulation(&config);
    // Everything is queued before the worker takes anything: one FIFO holds
    // the first run's records and snapshot, then the second run's.
    let mut first = telemetry.begin_run(start(&config)).unwrap();
    telemetry.run_state(&first, RunState::Running, 0);
    assert!(telemetry.snapshot(&mut first, &sim, Trigger::RunEnd));
    telemetry.end_run(first.clone(), EndStatus::Reset, 0, Flush::Background);
    sim.tick = 5;
    let mut second = telemetry.begin_run(start(&config)).unwrap();
    assert!(telemetry.snapshot(&mut second, &sim, Trigger::Transition));
    telemetry.run_state(&second, RunState::Running, 5);
    active(&telemetry).shared.release();
    telemetry.end_run(second.clone(), EndStatus::Shutdown, 5, Flush::Wait);
    telemetry.shutdown();

    let snapshots = receiver.snapshots();
    let ticks: Vec<(Option<&str>, Option<u64>)> = snapshots
        .iter()
        .map(|snapshot| (snapshot.run_id(), snapshot.tick()))
        .collect();
    assert_eq!(
        ticks,
        [(Some(first.id()), Some(0)), (Some(second.id()), Some(5))]
    );
    for (run, exported) in [(&first, "4"), (&second, "4")] {
        let line = run_line(&reports, run);
        assert_eq!(line["exported"], exported, "{line:?}");
        assert_eq!(line["failed"], "0", "{line:?}");
        assert_eq!(line["snapshots"], "1", "{line:?}");
    }
}

#[test]
fn a_snapshot_reaches_the_receiver_with_the_run_resource_and_census() {
    let receiver = Receiver::start();
    let telemetry = Telemetry::start_with(options(receiver.endpoint(), Limits::default()));
    let config = small_config();
    let mut sim = simulation(&config);
    for _ in 0..3 {
        v3_core::simulation::run_tick(&mut sim, &mut None);
    }
    let mut run = telemetry.begin_run(start(&config)).unwrap();
    assert!(telemetry.snapshot(&mut run, &sim, Trigger::RunEnd));
    telemetry.end_run(run.clone(), EndStatus::Completed, sim.tick, Flush::Wait);

    let snapshots = receiver.snapshots();
    assert_eq!(snapshots.len(), 1);
    let snapshot = &snapshots[0];
    let record = &receiver.records()[0];
    assert_eq!(snapshot.resource, record.resource);
    assert!(snapshot
        .points
        .iter()
        .all(|point| point.attribute("petri.run_id") == Some(run.id())));
    assert_eq!(snapshot.tick(), Some(3));
    let census = Census::of(&sim);
    let value = |name: &str| snapshot.value(name, &[]).unwrap().as_f64();
    assert_eq!(value("petri.tick.population"), census.population as f64);
    assert_eq!(
        value("petri.tick.mean_energy"),
        f64::from(sim.mean_energy())
    );
    assert_eq!(
        value("petri.run.creature_ticks"),
        sim.stats.creature_ticks_total as f64
    );
    for point in &snapshot.points {
        let sampled = point.name.starts_with("petri.tick.");
        assert_eq!(
            point.description == "sampled at the snapshot tick",
            sampled,
            "{}",
            point.name
        );
        match point.kind {
            crate::testing::MetricKind::Gauge => assert_eq!(point.start_time_unix_nano, 0),
            _ => assert!(
                point.start_time_unix_nano > 0
                    && point.start_time_unix_nano <= point.time_unix_nano,
                "{}",
                point.name
            ),
        }
    }
}

/// Answers every trace with a partial success rejecting one span; accepts
/// every snapshot.
#[derive(Debug)]
struct RejectSpans;

impl PostEncoded for RejectSpans {
    fn post(&self, signal: Signal, _body: Vec<u8>) -> Outcome {
        match signal {
            Signal::Metrics => Outcome::Exported { rejected: 0 },
            Signal::Traces | Signal::Windows => Outcome::Exported { rejected: 1 },
        }
    }
}

/// `small_config`'s simulation after `ticks` ticks.
fn ticked(config: &SimulationConfig, ticks: u64) -> Simulation {
    let mut sim = simulation(config);
    for _ in 0..ticks {
        v3_core::simulation::run_tick(&mut sim, &mut None);
    }
    sim
}

#[test]
fn a_tick_snapshot_takes_the_trace_of_the_tick_that_just_ran_and_only_then() {
    let config = small_config();
    let (telemetry, _) = held();
    let mut run = telemetry.begin_run(start(&config)).unwrap();
    // No tick has run: the snapshot is taken, no trace.
    let mut sim = simulation(&config);
    assert!(telemetry.tick_snapshot(&mut run, &sim, TickSample::RunEnd));
    assert_eq!(counts(&telemetry, &run).traces, 0);
    // A seam from an earlier tick is not this tick's.
    v3_core::simulation::run_tick(&mut sim, &mut None);
    sim.tick += 1;
    assert!(telemetry.tick_snapshot(&mut run, &sim, TickSample::RunEnd));
    assert_eq!(counts(&telemetry, &run).traces, 0);
    // A snapshot the cadence skips takes no trace.
    v3_core::simulation::run_tick(&mut sim, &mut None);
    assert!(!telemetry.tick_snapshot(&mut run, &sim, TickSample::Interval));
    assert_eq!(counts(&telemetry, &run).traces, 0);
    assert!(telemetry.tick_snapshot(&mut run, &sim, TickSample::RunEnd));
    let taken = counts(&telemetry, &run);
    assert_eq!((taken.snapshots, taken.traces), (3, 1));
    // A plain snapshot never takes one.
    v3_core::simulation::run_tick(&mut sim, &mut None);
    assert!(telemetry.snapshot(&mut run, &sim, Trigger::RunEnd));
    assert_eq!(counts(&telemetry, &run).traces, 1);
}

#[test]
fn tick_traces_off_take_snapshots_without_traces_and_say_so_on_run_started() {
    let receiver = Receiver::start();
    let config = small_config();
    let sim = ticked(&config, 2);
    let mut starts = Vec::new();
    for switch in [Switch::On, Switch::Off] {
        let options = Options {
            tick_traces: switch,
            ..options(receiver.endpoint(), Limits::default())
        };
        let reports = options.reports.clone();
        let telemetry = Telemetry::start_with(options);
        let mut run = telemetry.begin_run(start(&config)).unwrap();
        assert!(telemetry.tick_snapshot(&mut run, &sim, TickSample::RunEnd));
        telemetry.end_run(run.clone(), EndStatus::Completed, sim.tick, Flush::Wait);
        let line = run_line(&reports, &run);
        let traces = if switch == Switch::On { "1" } else { "0" };
        assert_eq!(line["traces"], traces, "{line:?}");
        assert_eq!(line["snapshots"], "1", "{line:?}");
        starts.push(run.id().to_owned());
    }
    let records = receiver.records();
    let started = |id: &str| {
        records
            .iter()
            .find(|record| {
                record.event_name == "run.started" && record.attribute("petri.run_id") == Some(id)
            })
            .and_then(|record| record.attribute("petri.tick_traces"))
            .map(str::to_owned)
    };
    assert_eq!(started(&starts[0]).as_deref(), Some("on"));
    assert_eq!(started(&starts[1]).as_deref(), Some("off"));
    let traces = receiver.traces();
    assert_eq!(traces.len(), 1);
    assert_eq!(traces[0].run_id(), Some(starts[0].as_str()));
    assert_eq!(traces[0].tick(), Some(2));
    assert_eq!(
        traces[0].spans[0].attribute("petri.sample_policy"),
        Some("run_end")
    );
    let record = records
        .iter()
        .find(|record| record.attribute("petri.run_id") == Some(starts[0].as_str()))
        .unwrap();
    assert_eq!(traces[0].resource, record.resource);
    assert!(records
        .iter()
        .all(|record| record.attribute("petri.tick_traces_capped").is_none()));
}

#[test]
fn trace_items_past_the_queue_bound_are_dropped_and_counted_as_taken() {
    let config = small_config();
    let limits = Limits {
        max_queue_records: 3,
        ..Limits::default()
    };
    let (telemetry, reports) = held_with(limits, AcceptAll);
    let mut run = telemetry.begin_run(start(&config)).unwrap();
    let mut sim = ticked(&config, 1);
    // `run.started`, a snapshot and a trace fill the queue; the next tick's
    // snapshot and trace are dropped.
    for _ in 0..2 {
        assert!(telemetry.tick_snapshot(&mut run, &sim, TickSample::RunEnd));
        v3_core::simulation::run_tick(&mut sim, &mut None);
    }
    let queued = counts(&telemetry, &run);
    assert_eq!(
        (
            queued.accepted,
            queued.dropped,
            queued.snapshots,
            queued.traces
        ),
        (3, 2, 2, 2)
    );
    active(&telemetry).shared.release();
    wait_for_drain(&telemetry, &run);
    telemetry.end_run(run.clone(), EndStatus::Completed, sim.tick, Flush::Wait);
    let line = run_line(&reports, &run);
    assert_eq!(line["exported"], "4");
    assert_eq!(line["dropped"], "2");
    assert_eq!(line["traces"], "2");
}

#[test]
fn a_trace_with_rejected_spans_counts_as_failed_with_no_bytes() {
    let config = small_config();
    let sim = ticked(&config, 1);
    let outcome = |rejecting: bool| {
        let (telemetry, reports) = if rejecting {
            held_with(Limits::default(), RejectSpans)
        } else {
            held()
        };
        let mut run = telemetry.begin_run(start(&config)).unwrap();
        telemetry.tick_snapshot(&mut run, &sim, TickSample::RunEnd);
        active(&telemetry).shared.release();
        telemetry.end_run(run.clone(), EndStatus::Completed, sim.tick, Flush::Wait);
        let line = run_line(&reports, &run);
        (
            line["exported"].clone(),
            line["failed"].clone(),
            line["bytes"].parse::<u64>().unwrap(),
        )
    };
    let (exported, failed, accepted_bytes) = outcome(false);
    assert_eq!((exported.as_str(), failed.as_str()), ("4", "0"));
    let (exported, failed, rejected_bytes) = outcome(true);
    assert_eq!((exported.as_str(), failed.as_str()), ("3", "1"));
    // The trace's encoded request, over 1 KB, is not counted.
    assert!(
        accepted_bytes > rejected_bytes + 1_024,
        "{accepted_bytes} vs {rejected_bytes}"
    );
}

#[test]
fn a_rejecting_or_garbling_collector_fails_the_trace_it_answers() {
    let config = small_config();
    let sim = ticked(&config, 1);
    for receiver in [Receiver::rejecting(1), Receiver::garbled()] {
        let options = options(receiver.endpoint(), Limits::default());
        let reports = options.reports.clone();
        let telemetry = Telemetry::start_with(options);
        let mut run = telemetry.begin_run(start(&config)).unwrap();
        telemetry.tick_snapshot(&mut run, &sim, TickSample::RunEnd);
        telemetry.end_run(run.clone(), EndStatus::Completed, sim.tick, Flush::Wait);
        let line = run_line(&reports, &run);
        assert_eq!(line["exported"], "0", "{line:?}");
        assert_eq!(line["failed"], "4", "{line:?}");
        assert_eq!(line["traces"], "1", "{line:?}");
        assert_eq!(line["bytes"], "0", "{line:?}");
    }
}

#[test]
fn the_65536th_trace_is_the_last_captured_and_run_ended_says_so() {
    let receiver = Receiver::start();
    let options = options(receiver.endpoint(), Limits::default());
    let reports = options.reports.clone();
    let telemetry = Telemetry::start_with(options);
    let config = small_config();
    let mut sim = ticked(&config, 1);
    let mut run = telemetry.begin_run(start(&config)).unwrap();
    run.traces_taken = MAX_TICK_TRACES_PER_RUN - 1;
    assert!(telemetry.tick_snapshot(&mut run, &sim, TickSample::RunEnd));
    assert_eq!(run.traces_taken, MAX_TICK_TRACES_PER_RUN);
    v3_core::simulation::run_tick(&mut sim, &mut None);
    assert!(telemetry.tick_snapshot(&mut run, &sim, TickSample::RunEnd));
    assert_eq!(run.traces_taken, MAX_TICK_TRACES_PER_RUN);
    telemetry.end_run(run.clone(), EndStatus::Completed, sim.tick, Flush::Wait);

    let line = run_line(&reports, &run);
    assert_eq!(line["traces"], "1", "{line:?}");
    assert_eq!(line["snapshots"], "2", "{line:?}");
    let ended = receiver
        .records()
        .into_iter()
        .find(|record| record.event_name == "run.ended")
        .unwrap();
    assert_eq!(ended.attribute("petri.tick_traces_capped"), Some("65536"));
    let ticks: Vec<Option<u64>> = receiver.traces().iter().map(ReceivedTrace::tick).collect();
    assert_eq!(ticks, [Some(1)]);
}

#[test]
fn a_trace_abandoned_by_the_flush_is_counted_for_its_run() {
    let config = small_config();
    let limits = Limits {
        flush_timeout: Duration::from_millis(50),
        ..Limits::default()
    };
    let (telemetry, reports) = held_with(limits, AcceptAll);
    let sim = ticked(&config, 1);
    let mut run = telemetry.begin_run(start(&config)).unwrap();
    assert!(telemetry.tick_snapshot(&mut run, &sim, TickSample::RunEnd));
    telemetry.end_run(run.clone(), EndStatus::Completed, sim.tick, Flush::Wait);
    let line = run_line(&reports, &run);
    assert_eq!(line["exported"], "0", "{line:?}");
    // `run.started`, the snapshot, the trace and `run.ended`.
    assert_eq!(line["abandoned"], "4", "{line:?}");
    assert_eq!(line["traces"], "1", "{line:?}");
}

/// A measurement's start with every attribute present.
fn measurement_start(seeds: &[u64]) -> MeasurementStart<'_> {
    MeasurementStart {
        command: "bench",
        feature: Some("t21-f06-test"),
        profile: Some("sweep"),
        seeds: Some(seeds),
        config_digest: Some("digest"),
        threads: Some(2),
        ..MeasurementStart::default()
    }
}

#[test]
fn a_held_telemetry_sends_nothing_until_released_then_everything() {
    let receiver = Receiver::start();
    let options = options(receiver.endpoint(), Limits::default());
    let reports = options.reports.clone();
    let telemetry = Telemetry::start_otlp(options, true);
    let config = small_config();
    let measurement = telemetry
        .begin_measurement(measurement_start(&[7]))
        .unwrap();
    let run = telemetry.begin_run(start(&config)).unwrap();
    telemetry.end_run(run.clone(), EndStatus::Completed, 10, Flush::Background);
    telemetry.end_measurement(measurement.clone(), MeasurementEnd::default());
    std::thread::sleep(Duration::from_millis(300));
    assert!(receiver.records().is_empty());
    assert_eq!(receiver.first_request_at(), None);
    assert!(reports
        .lines()
        .iter()
        .all(|line| parse_run_line(line).is_none()));

    let released = SystemTime::now();
    telemetry.release();
    let began = Instant::now();
    telemetry.shutdown();
    assert!(began.elapsed() < Limits::default().flush_timeout);
    assert_eq!(receiver.records().len(), 4);
    assert!(receiver.first_request_at().unwrap() >= released);
    let lines: Vec<_> = reports
        .lines()
        .iter()
        .filter_map(|line| parse_run_line(line))
        .collect();
    assert_eq!(
        lines.len(),
        2,
        "one line per run and one for the measurement"
    );
    for id in [run.id(), measurement.id()] {
        let line = lines.iter().find(|fields| fields["run"] == id).unwrap();
        assert_eq!(line["exported"], "2", "{line:?}");
        // The held period is not flush time.
        assert!(line["flush_ms"].parse::<u64>().unwrap() < 300, "{line:?}");
    }
}

#[test]
fn measurement_records_carry_their_attributes_and_omit_absent_ones() {
    let receiver = Receiver::start();
    let telemetry = Telemetry::start_otlp(options(receiver.endpoint(), Limits::default()), true);
    let seeds = [11, 22];
    let full = telemetry
        .begin_measurement(measurement_start(&seeds))
        .unwrap();
    telemetry.end_measurement(
        full.clone(),
        MeasurementEnd {
            exit_code: 3,
            incomplete: Some(true),
            severe: Some(true),
            stop_reason: Some("wall_cap"),
            horizon: Some(500),
            gate_favorable: Some(false),
            summary_path: Some("/tmp/summary.json"),
            raw_sha256: Some("abc"),
            raw_bytes: Some(12),
            body: Some(r#"{"a":1}"#.to_owned()),
        },
    );
    let bare = telemetry
        .begin_measurement(MeasurementStart {
            command: "run",
            assay: Some("food-seeking"),
            seed: Some(7),
            ..MeasurementStart::default()
        })
        .unwrap();
    telemetry.end_measurement(
        bare.clone(),
        MeasurementEnd {
            exit_code: 1,
            ..MeasurementEnd::default()
        },
    );
    telemetry.release();
    telemetry.shutdown();
    let records = receiver.records();
    let find = |id: &str, event: &str| {
        records
            .iter()
            .find(|r| r.event_name == event && r.attribute("petri.run_id") == Some(id))
            .unwrap_or_else(|| panic!("{event} for {id}"))
            .clone()
    };
    let started = find(full.id(), "measurement.started");
    let expected: BTreeMap<String, String> = [
        ("event.name", "measurement.started"),
        ("petri.run_id", full.id()),
        ("petri.tick", "0"),
        ("petri.command", "bench"),
        ("petri.feature", "t21-f06-test"),
        ("petri.profile", "sweep"),
        ("petri.seeds", "11,22"),
        ("petri.config_digest", "digest"),
        ("petri.threads", "2"),
    ]
    .into_iter()
    .map(|(key, value)| (key.to_owned(), value.to_owned()))
    .collect();
    assert_eq!(started.attributes, expected);
    assert_eq!(started.body, None);
    assert!(started.time_unix_nano > 0);
    let ended = find(full.id(), "measurement.ended");
    for (key, value) in [
        ("petri.tick", "0"),
        ("petri.exit_code", "3"),
        ("petri.incomplete", "true"),
        ("petri.severe", "true"),
        ("petri.stop_reason", "wall_cap"),
        ("petri.horizon", "500"),
        ("petri.gate_favorable", "false"),
        ("petri.summary_path", "/tmp/summary.json"),
        ("petri.raw_sha256", "abc"),
        ("petri.raw_bytes", "12"),
    ] {
        assert_eq!(ended.attribute(key), Some(value), "{key}");
    }
    assert!(ended.attribute("petri.wall_seconds").is_some());
    assert_eq!(ended.body.as_deref(), Some(r#"{"a":1}"#));
    assert!(ended.time_unix_nano >= started.time_unix_nano);

    let bare_started = find(bare.id(), "measurement.started");
    for key in [
        "petri.feature",
        "petri.profile",
        "petri.pilot",
        "petri.seeds",
        "petri.config_digest",
        "petri.threads",
    ] {
        assert_eq!(bare_started.attribute(key), None, "{key}");
    }
    assert_eq!(bare_started.attribute("petri.assay"), Some("food-seeking"));
    assert_eq!(bare_started.attribute("petri.seed"), Some("7"));
    let bare_ended = find(bare.id(), "measurement.ended");
    let keys: Vec<&str> = bare_ended.attributes.keys().map(String::as_str).collect();
    assert_eq!(
        keys,
        [
            "event.name",
            "petri.exit_code",
            "petri.run_id",
            "petri.tick",
            "petri.wall_seconds"
        ]
    );
    assert_eq!(bare_ended.body, None);
}

#[test]
fn run_ended_with_totals_encodes_each_as_petri_total() {
    let receiver = Receiver::start();
    let telemetry = Telemetry::start_with(options(receiver.endpoint(), Limits::default()));
    let config = small_config();
    let run = telemetry.begin_run(start(&config)).unwrap();
    telemetry.end_run_with_totals(
        run,
        EndStatus::Completed,
        20,
        Flush::Wait,
        &[("ticks", 20), ("creature_ticks", 320)],
    );
    let ended = receiver
        .records()
        .into_iter()
        .find(|record| record.event_name == "run.ended")
        .unwrap();
    assert_eq!(ended.attribute("petri.total.ticks"), Some("20"));
    assert_eq!(ended.attribute("petri.total.creature_ticks"), Some("320"));
    assert_eq!(ended.attribute("petri.tick"), Some("20"));
    assert!(ended.time_unix_nano > 0);
}
