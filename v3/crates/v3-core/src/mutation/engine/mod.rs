use rand::Rng;

use crate::config::MutationConfig;
use crate::creature::genome::CreatureGenome;
use crate::creature::parseability::ParseabilityGate;
use crate::mutation::graph::{GraphMutator, GraphOperator};
use crate::mutation::input_ref::{InputRefMutator, InputRefOperator};
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

        let mut summary = MutationSummary::zero();
        for _ in 0..event_count {
            // Pick domain uniformly: 0=Topology, 1=VM, 2=Graph, 3=InputRef.
            let (domain, operator, result) = match rng.gen_range(0u8..4) {
                0 => {
                    let op = TopologyOperator::random(rng);
                    (
                        MutationDomain::Topology,
                        topology_operator_key(op),
                        apply_topology_event(genome, op, rng),
                    )
                }
                1 => {
                    let op = VmOperator::random(rng);
                    (
                        MutationDomain::Vm,
                        vm_operator_key(op),
                        apply_vm_event(genome, op, rng),
                    )
                }
                2 => {
                    let op = GraphOperator::random(rng);
                    (
                        MutationDomain::Graph,
                        graph_operator_key(op),
                        apply_graph_event(genome, op, rng),
                    )
                }
                _ => {
                    let op = InputRefOperator::random(rng);
                    (
                        MutationDomain::InputRef,
                        input_ref_operator_key(op),
                        apply_input_ref_event(genome, op, rng),
                    )
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
) -> Result<(), MutationSkipReason> {
    let snapshot = genome.clone();
    match InputRefMutator::apply(genome, op, rng) {
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

        assert_eq!(attempted_by_domain, summary.attempted_events);
        assert_eq!(applied_by_domain, summary.applied_events);
        assert_eq!(attempted_by_operator, summary.attempted_events);
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
}
