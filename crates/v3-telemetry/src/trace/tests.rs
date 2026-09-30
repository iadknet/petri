use std::collections::BTreeSet;
use std::time::{Duration, Instant};

use proptest::prelude::*;
use prost::Message as _;
use v3_core::config::{FoodTypeConfig, SimulationConfig};
use v3_core::simulation::stats::{PhaseTiming, TickPhaseTimings};
use v3_core::simulation::SimStats;

use super::*;
use crate::testing::{decode_traces, ReceivedTrace};

const RUN_KEY: u128 = 0x0123_4567_89ab_cdef_fedc_ba98_7654_3210;
const STARTED_NS: u64 = 1_700_000_000_000_000_000;

fn run(started: Instant, recipe: Option<&str>) -> RunHandle {
    RunHandle {
        key: RUN_KEY,
        id: format!("{RUN_KEY:032x}"),
        seed: 7,
        world: "16x8".to_owned(),
        recipe: recipe.map(str::to_owned),
        config_digest: "digest".to_owned(),
        started,
        started_ns: STARTED_NS,
        last_snapshot: None,
        last_stamp_ns: None,
        traces_taken: 0,
    }
}

fn ms(value: u64) -> Duration {
    Duration::from_millis(value)
}

/// Tick 5 starting 2 ms after the run, 10 ms long; phase `i` starts at
/// `2i` ms and lasts 1 ms.
fn seam(run_started: Instant) -> TickPhaseTimings {
    let mut phases = [PhaseTiming::default(); 5];
    for (index, phase) in phases.iter_mut().enumerate() {
        *phase = PhaseTiming {
            offset: ms(2 * index as u64),
            elapsed: ms(1),
        };
    }
    TickPhaseTimings {
        tick: 5,
        population_start: 10,
        threads: 4,
        started: run_started + ms(2),
        elapsed: ms(10),
        phases,
    }
}

fn config_with_types(count: usize) -> SimulationConfig {
    let mut config = SimulationConfig::default();
    let template = config.world.food.types[0].clone();
    config.world.food.types = (0..count)
        .map(|index| FoodTypeConfig {
            name: format!("type-{index}"),
            ..template.clone()
        })
        .collect();
    config
}

fn stats(food_types: usize) -> SimStats {
    SimStats {
        last_tick_food_total_density_by_type: vec![0.5; food_types],
        ..SimStats::default()
    }
}

fn encoded(
    config: &SimulationConfig,
    stats: &SimStats,
    recipe: Option<&str>,
) -> (Vec<u8>, ReceivedTrace) {
    let started = Instant::now();
    let run = run(started, recipe);
    let seam = seam(started);
    let request = encode(
        &[],
        &run,
        &Capture {
            seam: &seam,
            stats,
            config,
            population: 12,
            sample: TickSample::Interval,
        },
    );
    let body = request.encode_to_vec();
    let mut traces = decode_traces(&body);
    assert_eq!(traces.len(), 1);
    (body, traces.remove(0))
}

fn keys(names: &[&str]) -> BTreeSet<String> {
    names.iter().map(|name| (*name).to_owned()).collect()
}

fn config_keys(paths: &[&str]) -> BTreeSet<String> {
    paths.iter().map(|path| config_key(path)).collect()
}

fn expected_keys(food_types: usize) -> [BTreeSet<String>; 6] {
    let every = keys(&[
        "petri.run_id",
        "petri.seed",
        "petri.world",
        "petri.recipe",
        "petri.config_digest",
        "petri.tick",
        "petri.sample_policy",
        "petri.phase",
    ]);
    let mut tick = keys(&[
        "petri.population_start",
        "petri.threads",
        "petri.population",
        "petri.actions.move",
        "petri.actions.eat",
        "petri.actions.noop",
        "petri.actions.reproduce",
        "petri.actions.steal",
        "petri.predation_kills",
        "petri.compute_total_mean",
        "petri.compute_total_min",
        "petri.compute_total_max",
        "petri.compute_vm_mean",
        "petri.compute_graph_mean",
        "petri.priority_bid_mean",
        "petri.priority_bidders_count",
    ]);
    tick.extend((0..food_types).map(|index| format!("petri.food_total_density.{index}")));
    let mut world_update = config_keys(&[
        "energy.lifecycle.energy_decay_per_tick",
        "energy.lifecycle.genome_carry_cost_per_unit",
        "shared_memory.decay_rate",
        "world.food.shared.growth_rate",
        "world.food.shared.max_density",
        "world.food.shared.grazing.enabled",
        "world.food.shared.occupancy_depletion.enabled",
        "world.food.annealing.enabled",
        "world.food.types",
    ]);
    for index in 0..food_types {
        for field in [
            "growth_inhibitor",
            "growth_rate",
            "recovery_spawn_rate",
            "energy_per_unit",
        ] {
            world_update.insert(config_key(&format!("world.food.types.{index}.{field}")));
        }
    }
    let sensor_assembly = config_keys(&["runtime.perception.vision_radius", "world.edge_mode"]);
    let cognition = config_keys(&[
        "runtime.max_mesh_hops",
        "runtime.max_vm_steps",
        "runtime.graph_node_base_cost",
        "runtime.plasticity_update_cost",
        "runtime.hop_ramp_allowance",
        "runtime.hop_ramp_cost",
        "runtime.max_actions_per_turn",
        "runtime.vm.opcode_cost_multiplier",
        "runtime.vm.step_ramp_allowance",
        "runtime.vm.step_ramp_cost",
    ]);
    let mut actions = config_keys(&[
        "energy.costs.move_cost",
        "energy.costs.eat_cost",
        "energy.costs.eat_reward_per_food",
        "energy.costs.noop_cost",
        "energy.costs.reproduce_cost",
        "energy.complexity_cost.enabled",
        "energy.complexity_cost.threshold",
        "energy.complexity_cost.scaling_factor",
        "energy.age_cost.enabled",
        "energy.age_cost.grace_ticks",
        "energy.age_cost.age_cap",
        "energy.age_cost.max_multiplier",
        "energy.lifecycle.min_reproduce_energy",
        "energy.lifecycle.min_reproduce_age",
        "energy.lifecycle.genome_replication_cost_per_unit",
        "population.max_creatures",
        "mutation.per_unit_rate",
        "mutation.genome_size_cap",
        "mutation.genome_size_pressure_enabled",
        "mutation.mesh_layer_probability",
        "mutation.executed_bias",
        "predation.steal_cost_rate",
        "predation.kill_complexity_bonus_multiplier",
    ]);
    actions.insert("petri.failed_action_penalty".to_owned());
    let reward_learning = config_keys(&["runtime.reward_learning_cost"]);
    [
        tick,
        world_update,
        sensor_assembly,
        cognition,
        actions,
        reward_learning,
    ]
    .map(|own| own.union(&every).cloned().collect())
}

#[test]
fn a_synthetic_seam_encodes_six_spans_with_predicted_ids_times_names_and_keys() {
    let config = config_with_types(2);
    let (_, trace) = encoded(&config, &stats(2), Some("recipe.json"));
    let trace_id = format!("{:016x}{:016x}", RUN_KEY >> 64, 5);
    let names: Vec<&str> = trace.spans.iter().map(|span| span.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "tick",
            "world_update",
            "sensor_assembly",
            "cognition",
            "actions",
            "reward_learning"
        ]
    );
    let tick_start = STARTED_NS + 2_000_000;
    for (index, (span, expected)) in trace.spans.iter().zip(expected_keys(2)).enumerate() {
        assert_eq!(span.trace_id, trace_id);
        assert_eq!(span.span_id, format!("{:016x}", index + 1));
        let parent = if index == 0 {
            String::new()
        } else {
            format!("{:016x}", 1)
        };
        assert_eq!(span.parent_span_id, parent);
        let (start, length) = if index == 0 {
            (tick_start, 10_000_000)
        } else {
            (tick_start + 2_000_000 * (index as u64 - 1), 1_000_000)
        };
        assert_eq!(span.start_time_unix_nano, start, "{}", span.name);
        assert_eq!(span.end_time_unix_nano, start + length, "{}", span.name);
        let actual: BTreeSet<String> = span.attributes.keys().cloned().collect();
        assert_eq!(actual, expected, "{}", span.name);
        assert_eq!(span.attribute("petri.tick"), Some("5"));
        assert_eq!(span.attribute("petri.sample_policy"), Some("interval"));
        assert_eq!(
            span.attribute("petri.run_id"),
            Some(trace.run_id().unwrap())
        );
    }
    let tick = &trace.spans[0];
    assert_eq!(tick.attribute("petri.phase"), Some("tick"));
    assert_eq!(tick.attribute("petri.population_start"), Some("10"));
    assert_eq!(tick.attribute("petri.threads"), Some("4"));
    assert_eq!(tick.attribute("petri.population"), Some("12"));
    let world_update = &trace.spans[1];
    assert_eq!(
        world_update.attribute("petri.config.world.food.types"),
        Some("2")
    );
    let sensors = &trace.spans[2];
    assert_eq!(
        sensors.attribute("petri.config.world.edge_mode"),
        serde_json::to_value(config.world.edge_mode)
            .unwrap()
            .as_str()
    );
    let actions = &trace.spans[4];
    assert_eq!(
        actions.attribute("petri.failed_action_penalty"),
        Some(
            f64::from(config.failed_action_penalty_for_tick(4))
                .to_string()
                .as_str()
        )
    );
}

#[test]
fn a_type_without_its_own_rates_carries_the_shared_ones_and_no_recipe_is_omitted() {
    let mut config = config_with_types(1);
    config.world.food.types[0].growth_rate = None;
    config.world.food.types[0].recovery_spawn_rate = None;
    config.world.food.types[0].energy_per_unit = None;
    config.world.food.types[0].growth_inhibitor = 0.25;
    let (_, trace) = encoded(&config, &stats(1), None);
    let world_update = &trace.spans[1];
    let value = |field: &str| {
        world_update
            .attribute(&format!("petri.config.world.food.types.0.{field}"))
            .unwrap()
            .to_owned()
    };
    let shared = &config.world.food.shared;
    assert_eq!(
        value("growth_rate"),
        f64::from(shared.growth_rate).to_string()
    );
    assert_eq!(
        value("recovery_spawn_rate"),
        f64::from(shared.recovery_spawn_rate).to_string()
    );
    assert_eq!(
        value("energy_per_unit"),
        f64::from(config.energy.costs.eat_reward_per_food).to_string()
    );
    assert_eq!(value("growth_inhibitor"), "0.25");
    assert!(trace
        .spans
        .iter()
        .all(|span| span.attribute("petri.recipe").is_none()));
}

#[test]
fn an_eight_type_config_encodes_under_16_kib() {
    let (body, trace) = encoded(&config_with_types(8), &stats(8), Some("recipe.json"));
    assert_eq!(trace.spans.len(), 6);
    assert!(body.len() < 16 * 1024, "{} bytes", body.len());
}

#[test]
fn the_tick_trace_switch_defaults_on_and_refuses_anything_but_on_or_off() {
    assert_eq!(resolve_tick_traces(None), Ok(Switch::On));
    assert_eq!(resolve_tick_traces(Some(" ")), Ok(Switch::On));
    assert_eq!(resolve_tick_traces(Some("off")), Ok(Switch::Off));
    assert_eq!(resolve_tick_traces(Some("on")), Ok(Switch::On));
    for invalid in ["yes", "OFF", "0"] {
        let error = resolve_tick_traces(Some(invalid)).unwrap_err();
        assert!(error.contains(TICK_TRACES_ENV), "{error}");
    }
}

proptest! {
    #[test]
    fn a_trace_id_is_the_run_ids_first_16_hex_digits_then_the_tick(key: u128, tick: u64) {
        let id: String = trace_id(key, tick).iter().map(|byte| format!("{byte:02x}")).collect();
        let run_id = format!("{key:032x}");
        prop_assert_eq!(id, format!("{}{tick:016x}", &run_id[..16]));
    }
}
