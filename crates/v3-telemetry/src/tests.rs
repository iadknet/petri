use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use opentelemetry_sdk::error::OTelSdkResult;
use opentelemetry_sdk::logs::{LogBatch, LogExporter};
use v3_core::config::SimulationConfig;

use super::*;
use crate::queue::RunCounts;
use crate::testing::{closed_endpoint, parse_run_line, Receiver};

/// Accepts every batch at once.
#[derive(Debug)]
struct AcceptAll;

impl LogExporter for AcceptAll {
    async fn export(&self, _batch: LogBatch<'_>) -> OTelSdkResult {
        Ok(())
    }
}

fn options(endpoint: &str, limits: Limits) -> Options {
    Options {
        service: Service::Cli,
        endpoint: endpoint.to_owned(),
        limits,
        reports: ReportSink::capture(),
    }
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
    let options = options("http://unused", Limits::default());
    let reports = options.reports.clone();
    (
        Telemetry::with_exporter(options, AcceptAll, Arc::default(), true),
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
