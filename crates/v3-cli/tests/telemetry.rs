//! Telemetry neutrality (T21.F01): one seeded run's canonical NDJSON is the same
//! with telemetry off, on with a receiver listening, and on with a closed port;
//! the run's snapshots (T21.F02) agree with its final `tick_sample`; and each
//! snapshot after a tick carries that tick's trace (T21.F03); and creature
//! windows (T21.F04) export the recorded creature's ticks. The measurement
//! commands' records (T21.F06) are in `telemetry/measurement.rs`.
#![cfg(feature = "telemetry")]

#[path = "telemetry/measurement.rs"]
mod measurement;

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use v3_telemetry::testing::{closed_endpoint, parse_run_line, PointValue, ReceivedTrace, Receiver};

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
/// `endpoint`, the metrics `interval` and the tick-trace switch `traces`.
fn command(
    switch: &str,
    endpoint: Option<&str>,
    interval: Option<&str>,
    traces: Option<&str>,
    shape: Shape,
) -> std::process::Output {
    command_with(switch, endpoint, interval, traces, shape, &[])
}

/// [`command`] with extra environment variables.
fn command_with(
    switch: &str,
    endpoint: Option<&str>,
    interval: Option<&str>,
    traces: Option<&str>,
    shape: Shape,
    env: &[(&str, &str)],
) -> std::process::Output {
    // Each call writes its own recipe file: tests run in parallel threads, so a
    // shared path could be read while another test rewrites it.
    static RECIPE_SEQ: AtomicUsize = AtomicUsize::new(0);
    let recipe = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "t21-{}-{}-{}.json",
        shape.name,
        std::process::id(),
        RECIPE_SEQ.fetch_add(1, Ordering::Relaxed)
    ));
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
        .env_remove(v3_telemetry::METRICS_INTERVAL_ENV)
        .env_remove(v3_telemetry::TICK_TRACES_ENV)
        .env_remove(v3_telemetry::CREATURE_WINDOWS_ENV)
        .env_remove(v3_telemetry::WINDOW_TICKS_ENV)
        .env_remove(v3_telemetry::WINDOW_INTERVAL_ENV)
        .env_remove(v3_telemetry::PRESET_ENV)
        .envs(env.iter().copied());
    if let Some(traces) = traces {
        command.env(v3_telemetry::TICK_TRACES_ENV, traces);
    }
    if let Some(endpoint) = endpoint {
        command.env(v3_telemetry::ENDPOINT_ENV, endpoint);
    }
    if let Some(interval) = interval {
        command.env(v3_telemetry::METRICS_INTERVAL_ENV, interval);
    }
    let output = command.output().expect("v3-cli runs");
    let _ = std::fs::remove_file(&recipe);
    output
}

fn run_with(
    switch: &str,
    endpoint: Option<&str>,
    interval: Option<&str>,
    shape: Shape,
) -> RunOutput {
    run_traced(switch, endpoint, interval, None, shape)
}

fn run_traced(
    switch: &str,
    endpoint: Option<&str>,
    interval: Option<&str>,
    traces: Option<&str>,
    shape: Shape,
) -> RunOutput {
    let output = command(switch, endpoint, interval, traces, shape);
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

/// The run's tick traces (`tick` roots), leaving creature windows out.
fn tick_traces(receiver: &Receiver, run_id: &str) -> Vec<ReceivedTrace> {
    receiver
        .traces()
        .into_iter()
        .filter(|trace| trace.run_id() == Some(run_id) && trace.spans[0].name == "tick")
        .collect()
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
    assert_eq!(names, ["run.started", "creature.genome", "run.ended"]);
    assert_eq!(records[0].resource["service.name"], "v3-cli");
    assert_eq!(records[0].attribute("petri.ticks_requested"), Some("30"));
    assert_eq!(
        records[0].attribute("petri.metrics_interval_ms"),
        Some("1000")
    );
    assert_eq!(records[0].attribute("petri.tick_traces"), Some("on"));
    assert_eq!(records[2].attribute("petri.status"), Some("completed"));
    assert_eq!(records[2].attribute("petri.tick"), Some("30"));
    // Thirty ticks fall inside one 1,000 ms interval: the run-end snapshot is
    // the only one.
    let ticks: Vec<Option<u64>> = receiver
        .snapshots()
        .iter()
        .filter(|snapshot| snapshot.run_id() == Some(run_id.as_str()))
        .map(|snapshot| snapshot.tick())
        .collect();
    assert_eq!(ticks, [Some(30)]);
    // So is its trace: the completion one.
    let traces: Vec<(Option<u64>, Option<String>)> = tick_traces(&receiver, &run_id)
        .iter()
        .map(|trace| {
            let policy = trace.spans[0].attribute("petri.sample_policy");
            (trace.tick(), policy.map(str::to_owned))
        })
        .collect();
    assert_eq!(traces, [(Some(30), Some("run_end".to_owned()))]);
    let line = run_line(&on.stderr);
    assert_eq!(line["run"], run_id);
    // `run.started`, the genome record, the window, the snapshot, its trace
    // and `run.ended`.
    assert_eq!(line["exported"], "6");
    assert_eq!(line["snapshots"], "1");
    assert_eq!(line["traces"], "1");
    assert_eq!(line["windows"], "1");
    assert_eq!(line["recorded"], "8");

    let closed = run("on", Some(&closed_endpoint()));
    let line = run_line(&closed.stderr);
    assert_eq!(line["exported"], "0");
    assert_eq!(line["failed"], "6");

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

    let snapshot_ticks: Vec<u64> = snapshots
        .iter()
        .map(|snapshot| snapshot.tick().unwrap())
        .collect();
    let traces = tick_traces(&receiver, &run_id);
    assert!(!traces.is_empty());
    assert_eq!(run_line(&on.stderr)["traces"], traces.len().to_string());
    let mut trace_ticks = std::collections::BTreeSet::new();
    for trace in &traces {
        let tick = trace.tick().unwrap();
        assert!(snapshot_ticks.contains(&tick), "trace tick {tick}");
        assert!(trace_ticks.insert(tick), "two traces of tick {tick}");
        assert_trace_shape(trace, &run_id, tick, TICKS);
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

/// Six spans, `tick` then the phases, with IDs derived from the run and the
/// tick, the identity attributes and `petri.phase` on each, and the sample
/// policy the tick's snapshot implies.
fn assert_trace_shape(trace: &ReceivedTrace, run_id: &str, tick: u64, last_tick: u64) {
    let names: Vec<&str> = trace.spans.iter().map(|span| span.name.as_str()).collect();
    let mut expected = vec!["tick"];
    expected.extend(v3_telemetry::PHASES);
    assert_eq!(names, expected);
    let trace_id = format!("{}{tick:016x}", &run_id[..16]);
    let policy = if tick == last_tick {
        ["interval", "run_end"].as_slice()
    } else {
        ["interval"].as_slice()
    };
    for (index, span) in trace.spans.iter().enumerate() {
        assert_eq!(span.trace_id, trace_id);
        assert_eq!(span.span_id, format!("{:016x}", index + 1));
        let parent = if index == 0 {
            String::new()
        } else {
            format!("{:016x}", 1)
        };
        assert_eq!(span.parent_span_id, parent);
        assert_eq!(span.attribute("petri.run_id"), Some(run_id));
        assert_eq!(
            span.attribute("petri.tick"),
            Some(tick.to_string().as_str())
        );
        assert_eq!(span.attribute("petri.seed"), Some("7"));
        assert!(span.attribute("petri.config_digest").is_some());
        assert!(span.attribute("petri.recipe").is_some());
        assert_eq!(span.attribute("petri.world"), Some("64x64"));
        assert!(policy.contains(&span.attribute("petri.sample_policy").unwrap()));
        assert_eq!(span.attribute("petri.phase"), Some(names[index]));
        assert!(span.start_time_unix_nano <= span.end_time_unix_nano);
    }
    let root = &trace.spans[0];
    assert!(trace.spans[1..].iter().all(|span| {
        span.start_time_unix_nano >= root.start_time_unix_nano
            && span.end_time_unix_nano <= root.end_time_unix_nano
    }));
    assert_eq!(trace.resource["service.name"], "v3-cli");
}

#[test]
fn tick_traces_off_leave_the_snapshots_and_send_no_trace() {
    let off = run_with("off", None, None, LIVING);
    let receiver = Receiver::start();
    let on = run_traced(
        "on",
        Some(receiver.endpoint()),
        Some("10"),
        Some("off"),
        LIVING,
    );
    assert_eq!(on.canonical, off.canonical);
    let line = run_line(&on.stderr);
    assert_eq!(line["traces"], "0");
    assert!(line["snapshots"].parse::<u64>().unwrap() >= 2, "{line:?}");
    assert!(tick_traces(&receiver, &announced_run(&on.stderr)).is_empty());
    assert!(!receiver.snapshots().is_empty());
    let started = &receiver.records()[0];
    assert_eq!(started.attribute("petri.tick_traces"), Some("off"));
}

#[test]
fn an_invalid_tick_trace_switch_refuses_to_start_a_run_with_telemetry_on() {
    for invalid in ["yes", "OFF", "1"] {
        let output = command("on", Some(&closed_endpoint()), None, Some(invalid), SMALL);
        assert!(!output.status.success(), "{invalid}");
        assert!(output.stdout.is_empty(), "{invalid}");
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.contains(v3_telemetry::TICK_TRACES_ENV), "{stderr}");
    }
}

#[test]
fn a_run_of_zero_ticks_takes_its_completion_snapshot_and_no_trace() {
    let receiver = Receiver::start();
    let reports = v3_telemetry::ReportSink::capture();
    v3_cli::telemetry::install(
        v3_telemetry::Telemetry::start_with(v3_telemetry::Options {
            service: v3_telemetry::Service::Cli,
            endpoint: receiver.endpoint().to_owned(),
            limits: v3_telemetry::Limits::default(),
            reports: reports.clone(),
            metrics_interval: v3_telemetry::DEFAULT_METRICS_INTERVAL,
            tick_traces: v3_telemetry::Switch::On,
            windows: v3_telemetry::WindowSettings::default(),
            preset: v3_telemetry::Preset::Standard,
        }),
        None,
    );
    let mut config = v3_core::config::SimulationConfig::default();
    config.world.width = 16;
    config.world.height = 16;
    let mut out = Vec::new();
    v3_cli::run_simulation(config, 7, 0, 10, &mut out).expect("the run completes");
    let line = reports
        .lines()
        .iter()
        .find_map(|line| parse_run_line(line))
        .expect("the run's line");
    assert_eq!(line["snapshots"], "1", "{line:?}");
    assert_eq!(line["traces"], "0", "{line:?}");
    assert_eq!(receiver.snapshots().len(), 1);
    assert!(receiver.traces().is_empty());
}

#[test]
fn an_invalid_metrics_interval_refuses_to_start_a_run_with_telemetry_on() {
    for invalid in ["0", "9", "3600001", "soon"] {
        let output = command("on", Some(&closed_endpoint()), Some(invalid), None, SMALL);
        assert!(!output.status.success(), "{invalid}");
        assert!(output.stdout.is_empty(), "{invalid}");
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(
            stderr.contains(v3_telemetry::METRICS_INTERVAL_ENV),
            "{stderr}"
        );
    }
}

/// Two food types, so the typed banks hold one entry per type.
const TWO_FOODS: Shape = Shape {
    name: "two-foods",
    recipe: r##"{"world":{"width":32,"height":32,"food":{"types":[
        {"name":"Grass","color":"#3a7d44","initial_density":0.6,"initial_coverage":0.4},
        {"name":"Fruit","color":"#b8413a","initial_density":0.8,"initial_coverage":0.2}]}},
        "population":{"initial_creatures":12}}"##,
    ticks: 12,
    sample_every: 6,
};

/// A run shorter than a default window.
const SHORT: Shape = Shape {
    name: "short",
    recipe: r#"{"world":{"width":32,"height":32},"population":{"initial_creatures":12}}"#,
    ticks: 3,
    sample_every: 3,
};

/// The run's creature windows, each as its root and its tick spans.
fn windows(receiver: &Receiver, run_id: &str) -> Vec<ReceivedTrace> {
    receiver
        .traces()
        .into_iter()
        .filter(|trace| trace.run_id() == Some(run_id) && trace.spans[0].name == "creature_window")
        .collect()
}

fn windowed(receiver: &Receiver, shape: Shape, env: &[(&str, &str)]) -> RunOutput {
    let output = command_with("on", Some(receiver.endpoint()), None, None, shape, env);
    assert!(output.status.success(), "{output:?}");
    RunOutput {
        canonical: canonical(&output.stdout),
        stderr: String::from_utf8(output.stderr).expect("stderr is UTF-8"),
    }
}

#[test]
fn a_window_exports_its_creature_ticks_events_and_genome() {
    let receiver = Receiver::start();
    let on = windowed(&receiver, TWO_FOODS, &[]);
    assert_eq!(
        on.canonical,
        run_with("off", None, None, TWO_FOODS).canonical
    );
    let run_id = announced_run(&on.stderr);
    let found = windows(&receiver, &run_id);
    assert_eq!(found.len(), 1);
    let trace = &found[0];
    let root = &trace.spans[0];
    let recorded: usize = root
        .attribute("petri.ticks_recorded")
        .unwrap()
        .parse()
        .unwrap();
    assert_eq!(recorded, 8);
    assert_eq!(root.attribute("petri.end_reason"), Some("complete"));
    assert_eq!(root.attribute("petri.sample_policy"), Some("window"));
    assert_eq!(root.attribute("petri.window"), Some("0"));
    let ticks = &trace.spans[1..];
    assert_eq!(ticks.len(), recorded);
    let trace_id = format!("{}8{:015x}", &run_id[..16], 0);
    assert!(trace.spans.iter().all(|span| span.trace_id == trace_id));
    assert_eq!(root.span_id, "0000000000000001");
    for (i, span) in ticks.iter().enumerate() {
        assert_eq!(span.name, "creature_tick");
        assert_eq!(span.span_id, format!("{:016x}", (i as u64 + 1) << 8));
        assert_eq!(span.parent_span_id, root.span_id);
        assert_eq!(
            span.attribute("petri.tick"),
            Some((i + 1).to_string().as_str())
        );
        assert_eq!(span.attribute("petri.run_id"), Some(run_id.as_str()));
        for key in [
            "petri.neighbor_food.0",
            "petri.neighbor_food.1",
            "petri.food_here_by_type",
        ] {
            assert!(span.attribute(key).is_some(), "{key}");
        }
        // The founder never reads extended perception, so its banks are absent.
        for key in [
            "petri.area_food",
            "petri.area_barrier",
            "petri.area_occupancy",
            "petri.nearby_core",
            "petri.nearby_vitals",
            "petri.nearby_identity",
            "petri.area_food.0",
            "petri.area_food.1",
        ] {
            assert!(span.attribute(key).is_none(), "{key}");
        }
    }
    let events = |name| {
        ticks
            .iter()
            .map(|span| span.events_named(name).count())
            .sum::<usize>()
    };
    assert!(events("hop") > 0 && events("pass") > 0 && events("action") > 0);
    let genome = receiver
        .records()
        .into_iter()
        .find(|record| record.event_name == "creature.genome")
        .expect("the genome record");
    assert_eq!(
        genome.attribute("petri.genome_hash"),
        root.attribute("petri.genome_hash")
    );
    assert!(genome.body.as_deref().unwrap().starts_with('{'));
    let line = run_line(&on.stderr);
    assert_eq!(line["windows"], "1");
    assert_eq!(line["recorded"], recorded.to_string());

    // The same seed picks the same first creature.
    let again = windowed(&receiver, TWO_FOODS, &[]);
    let again = windows(&receiver, &announced_run(&again.stderr));
    assert_eq!(
        again[0].spans[0].attribute("petri.creature_id"),
        root.attribute("petri.creature_id")
    );
}

#[test]
fn a_run_shorter_than_its_window_exports_it_as_run_end() {
    let receiver = Receiver::start();
    let on = windowed(&receiver, SHORT, &[]);
    let run_id = announced_run(&on.stderr);
    let found = windows(&receiver, &run_id);
    assert_eq!(found.len(), 1);
    let root = &found[0].spans[0];
    assert_eq!(root.attribute("petri.end_reason"), Some("run_end"));
    assert_eq!(root.attribute("petri.ticks_recorded"), Some("3"));
    assert_eq!(root.attribute("petri.ticks_requested"), Some("8"));
    assert_eq!(run_line(&on.stderr)["recorded"], "3");
}

#[test]
fn windows_off_export_no_window_and_invalid_settings_refuse_to_start() {
    let receiver = Receiver::start();
    let on = windowed(
        &receiver,
        SHORT,
        &[(v3_telemetry::CREATURE_WINDOWS_ENV, "off")],
    );
    let run_id = announced_run(&on.stderr);
    assert!(windows(&receiver, &run_id).is_empty());
    let line = run_line(&on.stderr);
    assert_eq!(line["windows"], "0");
    assert_eq!(line["recorded"], "0");
    let started = receiver
        .records()
        .into_iter()
        .find(|r| {
            r.event_name == "run.started" && r.attribute("petri.run_id") == Some(run_id.as_str())
        })
        .unwrap();
    assert_eq!(started.attribute("petri.creature_windows"), Some("off"));
    assert_eq!(started.attribute("petri.window_ticks"), Some("8"));
    assert_eq!(started.attribute("petri.window_interval_ms"), Some("10000"));
    for (name, invalid) in [
        (v3_telemetry::CREATURE_WINDOWS_ENV, "maybe"),
        (v3_telemetry::WINDOW_TICKS_ENV, "65"),
        (v3_telemetry::WINDOW_INTERVAL_ENV, "5"),
    ] {
        let output = command_with(
            "on",
            Some(&closed_endpoint()),
            None,
            None,
            SHORT,
            &[(name, invalid)],
        );
        assert!(!output.status.success(), "{name}");
        assert!(output.stdout.is_empty(), "{name}");
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.contains(name), "{stderr}");
    }
}

/// The start line's settings, as `key=value` fields.
fn start_line(stderr: &str) -> std::collections::BTreeMap<String, String> {
    stderr
        .lines()
        .find_map(|line| line.strip_prefix("telemetry: on "))
        .expect("the start line")
        .split(' ')
        .filter_map(|field| field.split_once('='))
        .map(|(key, value)| (key.to_owned(), value.to_owned()))
        .collect()
}

#[test]
fn a_preset_sets_the_base_settings_and_an_explicit_variable_overrides_one() {
    let receiver = Receiver::start();
    let on = windowed(
        &receiver,
        SHORT,
        &[
            (v3_telemetry::PRESET_ENV, "phases"),
            (v3_telemetry::METRICS_INTERVAL_ENV, "500"),
        ],
    );
    let line = start_line(&on.stderr);
    let expected = [
        ("preset", "phases"),
        ("interval_ms", "500"),
        ("tick_traces", "on"),
        ("windows", "off"),
        ("window_ticks", "8"),
        ("window_interval_ms", "10000"),
    ];
    for (key, value) in expected {
        assert_eq!(line[key], value, "{line:?}");
    }
    let run_id = announced_run(&on.stderr);
    assert!(windows(&receiver, &run_id).is_empty());
    let started = receiver
        .records()
        .into_iter()
        .find(|r| {
            r.event_name == "run.started" && r.attribute("petri.run_id") == Some(run_id.as_str())
        })
        .unwrap();
    assert_eq!(started.attribute("petri.preset"), Some("phases"));
    assert_eq!(started.attribute("petri.metrics_interval_ms"), Some("500"));
    assert_eq!(started.attribute("petri.creature_windows"), Some("off"));

    let output = command_with(
        "on",
        Some(&closed_endpoint()),
        None,
        None,
        SHORT,
        &[(v3_telemetry::PRESET_ENV, "full")],
    );
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains(v3_telemetry::PRESET_ENV), "{stderr}");
}
