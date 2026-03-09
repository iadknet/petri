use rand::Rng;

use crate::config::MutationConfig;
use crate::creature::genome::CreatureGenome;
use crate::creature::parseability::ParseabilityGate;
use crate::mutation::graph::{GraphMutator, GraphOperator};
use crate::mutation::input_ref::{InputRefMutator, InputRefOperator};
use crate::mutation::pressure;
use crate::mutation::topology::{TopologyMutator, TopologyOperator};
use crate::mutation::types::{
    MutationDomain, MutationOperator, MutationSkipReason, MutationSummary, TargetReachability,
};
use crate::mutation::vm::{VmMutator, VmOperator};

/// Orchestrates genome mutation events for offspring.
pub struct MutationEngine;

impl MutationEngine {
    /// Apply mutation events to a child genome and return a summary.
    ///
    /// `parent_reachable_nodes` is the parent's cached reachable set (sorted ascending),
    /// used to bias mutation target selection toward functional structure.
    ///
    /// Accounting invariant: `summary.attempted_events == summary.applied_events + summary.skipped_events`.
    pub fn apply_mutations(
        genome: &mut CreatureGenome,
        config: &MutationConfig,
        parent_reachable_nodes: &[usize],
        rng: &mut impl Rng,
    ) -> MutationSummary {
        // Probability gate.
        if !rng.gen_bool(config.mutation_probability) {
            return MutationSummary::zero();
        }

        // Determine number of mutation events this birth.
        let event_count = rng
            .gen_range(config.per_birth_mutation_events_min..=config.per_birth_mutation_events_max);

        // Genome size pressure: compute once before the event loop.
        let restricted = config.genome_size_pressure_enabled
            && pressure::is_restricted(genome.genome_size(), config.genome_size_cap, rng);

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
            let rb = &config.reachable_bias;
            let (domain, operator, result) = if rng.gen_bool(config.mesh_layer_probability) {
                // Layer 1: Mesh (Topology)
                let op = select_operator!(TopologyOperator, MutationDomain::Topology, summary, rng);
                (
                    MutationDomain::Topology,
                    topology_operator_key(op),
                    apply_topology_event(genome, op, parent_reachable_nodes, rb.topology, rng),
                )
            } else {
                // Layer 2: Node-internal (VM, Graph, InputRef — equal probability)
                match rng.gen_range(0u8..3) {
                    0 => {
                        let op = select_operator!(VmOperator, MutationDomain::Vm, summary, rng);
                        (
                            MutationDomain::Vm,
                            vm_operator_key(op),
                            apply_vm_event(genome, op, parent_reachable_nodes, rb.vm, rng, config),
                        )
                    }
                    1 => {
                        let op =
                            select_operator!(GraphOperator, MutationDomain::Graph, summary, rng);
                        (
                            MutationDomain::Graph,
                            graph_operator_key(op),
                            apply_graph_event(genome, op, parent_reachable_nodes, rb.graph, rng),
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
                            apply_input_ref_event(
                                genome,
                                op,
                                parent_reachable_nodes,
                                rb.input_ref,
                                rng,
                                config,
                            ),
                        )
                    }
                }
            };
            summary.record_attempt(domain, operator);

            match result {
                Ok(reachability) => {
                    summary.record_applied(domain, operator, operator.semantic_category());
                    summary.record_reachability(reachability);
                }
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
    reachable_nodes: &[usize],
    bias: f64,
    rng: &mut impl Rng,
) -> Result<TargetReachability, MutationSkipReason> {
    let snapshot = genome.clone();
    match TopologyMutator::apply(genome, op, reachable_nodes, bias, rng) {
        Ok(reachability) => {
            if ParseabilityGate::validate(genome).is_ok() {
                Ok(reachability)
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
    reachable_nodes: &[usize],
    bias: f64,
    rng: &mut impl Rng,
    config: &MutationConfig,
) -> Result<TargetReachability, MutationSkipReason> {
    let snapshot = genome.clone();
    match VmMutator::apply(genome, op, reachable_nodes, bias, rng, config) {
        Ok(reachability) => {
            if ParseabilityGate::validate(genome).is_ok() {
                Ok(reachability)
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
    reachable_nodes: &[usize],
    bias: f64,
    rng: &mut impl Rng,
) -> Result<TargetReachability, MutationSkipReason> {
    let snapshot = genome.clone();
    match GraphMutator::apply(genome, op, reachable_nodes, bias, rng) {
        Ok(reachability) => {
            if ParseabilityGate::validate(genome).is_ok() {
                Ok(reachability)
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
    reachable_nodes: &[usize],
    bias: f64,
    rng: &mut impl Rng,
    config: &MutationConfig,
) -> Result<TargetReachability, MutationSkipReason> {
    let snapshot = genome.clone();
    match InputRefMutator::apply(genome, op, reachable_nodes, bias, rng, config) {
        Ok(reachability) => {
            if ParseabilityGate::validate(genome).is_ok() {
                Ok(reachability)
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
        VmOperator::VmInsertReadStoreMotif => MutationOperator::VmInsertReadStoreMotif,
        VmOperator::VmInsertLoadCompareMotif => MutationOperator::VmInsertLoadCompareMotif,
        VmOperator::VmMutateSlotAddress => MutationOperator::VmMutateSlotAddress,
        VmOperator::VmMutatePairedSlotAddress => MutationOperator::VmMutatePairedSlotAddress,
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
mod tests;
