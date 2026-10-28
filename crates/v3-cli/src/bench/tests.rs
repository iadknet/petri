//! Tests that exercise items from several `bench` modules, and shared test helpers.

use super::comparison::compare_cases;
use super::profiles::{
    build_config, goal_case, goal_profile_params, NeighborhoodSizes, ProfileParams, GOAL_RECIPES,
};
use super::run::{build_report, run_deterministic, run_one_seed};
use super::schema::Report;
use super::tracking::{OccupancyGrid, SampleBlocks, SensorCensus, WorldTracking};
use v3_core::contracts::WorldInputKey;
use v3_core::neighborhood::occupancy::OCCUPANCY_CELLS_PER_AXIS;
use v3_core::neighborhood::occupancy::OCCUPANCY_CELL_COUNT;
use v3_core::neighborhood::sensor_census::world_input_key_label;
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

/// The 32x32, 8-founder sweep config the real-run-path tests share. It
/// asserts the world it built first, so a config that ignored the params
/// fails here at once instead of running a full-size world to the horizon.
fn small_sweep_config(seed: u64, horizon: u64) -> v3_core::config::SimulationConfig {
    let config = build_config(&ProfileParams {
        recipe: None,
        name: "sweep".to_string(),
        width: 32,
        height: 32,
        founders: 8,
        seeds: vec![seed],
        ticks: horizon,
        food_coverage: None,
        neighborhood: NeighborhoodSizes::default(),
        drift: Default::default(),
        mutation_effects: Default::default(),
    });
    assert_eq!(
        (
            config.world.width,
            config.world.height,
            config.population.initial_creatures
        ),
        (32, 32, 8)
    );
    config
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
    // recovered one whose readings differ from tick 100's. Seed 22 was
    // measured to survive the horizon: effort-cost defaults, 2026-10-10
    // (floor 0.4, attempt 1.0, move 1.6) sent seed 16 extinct at tick 150,
    // and only seeds 22 and 50 of 0..=60 survive (T11.F29 moved it from 13).
    // T11.F31 moved it from 22 to 14: the ring projection operators change
    // every Graph-domain draw and seed 22 goes extinct at tick 151; seeds
    // 14, 50, 67, 73, 78, 80 and 151 of 0..=200 survive. T11.F30 moved it
    // from 14 to 207: the heading tie sends seed 14 extinct at tick 194, no
    // seed of 0..=200 survives the horizon, and seeds 207, 281, 289, 499,
    // 597, 729, 730 and 1184 of 201..=1200 do. Moved from 207 to 6 on
    // 2026-10-12, when T20.F04 whole-family recruitment and T20.F05
    // refinement became defaults and seed 207 went extinct at tick 127;
    // seeds 6, 55, 208, 634, 729, 815, 982 and 1010 of 0..=1010 survive.
    // Moved from 6 to 19 on 2026-10-14 (T25.F02): the uniform size rule
    // moves the carrying cost and the mutation draw count, and seed 6 goes
    // extinct at tick 112; seed 19 is the lowest of 0..=19 that survives the
    // horizon (0 to 18 go extinct between ticks 112 and 187). The old size
    // rule and anchor 97 restore seed 6. Moved from 19 to 59 by T25.F02
    // commit 2 (default rate 0.005 -> 0.0060625): seed 19 goes extinct at
    // tick 159; seed 59 is the only one of 0..=60 that survives the horizon.
    // Rate 0.005 restores seed 19. Moved from 59 to 14 by T25.F04 commit 1:
    // `AddDirectionPairGroup` remaps every Graph draw and seed 59 goes extinct
    // before the horizon; 14 is the lowest of 0..=400 that survives it (14,
    // 42, 93, 158, 175 and 201 are the first six), and removing the catalog
    // entry restores seed 59. Moved from 14 to 129 by T25.F07 commit 2:
    // learned bundle creation, the scope coin and `SetScope` remap every Graph
    // draw and seed 14 goes extinct at tick 132; 129 is the lowest of 0..=400
    // that survives the horizon (129, 164 and 175 do). Moved from 129 to 73
    // by T25.F08 commit 1: deleting `RefineHeritableStructure` remaps every
    // Graph draw and seed 129 goes extinct; 73 is the lowest of 0..=400 that
    // survives the horizon (73, 147, 158, 197 and 201 do). Moved from 73 to
    // 129 by T25.F08 commit 2: the randomized creation operators and
    // `DetachGraphEdge` remap every Graph draw and seed 73 goes extinct; 129
    // is the only seed of 0..=400 that survives the horizon. Moved from 129
    // to 14 by T25.F09 commit 3: the six retention operators remap every
    // Graph draw and seed 129 goes extinct; 14 is the lowest of 0..=400 that
    // survives the horizon (14, 202, 268 and 358 do). Moved from 14 to 145
    // by T27.F01 commit 2: the 30-bucket input draw and the own-colour
    // `Swap` alternatives remap every mutation draw and seed 14 goes
    // extinct; of 0..=600, seeds 145, 175, 268, 319 and 513 survive the
    // horizon and 145 is the lowest. Moved from 145 to 831 by T26.F02: the
    // capacity chance drawn on every birth shifts every later draw and seed
    // 145 goes extinct at tick 149; of 0..=2000 seeds 52, 831, 1184 and 1871
    // survive the horizon, and 831 is the lowest that ends with more than
    // one creature, which the occupancy-grid test needs. Removing the
    // capacity draw restores seed 145. Moved from 831 to 14 by the T26.F02
    // sweep defaults (beta 1.0, u 0.002): seed 831 goes extinct at tick 126;
    // of 0..=2000 seeds 14, 15, 281, 295, 387, 865, 1296, 1300, 1570 and 1845
    // survive the horizon, and 14, the lowest, ends with three creatures.
    // Restoring (0.5, 0.0005) restores seed 831. Moved from 14 to 41 by
    // T27.F03 commit 3: the two share draws on every birth shift every later
    // draw and seed 14 goes extinct at tick 125; of 0..=2000 seeds 41, 199,
    // 272, 321, 351, 451, 509 and 519 are the first to survive the horizon
    // with more than one creature, and 41, the lowest, ends with two. Moved
    // from 41 to 199 by T27.F03 commit 4: the `GiveEnergy` kind widens the
    // kind, sink and field draws and seed 41 goes extinct at tick 123; of
    // 0..=2000 seeds 199, 414, 519, 782, 924, 953, 1232 and 1600 are the
    // first to survive the horizon with more than one creature, and 199, the
    // lowest, ends with three. Moved from 199 to 148 by T25.F11 commit 3:
    // the creation operators change every birth's draws and seed 199 goes
    // extinct at tick 160; of 0..=2000 seeds 148, 232, 417, 732, 1076, 1294
    // and 1621 survive the horizon with more than one creature, and 148, the
    // lowest, ends with seven.
    const SEED: u64 = 148;
    const HORIZON: u64 = 250;
    let config = small_sweep_config(SEED, HORIZON);

    // Act
    let run = run_one_seed(&config, SEED, HORIZON, false, None, None);
    let mut oracle = seed_simulation(config.clone(), SEED);
    for _ in 0..HORIZON {
        run_tick(&mut oracle, &mut None);
    }
    let terminal = SampleBlocks::observe(&oracle);

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

    assert_eq!(horizon_sample.blocks, terminal);

    // A checkpoint carries its own tick's readings, not the run's.
    let early_sample = run
        .persistence
        .samples
        .iter()
        .find(|sample| sample.tick == 100)
        .expect("tick 100 is sampled");
    assert_ne!(
        early_sample.blocks.mean_generation, horizon_sample.blocks.mean_generation,
        "generations advance between tick 100 and the horizon"
    );
    assert_ne!(
        early_sample.blocks.surviving_founder_clade_count,
        horizon_sample.blocks.surviving_founder_clade_count,
        "founder clades are lost between tick 100 and the horizon"
    );
}

#[test]
fn the_real_run_path_carries_a_census_of_the_whole_key_universe_at_every_checkpoint() {
    // Effort-cost defaults, 2026-10-10: seed 16 goes extinct at tick 150;
    // seed 22 survives the horizon. T11.F31: seed 22 goes extinct at tick
    // 151 and seed 14 survives (see the post-tick-state test). T11.F30: seed
    // 14 goes extinct at tick 194 and seed 207 survives. T25.F02: seed 6
    // goes extinct at tick 112 and seed 19 survives; at the T25.F02 rate
    // seed 19 goes extinct at tick 159 and seed 59 survives; T25.F04 commit
    // 1: seed 59 goes extinct and seed 14 survives (see the
    // post-tick-state test). T25.F07 commit 2: seed 14 goes extinct at tick
    // 132 and seed 129 survives (see the post-tick-state test). T25.F08
    // commit 1: seed 129 goes extinct and seed 73 survives (see the
    // post-tick-state test). T25.F08 commit 2: seed 73 goes extinct and seed
    // 129 survives (see the post-tick-state test). T25.F09 commit 3: seed
    // 129 goes extinct and seed 14 survives (see the post-tick-state test).
    // T27.F01 commit 2: seed 14 goes extinct and seed 145 survives (see the
    // post-tick-state test). T26.F02: 145 -> 831 -> 14 (see the
    // post-tick-state test). T27.F03 commit 3: 14 -> 41, commit 4: 41 ->
    // 199 (see the post-tick-state test). T25.F11 commit 3: 199 -> 148 (see
    // the post-tick-state test).
    const SEED: u64 = 148;
    const HORIZON: u64 = 250;
    let config = small_sweep_config(SEED, HORIZON);

    // Act
    let run = run_one_seed(&config, SEED, HORIZON, false, None, None);
    let mut oracle = seed_simulation(config.clone(), SEED);
    for _ in 0..HORIZON {
        run_tick(&mut oracle, &mut None);
    }

    // Assert: every checkpoint carries a census, and every census carries
    // the world's whole key universe in key order.
    let expected_keys: Vec<String> = WorldInputKey::universe(
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
            .blocks
            .sensor_census
            .as_ref()
            .expect("every checkpoint carries a census");
        let keys: Vec<String> = census
            .world_inputs
            .iter()
            .map(|row| row.key.clone())
            .collect();
        assert_eq!(keys, expected_keys, "tick {}", sample.tick);
        let decision_keys: Vec<&str> = census
            .decision_inputs
            .iter()
            .map(|row| row.key.as_str())
            .collect();
        assert_eq!(
            decision_keys,
            v3_core::genome::sensor_analysis::DecisionInputKey::ALL.map(|key| key.as_key()),
            "tick {}",
            sample.tick
        );
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
        horizon_sample.blocks.sensor_census.as_ref(),
        Some(&SensorCensus::observe(&oracle))
    );

    // The founder population's graph-backend references are visible.
    let first = run.persistence.samples[0]
        .blocks
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
    // Evolved-trajectory pin: the lowest seed in 0..=25 that keeps at least
    // 2 creatures over at least 2 cells at the horizon. Re-pinned by the
    // effort-cost defaults, 2026-10-10 (floor 0.4, attempt 1.0, move 1.6),
    // from seed 2 to 22: seed 2 goes extinct at tick 153; seed 22 keeps 6
    // creatures over 6 cells (22 of seeds 0..=22 go extinct). Re-pinned by T16.F02
    // from seed 0 to 2: the effort prices move every trajectory, and seed 0
    // goes extinct at tick 240; seed 2 keeps 3 creatures over 3 cells (6 of
    // seeds 0..=25 go extinct and 10 end on one cell). Re-pinned by T11.F29
    // from seed 1 to 0: the `AddFeedbackRoute` entry remaps every topology
    // draw, and seed 1 goes extinct at tick 191; seed 0 keeps 3 creatures
    // over 2 cells (6 of seeds 0..=25 go extinct and 9 end on one cell).
    // Re-pinned by T20.F16
    // from seed 0 to 1: Graph-node `Add` draws three more numbers and appends
    // a vote edge, and seed 0 goes extinct at tick 210; seed 1 keeps 3
    // creatures over 3 cells (7 of seeds 0..=25 go extinct and 3 end on one
    // cell). T24.F02 had moved it from seed 2 to 0, T24.F01 from 1 to 2,
    // T19.F04 from 3 to 1 (its vote founder sent seed 3 extinct), T11.F23
    // from 1 to 3, T16.F01 from 18 to 1, T11.F21 from 13 to 18. T11.F31
    // moved it from 22 to 14: seed 22 goes extinct at tick 151. T11.F30 moved
    // it from 14 to 207: seed 14 goes extinct at tick 194; seed 207 keeps 5
    // creatures over 5 cells. T25.F02 moved it from 6 to 19: seed 6 goes
    // extinct at tick 112, and seed 19 is the lowest of 0..=19 that survives
    // the horizon over more than one cell; the old size rule and anchor 97
    // restore seed 6. The T25.F02 rate moved it from 19 to 59: seed 19 goes
    // extinct at tick 159, and seed 59 is the only one of 0..=60 that
    // survives the horizon; rate 0.005 restores seed 19. T25.F04 commit 1
    // moved it from 59 to 14: seed 59 goes extinct before the horizon, and
    // seed 14 is the lowest of 0..=400 that survives it. T25.F07 commit 2
    // moved it from 14 to 164: seed 14 goes extinct at tick 132; of the
    // seeds of 0..=400 that survive the horizon (129, 164, 175), 129 ends on
    // one cell and 164 is the lowest over more than one. T25.F08 commit 1
    // moved it from 164 to 73: of the seeds of 0..=400 that survive the
    // horizon (73, 147, 158, 197, 201), 73 is the lowest and ends over three
    // cells. T25.F08 commit 2 moved it from 73 to 473: seed 73 goes extinct;
    // 129, the only seed of 0..=400 that survives the horizon, ends on one
    // occupancy cell, and 473, the first survivor above 400, ends over three.
    // T25.F09 commit 3 moved it from 473 to 14: the six retention operators
    // remap every Graph draw and seed 473 goes extinct; of the seeds of
    // 0..=600 that survive the horizon (14, 202, 268, 358, 423, 430, 510, 531,
    // 558), 14 is the lowest and ends over five cells. T27.F01 commit 2
    // moved it from 14 to 145: seed 14 goes extinct; of the seeds of 0..=600
    // that survive the horizon (145, 175, 268, 319, 513), 145 is the lowest
    // and ends over four cells. T26.F02 moved it 145 -> 831 -> 14 (see the
    // post-tick-state test). T27.F03 commit 3 moved it 14 -> 41 and commit 4
    // 41 -> 199 (see the post-tick-state test). T25.F11 commit 3 moved it
    // 199 -> 148 (see the post-tick-state test).
    const SEED: u64 = 148;
    const HORIZON: u64 = 250;
    let config = small_sweep_config(SEED, HORIZON);

    // Act
    let run = run_one_seed(&config, SEED, HORIZON, false, None, None);
    let mut oracle = seed_simulation(config.clone(), SEED);
    for _ in 0..HORIZON {
        run_tick(&mut oracle, &mut None);
    }

    assert!(!run.persistence.samples.is_empty());
    for sample in &run.persistence.samples {
        let grid = sample
            .blocks
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
        horizon_sample.blocks.occupancy_grid.as_ref(),
        Some(&OccupancyGrid::observe(&oracle))
    );
    assert!(
        horizon_sample
            .blocks
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
        mutation_effects: Default::default(),
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
    params.mutation_effects = Default::default();
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
    params.mutation_effects = Default::default();
    params
}

pub(super) fn small_world_set_report() -> Report {
    build_report(&small_world_set_params(), "t12-f04-world-set-check").expect("a valid profile")
}
