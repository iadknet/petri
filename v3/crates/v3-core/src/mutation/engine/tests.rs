use super::*;
use crate::config::SimulationConfig;
use crate::creature::founder::v3alpha1_founder_genome;
use crate::mutation::{MutationDomain, MutationOperator};
use rand::rngs::SmallRng;
use rand::SeedableRng;

fn rng(seed: u64) -> SmallRng {
    SmallRng::seed_from_u64(seed)
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

    assert_eq!(
        summary.applied_semantic_noop_events + summary.applied_semantic_change_events,
        summary.applied_events
    );
}

#[test]
fn engine_attempted_counters_cover_all_domains_and_operators_over_long_run() {
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

    for operator in MutationOperator::all() {
        assert!(
            operator_hits.get(&operator).copied().unwrap_or(0) > 0,
            "expected attempted events for operator {:?}",
            operator
        );
    }
}

#[test]
fn engine_applied_semantic_categories_record_noop_and_change_events() {
    let mut config = SimulationConfig::default().mutation;
    config.mutation_probability = 1.0;
    config.per_birth_mutation_events_min = 1;
    config.per_birth_mutation_events_max = 1;

    let mut noop_total: u64 = 0;
    let mut change_total: u64 = 0;
    let mut genome = v3alpha1_founder_genome();
    for seed in 0u64..10_000 {
        let mut r = rng(seed);
        let summary = MutationEngine::apply_mutations(&mut genome, &config, &[], &mut r);
        noop_total += summary.applied_semantic_noop_events as u64;
        change_total += summary.applied_semantic_change_events as u64;
    }

    assert!(
        noop_total > 0,
        "expected at least one applied semantic-noop mutation across long run"
    );
    assert!(
        change_total > 0,
        "expected at least one applied semantic-change mutation across long run"
    );
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
fn engine_restricted_vm_mutations_always_skipped() {
    // VM has 0 Decreasing operators, so restricted VM events must always be skipped.
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
    assert_eq!(
        vm_applied, 0,
        "VM domain must never apply when restricted (0 Decreasing operators)"
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
                targets: vec![NodeId::new(1)],
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
