use super::*;
use crate::config::SimulationConfig;
use crate::contracts::{InputReference, NodeId, RouteTarget, WorldInputKey};
use crate::creature::founder::v3alpha1_founder_genome;
use crate::creature::genome::cgp::{CgpGraphBackendDef, ExecuteGate, OutputSink, OutputSinkKind};
use crate::creature::genome::{BackendDef, CreatureGenome, NodeGenome};
use crate::mutation::{
    MutationAddedNodeInputClass, MutationDomain, MutationEventOutcome, MutationOperator,
    MutationSkipReason, MutationSummary,
};
use rand::rngs::SmallRng;
use rand::SeedableRng;

fn rng(seed: u64) -> SmallRng {
    SmallRng::seed_from_u64(seed)
}

fn forced_topology_config() -> crate::config::MutationConfig {
    let mut config = SimulationConfig::default().mutation;
    config.mutation_probability = 1.0;
    config.per_birth_mutation_events_min = 1;
    config.per_birth_mutation_events_max = 1;
    config.mesh_layer_probability = 1.0;
    config
}

fn classify_expected_added_input_classes(
    input_refs: &[InputReference],
) -> Vec<MutationAddedNodeInputClass> {
    let mut classes = std::collections::BTreeSet::new();
    for input_ref in input_refs {
        classes.insert(MutationAddedNodeInputClass::from(input_ref));
    }
    if classes.is_empty() {
        vec![MutationAddedNodeInputClass::None]
    } else {
        classes.into_iter().collect()
    }
}

fn single_graph_genome_with_inputs(input_refs: Vec<InputReference>) -> CreatureGenome {
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs,
            backend_def: BackendDef::Graph(CgpGraphBackendDef {
                birth_weights: None,
                compute_nodes: Vec::new(),
                output_sinks: vec![OutputSink {
                    kind: OutputSinkKind::CustomOutput(0),
                    inputs: Vec::new(),
                }],
                action_bank: Vec::new(),
                execute_gate: ExecuteGate { inputs: Vec::new() },
            }),
            targets: Vec::new(),
        }],
    }
}

fn genome_has_non_default_food_input_ref(genome: &CreatureGenome) -> bool {
    genome.nodes.iter().any(|node| {
        node.input_refs.iter().any(|input_ref| match input_ref {
            InputReference::World(WorldInputKey::FoodHere { type_idx })
            | InputReference::World(WorldInputKey::NeighborFoodRing { type_idx })
            | InputReference::World(WorldInputKey::AreaFoodSummary { type_idx }) => {
                *type_idx != crate::config::OrdinaryFoodTypeId::default()
            }
            _ => false,
        })
    })
}

#[test]
fn engine_accounting_invariant_always_holds() {
    let mut config = SimulationConfig::default().mutation;
    config.mutation_probability = 1.0;
    config.per_birth_mutation_events_min = 1;
    config.per_birth_mutation_events_max = 5;

    for seed in 0u64..100 {
        let mut genome = v3alpha1_founder_genome();
        let mut r = rng(seed);
        let summary = MutationEngine::apply_mutations(&mut genome, &config, &[], &mut r);
        assert_eq!(
            summary.attempted_events,
            summary.applied_events + summary.skipped_events,
            "accounting invariant violated at seed {}",
            seed
        );
    }
}

#[test]
fn engine_with_probability_zero_returns_zero_summary() {
    let mut config = SimulationConfig::default().mutation;
    config.mutation_probability = 0.0;
    let mut genome = v3alpha1_founder_genome();
    let mut r = rng(42);
    let summary = MutationEngine::apply_mutations(&mut genome, &config, &[], &mut r);
    assert_eq!(summary.attempted_events, 0);
    assert_eq!(summary.applied_events, 0);
    assert_eq!(summary.skipped_events, 0);
}

#[test]
fn engine_records_added_input_classes_for_topology_add_node() {
    let config = forced_topology_config();

    for seed in 0u64..2_000 {
        let mut genome = v3alpha1_founder_genome();
        let mut r = rng(seed);
        let summary = MutationEngine::apply_mutations(&mut genome, &config, &[], &mut r);
        if summary
            .applied_by_operator
            .contains_key(&MutationOperator::TopologyAddNode)
        {
            let newborn = genome.nodes.last().expect("newborn node should exist");
            let expected = classify_expected_added_input_classes(&newborn.input_refs);
            let recorded = summary
                .added_node_input_classes_by_operator
                .get(&MutationOperator::TopologyAddNode)
                .expect("input classes should be recorded");
            for class in expected {
                assert_eq!(recorded.get(&class), Some(&1));
            }
            return;
        }
    }

    panic!("failed to observe Topology.AddNode within search budget");
}

#[test]
fn engine_with_food_type_count_can_introduce_non_default_food_input_refs() {
    let mut config = SimulationConfig::default().mutation;
    config.mutation_probability = 1.0;
    config.per_birth_mutation_events_min = 1;
    config.per_birth_mutation_events_max = 1;
    config.mesh_layer_probability = 0.0; // node-internal domains only

    let mut r = rng(0xF00D);
    let mut saw_non_default = false;
    for _ in 0..4_000 {
        let mut genome = v3alpha1_founder_genome();
        let summary = MutationEngine::apply_mutations_with_food_type_count(
            &mut genome,
            &config,
            &[],
            ParentExecuted::NONE,
            &mut r,
            3,
        );
        if summary
            .applied_by_domain
            .get(&MutationDomain::InputRef)
            .copied()
            .unwrap_or(0)
            > 0
            && genome_has_non_default_food_input_ref(&genome)
        {
            saw_non_default = true;
            break;
        }
    }

    assert!(
        saw_non_default,
        "MutationEngine should be able to introduce non-default food input refs when food_type_count > 1"
    );
}

#[test]
fn engine_records_added_input_classes_for_topology_splice_node() {
    let config = forced_topology_config();
    let base_genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![
            NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![],
                backend_def: BackendDef::Graph(CgpGraphBackendDef {
                    birth_weights: None,
                    compute_nodes: Vec::new(),
                    output_sinks: vec![OutputSink {
                        kind: OutputSinkKind::CustomOutput(0),
                        inputs: Vec::new(),
                    }],
                    action_bank: Vec::new(),
                    execute_gate: ExecuteGate { inputs: Vec::new() },
                }),
                targets: vec![RouteTarget {
                    target_id: NodeId::new(1),
                    slot: 0,
                    gate_bias: 0.0,
                }],
            },
            NodeGenome {
                node_id: NodeId::new(1),
                input_refs: vec![],
                backend_def: BackendDef::Graph(CgpGraphBackendDef {
                    birth_weights: None,
                    compute_nodes: Vec::new(),
                    output_sinks: vec![OutputSink {
                        kind: OutputSinkKind::CustomOutput(0),
                        inputs: Vec::new(),
                    }],
                    action_bank: Vec::new(),
                    execute_gate: ExecuteGate { inputs: Vec::new() },
                }),
                targets: vec![],
            },
        ],
    };

    for seed in 0u64..2_000 {
        let mut genome = base_genome.clone();
        let mut r = rng(seed);
        let summary = MutationEngine::apply_mutations(&mut genome, &config, &[], &mut r);
        if summary
            .applied_by_operator
            .contains_key(&MutationOperator::TopologySpliceNode)
        {
            let newborn = genome.nodes.last().expect("spliced node should exist");
            let expected = classify_expected_added_input_classes(&newborn.input_refs);
            let recorded = summary
                .added_node_input_classes_by_operator
                .get(&MutationOperator::TopologySpliceNode)
                .expect("input classes should be recorded");
            for class in expected {
                assert_eq!(recorded.get(&class), Some(&1));
            }
            return;
        }
    }

    panic!("failed to observe Topology.SpliceNode within search budget");
}

#[test]
fn engine_records_added_input_classes_for_graph_add_internal_graph_node() {
    let mut config = SimulationConfig::default().mutation;
    config.mutation_probability = 1.0;
    config.per_birth_mutation_events_min = 1;
    config.per_birth_mutation_events_max = 1;
    config.mesh_layer_probability = 0.0;

    let expected = vec![
        MutationAddedNodeInputClass::Food,
        MutationAddedNodeInputClass::Barrier,
    ];
    let base_genome = single_graph_genome_with_inputs(vec![
        InputReference::World(WorldInputKey::FoodHere {
            type_idx: crate::config::OrdinaryFoodTypeId::default(),
        }),
        InputReference::World(WorldInputKey::NeighborBarrierRing),
    ]);

    for seed in 0u64..5_000 {
        let mut genome = base_genome.clone();
        let mut r = rng(seed);
        let summary = MutationEngine::apply_mutations(&mut genome, &config, &[], &mut r);
        if summary
            .applied_by_operator
            .contains_key(&MutationOperator::GraphAddInternalGraphNode)
        {
            let recorded = summary
                .added_node_input_classes_by_operator
                .get(&MutationOperator::GraphAddInternalGraphNode)
                .expect("input classes should be recorded");
            for class in expected {
                assert_eq!(recorded.get(&class), Some(&1));
            }
            return;
        }
    }

    panic!("failed to observe Graph.AddInternalGraphNode within search budget");
}

#[test]
fn engine_records_added_world_inputs_for_graph_add_internal_graph_node() {
    let mut config = SimulationConfig::default().mutation;
    config.mutation_probability = 1.0;
    config.per_birth_mutation_events_min = 1;
    config.per_birth_mutation_events_max = 1;
    config.mesh_layer_probability = 0.0;

    let base_genome = single_graph_genome_with_inputs(vec![
        InputReference::World(WorldInputKey::AreaFoodSummary {
            type_idx: crate::config::OrdinaryFoodTypeId::default(),
        }),
        InputReference::World(WorldInputKey::NeighborBarrierRing),
    ]);

    for seed in 0u64..5_000 {
        let mut genome = base_genome.clone();
        let mut r = rng(seed);
        let summary = MutationEngine::apply_mutations(&mut genome, &config, &[], &mut r);
        if summary
            .applied_by_operator
            .contains_key(&MutationOperator::GraphAddInternalGraphNode)
        {
            let recorded = summary
                .added_node_world_inputs_by_operator
                .get(&MutationOperator::GraphAddInternalGraphNode)
                .expect("world inputs should be recorded");
            assert_eq!(
                recorded.get(&WorldInputKey::AreaFoodSummary {
                    type_idx: crate::config::OrdinaryFoodTypeId::default()
                }),
                Some(&1)
            );
            assert_eq!(recorded.get(&WorldInputKey::NeighborBarrierRing), Some(&1));
            return;
        }
    }

    panic!("failed to observe Graph.AddInternalGraphNode within search budget");
}

#[test]
fn engine_with_probability_one_applies_events() {
    let mut config = SimulationConfig::default().mutation;
    config.mutation_probability = 1.0;
    config.per_birth_mutation_events_min = 3;
    config.per_birth_mutation_events_max = 3;
    let mut genome = v3alpha1_founder_genome();
    let mut r = rng(7);
    let summary = MutationEngine::apply_mutations(&mut genome, &config, &[], &mut r);
    assert_eq!(summary.attempted_events, 3, "must attempt exactly 3 events");
}

#[test]
fn engine_mutations_preserve_parseability() {
    let mut config = SimulationConfig::default().mutation;
    config.mutation_probability = 1.0;
    config.per_birth_mutation_events_min = 1;
    config.per_birth_mutation_events_max = 4;

    for seed in 0u64..50 {
        let mut genome = v3alpha1_founder_genome();
        let mut r = rng(seed);
        MutationEngine::apply_mutations(&mut genome, &config, &[], &mut r);
        assert!(
            ParseabilityGate::validate(&genome).is_ok(),
            "parseability violated at seed {}",
            seed
        );
    }
}

#[test]
fn engine_with_founder_genome_does_not_panic() {
    let mut config = SimulationConfig::default().mutation;
    config.mutation_probability = 1.0;
    config.per_birth_mutation_events_min = 1;
    config.per_birth_mutation_events_max = 4;

    let mut genome = v3alpha1_founder_genome();
    for seed in 0u64..1000 {
        let mut r = rng(seed);
        let summary = MutationEngine::apply_mutations(&mut genome, &config, &[], &mut r);
        assert_eq!(
            summary.attempted_events,
            summary.applied_events + summary.skipped_events
        );
    }
}

#[test]
fn diversity_test_mutated_clones_differ_from_original() {
    // Mutate 100 founder clones, verify 80%+ differ from original.
    let mut config = SimulationConfig::default().mutation;
    config.mutation_probability = 1.0;
    config.per_birth_mutation_events_min = 1;
    config.per_birth_mutation_events_max = 3;

    let original = v3alpha1_founder_genome();
    let mut differ_count = 0;
    for seed in 0u64..100 {
        let mut genome = original.clone();
        let mut r = rng(seed);
        MutationEngine::apply_mutations(&mut genome, &config, &[], &mut r);
        if genome != original {
            differ_count += 1;
        }
    }
    assert!(
        differ_count >= 80,
        "at least 80% must differ; got {}/100",
        differ_count
    );
}

#[test]
fn vm_variety_test_non_noop_instructions_after_mutations() {
    use crate::creature::genome::{BackendDef, VmInstruction};
    // After 1000 mutation passes on the same genome, non-Noop instructions must exist.
    let mut config = SimulationConfig::default().mutation;
    config.mutation_probability = 1.0;
    config.per_birth_mutation_events_min = 1;
    config.per_birth_mutation_events_max = 3;

    let mut genome = v3alpha1_founder_genome();
    for seed in 0u64..1000 {
        let mut r = rng(seed);
        MutationEngine::apply_mutations(&mut genome, &config, &[], &mut r);
    }
    let has_non_noop = genome.nodes.iter().any(|n| {
        if let BackendDef::Vm(ref vm) = n.backend_def {
            vm.program.iter().any(|i| !matches!(i, VmInstruction::Noop))
        } else {
            false
        }
    });
    assert!(
        has_non_noop,
        "after 1000 mutations, VM programs must contain non-Noop instructions"
    );
}

#[test]
fn stress_parseability_10000_chained_mutations() {
    // 100 copies x 100 generations of mutation, all must pass parseability.
    let mut config = SimulationConfig::default().mutation;
    config.mutation_probability = 1.0;
    config.per_birth_mutation_events_min = 1;
    config.per_birth_mutation_events_max = 3;

    for copy in 0u64..100 {
        let mut genome = v3alpha1_founder_genome();
        for gen in 0u64..100 {
            let mut r = rng(copy * 1000 + gen);
            MutationEngine::apply_mutations(&mut genome, &config, &[], &mut r);
        }
        assert!(
            ParseabilityGate::validate(&genome).is_ok(),
            "parseability violated for copy {} after 100 generations",
            copy
        );
    }
}

#[test]
fn engine_domain_and_operator_counters_reconcile_to_global_totals() {
    let mut config = SimulationConfig::default().mutation;
    config.mutation_probability = 1.0;
    config.per_birth_mutation_events_min = 5;
    config.per_birth_mutation_events_max = 5;

    let mut genome = v3alpha1_founder_genome();
    let mut rng = rng(123);
    let summary = MutationEngine::apply_mutations(&mut genome, &config, &[], &mut rng);

    let attempted_by_domain: u32 = summary.attempted_by_domain.values().sum();
    let applied_by_domain: u32 = summary.applied_by_domain.values().sum();
    let attempted_by_operator: u32 = summary.attempted_by_operator.values().sum();
    let applied_by_operator: u32 = summary.applied_by_operator.values().sum();
    let skipped_by_operator: u32 = summary.skipped_by_operator.values().sum();

    // Domain-level attempts include domain-skips (no applicable operator).
    assert_eq!(attempted_by_domain, summary.attempted_events);
    assert_eq!(applied_by_domain, summary.applied_events);
    // Operator-level attempts exclude domain-skips (no operator was selected).
    // Domain-level skips: events attempted where no operator could be selected
    // (e.g., restricted mode with no decreasing operator for a domain).
    assert!(
        attempted_by_operator <= summary.attempted_events,
        "operator attempts cannot exceed total attempts"
    );
    assert_eq!(applied_by_operator, summary.applied_events);
    assert!(
        skipped_by_operator <= summary.skipped_events,
        "operator skips cannot exceed total skips"
    );

    assert_eq!(
        summary.applied_semantic_noop_events + summary.applied_semantic_change_events,
        summary.applied_events
    );
}

#[test]
fn engine_attempted_counters_cover_all_domains_and_hit_each_domain_operator_surface() {
    let mut config = SimulationConfig::default().mutation;
    config.mutation_probability = 1.0;
    config.per_birth_mutation_events_min = 1;
    config.per_birth_mutation_events_max = 1;

    let mut domain_hits = std::collections::HashMap::<MutationDomain, u64>::new();
    let mut operator_hits = std::collections::HashMap::<MutationOperator, u64>::new();

    for seed in 0u64..20_000 {
        let mut genome = v3alpha1_founder_genome();
        let mut r = rng(seed);
        let summary = MutationEngine::apply_mutations(&mut genome, &config, &[], &mut r);
        for (domain, count) in summary.attempted_by_domain {
            *domain_hits.entry(domain).or_insert(0) += count as u64;
        }
        for (operator, count) in summary.attempted_by_operator {
            *operator_hits.entry(operator).or_insert(0) += count as u64;
        }
    }

    for domain in MutationDomain::all() {
        assert!(
            domain_hits.get(&domain).copied().unwrap_or(0) > 0,
            "expected attempted events for domain {:?}",
            domain
        );
    }

    let mut attempted_domains = std::collections::HashSet::new();
    for operator in operator_hits.keys().copied() {
        attempted_domains.insert(operator.domain());
    }
    for domain in MutationDomain::all() {
        assert!(
            attempted_domains.contains(&domain),
            "expected at least one attempted operator in domain {:?}",
            domain
        );
    }
}

#[test]
fn engine_live_operators_record_semantic_changes() {
    let mut config = SimulationConfig::default().mutation;
    config.mutation_probability = 1.0;
    config.per_birth_mutation_events_min = 1;
    config.per_birth_mutation_events_max = 1;
    for seed in 0..100 {
        let mut genome = v3alpha1_founder_genome();
        let summary = MutationEngine::apply_mutations(&mut genome, &config, &[], &mut rng(seed));
        assert_eq!(summary.applied_semantic_noop_events, 0);
        assert_eq!(
            summary.applied_semantic_change_events,
            summary.applied_events
        );
    }
}

#[test]
fn engine_mesh_layer_fires_less_than_node_internal() {
    let mut config = SimulationConfig::default().mutation;
    config.mutation_probability = 1.0;
    config.per_birth_mutation_events_min = 1;
    config.per_birth_mutation_events_max = 1;
    // mesh_layer_probability defaults to 0.2

    let mut topology_attempts: u64 = 0;
    let mut total_attempts: u64 = 0;
    for seed in 0u64..20_000 {
        let mut genome = v3alpha1_founder_genome();
        let mut r = rng(seed);
        let summary = MutationEngine::apply_mutations(&mut genome, &config, &[], &mut r);
        topology_attempts += summary
            .attempted_by_domain
            .get(&MutationDomain::Topology)
            .copied()
            .unwrap_or(0) as u64;
        total_attempts += summary.attempted_events as u64;
    }
    let topology_ratio = topology_attempts as f64 / total_attempts as f64;
    assert!(
        topology_ratio < 0.30,
        "topology should be ~20% of attempts; got {:.1}%",
        topology_ratio * 100.0,
    );
    assert!(
        topology_ratio > 0.10,
        "topology should be ~20% of attempts; got {:.1}%",
        topology_ratio * 100.0,
    );
}

#[test]
fn engine_mesh_layer_probability_zero_never_selects_topology() {
    let mut config = SimulationConfig::default().mutation;
    config.mutation_probability = 1.0;
    config.per_birth_mutation_events_min = 1;
    config.per_birth_mutation_events_max = 1;
    config.mesh_layer_probability = 0.0;

    for seed in 0u64..1000 {
        let mut genome = v3alpha1_founder_genome();
        let mut r = rng(seed);
        let summary = MutationEngine::apply_mutations(&mut genome, &config, &[], &mut r);
        assert_eq!(
            summary
                .attempted_by_domain
                .get(&MutationDomain::Topology)
                .copied()
                .unwrap_or(0),
            0,
            "topology must never be selected with mesh_layer_probability=0 at seed {}",
            seed,
        );
    }
}

#[test]
fn engine_mesh_layer_probability_one_always_selects_topology() {
    let mut config = SimulationConfig::default().mutation;
    config.mutation_probability = 1.0;
    config.per_birth_mutation_events_min = 1;
    config.per_birth_mutation_events_max = 1;
    config.mesh_layer_probability = 1.0;

    for seed in 0u64..1000 {
        let mut genome = v3alpha1_founder_genome();
        let mut r = rng(seed);
        let summary = MutationEngine::apply_mutations(&mut genome, &config, &[], &mut r);
        assert_eq!(
            summary
                .attempted_by_domain
                .get(&MutationDomain::Topology)
                .copied()
                .unwrap_or(0),
            1,
            "topology must always be selected with mesh_layer_probability=1 at seed {}",
            seed,
        );
    }
}

#[test]
fn apply_topology_event_adds_pass_through_detour() {
    use crate::creature::genome::BackendDef;
    use crate::mutation::topology::TopologyOperator;
    use crate::mutation::types::TargetReachability;

    let config = SimulationConfig::default().mutation;

    let mut genome = v3alpha1_founder_genome();
    let before_nodes = genome.nodes.len();
    let mut r = rng(10_001);
    let reachability = apply_topology_event(
        &mut genome,
        TopologyOperator::AddNode,
        &mut TargetSelector::reachable_only(&[], 0.0),
        &mut r,
        &config,
        1,
    )
    .expect("AddNode should apply");

    assert_eq!(reachability, TargetReachability::Unreachable);
    assert_eq!(genome.nodes.len(), before_nodes + 1);

    let newborn = genome.nodes.last().expect("newborn node must exist");
    match &newborn.backend_def {
        BackendDef::Vm(vm) => assert_eq!(
            vm.program,
            vec![crate::creature::genome::VmInstruction::Halt]
        ),
        BackendDef::Graph(graph) => assert_eq!(
            graph,
            &crate::creature::genome::cgp::CgpGraphBackendDef::new_with_fixed_outputs(&config)
        ),
    }
    assert!(newborn.input_refs.is_empty());
}

#[test]
fn engine_pressure_disabled_does_not_restrict() {
    use crate::mutation::types::ComplexityEffect;

    let mut config = SimulationConfig::default().mutation;
    config.mutation_probability = 1.0;
    config.per_birth_mutation_events_min = 1;
    config.per_birth_mutation_events_max = 1;
    config.genome_size_pressure_enabled = false;
    config.genome_size_cap = 1; // absurdly low cap

    // Even with a cap of 1, if pressure is disabled, increasing operators must still appear.
    let mut has_increasing = false;
    for seed in 0u64..5000 {
        let mut genome = v3alpha1_founder_genome();
        let mut r = rng(seed);
        let summary = MutationEngine::apply_mutations(&mut genome, &config, &[], &mut r);
        for (op, &count) in &summary.attempted_by_operator {
            if count > 0 && op.complexity_effect() == ComplexityEffect::Increasing {
                has_increasing = true;
            }
        }
        if has_increasing {
            break;
        }
    }
    assert!(
        has_increasing,
        "with pressure disabled, increasing operators must still be selected"
    );
}

#[test]
fn engine_pressure_at_cap_selects_only_decreasing() {
    use crate::mutation::types::ComplexityEffect;

    let mut config = SimulationConfig::default().mutation;
    config.mutation_probability = 1.0;
    config.per_birth_mutation_events_min = 1;
    config.per_birth_mutation_events_max = 1;
    config.genome_size_pressure_enabled = true;
    config.genome_size_cap = 1; // founder genome is well above 1

    for seed in 0u64..2000 {
        let mut genome = v3alpha1_founder_genome();
        let mut r = rng(seed);
        let summary = MutationEngine::apply_mutations(&mut genome, &config, &[], &mut r);
        for (op, &count) in &summary.attempted_by_operator {
            if count > 0 {
                assert_eq!(
                    op.complexity_effect(),
                    ComplexityEffect::Decreasing,
                    "at cap, only Decreasing operators allowed; got {:?} (seed {})",
                    op,
                    seed
                );
            }
        }
    }
}

#[test]
fn engine_restricted_vm_mutations_can_apply() {
    // Restricted VM events should be able to select and apply a Decreasing operator.
    let mut config = SimulationConfig::default().mutation;
    config.mutation_probability = 1.0;
    config.per_birth_mutation_events_min = 1;
    config.per_birth_mutation_events_max = 1;
    config.genome_size_pressure_enabled = true;
    config.genome_size_cap = 1;
    config.mesh_layer_probability = 0.0; // force node-internal only

    let mut vm_attempted: u64 = 0;
    let mut vm_applied: u64 = 0;
    for seed in 0u64..3000 {
        let mut genome = v3alpha1_founder_genome();
        let mut r = rng(seed);
        let summary = MutationEngine::apply_mutations(&mut genome, &config, &[], &mut r);
        vm_attempted += summary
            .attempted_by_domain
            .get(&MutationDomain::Vm)
            .copied()
            .unwrap_or(0) as u64;
        vm_applied += summary
            .applied_by_domain
            .get(&MutationDomain::Vm)
            .copied()
            .unwrap_or(0) as u64;
    }
    assert!(
        vm_attempted > 0,
        "VM domain must be attempted at least once over 3000 seeds"
    );
    assert!(
        vm_applied > 0,
        "VM domain should apply at least once when restricted"
    );
}

#[test]
fn engine_does_not_record_operator_no_applicable_skips() {
    let mut config = SimulationConfig::default().mutation;
    config.mutation_probability = 1.0;
    config.per_birth_mutation_events_min = 1;
    config.per_birth_mutation_events_max = 1;

    let mut operator_no_applicable_skips: u64 = 0;
    for seed in 0u64..2_000 {
        let mut genome = v3alpha1_founder_genome();
        let mut r = rng(seed);
        let summary = MutationEngine::apply_mutations(&mut genome, &config, &[], &mut r);
        for by_reason in summary.skip_reasons_by_operator.values() {
            operator_no_applicable_skips += by_reason
                .get(&MutationSkipReason::NoApplicableTarget)
                .copied()
                .unwrap_or(0) as u64;
        }
    }

    assert_eq!(
        operator_no_applicable_skips, 0,
        "operator-attributed NoApplicableTarget skips should be eliminated by applicability-aware selection"
    );
}

#[test]
fn engine_accounting_invariant_holds_with_decreasing_skips() {
    // When events are skipped due to no Decreasing operators (e.g. VM),
    // the accounting invariant must still hold.
    let mut config = SimulationConfig::default().mutation;
    config.mutation_probability = 1.0;
    config.per_birth_mutation_events_min = 1;
    config.per_birth_mutation_events_max = 5;
    config.genome_size_pressure_enabled = true;
    config.genome_size_cap = 1;

    for seed in 0u64..200 {
        let mut genome = v3alpha1_founder_genome();
        let mut r = rng(seed);
        let summary = MutationEngine::apply_mutations(&mut genome, &config, &[], &mut r);
        assert_eq!(
            summary.attempted_events,
            summary.applied_events + summary.skipped_events,
            "accounting invariant violated at seed {} with decreasing-only restriction",
            seed
        );
    }
}

#[test]
fn engine_pressure_accounting_invariant_holds_when_restricted() {
    let mut config = SimulationConfig::default().mutation;
    config.mutation_probability = 1.0;
    config.per_birth_mutation_events_min = 1;
    config.per_birth_mutation_events_max = 5;
    config.genome_size_pressure_enabled = true;
    config.genome_size_cap = 1;

    for seed in 0u64..100 {
        let mut genome = v3alpha1_founder_genome();
        let mut r = rng(seed);
        let summary = MutationEngine::apply_mutations(&mut genome, &config, &[], &mut r);
        assert_eq!(
            summary.attempted_events,
            summary.applied_events + summary.skipped_events,
            "accounting invariant violated at seed {} with pressure enabled",
            seed
        );
    }
}

#[test]
fn engine_pressure_preserves_parseability_when_restricted() {
    let mut config = SimulationConfig::default().mutation;
    config.mutation_probability = 1.0;
    config.per_birth_mutation_events_min = 1;
    config.per_birth_mutation_events_max = 4;
    config.genome_size_pressure_enabled = true;
    config.genome_size_cap = 1;

    for seed in 0u64..50 {
        let mut genome = v3alpha1_founder_genome();
        let mut r = rng(seed);
        MutationEngine::apply_mutations(&mut genome, &config, &[], &mut r);
        assert!(
            ParseabilityGate::validate(&genome).is_ok(),
            "parseability violated at seed {} with pressure enabled",
            seed
        );
    }
}

#[test]
fn engine_with_bias_1_targets_only_reachable_vm_nodes() {
    use crate::contracts::NodeId;
    use crate::creature::genome::{
        BackendDef, CreatureGenome, NodeGenome, VmBackendDef, VmInstruction,
    };

    // Build a 3-node genome: entry=0 -> 1, node 2 is unreachable.
    // All three are VM backends so VM domain mutations can target any.
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![
            NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![1.0],
                    program: vec![VmInstruction::Halt],
                }),
                targets: vec![RouteTarget {
                    target_id: NodeId::new(1),
                    slot: 0,
                    gate_bias: 0.0,
                }],
            },
            NodeGenome {
                node_id: NodeId::new(1),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![2.0],
                    program: vec![VmInstruction::Halt],
                }),
                targets: vec![],
            },
            NodeGenome {
                node_id: NodeId::new(2),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![99.0],
                    program: vec![VmInstruction::Halt],
                }),
                targets: vec![],
            },
        ],
    };

    // Reachable set: only nodes 0 and 1 (sorted).
    let reachable: &[usize] = &[0, 1];

    let mut config = SimulationConfig::default().mutation;
    config.mutation_probability = 1.0;
    config.per_birth_mutation_events_min = 1;
    config.per_birth_mutation_events_max = 1;
    // Force node-internal layer only (VM/Graph/InputRef).
    config.mesh_layer_probability = 0.0;
    // Set all biases to 1.0 — always pick reachable when available.
    config.reachable_bias.topology = 1.0;
    config.reachable_bias.vm = 1.0;
    config.reachable_bias.graph = 1.0;
    config.reachable_bias.input_ref = 1.0;

    // Run many mutations, snapshot node 2 each time.
    let node2_original = genome.nodes[2].clone();
    let mut node2_ever_changed = false;
    for seed in 0u64..500 {
        let mut g = genome.clone();
        let mut r = rng(seed);
        let summary = MutationEngine::apply_mutations(&mut g, &config, reachable, &mut r);
        if summary.applied_events > 0 && g.nodes[2] != node2_original {
            node2_ever_changed = true;
            break;
        }
    }
    assert!(
        !node2_ever_changed,
        "with bias=1.0, unreachable node 2 must never be targeted by VM mutations"
    );
}

#[test]
fn engine_pressure_restricted_deletions_bias_toward_unreachable_nodes() {
    use crate::contracts::NodeId;
    use crate::creature::genome::{
        BackendDef, CreatureGenome, NodeGenome, VmBackendDef, VmInstruction,
    };

    // Build a 3-node VM genome: entry=0 -> 1, node 2 is unreachable.
    // Each node has 2 instructions so VmDeleteInstruction is always applicable.
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![
            NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![1.0],
                    program: vec![VmInstruction::Noop, VmInstruction::Halt],
                }),
                targets: vec![RouteTarget {
                    target_id: NodeId::new(1),
                    slot: 0,
                    gate_bias: 0.0,
                }],
            },
            NodeGenome {
                node_id: NodeId::new(1),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![2.0],
                    program: vec![VmInstruction::Noop, VmInstruction::Halt],
                }),
                targets: vec![],
            },
            NodeGenome {
                node_id: NodeId::new(2),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![99.0],
                    program: vec![VmInstruction::Noop, VmInstruction::Halt],
                }),
                targets: vec![],
            },
        ],
    };

    let reachable: &[usize] = &[0, 1];

    let mut config = SimulationConfig::default().mutation;
    config.mutation_probability = 1.0;
    config.per_birth_mutation_events_min = 1;
    config.per_birth_mutation_events_max = 1;
    config.mesh_layer_probability = 0.0;
    config.genome_size_pressure_enabled = true;
    config.genome_size_cap = 1;
    config.reachable_bias.vm = 1.0;

    let mut reachable_targets = 0u64;
    let mut unreachable_targets = 0u64;
    for seed in 0u64..3000 {
        let mut g = genome.clone();
        let mut r = rng(seed);
        let summary = MutationEngine::apply_mutations(&mut g, &config, reachable, &mut r);
        reachable_targets += summary.reachable_target_events as u64;
        unreachable_targets += summary.unreachable_target_events as u64;
    }

    assert!(
        reachable_targets + unreachable_targets > 0,
        "expected at least one reachability-classified mutation event"
    );
    assert!(
        unreachable_targets > reachable_targets,
        "restricted deletions should favor unreachable targets under inverted pressure bias; reachable={}, unreachable={}",
        reachable_targets,
        unreachable_targets
    );
}

#[test]
fn provisional_supply_mean_and_single_event_share() {
    let config = MutationConfig::default();
    let mut random = rng(20_260_905);
    let mut events = 0u64;
    let mut triggered = 0u32;
    let mut singles = 0u32;
    let births = 200_000u32;
    for _ in 0..births {
        let count = requested_event_count(&config, &mut random);
        events += u64::from(count);
        triggered += u32::from(count > 0);
        singles += u32::from(count == 1);
    }
    // Fixed seed and tolerances wider than six standard errors at this sample size.
    assert!((f64::from(triggered) / f64::from(births) - 0.44).abs() < 0.01);
    assert!((events as f64 / f64::from(births) - 0.54999994368).abs() < 0.015);
    assert!((f64::from(singles) / f64::from(triggered) - 0.8).abs() < 0.015);
}

proptest::proptest! {
    #[test]
    fn bounded_supply_and_engine_accounting(
        seed in proptest::prelude::any::<u64>(), min in 1u32..8, extra in 0u32..8,
        continuation in 0.0f64..=1.0, trigger in 0.0f64..=1.0,
    ) {
        let config = MutationConfig {
            mutation_probability: trigger,
            per_birth_mutation_events_min: min,
            per_birth_mutation_events_max: min + extra,
            per_birth_mutation_event_continuation_probability: continuation,
            ..MutationConfig::default()
        };
        let requested = requested_event_count(&config, &mut rng(seed));
        proptest::prop_assert!(requested == 0 || (min..=min + extra).contains(&requested));
        let summary = MutationEngine::apply_mutations(&mut v3alpha1_founder_genome(), &config, &[], &mut rng(seed));
        proptest::prop_assert_eq!(summary.attempted_events, requested);
        proptest::prop_assert_eq!(summary.attempted_events, summary.applied_events + summary.skipped_events);
    }

    #[test]
    fn supply_probability_endpoints_and_equal_bounds(
        seed in proptest::prelude::any::<u64>(), min in 1u32..100, extra in 0u32..100,
    ) {
        let mut config = MutationConfig {
            mutation_probability: 1.0,
            per_birth_mutation_events_min: min,
            per_birth_mutation_events_max: min + extra,
            per_birth_mutation_event_continuation_probability: 0.0,
            ..MutationConfig::default()
        };
        proptest::prop_assert_eq!(requested_event_count(&config, &mut rng(seed)), min);
        config.per_birth_mutation_event_continuation_probability = 1.0;
        proptest::prop_assert_eq!(requested_event_count(&config, &mut rng(seed)), min + extra);
        config.per_birth_mutation_events_min = min + extra;
        config.per_birth_mutation_event_continuation_probability = 0.2;
        proptest::prop_assert_eq!(requested_event_count(&config, &mut rng(seed)), min + extra);
        config.mutation_probability = 0.0;
        proptest::prop_assert_eq!(requested_event_count(&config, &mut rng(seed)), 0);
    }
}

#[test]
fn supply_upper_bound_does_not_overflow() {
    let config = MutationConfig {
        mutation_probability: 1.0,
        per_birth_mutation_events_min: u32::MAX - 1,
        per_birth_mutation_events_max: u32::MAX,
        per_birth_mutation_event_continuation_probability: 1.0,
        ..MutationConfig::default()
    };
    assert_eq!(requested_event_count(&config, &mut rng(1)), u32::MAX);
}

// ── T11.F17: executed-biased targeting ───────────────────────────────────

/// A two-node executed core (entry chained to one successor) plus three
/// disconnected junk nodes carrying distinct VM constants.
fn core_and_junk_genome() -> CreatureGenome {
    let vm_node = |id: u32, constant: f32, target: Option<u32>| NodeGenome {
        node_id: NodeId::new(id),
        input_refs: vec![InputReference::World(WorldInputKey::FoodHere {
            type_idx: crate::config::OrdinaryFoodTypeId::default(),
        })],
        backend_def: BackendDef::Vm(crate::creature::genome::VmBackendDef {
            register_count: 4,
            constants: vec![constant],
            program: vec![
                crate::creature::genome::VmInstruction::LoadConst {
                    dst: 0,
                    const_idx: 0,
                },
                crate::creature::genome::VmInstruction::Halt,
            ],
        }),
        targets: target.map_or_else(Vec::new, |id| {
            vec![RouteTarget {
                target_id: NodeId::new(id),
                slot: 0,
                gate_bias: 0.0,
            }]
        }),
    };
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![
            vm_node(0, 1.0, Some(1)),
            vm_node(1, 2.0, None),
            vm_node(2, 3.0, None),
            vm_node(3, 4.0, None),
            vm_node(4, 5.0, None),
        ],
    }
}

fn node_internal_config(executed_bias: f64) -> MutationConfig {
    MutationConfig {
        mutation_probability: 1.0,
        per_birth_mutation_events_min: 4,
        per_birth_mutation_events_max: 4,
        mesh_layer_probability: 0.0,
        executed_bias,
        ..SimulationConfig::default().mutation
    }
}

#[test]
fn full_executed_bias_leaves_junk_nodes_untouched_and_counts_every_target() {
    let subject = core_and_junk_genome();
    let config = node_internal_config(1.0);
    let mut mutated_core = 0;
    for seed in 0..60 {
        let mut genome = subject.clone();
        let summary = MutationEngine::apply_mutations_with_food_type_count(
            &mut genome,
            &config,
            &[0, 1],
            ParentExecuted::Indices(&[0, 1]),
            &mut rng(seed),
            1,
        );
        assert_eq!(
            genome.nodes[2..],
            subject.nodes[2..],
            "seed {seed}: junk nodes are never targeted at executed bias 1.0"
        );
        assert_eq!(
            summary.executed_target_events,
            summary.applied_events - summary.not_applicable_events,
            "seed {seed}: every applied targeted event landed on an executed node"
        );
        mutated_core += u32::from(genome.nodes[..2] != subject.nodes[..2]);
    }
    assert!(
        mutated_core > 0,
        "the core must actually receive mutations in this fixture"
    );
}

#[test]
fn zero_executed_bias_reaches_junk_nodes_in_the_same_fixture() {
    let subject = core_and_junk_genome();
    let config = node_internal_config(0.0);
    let mut junk_hits = 0;
    let mut off_core_targets = 0;
    for seed in 0..60 {
        let mut genome = subject.clone();
        let summary = MutationEngine::apply_mutations_with_food_type_count(
            &mut genome,
            &config,
            &[0, 1],
            ParentExecuted::Indices(&[0, 1]),
            &mut rng(seed),
            1,
        );
        junk_hits += u32::from(genome.nodes[2..] != subject.nodes[2..]);
        // The counter reports membership, so without the layer some targeted
        // events still land on the core and some land off it.
        off_core_targets +=
            summary.applied_events - summary.not_applicable_events - summary.executed_target_events;
    }
    assert!(
        junk_hits > 0,
        "without the executed layer, junk nodes are ordinary targets"
    );
    assert!(
        off_core_targets > 0,
        "without the executed layer, targets land outside the executed core"
    );
}

/// While every eligible node is one the parent executed, the draw and its RNG
/// consumption are exactly the pre-T11.F17 ones. A single-event birth from an
/// all-executed parent therefore always matches; a multi-event birth can
/// diverge, because an earlier event that adds a mesh node leaves the later
/// events with an eligible node the parent never executed. That is the
/// intended behavior and the reason a founder's births are not *all*
/// byte-identical to before this feature.
#[test]
fn an_all_executed_eligible_set_reproduces_the_pre_feature_draw_byte_for_byte() {
    let founder = v3alpha1_founder_genome();
    let reachable = crate::creature::genome::analysis::mesh_reachable_nodes(&founder);
    let all_nodes: Vec<usize> = (0..founder.nodes.len()).collect();
    assert_eq!(
        reachable, all_nodes,
        "the founder's whole mesh is reachable"
    );
    let production = SimulationConfig::default().mutation;
    let single_event = MutationConfig {
        mutation_probability: 1.0,
        per_birth_mutation_events_min: 1,
        per_birth_mutation_events_max: 1,
        ..production.clone()
    };

    for config in [&single_event, &production] {
        let plain = MutationConfig {
            executed_bias: 0.0,
            ..config.clone()
        };
        let mut divergent = 0;
        for seed in 0..3_000 {
            let mut with_layer = founder.clone();
            let mut layer_rng = rng(seed);
            let summary = MutationEngine::apply_mutations_with_food_type_count(
                &mut with_layer,
                config,
                &reachable,
                ParentExecuted::Indices(&all_nodes),
                &mut layer_rng,
                1,
            );

            let mut without_layer = founder.clone();
            let mut plain_rng = rng(seed);
            MutationEngine::apply_mutations_with_food_type_count(
                &mut without_layer,
                &plain,
                &reachable,
                ParentExecuted::NONE,
                &mut plain_rng,
                1,
            );
            // Same offspring *and* the same position in the RNG stream.
            let identical =
                with_layer == without_layer && layer_rng.gen::<u64>() == plain_rng.gen::<u64>();
            if identical {
                assert_eq!(
                    summary.executed_target_events, summary.reachable_target_events,
                    "seed {seed}: every target of an all-executed parent is executed"
                );
                continue;
            }
            divergent += 1;
            assert!(
                summary.applied_events >= 2,
                "seed {seed}: only a later event in a multi-event birth can diverge"
            );
        }
        if config.per_birth_mutation_events_max == 1 {
            assert_eq!(divergent, 0, "single-event births never diverge");
        } else {
            assert!(
                divergent > 0,
                "the production sweep must exercise mid-birth mesh growth"
            );
            assert!(
                divergent < 60,
                "divergence is confined to mesh-growing multi-event births, got {divergent}/3000"
            );
        }
    }
}

#[test]
fn size_pressure_restriction_disables_the_executed_layer() {
    let subject = core_and_junk_genome();
    let restricted = MutationConfig {
        genome_size_pressure_enabled: true,
        genome_size_cap: 1,
        ..node_internal_config(1.0)
    };
    let reference = MutationConfig {
        executed_bias: 0.0,
        ..restricted.clone()
    };
    let mut differed_without_pressure = 0;
    for seed in 0..40 {
        let mut restricted_child = subject.clone();
        let mut restricted_rng = rng(seed);
        let summary = MutationEngine::apply_mutations_with_food_type_count(
            &mut restricted_child,
            &restricted,
            &[0, 1],
            ParentExecuted::Indices(&[0, 1]),
            &mut restricted_rng,
            1,
        );
        let mut reference_child = subject.clone();
        let mut reference_rng = rng(seed);
        let reference_summary = MutationEngine::apply_mutations_with_food_type_count(
            &mut reference_child,
            &reference,
            &[0, 1],
            ParentExecuted::Indices(&[0, 1]),
            &mut reference_rng,
            1,
        );
        assert_eq!(
            restricted_child, reference_child,
            "seed {seed}: under restriction the executed bias is 0.0"
        );
        assert_eq!(restricted_rng.gen::<u64>(), reference_rng.gen::<u64>());
        assert_eq!(
            summary.executed_target_events,
            reference_summary.executed_target_events
        );

        // Without size pressure the same seed does take the executed layer.
        let mut unpressured = subject.clone();
        MutationEngine::apply_mutations_with_food_type_count(
            &mut unpressured,
            &node_internal_config(1.0),
            &[0, 1],
            ParentExecuted::Indices(&[0, 1]),
            &mut rng(seed),
            1,
        );
        differed_without_pressure += u32::from(unpressured != restricted_child);
    }
    assert!(
        differed_without_pressure > 0,
        "the fixture must distinguish restricted from unrestricted targeting"
    );
}

#[test]
fn a_zero_event_birth_never_resolves_the_parent_dispatch_record() {
    let mut record = crate::creature::state::DispatchRecord::default();
    record.record_dispatch(0);
    let config = MutationConfig {
        mutation_probability: 0.0,
        ..SimulationConfig::default().mutation
    };
    let mut genome = v3alpha1_founder_genome();
    let summary = MutationEngine::apply_mutations_with_food_type_count(
        &mut genome,
        &config,
        &[0],
        ParentExecuted::Record(&record, 10),
        &mut rng(1),
        1,
    );
    assert_eq!(summary.attempted_events, 0);
    assert_eq!(summary.executed_target_events, 0);
}

#[test]
fn production_supply_keeps_all_operator_families_enabled() {
    let config = MutationConfig::default();
    assert!(config.mesh_layer_probability > 0.0 && config.mesh_layer_probability < 1.0);
    assert!(!config.genome_size_pressure_enabled);
    assert!(TopologyOperator::ALL.iter().all(|op| op.weight() > 0));
    assert!(VmOperator::ALL.iter().all(|op| op.weight() > 0));
    assert!(GraphOperator::ALL.iter().all(|op| op.weight() > 0));
    assert!(InputRefOperator::ALL.iter().all(|op| op.weight() > 0));
}

/// A configuration that draws exactly one node-internal event per birth.
fn single_node_internal_event_config() -> MutationConfig {
    let mut config = SimulationConfig::default().mutation;
    config.mutation_probability = 1.0;
    config.per_birth_mutation_events_min = 1;
    config.per_birth_mutation_events_max = 1;
    config.mesh_layer_probability = 0.0;
    config
}

/// The first seed in `0..2_000` whose single event lands on `domain`, with
/// that birth's summary.
fn first_event_in_domain(
    genome: &CreatureGenome,
    config: &MutationConfig,
    domain: MutationDomain,
) -> MutationSummary {
    for seed in 0u64..2_000 {
        let summary =
            MutationEngine::apply_mutations(&mut genome.clone(), config, &[0], &mut rng(seed));
        if summary
            .events
            .first()
            .is_some_and(|event| event.domain == domain)
        {
            return summary;
        }
    }
    panic!("no seed drew the {domain:?} domain");
}

#[test]
fn a_selected_module_with_no_applicable_site_is_recorded_with_its_node_id() {
    // Size pressure at the cap leaves only decreasing Graph operators, and an
    // edgeless Graph node offers none of them a site: every operator selects
    // the node and then reports NoApplicableTarget.
    let mut config = single_node_internal_event_config();
    config.genome_size_pressure_enabled = true;
    config.genome_size_cap = 1;
    let genome = single_graph_genome_with_inputs(vec![]);
    let summary = first_event_in_domain(&genome, &config, MutationDomain::Graph);
    assert_eq!(summary.attempted_events, 1);
    assert_eq!(summary.skipped_events, 1);
    let [event] = &summary.events[..] else {
        panic!("one event: {:?}", summary.events);
    };
    assert_eq!(event.domain, MutationDomain::Graph);
    assert_eq!(event.operator, None);
    assert_eq!(event.target, Some(NodeId::new(0)));
    assert_eq!(
        event.outcome,
        MutationEventOutcome::Skipped(MutationSkipReason::NoApplicableTarget)
    );
    // Every operator of the domain was tried and thrown away, and at least one
    // of them had already selected the edgeless node.
    assert!(!event.discarded.is_empty());
    assert!(event
        .discarded
        .iter()
        .all(|&(operator, _)| operator.domain() == MutationDomain::Graph));
    assert_eq!(
        event.discarded.iter().find_map(|&(_, pick)| pick),
        Some(NodeId::new(0))
    );
    let operators: BTreeSet<MutationOperator> = event
        .discarded
        .iter()
        .map(|&(operator, _)| operator)
        .collect();
    assert_eq!(
        operators.len(),
        event.discarded.len(),
        "no operator retried"
    );
}

#[test]
fn an_event_with_no_eligible_node_records_no_target() {
    // The same domain on a genome carrying no Graph node: nothing is ever
    // selected, so the record carries no target at all.
    let config = single_node_internal_event_config();
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: Vec::new(),
            backend_def: BackendDef::Vm(crate::creature::genome::VmBackendDef {
                register_count: 1,
                constants: Vec::new(),
                program: vec![crate::creature::genome::VmInstruction::Halt],
            }),
            targets: Vec::new(),
        }],
    };
    let summary = first_event_in_domain(&genome, &config, MutationDomain::Graph);
    let [event] = &summary.events[..] else {
        panic!("one event: {:?}", summary.events);
    };
    assert_eq!(event.domain, MutationDomain::Graph);
    assert_eq!(event.operator, None);
    assert_eq!(event.target, None);
    assert_eq!(
        event.outcome,
        MutationEventOutcome::Skipped(MutationSkipReason::NoApplicableTarget)
    );
    assert!(!event.discarded.is_empty());
    assert!(
        event.discarded.iter().all(|&(_, pick)| pick.is_none()),
        "nothing was ever selected: {:?}",
        event.discarded
    );
}

#[test]
fn a_discarded_operator_stays_visible_when_a_later_operator_applied() {
    // An edgeless Graph node offers the edge operators no site, so they select
    // it and report NoApplicableTarget; the engine then retries another Graph
    // operator, which applies. The discard is the T13.F01 fact and it must
    // survive the retry.
    let config = single_node_internal_event_config();
    let genome = single_graph_genome_with_inputs(vec![]);
    let mut seen = 0;
    let mut edge_operator_discarded = 0;
    for seed in 0u64..2_000 {
        let summary =
            MutationEngine::apply_mutations(&mut genome.clone(), &config, &[0], &mut rng(seed));
        let Some(event) = summary.events.first() else {
            continue;
        };
        if event.domain != MutationDomain::Graph
            || !event.outcome.is_applied()
            || event.discarded.is_empty()
        {
            continue;
        }
        seen += 1;
        assert!(
            event.operator.is_some(),
            "an applied event names its operator"
        );
        assert!(
            event
                .discarded
                .iter()
                .all(
                    |&(operator, pick)| operator.domain() == MutationDomain::Graph
                        && pick == Some(NodeId::new(0))
                ),
            "every discarded operator selected the edgeless module: {event:?}"
        );
        edge_operator_discarded += u32::from(event.discarded.iter().any(|&(operator, _)| {
            matches!(
                operator,
                MutationOperator::GraphRemoveGraphEdge
                    | MutationOperator::GraphRetargetGraphEdge
                    | MutationOperator::GraphAlterGraphEdgeWeight
                    | MutationOperator::GraphCopyEdgeBundle
            )
        }));
    }
    assert!(seen > 0, "no seed applied after discarding an operator");
    assert!(
        edge_operator_discarded > 0,
        "no applied event discarded an operator that needs an edge"
    );
}

#[test]
fn an_applied_event_records_the_node_the_genome_carried_before_it() {
    let config = forced_topology_config();
    let founder = v3alpha1_founder_genome();
    let before: Vec<NodeId> = founder.nodes.iter().map(|node| node.node_id).collect();
    let mut applied = 0;
    for seed in 0u64..200 {
        let summary =
            MutationEngine::apply_mutations(&mut founder.clone(), &config, &[0], &mut rng(seed));
        for event in &summary.events {
            if let Some(target) = event.target {
                assert!(before.contains(&target), "{event:?}");
            }
            if event.outcome.is_applied() {
                applied += 1;
                assert!(event.operator.is_some());
            }
        }
    }
    assert!(applied > 0, "the fixture must apply some events");
}

proptest::proptest! {
    /// Every attempted event leaves exactly one record, and the records
    /// reconcile with the applied/skipped totals and each operator's funnel.
    #[test]
    fn event_records_agree_with_the_totals_and_the_operator_funnels(
        seed in proptest::prelude::any::<u64>(),
        min in 1u32..6,
        extra in 0u32..6,
        mesh_layer in 0.0f64..=1.0,
        restricted in proptest::prelude::any::<bool>(),
    ) {
        let base = SimulationConfig::default().mutation;
        let config = MutationConfig {
            mutation_probability: 1.0,
            per_birth_mutation_events_min: min,
            per_birth_mutation_events_max: min + extra,
            mesh_layer_probability: mesh_layer,
            genome_size_pressure_enabled: restricted,
            genome_size_cap: if restricted { 1 } else { base.genome_size_cap },
            ..base
        };
        let summary = MutationEngine::apply_mutations(
            &mut v3alpha1_founder_genome(), &config, &[0], &mut rng(seed));

        proptest::prop_assert_eq!(summary.events.len(), summary.attempted_events as usize);
        let applied = summary.events.iter().filter(|e| e.outcome.is_applied()).count();
        proptest::prop_assert_eq!(applied, summary.applied_events as usize);
        proptest::prop_assert_eq!(
            summary.events.len() - applied, summary.skipped_events as usize);

        for (&operator, funnel) in &summary.operator_funnel_by_operator {
            let owned: Vec<_> = summary.events.iter()
                .filter(|e| e.operator == Some(operator)).collect();
            proptest::prop_assert_eq!(owned.len() as u64, funnel.attempted);
            proptest::prop_assert_eq!(
                owned.iter().filter(|e| e.outcome.is_applied()).count() as u64, funnel.applied);
            proptest::prop_assert_eq!(
                owned.iter().filter(|e| !e.outcome.is_applied()).count() as u64, funnel.skipped);
        }
        // A record naming no operator is a domain-exhausted skip, and it is
        // the only kind of record that carries no operator.
        for event in &summary.events {
            // Every discarded operator belongs to the event's domain, is
            // discarded at most once, and never repeats the accepted operator.
            let mut seen = std::collections::BTreeSet::new();
            for &(operator, _) in &event.discarded {
                proptest::prop_assert_eq!(operator.domain(), event.domain);
                proptest::prop_assert!(seen.insert(operator));
                proptest::prop_assert_ne!(Some(operator), event.operator);
            }
            if event.operator.is_none() {
                // A domain-exhausted event discarded every operator it tried
                // and carries the first node any of them selected.
                proptest::prop_assert!(!event.discarded.is_empty());
                proptest::prop_assert_eq!(
                    event.target, event.discarded.iter().find_map(|&(_, pick)| pick));
            }
            proptest::prop_assert!(event.operator.is_some() || matches!(
                event.outcome,
                MutationEventOutcome::Skipped(MutationSkipReason::NoApplicableTarget)));
            proptest::prop_assert_eq!(
                event.operator.map(MutationOperator::domain).unwrap_or(event.domain),
                event.domain);
        }
    }
}

#[test]
fn birth_tracking_preserves_mutation_rng_events_and_rollback() {
    use crate::creature::genome::cgp::{ComputeNode, ComputeNodeKind, GraphEdge, GraphSource};
    use crate::simulation::actions::cgp_reproduction::capture_birth_weights;
    use rand::RngCore;
    let mut original = single_graph_genome_with_inputs(vec![]);
    let BackendDef::Graph(def) = &mut original.nodes[0].backend_def else {
        unreachable!()
    };
    def.compute_nodes.push(ComputeNode {
        kind: ComputeNodeKind::Add,
        inputs: vec![GraphEdge {
            source: GraphSource::SharedMemory {
                slot: 0,
                previous: false,
            },
            weight: 0.5,
        }],
        plasticity: None,
    });
    let mut config = SimulationConfig::default().mutation;
    config.mutation_probability = 1.0;
    config.per_birth_mutation_events_min = 6;
    config.per_birth_mutation_events_max = 6;
    for seed in 0..32 {
        let mut plain = original.clone();
        let mut tracked = original.clone();
        let BackendDef::Graph(def) = &mut tracked.nodes[0].backend_def else {
            unreachable!()
        };
        capture_birth_weights(def, &[Box::new([0.75])]);
        let mut plain_rng = rng(seed);
        let mut tracked_rng = rng(seed);
        let plain_summary =
            MutationEngine::apply_mutations(&mut plain, &config, &[0], &mut plain_rng);
        let tracked_summary =
            MutationEngine::apply_mutations(&mut tracked, &config, &[0], &mut tracked_rng);
        assert_eq!(plain, tracked);
        assert_eq!(plain_summary.events, tracked_summary.events);
        assert_eq!(
            plain_summary.applied_semantic_noop_events,
            tracked_summary.applied_semantic_noop_events
        );
        assert_eq!(
            plain_summary.applied_semantic_change_events,
            tracked_summary.applied_semantic_change_events
        );
        assert_eq!(
            plain_summary.operator_funnel_by_operator,
            tracked_summary.operator_funnel_by_operator
        );
        assert_eq!(plain_rng.next_u64(), tracked_rng.next_u64());
        for node in &tracked.nodes {
            if let BackendDef::Graph(def) = &node.backend_def {
                if let Some(weights) = &def.birth_weights {
                    assert_eq!(weights.len(), def.compute_nodes.len());
                    for (row, node) in weights.iter().zip(&def.compute_nodes) {
                        assert_eq!(row.len(), node.inputs.len());
                    }
                }
            }
        }
    }
    original.nodes.push(original.nodes[0].clone());
    let BackendDef::Graph(def) = &mut original.nodes[0].backend_def else {
        unreachable!()
    };
    capture_birth_weights(def, &[Box::new([0.75])]);
    let before = original.clone();
    assert_eq!(
        apply_graph_event(
            &mut original,
            GraphOperator::AlterGraphEdgeWeight,
            &mut TargetSelector::reachable_only(&[0], 1.0),
            &mut rng(9),
            &config
        ),
        Err(MutationSkipReason::ParseabilityViolation)
    );
    assert_eq!(original, before);
    let BackendDef::Graph(def) = &original.nodes[0].backend_def else {
        unreachable!()
    };
    assert_eq!(def.birth_weights, Some(vec![vec![Some(0.75)]]));
}
