//! Tick traces as OTLP spans (T21.F03).
//!
//! A tick trace is taken with an interval snapshot, or with the CLI's
//! completion snapshot, right after the `run_tick` that produced the seam
//! (`SimStats::last_tick_phases`). It is one `tick` span with a child span per
//! timed phase, encoded here on the simulation thread as one
//! `ExportTraceServiceRequest` and posted by the queue's worker. IDs carry no
//! entropy: the trace ID is the run key's high 64 bits followed by the tick,
//! and the span IDs are `1` for `tick` and `2` to `6` for the phases in order.
//! Timestamps are the run's start plus monotonic offsets from it.

use std::time::Duration;

use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;
use opentelemetry_proto::tonic::common::v1::{
    any_value, AnyValue, ArrayValue, InstrumentationScope, KeyValue,
};
use opentelemetry_proto::tonic::resource::v1::Resource;
use opentelemetry_proto::tonic::trace::v1::{span::SpanKind, ResourceSpans, ScopeSpans, Span};
use v3_core::config::SimulationConfig;
use v3_core::simulation::stats::TickPhaseTimings;
use v3_core::simulation::SimStats;

use crate::{saturating_nanos, RunHandle, Switch, RUN_ID_KEY};

/// The environment variable that turns tick traces off while telemetry is on.
pub const TICK_TRACES_ENV: &str = "PETRI_TELEMETRY_TICK_TRACES";
/// The most tick traces one run captures.
pub const MAX_TICK_TRACES_PER_RUN: u64 = 65_536;
/// The first tick whose trace is not captured: trace IDs from `2^63` up
/// belong to creature windows (T21.F04).
pub const FIRST_UNTRACED_TICK: u64 = 1 << 63;
/// The phase spans' names, in `run_tick`'s order.
pub const PHASES: [&str; 5] = [
    "world_update",
    "sensor_assembly",
    "cognition",
    "actions",
    "reward_learning",
];
const CONFIG_PREFIX: &str = "petri.config.";

/// Parses [`TICK_TRACES_ENV`]: `on` or `off`; an unset or empty value is `on`.
pub fn resolve_tick_traces(env: Option<&str>) -> Result<Switch, String> {
    match env.map(str::trim).filter(|value| !value.is_empty()) {
        Some(value) => value
            .parse()
            .map_err(|error| format!("invalid {TICK_TRACES_ENV}: {error}")),
        None => Ok(Switch::On),
    }
}

/// The snapshot a tick trace accompanies; its `petri.sample_policy`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TickSample {
    /// An interval snapshot after a tick.
    Interval,
    /// The CLI's completion snapshot, after the run's last tick.
    RunEnd,
}

impl TickSample {
    pub(crate) fn policy(self) -> &'static str {
        match self {
            Self::Interval => "interval",
            Self::RunEnd => "run_end",
        }
    }
}

/// The trace ID of `tick` in the run keyed `run_key`: the key's high 64 bits,
/// then the tick, both big-endian.
pub(crate) fn trace_id(run_key: u128, tick: u64) -> [u8; 16] {
    let mut id = [0; 16];
    id[..8].copy_from_slice(&((run_key >> 64) as u64).to_be_bytes());
    id[8..].copy_from_slice(&tick.to_be_bytes());
    id
}

/// Span ID `index`: `1` for `tick`, `2` to `6` for the phases.
pub(crate) fn span_id(index: u64) -> [u8; 8] {
    index.to_be_bytes()
}

pub(crate) fn key_value(key: impl Into<String>, value: any_value::Value) -> KeyValue {
    KeyValue {
        key: key.into(),
        value: Some(AnyValue { value: Some(value) }),
        ..KeyValue::default()
    }
}

/// A span's attributes as they are built.
#[derive(Default)]
pub(crate) struct Attributes(pub(crate) Vec<KeyValue>);

impl Attributes {
    pub(crate) fn text(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.0
            .push(key_value(key, any_value::Value::StringValue(value.into())));
    }

    /// An integer, or its decimal string when it exceeds `i64`.
    pub(crate) fn int(&mut self, key: impl Into<String>, value: impl Into<u64>) {
        let value: u64 = value.into();
        match i64::try_from(value) {
            Ok(number) => self
                .0
                .push(key_value(key, any_value::Value::IntValue(number))),
            Err(_) => self.text(key, value.to_string()),
        }
    }

    pub(crate) fn count(&mut self, key: impl Into<String>, value: usize) {
        self.int(key, u64::try_from(value).unwrap_or(u64::MAX));
    }

    pub(crate) fn double(&mut self, key: impl Into<String>, value: impl Into<f64>) {
        self.0
            .push(key_value(key, any_value::Value::DoubleValue(value.into())));
    }

    pub(crate) fn flag(&mut self, key: impl Into<String>, value: bool) {
        self.0
            .push(key_value(key, any_value::Value::BoolValue(value)));
    }

    /// An OTLP array of `values`, in order.
    pub(crate) fn array(
        &mut self,
        key: impl Into<String>,
        values: impl IntoIterator<Item = any_value::Value>,
    ) {
        let values = values
            .into_iter()
            .map(|value| AnyValue { value: Some(value) })
            .collect();
        self.0.push(key_value(
            key,
            any_value::Value::ArrayValue(ArrayValue { values }),
        ));
    }

    /// An array of doubles.
    pub(crate) fn doubles<T: Into<f64> + Copy>(&mut self, key: impl Into<String>, values: &[T]) {
        self.array(
            key,
            values
                .iter()
                .map(|value| any_value::Value::DoubleValue((*value).into())),
        );
    }
}

/// The attributes every span of the trace carries.
fn identity(run: &RunHandle, tick: u64, sample: TickSample) -> Attributes {
    let mut a = Attributes::default();
    a.text(RUN_ID_KEY, run.id.clone());
    a.int("petri.seed", run.seed);
    a.text("petri.world", run.world.clone());
    if let Some(recipe) = &run.recipe {
        a.text("petri.recipe", recipe.clone());
    }
    a.text("petri.config_digest", run.config_digest.clone());
    a.int("petri.tick", tick);
    a.text("petri.sample_policy", sample.policy());
    a
}

pub(crate) fn config_key(path: &str) -> String {
    format!("{CONFIG_PREFIX}{path}")
}

/// The tick span's own attributes.
fn tick_attributes(
    a: &mut Attributes,
    seam: &TickPhaseTimings,
    stats: &SimStats,
    population: usize,
) {
    a.text("petri.phase", "tick");
    a.count("petri.population_start", seam.population_start);
    a.count("petri.threads", seam.threads);
    a.count("petri.population", population);
    for (action, count) in [
        ("move", stats.last_tick_move),
        ("eat", stats.last_tick_eat),
        ("noop", stats.last_tick_noop),
        ("reproduce", stats.last_tick_reproduce),
        ("steal", stats.last_tick_steal),
    ] {
        a.int(format!("petri.actions.{action}"), count);
    }
    a.int("petri.predation_kills", stats.last_tick_predation_kills);
    a.double(
        "petri.compute_total_mean",
        stats.last_tick_compute_total_mean,
    );
    a.double("petri.compute_total_min", stats.last_tick_compute_total_min);
    a.double("petri.compute_total_max", stats.last_tick_compute_total_max);
    a.double("petri.compute_vm_mean", stats.last_tick_compute_vm_mean);
    a.double(
        "petri.compute_graph_mean",
        stats.last_tick_compute_graph_mean,
    );
    a.double("petri.priority_bid_mean", stats.last_tick_priority_bid_mean);
    a.int(
        "petri.priority_bidders_count",
        stats.last_tick_priority_bidders_count,
    );
    for (index, density) in stats
        .last_tick_food_total_density_by_type
        .iter()
        .enumerate()
    {
        a.double(format!("petri.food_total_density.{index}"), *density);
    }
}

/// The config values governing phase `index`, and the penalty in force for
/// the tick that ran as `sim.tick` = `ran`.
fn phase_attributes(a: &mut Attributes, index: usize, config: &SimulationConfig, ran: u64) {
    a.text("petri.phase", PHASES[index]);
    match index {
        0 => world_update(a, config),
        1 => {
            a.int(
                config_key("runtime.perception.vision_radius"),
                config.runtime.perception.vision_radius,
            );
            let edge_mode = serde_json::to_value(config.world.edge_mode)
                .ok()
                .and_then(|value| value.as_str().map(str::to_owned));
            if let Some(edge_mode) = edge_mode {
                a.text(config_key("world.edge_mode"), edge_mode);
            }
        }
        2 => cognition(a, config),
        3 => {
            a.double(
                "petri.failed_action_penalty",
                config.failed_action_penalty_for_tick(ran),
            );
            actions(a, config);
        }
        _ => a.double(
            config_key("runtime.reward_learning_cost"),
            config.runtime.reward_learning_cost,
        ),
    }
}

fn world_update(a: &mut Attributes, config: &SimulationConfig) {
    let lifecycle = &config.energy.lifecycle;
    let shared = &config.world.food.shared;
    a.double(
        config_key("energy.lifecycle.energy_decay_per_tick"),
        lifecycle.energy_decay_per_tick,
    );
    a.double(
        config_key("energy.lifecycle.genome_carry_cost_per_unit"),
        lifecycle.genome_carry_cost_per_unit,
    );
    a.double(
        config_key("shared_memory.decay_rate"),
        config.shared_memory.decay_rate,
    );
    a.double(
        config_key("world.food.shared.growth_rate"),
        shared.growth_rate,
    );
    a.double(
        config_key("world.food.shared.max_density"),
        shared.max_density,
    );
    a.flag(
        config_key("world.food.shared.grazing.enabled"),
        shared.grazing.enabled,
    );
    a.flag(
        config_key("world.food.shared.occupancy_depletion.enabled"),
        shared.occupancy_depletion.enabled,
    );
    a.flag(
        config_key("world.food.annealing.enabled"),
        config.world.food.annealing.enabled,
    );
    a.count(
        config_key("world.food.types"),
        config.world.food.types.len(),
    );
    for (index, food) in config.world.food.types.iter().enumerate() {
        let key = |field: &str| config_key(&format!("world.food.types.{index}.{field}"));
        a.double(key("growth_inhibitor"), food.growth_inhibitor);
        a.double(
            key("growth_rate"),
            food.growth_rate.unwrap_or(shared.growth_rate),
        );
        a.double(
            key("recovery_spawn_rate"),
            food.recovery_spawn_rate
                .unwrap_or(shared.recovery_spawn_rate),
        );
        a.double(
            key("energy_per_unit"),
            food.energy_per_unit
                .unwrap_or(config.energy.costs.eat_reward_per_food),
        );
    }
}

pub(crate) fn cognition(a: &mut Attributes, config: &SimulationConfig) {
    let runtime = &config.runtime;
    a.int(config_key("runtime.max_mesh_hops"), runtime.max_mesh_hops);
    a.int(config_key("runtime.max_vm_steps"), runtime.max_vm_steps);
    a.double(
        config_key("runtime.graph_node_base_cost"),
        runtime.graph_node_base_cost,
    );
    a.double(
        config_key("runtime.plasticity_update_cost"),
        runtime.plasticity_update_cost,
    );
    a.int(
        config_key("runtime.hop_ramp_allowance"),
        runtime.hop_ramp_allowance,
    );
    a.double(config_key("runtime.hop_ramp_cost"), runtime.hop_ramp_cost);
    a.count(
        config_key("runtime.max_actions_per_turn"),
        runtime.max_actions_per_turn,
    );
    a.double(
        config_key("runtime.vm.opcode_cost_multiplier"),
        runtime.vm.opcode_cost_multiplier,
    );
    a.int(
        config_key("runtime.vm.step_ramp_allowance"),
        runtime.vm.step_ramp_allowance,
    );
    a.double(
        config_key("runtime.vm.step_ramp_cost"),
        runtime.vm.step_ramp_cost,
    );
}

pub(crate) fn actions(a: &mut Attributes, config: &SimulationConfig) {
    let costs = &config.energy.costs;
    for (field, value) in [
        ("move_cost", costs.move_cost),
        ("eat_cost", costs.eat_cost),
        ("eat_reward_per_food", costs.eat_reward_per_food),
        ("noop_cost", costs.noop_cost),
        ("reproduce_cost", costs.reproduce_cost),
    ] {
        a.double(config_key(&format!("energy.costs.{field}")), value);
    }
    let complexity = &config.energy.complexity_cost;
    a.flag(
        config_key("energy.complexity_cost.enabled"),
        complexity.enabled,
    );
    a.int(
        config_key("energy.complexity_cost.threshold"),
        complexity.threshold,
    );
    a.double(
        config_key("energy.complexity_cost.scaling_factor"),
        complexity.scaling_factor,
    );
    let age = &config.energy.age_cost;
    a.flag(config_key("energy.age_cost.enabled"), age.enabled);
    a.int(config_key("energy.age_cost.grace_ticks"), age.grace_ticks);
    a.int(config_key("energy.age_cost.age_cap"), age.age_cap);
    a.double(
        config_key("energy.age_cost.max_multiplier"),
        age.max_multiplier,
    );
    let lifecycle = &config.energy.lifecycle;
    a.double(
        config_key("energy.lifecycle.min_reproduce_energy"),
        lifecycle.min_reproduce_energy,
    );
    a.int(
        config_key("energy.lifecycle.min_reproduce_age"),
        lifecycle.min_reproduce_age,
    );
    a.double(
        config_key("energy.lifecycle.genome_replication_cost_per_unit"),
        lifecycle.genome_replication_cost_per_unit,
    );
    a.int(
        config_key("population.max_creatures"),
        config.population.max_creatures,
    );
    let mutation = &config.mutation;
    a.double(config_key("mutation.per_unit_rate"), mutation.per_unit_rate);
    a.int(
        config_key("mutation.genome_size_cap"),
        mutation.genome_size_cap,
    );
    a.flag(
        config_key("mutation.genome_size_pressure_enabled"),
        mutation.genome_size_pressure_enabled,
    );
    a.double(
        config_key("mutation.mesh_layer_probability"),
        mutation.mesh_layer_probability,
    );
    a.double(config_key("mutation.executed_bias"), mutation.executed_bias);
    a.double(
        config_key("predation.steal_cost_rate"),
        config.predation.steal_cost_rate,
    );
    a.double(
        config_key("predation.kill_complexity_bonus_multiplier"),
        config.predation.kill_complexity_bonus_multiplier,
    );
}

/// What one tick trace is made of, read on the simulation thread.
pub(crate) struct Capture<'a> {
    pub(crate) seam: &'a TickPhaseTimings,
    pub(crate) stats: &'a SimStats,
    pub(crate) config: &'a SimulationConfig,
    /// The population at capture.
    pub(crate) population: usize,
    pub(crate) sample: TickSample,
}

/// Encodes the trace of `capture`'s tick for `run`.
pub(crate) fn encode(
    resource: &[KeyValue],
    run: &RunHandle,
    capture: &Capture<'_>,
) -> ExportTraceServiceRequest {
    let seam = capture.seam;
    let trace = trace_id(run.key, seam.tick).to_vec();
    let tick_start_ns = run.started_ns.saturating_add(saturating_nanos(
        seam.started.saturating_duration_since(run.started),
    ));
    let root = span_id(1).to_vec();
    let span = |index: u64,
                name: &str,
                start_ns: u64,
                elapsed: Duration,
                parent: Vec<u8>,
                attributes: Attributes| Span {
        trace_id: trace.clone(),
        span_id: span_id(index).to_vec(),
        parent_span_id: parent,
        name: name.to_owned(),
        kind: SpanKind::Internal as i32,
        start_time_unix_nano: start_ns,
        end_time_unix_nano: start_ns.saturating_add(saturating_nanos(elapsed)),
        attributes: attributes.0,
        ..Span::default()
    };

    let mut attributes = identity(run, seam.tick, capture.sample);
    tick_attributes(&mut attributes, seam, capture.stats, capture.population);
    let mut spans = vec![span(
        1,
        "tick",
        tick_start_ns,
        seam.elapsed,
        Vec::new(),
        attributes,
    )];
    let ran = seam.tick.saturating_sub(1);
    for (index, phase) in seam.phases.iter().enumerate() {
        let mut attributes = identity(run, seam.tick, capture.sample);
        phase_attributes(&mut attributes, index, capture.config, ran);
        spans.push(span(
            index as u64 + 2,
            PHASES[index],
            tick_start_ns.saturating_add(saturating_nanos(phase.offset)),
            phase.elapsed,
            root.clone(),
            attributes,
        ));
    }

    request(resource, spans)
}

/// One trace request carrying `spans` under the `petri` scope.
pub(crate) fn request(resource: &[KeyValue], spans: Vec<Span>) -> ExportTraceServiceRequest {
    ExportTraceServiceRequest {
        resource_spans: vec![ResourceSpans {
            resource: Some(Resource {
                attributes: resource.to_vec(),
                ..Resource::default()
            }),
            scope_spans: vec![ScopeSpans {
                scope: Some(InstrumentationScope {
                    name: "petri".to_owned(),
                    ..InstrumentationScope::default()
                }),
                spans,
                ..ScopeSpans::default()
            }],
            ..ResourceSpans::default()
        }],
    }
}

#[cfg(test)]
mod tests;
