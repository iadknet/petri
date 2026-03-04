use rand::Rng;

use crate::config::MutationConfig;
use crate::creature::genome::CreatureGenome;
use crate::creature::parseability::ParseabilityGate;
use crate::mutation::graph::{GraphMutator, GraphOperator};
use crate::mutation::input_ref::{InputRefMutator, InputRefOperator};
use crate::mutation::pressure;
use crate::mutation::topology::{TopologyMutator, TopologyOperator};
use crate::mutation::types::{
    MutationDomain, MutationOperator, MutationSkipReason, MutationSummary,
};
use crate::mutation::vm::{VmMutator, VmOperator};

/// Orchestrates genome mutation events for offspring.
pub struct MutationEngine;

impl MutationEngine {
    /// Apply mutation events to a child genome and return a summary.
    ///
    /// Accounting invariant: `summary.attempted_events == summary.applied_events + summary.skipped_events`.
    pub fn apply_mutations(
        genome: &mut CreatureGenome,
        config: &MutationConfig,
        rng: &mut impl Rng,
    ) -> MutationSummary {
        // Probability gate.
        if !rng.gen_bool(config.mutation_probability) {
            return MutationSummary::zero();
        }

        // Determine number of mutation events this birth.
        let event_count = rng
            .gen_range(config.per_birth_mutation_events_min..=config.per_birth_mutation_events_max);

        // Complexity pressure: compute once before the event loop.
        let restricted = config.complexity_pressure_enabled
            && pressure::is_restricted(genome.complexity(), config.complexity_cap, rng);

        // Select an operator: Decreasing-only when restricted, unrestricted otherwise.
        // Skips the event (continue) when no Decreasing operator exists for the domain.
        macro_rules! select_operator {
            ($Op:ty, $domain:expr, $summary:expr, $rng:expr) => {
                if restricted {
                    match <$Op>::random_decreasing($rng) {
                        Some(op) => op,
                        None => {
                            $summary.record_domain_skip(
                                $domain,
                                MutationSkipReason::NoApplicableTarget,
                            );
                            continue;
                        }
                    }
                } else {
                    <$Op>::random($rng)
                }
            };
        }

        let mut summary = MutationSummary::zero();
        for _ in 0..event_count {
            // Two-layer dispatch: mesh (Topology) vs node-internal (VM/Graph/InputRef).
            let (domain, operator, result) = if rng.gen_bool(config.mesh_layer_probability) {
                // Layer 1: Mesh (Topology)
                let op = select_operator!(TopologyOperator, MutationDomain::Topology, summary, rng);
                (
                    MutationDomain::Topology,
                    topology_operator_key(op),
                    apply_topology_event(genome, op, rng),
                )
            } else {
                // Layer 2: Node-internal (VM, Graph, InputRef — equal probability)
                match rng.gen_range(0u8..3) {
                    0 => {
                        let op = select_operator!(VmOperator, MutationDomain::Vm, summary, rng);
                        (
                            MutationDomain::Vm,
                            vm_operator_key(op),
                            apply_vm_event(genome, op, rng),
                        )
                    }
                    1 => {
                        let op =
                            select_operator!(GraphOperator, MutationDomain::Graph, summary, rng);
                        (
                            MutationDomain::Graph,
                            graph_operator_key(op),
                            apply_graph_event(genome, op, rng),
                        )
                    }
                    _ => {
                        let op = select_operator!(
                            InputRefOperator,
                            MutationDomain::InputRef,
                            summary,
                            rng
                        );
                        (
                            MutationDomain::InputRef,
                            input_ref_operator_key(op),
                            apply_input_ref_event(genome, op, rng, config),
                        )
                    }
                }
            };
            summary.record_attempt(domain, operator);

            match result {
                Ok(()) => summary.record_applied(domain, operator, operator.semantic_category()),
                Err(reason) => summary.record_skipped(reason),
            }
        }

        summary
    }
}

/// Apply one topology mutation event with parseability gate.
fn apply_topology_event(
    genome: &mut CreatureGenome,
    op: TopologyOperator,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let snapshot = genome.clone();
    match TopologyMutator::apply(genome, op, rng) {
        Ok(()) => {
            if ParseabilityGate::validate(genome).is_ok() {
                Ok(())
            } else {
                *genome = snapshot;
                Err(MutationSkipReason::ParseabilityViolation)
            }
        }
        Err(reason) => {
            *genome = snapshot;
            Err(reason)
        }
    }
}

/// Apply one VM mutation event with parseability gate.
fn apply_vm_event(
    genome: &mut CreatureGenome,
    op: VmOperator,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let snapshot = genome.clone();
    match VmMutator::apply(genome, op, rng) {
        Ok(()) => {
            if ParseabilityGate::validate(genome).is_ok() {
                Ok(())
            } else {
                *genome = snapshot;
                Err(MutationSkipReason::ParseabilityViolation)
            }
        }
        Err(reason) => {
            *genome = snapshot;
            Err(reason)
        }
    }
}

/// Apply one graph mutation event with parseability gate.
fn apply_graph_event(
    genome: &mut CreatureGenome,
    op: GraphOperator,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let snapshot = genome.clone();
    match GraphMutator::apply(genome, op, rng) {
        Ok(()) => {
            if ParseabilityGate::validate(genome).is_ok() {
                Ok(())
            } else {
                *genome = snapshot;
                Err(MutationSkipReason::ParseabilityViolation)
            }
        }
        Err(reason) => {
            *genome = snapshot;
            Err(reason)
        }
    }
}

/// Apply one input ref mutation event with parseability gate.
fn apply_input_ref_event(
    genome: &mut CreatureGenome,
    op: InputRefOperator,
    rng: &mut impl Rng,
    config: &MutationConfig,
) -> Result<(), MutationSkipReason> {
    let snapshot = genome.clone();
    match InputRefMutator::apply(genome, op, rng, config) {
        Ok(()) => {
            if ParseabilityGate::validate(genome).is_ok() {
                Ok(())
            } else {
                *genome = snapshot;
                Err(MutationSkipReason::ParseabilityViolation)
            }
        }
        Err(reason) => {
            *genome = snapshot;
            Err(reason)
        }
    }
}

fn topology_operator_key(op: TopologyOperator) -> MutationOperator {
    match op {
        TopologyOperator::AddNode => MutationOperator::TopologyAddNode,
        TopologyOperator::RemoveNode => MutationOperator::TopologyRemoveNode,
        TopologyOperator::RetargetNodeTarget => MutationOperator::TopologyRetargetNodeTarget,
        TopologyOperator::AddRouteTarget => MutationOperator::TopologyAddRouteTarget,
        TopologyOperator::RemoveRouteTarget => MutationOperator::TopologyRemoveRouteTarget,
        TopologyOperator::ChangeEntryNode => MutationOperator::TopologyChangeEntryNode,
        TopologyOperator::SwapNodeBackend => MutationOperator::TopologySwapNodeBackend,
        TopologyOperator::RewriteNodeId => MutationOperator::TopologyRewriteNodeId,
        TopologyOperator::CopyNode => MutationOperator::TopologyCopyNode,
        TopologyOperator::CopyMeshBackwardSlice => MutationOperator::TopologyCopyMeshBackwardSlice,
        TopologyOperator::CopyMeshForwardSlice => MutationOperator::TopologyCopyMeshForwardSlice,
        TopologyOperator::SpliceNode => MutationOperator::TopologySpliceNode,
        TopologyOperator::SwapRouteTargets => MutationOperator::TopologySwapRouteTargets,
    }
}

fn vm_operator_key(op: VmOperator) -> MutationOperator {
    match op {
        VmOperator::VmConstantMutation => MutationOperator::VmConstantMutation,
        VmOperator::VmInstructionMutation => MutationOperator::VmInstructionMutation,
        VmOperator::VmRegisterCountMutation => MutationOperator::VmRegisterCountMutation,
        VmOperator::VmInstructionRawFieldMutation => {
            MutationOperator::VmInstructionRawFieldMutation
        }
        VmOperator::VmCopyInstructionBlock => MutationOperator::VmCopyInstructionBlock,
        VmOperator::VmCopyInstructionBlockRemapped => {
            MutationOperator::VmCopyInstructionBlockRemapped
        }
        VmOperator::VmCopyConstantBlock => MutationOperator::VmCopyConstantBlock,
        VmOperator::VmCopyGeneBackwardSlice => MutationOperator::VmCopyGeneBackwardSlice,
        VmOperator::VmCopyGeneForwardSlice => MutationOperator::VmCopyGeneForwardSlice,
    }
}

fn graph_operator_key(op: GraphOperator) -> MutationOperator {
    match op {
        GraphOperator::AlterGraphEdgeWeight => MutationOperator::GraphAlterGraphEdgeWeight,
        GraphOperator::SwapGraphOperator => MutationOperator::GraphSwapGraphOperator,
        GraphOperator::MutateGraphOperatorParam => MutationOperator::GraphMutateGraphOperatorParam,
        GraphOperator::AddInternalGraphNode => MutationOperator::GraphAddInternalGraphNode,
        GraphOperator::RemoveInternalGraphNode => MutationOperator::GraphRemoveInternalGraphNode,
        GraphOperator::AddGraphEdge => MutationOperator::GraphAddGraphEdge,
        GraphOperator::RetargetGraphEdge => MutationOperator::GraphRetargetGraphEdge,
        GraphOperator::RemoveGraphEdge => MutationOperator::GraphRemoveGraphEdge,
        GraphOperator::GraphRawFieldMutation => MutationOperator::GraphRawFieldMutation,
        GraphOperator::CopyInternalNode => MutationOperator::GraphCopyInternalNode,
        GraphOperator::CopySubgraph => MutationOperator::GraphCopySubgraph,
        GraphOperator::CopyEdgeBundle => MutationOperator::GraphCopyEdgeBundle,
        GraphOperator::EnableHebbian => MutationOperator::GraphEnableHebbian,
        GraphOperator::DisableHebbian => MutationOperator::GraphDisableHebbian,
        GraphOperator::MutateHebbianRule => MutationOperator::GraphMutateHebbianRule,
        GraphOperator::MutateHebbianRate => MutationOperator::GraphMutateHebbianRate,
        GraphOperator::ToggleHebbianLamarckian => MutationOperator::GraphToggleHebbianLamarckian,
        GraphOperator::EnableRewardModulation => MutationOperator::GraphEnableRewardModulation,
        GraphOperator::DisableRewardModulation => MutationOperator::GraphDisableRewardModulation,
        GraphOperator::MutateRewardSource => MutationOperator::GraphMutateRewardSource,
        GraphOperator::MutateTraceDecay => MutationOperator::GraphMutateTraceDecay,
    }
}

fn input_ref_operator_key(op: InputRefOperator) -> MutationOperator {
    match op {
        InputRefOperator::Add => MutationOperator::InputRefAdd,
        InputRefOperator::Remove => MutationOperator::InputRefRemove,
        InputRefOperator::Swap => MutationOperator::InputRefSwap,
        InputRefOperator::RawFieldMutation => MutationOperator::InputRefRawFieldMutation,
    }
}

#[cfg(test)]
mod tests {
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
            let summary = MutationEngine::apply_mutations(&mut genome, &config, &mut r);
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
        let summary = MutationEngine::apply_mutations(&mut genome, &config, &mut r);
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
        let summary = MutationEngine::apply_mutations(&mut genome, &config, &mut r);
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
            MutationEngine::apply_mutations(&mut genome, &config, &mut r);
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
            let summary = MutationEngine::apply_mutations(&mut genome, &config, &mut r);
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
            MutationEngine::apply_mutations(&mut genome, &config, &mut r);
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
            MutationEngine::apply_mutations(&mut genome, &config, &mut r);
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
                MutationEngine::apply_mutations(&mut genome, &config, &mut r);
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
        let summary = MutationEngine::apply_mutations(&mut genome, &config, &mut rng);

        let attempted_by_domain: u32 = summary.attempted_by_domain.values().sum();
        let applied_by_domain: u32 = summary.applied_by_domain.values().sum();
        let attempted_by_operator: u32 = summary.attempted_by_operator.values().sum();
        let applied_by_operator: u32 = summary.applied_by_operator.values().sum();

        // Domain-level attempts include domain-skips (no applicable operator).
        assert_eq!(attempted_by_domain, summary.attempted_events);
        assert_eq!(applied_by_domain, summary.applied_events);
        // Operator-level attempts exclude domain-skips (no operator was selected).
        let domain_skips = summary
            .skip_reasons
            .get(&MutationSkipReason::NoApplicableTarget)
            .copied()
            .unwrap_or(0);
        assert_eq!(
            attempted_by_operator + domain_skips,
            summary.attempted_events
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
            let summary = MutationEngine::apply_mutations(&mut genome, &config, &mut r);
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
            let summary = MutationEngine::apply_mutations(&mut genome, &config, &mut r);
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
            let summary = MutationEngine::apply_mutations(&mut genome, &config, &mut r);
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
            let summary = MutationEngine::apply_mutations(&mut genome, &config, &mut r);
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
            let summary = MutationEngine::apply_mutations(&mut genome, &config, &mut r);
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
        config.complexity_pressure_enabled = false;
        config.complexity_cap = 1; // absurdly low cap

        // Even with a cap of 1, if pressure is disabled, increasing operators must still appear.
        let mut has_increasing = false;
        for seed in 0u64..5000 {
            let mut genome = v3alpha1_founder_genome();
            let mut r = rng(seed);
            let summary = MutationEngine::apply_mutations(&mut genome, &config, &mut r);
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
        config.complexity_pressure_enabled = true;
        config.complexity_cap = 1; // founder genome is well above 1

        for seed in 0u64..2000 {
            let mut genome = v3alpha1_founder_genome();
            let mut r = rng(seed);
            let summary = MutationEngine::apply_mutations(&mut genome, &config, &mut r);
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
        config.complexity_pressure_enabled = true;
        config.complexity_cap = 1;
        config.mesh_layer_probability = 0.0; // force node-internal only

        let mut vm_attempted: u64 = 0;
        let mut vm_applied: u64 = 0;
        for seed in 0u64..3000 {
            let mut genome = v3alpha1_founder_genome();
            let mut r = rng(seed);
            let summary = MutationEngine::apply_mutations(&mut genome, &config, &mut r);
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
        config.complexity_pressure_enabled = true;
        config.complexity_cap = 1;

        for seed in 0u64..200 {
            let mut genome = v3alpha1_founder_genome();
            let mut r = rng(seed);
            let summary = MutationEngine::apply_mutations(&mut genome, &config, &mut r);
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
        config.complexity_pressure_enabled = true;
        config.complexity_cap = 1;

        for seed in 0u64..100 {
            let mut genome = v3alpha1_founder_genome();
            let mut r = rng(seed);
            let summary = MutationEngine::apply_mutations(&mut genome, &config, &mut r);
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
        config.complexity_pressure_enabled = true;
        config.complexity_cap = 1;

        for seed in 0u64..50 {
            let mut genome = v3alpha1_founder_genome();
            let mut r = rng(seed);
            MutationEngine::apply_mutations(&mut genome, &config, &mut r);
            assert!(
                ParseabilityGate::validate(&genome).is_ok(),
                "parseability violated at seed {} with pressure enabled",
                seed
            );
        }
    }
}
