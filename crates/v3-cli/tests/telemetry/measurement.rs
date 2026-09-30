//! Measurement commands (T21.F06): `bench`, `recruitment` and
//! `input-opportunity` with telemetry on record the measurement and, for
//! `bench`, each seed at region boundaries only; the exporter sends nothing
//! before the measurement has ended; the deterministic output is the same
//! with telemetry off, on, and on against a closed port.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::Value;
use v3_telemetry::testing::{closed_endpoint, parse_run_line, ReceivedRecord, Receiver};

/// The `petri.total.*` fields of a seed's `run.ended`.
const TOTALS: [&str; 13] = [
    "ticks",
    "creature_ticks",
    "mesh_hops",
    "vm_steps",
    "graph_relax_iters",
    "plasticity_updates",
    "actions_applied",
    "births",
    "pass_cap_hits",
    "passes",
    "decided_passes",
    "final_population",
    "extinction_tick",
];

/// A fresh directory under the test target directory; removed on drop.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let path = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("t21-f06-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("create scratch");
        Self(path.canonicalize().expect("canonical scratch"))
    }

    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// `v3-cli <args>` in `cwd` with every telemetry variable scrubbed, then
/// `endpoint` and `env` set.
fn cli(cwd: &Path, args: &[&str], endpoint: Option<&str>, env: &[(&str, &str)]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_v3-cli"));
    command.current_dir(cwd).args(args);
    for name in [
        v3_telemetry::SWITCH_ENV,
        v3_telemetry::ENDPOINT_ENV,
        v3_telemetry::METRICS_INTERVAL_ENV,
        v3_telemetry::TICK_TRACES_ENV,
        v3_telemetry::CREATURE_WINDOWS_ENV,
        v3_telemetry::WINDOW_TICKS_ENV,
        v3_telemetry::WINDOW_INTERVAL_ENV,
    ] {
        command.env_remove(name);
    }
    if let Some(endpoint) = endpoint {
        command.env(v3_telemetry::ENDPOINT_ENV, endpoint);
    }
    command.envs(env.iter().copied());
    command.output().expect("v3-cli runs")
}

/// The fixture workload's arguments with the telemetry `switch`, writing to
/// `raw` and `summary`, then `extra`.
fn sweep_args<'a>(
    switch: &'a str,
    raw: &'a str,
    summary: &'a str,
    extra: &[&'a str],
) -> Vec<&'a str> {
    let mut args = vec![
        "--telemetry",
        switch,
        "bench",
        "--profile",
        "sweep",
        "--width",
        "32",
        "--height",
        "32",
        "--founders",
        "16",
        "--seeds",
        "11,22",
        "--ticks",
        "20",
        "--feature",
        "t21-f06-fixture",
        "--out",
        raw,
        "--summary-out",
        summary,
    ];
    args.extend_from_slice(extra);
    args
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(path).expect("read JSON")).expect("parse JSON")
}

/// Compact JSON with keys sorted (`serde_json::Value` maps are ordered).
fn compact(value: &Value) -> String {
    serde_json::to_string(value).expect("value serializes")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).expect("UTF-8")
}

/// The receiver's records, by timestamp.
fn by_time(receiver: &Receiver) -> Vec<ReceivedRecord> {
    let mut records = receiver.records();
    records.sort_by_key(|record| record.time_unix_nano);
    records
}

fn run_lines(stderr: &str) -> Vec<BTreeMap<String, String>> {
    stderr.lines().filter_map(parse_run_line).collect()
}

fn unix_ns(time: SystemTime) -> u64 {
    u64::try_from(time.duration_since(UNIX_EPOCH).unwrap().as_nanos()).unwrap()
}

/// The receiver's first request arrived no earlier than `ended`, the
/// `measurement.ended` record.
fn assert_exported_after(receiver: &Receiver, ended: &ReceivedRecord) {
    let first = receiver.first_request_at().expect("a request arrived");
    assert!(
        unix_ns(first) >= ended.time_unix_nano,
        "first request {} before measurement.ended {}",
        unix_ns(first),
        ended.time_unix_nano
    );
}

/// The measurement `id` has one run line, and the run lines follow every
/// line of the command's own stderr output.
fn assert_line_last(stderr: &str, id: &str) {
    let lines: Vec<&str> = stderr.lines().collect();
    let own = lines
        .iter()
        .rposition(|line| !line.starts_with("telemetry: "))
        .expect("the command wrote to stderr");
    let first_run_line = lines
        .iter()
        .position(|line| line.starts_with("telemetry: run="))
        .expect("a run line");
    assert!(own < first_run_line, "{stderr}");
    let mine = run_lines(stderr)
        .into_iter()
        .filter(|line| line["run"] == id)
        .count();
    assert_eq!(mine, 1, "{stderr}");
}

/// The eleven `Totals` fields (the leading `TOTALS` entries) summed over a
/// written summary's `deterministic.per_seed` rows, as compact key-sorted JSON.
fn summed_totals(summary: &Value) -> String {
    let rows = summary["deterministic"]["per_seed"].as_array().unwrap();
    assert!(!rows.is_empty(), "the summary keeps its per_seed rows");
    let sums: serde_json::Map<String, Value> = TOTALS[..11]
        .iter()
        .map(|field| {
            let sum: u64 = rows.iter().map(|row| row[*field].as_u64().unwrap()).sum();
            ((*field).to_owned(), Value::from(sum))
        })
        .collect();
    compact(&Value::Object(sums))
}

/// A sweep's `deterministic` blocks (raw report, summary) and stdout.
fn deterministic(scratch: &Scratch, output: &Output) -> (String, String, String) {
    let raw = read_json(&scratch.path("raw.json"));
    let summary = read_json(&scratch.path("summary.json"));
    (
        compact(&raw["deterministic"]),
        compact(&summary["deterministic"]),
        text(&output.stdout),
    )
}

#[test]
fn bench_records_region_boundaries_only_and_keeps_its_deterministic_output() {
    let scratch = Scratch::new("fixture");
    let raw_path = scratch.path("raw.json");
    let summary_path = scratch.path("summary.json");
    let (raw_arg, summary_arg) = (
        raw_path.to_str().unwrap().to_owned(),
        summary_path.to_str().unwrap().to_owned(),
    );

    let off = cli(
        &scratch.0,
        &sweep_args("off", &raw_arg, &summary_arg, &[]),
        None,
        &[],
    );
    assert!(off.status.success(), "{off:?}");
    assert!(!text(&off.stderr).contains("telemetry:"));
    let off_blocks = deterministic(&scratch, &off);

    let receiver = Receiver::start();
    let on = cli(
        &scratch.0,
        &sweep_args("on", &raw_arg, &summary_arg, &[]),
        Some(receiver.endpoint()),
        &[
            (v3_telemetry::METRICS_INTERVAL_ENV, "10"),
            (v3_telemetry::TICK_TRACES_ENV, "on"),
            (v3_telemetry::CREATURE_WINDOWS_ENV, "on"),
        ],
    );
    assert_eq!(on.status.code(), Some(0), "{on:?}");
    let on_blocks = deterministic(&scratch, &on);
    let raw = read_json(&raw_path);
    let summary = read_json(&summary_path);

    let closed = cli(
        &scratch.0,
        &sweep_args("on", &raw_arg, &summary_arg, &[]),
        Some(&closed_endpoint()),
        &[],
    );
    assert_eq!(closed.status.code(), Some(0), "{closed:?}");
    assert_eq!(on_blocks, off_blocks);
    assert_eq!(deterministic(&scratch, &closed), off_blocks);

    // Exactly the six boundary records, and nothing sampled inside the loop.
    let records = by_time(&receiver);
    let names: Vec<&str> = records.iter().map(|r| r.event_name.as_str()).collect();
    assert_eq!(
        names,
        [
            "measurement.started",
            "run.started",
            "run.ended",
            "run.started",
            "run.ended",
            "measurement.ended"
        ]
    );
    assert!(receiver.snapshots().is_empty());
    assert!(receiver.traces().is_empty());
    for stderr in [text(&on.stderr), text(&closed.stderr)] {
        let lines = run_lines(&stderr);
        assert_eq!(lines.len(), 3, "{stderr}");
        assert!(
            lines.iter().all(|line| line["snapshots"] == "0"),
            "{stderr}"
        );
    }

    let (started, ended) = (&records[0], &records[5]);
    let measurement = started.attribute("petri.run_id").unwrap();
    assert_eq!(ended.attribute("petri.run_id"), Some(measurement));
    for (key, value) in [
        ("petri.command", "bench"),
        ("petri.profile", "sweep"),
        ("petri.feature", "t21-f06-fixture"),
        ("petri.seeds", "11,22"),
        ("petri.tick", "0"),
    ] {
        assert_eq!(started.attribute(key), Some(value), "{key}");
    }
    assert_eq!(
        started.attribute("petri.config_digest"),
        raw["measurement_evidence"]["effective_config_digest"].as_str()
    );
    assert_eq!(started.attribute("petri.threads"), None);

    for (index, seed) in [(0_usize, 11_u64), (1, 22)] {
        let (run_started, run_ended) = (&records[1 + 2 * index], &records[2 + 2 * index]);
        let id = run_started.attribute("petri.run_id").unwrap();
        assert_ne!(id, measurement);
        assert_eq!(run_ended.attribute("petri.run_id"), Some(id));
        assert_eq!(
            run_started.attribute("petri.seed"),
            Some(seed.to_string().as_str())
        );
        assert_eq!(run_started.attribute("petri.ticks_requested"), Some("20"));
        assert_eq!(run_started.resource, started.resource, "one invocation");
        let wall_ms = raw["environment"]["wall_clock_ms_per_seed"][index]["wall_clock_ms"]
            .as_f64()
            .unwrap();
        let captured_ns = run_ended.time_unix_nano - run_started.time_unix_nano;
        assert!(captured_ns as f64 >= wall_ms * 1e6, "seed {seed}");
        let per_seed = &raw["deterministic"]["per_seed"][index];
        assert_eq!(run_ended.attribute("petri.status"), Some("completed"));
        assert_eq!(
            run_ended.attribute("petri.tick"),
            Some(per_seed["ticks"].to_string().as_str())
        );
        for field in TOTALS {
            let key = format!("petri.total.{field}");
            match &per_seed[field] {
                Value::Null => assert_eq!(run_ended.attribute(&key), None, "{key}"),
                value => assert_eq!(
                    run_ended.attribute(&key),
                    Some(value.to_string().as_str()),
                    "{key}"
                ),
            }
        }
    }
    assert!(started.time_unix_nano <= records[1].time_unix_nano);
    assert!(ended.time_unix_nano >= records[4].time_unix_nano);

    assert_eq!(ended.attribute("petri.exit_code"), Some("0"));
    assert_eq!(ended.attribute("petri.severe"), Some("false"));
    assert_eq!(
        ended.attribute("petri.summary_path"),
        Some(summary_arg.as_str())
    );
    assert_eq!(
        ended.attribute("petri.raw_sha256"),
        summary["raw"]["sha256"].as_str()
    );
    assert_eq!(
        ended.attribute("petri.raw_bytes"),
        Some(summary["raw"]["bytes"].to_string().as_str())
    );
    // The totals block: the `Totals` fields summed over the written summary's
    // `per_seed` rows, which equal the raw report's `deterministic.totals`.
    let summed = summed_totals(&summary);
    assert_eq!(ended.body.as_deref(), Some(summed.as_str()));
    assert_eq!(summed, compact(&raw["deterministic"]["totals"]));
    assert_exported_after(&receiver, ended);
}

/// Divides every positive `per_creature_tick` reading of the raw report at
/// `from` by ten and writes it to `to`: the current run is then a severe
/// regression against it.
fn tenth_of(from: &Path, to: &Path) {
    let mut report = read_json(from);
    let readings = report["deterministic"]["per_creature_tick"]
        .as_object_mut()
        .unwrap();
    for value in readings.values_mut() {
        let reading: f64 = value.as_str().unwrap().parse().unwrap();
        if reading > 0.0 {
            *value = Value::from(format!("{:.6}", reading / 10.0));
        }
    }
    std::fs::write(to, serde_json::to_vec(&report).unwrap()).unwrap();
}

/// The measurement records of the one invocation `receiver` heard.
fn measurement_pair(receiver: &Receiver) -> (ReceivedRecord, ReceivedRecord) {
    let records: Vec<ReceivedRecord> = by_time(receiver)
        .into_iter()
        .filter(|record| record.event_name.starts_with("measurement."))
        .collect();
    let names: Vec<&str> = records.iter().map(|r| r.event_name.as_str()).collect();
    assert_eq!(names, ["measurement.started", "measurement.ended"]);
    (records[0].clone(), records[1].clone())
}

#[test]
fn bench_exit_paths_end_the_measurement_with_their_code() {
    let scratch = Scratch::new("exits");
    // An `--out` naming a directory resolves, so the failure lands when the
    // built report is written.
    std::fs::create_dir(scratch.path("occupied")).unwrap();
    let receiver = Receiver::start();
    let failed = cli(
        &scratch.0,
        &sweep_args("on", "occupied", "failed-summary.json", &[]),
        Some(receiver.endpoint()),
        &[],
    );
    assert_eq!(failed.status.code(), Some(1), "{failed:?}");
    let (_, ended) = measurement_pair(&receiver);
    assert_eq!(ended.attribute("petri.exit_code"), Some("1"));
    assert_eq!(ended.body, None);
    assert_eq!(ended.attribute("petri.summary_path"), None);
    assert_line_last(
        &text(&failed.stderr),
        ended.attribute("petri.run_id").unwrap(),
    );

    let reference = cli(
        &scratch.0,
        &sweep_args("off", "reference-raw.json", "reference-summary.json", &[]),
        None,
        &[],
    );
    assert!(reference.status.success(), "{reference:?}");
    tenth_of(
        &scratch.path("reference-raw.json"),
        &scratch.path("tenth.json"),
    );
    let receiver = Receiver::start();
    let severe = cli(
        &scratch.0,
        &sweep_args(
            "on",
            "severe.json",
            "severe-summary.json",
            &["--compare", "tenth.json"],
        ),
        Some(receiver.endpoint()),
        &[],
    );
    assert_eq!(severe.status.code(), Some(3), "{severe:?}");
    let (_, ended) = measurement_pair(&receiver);
    assert_eq!(ended.attribute("petri.exit_code"), Some("3"));
    assert_eq!(ended.attribute("petri.severe"), Some("true"));
    let raw = read_json(&scratch.path("severe.json"));
    assert_eq!(
        ended.body.as_deref(),
        Some(compact(&raw["deterministic"]["totals"]).as_str())
    );
    assert_line_last(
        &text(&severe.stderr),
        ended.attribute("petri.run_id").unwrap(),
    );

    let receiver = Receiver::start();
    let refused = cli(
        &scratch.0,
        &sweep_args("on", "refused.json", "refused-summary.json", &[]),
        Some(receiver.endpoint()),
        &[(v3_telemetry::METRICS_INTERVAL_ENV, "9")],
    );
    assert_eq!(refused.status.code(), Some(1), "{refused:?}");
    assert!(refused.stdout.is_empty());
    assert!(text(&refused.stderr).contains(v3_telemetry::METRICS_INTERVAL_ENV));
    std::thread::sleep(Duration::from_millis(100));
    assert!(receiver.records().is_empty());
    assert!(!scratch.path("refused.json").exists());
}

/// A git checkout with one empty commit, as the recruitment CLI test builds.
fn checkout(scratch: &Scratch) {
    let git = |args: &[&str]| {
        let output = Command::new("git")
            .current_dir(&scratch.0)
            .args(args)
            .output()
            .unwrap();
        assert!(output.status.success(), "{}", text(&output.stderr));
    };
    git(&["init", "--quiet"]);
    git(&[
        "-c",
        "user.name=Test",
        "-c",
        "user.email=test@example.invalid",
        "-c",
        "core.hooksPath=/dev/null",
        "commit",
        "--quiet",
        "--allow-empty",
        "-m",
        "fixture",
    ]);
}

#[test]
fn assays_record_one_measurement_with_their_summary_totals() {
    let scratch = Scratch::new("recruitment");
    checkout(&scratch);
    let raw_path = scratch.path("raw.json");
    let summary_path = scratch.path("summary.json");
    let receiver = Receiver::start();
    let output = cli(
        &scratch.0,
        &[
            "--telemetry",
            "on",
            "recruitment",
            "--feature",
            "t21-f06-fixture",
            "--pilot",
            "--threads",
            "1",
            "--wall-cap-secs",
            "0",
            "--out",
            raw_path.to_str().unwrap(),
            "--summary-out",
            summary_path.to_str().unwrap(),
        ],
        Some(receiver.endpoint()),
        &[],
    );
    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert_eq!(receiver.records().len(), 2);
    let (started, ended) = measurement_pair(&receiver);
    let summary = read_json(&summary_path);
    assert_eq!(started.attribute("petri.command"), Some("recruitment"));
    assert_eq!(started.attribute("petri.pilot"), Some("true"));
    assert_eq!(started.attribute("petri.threads"), Some("1"));
    assert_eq!(
        started.attribute("petri.config_digest"),
        summary["config_digest"].as_str()
    );
    assert_eq!(ended.attribute("petri.exit_code"), Some("3"));
    assert_eq!(ended.attribute("petri.incomplete"), Some("true"));
    assert_eq!(
        ended.attribute("petri.stop_reason"),
        summary["stop_reason"].as_str()
    );
    assert!(ended.attribute("petri.stop_reason").is_some());
    assert_eq!(ended.attribute("petri.summary_path"), summary_path.to_str());
    assert_eq!(
        ended.attribute("petri.raw_sha256"),
        summary["raw"]["sha256"].as_str()
    );
    assert_eq!(
        ended.attribute("petri.raw_bytes"),
        Some(summary["raw"]["bytes"].to_string().as_str())
    );
    let counts = serde_json::json!({
        "expected_proposals": summary["expected_proposals"],
        "lineage_count": summary["lineage_count"],
        "proposal_count": summary["proposal_count"],
    });
    assert_eq!(ended.body.as_deref(), Some(compact(&counts).as_str()));
    assert_exported_after(&receiver, &ended);
    let stderr = text(&output.stderr);
    assert!(stderr.contains("incomplete:"), "{stderr}");
    assert_line_last(&stderr, ended.attribute("petri.run_id").unwrap());

    let blocker = scratch.path("blocker");
    std::fs::write(&blocker, "").unwrap();
    let unwritable = blocker.join("raw.json");
    let receiver = Receiver::start();
    let output = cli(
        &scratch.0,
        &[
            "--telemetry",
            "on",
            "input-opportunity",
            "--feature",
            "t21-f06-fixture",
            "--pilot",
            "--out",
            unwritable.to_str().unwrap(),
        ],
        Some(receiver.endpoint()),
        &[],
    );
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(output.stdout.is_empty(), "no replicate ran");
    let (started, ended) = measurement_pair(&receiver);
    assert_eq!(
        started.attribute("petri.command"),
        Some("input-opportunity")
    );
    assert_eq!(started.attribute("petri.config_digest"), None);
    assert_eq!(ended.attribute("petri.exit_code"), Some("1"));
    assert_eq!(ended.body, None);
    assert_line_last(
        &text(&output.stderr),
        ended.attribute("petri.run_id").unwrap(),
    );
}
