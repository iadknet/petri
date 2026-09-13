use super::*;
use crate::bench::tests::small_profile;
use proptest::prelude::*;
use v3_core::contracts::{Direction, WorldAction};
use v3_core::simulation::seed_simulation;

fn companion_population() -> [v3_core::creature::genome::CreatureGenome; 3] {
    use v3_core::contracts::NodeId;
    use v3_core::creature::genome::cgp::{
        CgpGraphBackendDef, ComputeNode, ComputeNodeKind, GraphEdge, GraphSource,
    };
    use v3_core::creature::genome::{
        BackendDef, CreatureGenome, HebbianRule, NodeGenome, PlasticityConfig, VmBackendDef,
        VmInstruction,
    };
    let make = |backend_def| CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![],
            backend_def,
            targets: vec![],
        }],
    };
    let reader_writer = make(BackendDef::Vm(VmBackendDef {
        register_count: 1,
        constants: vec![],
        program: vec![
            VmInstruction::LoadSlotImm {
                dst: 0,
                slot_idx: 0,
            },
            VmInstruction::ClearSlot { slot_idx: 0 },
        ],
    }));
    let stateful_plastic = make(BackendDef::Graph(CgpGraphBackendDef {
        compute_nodes: vec![ComputeNode {
            kind: ComputeNodeKind::DecayIntegrator(0.5),
            inputs: vec![GraphEdge {
                source: GraphSource::SharedMemory {
                    slot: 0,
                    previous: true,
                },
                weight: 1.0,
            }],
            plasticity: Some(PlasticityConfig {
                rule: HebbianRule::Classic,
                learning_rate: 0.1,
                weight_clamp: 1.0,
                lamarckian: false,
                modulation: None,
            }),
        }],
        birth_weights: None,
        output_sinks: vec![],
        action_bank: vec![],
        execute_gate: v3_core::creature::genome::cgp::ExecuteGate { inputs: vec![] },
    }));
    let mut inert = make(BackendDef::Vm(VmBackendDef {
        register_count: 1,
        constants: vec![],
        program: vec![VmInstruction::Halt],
    }));
    let mut unreachable = stateful_plastic.nodes[0].clone();
    unreachable.node_id = NodeId::new(1);
    inert.nodes.push(unreachable);
    [reader_writer, stateful_plastic, inert]
}

#[test]
fn companion_census_counts_overlapping_reachable_carriers_and_empty_population() {
    let population = companion_population();
    let census = structural_companions_census(11, population.iter());
    assert_eq!(
        census,
        StructuralCompanionsSeed {
            seed: 11,
            final_creature_count: 3,
            reads_shared_memory: 2,
            writes_shared_memory: 1,
            has_stateful_compute_node: 1,
            has_plasticity: 1,
        }
    );
    assert_eq!(
        structural_companions_census(11, std::iter::empty()),
        StructuralCompanionsSeed {
            seed: 11,
            ..Default::default()
        }
    );
}

proptest! {
    #[test]
    fn companion_census_counts_each_flag_independently_of_population_order(
        carriers in prop::collection::vec(0usize..3, 0..24),
    ) {
        let population = companion_population();
        let census = structural_companions_census(11, carriers.iter().map(|&i| &population[i]));
        prop_assert_eq!(&census, &structural_companions_census(11, carriers.iter().rev().map(|&i| &population[i])));
        prop_assert_eq!(census.final_creature_count, carriers.len() as u64);
        prop_assert_eq!(census.reads_shared_memory, carriers.iter().filter(|&&i| i != 2).count() as u64);
        prop_assert_eq!(census.writes_shared_memory, carriers.iter().filter(|&&i| i == 0).count() as u64);
        prop_assert_eq!(census.has_stateful_compute_node, carriers.iter().filter(|&&i| i == 1).count() as u64);
        prop_assert_eq!(census.has_plasticity, carriers.iter().filter(|&&i| i == 1).count() as u64);
    }
}

proptest! {
    #[test]
    fn births_rate_uses_complete_profile_totals(births in 0u64..1_000_000, ticks in 0u64..100_000) {
        for ticks in [0, ticks.max(1)] {
            let inputs = GoalIndicatorInputs {
                population_persistence_per_seed: vec![],
                lineage_diversity_per_seed: vec![],
                memory_sensitivity_per_seed: vec![],
                structural_companions_per_seed: vec![],
                temporal_memory_sensitivity_per_seed: vec![],
                evolved_neighborhood_per_seed: vec![],
                pooled_complexities: vec![],
                neighborhood_founder: None,
                drift_depth: Indicator::Undefined("test profile".into()),
                case_observations: vec![],
            };
            let totals = Totals { births, ticks, ..Totals::default() };
            let indicators = assemble_goal_indicators(&small_profile("synthetic"), &totals, inputs);
            let expected = if ticks == 0 { 0.0 } else { births as f64 * 100.0 / ticks as f64 };
            let actual: f64 = indicators.births_per_100_ticks.parse().unwrap();
            prop_assert!((actual - expected).abs() <= 0.000_001);
        }
    }
}

/// A recruitment reading over no lineages at all, for checkpoint
/// conversions that only exercise the mesh and birth fields.
fn empty_recruitment(depth: u64) -> neighborhood::recruitment::RecruitmentCheckpoint {
    neighborhood::recruitment::RecruitmentTracker::new(0).checkpoint(depth)
}

#[test]
fn drift_checkpoint_reports_recruitment_and_opportunities_and_still_loads_older_reports() {
    use neighborhood::recruitment::{BirthObservation, RecruitmentTracker};
    use v3_core::contracts::NodeId;
    use v3_core::mutation::{
        MutationDomain, MutationEventOutcome, MutationEventRecord, MutationOperator,
        MutationSkipReason, MutationSummary, TargetReachability,
    };

    let founder = founder_genome(v3_core::config::FounderProfile::V3Alpha1);
    let mut grown = founder.nodes.clone();
    let mut detour = grown[0].clone();
    detour.node_id = NodeId::new(900);
    grown.push(detour);

    // One birth that applied an AddNode event naming the founder's entry
    // node after discarding an operator that had already selected it, and
    // one event that selected a node with no applicable site. The two
    // splits therefore differ: two operators were discarded, one event
    // exhausted its domain.
    let mut summary = MutationSummary::zero();
    summary.record_attempt(MutationDomain::Topology, MutationOperator::TopologyAddNode);
    summary.record_applied(MutationDomain::Topology, MutationOperator::TopologyAddNode);
    summary.record_reachability(TargetReachability::Reachable);
    summary.record_event(MutationEventRecord {
        domain: MutationDomain::Topology,
        operator: Some(MutationOperator::TopologyAddNode),
        target: Some(founder.nodes[0].node_id),
        outcome: MutationEventOutcome::Applied(TargetReachability::Reachable),
        discarded: vec![(
            MutationOperator::TopologyRemoveRouteTarget,
            Some(founder.nodes[0].node_id),
        )],
    });
    summary.record_domain_skip(
        MutationDomain::Graph,
        MutationSkipReason::NoApplicableTarget,
    );
    summary.record_event(MutationEventRecord {
        domain: MutationDomain::Graph,
        operator: None,
        target: Some(founder.nodes[0].node_id),
        outcome: MutationEventOutcome::Skipped(MutationSkipReason::NoApplicableTarget),
        discarded: vec![(
            MutationOperator::GraphAddGraphEdge,
            Some(founder.nodes[0].node_id),
        )],
    });

    let mut tracker = RecruitmentTracker::new(1);
    tracker.seed_founder(0, &founder.nodes);
    tracker.record_birth(BirthObservation {
        lineage: 0,
        depth: 1,
        after: &grown,
        summary: &summary,
    });
    let executed = std::collections::BTreeSet::from([NodeId::new(900)]);
    tracker.record_reading(0, 1, &executed, Some(&executed));
    let reading = tracker.checkpoint(1);

    let report = drift_checkpoint(
        neighborhood::drift::Checkpoint {
            depth: 1,
            ..neighborhood::drift::Checkpoint::default()
        },
        &reading,
    );
    let recruitment = report.recruitment.clone().expect("a v3 recruitment block");
    assert_eq!(recruitment.cohort.created, 1);
    assert_eq!(recruitment.cohort.present, 1);
    assert_eq!(recruitment.cohort.contributing, 1);
    assert_eq!(recruitment.cohort.present_fraction, "1.000000");
    assert_eq!(recruitment.cohort.contributing_fraction, "1.000000");
    assert_eq!(recruitment.founders.created, founder.nodes.len() as u64);
    assert_eq!(recruitment.founders.contributing, 0);
    assert_eq!(recruitment.lineages.len(), 1);
    assert_eq!(recruitment.retention, None);
    let dispatch = recruitment
        .time_to_first
        .iter()
        .find(|row| row.fact == "dispatch")
        .expect("a dispatch row");
    assert_eq!(dispatch.reached, 1);
    assert_eq!(dispatch.median_generations, Some(0));
    assert_eq!(dispatch.reached_fraction, "1.000000");
    let never = recruitment
        .time_to_first
        .iter()
        .find(|row| row.fact == "selection")
        .expect("a selection row");
    assert_eq!(never.reached, 0);
    assert_eq!(never.censored_present, 1);
    assert_eq!(never.median_generations, None);

    let opportunities = report
        .opportunities
        .clone()
        .expect("a v3 opportunity block");
    assert_eq!(opportunities.births, 1);
    assert_eq!(opportunities.attempted, 2);
    assert_eq!(opportunities.applied, 1);
    assert_eq!(opportunities.applied_fraction, "0.500000");
    assert_eq!(
        opportunities.selected_inapplicable_by_domain,
        BTreeMap::from([("Graph".to_string(), 1)])
    );
    assert!(opportunities.no_eligible_node_by_domain.is_empty());
    // Discarded operators are reported by operator, not by domain, and an
    // operator discarded by an event that later applied counts too.
    assert_eq!(
        opportunities.discarded_selected_inapplicable_by_operator,
        BTreeMap::from([
            ("Graph.AddGraphEdge".to_string(), 1),
            ("Topology.RemoveRouteTarget".to_string(), 1)
        ])
    );
    assert!(opportunities
        .discarded_no_eligible_node_by_operator
        .is_empty());
    assert_eq!(opportunities.lineages.len(), 1);
    // The per-lineage row carries the discarded-operator count, which the
    // event-level per-domain count (1) undercounts, and the rows sum to
    // the pooled map.
    assert_eq!(opportunities.lineages[0].discarded_selected_inapplicable, 2);
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
    assert_eq!(
        opportunities.operators,
        vec![OperatorOpportunityRow {
            operator: "Topology.AddNode".to_string(),
            attempted: 1,
            applied: 1,
            applied_fraction: "1.000000".to_string(),
            skipped_by_reason: BTreeMap::new(),
        }]
    );

    // A drift-depth-v2 report carries neither block, and loads as absent.
    let mut historical = serde_json::to_value(&report).unwrap();
    let object = historical.as_object_mut().unwrap();
    object.remove("recruitment");
    object.remove("opportunities");
    let historical: DriftDepthCheckpoint = serde_json::from_value(historical).unwrap();
    assert_eq!(historical.recruitment, None);
    assert_eq!(historical.opportunities, None);
    assert_eq!(historical.depth, 1);
}

#[test]
fn drift_checkpoint_uses_pooled_lineage_execution_and_all_birth_denominators() {
    let row = neighborhood::drift::Checkpoint {
        depth: 250,
        mesh: neighborhood::drift::MeshTotals {
            backends: neighborhood::mesh_execution::MeshBackendCounts {
                graph: neighborhood::mesh_execution::BackendNodeCounts {
                    total: 8,
                    executed: 3,
                    contributing: 1,
                },
                vm: neighborhood::mesh_execution::BackendNodeCounts {
                    total: 12,
                    executed: 4,
                    contributing: 3,
                },
            },
            lineages: 4,
            total_nodes: 20,
            reachable_nodes: 10,
            executed_nodes: 7,
            knockout_nodes: 3,
            route_varying_lineages: 1,
            hop_cap_hits: 8,
        },
        births: BirthResult {
            births_total: 20,
            zero_event_births: 10,
            any_events: Tally {
                trials: 10,
                silent: 5,
                changed: 3,
                dead: 2,
                ..Tally::default()
            },
            ..BirthResult::default()
        },
    };
    let report = drift_checkpoint(row, &empty_recruitment(250));
    let backends = report.backends.expect("measured backend totals");
    assert_eq!(
        (
            backends.graph.total,
            backends.graph.executed,
            backends.graph.contributing
        ),
        (8, 3, 1)
    );
    assert_eq!(
        (
            backends.vm.total,
            backends.vm.executed,
            backends.vm.contributing
        ),
        (12, 4, 3)
    );
    let mut historical = serde_json::to_value(&report).unwrap();
    historical.as_object_mut().unwrap().remove("backends");
    let historical: DriftDepthCheckpoint = serde_json::from_value(historical).unwrap();
    assert_eq!(historical.backends, None);
    assert_eq!(historical.total_nodes, 20);
    assert_eq!(report.depth, 250);
    assert_eq!(report.mean_total_nodes, "5.000000");
    assert_eq!(report.mean_reachable_nodes, "2.500000");
    assert_eq!(report.mean_executed_nodes, "1.750000");
    assert_eq!(report.mean_knockout_nodes, "0.750000");
    assert_eq!(report.route_varying_fraction, "0.250000");
    assert_eq!(report.battery_executions, 320);
    assert_eq!(report.hop_cap_fraction, "0.025000");
    assert_eq!(report.silent_per_all_births, "0.250000");
    assert_eq!(report.changed_per_all_births, "0.150000");
    assert_eq!(report.dead_per_all_births, "0.100000");
    assert_eq!(report.births.any_events.changed_fraction, "0.300000");
    let empty = drift_checkpoint(
        neighborhood::drift::Checkpoint::default(),
        &empty_recruitment(0),
    );
    assert_eq!(empty.mean_total_nodes, UNDEFINED);
    assert_eq!(empty.hop_cap_fraction, UNDEFINED);
    assert_eq!(empty.changed_per_all_births, UNDEFINED);
}

#[test]
fn lineage_diversity_handles_empty_one_balanced_and_unequal_populations() {
    assert_eq!(
        lineage_diversity(11, []).shannon_entropy_nats,
        UNDEFINED,
        "empty populations have undefined entropy"
    );
    assert_eq!(lineage_diversity(11, [7]).shannon_entropy_nats, "0.000000");
    assert_eq!(
        lineage_diversity(11, [3, 9]).shannon_entropy_nats,
        "0.693147"
    );
    assert_eq!(
        lineage_diversity(11, [3, 3, 3, 9]).shannon_entropy_nats,
        "0.562335"
    );
}

#[test]
fn temporal_report_keeps_substrate_counts_separate() {
    use v3_core::contracts::NodeId;
    use v3_core::creature::genome::cgp::{
        ActionSlot, ActionSlotBehavior, CgpGraphBackendDef, ComputeNode, ComputeNodeKind,
        ExecuteGate, GraphEdge, GraphSource, WorldActionKind,
    };
    use v3_core::creature::genome::{BackendDef, CreatureGenome, NodeGenome};
    let mut config = SimulationConfig::default();
    config.population.initial_creatures = 1;
    let mut sim = seed_simulation(config, 11);
    let id = sim.creatures.keys().next().unwrap();
    let edge = GraphEdge {
        source: GraphSource::SharedMemory {
            slot: 0,
            previous: true,
        },
        weight: 1.0,
    };
    sim.creatures[id].genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![],
            targets: vec![],
            backend_def: BackendDef::Graph(CgpGraphBackendDef {
                birth_weights: None,
                compute_nodes: vec![ComputeNode {
                    kind: ComputeNodeKind::Constant(0.0),
                    inputs: vec![],
                    plasticity: None,
                }],
                output_sinks: vec![],
                action_bank: vec![ActionSlot {
                    behavior: ActionSlotBehavior::Emit(WorldActionKind::Eat),
                    gate_inputs: vec![edge],
                    param_inputs: vec![],
                }],
                execute_gate: ExecuteGate { inputs: vec![edge] },
            }),
        }],
    };
    sim.creatures[id].prev_shared_memory[0] = 1.0;
    let report = temporal_memory_sensitivity(11, &sim);
    assert_eq!(report.previous_slots.final_creature_count, 1);
    assert_eq!(report.previous_slots.different_from_zeroed_count, 1);
    assert_eq!(report.previous_slots.different_from_scrambled_count, 1);
    for component in [report.persisted_outputs, report.operator_state] {
        assert_eq!(component.final_creature_count, 1);
        assert_eq!(component.different_from_either_count, 0);
    }
}

#[test]
fn memory_sensitivity_counts_full_action_differences_and_union_once() {
    let mut config = SimulationConfig::default();
    config.world.width = 16;
    config.world.height = 16;
    config.population.initial_creatures = 3;
    let sim = seed_simulation(config, 11);
    let ids: Vec<_> = sim.creatures.keys().collect();
    let intact = vec![WorldAction::Reproduce {
        direction: Direction::N,
        energy_transfer: 1.0,
    }];
    let payload_changed = vec![WorldAction::Reproduce {
        direction: Direction::N,
        energy_transfer: 2.0,
    }];
    let direction_changed = vec![WorldAction::Reproduce {
        direction: Direction::E,
        energy_transfer: 1.0,
    }];
    let readings = vec![
        FinalActionObservation {
            creature_id: ids[0],
            intact: intact.clone(),
            zeroed: payload_changed.clone(),
            scrambled: intact.clone(),
        },
        FinalActionObservation {
            creature_id: ids[1],
            intact: intact.clone(),
            zeroed: intact.clone(),
            scrambled: direction_changed.clone(),
        },
        FinalActionObservation {
            creature_id: ids[2],
            intact: intact.clone(),
            zeroed: payload_changed,
            scrambled: direction_changed,
        },
    ];

    let indicator = memory_sensitivity(11, &readings);
    assert_eq!(indicator.different_from_zeroed_count, 2);
    assert_eq!(indicator.different_from_scrambled_count, 2);
    assert_eq!(indicator.different_from_either_count, 3);
    assert_eq!(indicator.different_from_either_fraction, "1.000000");
    let empty = memory_sensitivity(11, &[]);
    assert_eq!(empty.final_creature_count, 0);
    assert_eq!(empty.different_from_either_fraction, UNDEFINED);
}

proptest! {
    #[test]
    fn lineage_diversity_count_is_the_number_of_distinct_surviving_lineages(
        lineage_ids in proptest::collection::vec(0_u32..32, 0..128),
    ) {
        let result = lineage_diversity(11, lineage_ids.iter().copied());
        let distinct: std::collections::BTreeSet<_> = lineage_ids.iter().copied().collect();
        prop_assert_eq!(result.surviving_founder_clade_count, distinct.len() as u64);
        if lineage_ids.is_empty() {
            prop_assert_eq!(result.shannon_entropy_nats, UNDEFINED);
        } else {
            prop_assert_ne!(result.shannon_entropy_nats, UNDEFINED);
        }
    }

    #[test]
    fn memory_sensitivity_union_and_fractions_match_generated_differences(
        differences in proptest::collection::vec((any::<bool>(), any::<bool>()), 0..64),
    ) {
        let mut config = SimulationConfig::default();
        config.world.width = 16;
        config.world.height = 16;
        config.population.initial_creatures = 1;
        let sim = seed_simulation(config, 11);
        let creature_id = sim.creatures.keys().next().expect("one founder");
        let intact = vec![WorldAction::NoOp];
        let changed = vec![WorldAction::Move(Direction::N)];
        let observations: Vec<_> = differences
            .iter()
            .map(|&(zeroed_differs, scrambled_differs)| FinalActionObservation {
                creature_id,
                intact: intact.clone(),
                zeroed: if zeroed_differs { changed.clone() } else { intact.clone() },
                scrambled: if scrambled_differs { changed.clone() } else { intact.clone() },
            })
            .collect();

        let result = memory_sensitivity(11, &observations);
        let zeroed_count = differences.iter().filter(|(zeroed, _)| *zeroed).count() as u64;
        let scrambled_count = differences.iter().filter(|(_, scrambled)| *scrambled).count() as u64;
        let both_count = differences.iter().filter(|(zeroed, scrambled)| *zeroed && *scrambled).count() as u64;
        let union_count = zeroed_count + scrambled_count - both_count;

        prop_assert_eq!(result.different_from_zeroed_count, zeroed_count);
        prop_assert_eq!(result.different_from_scrambled_count, scrambled_count);
        prop_assert_eq!(result.different_from_either_count, union_count);
        prop_assert!(union_count >= zeroed_count && union_count >= scrambled_count);
        if differences.is_empty() {
            prop_assert_eq!(result.different_from_zeroed_fraction, UNDEFINED);
            prop_assert_eq!(result.different_from_scrambled_fraction, UNDEFINED);
            prop_assert_eq!(result.different_from_either_fraction, UNDEFINED);
        } else {
            let total = differences.len() as f64;
            prop_assert_eq!(result.different_from_zeroed_fraction, six(zeroed_count as f64 / total));
            prop_assert_eq!(result.different_from_scrambled_fraction, six(scrambled_count as f64 / total));
            prop_assert_eq!(result.different_from_either_fraction, six(union_count as f64 / total));
        }
    }
}

// ── Mutational neighborhood (T11.F01) ───────────────────────────────

#[test]
fn neighborhood_tally_conversion_computes_fractions_against_applied_not_trials() {
    let tally = Tally {
        trials: 10,
        skipped: 2,
        silent: 3,
        changed: 4,
        dead: 1,
        changed_only_in_sequences: 1,
        differing_executions_total: 40,
        total_executions_total: 400,
    };
    let report = to_neighborhood_tally(&tally);
    assert_eq!(report.applied, 8);
    assert_eq!(report.silent_fraction, "0.375000");
    assert_eq!(report.changed_fraction, "0.500000");
    assert_eq!(report.dead_fraction, "0.125000");
}

#[test]
fn neighborhood_tally_conversion_reports_undefined_fractions_when_nothing_applied() {
    let tally = Tally {
        trials: 5,
        skipped: 5,
        ..Tally::default()
    };
    let report = to_neighborhood_tally(&tally);
    assert_eq!(report.applied, 0);
    assert_eq!(report.silent_fraction, UNDEFINED);
    assert_eq!(report.changed_fraction, UNDEFINED);
    assert_eq!(report.dead_fraction, UNDEFINED);
}

#[test]
fn neighborhood_battery_execution_count_is_48_snapshots_plus_32_sequence_ticks() {
    assert_eq!(neighborhood_battery_execution_count(), 80);
}

#[test]
fn to_neighborhood_operator_rows_preserves_every_row_in_order() {
    let rows = vec![
        OperatorRow {
            family: "vm",
            operator: "Example".to_string(),
            tally: Tally::default(),
        },
        OperatorRow {
            family: "graph",
            operator: "Other".to_string(),
            tally: Tally::default(),
        },
    ];
    let converted = to_neighborhood_operator_rows(&rows);
    assert_eq!(converted.len(), 2);
    assert_eq!(converted[0].family, "vm");
    assert_eq!(converted[0].operator, "Example");
    assert_eq!(converted[1].family, "graph");
    assert_eq!(converted[1].operator, "Other");
}

/// `evolved_neighborhood_for_seed` seeds sampled genome `i` (in rank
/// order) by `EVOLVED_SEED_MULTIPLIER * (i + 1)`. This reconstructs the
/// same per-genome reading independently, via the same public
/// `evaluate_genome` seam, and checks it against the function's own
/// output: a `+`, `/`, or a `*` swapped for the `+` inside
/// `(genome_index + 1)` would draw a different seed offset and, with
/// overwhelming likelihood, a different reading.
#[test]
fn evolved_neighborhood_for_seed_offsets_each_sampled_genome_by_its_rank_order() {
    let mut config = SimulationConfig::default();
    config.world.width = 16;
    config.world.height = 16;
    config.population.initial_creatures = 3;
    let sim = seed_simulation(config.clone(), 11);
    let battery = Battery::generate(config.world.food.types.len());
    let context = EvalContext::from_config(&config);
    // Larger than `NeighborhoodSizes::default()`'s fixture sizes: at the
    // production ~8.8% mutated birth rate and near-deterministic
    // per-operator classes, a handful of trials can coincidentally match
    // under a wrong seed offset. 60 births and 3 operator trials make
    // that implausible while staying well under a second for 3 genomes.
    let sizes = NeighborhoodSizes {
        evolved_operator_trials: 3,
        evolved_births: 60,
        ..NeighborhoodSizes::default()
    };

    let actual =
        evolved_neighborhood_for_seed(11, &sim, &battery, &config.mutation, &context, sizes);

    let mut creature_ids: Vec<_> = sim.creatures.keys().collect();
    creature_ids.sort();
    assert_eq!(
        actual.sampled_genomes.len(),
        creature_ids.len(),
        "a population below the sample size takes every rank"
    );

    for (genome_index, &creature_id) in creature_ids.iter().enumerate() {
        let creature = &sim.creatures[creature_id];
        let seed_offset =
            v3_core::neighborhood::EVOLVED_SEED_MULTIPLIER * (genome_index as u64 + 1);
        let expected = evaluate_genome(
            &creature.genome,
            &battery,
            &config.mutation,
            &context,
            sizes.evolved_operator_trials,
            sizes.evolved_births,
            seed_offset,
        );
        assert_eq!(
            actual.sampled_genomes[genome_index].operator_rows,
            to_neighborhood_operator_rows(&expected.operator_rows),
            "genome_index {genome_index}"
        );
        assert_eq!(
            actual.sampled_genomes[genome_index].births,
            to_neighborhood_births(&expected.births),
            "genome_index {genome_index}"
        );
    }
}

proptest::proptest! {
    #[test]
    fn requested_birth_histogram_roundtrips(
        histogram in proptest::collection::btree_map(0u32..10, 1u32..100, 0..10),
    ) {
        let result = BirthResult {
            births_total: histogram.values().sum(),
            by_requested_events: histogram,
            ..BirthResult::default()
        };
        let report = to_neighborhood_births(&result);
        let encoded = serde_json::to_value(&report).unwrap();
        let decoded: NeighborhoodBirths = serde_json::from_value(encoded.clone()).unwrap();
        proptest::prop_assert_eq!(&decoded, &report);
        let wrapped = Indicator::Defined(report.clone());
        let wrapped_encoded = serde_json::to_string(&wrapped).unwrap();
        let decoded: Indicator<NeighborhoodBirths> = serde_json::from_str(&wrapped_encoded).unwrap();
        proptest::prop_assert_eq!(decoded, wrapped);
        let mut historical = encoded;
        historical.as_object_mut().unwrap().remove("by_requested_events");
        let decoded: NeighborhoodBirths = serde_json::from_value(historical).unwrap();
        proptest::prop_assert!(decoded.by_requested_events.is_empty());
    }
}

#[test]
fn merge_birth_results_sums_totals_and_merges_matching_buckets() {
    let mut a = BirthResult {
        births_total: 10,
        zero_event_births: 4,
        ..BirthResult::default()
    };
    a.any_events = Tally {
        trials: 6,
        silent: 2,
        changed: 3,
        dead: 1,
        ..Tally::default()
    };
    a.by_events.insert(1, a.any_events);
    a.by_requested_events.insert(0, 4);
    a.by_requested_events.insert(2, 6);

    let mut b = BirthResult {
        births_total: 5,
        zero_event_births: 1,
        ..BirthResult::default()
    };
    b.any_events = Tally {
        trials: 4,
        silent: 1,
        changed: 3,
        ..Tally::default()
    };
    b.by_events.insert(1, b.any_events);
    b.by_requested_events.insert(0, 1);
    b.by_requested_events.insert(2, 4);

    let merged = a.merge(&b);
    assert_eq!(merged.births_total, 15);
    assert_eq!(merged.zero_event_births, 5);
    assert_eq!(merged.any_events.trials, 10);
    assert_eq!(merged.by_events[&1].trials, 10);
    assert_eq!(
        merged.by_requested_events,
        BTreeMap::from([(0, 5), (2, 10)])
    );
    assert_eq!(
        to_neighborhood_births(&merged).by_requested_events,
        vec![
            NeighborhoodRequestedBirthBucket {
                requested_events: 0,
                births: 5
            },
            NeighborhoodRequestedBirthBucket {
                requested_events: 2,
                births: 10
            },
        ]
    );
}

#[test]
fn evolved_depth_uses_whole_population_and_actual_sample_generation() {
    let mut config = SimulationConfig::default();
    config.world.width = 16;
    config.world.height = 16;
    config.population.initial_creatures = 25;
    let mut sim = seed_simulation(config.clone(), 11);
    let mut ids: Vec<_> = sim.creatures.keys().collect();
    ids.sort();
    let ranks = evolved_sample_ranks(ids.len());
    let high = u64::from(u32::MAX) + 12;
    for (rank, id) in ids.iter().enumerate() {
        sim.creatures.get_mut(*id).unwrap().generation = if rank == ranks[0] {
            high + 1
        } else if ranks.contains(&rank) {
            3
        } else {
            high
        };
    }
    let battery = Battery::generate(config.world.food.types.len());
    let reading = evolved_neighborhood_for_seed(
        11,
        &sim,
        &battery,
        &config.mutation,
        &EvalContext::from_config(&config),
        NeighborhoodSizes::default(),
    );
    assert_eq!(reading.final_population_size, 25);
    assert_eq!(
        reading.generation_distribution,
        Indicator::Defined(GenerationDistribution {
            median: high,
            max: high + 1
        })
    );
    for sample in reading.sampled_genomes {
        assert_eq!(
            sample.generation,
            Some(sim.creatures[ids[sample.rank as usize]].generation)
        );
        assert_eq!(
            sample.generation,
            Some(if sample.rank == ranks[0] as u64 {
                high + 1
            } else {
                3
            })
        );
        assert!(matches!(sample.mesh_execution, Indicator::Defined(_)));
    }
}

#[test]
fn population_depth_uses_u64_upper_median_and_extinction_is_undefined() {
    assert!(matches!(
        generation_distribution(vec![]),
        Indicator::Undefined(_)
    ));
    assert_eq!(
        generation_distribution(vec![9, 1, 5]),
        Indicator::Defined(GenerationDistribution { median: 5, max: 9 })
    );
    let large = u64::from(u32::MAX) + 9;
    assert_eq!(
        generation_distribution(vec![large, 0, 2, 1]),
        Indicator::Defined(GenerationDistribution {
            median: 2,
            max: large
        })
    );
}
