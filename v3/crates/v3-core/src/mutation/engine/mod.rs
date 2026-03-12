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

        let mut summary = MutationSummary::zero();
        for _ in 0..event_count {
            // Two-layer dispatch: mesh (Topology) vs node-internal (VM/Graph/InputRef).
            let rb = &config.reachable_bias;
            let (domain, operator, result) = if rng.gen_bool(config.mesh_layer_probability) {
                // Layer 1: Mesh (Topology)
                let mut available: Vec<TopologyOperator> = TopologyOperator::ALL
                    .iter()
                    .copied()
                    .filter(|op| !restricted || op.complexity_effect().is_decreasing())
                    .collect();
                let selected = loop {
                    if available.is_empty() {
                        break None;
                    }
                    let idx = select_weighted_index(&available, |op| op.weight() as u16, rng);
                    let op = available[idx];
                    let result = apply_topology_event(
                        genome,
                        op,
                        parent_reachable_nodes,
                        rb.topology,
                        rng,
                        config,
                    );
                    if matches!(result, Err(MutationSkipReason::NoApplicableTarget)) {
                        available.swap_remove(idx);
                        continue;
                    }
                    break Some((op, result));
                };
                let Some((op, result)) = selected else {
                    summary.record_domain_skip(
                        MutationDomain::Topology,
                        MutationSkipReason::NoApplicableTarget,
                    );
                    continue;
                };
                (MutationDomain::Topology, topology_operator_key(op), result)
            } else {
                // Layer 2: Node-internal (VM, Graph, InputRef — equal probability)
                match rng.gen_range(0u8..3) {
                    0 => {
                        let mut available: Vec<VmOperator> = VmOperator::ALL
                            .iter()
                            .copied()
                            .filter(|op| !restricted || op.complexity_effect().is_decreasing())
                            .collect();
                        let selected = loop {
                            if available.is_empty() {
                                break None;
                            }
                            let idx =
                                select_weighted_index(&available, |op| op.weight() as u16, rng);
                            let op = available[idx];
                            let result = apply_vm_event(
                                genome,
                                op,
                                parent_reachable_nodes,
                                rb.vm,
                                rng,
                                config,
                            );
                            if matches!(result, Err(MutationSkipReason::NoApplicableTarget)) {
                                available.swap_remove(idx);
                                continue;
                            }
                            break Some((op, result));
                        };
                        let Some((op, result)) = selected else {
                            summary.record_domain_skip(
                                MutationDomain::Vm,
                                MutationSkipReason::NoApplicableTarget,
                            );
                            continue;
                        };
                        (MutationDomain::Vm, vm_operator_key(op), result)
                    }
                    1 => {
                        let mut available: Vec<GraphOperator> = GraphOperator::ALL
                            .iter()
                            .copied()
                            .filter(|op| !restricted || op.complexity_effect().is_decreasing())
                            .collect();
                        let selected = loop {
                            if available.is_empty() {
                                break None;
                            }
                            let idx =
                                select_weighted_index(&available, |op| op.weight() as u16, rng);
                            let op = available[idx];
                            let result = apply_graph_event(
                                genome,
                                op,
                                parent_reachable_nodes,
                                rb.graph,
                                rng,
                            );
                            if matches!(result, Err(MutationSkipReason::NoApplicableTarget)) {
                                available.swap_remove(idx);
                                continue;
                            }
                            break Some((op, result));
                        };
                        let Some((op, result)) = selected else {
                            summary.record_domain_skip(
                                MutationDomain::Graph,
                                MutationSkipReason::NoApplicableTarget,
                            );
                            continue;
                        };
                        (MutationDomain::Graph, graph_operator_key(op), result)
                    }
                    _ => {
                        let mut available: Vec<InputRefOperator> = InputRefOperator::ALL
                            .iter()
                            .copied()
                            .filter(|op| !restricted || op.complexity_effect().is_decreasing())
                            .collect();
                        let selected = loop {
                            if available.is_empty() {
                                break None;
                            }
                            let idx =
                                select_weighted_index(&available, |op| op.weight() as u16, rng);
                            let op = available[idx];
                            let result = apply_input_ref_event(
                                genome,
                                op,
                                parent_reachable_nodes,
                                rb.input_ref,
                                rng,
                                config,
                            );
                            if matches!(result, Err(MutationSkipReason::NoApplicableTarget)) {
                                available.swap_remove(idx);
                                continue;
                            }
                            break Some((op, result));
                        };
                        let Some((op, result)) = selected else {
                            summary.record_domain_skip(
                                MutationDomain::InputRef,
                                MutationSkipReason::NoApplicableTarget,
                            );
                            continue;
                        };
                        (MutationDomain::InputRef, input_ref_operator_key(op), result)
                    }
                }
            };
            summary.record_attempt(domain, operator);

            match result {
                Ok(reachability) => {
                    summary.record_applied(domain, operator, operator.semantic_category());
                    summary.record_reachability(reachability);
                }
                Err(reason) => summary.record_skipped(operator, reason),
            }
        }

        summary
    }
}

fn select_weighted_index<T: Copy>(
    options: &[T],
    mut weight_of: impl FnMut(T) -> u16,
    rng: &mut impl Rng,
) -> usize {
    debug_assert!(!options.is_empty(), "weighted selection requires options");
    let total_weight: u16 = options.iter().copied().map(&mut weight_of).sum();
    debug_assert!(
        total_weight > 0,
        "weighted selection requires positive total weight"
    );
    let mut draw = rng.gen_range(0..total_weight);
    for (idx, op) in options.iter().copied().enumerate() {
        let weight = weight_of(op);
        if draw < weight {
            return idx;
        }
        draw -= weight;
    }
    unreachable!("weight draw must return an index");
}

/// Apply one topology mutation event with parseability gate.
fn apply_topology_event(
    genome: &mut CreatureGenome,
    op: TopologyOperator,
    reachable_nodes: &[usize],
    bias: f64,
    rng: &mut impl Rng,
    config: &MutationConfig,
) -> Result<TargetReachability, MutationSkipReason> {
    let snapshot = genome.clone();
    match TopologyMutator::apply(genome, op, reachable_nodes, bias, rng, config) {
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
        VmOperator::VmDeleteInstruction => MutationOperator::VmDeleteInstruction,
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
        VmOperator::VmInsertReadBidMotif => MutationOperator::VmInsertReadBidMotif,
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
        GraphOperator::MutateActionSlotBehavior => MutationOperator::GraphMutateActionSlotBehavior,
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
