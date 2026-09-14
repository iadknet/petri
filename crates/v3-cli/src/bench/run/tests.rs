use super::*;
use crate::bench::profiles::{
    goal_profile_params, goal_recipe_seeds, GOAL_RECIPES, SAMPLE_EVERY_TICKS,
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
    let run = run_one_seed(&config, 11, 0, true, None);
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
    params.ticks = 60;
    params.neighborhood = NeighborhoodSizes::default();
    params.drift = Default::default();
    let (report, _) = run_deterministic(&params).expect("a valid profile");
    let mut expected_cases = Vec::new();
    let mut pooled = Vec::new();
    for (index, case) in report.goal_indicators.cases.iter().enumerate() {
        let (_, config) = goal_case(&params, &GOAL_RECIPES[index]);
        let run = run_one_seed(&config, params.seeds[index], params.ticks, false, None);
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
    assert_eq!(drift.version, "drift-depth-v3");
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
    assert_eq!(drift.battery_version, "neighborhood-v1");
    assert_eq!(drift.mesh_version, "mesh-execution-v1");
    assert_eq!(drift.knockout_method, "static-successor-bypass-v1");
    assert_eq!(
        drift.executed_source,
        "battery hop records (mesh-execution-v1), node ids"
    );
    assert_eq!(
        drift.executed_refresh,
        "walk: depth 0 and every 10 generations; births: derived at each checkpoint"
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
        recruitment_paths_wall_clock_ms: None,
        neighborhood_evolved_wall_clock_ms_per_seed: Vec::new(),
        neighborhood_read_wall_clock_ms_per_seed: Vec::new(),
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

#[test]
fn recruitment_paths_is_goal_only_and_historical_absence_is_unmeasured() {
    let goal = build_report(&small_profile("goal"), "recruitment_paths-test").unwrap();
    let reading = goal
        .deterministic
        .goal_indicators
        .recruitment_paths
        .defined()
        .unwrap();
    assert_eq!(reading.version, "recruitment-paths-v1");
    assert_eq!(reading.total_proposals, reading.sizes.proposals());
    assert!(goal.environment.recruitment_paths_wall_clock_ms.unwrap() > 0.0);
    for name in ["gate", "sweep", "synthetic"] {
        let report = build_report(&small_profile(name), "recruitment_paths-test").unwrap();
        assert!(report
            .deterministic
            .goal_indicators
            .recruitment_paths
            .defined()
            .is_none());
        assert_eq!(report.environment.recruitment_paths_wall_clock_ms, None);
    }
    let mut old = serde_json::to_value(goal).unwrap();
    old["deterministic"]["goal_indicators"]
        .as_object_mut()
        .unwrap()
        .remove("recruitment_paths");
    old["environment"]
        .as_object_mut()
        .unwrap()
        .remove("recruitment_paths_wall_clock_ms");
    let old: Report = serde_json::from_value(old).unwrap();
    assert!(old
        .deterministic
        .goal_indicators
        .recruitment_paths
        .defined()
        .is_none());
    assert_eq!(old.environment.recruitment_paths_wall_clock_ms, None);
}

#[test]
fn recruitment_paths_is_once_per_world_set_and_reduced_results_are_deterministic() {
    let mut params = small_profile(GOAL_WORLD_SET);
    params.seeds = goal_recipe_seeds();
    let (report, timings) = run_deterministic(&params).unwrap();
    assert_eq!(report.goal_indicators.cases.len(), 3);
    let reading = report.goal_indicators.recruitment_paths.defined().unwrap();
    assert_eq!(
        reading.total_proposals,
        neighborhood::recruitment_paths::Sizes::TEST.proposals()
    );
    let encoded = serde_json::to_value(&report.goal_indicators).unwrap();
    for case in encoded["cases"].as_array().unwrap() {
        assert!(case.get("recruitment_paths").is_none());
    }
    let (again, elapsed) = timed_recruitment_paths(true);
    assert_eq!(
        serde_json::to_value(&again).unwrap(),
        serde_json::to_value(&report.goal_indicators.recruitment_paths).unwrap()
    );
    assert!(timings.recruitment_paths_wall_clock_ms.unwrap() > 0.0);
    assert!(elapsed.unwrap() > 0.0);
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
    let historical: MeshExecution = serde_json::from_value(historical).unwrap();
    assert_eq!(historical.backends, None);
    assert_eq!(historical.total_node_count, mesh.total_node_count);
    assert_eq!(mesh.version, "mesh-execution-v1");
    assert_eq!(mesh.executions_per_genome, 80);
    assert_eq!(mesh.snapshot_route_probes, 48);
    assert_eq!(mesh.knockout_method, "static-successor-bypass-v1");
    assert!(
        matches!(gate_neighborhood.evolved, Indicator::Undefined(_)),
        "the evolved half never runs in the gate profile"
    );

    let (goal_det, _) = run_deterministic(&small_profile("goal")).expect("a valid profile");
    let Indicator::Defined(goal_neighborhood) = goal_det.goal_indicators.mutational_neighborhood
    else {
        panic!("the goal profile must define mutational_neighborhood");
    };
    assert!(
        matches!(goal_neighborhood.evolved, Indicator::Defined(_)),
        "the evolved half runs in the goal profile"
    );

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
            (0u64..1000, 0u64..1000, 0u64..1000, 0u64..1000, 0u64..1000, 0u64..1000, 0u64..1000, 0u64..1000),
            0..6usize,
        )
    ) {
        let per_seed: Vec<PerSeed> = rows
            .iter()
            .enumerate()
            .map(|(index, row)| PerSeed {
                tick_zero_connectivity: None,
                seed: index as u64,
                ticks: row.0,
                creature_ticks: row.1,
                mesh_hops: row.2,
                vm_steps: row.3,
                graph_relax_iters: row.4,
                plasticity_updates: row.5,
                actions_applied: row.6,
                births: row.7,
                final_population: 0,
                extinction_tick: None,
            })
            .collect();
        let totals = accumulate_totals(&per_seed);
        prop_assert_eq!(totals.ticks, rows.iter().map(|r| r.0).sum::<u64>());
        prop_assert_eq!(totals.creature_ticks, rows.iter().map(|r| r.1).sum::<u64>());
        prop_assert_eq!(totals.mesh_hops, rows.iter().map(|r| r.2).sum::<u64>());
        prop_assert_eq!(totals.vm_steps, rows.iter().map(|r| r.3).sum::<u64>());
        prop_assert_eq!(totals.graph_relax_iters, rows.iter().map(|r| r.4).sum::<u64>());
        prop_assert_eq!(totals.plasticity_updates, rows.iter().map(|r| r.5).sum::<u64>());
        prop_assert_eq!(totals.actions_applied, rows.iter().map(|r| r.6).sum::<u64>());
        prop_assert_eq!(totals.births, rows.iter().map(|r| r.7).sum::<u64>());
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
