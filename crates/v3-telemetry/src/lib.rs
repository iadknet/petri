//! Run identity and bounded, best-effort OTLP log export for the Petri
//! binaries (T21.F01).
//!
//! A [`Telemetry`] is either off (every call is a no-op) or on, in which case
//! it owns an OpenTelemetry logger whose records go through a bounded queue to
//! one exporter thread speaking OTLP/HTTP with the blocking client. Nothing
//! here writes to stdout, draws from a simulation RNG or changes a run: IDs
//! come from OS entropy and the flag enters no recorded configuration.
//!
//! When on, the binary writes one stderr line when its first run starts
//! (`telemetry: on endpoint=… invocation=… run=…`) and one per run once each of
//! the run's records has been exported, failed, dropped or abandoned
//! (`telemetry: run=… exported=… failed=… dropped=… abandoned=… bytes=…
//! self_time_us=… flush_ms=… snapshots=… traces=…`).
//!
//! A run's snapshots (T21.F02, [`Telemetry::snapshot`]) export its cumulative
//! counters and per-tick values as OTLP metrics through the same queue, and
//! the tick traces taken with its interval and completion snapshots (T21.F03,
//! [`Telemetry::tick_snapshot`]) export one span per timed phase of the tick.

mod export;
mod metrics;
mod queue;
pub mod testing;
mod trace;
mod windows;

use std::fmt;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use opentelemetry::logs::{AnyValue, LogRecord as _, Logger as _, LoggerProvider as _, Severity};
use opentelemetry::KeyValue;
use opentelemetry_proto::transform::common::tonic::ResourceAttributesWithSchema;
use opentelemetry_sdk::logs::{LogExporter, SdkLogger, SdkLoggerProvider};
use opentelemetry_sdk::Resource;
use prost::Message as _;
use rand::RngCore;
use v3_core::config::SimulationConfig;
use v3_core::simulation::Simulation;

use crate::export::Rejections;
use crate::metrics::{Census, Moment, Taken};
use crate::queue::{PostEncoded, QueueProcessor, RunKey, Shared, Signal};

pub use crate::metrics::{
    resolve_metrics_interval, Trigger, DEFAULT_METRICS_INTERVAL, METRICS_INTERVAL_ENV,
};
pub use crate::trace::{
    resolve_tick_traces, TickSample, FIRST_UNTRACED_TICK, MAX_TICK_TRACES_PER_RUN, PHASES,
    TICK_TRACES_ENV,
};
pub use crate::windows::{
    WindowSettings, CREATURE_WINDOWS_ENV, MAX_EVENTS_PER_SAMPLE, MAX_SAMPLES_PER_RUN,
    MAX_SAMPLE_BYTES_PER_RUN, WINDOW_BUDGET, WINDOW_INTERVAL_ENV, WINDOW_TICKS_ENV,
};
use v3_core::runtime::trace::recording::ActiveTrace;

/// The environment variable that switches telemetry when no flag is given.
pub const SWITCH_ENV: &str = "PETRI_TELEMETRY";
/// The OTLP base endpoint variable; `/v1/logs` is appended for logs.
pub const ENDPOINT_ENV: &str = "OTEL_EXPORTER_OTLP_ENDPOINT";
/// The endpoint used when [`ENDPOINT_ENV`] is unset or empty.
pub const DEFAULT_ENDPOINT: &str = "http://127.0.0.1:4318";
/// The commit this crate was built from (see `build.rs`).
pub const BUILD_REVISION: &str = env!("PETRI_BUILD_REVISION_RESOLVED");

pub(crate) const RUN_ID_KEY: &str = "petri.run_id";

/// Whether a process exports telemetry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Switch {
    On,
    #[default]
    Off,
}

impl FromStr for Switch {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "on" => Ok(Self::On),
            "off" => Ok(Self::Off),
            other => Err(format!("expected `on` or `off`, found `{other}`")),
        }
    }
}

impl fmt::Display for Switch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::On => "on",
            Self::Off => "off",
        })
    }
}

/// The flag beats [`SWITCH_ENV`], which beats the default `off`. An unset or
/// empty variable counts as absent; any other value must be `on` or `off`.
pub fn resolve_switch(flag: Option<Switch>, env: Option<&str>) -> Result<Switch, String> {
    if let Some(flag) = flag {
        return Ok(flag);
    }
    match env.map(str::trim).filter(|value| !value.is_empty()) {
        Some(value) => value
            .parse()
            .map_err(|error| format!("invalid {SWITCH_ENV}: {error}")),
        None => Ok(Switch::Off),
    }
}

/// [`resolve_switch`] against the process environment.
pub fn switch_from_env(flag: Option<Switch>) -> Result<Switch, String> {
    resolve_switch(flag, std::env::var(SWITCH_ENV).ok().as_deref())
}

/// The binary a record came from; its `service.name`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Service {
    Cli,
    Server,
    Lab,
}

impl Service {
    pub fn name(self) -> &'static str {
        match self {
            Self::Cli => "v3-cli",
            Self::Server => "v3-server",
            Self::Lab => "v3-lab",
        }
    }
}

/// The exporter's bounds. [`Limits::default`] is the contract; tests shorten
/// the timeouts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    pub max_queue_records: usize,
    pub max_queue_bytes: u64,
    pub max_body_bytes: u64,
    pub batch_records: usize,
    pub request_timeout: Duration,
    pub flush_timeout: Duration,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_queue_records: 2_048,
            max_queue_bytes: 8 * 1024 * 1024,
            max_body_bytes: 4 * 1024 * 1024,
            batch_records: 512,
            request_timeout: Duration::from_secs(5),
            flush_timeout: Duration::from_secs(10),
        }
    }
}

/// Where the self-report lines go: stderr, or a buffer a test reads.
#[derive(Debug, Clone, Default)]
pub enum ReportSink {
    #[default]
    Stderr,
    Capture(Arc<Mutex<Vec<String>>>),
}

impl ReportSink {
    pub fn capture() -> Self {
        Self::Capture(Arc::default())
    }

    /// The captured lines so far; empty for [`ReportSink::Stderr`].
    pub fn lines(&self) -> Vec<String> {
        match self {
            Self::Stderr => Vec::new(),
            Self::Capture(lines) => lines
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .clone(),
        }
    }

    fn write(&self, line: &str) {
        match self {
            Self::Stderr => eprintln!("{line}"),
            Self::Capture(lines) => lines
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .push(line.to_owned()),
        }
    }
}

/// How an enabled [`Telemetry`] is set up.
#[derive(Debug, Clone)]
pub struct Options {
    pub service: Service,
    /// OTLP base endpoint, without the `/v1/logs` or `/v1/metrics` path.
    pub endpoint: String,
    pub limits: Limits,
    pub reports: ReportSink,
    /// The least wall time between a run's interval snapshots.
    pub metrics_interval: Duration,
    /// Whether interval and completion snapshots carry a tick trace.
    pub tick_traces: Switch,
    /// Creature windows (T21.F04).
    pub windows: WindowSettings,
}

impl Options {
    /// Stderr reports, contract limits, the endpoint from [`ENDPOINT_ENV`], the
    /// interval from [`METRICS_INTERVAL_ENV`] and the tick-trace switch from
    /// [`TICK_TRACES_ENV`], both of which must be valid.
    pub fn from_env(service: Service) -> Result<Self, String> {
        let endpoint = std::env::var(ENDPOINT_ENV)
            .ok()
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| DEFAULT_ENDPOINT.to_owned());
        let metrics_interval =
            resolve_metrics_interval(std::env::var(METRICS_INTERVAL_ENV).ok().as_deref())?;
        let tick_traces = resolve_tick_traces(std::env::var(TICK_TRACES_ENV).ok().as_deref())?;
        let windows = WindowSettings::from_env()?;
        Ok(Self {
            service,
            endpoint,
            limits: Limits::default(),
            reports: ReportSink::Stderr,
            metrics_interval,
            tick_traces,
            windows,
        })
    }
}

/// How a run ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndStatus {
    Completed,
    Reset,
    Shutdown,
}

impl EndStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Reset => "reset",
            Self::Shutdown => "shutdown",
        }
    }
}

/// Whether ending a run waits for its records (the CLI's end-of-run flush) or
/// returns at once and lets the line follow (a server reset).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flush {
    Wait,
    Background,
}

/// A server run's status, as `run.state` reports it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunState {
    Idle,
    Running,
    Paused,
}

impl RunState {
    fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Running => "running",
            Self::Paused => "paused",
        }
    }
}

/// What `run.started` records about a freshly seeded run.
#[derive(Debug, Clone, Copy)]
pub struct RunStart<'a> {
    pub seed: u64,
    pub config: &'a SimulationConfig,
    /// The `--config` path, when one was given.
    pub recipe: Option<&'a str>,
    pub tick: u64,
    pub ticks_requested: Option<u64>,
    pub sample_every: Option<u64>,
}

/// The identity every record of one run carries, and its snapshot cadence.
#[derive(Debug, Clone)]
pub struct RunHandle {
    key: RunKey,
    id: String,
    seed: u64,
    world: String,
    recipe: Option<String>,
    config_digest: String,
    started: Instant,
    /// The run's start in Unix nanoseconds, carried by its cumulative sums.
    started_ns: u64,
    last_snapshot: Option<Taken>,
    last_stamp_ns: Option<u64>,
    /// Tick traces captured, up to [`MAX_TICK_TRACES_PER_RUN`].
    traces_taken: u64,
    /// Creature windows and manual samples (T21.F04).
    samples: windows::Samples,
}

impl RunHandle {
    /// The run ID: 32 lowercase hex digits.
    pub fn id(&self) -> &str {
        &self.id
    }
}

/// Telemetry for one process; cheap to clone, a no-op when off.
#[derive(Debug, Clone, Default)]
pub struct Telemetry {
    active: Option<Arc<Active>>,
}

struct Active {
    // Kept so the logger's provider outlives every emit.
    _provider: SdkLoggerProvider,
    logger: SdkLogger,
    shared: Arc<Shared>,
    endpoint: String,
    invocation_id: String,
    announced: AtomicBool,
    /// The log resource's attributes, for snapshot requests.
    resource: Vec<opentelemetry_proto::tonic::common::v1::KeyValue>,
    metrics_interval: Duration,
    tick_traces: Switch,
    windows: WindowSettings,
}

impl fmt::Debug for Active {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Active")
            .field("endpoint", &self.endpoint)
            .field("invocation_id", &self.invocation_id)
            .finish_non_exhaustive()
    }
}

/// 128 bits of OS entropy as 32 lowercase hex digits.
fn entropy_id() -> (u128, String) {
    let mut bytes = [0_u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    let value = u128::from_be_bytes(bytes);
    (value, format!("{value:032x}"))
}

/// `duration` in nanoseconds, saturating at `u64::MAX`.
pub(crate) fn saturating_nanos(duration: Duration) -> u64 {
    u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
}

/// `time` in Unix nanoseconds; `0` before the epoch.
fn unix_ns(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH).map_or(0, saturating_nanos)
}

/// An integer attribute, or its decimal string when it exceeds `i64`.
pub(crate) fn u64_value(value: u64) -> AnyValue {
    i64::try_from(value).map_or_else(|_| AnyValue::from(value.to_string()), AnyValue::Int)
}

impl Telemetry {
    /// Telemetry that does nothing.
    pub fn off() -> Self {
        Self::default()
    }

    /// Off for [`Switch::Off`]; otherwise exports to the environment's
    /// endpoint and reports on stderr. An invalid [`METRICS_INTERVAL_ENV`] or
    /// [`TICK_TRACES_ENV`] refuses to start when the switch is on.
    pub fn start(service: Service, switch: Switch) -> Result<Self, String> {
        match switch {
            Switch::Off => Ok(Self::off()),
            Switch::On => Options::from_env(service).map(Self::start_with),
        }
    }

    /// Enabled telemetry with explicit options. When the exporter cannot be
    /// built, it says so on the report sink and returns telemetry that is off.
    pub fn start_with(options: Options) -> Self {
        Self::start_otlp(options, false)
    }

    /// [`Telemetry::start_with`]; with `held`, the worker takes no batch until
    /// released (tests).
    fn start_otlp(options: Options, held: bool) -> Self {
        let rejections = Arc::<Rejections>::default();
        match export::build(
            &options.endpoint,
            options.limits.request_timeout,
            Arc::clone(&rejections),
        ) {
            Ok((exporter, encoded)) => {
                Self::with_exporter(options, exporter, encoded, rejections, held)
            }
            Err(error) => {
                options
                    .reports
                    .write(&format!("telemetry: off (exporter not built: {error})"));
                Self::off()
            }
        }
    }

    fn with_exporter<E, P>(
        options: Options,
        mut exporter: E,
        encoded: P,
        rejections: Arc<Rejections>,
        held: bool,
    ) -> Self
    where
        E: LogExporter + 'static,
        P: PostEncoded + Send + 'static,
    {
        let (_, invocation_id) = entropy_id();
        let resource = Resource::builder_empty()
            .with_attributes([
                KeyValue::new("service.name", options.service.name()),
                KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
                KeyValue::new("process.pid", i64::from(std::process::id())),
                KeyValue::new("petri.invocation_id", invocation_id.clone()),
                KeyValue::new("petri.build_revision", BUILD_REVISION),
            ])
            .build();
        exporter.set_resource(&resource);
        let proto_resource = ResourceAttributesWithSchema::from(&resource).attributes.0;
        let shared = Shared::new(options.limits, options.reports, held);
        let worker_shared = Arc::clone(&shared);
        let spawned = std::thread::Builder::new()
            .name("petri-telemetry".to_owned())
            .spawn(move || queue::run_worker(&worker_shared, exporter, encoded, &rejections));
        if let Err(error) = spawned {
            shared
                .sink()
                .write(&format!("telemetry: off (worker not started: {error})"));
            return Self::off();
        }
        let provider = SdkLoggerProvider::builder()
            .with_resource(resource)
            .with_log_processor(QueueProcessor {
                shared: Arc::clone(&shared),
            })
            .build();
        let logger = provider.logger("petri");
        Self {
            active: Some(Arc::new(Active {
                _provider: provider,
                logger,
                shared,
                endpoint: options.endpoint,
                invocation_id,
                announced: AtomicBool::new(false),
                resource: proto_resource,
                metrics_interval: options.metrics_interval,
                tick_traces: options.tick_traces,
                windows: options.windows,
            })),
        }
    }

    pub fn is_on(&self) -> bool {
        self.active.is_some()
    }

    /// Draws a run ID and emits `run.started`; `None` when off. The first run
    /// also writes the process's `telemetry: on` line.
    pub fn begin_run(&self, start: RunStart<'_>) -> Option<RunHandle> {
        let active = self.active.as_ref()?;
        let began = Instant::now();
        let (key, id) = entropy_id();
        let run = RunHandle {
            key,
            id,
            seed: start.seed,
            world: format!("{}x{}", start.config.world.width, start.config.world.height),
            recipe: start.recipe.map(str::to_owned),
            config_digest: v3_core::config::config_digest(start.config),
            started: began,
            started_ns: unix_ns(SystemTime::now()),
            last_snapshot: None,
            last_stamp_ns: None,
            traces_taken: 0,
            samples: windows::Samples::default(),
        };
        active.shared.register_run(key, run.id.clone());
        if !active.announced.swap(true, Ordering::Relaxed) {
            active.shared.sink().write(&format!(
                "telemetry: on endpoint={} invocation={} run={}",
                active.endpoint, active.invocation_id, run.id
            ));
        }
        let mut extra = Vec::new();
        if let Some(ticks) = start.ticks_requested {
            extra.push(("petri.ticks_requested", u64_value(ticks)));
        }
        if let Some(every) = start.sample_every {
            extra.push(("petri.sample_every", u64_value(every)));
        }
        let interval_ms = u64::try_from(active.metrics_interval.as_millis()).unwrap_or(u64::MAX);
        extra.push(("petri.metrics_interval_ms", u64_value(interval_ms)));
        extra.push((
            "petri.tick_traces",
            AnyValue::from(active.tick_traces.to_string()),
        ));
        extra.push((
            "petri.creature_windows",
            AnyValue::from(active.windows.switch.to_string()),
        ));
        extra.push((
            "petri.window_ticks",
            AnyValue::Int(i64::from(active.windows.ticks)),
        ));
        extra.push((
            "petri.window_interval_ms",
            u64_value(active.windows.interval_ms()),
        ));
        let body = serde_json::to_string(start.config).expect("config must serialize");
        active.emit(&run, "run.started", start.tick, extra, Some(body));
        active.shared.add_self_time(key, began.elapsed());
        Some(run)
    }

    /// Takes a snapshot of `sim` for `run` when `trigger`'s cadence rule says
    /// it is due, and offers it to the queue; returns whether one was taken.
    /// Call it before the `run.state` or `run.ended` it accompanies, on the
    /// simulation thread, with the run's current simulation.
    pub fn snapshot(&self, run: &mut RunHandle, sim: &Simulation, trigger: Trigger) -> bool {
        let Some(active) = &self.active else {
            return false;
        };
        let began = Instant::now();
        if !metrics::due(
            trigger,
            run.last_snapshot,
            run.started,
            sim.tick,
            began,
            active.metrics_interval,
        ) {
            return false;
        }
        run.last_snapshot = Some(Taken {
            tick: sim.tick,
            at: began,
        });
        let time_ns = metrics::stamp(unix_ns(SystemTime::now()), run.last_stamp_ns);
        run.last_stamp_ns = Some(time_ns);
        let request = metrics::encode(
            &active.resource,
            &run.id,
            &sim.stats,
            &Census::of(sim),
            Moment {
                tick: sim.tick,
                elapsed: run.started.elapsed(),
                time_ns,
                start_ns: run.started_ns,
            },
        );
        active
            .shared
            .offer_encoded(run.key, Signal::Metrics, request.encode_to_vec());
        active.shared.add_self_time(run.key, began.elapsed());
        true
    }

    /// [`Telemetry::snapshot`] right after the `run_tick` that produced `sim`,
    /// with `sample`'s trigger; when the snapshot is taken and tick traces are
    /// on, also takes the trace of that tick unless the run has reached
    /// [`MAX_TICK_TRACES_PER_RUN`]. Returns whether the snapshot was taken.
    /// A server transition, reset or shutdown snapshot uses
    /// [`Telemetry::snapshot`] instead: its config may postdate the tick.
    pub fn tick_snapshot(&self, run: &mut RunHandle, sim: &Simulation, sample: TickSample) -> bool {
        let trigger = match sample {
            TickSample::Interval => Trigger::Interval,
            TickSample::RunEnd => Trigger::RunEnd,
        };
        if !self.snapshot(run, sim, trigger) {
            return false;
        }
        if let Some(active) = &self.active {
            active.tick_trace(run, sim, sample);
        }
        true
    }

    /// Emits `run.state` for a status transition.
    pub fn run_state(&self, run: &RunHandle, state: RunState, tick: u64) {
        if let Some(active) = &self.active {
            let began = Instant::now();
            let extra = vec![("petri.state", AnyValue::from(state.as_str()))];
            active.emit(run, "run.state", tick, extra, None);
            active.shared.add_self_time(run.key, began.elapsed());
        }
    }

    /// Emits `run.config` for an accepted config change; later records carry
    /// the new digest.
    pub fn run_config(&self, run: &mut RunHandle, config: &SimulationConfig, tick: u64) {
        if let Some(active) = &self.active {
            let began = Instant::now();
            run.config_digest = v3_core::config::config_digest(config);
            let body = serde_json::to_string(config).expect("config must serialize");
            active.emit(run, "run.config", tick, Vec::new(), Some(body));
            active.shared.add_self_time(run.key, began.elapsed());
        }
    }

    /// Emits `run.ended`. With [`Flush::Wait`] it returns once the run's line
    /// is written, abandoning what is still pending after the flush bound.
    pub fn end_run(&self, run: RunHandle, status: EndStatus, tick: u64, flush: Flush) {
        let Some(active) = &self.active else {
            return;
        };
        let began = Instant::now();
        let mut extra = vec![
            ("petri.status", AnyValue::from(status.as_str())),
            (
                "petri.wall_seconds",
                AnyValue::Double(run.started.elapsed().as_secs_f64()),
            ),
        ];
        if run.traces_taken >= MAX_TICK_TRACES_PER_RUN {
            extra.push((
                "petri.tick_traces_capped",
                u64_value(MAX_TICK_TRACES_PER_RUN),
            ));
        }
        extra.push(("petri.samples_skipped", u64_value(run.samples.skipped)));
        if let Some(cap) = run.samples.capped {
            extra.push(("petri.windows_capped", AnyValue::from(cap.as_str())));
        }
        active.emit(&run, "run.ended", tick, extra, None);
        active.shared.add_self_time(run.key, began.elapsed());
        active.shared.mark_ended(run.key);
        if flush == Flush::Wait {
            active.shared.flush_run(run.key);
        }
    }

    /// Starts a creature window into `slot` before a tick when one is due:
    /// windows are on, neither a window nor a manual sample is active, the
    /// interval has passed since the run's last sample start, the run is
    /// below its caps and the population is not empty. Pass `slot` to
    /// `run_tick` unless a manual sample is active, then call
    /// [`Telemetry::after_tick`].
    pub fn before_tick(
        &self,
        run: &mut RunHandle,
        sim: &Simulation,
        slot: &mut Option<ActiveTrace>,
        manual_active: bool,
    ) {
        if let Some(active) = &self.active {
            active.before_tick(run, sim, slot, manual_active);
        }
    }

    /// After `run_tick`: notes the digest each recorded tick ran under, and
    /// exports and clears a window in `slot` that ended (complete, died or
    /// truncated). `manual` is the server sampler's recording, if any.
    pub fn after_tick(
        &self,
        run: &mut RunHandle,
        sim: &Simulation,
        slot: &mut Option<ActiveTrace>,
        manual: Option<&ActiveTrace>,
    ) {
        if let Some(active) = &self.active {
            active.after_tick(run, sim, slot, manual);
        }
    }

    /// A manual sample starts: exports an open window as `manual` and an
    /// unfetched sample it replaces as `replaced`, then admits `sample` for
    /// export when the interval has passed and the run is below its caps
    /// (else counts it in `petri.samples_skipped`).
    pub fn start_manual(
        &self,
        run: &mut RunHandle,
        sim: &Simulation,
        slot: &mut Option<ActiveTrace>,
        replaced: Option<&ActiveTrace>,
        sample: &ActiveTrace,
    ) {
        if let Some(active) = &self.active {
            active.start_manual(run, sim, slot, replaced, sample);
        }
    }

    /// Exports an admitted manual sample once, as `complete`, when the
    /// sampler hands it over.
    pub fn hand_over(&self, run: &mut RunHandle, sim: &Simulation, sample: &ActiveTrace) {
        if let Some(active) = &self.active {
            active.hand_over(run, sim, sample);
        }
    }

    /// A config patch landed: exports an open window as `config_change` and
    /// marks an admitted manual sample `petri.config_changed`.
    pub fn config_patched(
        &self,
        run: &mut RunHandle,
        sim: &Simulation,
        slot: &mut Option<ActiveTrace>,
    ) {
        if let Some(active) = &self.active {
            active.config_patched(run, sim, slot);
        }
    }

    /// The run ends: exports an open window and an admitted, unfetched
    /// manual sample as `run_end`. Call before [`Telemetry::end_run`].
    pub fn end_samples(
        &self,
        run: &mut RunHandle,
        sim: &Simulation,
        slot: &mut Option<ActiveTrace>,
        manual: Option<&ActiveTrace>,
    ) {
        if let Some(active) = &self.active {
            active.end_samples(run, sim, slot, manual);
        }
    }

    /// The process-shutdown flush: waits for every pending record up to the
    /// flush bound, abandons the rest, writes every outstanding run line and
    /// stops the exporter thread.
    pub fn shutdown(&self) {
        if let Some(active) = &self.active {
            active.shared.shutdown();
        }
    }
}

impl Active {
    /// Captures and offers the trace of the tick `sim` just ran, when tick
    /// traces are on, the run is below its cap and the seam is that tick's.
    fn tick_trace(&self, run: &mut RunHandle, sim: &Simulation, sample: TickSample) {
        if self.tick_traces == Switch::Off
            || run.traces_taken >= MAX_TICK_TRACES_PER_RUN
            || sim.tick >= FIRST_UNTRACED_TICK
        {
            return;
        }
        let Some(seam) = sim
            .stats
            .last_tick_phases
            .filter(|seam| seam.tick == sim.tick)
        else {
            return;
        };
        let began = Instant::now();
        run.traces_taken += 1;
        let request = trace::encode(
            &self.resource,
            run,
            &trace::Capture {
                seam: &seam,
                stats: &sim.stats,
                config: &sim.config,
                population: sim.creatures.len(),
                sample,
            },
        );
        self.shared
            .offer_encoded(run.key, Signal::Traces, request.encode_to_vec());
        self.shared.add_self_time(run.key, began.elapsed());
    }

    fn emit(
        &self,
        run: &RunHandle,
        event: &'static str,
        tick: u64,
        extra: Vec<(&'static str, AnyValue)>,
        body: Option<String>,
    ) {
        let mut record = self.logger.create_log_record();
        let now = SystemTime::now();
        record.set_event_name(event);
        record.set_timestamp(now);
        record.set_observed_timestamp(now);
        record.set_severity_number(Severity::Info);
        record.set_severity_text("INFO");
        // Loki 3.7 drops the OTLP `event_name` field; the attribute keeps the
        // record's name queryable there as `event_name`.
        record.add_attribute("event.name", event);
        record.add_attribute(RUN_ID_KEY, run.id.clone());
        record.add_attribute("petri.seed", u64_value(run.seed));
        record.add_attribute("petri.world", run.world.clone());
        if let Some(recipe) = &run.recipe {
            record.add_attribute("petri.recipe", recipe.clone());
        }
        record.add_attribute("petri.config_digest", run.config_digest.clone());
        record.add_attribute("petri.tick", u64_value(tick));
        for (key, value) in extra {
            record.add_attribute(key, value);
        }
        if let Some(body) = body {
            record.set_body(AnyValue::from(body));
        }
        self.logger.emit(record);
    }
}

#[cfg(test)]
mod tests;
