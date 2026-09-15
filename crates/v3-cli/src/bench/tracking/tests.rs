use super::*;
use crate::bench::indicators::lineage_diversity;
use crate::bench::profiles::build_config;
use crate::bench::tests::small_profile;
use crate::UNDEFINED;
use proptest::prelude::*;
use v3_core::config::SimulationConfig;
use v3_core::contracts::Position;
use v3_core::kernel::occupancy_grid::OCCUPANCY_CELL_COUNT;
use v3_core::simulation::seed_simulation;

#[test]
fn cognition_tracking_transfers_every_terminal_counter_and_preserves_absence() {
    let mut sim = seed_simulation(SimulationConfig::default(), 7);
    sim.stats.plasticity_updates_total = 19;
    sim.stats.plasticity_changes_total = 7;
    sim.stats.hebbian_updates_total = 8;
    sim.stats.hebbian_changes_total = 2;
    sim.stats.reward_modulated_updates_total = 11;
    sim.stats.reward_modulated_changes_total = 5;
    sim.stats.shared_memory_writes_changed_total = 23;
    let terminal = WorldTracking::observe(&sim).with_transferred_counters(&sim);
    let value = serde_json::to_value(&terminal).unwrap();
    assert_eq!(
        value["cognition"],
        serde_json::json!({
            "plasticity_updates_total": 19, "plasticity_changes_total": 7,
            "hebbian_updates_total": 8, "hebbian_changes_total": 2,
            "reward_modulated_updates_total": 11, "reward_modulated_changes_total": 5,
            "shared_memory_writes_changed_total": 23,
        })
    );
    let round_trip: WorldTracking = serde_json::from_value(value).unwrap();
    assert_eq!(terminal, round_trip);
    assert!(WorldTracking::observe(&sim).cognition.is_none());
    let historical: WorldTracking = serde_json::from_str("{}").unwrap();
    assert!(historical.cognition.is_none());
    assert!(serde_json::to_value(historical)
        .unwrap()
        .get("cognition")
        .is_none());
}

/// A stand-in census of a one-food-type world: the whole key universe is
/// present, `FoodHere:0` is referenced by every living creature, and the
/// stateful counts are a fixed share of the population. An empty
/// population makes every row a true `0`, exactly as
/// `SensorCensus::observe` does.
fn census_for(population: u64) -> SensorCensus {
    use v3_core::config::OrdinaryFoodTypeId;
    let food_here = WorldInputKey::food_here(OrdinaryFoodTypeId::new(0));
    SensorCensus {
        world_inputs: world_input_key_universe([OrdinaryFoodTypeId::new(0)])
            .into_iter()
            .map(|key| WorldInputCensusRow {
                key: world_input_key_label(key),
                creatures: if key == food_here { population } else { 0 },
            })
            .collect(),
        creatures_reading_shared_memory: population / 2,
        creatures_with_stateful_node: population / 4,
        creatures_with_any_stateful_read: population / 2,
    }
}

/// A stand-in occupancy grid: the whole `population` standing in the first
/// cell as one clade, every other cell a true zero. An empty population
/// makes the whole grid zero, exactly as `OccupancyGrid::observe` does.
fn grid_for(population: u64) -> OccupancyGrid {
    let mut population_cells = vec![0; OCCUPANCY_CELL_COUNT];
    let mut clade_cells = vec![0; OCCUPANCY_CELL_COUNT];
    population_cells[0] = population;
    clade_cells[0] = u64::from(population > 0);
    OccupancyGrid {
        cells_x: OCCUPANCY_CELLS_PER_AXIS,
        cells_y: OCCUPANCY_CELLS_PER_AXIS,
        population: population_cells,
        distinct_clades: clade_cells,
    }
}

/// The reading `PopulationReadings::observe` produces for an empty
/// population: zero means that never reach the report, no surviving clade,
/// undefined entropy, and an all-zero census.
fn empty_readings() -> PopulationReadings {
    PopulationReadings {
        mean_genome_size: 0.0,
        mean_mesh_nodes: 0.0,
        mean_generation: 0.0,
        surviving_founder_clade_count: 0,
        shannon_entropy_nats: UNDEFINED.to_string(),
        sensor_census: census_for(0),
        occupancy_grid: grid_for(0),
    }
}

/// Population readings a living tick stands in with: values keyed to the
/// tick so a sample can be traced back to the tick it was taken on. An
/// empty population reads [`empty_readings`], exactly as
/// `PopulationReadings::observe` does.
fn readings_for(tick: u64, population: u64) -> PopulationReadings {
    if population == 0 {
        return empty_readings();
    }
    PopulationReadings {
        mean_genome_size: tick as f64,
        mean_mesh_nodes: tick as f64 * 2.0,
        mean_generation: tick as f64 * 3.0,
        surviving_founder_clade_count: population,
        shannon_entropy_nats: six(tick as f64 / 4.0),
        sensor_census: census_for(population),
        occupancy_grid: grid_for(population),
    }
}

/// Feed the accumulator one observation per tick from a population
/// series (index 0 is tick 1), with a constant mean creature energy, a
/// cumulative birth count equal to the tick, and [`readings_for`].
fn observe_series(
    horizon: u64,
    seeded_population: u64,
    populations: &[u64],
    energy: f64,
) -> PopulationPersistenceSeed {
    let mut accumulator = PersistenceAccumulator::new(horizon, seeded_population);
    for (index, &population) in populations.iter().enumerate() {
        let tick = index as u64 + 1;
        accumulator.observe(
            tick,
            population,
            tick,
            || energy,
            || readings_for(tick, population),
            WorldTracking::default,
        );
    }
    accumulator.finish(7)
}

#[test]
fn peak_records_the_maximum_population_and_the_first_tick_reaching_it() {
    let summary = observe_series(8, 10, &[12, 20, 15, 20, 18, 9, 9, 9], 1.0);

    assert_eq!(summary.peak_population, 20);
    assert_eq!(summary.peak_tick, 2, "the first tick at the peak wins");
    assert_eq!(summary.minimum_population, 9);
}

#[test]
fn seeding_population_is_the_peak_when_no_tick_exceeds_it() {
    let summary = observe_series(4, 50, &[40, 30, 20, 10], 1.0);

    assert_eq!(summary.peak_population, 50);
    assert_eq!(summary.peak_tick, 0, "tick 0 is the founder population");
}

#[test]
fn plateau_is_null_when_the_run_goes_extinct_before_the_window() {
    // Horizon 100: the plateau window is ticks 76..=100. Extinction at
    // tick 3 means no observation ever lands in the window.
    let summary = observe_series(100, 4, &[3, 1, 0], 1.0);

    assert_eq!(summary.extinction_tick, Some(3));
    assert_eq!(summary.plateau_population, None);
    assert_eq!(summary.final_population, 0);
}

#[test]
fn plateau_averages_only_the_ticks_executed_inside_a_partial_window() {
    // Horizon 8: the window is the ticks strictly after 6, so 7 and 8.
    // The run goes extinct at tick 8, so the window holds 10 and 0.
    let summary = observe_series(8, 4, &[4, 4, 4, 4, 4, 4, 10, 0], 1.0);

    assert_eq!(summary.extinction_tick, Some(8));
    assert_eq!(summary.plateau_population.as_deref(), Some("5.000000"));
}

#[test]
fn mean_energy_is_null_at_extinction_and_the_final_value_otherwise() {
    let extinct = observe_series(4, 4, &[4, 2, 0], 2.5);
    assert_eq!(extinct.mean_energy, None);
    assert_eq!(
        extinct
            .samples
            .last()
            .expect("the extinction tick is sampled")
            .mean_energy,
        None
    );

    let survived = observe_series(4, 4, &[4, 4, 4, 4], 2.5);
    assert_eq!(survived.mean_energy.as_deref(), Some("2.500000"));
}

#[test]
fn every_checkpoint_carries_the_population_readings_of_its_own_tick() {
    let populations = [6_u64; 250];
    let summary = observe_series(250, 6, &populations, 1.0);

    assert_eq!(
        summary.samples.iter().map(|s| s.tick).collect::<Vec<_>>(),
        vec![100, 200, 250]
    );
    for sample in &summary.samples {
        let expected = readings_for(sample.tick, sample.population);
        assert_eq!(
            sample.mean_genome_size.as_deref(),
            Some(six(expected.mean_genome_size).as_str()),
            "tick {}",
            sample.tick
        );
        assert_eq!(
            sample.mean_mesh_nodes.as_deref(),
            Some(six(expected.mean_mesh_nodes).as_str())
        );
        assert_eq!(
            sample.mean_generation.as_deref(),
            Some(six(expected.mean_generation).as_str())
        );
        assert_eq!(sample.surviving_founder_clade_count, Some(6));
        assert_eq!(
            sample.shannon_entropy_nats.as_deref(),
            Some(expected.shannon_entropy_nats.as_str())
        );
    }
}

/// The census reaches every checkpoint of a real `run_one_seed` loop, and
/// the horizon checkpoint equals a census computed independently from an
/// oracle re-run of the same seed. The founder population is Graph-backend
/// and wires its world inputs to output sinks, so a VM-only or
/// compute-node-only census would report `FoodHere:0` as zero here.
/// Each stateful split sums one per qualifying creature over the whole
/// population, separately from the other two. A real run reaches the
/// shared-memory split but carries no stateful compute node, so the
/// population here is built to hold one creature of each kind.
#[test]
fn each_stateful_split_counts_the_creatures_that_qualify_for_it() {
    use v3_core::contracts::NodeId;
    use v3_core::creature::genome::cgp::{
        CgpGraphBackendDef, ComputeNode, ComputeNodeKind, ExecuteGate, GraphEdge, GraphSource,
        OutputSink, OutputSinkKind,
    };
    use v3_core::creature::genome::{BackendDef, CreatureGenome, NodeGenome};

    // Arrange: three creatures — one reads a shared-memory slot, one holds
    // a live stateful compute node, one reads neither.
    fn graph_genome(graph: CgpGraphBackendDef) -> CreatureGenome {
        CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![],
                targets: vec![],
                backend_def: BackendDef::Graph(graph),
            }],
        }
    }
    fn sink(source: GraphSource) -> OutputSink {
        OutputSink {
            kind: OutputSinkKind::CustomOutput(0),
            inputs: vec![GraphEdge {
                source,
                weight: 1.0,
            }],
        }
    }
    let inert = CgpGraphBackendDef {
        birth_weights: None,
        compute_nodes: vec![ComputeNode {
            kind: ComputeNodeKind::Constant(0.0),
            inputs: vec![],
            plasticity: None,
        }],
        output_sinks: vec![sink(GraphSource::ComputeNode(0))],
        action_bank: vec![],
        execute_gate: ExecuteGate { inputs: vec![] },
    };
    let mut shared_memory_reader = inert.clone();
    shared_memory_reader.output_sinks = vec![sink(GraphSource::SharedMemory {
        slot: 0,
        previous: false,
    })];
    let mut stateful_node_holder = inert.clone();
    stateful_node_holder.compute_nodes[0].kind = ComputeNodeKind::DecayIntegrator(0.5);

    let mut config = SimulationConfig::default();
    config.population.initial_creatures = 3;
    let mut sim = seed_simulation(config, 11);
    let ids: Vec<_> = sim.creatures.keys().collect();
    assert_eq!(ids.len(), 3);
    for (id, graph) in ids
        .iter()
        .zip([shared_memory_reader, stateful_node_holder, inert])
    {
        sim.creatures[*id].genome = graph_genome(graph);
        sim.creatures[*id].cached_reachable_nodes = vec![0].into_boxed_slice();
    }

    // Act
    let census = SensorCensus::observe(&sim);

    // Assert: one creature per split, and the combined count is the union.
    assert_eq!(census.creatures_reading_shared_memory, 1);
    assert_eq!(census.creatures_with_stateful_node, 1);
    assert_eq!(census.creatures_with_any_stateful_read, 2);
}

/// At extinction the three means and the entropy report absence, never
/// zero; the clade count is a true `0`.
#[test]
fn the_extinction_checkpoint_reports_absent_means_and_a_zero_clade_count() {
    let extinct = observe_series(4, 4, &[4, 2, 0], 2.5);
    let last = extinct
        .samples
        .last()
        .expect("the extinction tick is sampled");

    assert_eq!(last.population, 0);
    assert_eq!(last.mean_genome_size, None);
    assert_eq!(last.mean_mesh_nodes, None);
    assert_eq!(last.mean_generation, None);
    assert_eq!(last.surviving_founder_clade_count, Some(0));
    assert_eq!(last.shannon_entropy_nats.as_deref(), Some(UNDEFINED));

    let census = last
        .sensor_census
        .as_ref()
        .expect("a census of an empty population is zero, not unmeasured");
    assert!(
        census.world_inputs.iter().all(|row| row.creatures == 0),
        "every key universe row is a true zero: {census:?}"
    );
    assert!(!census.world_inputs.is_empty());
    assert_eq!(census.creatures_reading_shared_memory, 0);
    assert_eq!(census.creatures_with_stateful_node, 0);
    assert_eq!(census.creatures_with_any_stateful_read, 0);

    let grid = last
        .occupancy_grid
        .as_ref()
        .expect("the grid of an empty population is zero, not unmeasured");
    assert_eq!(grid.cells_x, OCCUPANCY_CELLS_PER_AXIS);
    assert_eq!(grid.cells_y, OCCUPANCY_CELLS_PER_AXIS);
    assert_eq!(grid.population.len(), OCCUPANCY_CELL_COUNT);
    assert_eq!(grid.distinct_clades.len(), OCCUPANCY_CELL_COUNT);
    assert!(grid.population.iter().all(|&count| count == 0));
    assert!(grid.distinct_clades.iter().all(|&count| count == 0));
}

/// `OccupancyGrid::observe` reads the living population's positions and
/// lineage ids against the world's own extent: two clades in different
/// parts of the world occupy different cells, and two clades in one part
/// share a cell.
#[test]
fn the_observed_grid_separates_clades_by_where_they_stand() {
    let mut config = SimulationConfig::default();
    config.world.width = 160;
    config.world.height = 160;
    config.population.initial_creatures = 4;
    let mut sim = seed_simulation(config, 42);
    let ids: Vec<_> = sim.creatures.keys().collect();
    assert_eq!(ids.len(), 4);

    // Two clades in the top-left cell, and the same two clades again in
    // the cell one row down and one column right.
    for (id, (position, lineage_id)) in ids.iter().zip([
        (Position::new(0, 0), 1),
        (Position::new(9, 9), 2),
        (Position::new(10, 10), 1),
        (Position::new(19, 19), 2),
    ]) {
        sim.creatures[*id].position = position;
        sim.creatures[*id].identity.lineage_id = lineage_id;
    }

    let grid = OccupancyGrid::observe(&sim);

    assert_eq!(grid.cells_x, OCCUPANCY_CELLS_PER_AXIS);
    assert_eq!(grid.cells_y, OCCUPANCY_CELLS_PER_AXIS);
    assert_eq!(grid.population[0], 2);
    assert_eq!(grid.distinct_clades[0], 2);
    // Row-major: the cell at (1, 1) is index 17.
    assert_eq!(grid.population[17], 2);
    assert_eq!(grid.distinct_clades[17], 2);
    assert_eq!(grid.population.iter().sum::<u64>(), 4);
    assert_eq!(grid.distinct_clades.iter().sum::<u64>(), 4);
}

#[test]
fn population_readings_average_the_living_population_and_count_its_clades() {
    let mut config = SimulationConfig::default();
    config.world.width = 32;
    config.world.height = 32;
    config.population.initial_creatures = 4;
    let sim = seed_simulation(config, 42);
    assert_eq!(sim.creatures.len(), 4);

    let expected_genome_size = f64::from(
        sim.creatures
            .values()
            .map(|c| c.cached_genome_size)
            .sum::<u32>(),
    ) / 4.0;
    let readings = PopulationReadings::observe(&sim);
    assert_eq!(readings.mean_genome_size, expected_genome_size);
    assert_eq!(readings.mean_mesh_nodes, 2.0);
    assert_eq!(readings.mean_generation, 0.0);
    // Every founder is its own clade, so the distribution is uniform and
    // the entropy is ln(4).
    assert_eq!(readings.surviving_founder_clade_count, 4);
    assert_eq!(readings.shannon_entropy_nats, six(4.0_f64.ln()));
    assert_eq!(
        readings.shannon_entropy_nats,
        lineage_diversity(42, sim.creatures.values().map(|c| c.identity.lineage_id))
            .shannon_entropy_nats,
        "the checkpoint reading and the terminal reading share one computation"
    );
}

proptest! {
    /// The optional readings survive a JSON round trip beside the
    /// flattened tracking block, and a wire form without them reads them
    /// as absent rather than as zero.
    #[test]
    fn persistence_sample_readings_survive_a_json_round_trip(
        mean_genome_size in proptest::option::of(0.0f64..1e6),
        mean_mesh_nodes in proptest::option::of(0.0f64..1e6),
        mean_generation in proptest::option::of(0.0f64..1e6),
        surviving_founder_clade_count in proptest::option::of(0u64..10_000),
        shannon_entropy_nats in proptest::option::of(0.0f64..20.0),
        population in 0u64..1000,
    ) {
        let sample = PersistenceSample {
            tick: 100,
            population,
            mean_energy: None,
            births_total: 7,
            mean_genome_size: mean_genome_size.map(six),
            mean_mesh_nodes: mean_mesh_nodes.map(six),
            mean_generation: mean_generation.map(six),
            surviving_founder_clade_count,
            shannon_entropy_nats: shannon_entropy_nats.map(six),
            sensor_census: Some(census_for(population)),
            occupancy_grid: Some(grid_for(population)),
            tracking: WorldTracking::default(),
        };

        let wire = serde_json::to_string(&sample).expect("serializable");
        let decoded: PersistenceSample =
            serde_json::from_str(&wire).expect("deserializable");
        prop_assert_eq!(decoded.population, sample.population);
        prop_assert_eq!(&decoded.mean_genome_size, &sample.mean_genome_size);
        prop_assert_eq!(&decoded.mean_mesh_nodes, &sample.mean_mesh_nodes);
        prop_assert_eq!(&decoded.mean_generation, &sample.mean_generation);
        prop_assert_eq!(
            decoded.surviving_founder_clade_count,
            sample.surviving_founder_clade_count
        );
        prop_assert_eq!(&decoded.shannon_entropy_nats, &sample.shannon_entropy_nats);
        prop_assert_eq!(&decoded.sensor_census, &sample.sensor_census);
        prop_assert_eq!(&decoded.occupancy_grid, &sample.occupancy_grid);
        prop_assert_eq!(&decoded.tracking, &sample.tracking);

        let mut stripped: serde_json::Value =
            serde_json::from_str(&wire).expect("an object on the wire");
        let object = stripped.as_object_mut().expect("an object on the wire");
        for key in [
            "mean_genome_size",
            "mean_mesh_nodes",
            "mean_generation",
            "surviving_founder_clade_count",
            "shannon_entropy_nats",
            "sensor_census",
            "occupancy_grid",
        ] {
            object.remove(key);
        }
        let historical: PersistenceSample =
            serde_json::from_value(stripped).expect("a historical sample must parse");
        prop_assert_eq!(historical.mean_genome_size, None);
        prop_assert_eq!(historical.mean_mesh_nodes, None);
        prop_assert_eq!(historical.mean_generation, None);
        prop_assert_eq!(historical.surviving_founder_clade_count, None);
        prop_assert_eq!(historical.shannon_entropy_nats, None);
        prop_assert_eq!(historical.sensor_census, None);
        prop_assert_eq!(historical.occupancy_grid, None);
    }
}

#[test]
fn population_readings_of_an_empty_population_are_zero_means_and_undefined_entropy() {
    let mut config = SimulationConfig::default();
    config.world.width = 32;
    config.world.height = 32;
    config.population.initial_creatures = 0;
    let sim = seed_simulation(config, 42);
    assert_eq!(sim.creatures.len(), 0);

    assert_eq!(PopulationReadings::observe(&sim), empty_readings());
}

#[test]
fn samples_cover_every_hundredth_tick_plus_a_deduplicated_final_tick() {
    let populations = [5_u64; 250];
    let summary = observe_series(250, 5, &populations, 1.0);

    let ticks: Vec<u64> = summary.samples.iter().map(|s| s.tick).collect();
    assert_eq!(ticks, vec![100, 200, 250], "tick 0 is never sampled");
    assert_eq!(summary.samples[0].births_total, 100);

    // A horizon that is itself a multiple of the cadence yields one
    // sample for the final tick, not two.
    let exact = observe_series(200, 5, &populations[..200], 1.0);
    let exact_ticks: Vec<u64> = exact.samples.iter().map(|s| s.tick).collect();
    assert_eq!(exact_ticks, vec![100, 200]);
}

#[test]
fn a_horizon_with_no_executed_ticks_reports_the_seeded_population() {
    let summary = observe_series(0, 6, &[], 1.0);

    assert_eq!(summary.peak_population, 6);
    assert_eq!(summary.final_population, 6);
    assert_eq!(summary.plateau_population, None);
    assert_eq!(summary.mean_energy, None);
    assert!(summary.samples.is_empty());
}

#[test]
fn reproductive_success_report_transfers_all_twelve_integers_and_stable_keys() {
    use v3_core::simulation::reproductive_success::ReproductiveSuccessTotals;
    let mut sim = seed_simulation(SimulationConfig::default(), 7);
    let rows = [
        (1, 2, 3),
        (4, 5, 6),
        (7, 8, 9),
        (10, 11, 9_007_199_254_740_993),
    ];
    for (destination, (creatures, offspring, age)) in sim
        .stats
        .reproductive_success_by_cognitive_class
        .by_class
        .iter_mut()
        .zip(rows)
    {
        *destination = ReproductiveSuccessTotals {
            creatures_observed_total: creatures,
            offspring_spawned_sum: offspring,
            survival_ticks_sum: age,
        };
    }
    let terminal = WorldTracking::observe(&sim).with_transferred_counters(&sim);
    let block = terminal
        .reproductive_success_by_cognitive_class
        .as_ref()
        .unwrap();
    assert_eq!(
        block.definition,
        "reproductive-success-by-cognitive-class-v1"
    );
    assert_eq!(
        block
            .by_class
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["none", "plasticity", "shared_memory", "stateful"]
    );
    let wire = serde_json::to_value(block).unwrap();
    assert_eq!(wire.as_object().unwrap().len(), 2);
    for (key, (creatures, offspring, age)) in ["plasticity", "stateful", "shared_memory", "none"]
        .into_iter()
        .zip(rows)
    {
        assert_eq!(
            wire["by_class"][key],
            serde_json::json!({
                "creatures_observed_total": creatures,
                "offspring_spawned_sum": offspring,
                "survival_ticks_sum": age,
            })
        );
    }
    let round_trip: WorldTracking =
        serde_json::from_value(serde_json::to_value(&terminal).unwrap()).unwrap();
    assert_eq!(round_trip, terminal);
}

#[test]
fn reproductive_success_report_keeps_all_empty_cohorts_as_integer_zeros() {
    let sim = seed_simulation(SimulationConfig::default(), 7);
    let terminal = WorldTracking::observe(&sim).with_transferred_counters(&sim);
    let block = terminal.reproductive_success_by_cognitive_class.unwrap();
    let wire = serde_json::to_value(block).unwrap();
    let rows = wire["by_class"].as_object().unwrap();
    assert_eq!(rows.len(), 4);
    for key in ["none", "plasticity", "shared_memory", "stateful"] {
        assert_eq!(
            rows[key],
            serde_json::json!({
                "creatures_observed_total": 0,
                "offspring_spawned_sum": 0,
                "survival_ticks_sum": 0,
            })
        );
    }
}

/// Every transferred block is the `SimStats` counter behind it, read
/// field for field from a simulation whose counters are set by hand: the
/// replay test above exercises a short run, where most of these are zero.
#[test]
fn transferred_tracking_blocks_read_the_stats_counters_behind_them() {
    use v3_core::config::OrdinaryFoodTypeId;
    use v3_core::mutation::MutationOperator;
    use v3_core::simulation::actions::PredationActionResult;
    use v3_core::simulation::seed_simulation;
    use v3_core::simulation::stats::MutationValueTotals;

    let mut sim = seed_simulation(SimulationConfig::default(), 7);
    let stats = &mut sim.stats;
    stats.mutation_events_attempted_total = 40;
    stats.mutation_events_applied_total = 31;
    stats.mutation_events_skipped_total = 9;
    stats.mutation_executed_target_total = 6;
    stats.mutation_reachable_target_total = 17;
    stats.mutation_unreachable_target_total = 11;
    stats.mutation_not_applicable_target_total = 3;
    stats.mutation_outcome_summary = MutationValueTotals {
        carriers_observed_total: 5,
        survival_ticks_sum: 120,
        offspring_spawned_sum: 4,
        // Excluded from the report: zero on every benchmark path.
        final_energy_sum: 9.5,
        ..MutationValueTotals::default()
    };
    stats.mutation_value_totals_by_operator.insert(
        MutationOperator::TopologyAddNode,
        MutationValueTotals {
            carriers_observed_total: 2,
            invalid_action_total: 1,
            ..MutationValueTotals::default()
        },
    );
    stats.predation_actions_attempted_total = 8;
    stats.predation_actions_transferred_total = 5;
    stats.predation_actions_rejected_total = 3;
    stats.predation_kills_total = 2;
    stats
        .predation_actions_by_result
        .insert(PredationActionResult::TransferredAndKilled, 2);
    stats.mesh_dispatches_energy_exhausted_total = 14;
    stats
        .eat_actions_failed_total_by_type
        .insert(OrdinaryFoodTypeId::default(), 6);

    let tracking = WorldTracking::observe(&sim).with_transferred_counters(&sim);

    assert_eq!(
        tracking.mutation_supply,
        Some(MutationSupply {
            events_attempted_total: 40,
            events_applied_total: 31,
            events_skipped_total: 9,
            executed_target_total: 6,
            reachable_target_total: 17,
            unreachable_target_total: 11,
            not_applicable_target_total: 3,
        })
    );
    assert_eq!(
        tracking.mutation_outcome_summary,
        Some(MutationOutcomeTotals {
            carriers_observed_total: 5,
            survival_ticks_sum: 120,
            offspring_spawned_sum: 4,
            ..MutationOutcomeTotals::default()
        })
    );
    assert_eq!(
        tracking.mutation_value_totals_by_operator,
        Some(BTreeMap::from([(
            "Topology.AddNode".to_string(),
            MutationOutcomeTotals {
                carriers_observed_total: 2,
                invalid_action_total: 1,
                ..MutationOutcomeTotals::default()
            },
        )]))
    );
    assert_eq!(
        tracking.predation,
        Some(PredationTracking {
            actions_attempted_total: 8,
            actions_transferred_total: 5,
            actions_rejected_total: 3,
            kills_total: 2,
            actions_by_result: BTreeMap::from([("TransferredAndKilled".to_string(), 2)]),
        })
    );
    assert_eq!(tracking.mesh_dispatches_energy_exhausted_total, Some(14));
    assert_eq!(tracking.typed_eats_failed_total, Some(vec![6]));
}

/// A checkpoint sample carries exactly the keys it carried before T14.F02:
/// the transferred blocks are not merely null there, they are absent, and
/// only the end-of-run per-case block writes them.
#[test]
fn checkpoint_tracking_omits_every_transferred_block() {
    use v3_core::simulation::seed_simulation;

    const TRANSFERRED_KEYS: [&str; 10] = [
        "typed_eats_failed_total",
        "mesh_dispatches_energy_exhausted_total",
        "mutation_supply",
        "mutation_outcome_summary",
        "mutation_value_totals_by_operator",
        "predation",
        "mortality",
        "energy_flows",
        "cognition",
        "reproductive_success_by_cognitive_class",
    ];

    let sim = seed_simulation(SimulationConfig::default(), 7);
    let checkpoint = serde_json::to_value(WorldTracking::observe(&sim)).expect("serializable");
    let end_of_run =
        serde_json::to_value(WorldTracking::observe(&sim).with_transferred_counters(&sim))
            .expect("serializable");

    for key in TRANSFERRED_KEYS {
        assert!(
            checkpoint.get(key).is_none(),
            "checkpoint samples must not carry {key}"
        );
        assert!(
            end_of_run.get(key).is_some_and(|value| !value.is_null()),
            "the end-of-run block must carry {key}"
        );
    }
    assert_eq!(
        end_of_run.as_object().expect("an object").len(),
        checkpoint.as_object().expect("an object").len() + TRANSFERRED_KEYS.len(),
        "the two shapes differ only by the transferred blocks"
    );
}

#[test]
fn applied_accounting_report_copies_every_source_field_and_stable_cause_key() {
    use v3_core::simulation::energy_accounting::{ActionCharges, DeathCause, EnergyFlows};
    let mut sim = seed_simulation(SimulationConfig::default(), 7);
    sim.stats.mortality.record(DeathCause::RewardLearning);
    sim.stats.energy_flows = EnergyFlows {
        food_intake_by_type: vec![0.123456789, -2.0, 0.0],
        action_charges: ActionCharges {
            noop: 1.0,
            eat: 2.0,
            r#move: 3.0,
            reproduce: 4.0,
            steal_energy: 5.0,
        },
        failed_action_penalty: 6.0,
        vm_compute: 7.0,
        priority_bid: 8.0,
        graph_compute: 9.0,
        hebbian_learning: 10.0,
        reward_learning: 11.0,
        lifecycle_decay: 12.0,
        genome_carrying: 13.0,
        genome_size_creature_ticks: 14,
        parental_transfer_debit: 15.0,
        offspring_energy_credit: 16.0,
        predation_victim_debit: -17.0,
        predation_attacker_credit: -18.0,
        predation_kill_bonus_credit: 19.0,
        maximum_energy_clamp_loss: 20.0,
        zero_floor_credit: 21.0,
        external_removal_loss: 22.0,
    };
    let tracking = WorldTracking::observe(&sim).with_transferred_counters(&sim);
    let mortality = tracking.mortality.unwrap();
    assert_eq!(mortality.definition, "applied-mortality-v1");
    assert_eq!(mortality.deaths_total, 1);
    let expected_keys = [
        "lifecycle_decay",
        "genome_carrying",
        "vm_compute",
        "graph_compute",
        "hebbian_learning",
        "reward_learning",
        "priority_bid",
        "action_noop",
        "action_eat",
        "action_move",
        "action_reproduce",
        "action_steal_energy",
        "failed_action_penalty",
        "parental_transfer",
        "predation",
        "external_removal",
        "unattributed",
    ];
    assert_eq!(mortality.by_cause.len(), expected_keys.len());
    for key in expected_keys {
        assert_eq!(mortality.by_cause[key], u64::from(key == "reward_learning"));
    }
    assert_eq!(
        serde_json::to_value(tracking.energy_flows.unwrap()).unwrap(),
        serde_json::json!({
            "definition": "applied-energy-flows-v1",
            "food_intake_by_type": ["0.123457", "-2.000000", "0.000000"],
            "action_charges": {"noop": "1.000000", "eat": "2.000000", "move": "3.000000", "reproduce": "4.000000", "steal_energy": "5.000000"},
            "failed_action_penalty": "6.000000", "vm_compute": "7.000000", "priority_bid": "8.000000",
            "graph_compute": "9.000000", "hebbian_learning": "10.000000", "reward_learning": "11.000000",
            "lifecycle_decay": "12.000000", "genome_carrying": "13.000000", "genome_size_creature_ticks": 14,
            "parental_transfer_debit": "15.000000", "offspring_energy_credit": "16.000000",
            "predation_victim_debit": "-17.000000", "predation_attacker_credit": "-18.000000",
            "predation_kill_bonus_credit": "19.000000", "maximum_energy_clamp_loss": "20.000000",
            "zero_floor_credit": "21.000000", "external_removal_loss": "22.000000",
        })
    );
}

/// The world-set tracking fractions read against the totals they came
/// from, and the two reader-state fractions use their own denominators:
/// the barrier-block rate is per state, against that state's own attempts
/// beside a barrier; the avoidable share is against every move the whole
/// population attempted.
#[test]
fn tracking_fractions_divide_each_reading_by_its_own_denominator() {
    let tracking = WorldTracking {
        typed_eats_total: vec![3, 1],
        food_density_total: vec![six(1.5)],
        moves_attempted_total: 8,
        moves_blocked_barrier_total: 2,
        moves_blocked_total_by_cause: Some(MovesBlockedByCause {
            barrier: 2,
            occupied: 3,
            out_of_bounds: 1,
        }),
        moves_blocked_avoidable_by_reader_state: ByReaderState {
            has_barrier_reader: 1,
            no_barrier_reader: 3,
        },
        move_attempts_with_barrier_neighbor_by_reader_state: ByReaderState {
            has_barrier_reader: 4,
            no_barrier_reader: 16,
        },
        moves_blocked_barrier_with_barrier_neighbor_by_reader_state: ByReaderState {
            has_barrier_reader: 1,
            no_barrier_reader: 10,
        },
        ..WorldTracking::default()
    };
    let fractions = tracking.fractions();
    assert_eq!(fractions.typed_eat_share, vec![six(0.75), six(0.25)]);
    assert_eq!(fractions.blocked_move_fraction, six(0.25));
    assert_eq!(
        fractions.barrier_blocked_fraction_by_reader_state,
        ByReaderState {
            has_barrier_reader: six(0.25),
            no_barrier_reader: six(0.625),
        },
        "each state's barrier blocks against that same state's attempts beside a barrier"
    );
    assert_eq!(
        fractions.avoidable_blocked_share_of_all_moves_by_reader_state,
        ByReaderState {
            has_barrier_reader: six(0.125),
            no_barrier_reader: six(0.375),
        }
    );

    let empty = WorldTracking {
        typed_eats_total: vec![0, 0],
        ..WorldTracking::default()
    }
    .fractions();
    assert_eq!(empty.typed_eat_share, vec![UNDEFINED, UNDEFINED]);
    assert_eq!(empty.blocked_move_fraction, UNDEFINED);
    assert_eq!(
        empty.barrier_blocked_fraction_by_reader_state,
        ByReaderState {
            has_barrier_reader: UNDEFINED.to_string(),
            no_barrier_reader: UNDEFINED.to_string(),
        },
        "a state that never moved beside a barrier is unmeasured, not zero"
    );
    assert_eq!(
        empty.avoidable_blocked_share_of_all_moves_by_reader_state,
        ByReaderState {
            has_barrier_reader: UNDEFINED.to_string(),
            no_barrier_reader: UNDEFINED.to_string(),
        }
    );

    let barrier_free = WorldTracking {
        moves_attempted_total: 10,
        moves_blocked_avoidable_by_reader_state: ByReaderState {
            has_barrier_reader: 0,
            no_barrier_reader: 2,
        },
        ..WorldTracking::default()
    }
    .fractions();
    assert_eq!(
        barrier_free.barrier_blocked_fraction_by_reader_state,
        ByReaderState {
            has_barrier_reader: UNDEFINED.to_string(),
            no_barrier_reader: UNDEFINED.to_string(),
        },
        "a world without barriers reports no barrier-block rate at all"
    );
    assert_eq!(
        barrier_free
            .avoidable_blocked_share_of_all_moves_by_reader_state
            .no_barrier_reader,
        six(0.2),
        "the avoidable share still counts blocks of every other cause"
    );
}

/// Every blocked-move cause is read from its own key of
/// `move_actions_blocked_total_by_cause`, and the kept
/// `moves_blocked_barrier_total` is exactly that map's barrier entry, so
/// the two can never disagree.
#[test]
fn observe_reads_each_blocked_move_cause_from_its_own_stats_key() {
    use v3_core::simulation::actions::MoveBlockedCause::{Barrier, Occupied, OutOfBounds};
    let params = small_profile("sweep");
    let mut sim = seed_simulation(build_config(&params), 1);
    assert_eq!(
        WorldTracking::observe(&sim).moves_blocked_total_by_cause,
        Some(MovesBlockedByCause::default()),
        "a measured run whose creatures never hit a cause reads zero, not absent"
    );
    for (cause, count) in [(Barrier, 5), (Occupied, 9), (OutOfBounds, 13)] {
        sim.stats
            .move_actions_blocked_total_by_cause
            .insert(cause, count);
    }

    let tracking = WorldTracking::observe(&sim);
    assert_eq!(
        tracking.moves_blocked_total_by_cause,
        Some(MovesBlockedByCause {
            barrier: 5,
            occupied: 9,
            out_of_bounds: 13,
        })
    );
    assert_eq!(
        tracking.moves_blocked_barrier_total, 5,
        "the kept barrier total is the map's barrier entry"
    );
}

/// Each barrier counter comes from its own stats map under its own
/// reader-state key. The barrier-block rate's numerator and denominator
/// are separate measurements, so reading either from the other's map, or
/// under the other state's key, would misreport barrier awareness.
#[test]
fn observe_reads_each_barrier_counter_from_its_own_stats_map() {
    use v3_core::simulation::actions::BarrierReaderState::{HasBarrierReader, NoBarrierReader};
    let params = small_profile("sweep");
    let mut sim = seed_simulation(build_config(&params), 1);
    for (state, attempts, blocked, avoidable) in
        [(HasBarrierReader, 7, 3, 5), (NoBarrierReader, 11, 2, 13)]
    {
        sim.stats
            .move_attempts_with_barrier_neighbor_total_by_reader_state
            .insert(state, attempts);
        sim.stats
            .move_blocked_barrier_with_barrier_neighbor_total_by_reader_state
            .insert(state, blocked);
        sim.stats
            .move_actions_blocked_avoidable_total_by_reader_state
            .insert(state, avoidable);
    }

    let tracking = WorldTracking::observe(&sim);
    assert_eq!(
        tracking.move_attempts_with_barrier_neighbor_by_reader_state,
        ByReaderState {
            has_barrier_reader: 7,
            no_barrier_reader: 11,
        }
    );
    assert_eq!(
        tracking.moves_blocked_barrier_with_barrier_neighbor_by_reader_state,
        ByReaderState {
            has_barrier_reader: 3,
            no_barrier_reader: 2,
        }
    );
    assert_eq!(
        tracking.moves_blocked_avoidable_by_reader_state,
        ByReaderState {
            has_barrier_reader: 5,
            no_barrier_reader: 13,
        }
    );
}

/// Historical reports predate every tracking field, and a round trip of a
/// current report keeps them flat on the wire.
#[test]
fn tracking_fields_default_when_absent_and_survive_a_round_trip() {
    let legacy: PersistenceSample = serde_json::from_value(serde_json::json!({
        "tick": 100, "population": 5, "mean_energy": "1.000000", "births_total": 2
    }))
    .expect("a pre-T12.F04 sample must still parse");
    assert_eq!(legacy.tracking, WorldTracking::default());
    assert_eq!(legacy.mean_genome_size, None);
    assert_eq!(legacy.mean_mesh_nodes, None);
    assert_eq!(legacy.mean_generation, None);
    assert_eq!(legacy.surviving_founder_clade_count, None);
    assert_eq!(legacy.shannon_entropy_nats, None);
    assert_eq!(legacy.sensor_census, None);
    assert_eq!(legacy.occupancy_grid, None);

    let sample = PersistenceSample {
        tick: 100,
        population: 5,
        mean_energy: None,
        births_total: 2,
        mean_genome_size: Some(six(111.0)),
        mean_mesh_nodes: Some(six(8.0)),
        mean_generation: Some(six(3.5)),
        surviving_founder_clade_count: Some(4),
        shannon_entropy_nats: Some(six(1.25)),
        sensor_census: Some(census_for(5)),
        occupancy_grid: Some(grid_for(5)),
        tracking: WorldTracking {
            typed_eats_total: vec![7],
            food_density_total: vec![six(2.0)],
            grazing_modifier_mean: vec![six(0.875)],
            grazed_cell_share: vec![six(0.125)],
            moves_attempted_total: 9,
            moves_blocked_barrier_total: 1,
            moves_blocked_total_by_cause: Some(MovesBlockedByCause {
                barrier: 1,
                occupied: 2,
                out_of_bounds: 3,
            }),
            moves_blocked_avoidable_by_reader_state: ByReaderState {
                has_barrier_reader: 0,
                no_barrier_reader: 1,
            },
            move_attempts_with_barrier_neighbor_by_reader_state: ByReaderState {
                has_barrier_reader: 4,
                no_barrier_reader: 3,
            },
            moves_blocked_barrier_with_barrier_neighbor_by_reader_state: ByReaderState {
                has_barrier_reader: 2,
                no_barrier_reader: 1,
            },
            typed_eats_failed_total: Some(vec![4]),
            mesh_dispatches_energy_exhausted_total: Some(6),
            mutation_supply: Some(MutationSupply {
                events_attempted_total: 12,
                events_applied_total: 9,
                events_skipped_total: 3,
                executed_target_total: 2,
                reachable_target_total: 5,
                unreachable_target_total: 3,
                not_applicable_target_total: 1,
            }),
            mutation_outcome_summary: Some(MutationOutcomeTotals {
                carriers_observed_total: 4,
                survival_ticks_sum: 40,
                offspring_spawned_sum: 2,
                ..MutationOutcomeTotals::default()
            }),
            mutation_value_totals_by_operator: Some(BTreeMap::from([(
                "Topology.AddNode".to_string(),
                MutationOutcomeTotals {
                    carriers_observed_total: 1,
                    ..MutationOutcomeTotals::default()
                },
            )])),
            predation: Some(PredationTracking {
                actions_attempted_total: 5,
                actions_transferred_total: 3,
                actions_rejected_total: 2,
                kills_total: 1,
                actions_by_result: BTreeMap::from([("Transferred".to_string(), 3)]),
            }),
            mortality: None,
            reproductive_success_by_cognitive_class: None,
            energy_flows: None,
            cognition: None,
        },
    };
    let wire = serde_json::to_value(&sample).unwrap();
    assert_eq!(
        wire["moves_attempted_total"], 9,
        "tracking stays flat: {wire}"
    );
    assert_eq!(
        wire["move_attempts_with_barrier_neighbor_by_reader_state"]["has_barrier_reader"], 4,
        "the barrier-block denominator is on the wire under its own key: {wire}"
    );
    assert_eq!(
        wire["moves_blocked_total_by_cause"]["occupied"], 2,
        "each blocked-move cause is on the wire under its own key: {wire}"
    );
    assert_eq!(
        wire["mutation_supply"]["executed_target_total"], 2,
        "the mutation target split is on the wire under its own key: {wire}"
    );
    assert_eq!(
        wire["grazing_modifier_mean"][0], "0.875000",
        "the grazing reading is on the wire beside the food density: {wire}"
    );
    assert_eq!(wire["grazed_cell_share"][0], "0.125000");
    assert_eq!(
        wire["typed_eats_failed_total"][0], 4,
        "failed eats are indexed by food type: {wire}"
    );
    let decoded: PersistenceSample = serde_json::from_value(wire).unwrap();
    assert_eq!(decoded.tracking, sample.tracking);
}

/// A report stored before T14.F02 carries no transferred block, and every
/// one of them reads as absent rather than as a zero the run produced.
#[test]
fn transferred_tracking_blocks_are_absent_not_zero_in_a_historical_report() {
    let legacy: PersistenceSample = serde_json::from_value(serde_json::json!({
        "tick": 100, "population": 5, "births_total": 2,
        "typed_eats_total": [7], "moves_attempted_total": 9
    }))
    .expect("a pre-T14.F02 sample must still parse");
    assert_eq!(legacy.tracking.typed_eats_total, vec![7]);
    assert_eq!(legacy.tracking.typed_eats_failed_total, None);
    assert_eq!(legacy.tracking.mesh_dispatches_energy_exhausted_total, None);
    assert_eq!(legacy.tracking.mutation_supply, None);
    assert_eq!(legacy.tracking.mutation_outcome_summary, None);
    assert_eq!(legacy.tracking.mutation_value_totals_by_operator, None);
    assert_eq!(legacy.tracking.predation, None);
    assert_eq!(legacy.tracking.mortality, None);
    assert_eq!(legacy.tracking.energy_flows, None);
    assert_eq!(
        legacy.tracking.reproductive_success_by_cognitive_class,
        None
    );
    let encoded = serde_json::to_value(legacy).unwrap();
    assert!(encoded.get("mortality").is_none());
    assert!(encoded.get("energy_flows").is_none());
    assert!(encoded
        .get("reproductive_success_by_cognitive_class")
        .is_none());
}
