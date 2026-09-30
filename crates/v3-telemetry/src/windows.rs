//! Creature cognition windows (T21.F04).
//!
//! A window records one creature for a run of consecutive ticks through the
//! recorder's `ActiveTrace` and exports them as one trace: a
//! `creature_window` root span and a `creature_tick` child per recorded tick,
//! whose events carry every hop, pass and applied action. The server's manual
//! Execution Sampler exports the same way with policy `manual`. Selection is
//! a pure function of the seed, the run's sample index and the population;
//! nothing here draws from a simulation RNG or changes a decision.
//!
//! IDs carry no entropy: sample `k`'s trace ID is the run key's high 64 bits
//! followed by `2^63 + k`, so it never meets a tick trace's (whose low half is
//! a tick below `2^63`); span `1` is the root and the `i`th recorded tick is
//! `(i + 1) << 8`.

use std::time::{Duration, Instant};

use opentelemetry::logs::AnyValue as LogValue;
use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;
use opentelemetry_proto::tonic::common::v1::{any_value::Value, KeyValue};
use opentelemetry_proto::tonic::trace::v1::span::{Event, SpanKind};
use opentelemetry_proto::tonic::trace::v1::Span;
use prost::Message as _;
use sha2::{Digest as _, Sha256};
use slotmap::Key as _;
use v3_core::contracts::{CreatureId, WorldAction};
use v3_core::creature::action_log::NO_DIRECTION;
use v3_core::creature::genome::CreatureGenome;
use v3_core::runtime::trace::domain::{
    AppliedAction, BackendTrace, MeshHopTrace, MeshPassTrace, TickOutcome, TickTrace,
};
use v3_core::runtime::trace::recording::{ActiveTrace, TraceBudget, TruncationReason};
use v3_core::simulation::energy_accounting::DeathCause;
use v3_core::simulation::Simulation;

use crate::queue::Signal;
use crate::trace::{self, Attributes};
use crate::{saturating_nanos, Active, RunHandle, Switch, RUN_ID_KEY};

/// Turns creature windows (and manual-sample export) off while telemetry is on.
pub const CREATURE_WINDOWS_ENV: &str = "PETRI_TELEMETRY_CREATURE_WINDOWS";
/// Ticks per window, `1` to `64`.
pub const WINDOW_TICKS_ENV: &str = "PETRI_TELEMETRY_WINDOW_TICKS";
/// The least wall time between sample starts, `10` to `3600000` ms.
pub const WINDOW_INTERVAL_ENV: &str = "PETRI_TELEMETRY_WINDOW_INTERVAL_MS";
/// The most samples (windows and manual) one run exports.
pub const MAX_SAMPLES_PER_RUN: u64 = 4_096;
/// The most encoded sample bytes (window requests and genome bodies) per run.
pub const MAX_SAMPLE_BYTES_PER_RUN: u64 = 256 * MIB;
/// The most events one exported sample carries.
pub const MAX_EVENTS_PER_SAMPLE: u32 = 2_048;
/// The recorder's caps for a window.
pub const WINDOW_BUDGET: TraceBudget = TraceBudget {
    max_events: MAX_EVENTS_PER_SAMPLE,
    max_bytes: 4 * MIB,
};

const MIB: u64 = 1024 * 1024;
/// What one body counts toward the per-run bytes at most: the queue's body
/// cap, which drops anything larger whole.
const BODY_RESERVATION: u64 = 4 * MIB;
const WINDOW_TICKS: std::ops::RangeInclusive<u32> = 1..=64;
const INTERVAL_MS: std::ops::RangeInclusive<u64> = 10..=3_600_000;
/// `2^63`: the low trace-ID half of sample 0.
const SAMPLE_ID_BASE: u64 = 1 << 63;

/// The window settings, read only when telemetry resolves on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowSettings {
    pub switch: Switch,
    pub ticks: u32,
    pub interval: Duration,
}

impl Default for WindowSettings {
    fn default() -> Self {
        Self {
            switch: Switch::On,
            ticks: 8,
            interval: Duration::from_secs(10),
        }
    }
}

fn present(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

impl WindowSettings {
    /// Parses the three variables' values; unset or empty takes the default,
    /// anything else out of range is refused.
    pub fn resolve(
        switch: Option<&str>,
        ticks: Option<&str>,
        interval_ms: Option<&str>,
    ) -> Result<Self, String> {
        Self::resolve_over(Self::default(), switch, ticks, interval_ms)
    }

    /// [`WindowSettings::resolve`] with `defaults` (a preset's windows) in
    /// place of the built-in defaults.
    pub fn resolve_over(
        defaults: Self,
        switch: Option<&str>,
        ticks: Option<&str>,
        interval_ms: Option<&str>,
    ) -> Result<Self, String> {
        let switch = match present(switch) {
            Some(value) => value
                .parse()
                .map_err(|error| format!("invalid {CREATURE_WINDOWS_ENV}: {error}"))?,
            None => defaults.switch,
        };
        let ticks = match present(ticks) {
            Some(value) => value
                .parse::<u32>()
                .ok()
                .filter(|ticks| WINDOW_TICKS.contains(ticks))
                .ok_or_else(|| {
                    format!("invalid {WINDOW_TICKS_ENV}: expected 1 to 64 ticks, found `{value}`")
                })?,
            None => defaults.ticks,
        };
        let interval = match present(interval_ms) {
            Some(value) => value
                .parse::<u64>()
                .ok()
                .filter(|ms| INTERVAL_MS.contains(ms))
                .map(Duration::from_millis)
                .ok_or_else(|| {
                    format!(
                        "invalid {WINDOW_INTERVAL_ENV}: expected whole milliseconds from 10 to \
                         3600000, found `{value}`"
                    )
                })?,
            None => defaults.interval,
        };
        Ok(Self {
            switch,
            ticks,
            interval,
        })
    }

    pub(crate) fn interval_ms(&self) -> u64 {
        u64::try_from(self.interval.as_millis()).unwrap_or(u64::MAX)
    }
}

/// SplitMix64's output function.
fn splitmix64(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// The position, in `sim.creatures` iteration order, of window `k`'s
/// creature among `population` (non-zero).
pub(crate) fn select(seed: u64, k: u64, population: usize) -> usize {
    let draw = splitmix64(seed ^ k.wrapping_mul(0x9E37_79B9_7F4A_7C15));
    (draw % population as u64) as usize
}

/// Sample `k`'s trace ID in the run keyed `run_key`.
pub(crate) fn sample_trace_id(run_key: u128, k: u64) -> [u8; 16] {
    trace::trace_id(run_key, SAMPLE_ID_BASE.wrapping_add(k))
}

/// The `i`th recorded tick's span ID (the root is `1`).
fn tick_span_id(i: usize) -> [u8; 8] {
    trace::span_id(((i as u64) + 1) << 8)
}

/// Which policy selected a sample.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Policy {
    Window,
    Manual,
}

impl Policy {
    fn as_str(self) -> &'static str {
        match self {
            Self::Window => "window",
            Self::Manual => "manual",
        }
    }
}

/// Why a sample ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EndReason {
    Complete,
    Died,
    Truncated,
    Manual,
    Replaced,
    ConfigChange,
    RunEnd,
}

impl EndReason {
    fn as_str(self) -> &'static str {
        match self {
            Self::Complete => "complete",
            Self::Died => "died",
            Self::Truncated => "truncated",
            Self::Manual => "manual",
            Self::Replaced => "replaced",
            Self::ConfigChange => "config_change",
            Self::RunEnd => "run_end",
        }
    }
}

/// Which per-run cap stopped samples.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Cap {
    Count,
    Bytes,
}

impl Cap {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Count => "count",
            Self::Bytes => "bytes",
        }
    }
}

/// What a sample captured when it started.
#[derive(Debug, Clone)]
pub(crate) struct SampleMeta {
    pub(crate) index: u64,
    pub(crate) policy: Policy,
    /// The creature's server ID (`CreatureId` ffi).
    pub(crate) creature_id: u64,
    pub(crate) started_at: Instant,
    pub(crate) digest: String,
    /// The root's creature, genome and config attributes, read at start.
    pub(crate) root: Vec<KeyValue>,
    pub(crate) ticks_requested: u32,
    /// The digest in force when each recorded tick ran.
    pub(crate) tick_digests: Vec<String>,
    pub(crate) config_changed: bool,
}

/// A run's samples: index allocation, per-run caps and the open samples.
#[derive(Debug, Clone, Default)]
pub(crate) struct Samples {
    pub(crate) next_index: u64,
    /// Encoded window requests and genome bodies counted, each at most
    /// [`BODY_RESERVATION`].
    pub(crate) bytes: u64,
    pub(crate) last_admitted: Option<Instant>,
    pub(crate) capped: Option<Cap>,
    /// Manual samples started inside the interval or past a cap.
    pub(crate) skipped: u64,
    pub(crate) window: Option<SampleMeta>,
    pub(crate) manual: Option<SampleMeta>,
}

impl Samples {
    /// Whether one more sample fits the per-run caps; records the cap met.
    fn fits_caps(&mut self) -> bool {
        let cap = if self.next_index >= MAX_SAMPLES_PER_RUN {
            Some(Cap::Count)
        } else if self.bytes + 2 * BODY_RESERVATION > MAX_SAMPLE_BYTES_PER_RUN {
            Some(Cap::Bytes)
        } else {
            None
        };
        if let Some(cap) = cap {
            self.capped.get_or_insert(cap);
        }
        cap.is_none()
    }

    fn interval_passed(&self, now: Instant, interval: Duration) -> bool {
        self.last_admitted
            .is_none_or(|at| now.saturating_duration_since(at) >= interval)
    }
}

/// How and when a sample ended.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Ending {
    pub(crate) reason: EndReason,
    /// The tick count when it ended.
    pub(crate) tick: u64,
    /// The end of the tick it ended in, when that tick ran and recorded
    /// nothing for it (a death before recording).
    pub(crate) tick_end: Option<Instant>,
}

/// What an encoded sample holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Encoded {
    pub(crate) ticks: u64,
    pub(crate) events: u32,
}

/// The genome's compact JSON with keys sorted recursively, as the config
/// digest canonicalizes, and its `sha256:<hex>`.
pub(crate) fn genome_body(genome: &CreatureGenome) -> (String, String) {
    let value = serde_json::to_value(genome).expect("genomes serialize");
    let body = serde_json::to_string(&v3_core::config::sort_json_keys_recursive(value))
        .expect("sorted JSON serializes");
    let hash = hex::encode(Sha256::digest(body.as_bytes()));
    (body, format!("sha256:{hash}"))
}

/// An applied or selected action as a short string: `noop`, `eat:<type>`,
/// `move:<dir>`, `reproduce:<dir>`, `steal:<dir>`.
pub(crate) fn action_label(action: &WorldAction) -> String {
    match action {
        WorldAction::NoOp => "noop".to_owned(),
        WorldAction::Eat { type_idx } => format!("eat:{}", type_idx.get()),
        WorldAction::Move(direction) => format!("move:{}", direction.to_index()),
        WorldAction::Reproduce { direction, .. } => format!("reproduce:{}", direction.to_index()),
        WorldAction::StealEnergy { direction, .. } => format!("steal:{}", direction.to_index()),
    }
}

/// A unit-variant enum's name (`NoDecision`, `Decided`, …), which its serde
/// name also is.
fn variant<T: std::fmt::Debug>(value: &T) -> String {
    format!("{value:?}")
}

fn ints(a: &mut Attributes, key: &str, values: impl IntoIterator<Item = i64>) {
    a.array(key, values.into_iter().map(Value::IntValue));
}

fn texts(a: &mut Attributes, key: &str, values: impl IntoIterator<Item = String>) {
    a.array(key, values.into_iter().map(Value::StringValue));
}

/// The attributes every span of the sample carries.
fn identity(
    run: &RunHandle,
    meta: &SampleMeta,
    digest: &str,
    tick: u64,
    phase: &str,
) -> Attributes {
    let mut a = Attributes::default();
    a.text(RUN_ID_KEY, run.id.clone());
    a.int("petri.seed", run.seed);
    a.text("petri.world", run.world.clone());
    if let Some(recipe) = &run.recipe {
        a.text("petri.recipe", recipe.clone());
    }
    a.text("petri.config_digest", digest.to_owned());
    a.int("petri.tick", tick);
    a.text("petri.phase", phase.to_owned());
    a.text("petri.sample_policy", meta.policy.as_str());
    a.int("petri.window", meta.index);
    a.text("petri.creature_id", meta.creature_id.to_string());
    a
}

fn at(run: &RunHandle, instant: Instant) -> u64 {
    run.started_ns.saturating_add(saturating_nanos(
        instant.saturating_duration_since(run.started),
    ))
}

fn hop_event(hop: &MeshHopTrace, time: u64) -> Event {
    let mut a = Attributes::default();
    a.count("petri.hop", hop.hop_index);
    a.int("petri.pass", hop.pass_index);
    a.int("petri.node", u64::from(hop.node_id.0));
    let backend = match hop.backend_trace {
        BackendTrace::Vm(_) => "vm",
        BackendTrace::Graph(_) => "graph",
    };
    a.text("petri.backend", backend);
    a.double("petri.energy_before", hop.energy_before);
    a.double("petri.energy_after", hop.energy_after);
    a.doubles("petri.output_slots", &hop.output_slots);
    a.doubles("petri.vote_contribution", &hop.vote_contribution);
    if let Some(route) = &hop.route {
        let scores = &route.gate_scores;
        ints(
            &mut a,
            "petri.route.slots",
            scores.iter().map(|s| i64::from(s.slot)),
        );
        ints(
            &mut a,
            "petri.route.targets",
            scores.iter().map(|s| i64::from(s.target_id.0)),
        );
        let effective: Vec<f32> = scores.iter().map(|s| s.effective_score).collect();
        a.doubles("petri.route.scores", &effective);
        a.count("petri.route.selected", route.selected_target_idx);
    }
    event("hop", time, a)
}

fn pass_event(pass: &MeshPassTrace, time: u64) -> Event {
    let mut a = Attributes::default();
    a.int("petri.pass", pass.pass_index);
    a.text("petri.end_reason", variant(&pass.end_reason));
    a.int("petri.hops", pass.hops);
    a.doubles("petri.votes", &pass.votes);
    a.doubles("petri.effective_votes", &pass.effective_votes);
    if let Some(committed) = &pass.committed {
        a.text("petri.committed", action_label(committed));
    }
    event("pass", time, a)
}

fn action_event(index: usize, applied: &AppliedAction, time: u64) -> Event {
    let entry = &applied.entry;
    let mut a = Attributes::default();
    a.text("petri.phase", "actions");
    a.count("petri.index", index);
    a.text("petri.action", entry.action_type.as_key());
    a.text("petri.result", entry.result.as_key());
    if entry.direction != NO_DIRECTION {
        a.int("petri.direction", entry.direction);
    }
    a.double("petri.amount", entry.amount);
    if let Some(food_type) = entry.food_type {
        a.int("petri.food_type", food_type.get());
    }
    a.double("petri.energy_before", entry.energy_before);
    a.double("petri.energy_after", entry.energy_after);
    a.double("petri.charge", applied.charge);
    a.double("petri.penalty", applied.penalty);
    a.double("petri.reward", applied.reward);
    event("action", time, a)
}

fn event(name: &str, time: u64, attributes: Attributes) -> Event {
    Event {
        time_unix_nano: time,
        name: name.to_owned(),
        attributes: attributes.0,
        ..Event::default()
    }
}

/// The tick span's sensed, decided, outcome and after-state attributes.
fn tick_attributes(a: &mut Attributes, record: &TickTrace) {
    let sensed = &record.static_inputs;
    a.double("petri.food_here", sensed.food_here);
    a.doubles("petri.neighbor_food", &sensed.neighbor_food);
    a.doubles("petri.neighbor_barrier", &sensed.neighbor_barrier);
    a.doubles("petri.neighbor_occupied", &sensed.neighbor_occupied);
    a.double("petri.age_ticks", sensed.age_ticks);
    a.doubles("petri.previous_outcome", &sensed.previous_outcome);
    let outcome = record.outcome.as_ref();
    let perceived = outcome.is_some_and(|outcome| outcome.uses_extended_perception);
    if let Some(p) = record.debug_perception.as_ref().filter(|_| perceived) {
        a.doubles("petri.area_food", &p.area_food);
        a.doubles("petri.area_barrier", &p.area_barrier);
        a.doubles("petri.area_occupancy", &p.area_occupancy);
        a.doubles("petri.nearby_core", &p.nearby_core);
        a.doubles("petri.nearby_vitals", &p.nearby_vitals);
        a.doubles("petri.nearby_identity", &p.nearby_identity);
    }
    a.double("petri.energy_start", record.energy_before);
    a.double("petri.energy_after_cognition", record.energy_after);
    texts(
        a,
        "petri.actions_selected",
        record.final_actions.iter().map(action_label),
    );
    a.text(
        "petri.termination_reason",
        variant(&record.termination_reason),
    );
    a.double("petri.priority_bid", record.priority_bid);
    ints(
        a,
        "petri.commit_counts",
        record.commit_counts.iter().map(|&count| i64::from(count)),
    );
    a.count("petri.hops", record.hops.len());
    a.count("petri.passes", record.passes.len());
    if let Some(outcome) = outcome {
        outcome_attributes(a, outcome);
    }
}

fn outcome_attributes(a: &mut Attributes, outcome: &TickOutcome) {
    if outcome.uses_typed_local_food {
        let local = &outcome.typed_local_food;
        a.doubles("petri.food_here_by_type", &local.food_here_by_type);
        for (index, ring) in local.neighbor_food_by_type.iter().enumerate() {
            a.doubles(format!("petri.neighbor_food.{index}"), ring);
        }
    }
    if outcome.uses_extended_perception {
        for (index, area) in outcome.typed_area_food.iter().enumerate() {
            a.doubles(format!("petri.area_food.{index}"), area);
        }
    }
    ints(
        a,
        "petri.position",
        [i64::from(outcome.position.x), i64::from(outcome.position.y)],
    );
    a.int("petri.age", outcome.age);
    a.double("petri.failed_action_penalty", outcome.failed_action_penalty);
    a.count("petri.actions_applied", outcome.applied.len());
    a.double("petri.damage_received", outcome.damage_received);
    a.int("petri.offspring_spawned", outcome.offspring_spawned);
    if let Some(cause) = outcome.died {
        a.text("petri.died", cause.as_key());
    }
    if let Some(after) = &outcome.after {
        ints(
            a,
            "petri.position_end",
            [i64::from(after.position.x), i64::from(after.position.y)],
        );
        a.double("petri.energy_end", after.energy);
        a.doubles("petri.shared_memory", &after.shared_memory);
        a.doubles("petri.previous_outcome_end", &after.previous_outcome);
    }
}

/// The recorded tick's events, at most `left` of them, taken from `left`;
/// returns whether any were cut. Events past the allowance are never built.
fn tick_events(
    record: &TickTrace,
    cognition: u64,
    actions: u64,
    left: &mut u32,
) -> (Vec<Event>, bool) {
    let applied = record.outcome.as_ref().map_or(&[][..], |o| &o.applied[..]);
    let total = record.hops.len() + record.passes.len() + applied.len();
    let kept = total.min(*left as usize);
    let events: Vec<Event> = record
        .hops
        .iter()
        .map(|hop| hop_event(hop, cognition))
        .chain(record.passes.iter().map(|pass| pass_event(pass, cognition)))
        .chain(
            applied
                .iter()
                .enumerate()
                .map(|(index, action)| action_event(index, action, actions)),
        )
        .take(kept)
        .collect();
    *left -= kept as u32;
    (events, kept < total)
}

/// The recorded tick's start and end, and its cognition and actions phase
/// starts, in Unix nanoseconds.
fn tick_times(run: &RunHandle, record: &TickTrace) -> Option<[u64; 4]> {
    let seam = record.outcome.as_ref()?.phases?;
    let start = at(run, seam.started);
    Some([
        start,
        start.saturating_add(saturating_nanos(seam.elapsed)),
        start.saturating_add(saturating_nanos(seam.phases[2].offset)),
        start.saturating_add(saturating_nanos(seam.phases[3].offset)),
    ])
}

/// Encodes `trace`, started as `meta`, ended as `ending`, for `run`.
pub(crate) fn encode(
    resource: &[KeyValue],
    run: &RunHandle,
    meta: &SampleMeta,
    trace: &ActiveTrace,
    ending: &Ending,
) -> (ExportTraceServiceRequest, Encoded) {
    let trace_id = sample_trace_id(run.key, meta.index).to_vec();
    let root_id = trace::span_id(1).to_vec();
    let start = at(run, meta.started_at);
    let mut spans = Vec::with_capacity(trace.ticks.len() + 1);
    let mut left = MAX_EVENTS_PER_SAMPLE;
    let mut cut: Option<u64> = None;
    let mut end = ending.tick_end.map_or(start, |instant| at(run, instant));
    for (i, record) in trace.ticks.iter().enumerate() {
        let tick = record.tick_number + 1;
        let [tick_start, tick_end, cognition, actions] =
            tick_times(run, record).unwrap_or([start; 4]);
        end = end.max(tick_end);
        let digest = meta.tick_digests.get(i).unwrap_or(&meta.digest);
        let mut a = identity(run, meta, digest, tick, "cognition");
        tick_attributes(&mut a, record);
        let (events, was_cut) = tick_events(record, cognition, actions, &mut left);
        if was_cut && cut.is_none() {
            cut = Some(tick);
        }
        spans.push(Span {
            trace_id: trace_id.clone(),
            span_id: tick_span_id(i).to_vec(),
            parent_span_id: root_id.clone(),
            name: "creature_tick".to_owned(),
            kind: SpanKind::Internal as i32,
            start_time_unix_nano: tick_start,
            end_time_unix_nano: tick_end,
            attributes: a.0,
            events,
            ..Span::default()
        });
    }
    let first = trace
        .ticks
        .first()
        .map_or(ending.tick, |t| t.tick_number + 1);
    let last = trace
        .ticks
        .last()
        .map_or(ending.tick, |t| t.tick_number + 1);
    let events = MAX_EVENTS_PER_SAMPLE - left;
    let mut a = identity(run, meta, &meta.digest, first, "cognition");
    a.int("petri.tick_end", last);
    a.int("petri.ticks_requested", meta.ticks_requested);
    a.count("petri.ticks_recorded", trace.ticks.len());
    a.text("petri.end_reason", ending.reason.as_str());
    if let Some(cause) = died(trace) {
        a.text("petri.died", cause.as_key());
    }
    let truncated = trace
        .truncated
        .map(|t| {
            let reason = match t.reason {
                TruncationReason::Events => "events",
                TruncationReason::Bytes => "bytes",
            };
            (reason, t.tick + 1)
        })
        .or(cut.map(|tick| ("events", tick)));
    a.flag("petri.truncated", truncated.is_some());
    if let Some((reason, tick)) = truncated {
        a.text("petri.truncated_reason", reason);
        a.int("petri.truncated_tick", tick);
    }
    a.int("petri.events", events);
    if meta.config_changed {
        a.flag("petri.config_changed", true);
    }
    a.0.extend(meta.root.iter().cloned());
    spans.insert(
        0,
        Span {
            trace_id: trace_id.clone(),
            span_id: root_id,
            name: "creature_window".to_owned(),
            kind: SpanKind::Internal as i32,
            start_time_unix_nano: start,
            end_time_unix_nano: end,
            attributes: a.0,
            ..Span::default()
        },
    );
    (
        trace::request(resource, spans),
        Encoded {
            ticks: trace.ticks.len() as u64,
            events,
        },
    )
}

/// A fresh recording of `id` for a window.
fn window_trace(id: CreatureId, ticks: u32) -> ActiveTrace {
    let mut trace = ActiveTrace::budgeted(id, ticks, WINDOW_BUDGET);
    trace.include_perception_debug = true;
    trace
}

/// The removal cause of the recording's creature: gone at a tick's start, or
/// removed in its last recorded tick.
fn died(trace: &ActiveTrace) -> Option<DeathCause> {
    trace.removed.or_else(|| {
        trace
            .ticks
            .last()
            .and_then(|t| t.outcome.as_ref())
            .and_then(|o| o.died)
    })
}

/// How a window that just ran a tick ended, if it did.
fn window_end(trace: &ActiveTrace) -> Option<EndReason> {
    if trace.truncated.is_some() {
        Some(EndReason::Truncated)
    } else if died(trace).is_some() {
        Some(EndReason::Died)
    } else if trace.is_complete() {
        Some(EndReason::Complete)
    } else {
        None
    }
}

/// The end of the tick `sim` just ran, when its seam is that tick's.
fn last_tick_end(sim: &Simulation) -> Option<Instant> {
    sim.stats
        .last_tick_phases
        .filter(|seam| seam.tick == sim.tick)
        .map(|seam| seam.started + seam.elapsed)
}

impl Active {
    fn windows_on(&self) -> bool {
        self.windows.switch == Switch::On
    }

    /// Captures a sample's start metadata for the living creature `id` and
    /// emits its genome record; allocates the run's next sample index.
    fn start_sample(
        &self,
        run: &mut RunHandle,
        sim: &Simulation,
        id: CreatureId,
        policy: Policy,
        ticks_requested: u32,
    ) -> Option<SampleMeta> {
        let creature = sim.creatures.get(id)?;
        let started_at = Instant::now();
        let index = run.samples.next_index;
        run.samples.next_index += 1;
        run.samples.last_admitted = Some(started_at);
        let creature_id = id.data().as_ffi();
        let (body, hash) = genome_body(&creature.genome);
        let body_len = body.len() as u64;
        let dropped = body_len > self.shared.max_body_bytes();
        run.samples.bytes += body_len.min(BODY_RESERVATION);
        let extra = vec![
            ("petri.window", crate::u64_value(index)),
            ("petri.creature_id", LogValue::from(creature_id.to_string())),
            ("petri.genome_hash", LogValue::from(hash.clone())),
            (
                "petri.genome_size",
                LogValue::Int(i64::from(creature.cached_genome_size)),
            ),
        ];
        self.emit(run, "creature.genome", sim.tick, extra, Some(body));
        let mut root = Attributes::default();
        root.int("petri.lineage_id", creature.identity.lineage_id);
        root.int("petri.kin_tag", creature.identity.kin_tag);
        root.int("petri.generation", creature.generation);
        root.int("petri.age_start", creature.age);
        root.int("petri.genome_size", creature.cached_genome_size);
        root.text("petri.genome_hash", hash);
        if dropped {
            root.flag("petri.genome_dropped", true);
        }
        trace::cognition(&mut root, &sim.config);
        trace::actions(&mut root, &sim.config);
        Some(SampleMeta {
            index,
            policy,
            creature_id,
            started_at,
            digest: run.config_digest.clone(),
            root: root.0,
            ticks_requested,
            tick_digests: Vec::new(),
            config_changed: false,
        })
    }

    /// Encodes and offers one sample that ended as `reason` at `sim`'s
    /// tick, counting its bytes toward the run's.
    fn export(
        &self,
        run: &mut RunHandle,
        sim: &Simulation,
        meta: &SampleMeta,
        trace: &ActiveTrace,
        reason: EndReason,
        tick_end: Option<Instant>,
    ) {
        let ending = Ending {
            reason,
            tick: sim.tick,
            tick_end,
        };
        let (request, encoded) = encode(&self.resource, run, meta, trace, &ending);
        let body = request.encode_to_vec();
        run.samples.bytes += (body.len() as u64).min(BODY_RESERVATION);
        self.shared.add_recorded(run.key, encoded.ticks);
        self.shared.offer_encoded(run.key, Signal::Windows, body);
    }

    /// Exports the open window in `slot` as `reason` and clears the slot.
    fn end_window(
        &self,
        run: &mut RunHandle,
        sim: &Simulation,
        slot: &mut Option<ActiveTrace>,
        reason: EndReason,
    ) {
        if let (Some(meta), Some(trace)) = (run.samples.window.take(), slot.take()) {
            self.export(run, sim, &meta, &trace, reason, None);
        }
    }

    /// Exports the admitted manual sample, recorded as `trace`, as `reason`.
    fn end_manual(
        &self,
        run: &mut RunHandle,
        sim: &Simulation,
        trace: Option<&ActiveTrace>,
        reason: EndReason,
    ) {
        if let (Some(meta), Some(trace)) = (run.samples.manual.take(), trace) {
            self.export(run, sim, &meta, trace, reason, None);
        }
    }

    pub(crate) fn before_tick(
        &self,
        run: &mut RunHandle,
        sim: &Simulation,
        slot: &mut Option<ActiveTrace>,
        manual_active: bool,
    ) {
        if !self.windows_on() || slot.is_some() || manual_active || sim.creatures.is_empty() {
            return;
        }
        let began = Instant::now();
        if !run.samples.interval_passed(began, self.windows.interval) || !run.samples.fits_caps() {
            return;
        }
        let n = select(run.seed, run.samples.next_index, sim.creatures.len());
        let id = sim
            .creatures
            .keys()
            .nth(n)
            .expect("n is below the population");
        run.samples.window = self.start_sample(run, sim, id, Policy::Window, self.windows.ticks);
        *slot = Some(window_trace(id, self.windows.ticks));
        self.shared.add_self_time(run.key, began.elapsed());
    }

    pub(crate) fn after_tick(
        &self,
        run: &mut RunHandle,
        sim: &Simulation,
        slot: &mut Option<ActiveTrace>,
        manual: Option<&ActiveTrace>,
    ) {
        let began = Instant::now();
        let digest = run.config_digest.clone();
        for (meta, trace) in [
            (run.samples.window.as_mut(), slot.as_ref()),
            (run.samples.manual.as_mut(), manual),
        ] {
            if let (Some(meta), Some(trace)) = (meta, trace) {
                meta.tick_digests.resize(trace.ticks.len(), digest.clone());
            }
        }
        let Some(reason) = slot.as_ref().and_then(window_end) else {
            return;
        };
        let (Some(meta), Some(trace)) = (run.samples.window.take(), slot.take()) else {
            return;
        };
        let tick_end = trace.ticks.is_empty().then(|| last_tick_end(sim)).flatten();
        self.export(run, sim, &meta, &trace, reason, tick_end);
        self.shared.add_self_time(run.key, began.elapsed());
    }

    pub(crate) fn start_manual(
        &self,
        run: &mut RunHandle,
        sim: &Simulation,
        slot: &mut Option<ActiveTrace>,
        replaced: Option<&ActiveTrace>,
        sample: &ActiveTrace,
    ) {
        if !self.windows_on() {
            return;
        }
        let began = Instant::now();
        self.end_window(run, sim, slot, EndReason::Manual);
        self.end_manual(run, sim, replaced, EndReason::Replaced);
        if run.samples.interval_passed(began, self.windows.interval) && run.samples.fits_caps() {
            run.samples.manual = self.start_sample(
                run,
                sim,
                sample.creature_id,
                Policy::Manual,
                sample.ticks_remaining,
            );
        } else {
            run.samples.skipped += 1;
        }
        self.shared.add_self_time(run.key, began.elapsed());
    }

    pub(crate) fn hand_over(&self, run: &mut RunHandle, sim: &Simulation, sample: &ActiveTrace) {
        let began = Instant::now();
        self.end_manual(run, sim, Some(sample), EndReason::Complete);
        self.shared.add_self_time(run.key, began.elapsed());
    }

    pub(crate) fn config_patched(
        &self,
        run: &mut RunHandle,
        sim: &Simulation,
        slot: &mut Option<ActiveTrace>,
    ) {
        let began = Instant::now();
        self.end_window(run, sim, slot, EndReason::ConfigChange);
        if let Some(meta) = run.samples.manual.as_mut() {
            meta.config_changed = true;
        }
        self.shared.add_self_time(run.key, began.elapsed());
    }

    pub(crate) fn end_samples(
        &self,
        run: &mut RunHandle,
        sim: &Simulation,
        slot: &mut Option<ActiveTrace>,
        manual: Option<&ActiveTrace>,
    ) {
        let began = Instant::now();
        self.end_window(run, sim, slot, EndReason::RunEnd);
        self.end_manual(run, sim, manual, EndReason::RunEnd);
        self.shared.add_self_time(run.key, began.elapsed());
    }
}

#[cfg(test)]
mod tests;
