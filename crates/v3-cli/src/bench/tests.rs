//! Tests that exercise items from several `bench` modules, and shared test helpers.

use super::comparison::compare_cases;
use super::profiles::{
    build_config, goal_case, goal_profile_params, NeighborhoodSizes, ProfileParams, GOAL_RECIPES,
};
use super::run::{build_report, run_deterministic, run_one_seed};
use super::schema::Report;
use super::tracking::{OccupancyGrid, PopulationReadings, SensorCensus, WorldTracking};
use crate::six;
use v3_core::creature::sensor_census::{world_input_key_label, world_input_key_universe};
use v3_core::kernel::occupancy_grid::OCCUPANCY_CELLS_PER_AXIS;
use v3_core::kernel::occupancy_grid::OCCUPANCY_CELL_COUNT;
use v3_core::simulation::{run_tick, seed_simulation};

#[test]
fn companion_reports_share_final_population_and_do_not_change_comparisons() {
    let report = small_world_set_report();
    let indicators = &report.deterministic.goal_indicators;
    let companions = indicators.structural_companions.as_ref().unwrap();
    let memory = indicators.memory_sensitivity.defined().unwrap();
    assert_eq!(companions.per_seed.len(), memory.per_seed.len());
    for (census, sensitivity) in companions.per_seed.iter().zip(&memory.per_seed) {
        assert_eq!(census.seed, sensitivity.seed);
        assert_eq!(
            census.final_creature_count,
            sensitivity.final_creature_count
        );
    }
    let mut historical = report.clone();
    historical
        .deterministic
        .goal_indicators
        .structural_companions = None;
    for case in &mut historical.deterministic.goal_indicators.cases {
        case.tracking.cognition = None;
    }
    assert_eq!(
        compare_cases(&(&report).into(), &(&report).into()),
        compare_cases(&(&report).into(), &(&historical).into())
    );
    let value = serde_json::to_value(&historical).unwrap();
    assert!(value["deterministic"]["goal_indicators"]
        .get("structural_companions")
        .is_none());
    let loaded: Report = serde_json::from_value(value).unwrap();
    assert!(loaded
        .deterministic
        .goal_indicators
        .structural_companions
        .is_none());
    assert_eq!(
        loaded.deterministic.goal_indicators.memory_sensitivity,
        indicators.memory_sensitivity
    );
    assert_eq!(
        loaded
            .deterministic
            .goal_indicators
            .temporal_memory_sensitivity,
        indicators.temporal_memory_sensitivity
    );
    let (unmeasured, _) = run_deterministic(&small_profile("synthetic")).unwrap();
    assert!(unmeasured.goal_indicators.structural_companions.is_none());
}

/// The synthetic-series test above drives the accumulator directly; this
/// one binds the composition inside `run_one_seed`, the call site T14.F08,
/// T14.F09 and T14.F10 hang their own series off. The oracle is an
/// independent re-run of the same seed to the same horizon: byte-identical
/// reproducibility (T10.F11) makes the oracle's terminal state the state
/// `run_one_seed` sampled on its horizon tick, so the equality catches a
/// reading taken before `run_tick`, and the inequality against tick 100
/// catches one reading reused for every checkpoint.
#[test]
fn the_real_run_path_reads_every_checkpoint_from_its_own_post_tick_state() {
    // Arrange: a 32x32 world with 8 founders over 250 ticks, past the
    // founding crash near tick 150, so the horizon population is a
    // recovered one whose readings differ from tick 100's. Seed 13 was
    // measured to survive the horizon.
    const SEED: u64 = 13;
    const HORIZON: u64 = 250;
    let config = build_config(&ProfileParams {
        recipe: None,
        name: "sweep".to_string(),
        width: 32,
        height: 32,
        founders: 8,
        seeds: vec![SEED],
        ticks: HORIZON,
        food_coverage: None,
        neighborhood: NeighborhoodSizes::default(),
        drift: Default::default(),
        recruitment: v3_core::neighborhood::recruitment_paths::Sizes::TEST,
    });

    // Act
    let run = run_one_seed(&config, SEED, HORIZON, false, None);
    let mut oracle = seed_simulation(config.clone(), SEED);
    for _ in 0..HORIZON {
        run_tick(&mut oracle, &mut None);
    }
    let terminal = PopulationReadings::observe(&oracle);

    // Assert: the run reached the horizon alive, so the horizon readings
    // are measured values and the comparison below is not `None` against
    // `Some`.
    assert_eq!(
        run.persistence.extinction_tick, None,
        "the fixture must survive the horizon for this test to bind anything"
    );
    let horizon_sample = run
        .persistence
        .samples
        .iter()
        .find(|sample| sample.tick == HORIZON)
        .expect("the horizon tick is sampled");
    assert_eq!(horizon_sample.population, oracle.creatures.len() as u64);
    assert!(horizon_sample.population > 0);

    assert_eq!(
        horizon_sample.mean_genome_size.as_deref(),
        Some(six(terminal.mean_genome_size).as_str())
    );
    assert_eq!(
        horizon_sample.mean_mesh_nodes.as_deref(),
        Some(six(terminal.mean_mesh_nodes).as_str())
    );
    assert_eq!(
        horizon_sample.mean_generation.as_deref(),
        Some(six(terminal.mean_generation).as_str())
    );
    assert_eq!(
        horizon_sample.surviving_founder_clade_count,
        Some(terminal.surviving_founder_clade_count)
    );
    assert_eq!(
        horizon_sample.shannon_entropy_nats.as_deref(),
        Some(terminal.shannon_entropy_nats.as_str())
    );

    // A checkpoint carries its own tick's readings, not the run's.
    let early_sample = run
        .persistence
        .samples
        .iter()
        .find(|sample| sample.tick == 100)
        .expect("tick 100 is sampled");
    assert_ne!(
        early_sample.mean_generation, horizon_sample.mean_generation,
        "generations advance between tick 100 and the horizon"
    );
    assert_ne!(
        early_sample.surviving_founder_clade_count, horizon_sample.surviving_founder_clade_count,
        "founder clades are lost between tick 100 and the horizon"
    );
}

#[test]
fn the_real_run_path_carries_a_census_of_the_whole_key_universe_at_every_checkpoint() {
    const SEED: u64 = 13;
    const HORIZON: u64 = 250;
    let config = build_config(&ProfileParams {
        recipe: None,
        name: "sweep".to_string(),
        width: 32,
        height: 32,
        founders: 8,
        seeds: vec![SEED],
        ticks: HORIZON,
        food_coverage: None,
        neighborhood: NeighborhoodSizes::default(),
        drift: Default::default(),
        recruitment: v3_core::neighborhood::recruitment_paths::Sizes::TEST,
    });

    // Act
    let run = run_one_seed(&config, SEED, HORIZON, false, None);
    let mut oracle = seed_simulation(config.clone(), SEED);
    for _ in 0..HORIZON {
        run_tick(&mut oracle, &mut None);
    }

    // Assert: every checkpoint carries a census, and every census carries
    // the world's whole key universe in key order.
    let expected_keys: Vec<String> = world_input_key_universe(
        oracle
            .world
            .food()
            .food_types()
            .iter()
            .map(|food_type| food_type.id),
    )
    .into_iter()
    .map(world_input_key_label)
    .collect();
    assert!(!run.persistence.samples.is_empty());
    for sample in &run.persistence.samples {
        let census = sample
            .sensor_census
            .as_ref()
            .expect("every checkpoint carries a census");
        let keys: Vec<String> = census
            .world_inputs
            .iter()
            .map(|row| row.key.clone())
            .collect();
        assert_eq!(keys, expected_keys, "tick {}", sample.tick);
        assert!(
            census.creatures_with_any_stateful_read
                >= census
                    .creatures_reading_shared_memory
                    .max(census.creatures_with_stateful_node),
            "the combined count covers each split: {census:?}"
        );
        for row in &census.world_inputs {
            assert!(
                row.creatures <= sample.population,
                "a creature counts at most once per key: {row:?} at tick {}",
                sample.tick
            );
        }
        for count in [
            census.creatures_reading_shared_memory,
            census.creatures_with_stateful_node,
            census.creatures_with_any_stateful_read,
        ] {
            assert!(
                count <= sample.population,
                "a creature counts at most once per stateful reading: \
                 {census:?} at tick {}",
                sample.tick
            );
        }
    }

    // The horizon checkpoint equals the census of the oracle's terminal
    // state — the state `run_one_seed` sampled on that tick, by T10.F11.
    let horizon_sample = run
        .persistence
        .samples
        .iter()
        .find(|sample| sample.tick == HORIZON)
        .expect("the horizon tick is sampled");
    assert!(horizon_sample.population > 0);
    assert_eq!(
        horizon_sample.sensor_census.as_ref(),
        Some(&SensorCensus::observe(&oracle))
    );

    // The founder population's graph-backend references are visible.
    let first = run.persistence.samples[0]
        .sensor_census
        .as_ref()
        .expect("the first checkpoint carries a census");
    let food_here = first
        .world_inputs
        .iter()
        .find(|row| row.key == "FoodHere:0")
        .expect("the primary food key is in the universe");
    assert!(
        food_here.creatures > 0,
        "graph-backend world input references are counted: {first:?}"
    );
}

#[test]
fn the_real_run_path_carries_a_full_occupancy_grid_at_every_checkpoint() {
    // Evolved-trajectory pin. T19.F04's vote founder moves every
    // trajectory: seed 3 now goes extinct before the horizon, while seed 1
    // keeps 2 creatures over 2 cells (the lowest seed that does; 5 of seeds
    // 0..=25 go extinct and 11 end on one cell). T11.F23 had moved this pin
    // from seed 1 to 3, T16.F01 from 18 to 1, T11.F21 from 13 to 18.
    const SEED: u64 = 1;
    const HORIZON: u64 = 250;
    let config = build_config(&ProfileParams {
        recipe: None,
        name: "sweep".to_string(),
        width: 32,
        height: 32,
        founders: 8,
        seeds: vec![SEED],
        ticks: HORIZON,
        food_coverage: None,
        neighborhood: NeighborhoodSizes::default(),
        drift: Default::default(),
        recruitment: v3_core::neighborhood::recruitment_paths::Sizes::TEST,
    });

    // Act
    let run = run_one_seed(&config, SEED, HORIZON, false, None);
    let mut oracle = seed_simulation(config.clone(), SEED);
    for _ in 0..HORIZON {
        run_tick(&mut oracle, &mut None);
    }

    assert!(!run.persistence.samples.is_empty());
    for sample in &run.persistence.samples {
        let grid = sample
            .occupancy_grid
            .as_ref()
            .expect("every checkpoint carries an occupancy grid");
        assert_eq!(grid.cells_x, OCCUPANCY_CELLS_PER_AXIS);
        assert_eq!(grid.cells_y, OCCUPANCY_CELLS_PER_AXIS);
        assert_eq!(grid.population.len(), OCCUPANCY_CELL_COUNT);
        assert_eq!(grid.distinct_clades.len(), OCCUPANCY_CELL_COUNT);
        assert_eq!(
            grid.population.iter().sum::<u64>(),
            sample.population,
            "every living creature is binned exactly once at tick {}",
            sample.tick
        );
        for cell in 0..OCCUPANCY_CELL_COUNT {
            assert!(
                grid.distinct_clades[cell] <= grid.population[cell],
                "a cell counts at most one clade per creature: cell {cell} at tick {}",
                sample.tick
            );
        }
    }

    // The horizon checkpoint equals the grid of the oracle's terminal
    // state — the state `run_one_seed` sampled on that tick, by T10.F11.
    let horizon_sample = run
        .persistence
        .samples
        .iter()
        .find(|sample| sample.tick == HORIZON)
        .expect("the horizon tick is sampled");
    assert!(horizon_sample.population > 0);
    assert_eq!(
        horizon_sample.occupancy_grid.as_ref(),
        Some(&OccupancyGrid::observe(&oracle))
    );
    assert!(
        horizon_sample
            .occupancy_grid
            .as_ref()
            .is_some_and(|grid| grid.population.iter().filter(|&&count| count > 0).count() > 1),
        "a living population spread over a world occupies more than one cell"
    );
}

pub(super) fn small_profile(name: &str) -> ProfileParams {
    ProfileParams {
        recipe: None,
        name: name.to_string(),
        width: 8,
        height: 8,
        founders: 4,
        seeds: vec![1],
        ticks: 2,
        food_coverage: Some(1.0),
        neighborhood: NeighborhoodSizes::default(),
        drift: Default::default(),
        recruitment: v3_core::neighborhood::recruitment_paths::Sizes::TEST,
    }
}

/// Each world-set case reports the same cumulative behavior a replay of
/// that case's own config and seed produces, and every fraction is derived
/// from those totals rather than measured separately.
#[test]
fn world_set_case_tracking_matches_a_replayed_run() {
    let mut params = goal_profile_params();
    params.width = 24;
    params.height = 24;
    params.founders = 16;
    params.ticks = 4;
    params.neighborhood = NeighborhoodSizes::default();
    params.drift = Default::default();
    params.recruitment = v3_core::neighborhood::recruitment_paths::Sizes::TEST;
    let (report, _) = run_deterministic(&params).expect("a valid profile");

    for (index, case) in report.goal_indicators.cases.iter().enumerate() {
        let (_, config) = goal_case(&params, &GOAL_RECIPES[index]);
        let mut sim = seed_simulation(config, case.case.seed);
        for _ in 0..params.ticks {
            run_tick(&mut sim, &mut None);
            if sim.creatures.is_empty() {
                break;
            }
        }
        let expected = WorldTracking::observe(&sim).with_transferred_counters(&sim);
        assert_eq!(case.tracking, expected, "case {}", case.case.name);
        assert_eq!(
            case.tracking.typed_eats_total.len(),
            case.case.food_type_count,
            "one eat counter per configured food type"
        );
        assert_eq!(
            case.tracking.food_density_total.len(),
            case.case.food_type_count
        );
        assert_eq!(
            case.tracking.grazing_modifier_mean.len(),
            case.case.food_type_count,
            "one grazing modifier reading per configured food type"
        );
        assert_eq!(
            case.tracking.grazed_cell_share.len(),
            case.case.food_type_count
        );
        assert_eq!(case.fractions, expected.fractions());

        // The transferred blocks come from the replayed run above (the
        // `expected` equality covers their values); only their shape is
        // not implied by it.
        let failed = case
            .tracking
            .typed_eats_failed_total
            .as_ref()
            .expect("a new report carries failed eats by type");
        assert_eq!(failed.len(), case.case.food_type_count);
        let flows = case.tracking.energy_flows.as_ref().expect("terminal flows");
        assert_eq!(flows.food_intake_by_type.len(), case.case.food_type_count);
        // The T19.F04 founder is all Graph, so its compute is charged there.
        assert!(flows.graph_compute.parse::<f64>().unwrap() > 0.0);
        assert!(flows.lifecycle_decay.parse::<f64>().unwrap() > 0.0);
        assert!(flows.genome_size_creature_ticks > 0);
        let deaths = case
            .tracking
            .mortality
            .as_ref()
            .expect("terminal mortality");
        assert_eq!(deaths.by_cause.values().sum::<u64>(), deaths.deaths_total);
        let reproduction = case
            .tracking
            .reproductive_success_by_cognitive_class
            .as_ref()
            .expect("terminal reproductive success");
        assert_eq!(reproduction.by_class.len(), 4);
        assert_eq!(
            reproduction
                .by_class
                .values()
                .map(|row| row.creatures_observed_total)
                .sum::<u64>(),
            deaths.deaths_total
        );
        let wire = serde_json::to_value(case).unwrap();
        assert_eq!(
            wire["reproductive_success_by_cognitive_class"],
            serde_json::to_value(reproduction).unwrap()
        );
    }
}

/// The world-set profile shrunk to test size: the same recipes, seeds, and
/// per-case machinery, on a world small enough to run in a test.
pub(super) fn small_world_set_params() -> ProfileParams {
    let mut params = goal_profile_params();
    params.width = 16;
    params.height = 16;
    params.founders = 4;
    params.ticks = 1;
    params.neighborhood = NeighborhoodSizes::default();
    params.drift = Default::default();
    params.recruitment = v3_core::neighborhood::recruitment_paths::Sizes::TEST;
    params
}

pub(super) fn small_world_set_report() -> Report {
    build_report(&small_world_set_params(), "t12-f04-world-set-check").expect("a valid profile")
}
