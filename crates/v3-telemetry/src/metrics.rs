//! Run snapshots as OTLP metrics (T21.F02).
//!
//! A snapshot reads the exported `SimStats` fields, the population census, the
//! tick and the run's elapsed wall time in one pass on the simulation thread
//! and encodes them as one `ExportMetricsServiceRequest`. Cumulative counters
//! are `petri.run.*` sums (monotonic for integers and durations, non-monotonic
//! for the signed `f64` totals); per-tick values are `petri.tick.*` gauges
//! sampled at the snapshot tick. Every data point carries `petri.run_id`;
//! breakdown maps add the key's `as_key()` string (a food type or vector index
//! is its decimal index). The tick is the `petri.run.tick` gauge, never an
//! attribute.

use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest;
use opentelemetry_proto::tonic::common::v1::{any_value, AnyValue, InstrumentationScope, KeyValue};
use opentelemetry_proto::tonic::metrics::v1::{
    metric, number_data_point, AggregationTemporality, Gauge, Metric, NumberDataPoint,
    ResourceMetrics, ScopeMetrics, Sum,
};
use opentelemetry_proto::tonic::resource::v1::Resource;
use v3_core::simulation::energy_accounting::DeathCause;
use v3_core::simulation::reproductive_success::CognitiveClass;
use v3_core::simulation::stats::MutationValueTotals;
use v3_core::simulation::{SimStats, Simulation};

use crate::RUN_ID_KEY;

/// The environment variable holding the interval between interval snapshots.
pub const METRICS_INTERVAL_ENV: &str = "PETRI_TELEMETRY_METRICS_INTERVAL_MS";
/// The interval used when [`METRICS_INTERVAL_ENV`] is unset or empty.
pub const DEFAULT_METRICS_INTERVAL: Duration = Duration::from_millis(1_000);
const MIN_INTERVAL_MS: u64 = 10;
const MAX_INTERVAL_MS: u64 = 3_600_000;
/// A transition snapshot needs this long since the run's last snapshot.
const TRANSITION_GAP: Duration = Duration::from_millis(10);
/// Prometheus's timestamp resolution: consecutive stamps differ by at least this.
const STAMP_STEP_NS: u64 = 1_000_000;
const SAMPLED: &str = "sampled at the snapshot tick";

/// Parses [`METRICS_INTERVAL_ENV`]: whole milliseconds from 10 to 3,600,000;
/// an unset or empty value is the default.
pub fn resolve_metrics_interval(env: Option<&str>) -> Result<Duration, String> {
    let Some(value) = env.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(DEFAULT_METRICS_INTERVAL);
    };
    value
        .parse::<u64>()
        .ok()
        .filter(|ms| (MIN_INTERVAL_MS..=MAX_INTERVAL_MS).contains(ms))
        .map(Duration::from_millis)
        .ok_or_else(|| {
            format!(
                "invalid {METRICS_INTERVAL_ENV}: expected whole milliseconds from \
                 {MIN_INTERVAL_MS} to {MAX_INTERVAL_MS}, found `{value}`"
            )
        })
}

/// What asks for a snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trigger {
    /// After a tick: taken once the interval has passed since the run's last
    /// snapshot (or its start).
    Interval,
    /// A server `run.state` change: taken when at least 10 ms have passed
    /// since the run's last snapshot.
    Transition,
    /// The run's end: always taken.
    RunEnd,
}

/// The tick and instant of a run's last snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Taken {
    pub(crate) tick: u64,
    pub(crate) at: Instant,
}

/// Whether `trigger` takes a snapshot at `tick` and `now`: never twice in one
/// tick, and otherwise by the trigger's rule.
pub(crate) fn due(
    trigger: Trigger,
    last: Option<Taken>,
    started: Instant,
    tick: u64,
    now: Instant,
    interval: Duration,
) -> bool {
    if last.is_some_and(|last| last.tick == tick) {
        return false;
    }
    match trigger {
        Trigger::Interval => {
            now.saturating_duration_since(last.map_or(started, |last| last.at)) >= interval
        }
        Trigger::Transition => {
            last.is_none_or(|last| now.saturating_duration_since(last.at) >= TRANSITION_GAP)
        }
        Trigger::RunEnd => true,
    }
}

/// A snapshot's stamp: the later of its capture time and the previous stamp
/// plus one millisecond. Never waits; absorbs a clock that steps back.
pub(crate) fn stamp(now_ns: u64, previous_ns: Option<u64>) -> u64 {
    previous_ns.map_or(now_ns, |previous| {
        now_ns.max(previous.saturating_add(STAMP_STEP_NS))
    })
}

/// The population census, with the CLI `tick_sample`'s expressions.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Census {
    pub population: usize,
    pub mean_energy: f32,
    pub mean_genome_size: f64,
    pub mean_mesh_nodes: f64,
    pub mean_generation: f64,
}

impl Census {
    /// All means are `0.0` for an empty population.
    pub fn of(sim: &Simulation) -> Self {
        let population = sim.creatures.len();
        if population == 0 {
            return Self::default();
        }
        let mean = |total: u64| total as f64 / population as f64;
        Self {
            population,
            mean_energy: sim.mean_energy(),
            mean_genome_size: mean(
                sim.creatures
                    .values()
                    .map(|c| u64::from(c.cached_genome_size))
                    .sum(),
            ),
            mean_mesh_nodes: mean(
                sim.creatures
                    .values()
                    .map(|c| c.genome.nodes.len() as u64)
                    .sum(),
            ),
            mean_generation: mean(sim.creatures.values().map(|c| c.generation).sum()),
        }
    }
}

/// A data point's value.
#[derive(Debug, Clone, Copy)]
enum Value {
    Int(u64),
    /// A signed total: its sum is not monotonic.
    Double(f64),
    /// Wall time in seconds: a monotonic `f64` sum.
    Seconds(f64),
}

impl From<u64> for Value {
    fn from(value: u64) -> Self {
        Self::Int(value)
    }
}

impl From<u32> for Value {
    fn from(value: u32) -> Self {
        Self::Int(u64::from(value))
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::Double(value)
    }
}

impl From<f32> for Value {
    fn from(value: f32) -> Self {
        Self::Double(f64::from(value))
    }
}

impl From<Duration> for Value {
    fn from(value: Duration) -> Self {
        Self::Seconds(value.as_secs_f64())
    }
}

/// How a family is exported.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    /// A gauge; `sampled` ones hold a per-tick value.
    Gauge { sampled: bool },
    /// A cumulative sum, monotonic unless it is a signed `f64` total.
    Sum,
}

type Labels = Vec<(&'static str, String)>;

/// The families of one snapshot, as they are built.
struct Families {
    run_id: KeyValue,
    time_ns: u64,
    start_ns: u64,
    metrics: Vec<Metric>,
}

fn string_value(key: &str, value: String) -> KeyValue {
    KeyValue {
        key: key.to_owned(),
        value: Some(AnyValue {
            value: Some(any_value::Value::StringValue(value)),
        }),
        ..KeyValue::default()
    }
}

/// A cumulative field's OTel name: every `_total` token removed.
fn run_name(field: &str) -> String {
    format!("petri.run.{}", field.replace("_total", ""))
}

impl Families {
    fn push(
        &mut self,
        name: String,
        kind: Kind,
        unit: &str,
        points: impl IntoIterator<Item = (Labels, Value)>,
    ) {
        let mut monotonic = true;
        let data_points: Vec<NumberDataPoint> = points
            .into_iter()
            .map(|(labels, value)| {
                let mut attributes = vec![self.run_id.clone()];
                attributes.extend(
                    labels
                        .into_iter()
                        .map(|(key, text)| string_value(key, text)),
                );
                let value = match value {
                    Value::Int(count) => {
                        number_data_point::Value::AsInt(i64::try_from(count).unwrap_or(i64::MAX))
                    }
                    Value::Double(amount) => {
                        monotonic = false;
                        number_data_point::Value::AsDouble(amount)
                    }
                    Value::Seconds(seconds) => number_data_point::Value::AsDouble(seconds),
                };
                NumberDataPoint {
                    attributes,
                    start_time_unix_nano: if kind == Kind::Sum { self.start_ns } else { 0 },
                    time_unix_nano: self.time_ns,
                    value: Some(value),
                    ..NumberDataPoint::default()
                }
            })
            .collect();
        if data_points.is_empty() {
            return;
        }
        let (description, data) = match kind {
            Kind::Gauge { sampled } => (
                if sampled { SAMPLED } else { "" },
                metric::Data::Gauge(Gauge { data_points }),
            ),
            Kind::Sum => (
                "",
                metric::Data::Sum(Sum {
                    data_points,
                    aggregation_temporality: AggregationTemporality::Cumulative as i32,
                    is_monotonic: monotonic,
                }),
            ),
        };
        self.metrics.push(Metric {
            name,
            description: description.to_owned(),
            unit: unit.to_owned(),
            data: Some(data),
            ..Metric::default()
        });
    }

    fn scalar(&mut self, name: String, kind: Kind, value: impl Into<Value>) {
        self.push(name, kind, "", [(Vec::new(), value.into())]);
    }

    /// A cumulative map family keyed by one attribute. Keys sharing an
    /// `as_key()` string are summed.
    fn counts<'a, K: 'a>(
        &mut self,
        name: String,
        attribute: &'static str,
        entries: impl IntoIterator<Item = (&'a K, &'a u64)>,
        key: impl Fn(&K) -> String,
    ) {
        let mut summed: BTreeMap<String, u64> = BTreeMap::new();
        for (k, count) in entries {
            *summed.entry(key(k)).or_default() += count;
        }
        self.push(
            name,
            Kind::Sum,
            "",
            summed
                .into_iter()
                .map(|(k, count)| (vec![(attribute, k)], Value::Int(count))),
        );
    }

    /// A cumulative map family keyed by operator and one inner attribute.
    fn nested<'a, K: 'a, I: 'a>(
        &mut self,
        name: String,
        inner: &'static str,
        entries: impl IntoIterator<Item = (&'a K, I)>,
        operator: impl Fn(&K) -> String,
        rows: impl Fn(I) -> Vec<(String, u64)>,
    ) {
        let mut summed: BTreeMap<(String, String), u64> = BTreeMap::new();
        for (k, value) in entries {
            let op = operator(k);
            for (key, count) in rows(value) {
                *summed.entry((op.clone(), key)).or_default() += count;
            }
        }
        self.push(
            name,
            Kind::Sum,
            "",
            summed.into_iter().map(|((op, key), count)| {
                (
                    vec![("petri.operator", op), (inner, key)],
                    Value::Int(count),
                )
            }),
        );
    }
}

/// The twenty fields of a [`MutationValueTotals`], `_total` removed.
fn value_fields(totals: &MutationValueTotals) -> [(&'static str, Value); 20] {
    [
        ("carriers_observed", totals.carriers_observed_total.into()),
        ("survival_ticks_sum", totals.survival_ticks_sum.into()),
        ("offspring_spawned_sum", totals.offspring_spawned_sum.into()),
        ("final_energy_sum", totals.final_energy_sum.into()),
        ("helpful", totals.helpful_total.into()),
        ("neutral", totals.neutral_total.into()),
        ("detrimental", totals.detrimental_total.into()),
        ("confidence_low", totals.confidence_low_total.into()),
        ("confidence_medium", totals.confidence_medium_total.into()),
        ("confidence_high", totals.confidence_high_total.into()),
        ("viability_score_sum", totals.viability_score_sum.into()),
        (
            "viability_score_delta_sum",
            totals.viability_score_delta_sum.into(),
        ),
        (
            "survived_short_horizon",
            totals.survived_short_horizon_total.into(),
        ),
        (
            "survived_long_horizon",
            totals.survived_long_horizon_total.into(),
        ),
        ("reproduced_once", totals.reproduced_once_total.into()),
        (
            "mean_lifetime_energy_sum",
            totals.mean_lifetime_energy_sum.into(),
        ),
        ("action_attempted", totals.action_attempted_total.into()),
        ("blocked_move", totals.blocked_move_total.into()),
        ("invalid_reproduce", totals.invalid_reproduce_total.into()),
        ("invalid_action", totals.invalid_action_total.into()),
    ]
}

/// Where and when a snapshot was taken.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Moment {
    pub(crate) tick: u64,
    pub(crate) elapsed: Duration,
    /// The snapshot's stamp.
    pub(crate) time_ns: u64,
    /// The run's start, carried by the cumulative sums.
    pub(crate) start_ns: u64,
}

macro_rules! run_scalars {
    ($families:ident, $stats:ident; $($field:ident),+ $(,)?) => {
        $( $families.scalar(run_name(stringify!($field)), Kind::Sum, $stats.$field); )+
    };
}

macro_rules! tick_gauges {
    ($families:ident, $stats:ident; $($field:ident => $name:literal),+ $(,)?) => {
        $(
            $families.scalar(
                concat!("petri.tick.", $name).to_owned(),
                Kind::Gauge { sampled: true },
                $stats.$field,
            );
        )+
    };
}

/// The cumulative `u64` scalars, deaths and reproductive success.
fn counters(f: &mut Families, s: &SimStats) {
    run_scalars!(f, s;
        reproduction_actions_attempted_total,
        reproduction_actions_spawned_total,
        reproduction_actions_rejected_total,
        mutation_events_attempted_total,
        mutation_events_applied_total,
        mutation_events_skipped_total,
        move_actions_attempted_total,
        mutation_reachable_target_total,
        mutation_unreachable_target_total,
        mutation_executed_target_total,
        mutation_not_applicable_target_total,
        mesh_hops_total,
        vm_steps_total,
        graph_relax_iters_total,
        plasticity_updates_total,
        plasticity_changes_total,
        hebbian_updates_total,
        hebbian_changes_total,
        reward_modulated_updates_total,
        reward_modulated_changes_total,
        shared_memory_writes_changed_total,
        creature_ticks_total,
        mesh_dispatches_energy_exhausted_total,
        pass_cap_hits_total,
        passes_total,
        decided_passes_total,
        actions_applied_total,
        predation_actions_attempted_total,
        predation_actions_transferred_total,
        predation_actions_rejected_total,
        predation_kills_total,
    );

    f.scalar(
        "petri.run.deaths".to_owned(),
        Kind::Sum,
        s.mortality.deaths_total,
    );
    f.push(
        "petri.run.deaths_by_cause".to_owned(),
        Kind::Sum,
        "",
        DeathCause::ALL.map(|cause| {
            (
                vec![("petri.cause", cause.as_key().to_owned())],
                Value::Int(s.mortality.count(cause)),
            )
        }),
    );
    let success = &s.reproductive_success_by_cognitive_class;
    for (field, read) in [
        ("creatures_observed", 0),
        ("offspring_spawned", 1),
        ("survival_ticks", 2),
    ] {
        f.push(
            format!("petri.run.reproductive_success.{field}"),
            Kind::Sum,
            "",
            CognitiveClass::ALL.map(|class| {
                let totals = success.totals(class);
                let value = match read {
                    0 => totals.creatures_observed_total,
                    1 => totals.offspring_spawned_sum,
                    _ => totals.survival_ticks_sum,
                };
                (
                    vec![("petri.cognitive_class", class.as_key().to_owned())],
                    Value::Int(value),
                )
            }),
        );
    }
}

/// Energy flows, genome carrying and phase wall time.
fn energy(f: &mut Families, s: &SimStats) {
    let flows = &s.energy_flows;
    let charges = &flows.action_charges;
    let mut flow_points: Vec<(String, f64)> = vec![
        (
            "failed_action_penalty".to_owned(),
            flows.failed_action_penalty,
        ),
        ("vm_compute".to_owned(), flows.vm_compute),
        ("priority_bid".to_owned(), flows.priority_bid),
        ("graph_compute".to_owned(), flows.graph_compute),
        ("mesh_ramp".to_owned(), flows.mesh_ramp),
        ("hebbian_learning".to_owned(), flows.hebbian_learning),
        ("reward_learning".to_owned(), flows.reward_learning),
        ("lifecycle_decay".to_owned(), flows.lifecycle_decay),
        ("genome_carrying".to_owned(), flows.genome_carrying),
        (
            "parental_transfer_debit".to_owned(),
            flows.parental_transfer_debit,
        ),
        (
            "offspring_energy_credit".to_owned(),
            flows.offspring_energy_credit,
        ),
        (
            "predation_victim_debit".to_owned(),
            flows.predation_victim_debit,
        ),
        (
            "predation_attacker_credit".to_owned(),
            flows.predation_attacker_credit,
        ),
        (
            "predation_kill_bonus_credit".to_owned(),
            flows.predation_kill_bonus_credit,
        ),
        (
            "maximum_energy_clamp_loss".to_owned(),
            flows.maximum_energy_clamp_loss,
        ),
        ("zero_floor_credit".to_owned(), flows.zero_floor_credit),
        (
            "external_removal_loss".to_owned(),
            flows.external_removal_loss,
        ),
        ("action_charge.noop".to_owned(), charges.noop),
        ("action_charge.eat".to_owned(), charges.eat),
        ("action_charge.move".to_owned(), charges.r#move),
        ("action_charge.reproduce".to_owned(), charges.reproduce),
        (
            "action_charge.steal_energy".to_owned(),
            charges.steal_energy,
        ),
    ];
    flow_points.extend(
        flows
            .food_intake_by_type
            .iter()
            .enumerate()
            .map(|(index, intake)| (format!("food_intake.{index}"), *intake)),
    );
    f.push(
        "petri.run.energy_flow".to_owned(),
        Kind::Sum,
        "",
        flow_points
            .into_iter()
            .map(|(flow, amount)| (vec![("petri.flow", flow)], Value::Double(amount))),
    );
    f.scalar(
        "petri.run.genome_size_creature_ticks".to_owned(),
        Kind::Sum,
        flows.genome_size_creature_ticks,
    );
    let phases = &s.phase_wall_clock;
    f.push(
        "petri.run.phase_wall_clock".to_owned(),
        Kind::Sum,
        "s",
        [
            ("world_update", phases.world_update),
            ("sensor_assembly", phases.sensor_assembly),
            ("cognition", phases.cognition),
            ("actions", phases.actions),
            ("reward_learning", phases.reward_learning),
        ]
        .map(|(phase, spent)| (vec![("petri.phase", phase.to_owned())], spent.into())),
    );
}

/// The mutation breakdowns and lifecycle value totals.
fn mutations(f: &mut Families, s: &SimStats) {
    for (field, map) in [
        (
            "mutation_events_attempted_total_by_domain",
            &s.mutation_events_attempted_total_by_domain,
        ),
        (
            "mutation_events_applied_total_by_domain",
            &s.mutation_events_applied_total_by_domain,
        ),
    ] {
        f.counts(run_name(field), "petri.domain", map, |k| {
            k.as_key().to_owned()
        });
    }
    for (field, map) in [
        (
            "mutation_events_attempted_total_by_operator",
            &s.mutation_events_attempted_total_by_operator,
        ),
        (
            "mutation_events_applied_total_by_operator",
            &s.mutation_events_applied_total_by_operator,
        ),
        (
            "mutation_events_skipped_total_by_operator",
            &s.mutation_events_skipped_total_by_operator,
        ),
    ] {
        f.counts(run_name(field), "petri.operator", map, |k| {
            k.as_key().to_owned()
        });
    }
    f.counts(
        run_name("mutation_events_skipped_by_reason"),
        "petri.skip_reason",
        &s.mutation_events_skipped_by_reason,
        |k| k.as_key().to_owned(),
    );
    f.nested(
        "petri.run.mutation_operator_funnel".to_owned(),
        "petri.stage",
        &s.mutation_operator_funnel_total_by_operator,
        |k| k.as_key().to_owned(),
        |funnel| {
            vec![
                ("attempted".to_owned(), funnel.attempted),
                ("applicable".to_owned(), funnel.applicable),
                ("structurally_valid".to_owned(), funnel.structurally_valid),
                ("applied".to_owned(), funnel.applied),
                ("skipped".to_owned(), funnel.skipped),
            ]
        },
    );
    f.nested(
        run_name("mutation_skip_reasons_total_by_operator"),
        "petri.skip_reason",
        &s.mutation_skip_reasons_total_by_operator,
        |k| k.as_key().to_owned(),
        |reasons| {
            reasons
                .iter()
                .map(|(reason, count)| (reason.as_key().to_owned(), *count))
                .collect()
        },
    );
    f.nested(
        run_name("mutation_added_node_input_classes_total_by_operator"),
        "petri.input_class",
        &s.mutation_added_node_input_classes_total_by_operator,
        |k| k.as_key().to_owned(),
        |classes| {
            classes
                .iter()
                .map(|(class, count)| (class.as_key().to_owned(), *count))
                .collect()
        },
    );
    f.nested(
        run_name("mutation_added_node_world_inputs_total_by_operator"),
        "petri.world_input",
        &s.mutation_added_node_world_inputs_total_by_operator,
        |k| k.as_key().to_owned(),
        |inputs| {
            inputs
                .iter()
                .map(|(input, count)| (input.as_key().to_owned(), *count))
                .collect()
        },
    );

    let by_operator: Vec<_> = s
        .mutation_value_totals_by_operator
        .iter()
        .map(|(operator, totals)| (operator.as_key(), value_fields(totals)))
        .collect();
    for (index, (field, _)) in value_fields(&MutationValueTotals::default())
        .into_iter()
        .enumerate()
    {
        let points = by_operator.iter().map(|(operator, fields)| {
            (
                vec![("petri.operator", (*operator).to_owned())],
                fields[index].1,
            )
        });
        f.push(
            format!("petri.run.mutation_value.{field}"),
            Kind::Sum,
            "",
            points,
        );
    }
    for (field, value) in value_fields(&s.mutation_outcome_summary) {
        f.scalar(
            format!("petri.run.mutation_outcome_summary.{field}"),
            Kind::Sum,
            value,
        );
    }
}

/// Reproduction, movement, eating and predation outcomes by key.
fn outcomes(f: &mut Families, s: &SimStats) {
    f.counts(
        run_name("reproduction_actions_rejected_by_reason"),
        "petri.reason",
        &s.reproduction_actions_rejected_by_reason,
        |k| k.as_key().to_owned(),
    );
    f.counts(
        run_name("reproduction_actions_rejected_invalid_target_total_by_cause"),
        "petri.cause",
        &s.reproduction_actions_rejected_invalid_target_total_by_cause,
        |k| k.as_key().to_owned(),
    );
    for (field, map) in [
        (
            "reproduction_actions_rejected_invalid_target_avoidable_total_by_reader_state",
            &s.reproduction_actions_rejected_invalid_target_avoidable_total_by_reader_state,
        ),
        (
            "move_actions_blocked_avoidable_total_by_reader_state",
            &s.move_actions_blocked_avoidable_total_by_reader_state,
        ),
        (
            "move_attempts_with_barrier_neighbor_total_by_reader_state",
            &s.move_attempts_with_barrier_neighbor_total_by_reader_state,
        ),
        (
            "move_blocked_barrier_with_barrier_neighbor_total_by_reader_state",
            &s.move_blocked_barrier_with_barrier_neighbor_total_by_reader_state,
        ),
        (
            "reproduction_attempts_with_barrier_neighbor_total_by_reader_state",
            &s.reproduction_attempts_with_barrier_neighbor_total_by_reader_state,
        ),
        (
            "reproduction_invalid_target_barrier_with_barrier_neighbor_total_by_reader_state",
            &s.reproduction_invalid_target_barrier_with_barrier_neighbor_total_by_reader_state,
        ),
    ] {
        f.counts(run_name(field), "petri.reader_state", map, |k| {
            k.as_key().to_owned()
        });
    }
    for (field, map) in [
        (
            "eat_actions_applied_total_by_type",
            &s.eat_actions_applied_total_by_type,
        ),
        (
            "eat_actions_failed_total_by_type",
            &s.eat_actions_failed_total_by_type,
        ),
    ] {
        f.counts(run_name(field), "petri.food_type", map, |k| {
            k.get().to_string()
        });
    }
    f.counts(
        run_name("move_actions_blocked_total_by_cause"),
        "petri.cause",
        &s.move_actions_blocked_total_by_cause,
        |k| k.as_key().to_owned(),
    );
    f.counts(
        run_name("predation_actions_by_result"),
        "petri.result",
        &s.predation_actions_by_result,
        |k| k.as_key().to_owned(),
    );
}

/// The per-tick values and the census, sampled at the snapshot tick.
fn tick_values(f: &mut Families, s: &SimStats, census: &Census) {
    let sampled = Kind::Gauge { sampled: true };
    f.push(
        "petri.tick.actions".to_owned(),
        sampled,
        "",
        [
            ("move", s.last_tick_move),
            ("eat", s.last_tick_eat),
            ("noop", s.last_tick_noop),
            ("reproduce", s.last_tick_reproduce),
            ("steal", s.last_tick_steal),
        ]
        .map(|(action, count)| (vec![("petri.action", action.to_owned())], count.into())),
    );
    tick_gauges!(f, s;
        last_tick_predation_kills => "predation_kills",
        last_tick_compute_total_mean => "compute_total_mean",
        last_tick_compute_total_min => "compute_total_min",
        last_tick_compute_total_max => "compute_total_max",
        last_tick_compute_vm_mean => "compute_vm_mean",
        last_tick_compute_graph_mean => "compute_graph_mean",
        last_tick_priority_bid_mean => "priority_bid_mean",
        last_tick_priority_bidders_count => "priority_bidders_count",
        last_tick_food_occupancy_depletion_mean => "food_occupancy_depletion_mean",
        last_tick_food_occupancy_depletion_occupied_cells => "food_occupancy_depletion_occupied_cells",
        last_tick_food_growth_suppressed_by_occupancy_depletion => "food_growth_suppressed_by_occupancy_depletion",
        last_tick_food_cells_with_type_inhibition => "food_cells_with_type_inhibition",
        last_tick_food_growth_suppressed_by_type_inhibition => "food_growth_suppressed_by_type_inhibition",
    );
    for (name, values) in [
        (
            "food_total_density",
            &s.last_tick_food_total_density_by_type,
        ),
        (
            "food_grazing_modifier_mean",
            &s.last_tick_food_grazing_modifier_mean_by_type,
        ),
        (
            "food_grazed_cell_share",
            &s.last_tick_food_grazed_cell_share_by_type,
        ),
    ] {
        f.push(
            format!("petri.tick.{name}"),
            sampled,
            "",
            values.iter().enumerate().map(|(index, value)| {
                (
                    vec![("petri.food_type", index.to_string())],
                    (*value).into(),
                )
            }),
        );
    }
    f.scalar(
        "petri.tick.population".to_owned(),
        sampled,
        census.population as u64,
    );
    f.scalar(
        "petri.tick.mean_energy".to_owned(),
        sampled,
        census.mean_energy,
    );
    f.scalar(
        "petri.tick.mean_genome_size".to_owned(),
        sampled,
        census.mean_genome_size,
    );
    f.scalar(
        "petri.tick.mean_mesh_nodes".to_owned(),
        sampled,
        census.mean_mesh_nodes,
    );
    f.scalar(
        "petri.tick.mean_generation".to_owned(),
        sampled,
        census.mean_generation,
    );
}

/// Encodes one snapshot as an OTLP metrics request.
pub(crate) fn encode(
    resource: &[KeyValue],
    run_id: &str,
    stats: &SimStats,
    census: &Census,
    moment: Moment,
) -> ExportMetricsServiceRequest {
    let mut f = Families {
        run_id: string_value(RUN_ID_KEY, run_id.to_owned()),
        time_ns: moment.time_ns,
        start_ns: moment.start_ns,
        metrics: Vec::new(),
    };
    let gauge = Kind::Gauge { sampled: false };
    f.scalar("petri.run.tick".to_owned(), gauge, moment.tick);
    f.scalar(
        "petri.run.elapsed_seconds".to_owned(),
        gauge,
        moment.elapsed.as_secs_f64(),
    );
    counters(&mut f, stats);
    energy(&mut f, stats);
    mutations(&mut f, stats);
    outcomes(&mut f, stats);
    tick_values(&mut f, stats, census);

    ExportMetricsServiceRequest {
        resource_metrics: vec![ResourceMetrics {
            resource: Some(Resource {
                attributes: resource.to_vec(),
                ..Resource::default()
            }),
            scope_metrics: vec![ScopeMetrics {
                scope: Some(InstrumentationScope {
                    name: "petri".to_owned(),
                    ..InstrumentationScope::default()
                }),
                metrics: f.metrics,
                ..ScopeMetrics::default()
            }],
            ..ResourceMetrics::default()
        }],
    }
}

#[cfg(test)]
mod tests;
