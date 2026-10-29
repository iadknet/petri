use super::*;
use crate::bench::profiles::{
    goal_profile_params, goal_recipe_seeds, GOAL_RECIPES, GOAL_WORLD_SET, SAMPLE_EVERY_TICKS,
};
use crate::bench::schema::MeshExecution;
use crate::bench::tests::small_profile;
use crate::{fraction_or_undefined, UNDEFINED};
use proptest::prelude::*;
use v3_core::neighborhood;

#[test]
fn empty_measured_population_reports_zero_companion_counts() {
    let mut config = SimulationConfig::default();
    config.population.initial_creatures = 0;
    let run = run_one_seed(&config, 11, 0, true, None, None);
    let observation = run.goal_observation.expect("measured final population");
    assert_eq!(observation.memory_sensitivity.final_creature_count, 0);
    assert_eq!(
        observation.structural_companions,
        StructuralCompanionsSeed {
            seed: 11,
            ..Default::default()
        }
    );
    assert_eq!(
        serde_json::to_value(observation.structural_companions).unwrap(),
        serde_json::json!({
            "seed": 11, "final_creature_count": 0, "reads_shared_memory": 0,
            "writes_shared_memory": 0, "has_stateful_compute_node": 0, "has_plasticity": 0,
        })
    );
}

#[test]
fn goal_case_adds_observation_time_to_zero_accumulators() {
    let mut params = small_profile(GOAL_WORLD_SET);
    params.seeds = goal_recipe_seeds();
    let mut founder_ms = 0.0;
    let mut drift_ms = Some(0.0);
    let _case = prepare_goal_case(&params, &GOAL_RECIPES[0], &mut founder_ms, &mut drift_ms);
    assert!(founder_ms > 0.0);
    assert!(drift_ms.unwrap() > 0.0);
}

#[test]
fn goal_cases_keep_distinct_full_population_structure_distributions() {
    let mut params = goal_profile_params();
    params.width = 32;
    params.height = 32;
    params.founders = 32;
    // Effort-cost defaults, 2026-10-10: 40, not 60, so every goal case still
    // holds more than `SAMPLE_SIZE` creatures (Orchards fell to 9 at 60).
    params.ticks = 40;
    params.neighborhood = NeighborhoodSizes::default();
    params.drift = Default::default();
    params.mutation_effects = Default::default();
    let (report, _) = run_deterministic(&params).expect("a valid profile");
    let mut expected_cases = Vec::new();
    let mut pooled = Vec::new();
    for (index, case) in report.goal_indicators.cases.iter().enumerate() {
        let (_, config) = goal_case(&params, &GOAL_RECIPES[index]);
        let run = run_one_seed(
            &config,
            params.seeds[index],
            params.ticks,
            false,
            None,
            None,
        );
        assert_eq!(run.complexities.len() as u64, run.per_seed.final_population);
        assert!(run.complexities.len() > neighborhood::SAMPLE_SIZE);
        pooled.extend_from_slice(&run.complexities);
        let expected = serde_json::to_value(structure_size_distribution(run.complexities)).unwrap();
        assert_eq!(
            serde_json::to_value(case.reachable_structure_size_distribution.as_ref().unwrap())
                .unwrap(),
            expected
        );
        expected_cases.push(expected);
        let mut historical = serde_json::to_value(case).unwrap();
        historical
            .as_object_mut()
            .unwrap()
            .remove("reachable_structure_size_distribution");
        let historical: GoalCaseObservation = serde_json::from_value(historical).unwrap();
        assert!(historical.reachable_structure_size_distribution.is_none());
    }
    assert!(expected_cases.windows(2).any(|pair| pair[0] != pair[1]));
    assert_eq!(
        serde_json::to_value(report.goal_indicators.reachable_structure_size_distribution).unwrap(),
        serde_json::to_value(structure_size_distribution(pooled)).unwrap()
    );
}

#[test]
fn goal_world_set_executes_three_named_configs_with_case_observations() {
    let mut params = goal_profile_params();
    params.width = 16;
    params.height = 16;
    params.founders = 4;
    params.ticks = 1;
    params.neighborhood = NeighborhoodSizes::default();
    params.drift = Default::default();
    params.mutation_effects = Default::default();
    let (report, timings) = run_deterministic(&params).expect("a valid profile");
    assert_eq!(report.profile.name, "goal-worlds-v1");
    assert_eq!(report.per_seed.len(), 3);
    assert_eq!(
        report
            .profile
            .cases
            .iter()
            .map(|case| case.seed)
            .collect::<Vec<_>>(),
        vec![11, 22, 33]
    );
    assert_eq!(
        report
            .profile
            .cases
            .iter()
            .map(|case| case.food_type_count)
            .collect::<Vec<_>>(),
        vec![2, 1, 2]
    );
    assert_eq!(report.goal_indicators.cases.len(), 3);
    assert!(
        matches!(report.goal_indicators.mutational_neighborhood, Indicator::Undefined(ref reason) if reason == "reported per case")
    );
    assert!(
        matches!(report.goal_indicators.drift_depth, Indicator::Undefined(ref reason) if reason == "reported per case")
    );
    for (index, case) in report.goal_indicators.cases.iter().enumerate() {
        let (expected, config) = goal_case(&params, &GOAL_RECIPES[index]);
        assert_eq!(case.case, expected);
        let seeded = seed_simulation(config.clone(), expected.seed);
        assert_eq!(
            report.per_seed[index].tick_zero_connectivity,
            Some(seeded.world.passable_connectivity())
        );
        let Indicator::Defined(neighborhood) = &case.mutational_neighborhood else {
            panic!("case neighborhood missing")
        };
        let battery = Battery::generate(config.world.food.types.len());
        let expected_founder = compute_founder_neighborhood(&config, &battery, params.neighborhood);
        assert_eq!(
            serde_json::to_value(&neighborhood.founder).unwrap(),
            serde_json::to_value(expected_founder).unwrap()
        );
        let Indicator::Defined(evolved) = &neighborhood.evolved else {
            panic!("case evolved reading missing")
        };
        assert_eq!(evolved.per_seed.len(), 1);
        assert_eq!(evolved.per_seed[0].seed, expected.seed);
        assert!(matches!(case.drift_depth, Indicator::Defined(_)));
    }
    assert!(timings.neighborhood_founder_wall_clock_ms > 0.0);
    assert!(timings.drift_depth_wall_clock_ms.unwrap() > 0.0);
    assert_eq!(timings.neighborhood_evolved_wall_clock_ms_per_seed.len(), 3);
}

#[test]
fn reduced_drift_deterministic_output_matches_across_thread_counts_and_seed_counts() {
    let mut params = small_profile("goal");
    let one = rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .build()
        .unwrap()
        .install(|| run_deterministic(&params).expect("a valid profile").0);
    let two = rayon::ThreadPoolBuilder::new()
        .num_threads(2)
        .build()
        .unwrap()
        .install(|| run_deterministic(&params).expect("a valid profile").0);
    assert_eq!(
        serde_json::to_vec(&one).unwrap(),
        serde_json::to_vec(&two).unwrap()
    );
    params.seeds.push(99);
    let more = run_deterministic(&params).expect("a valid profile").0;
    assert_eq!(
        one.goal_indicators.drift_depth,
        more.goal_indicators.drift_depth
    );
}

#[test]
fn drift_is_goal_only_once_and_historical_fields_are_unavailable() {
    let goal = build_report(&small_profile("goal"), "test").expect("a valid profile");
    let Indicator::Defined(drift) = &goal.deterministic.goal_indicators.drift_depth else {
        panic!("goal drift missing")
    };
    assert_eq!(drift.version, "drift-depth-v4");
    assert_eq!(drift.recruitment_version, "module-recruitment-v1");
    assert!(drift.module_identity.contains("creation depth"));
    assert!(drift.provenance_rule.contains("Topology.CopyNode"));
    assert_eq!(drift.founder, "V3Alpha1");
    assert_eq!(
        drift.birth_subset,
        "first lineage indices in ascending order"
    );
    assert_eq!(drift.walk_seed_formula, "90000 + lineage_index");
    assert_eq!(
        drift.birth_seed_formula,
        "7000000 + 1000 * (lineage_index + 1) + checkpoint + 9000 + trial_index"
    );
    assert_eq!(drift.battery_version, "neighborhood-v2");
    assert_eq!(drift.mesh_version, "mesh-execution-v2");
    assert_eq!(drift.knockout_method, "static-successor-bypass-v1");
    assert_eq!(
        drift.executed_source,
        "battery hop records (mesh-execution-v2), node ids"
    );
    assert_eq!(
        drift.executed_refresh,
        "walk: depth 0 and every 10 generations; births: derived at each checkpoint"
    );
    assert_eq!(
        drift.supply_rule,
        // T29.F02: per-chromosome draws on the founder's 80 brain and 5 body
        // units at the rate 0.485 / 85 (T25.F02: one draw on 80 units at
        // 0.0060625).
        "per-unit draws on the canonical V3Alpha1 founder's units, brain 80 and body 5, at \
         per_unit_rate 0.005705882352941176: Binomial(80, 0.005705882352941176) brain and \
         Binomial(5, 0.005705882352941176) body events per walk and checkpoint birth"
    );
    assert_eq!(
        (
            drift.executions_per_genome,
            drift.snapshot_count,
            drift.sequence_count,
            drift.sequence_len
        ),
        (80, 48, 8, 4)
    );
    let production = goal_profile_params().drift;
    assert_eq!(
        (
            production.lineages,
            production.birth_lineages,
            production.births
        ),
        (50, 20, 100)
    );
    assert_eq!(production.checkpoints, &[0, 22, 250, 1000, 2000]);
    assert_eq!(drift.lineages, 2);
    assert_eq!(drift.birth_lineages, 1);
    assert_eq!(drift.birth_trials, 2);
    assert_eq!(drift.checkpoints, vec![0, 2]);
    assert_eq!(drift.readings.len(), 2);
    assert_eq!(drift.readings[0].births.births_total, 2);
    assert_eq!(drift.readings[0].battery_executions, 160);
    // Every checkpoint carries the v3 blocks, with the founder reference
    // row and the pooled opportunity denominators.
    for reading in &drift.readings {
        let recruitment = reading.recruitment.as_ref().expect("a recruitment block");
        assert_eq!(
            recruitment.founders.created,
            recruitment.founders.present + recruitment.founders.deleted
        );
        assert_eq!(recruitment.lineages.len(), drift.lineages as usize);
        let opportunities = reading
            .opportunities
            .as_ref()
            .expect("an opportunity block");
        assert_eq!(
            opportunities.births,
            reading.depth * u64::from(drift.lineages)
        );
        assert_eq!(
            opportunities.attempted,
            opportunities.applied + opportunities.skipped
        );
        // The per-lineage rows sum to the pooled discard total.
        assert_eq!(
            opportunities
                .lineages
                .iter()
                .map(|row| row.discarded_selected_inapplicable)
                .sum::<u64>(),
            opportunities
                .discarded_selected_inapplicable_by_operator
                .values()
                .sum::<u64>()
        );
    }
    assert!(goal.environment.drift_depth_wall_clock_ms.is_some());
    for name in ["gate", "sweep", "synthetic"] {
        let report = build_report(&small_profile(name), "test").expect("a valid profile");
        assert!(matches!(
            report.deterministic.goal_indicators.drift_depth,
            Indicator::Undefined(_)
        ));
        assert_eq!(report.environment.drift_depth_wall_clock_ms, None);
    }
    let mut historical = serde_json::to_value(&goal).unwrap();
    historical["deterministic"]["goal_indicators"]
        .as_object_mut()
        .unwrap()
        .remove("drift_depth");
    historical["environment"]
        .as_object_mut()
        .unwrap()
        .remove("drift_depth_wall_clock_ms");
    let historical: Report = serde_json::from_value(historical).unwrap();
    assert!(matches!(
        historical.deterministic.goal_indicators.drift_depth,
        Indicator::Undefined(_)
    ));
    assert_eq!(historical.environment.drift_depth_wall_clock_ms, None);
}

/// `millis` converts to fractional milliseconds. A pure unit conversion,
/// not a wall-clock magnitude: the input duration is synthetic.
#[test]
fn millis_converts_a_duration_to_fractional_milliseconds() {
    use std::time::Duration;

    assert_eq!(millis(Duration::from_millis(1500)), 1500.0);
    assert_eq!(millis(Duration::from_micros(1)), 0.001);
    assert_eq!(millis(Duration::ZERO), 0.0);
}

fn synthetic_timings(seed_wall_clock_ms: &[f64]) -> RunTimings {
    RunTimings {
        wall_clock_ms_per_seed: seed_wall_clock_ms
            .iter()
            .enumerate()
            .map(|(index, &wall_clock_ms)| SeedWallClock {
                seed: index as u64,
                wall_clock_ms,
            })
            .collect(),
        phase_wall_clock_ms_per_seed: Vec::new(),
        throughput_per_seed: Vec::new(),
        final_state_observation_ms_per_seed: Vec::new(),
        neighborhood_founder_wall_clock_ms: 0.0,
        drift_depth_wall_clock_ms: None,
        neighborhood_evolved_wall_clock_ms_per_seed: Vec::new(),
        neighborhood_read_wall_clock_ms_per_seed: Vec::new(),
        mutation_effects_wall_clock_ms_per_seed: Vec::new(),
        input_use_wall_clock_ms_per_seed: Vec::new(),
    }
}

/// `build_environment` sums the per-seed wall-clock, divides it by the
/// run's creature-ticks, and records the thread count it was given. The
/// inputs are synthetic, so this asserts the arithmetic, never a timing.
#[test]
fn build_environment_derives_the_per_creature_tick_cost_and_throughput() {
    let totals = Totals {
        ticks: 200,
        creature_ticks: 4_000,
        births: 50,
        ..Totals::default()
    };

    let environment = build_environment(synthetic_timings(&[300.0, 500.0]), &totals, 4);

    assert_eq!(environment.wall_clock_ms_total, 800.0);
    assert_eq!(environment.wall_clock_ms_per_creature_tick, 0.2);
    assert_eq!(environment.threads, Some(4));
    assert_eq!(environment.rand_version.as_deref(), Some("0.8.6"));
    let mut historical = serde_json::to_value(&environment).unwrap();
    historical.as_object_mut().unwrap().remove("rand_version");
    assert_eq!(
        serde_json::from_value::<Environment>(historical)
            .unwrap()
            .rand_version,
        None
    );
    assert_eq!(
        environment.throughput.total,
        throughput_rates(200, 4_000, 50, 800.0)
    );
    assert_eq!(environment.throughput.total.ticks_per_second, 250.0);
}

/// A run with no creature-ticks reports a zero per-creature-tick cost
/// rather than dividing by zero.
#[test]
fn build_environment_reports_zero_cost_when_no_creature_ran() {
    let environment = build_environment(synthetic_timings(&[12.5]), &Totals::default(), 1);

    assert_eq!(environment.wall_clock_ms_total, 12.5);
    assert_eq!(environment.wall_clock_ms_per_creature_tick, 0.0);
}

/// Stored reports carry blocks a later closure retired (T30.F09 removed a
/// goal-only experiment and its wall clock); the reader ignores them.
#[test]
fn a_report_carrying_a_retired_block_still_parses() {
    let report = build_report(&small_profile("gate"), "retired-block-test").unwrap();
    let mut stored = serde_json::to_value(&report).unwrap();
    stored["deterministic"]["goal_indicators"]["retired_experiment"] =
        serde_json::json!("unmeasured; retired-experiment-v1 is goal-only");
    stored["environment"]["retired_experiment_wall_clock_ms"] = serde_json::Value::Null;
    let read: Report = serde_json::from_value(stored).unwrap();
    assert_eq!(
        serde_json::to_value(&read).unwrap(),
        serde_json::to_value(&report).unwrap()
    );
}

#[test]
fn mesh_report_deterministic_fields_match_across_runs_and_threads() {
    for profile in ["gate", "goal"] {
        let params = small_profile(profile);
        let run = |threads| {
            rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .build()
                .unwrap()
                .install(|| {
                    serde_json::to_vec(&run_deterministic(&params).expect("a valid profile").0)
                        .unwrap()
                })
        };
        assert_eq!(run(1), run(2));
    }
}

#[test]
fn mutational_neighborhood_is_defined_only_for_the_gate_and_goal_profile_names() {
    let (gate_det, _) = run_deterministic(&small_profile("gate")).expect("a valid profile");
    let Indicator::Defined(gate_neighborhood) = gate_det.goal_indicators.mutational_neighborhood
    else {
        panic!("the gate profile must define mutational_neighborhood");
    };
    assert_eq!(gate_neighborhood.founder.generation, Some(0));
    let Indicator::Defined(mesh) = &gate_neighborhood.founder.mesh_execution else {
        panic!("founder mesh reading");
    };
    let counts = mesh.backends.expect("present backend measurement");
    assert_eq!(counts.graph.total + counts.vm.total, mesh.total_node_count);
    assert_eq!(
        counts.graph.executed + counts.vm.executed,
        mesh.executed_node_count
    );
    assert_eq!(
        counts.graph.contributing + counts.vm.contributing,
        mesh.executed_node_count - mesh.knockout_count
    );
    let mut historical = serde_json::to_value(mesh).unwrap();
    historical.as_object_mut().unwrap().remove("backends");
    historical
        .as_object_mut()
        .unwrap()
        .remove("route_destination_varies");
    let historical: MeshExecution = serde_json::from_value(historical).unwrap();
    assert_eq!(historical.backends, None);
    assert!(!historical.route_destination_varies);
    assert_eq!(historical.total_node_count, mesh.total_node_count);
    assert_eq!(mesh.version, "mesh-execution-v2");
    assert_eq!(mesh.executions_per_genome, 80);
    assert_eq!(mesh.snapshot_route_probes, 48);
    assert_eq!(mesh.knockout_method, "static-successor-bypass-v1");
    let Indicator::Defined(steering) = &gate_neighborhood.founder.steering else {
        panic!("founder steering reading");
    };
    assert_eq!(steering.version, "steering-v1");
    assert_eq!(steering.seed, 9);
    assert_eq!(steering.base_count, 6);
    assert_eq!(steering.reading.scenarios, 48);
    assert!(steering.reading.move_voted, "the vote founder votes Move");
    assert!(
        matches!(gate_neighborhood.evolved, Indicator::Undefined(_)),
        "the evolved half never runs in the gate profile"
    );

    let (goal_det, _) = run_deterministic(&small_profile("goal")).expect("a valid profile");
    let Indicator::Defined(goal_neighborhood) = goal_det.goal_indicators.mutational_neighborhood
    else {
        panic!("the goal profile must define mutational_neighborhood");
    };
    let Indicator::Defined(evolved) = &goal_neighborhood.evolved else {
        panic!("the evolved half runs in the goal profile");
    };
    for seed in &evolved.per_seed {
        let Indicator::Defined(pooled) = &seed.steering_pooled else {
            panic!("pooled steering on every evolved seed");
        };
        assert_eq!(pooled.version, "steering-v1");
        assert_eq!(pooled.sums.genomes, seed.sampled_genomes.len() as u64);
        assert_eq!(pooled.chance.exact, 0.125);
        assert_eq!(pooled.chance.within_45, 0.375);
        let mut moves = 0;
        for sample in &seed.sampled_genomes {
            let Indicator::Defined(steering) = &sample.steering else {
                panic!("steering on every sampled genome");
            };
            moves += steering.reading.moves;
        }
        assert_eq!(pooled.sums.moves, moves);
        if pooled.sums.moves == 0 {
            assert_eq!(pooled.exact_hit_fraction, "Undefined");
        }
    }

    let (synthetic_det, _) =
        run_deterministic(&small_profile("synthetic")).expect("a valid profile");
    assert!(matches!(
        synthetic_det.goal_indicators.mutational_neighborhood,
        Indicator::Undefined(_)
    ));

    let (sweep_det, _) = run_deterministic(&small_profile("sweep")).expect("a valid profile");
    assert!(matches!(
        sweep_det.goal_indicators.mutational_neighborhood,
        Indicator::Undefined(_)
    ));
}
/// Every persistence sample carries the tracking counters as of its tick,
/// on the same cadence `births_total` uses, and they never go backwards.
#[test]
fn persistence_samples_carry_world_tracking_on_the_births_cadence() {
    let mut params = small_profile("sweep");
    params.width = 24;
    params.height = 24;
    params.founders = 16;
    // Effort-cost defaults, 2026-10-10: seed 1 goes extinct at tick 91; seed
    // 15 is the lowest of 0..=40 that survives the run (15 and 33 do).
    // T11.F30 (founder heading draws): seed 15 goes extinct at tick 83; seed
    // 1 is the lowest of 0..=40 that survives (1, 3, 19, 26, 33, 37 do).
    // T20.F04/F05 as defaults, 2026-10-12: seed 1 goes extinct at tick 99;
    // seed 4 is the lowest of 0..=40 that survives (4 and 18 do).
    // T25.F02 (uniform size rule), 2026-10-14: seed 4 goes extinct at tick
    // 86; seed 11 is the lowest of 0..=40 that passes. The old size rule and
    // anchor 97 restore seed 4. T25.F02 commit 2 (default rate 0.005 ->
    // 0.0060625): seed 11 goes extinct at tick 91; seed 9 is the lowest of
    // 0..=60 that survives (9, 37, 45 and 50 do). Rate 0.005 restores seed 11.
    // T25.F04 commit 1 (`AddDirectionPairGroup` remaps every Graph draw):
    // seed 9 goes extinct; seed 25 is the lowest of 0..=200 that survives
    // (25, 28, 37, 50, 53 and 64 are the first six). Removing the catalog
    // entry restores seed 9. T25.F07 commit 2 (learned bundle creation, the
    // scope coin and `SetScope` remap every Graph draw): seed 25 goes
    // extinct; seed 3 is the lowest of 0..=200 that survives (3, 28, 37, 45,
    // 48 and 50 are the first six). T25.F08 commit 1 (deleting
    // `RefineHeritableStructure` remaps every Graph draw): seed 3 goes
    // extinct at tick 93; seed 9 is the lowest of 0..=200 that survives (9,
    // 37, 45, 50, 63 and 64 are the first six). T25.F08 commit 2 (the
    // randomized creation operators and `DetachGraphEdge` remap every Graph
    // draw): seed 9 goes extinct; seed 14 is the lowest of 0..=200 that
    // survives (14, 37, 40, 50, 51 and 53 are the first six). T25.F09 commit
    // 3 (the six retention operators remap every Graph draw): seed 14 goes
    // extinct; seed 2 is the lowest of 0..=200 that survives (2, 15, 18, 37,
    // 53 and 79 are the first six). T26.F02 (the capacity chance drawn on
    // every birth shifts every later draw): seed 2 goes extinct at tick 87;
    // seed 4 is the lowest of 0..=200 that survives (4, 11, 20, 33, 40 and
    // 42 are the first six). Removing the capacity draw restores seed 2.
    // T27.F03 commit 3 (the two share draws on every birth shift every later
    // draw): seed 4 goes extinct at tick 84; seed 1 is the lowest of 0..=200
    // that survives (1, 9, 37, 43, 75 and 110 are the first six). T27.F03
    // commit 4 (the `GiveEnergy` kind widens the kind, sink and field
    // draws): seed 1 goes extinct; seed 9 is the lowest of 0..=200 that
    // survives (9, 37, 43, 53, 60 and 75 are the first six). T25.F12 commit
    // 2 (the learning catalog, the target draw over edges and slots and the
    // bundle birth draws remap every Graph draw): seed 9 goes extinct at
    // tick 79; seed 1 is the lowest of 0..=200 that survives. T29.F02 (the
    // body genes' carrying units and the body's own supply): seed 1 goes
    // extinct at tick 87; seed 102 is the lowest of 0..=400 that survives
    // (102, 112, 149, 182, 193 and 227 are the first six).
    params.seeds = vec![102];
    params.ticks = SAMPLE_EVERY_TICKS + 5;
    let (report, _) = run_deterministic(&params).expect("a valid profile");
    let seed = &report.goal_indicators.population_persistence.per_seed[0];
    assert_eq!(
        seed.samples.iter().map(|s| s.tick).collect::<Vec<_>>(),
        vec![SAMPLE_EVERY_TICKS, params.ticks],
        "the cadence is every hundredth tick plus the final tick"
    );
    let mut previous = 0;
    for sample in &seed.samples {
        assert_eq!(sample.tracking.typed_eats_total.len(), 1);
        assert_eq!(sample.tracking.food_density_total.len(), 1);
        assert!(
            sample.tracking.moves_attempted_total >= previous,
            "move attempts are cumulative"
        );
        previous = sample.tracking.moves_attempted_total;
        assert!(
            sample.tracking.moves_blocked_barrier_total <= sample.tracking.moves_attempted_total
        );
        let by_cause = sample
            .tracking
            .moves_blocked_total_by_cause
            .as_ref()
            .expect("a measured sample carries every blocked-move cause");
        assert_eq!(
            by_cause.barrier, sample.tracking.moves_blocked_barrier_total,
            "the kept barrier total and the by-cause barrier entry are one measurement"
        );
        assert!(
            by_cause.barrier + by_cause.occupied + by_cause.out_of_bounds
                <= sample.tracking.moves_attempted_total,
            "every blocked move, of any cause, is one of the moves attempted"
        );
        let beside = &sample
            .tracking
            .move_attempts_with_barrier_neighbor_by_reader_state;
        let blocked_beside = &sample
            .tracking
            .moves_blocked_barrier_with_barrier_neighbor_by_reader_state;
        assert!(
            blocked_beside.has_barrier_reader <= beside.has_barrier_reader
                && blocked_beside.no_barrier_reader <= beside.no_barrier_reader,
            "a barrier block beside a barrier is one of that state's attempts beside one"
        );
        assert!(
            beside.has_barrier_reader + beside.no_barrier_reader
                <= sample.tracking.moves_attempted_total,
            "attempts beside a barrier are a subset of every move attempted"
        );
    }
    assert!(previous > 0, "a moving population must attempt moves");
}

proptest! {
    /// The profile totals are exactly the field-wise sum of the per-case
    /// (world-set) or per-seed rows the same report carries.
    #[test]
    fn profile_totals_are_the_field_wise_sum_of_every_case_row(
        rows in proptest::collection::vec(
            (
                (0u64..1000, 0u64..1000, 0u64..1000, 0u64..1000, 0u64..1000, 0u64..1000),
                (0u64..1000, 0u64..1000, 0u64..1000, 0u64..1000, 0u64..1000),
            ),
            0..6usize,
        )
    ) {
        let per_seed: Vec<PerSeed> = rows
            .iter()
            .enumerate()
            .map(|(index, (a, b))| PerSeed {
                tick_zero_connectivity: None,
                seed: index as u64,
                ticks: a.0,
                creature_ticks: a.1,
                mesh_hops: a.2,
                vm_steps: a.3,
                graph_relax_iters: a.4,
                plasticity_updates: a.5,
                actions_applied: b.0,
                births: b.1,
                pass_cap_hits: Some(b.2),
                passes: Some(b.3),
                decided_passes: Some(b.4),
                final_population: 0,
                extinction_tick: None,
            })
            .collect();
        let totals = accumulate_totals(&per_seed);
        let sum = |field: fn(&PerSeed) -> u64| per_seed.iter().map(field).sum::<u64>();
        prop_assert_eq!(totals.ticks, rows.iter().map(|(a, _)| a.0).sum::<u64>());
        prop_assert_eq!(totals.creature_ticks, sum(|r| r.creature_ticks));
        prop_assert_eq!(totals.mesh_hops, sum(|r| r.mesh_hops));
        prop_assert_eq!(totals.vm_steps, sum(|r| r.vm_steps));
        prop_assert_eq!(totals.graph_relax_iters, sum(|r| r.graph_relax_iters));
        prop_assert_eq!(totals.plasticity_updates, sum(|r| r.plasticity_updates));
        prop_assert_eq!(totals.actions_applied, sum(|r| r.actions_applied));
        prop_assert_eq!(totals.births, sum(|r| r.births));
        prop_assert_eq!(totals.pass_cap_hits, sum(|r| r.pass_cap_hits.unwrap()));
        prop_assert_eq!(totals.passes, sum(|r| r.passes.unwrap()));
        prop_assert_eq!(totals.decided_passes, sum(|r| r.decided_passes.unwrap()));
    }
}

/// The neighborhood read (T14.F12) is defined on every world of the goal
/// world set, sized from `NeighborhoodSizes`, pooled from its own rows over
/// all births, byte-identical across thread counts, and absent — never zero
/// — off the world set and in reports stored before it.
#[test]
fn world_set_neighborhood_read_is_defined_per_world_and_byte_identical_across_thread_counts() {
    let params = crate::bench::tests::small_world_set_params();
    let one = rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .build()
        .unwrap()
        .install(|| run_deterministic(&params).expect("a valid profile"));
    let two = rayon::ThreadPoolBuilder::new()
        .num_threads(2)
        .build()
        .unwrap()
        .install(|| run_deterministic(&params).expect("a valid profile").0);
    assert_eq!(
        serde_json::to_vec(&one.0).unwrap(),
        serde_json::to_vec(&two).unwrap()
    );
    let (report, timings) = one;
    assert_eq!(report.goal_indicators.cases.len(), 3);
    assert_eq!(timings.neighborhood_read_wall_clock_ms_per_seed.len(), 3);
    for (case, timing) in report
        .goal_indicators
        .cases
        .iter()
        .zip(&timings.neighborhood_read_wall_clock_ms_per_seed)
    {
        assert_eq!(timing.seed, case.case.seed);
        let read = case.neighborhood_read.defined().expect("a defined read");
        assert_eq!(read.version, "neighborhood-read-v1");
        assert_eq!(read.sample_size_requested, params.neighborhood.read_sample);
        assert_eq!(read.birth_trials, params.neighborhood.read_births);
        assert_eq!(
            u64::from(read.sample_size),
            read.population_size
                .min(u64::from(params.neighborhood.read_sample))
        );
        assert_eq!(read.genomes.len(), read.sample_size as usize);
        assert_eq!(
            read.births.births_total,
            read.sample_size * read.birth_trials
        );
        assert_eq!(
            read.genomes.iter().map(|row| row.changed).sum::<u32>(),
            read.births.any_events.changed
        );
        assert_eq!(
            read.changed_per_all_births,
            fraction_or_undefined(
                u64::from(read.births.any_events.changed),
                u64::from(read.births.births_total)
            )
        );
        assert!(read
            .genomes
            .windows(2)
            .all(|pair| pair[0].rank < pair[1].rank));
    }

    let goal = run_deterministic(&small_profile("goal"))
        .expect("a valid profile")
        .1;
    assert!(goal.neighborhood_read_wall_clock_ms_per_seed.is_empty());

    let mut historical = serde_json::to_value(&report).unwrap();
    historical["goal_indicators"]["cases"][0]
        .as_object_mut()
        .unwrap()
        .remove("neighborhood_read");
    let historical: Deterministic = serde_json::from_value(historical).unwrap();
    assert!(matches!(
        historical.goal_indicators.cases[0].neighborhood_read,
        Indicator::Undefined(ref reason) if reason == UNDEFINED
    ));
}
