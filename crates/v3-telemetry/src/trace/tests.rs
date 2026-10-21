use std::collections::BTreeSet;
use std::time::{Duration, Instant};

use proptest::prelude::*;
use prost::Message as _;
use v3_core::config::{FoodTypeConfig, OrdinaryFoodTypeId, SimulationConfig};
use v3_core::kernel::FoodPass;
use v3_core::simulation::stats::{FoodPassSpan, PhaseTiming, TickPhaseTimings};
use v3_core::simulation::SimStats;

use super::*;
use crate::testing::{decode_traces, ReceivedTrace};

const RUN_KEY: u128 = 0x0123_4567_89ab_cdef_fedc_ba98_7654_3210;
const STARTED_NS: u64 = 1_700_000_000_000_000_000;
/// A config digest of realistic length (64 hex digits).
const DIGEST: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn run(started: Instant, recipe: Option<&str>) -> RunHandle {
    RunHandle {
        key: RUN_KEY,
        id: format!("{RUN_KEY:032x}"),
        seed: 7,
        world: "16x8".to_owned(),
        recipe: recipe.map(str::to_owned),
        config_digest: DIGEST.to_owned(),
        started,
        started_ns: STARTED_NS,
        last_snapshot: None,
        last_stamp_ns: None,
        traces_taken: 0,
        samples: crate::windows::Samples::default(),
    }
}

fn ms(value: u64) -> Duration {
    Duration::from_millis(value)
}

/// Every food pass, `recovery_spawns` included, for `food_types` types, in
/// execution order.
fn all_passes(food_types: usize) -> Vec<(FoodPass, Option<u16>)> {
    let mut passes = vec![
        (FoodPass::DepletionRecovery, None),
        (FoodPass::GrazingRecovery, None),
        (FoodPass::InhibitionSums, None),
    ];
    for food_type in 0..food_types as u16 {
        for pass in [
            FoodPass::DensityTotal,
            FoodPass::GrowthSpread,
            FoodPass::RecoverySpawns,
            FoodPass::WriteBack,
        ] {
            passes.push((pass, Some(food_type)));
        }
    }
    passes.push((FoodPass::InhibitedCount, None));
    passes
}

/// Tick 5 starting 2 ms after the run, 14 ms long; phase `i` starts at
/// `2i` ms and lasts 1 ms; food pass `k` starts at `10k` us and lasts 5 us,
/// inside `food_growth`.
fn seam(run_started: Instant, food_types: usize) -> (TickPhaseTimings, Vec<FoodPassSpan>) {
    let mut phases = [PhaseTiming::default(); 6];
    for (index, phase) in phases.iter_mut().enumerate() {
        *phase = PhaseTiming {
            offset: ms(2 * index as u64),
            elapsed: ms(1),
        };
    }
    let food_passes = all_passes(food_types)
        .into_iter()
        .enumerate()
        .map(|(index, (pass, food_type))| FoodPassSpan {
            pass,
            food_type: food_type.map(OrdinaryFoodTypeId::new),
            timing: PhaseTiming {
                offset: Duration::from_micros(10 * index as u64),
                elapsed: Duration::from_micros(5),
            },
        })
        .collect();
    let seam = TickPhaseTimings {
        tick: 5,
        population_start: 10,
        threads: 4,
        started: run_started + ms(2),
        elapsed: ms(14),
        phases,
    };
    (seam, food_passes)
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
    let (seam, food_passes) = seam(started, config.world.food.types.len());
    let request = encode(
        &[],
        &run,
        &Capture {
            seam: &seam,
            food_passes: &food_passes,
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

/// The keys every span carries.
fn every_keys() -> BTreeSet<String> {
    keys(&[
        "petri.run_id",
        "petri.seed",
        "petri.world",
        "petri.recipe",
        "petri.config_digest",
        "petri.tick",
        "petri.sample_policy",
        "petri.phase",
    ])
}

/// A food pass span's keys: the every-span set, plus `petri.food_type` on a
/// per-type pass.
fn pass_keys(per_type: bool) -> BTreeSet<String> {
    let mut keys = every_keys();
    if per_type {
        keys.insert("petri.food_type".to_owned());
    }
    keys
}

/// The key sets of `tick` and the six phase spans, in span order.
fn expected_keys(food_types: usize) -> [BTreeSet<String>; 7] {
    let every = every_keys();
    let mut tick = keys(&[
        "petri.population_start",
        "petri.threads",
        "petri.population",
        "petri.actions.move",
        "petri.actions.eat",
        "petri.actions.noop",
        "petri.actions.reproduce",
        "petri.actions.steal",
        "petri.actions.give",
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
    let world_update = config_keys(&[
        "energy.lifecycle.energy_decay_per_tick",
        "energy.lifecycle.genome_carry_cost_per_unit",
        "shared_memory.decay_rate",
    ]);
    let mut food_growth = config_keys(&[
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
            food_growth.insert(config_key(&format!("world.food.types.{index}.{field}")));
        }
    }
    let sensor_assembly = config_keys(&["runtime.perception.vision_radius", "world.edge_mode"]);
    let cognition = config_keys(&[
        "runtime.max_mesh_hops",
        "runtime.max_vm_steps",
        "runtime.graph_node_base_cost",
        "runtime.hop_ramp_allowance",
        "runtime.hop_ramp_cost",
        "runtime.max_actions_per_turn",
        "runtime.vm.opcode_cost_multiplier",
        "runtime.vm.step_ramp_allowance",
        "runtime.vm.step_ramp_cost",
    ]);
    let actions = config_keys(&[
        "energy.costs.move_cost",
        "energy.costs.eat_cost",
        "energy.costs.eat_reward_per_food",
        "energy.costs.noop_cost",
        "energy.costs.reproduce_cost",
        "energy.costs.reproduce_attempt_cost",
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
        "energy.lifecycle.gestation_ticks_per_founder_genome",
        "population.max_creatures",
        "mutation.per_unit_rate",
        "mutation.genome_size_cap",
        "mutation.genome_size_pressure_enabled",
        "mutation.mesh_layer_probability",
        "mutation.executed_bias",
        "predation.steal_cost_rate",
        "predation.kill_complexity_bonus_multiplier",
        "exchange.bite",
    ]);
    let reward_learning = config_keys(&[]);
    [
        tick,
        food_growth,
        world_update,
        sensor_assembly,
        cognition,
        actions,
        reward_learning,
    ]
    .map(|own| own.union(&every).cloned().collect())
}

#[test]
fn a_synthetic_seam_encodes_tick_six_phases_and_the_food_passes_with_predicted_ids_times_names_and_keys(
) {
    let config = config_with_types(2);
    let (_, trace) = encoded(&config, &stats(2), Some("recipe.json"));
    let trace_id = format!("{:016x}{:016x}", RUN_KEY >> 64, 5);
    let passes = all_passes(2);
    let names: Vec<&str> = trace.spans.iter().map(|span| span.name.as_str()).collect();
    let mut expected_names = vec![
        "tick",
        "food_growth",
        "world_update",
        "sensor_assembly",
        "cognition",
        "actions",
        "reward_learning",
    ];
    expected_names.extend(passes.iter().map(|(pass, _)| pass.name()));
    assert_eq!(names, expected_names);
    assert_eq!(trace.spans.len(), 7 + 12);
    let tick_start = STARTED_NS + 2_000_000;
    let mut expected = expected_keys(2).to_vec();
    expected.extend(
        passes
            .iter()
            .map(|(_, food_type)| pass_keys(food_type.is_some())),
    );
    for (index, (span, expected)) in trace.spans.iter().zip(expected).enumerate() {
        assert_eq!(span.trace_id, trace_id);
        assert_eq!(span.span_id, format!("{:016x}", index + 1), "{}", span.name);
        let parent = match index {
            0 => String::new(),
            1..=6 => format!("{:016x}", 1),
            _ => format!("{:016x}", 2),
        };
        assert_eq!(span.parent_span_id, parent, "{}", span.name);
        let (start, length) = match index {
            0 => (tick_start, 14_000_000),
            1..=6 => (tick_start + 2_000_000 * (index as u64 - 1), 1_000_000),
            _ => (tick_start + 10_000 * (index as u64 - 7), 5_000),
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
    let food_growth = &trace.spans[1];
    assert_eq!(food_growth.attribute("petri.phase"), Some("food_growth"));
    assert_eq!(
        food_growth.attribute("petri.config.world.food.types"),
        Some("2")
    );
    let sensors = &trace.spans[3];
    assert_eq!(
        sensors.attribute("petri.config.world.edge_mode"),
        serde_json::to_value(config.world.edge_mode)
            .unwrap()
            .as_str()
    );
    let actions = &trace.spans[5];
    assert_eq!(actions.attribute("petri.failed_action_penalty"), None);
    assert_eq!(
        actions.attribute("petri.config.energy.costs.reproduce_attempt_cost"),
        Some(
            f64::from(config.energy.costs.reproduce_attempt_cost)
                .to_string()
                .as_str()
        )
    );
    for (span, (_, food_type)) in trace.spans[7..].iter().zip(&passes) {
        assert_eq!(span.attribute("petri.phase"), Some("food_growth"));
        let expected = food_type.map(|index| index.to_string());
        assert_eq!(
            span.attribute("petri.food_type"),
            expected.as_deref(),
            "{}",
            span.name
        );
    }
}

#[test]
fn a_type_without_its_own_rates_carries_the_shared_ones_and_no_recipe_is_omitted() {
    let mut config = config_with_types(1);
    config.world.food.types[0].growth_rate = None;
    config.world.food.types[0].recovery_spawn_rate = None;
    config.world.food.types[0].energy_per_unit = None;
    config.world.food.types[0].growth_inhibitor = 0.25;
    let (_, trace) = encoded(&config, &stats(1), None);
    let food_growth = &trace.spans[1];
    let value = |field: &str| {
        food_growth
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
fn a_two_type_trace_with_every_food_pass_encodes_under_16_kib() {
    let (body, trace) = encoded(&config_with_types(2), &stats(2), Some("recipe.json"));
    assert_eq!(trace.spans.len(), 7 + 4 + 4 * 2);
    assert!(body.len() < 16 * 1024, "{} bytes", body.len());
}

#[test]
fn an_eight_type_trace_with_every_food_pass_encodes_under_32_kib() {
    let (body, trace) = encoded(&config_with_types(8), &stats(8), Some("recipe.json"));
    assert_eq!(trace.spans.len(), 7 + 4 + 4 * 8);
    assert!(body.len() < 32 * 1024, "{} bytes", body.len());
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
