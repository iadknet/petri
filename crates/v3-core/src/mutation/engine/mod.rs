use std::collections::BTreeSet;

use rand::Rng;

use crate::config::MutationConfig;
use crate::contracts::NodeId;
use crate::creature::genome::{BackendDef, CreatureGenome};
use crate::creature::parseability::ParseabilityGate;
use crate::mutation::graph::{GraphMutator, GraphOperator};
use crate::mutation::input_ref::{InputRefMutator, InputRefOperator};
use crate::mutation::pressure;
use crate::mutation::reachability::{ParentExecuted, TargetSelector, TargetSets};
use crate::mutation::topology::{TopologyMutator, TopologyOperator};
use crate::mutation::types::{
    MutationAddedNodeInputClass, MutationDomain, MutationEventOutcome, MutationEventRecord,
    MutationOperator, MutationSkipReason, MutationSummary, TargetReachability,
};
use crate::mutation::vm::{VmMutator, VmOperator};

/// Draw requested supply from normalized config, independently of operator success.
///
/// Production rule (T11.F19): one Bernoulli trial per genome unit at
/// `per_unit_rate`, so the count is `Binomial(genome_size, per_unit_rate)`
/// with no trigger, minimum, maximum, or continuation. The disabled legacy
/// rule below keeps its exact RNG consumption for the drift walk and T11.F13.
fn requested_event_count(config: &MutationConfig, genome_size: u32, rng: &mut impl Rng) -> u32 {
    if config.per_unit_supply_enabled {
        return (0..genome_size)
            .map(|_| u32::from(rng.gen_bool(config.per_unit_rate)))
            .sum();
    }
    if !rng.gen_bool(config.mutation_probability) {
        return 0;
    }
    let mut count = config.per_birth_mutation_events_min;
    while count < config.per_birth_mutation_events_max
        && rng.gen_bool(config.per_birth_mutation_event_continuation_probability)
    {
        count += 1;
    }
    count
}

/// What one attempted mutation event produced, once an operator of the drawn
/// domain accepted the attempt.
struct SelectedEvent {
    operator: MutationOperator,
    tracked_before: Option<CreatureGenome>,
    result: Result<TargetReachability, MutationSkipReason>,
    executed_hits: u32,
    /// The index this event's kept selector first returned, into the genome as
    /// it stood before the event (T13.F01).
    first_pick: Option<usize>,
}

/// Orchestrates genome mutation events for offspring.
pub struct MutationEngine;

impl MutationEngine {
    /// Apply mutation events to a child genome and return a summary.
    ///
    /// `parent_reachable_nodes` is the parent's cached reachable set (sorted ascending),
    /// used to bias mutation target selection toward functional structure.
    /// The parent carries no executed set here, so the executed layer never fires.
    ///
    /// Accounting invariant: `summary.attempted_events == summary.applied_events + summary.skipped_events`.
    #[cfg(test)]
    pub fn apply_mutations(
        genome: &mut CreatureGenome,
        config: &MutationConfig,
        parent_reachable_nodes: &[usize],
        rng: &mut impl Rng,
    ) -> MutationSummary {
        Self::apply_mutations_with_food_type_count(
            genome,
            config,
            parent_reachable_nodes,
            ParentExecuted::NONE,
            rng,
            1,
        )
    }

    /// Apply mutation events to a child genome using the configured number of
    /// available food types for typed input-ref/topology sampling.
    #[allow(
        clippy::too_many_lines,
        reason = "one match arm per mutation event kind; splitting it would spread \
                  the operator dispatch table across several functions"
    )]
    pub fn apply_mutations_with_food_type_count(
        genome: &mut CreatureGenome,
        config: &MutationConfig,
        parent_reachable_nodes: &[usize],
        parent_executed: ParentExecuted<'_>,
        rng: &mut impl Rng,
        food_type_count: usize,
    ) -> MutationSummary {
        // The genome is immutable after birth, so its size read once here is
        // the count's population.
        let event_count = requested_event_count(config, genome.genome_size(), rng);
        if event_count == 0 {
            return MutationSummary::zero();
        }

        // Genome size pressure: compute once before the event loop.
        let restricted = config.genome_size_pressure_enabled
            && pressure::is_restricted(genome.genome_size(), config.genome_size_cap, rng);

        // The parent's recently executed nodes, derived once for a birth that
        // draws events. Under size-pressure restriction the executed layer is
        // off, so the inverted reachable bias prunes unreachable structure
        // first and the executed core is never targeted for removal.
        let executed = parent_executed.resolve(config.executed_window_ticks);
        let executed_bias = if restricted {
            0.0
        } else {
            config.executed_bias
        };
        let sets = TargetSets::new(parent_reachable_nodes, executed.as_ref());
        let selector = |domain_bias: f64| {
            sets.selector(
                pressure_adjusted_bias(domain_bias, restricted),
                executed_bias,
            )
        };

        let mut summary = MutationSummary::zero();
        // Reused across events: the ids the genome carries before each event,
        // so a selected index can be recorded as the node it named (T13.F01).
        let mut node_ids: Vec<NodeId> = Vec::with_capacity(genome.nodes.len());
        for _ in 0..event_count {
            node_ids.clear();
            node_ids.extend(genome.nodes.iter().map(|node| node.node_id));
            // Every operator this event tried and threw away for reporting no
            // applicable site, with the node it first selected (T13.F01).
            let mut discarded: Vec<(MutationOperator, Option<NodeId>)> = Vec::new();
            // Two-layer dispatch: mesh (Topology) vs node-internal (VM/Graph/InputRef).
            let rb = &config.reachable_bias;
            let (domain, selected) = if rng.gen_bool(config.mesh_layer_probability) {
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
                    let operator = topology_operator_key(op);
                    let tracked_before = if operator_requires_added_node_input_tracking(operator) {
                        Some(genome.clone())
                    } else {
                        None
                    };
                    let mut targets = selector(rb.topology);
                    let result = apply_topology_event(
                        genome,
                        op,
                        &mut targets,
                        rng,
                        config,
                        food_type_count,
                    );
                    if matches!(result, Err(MutationSkipReason::NoApplicableTarget)) {
                        discarded.push((
                            operator,
                            targets.first_pick().and_then(|i| node_ids.get(i).copied()),
                        ));
                        available.swap_remove(idx);
                        continue;
                    }
                    break Some(SelectedEvent {
                        operator,
                        tracked_before,
                        result,
                        executed_hits: targets.executed_hits(),
                        first_pick: targets.first_pick(),
                    });
                };
                (MutationDomain::Topology, selected)
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
                            let operator = vm_operator_key(op);
                            let tracked_before =
                                if operator_requires_added_node_input_tracking(operator) {
                                    Some(genome.clone())
                                } else {
                                    None
                                };
                            let mut targets = selector(rb.vm);
                            let result = apply_vm_event(genome, op, &mut targets, rng, config);
                            if matches!(result, Err(MutationSkipReason::NoApplicableTarget)) {
                                discarded.push((
                                    operator,
                                    targets.first_pick().and_then(|i| node_ids.get(i).copied()),
                                ));
                                available.swap_remove(idx);
                                continue;
                            }
                            break Some(SelectedEvent {
                                operator,
                                tracked_before,
                                result,
                                executed_hits: targets.executed_hits(),
                                first_pick: targets.first_pick(),
                            });
                        };
                        (MutationDomain::Vm, selected)
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
                            let operator = graph_operator_key(op);
                            let tracked_before =
                                if operator_requires_added_node_input_tracking(operator) {
                                    Some(genome.clone())
                                } else {
                                    None
                                };
                            let mut targets = selector(rb.graph);
                            let result = apply_graph_event(genome, op, &mut targets, rng, config);
                            if matches!(result, Err(MutationSkipReason::NoApplicableTarget)) {
                                discarded.push((
                                    operator,
                                    targets.first_pick().and_then(|i| node_ids.get(i).copied()),
                                ));
                                available.swap_remove(idx);
                                continue;
                            }
                            break Some(SelectedEvent {
                                operator,
                                tracked_before,
                                result,
                                executed_hits: targets.executed_hits(),
                                first_pick: targets.first_pick(),
                            });
                        };
                        (MutationDomain::Graph, selected)
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
                            let operator = input_ref_operator_key(op);
                            let tracked_before =
                                if operator_requires_added_node_input_tracking(operator) {
                                    Some(genome.clone())
                                } else {
                                    None
                                };
                            let mut targets = selector(rb.input_ref);
                            let result = apply_input_ref_event(
                                genome,
                                op,
                                &mut targets,
                                rng,
                                config,
                                food_type_count,
                            );
                            if matches!(result, Err(MutationSkipReason::NoApplicableTarget)) {
                                discarded.push((
                                    operator,
                                    targets.first_pick().and_then(|i| node_ids.get(i).copied()),
                                ));
                                available.swap_remove(idx);
                                continue;
                            }
                            break Some(SelectedEvent {
                                operator,
                                tracked_before,
                                result,
                                executed_hits: targets.executed_hits(),
                                first_pick: targets.first_pick(),
                            });
                        };
                        (MutationDomain::InputRef, selected)
                    }
                }
            };
            let Some(SelectedEvent {
                operator,
                tracked_before,
                result,
                executed_hits,
                first_pick,
            }) = selected
            else {
                // Every operator of the drawn domain reported no applicable
                // site, so the attempt belongs to the domain, not an operator.
                summary.record_domain_skip(domain, MutationSkipReason::NoApplicableTarget);
                summary.record_event(MutationEventRecord {
                    domain,
                    operator: None,
                    // The first node any discarded operator selected, which is
                    // what separates "selected, no applicable site" from
                    // "no eligible node".
                    target: discarded.iter().find_map(|&(_, pick)| pick),
                    outcome: MutationEventOutcome::Skipped(MutationSkipReason::NoApplicableTarget),
                    discarded,
                });
                continue;
            };
            summary.record_attempt(domain, operator);
            summary.record_event(MutationEventRecord {
                domain,
                operator: Some(operator),
                target: first_pick.and_then(|index| node_ids.get(index).copied()),
                outcome: match result {
                    Ok(reachability) => MutationEventOutcome::Applied(reachability),
                    Err(reason) => MutationEventOutcome::Skipped(reason),
                },
                discarded,
            });

            match result {
                Ok(reachability) => {
                    summary.record_applied(domain, operator);
                    if let Some(before) = tracked_before.as_ref() {
                        if let Some(classes) =
                            collect_added_node_input_classes(before, genome, operator)
                        {
                            summary.record_added_node_input_classes(operator, &classes);
                        }
                        if let Some(keys) =
                            collect_added_node_world_inputs_for_operator(before, genome, operator)
                        {
                            summary.record_added_node_world_inputs(operator, &keys);
                        }
                    }
                    summary.record_reachability(reachability);
                    summary.record_executed_targets(executed_hits);
                }
                Err(reason) => summary.record_skipped(operator, reason),
            }
        }

        summary
    }
}

/// Under active genome-size pressure restriction, invert reachability targeting so
/// decreasing operators preferentially prune unreachable structure first.
fn pressure_adjusted_bias(base_bias: f64, restricted: bool) -> f64 {
    if restricted {
        -base_bias
    } else {
        base_bias
    }
}

fn operator_requires_added_node_input_tracking(operator: MutationOperator) -> bool {
    matches!(
        operator,
        MutationOperator::TopologyAddNode
            | MutationOperator::TopologySpliceNode
            | MutationOperator::GraphAddInternalGraphNode
    )
}

fn classify_added_node_input_classes(
    input_refs: &[crate::contracts::InputReference],
) -> Vec<MutationAddedNodeInputClass> {
    let classes: BTreeSet<_> = input_refs
        .iter()
        .map(MutationAddedNodeInputClass::from)
        .collect();
    if classes.is_empty() {
        vec![MutationAddedNodeInputClass::None]
    } else {
        classes.into_iter().collect()
    }
}

fn collect_added_node_world_inputs(
    input_refs: &[crate::contracts::InputReference],
) -> Vec<crate::contracts::WorldInputKey> {
    input_refs
        .iter()
        .filter_map(|input_ref| match input_ref {
            crate::contracts::InputReference::World(key) => Some(*key),
            _ => None,
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn collect_added_node_input_classes(
    before: &CreatureGenome,
    after: &CreatureGenome,
    operator: MutationOperator,
) -> Option<Vec<MutationAddedNodeInputClass>> {
    match operator {
        MutationOperator::TopologyAddNode | MutationOperator::TopologySpliceNode => {
            let before_ids: BTreeSet<_> = before.nodes.iter().map(|node| node.node_id).collect();
            let newborn = after
                .nodes
                .iter()
                .find(|node| !before_ids.contains(&node.node_id))?;
            Some(classify_added_node_input_classes(&newborn.input_refs))
        }
        MutationOperator::GraphAddInternalGraphNode => {
            let target = before.nodes.iter().zip(after.nodes.iter()).find_map(
                |(before_node, after_node)| match (
                    &before_node.backend_def,
                    &after_node.backend_def,
                ) {
                    (BackendDef::Graph(before_graph), BackendDef::Graph(after_graph))
                        if after_graph.compute_nodes.len()
                            == before_graph.compute_nodes.len() + 1 =>
                    {
                        Some(after_node)
                    }
                    _ => None,
                },
            )?;
            Some(classify_added_node_input_classes(&target.input_refs))
        }
        _ => None,
    }
}

fn collect_added_node_world_inputs_for_operator(
    before: &CreatureGenome,
    after: &CreatureGenome,
    operator: MutationOperator,
) -> Option<Vec<crate::contracts::WorldInputKey>> {
    match operator {
        MutationOperator::TopologyAddNode | MutationOperator::TopologySpliceNode => {
            let before_ids: BTreeSet<_> = before.nodes.iter().map(|node| node.node_id).collect();
            let newborn = after
                .nodes
                .iter()
                .find(|node| !before_ids.contains(&node.node_id))?;
            Some(collect_added_node_world_inputs(&newborn.input_refs))
        }
        MutationOperator::GraphAddInternalGraphNode => {
            let target = before.nodes.iter().zip(after.nodes.iter()).find_map(
                |(before_node, after_node)| match (
                    &before_node.backend_def,
                    &after_node.backend_def,
                ) {
                    (BackendDef::Graph(before_graph), BackendDef::Graph(after_graph))
                        if after_graph.compute_nodes.len()
                            == before_graph.compute_nodes.len() + 1 =>
                    {
                        Some(after_node)
                    }
                    _ => None,
                },
            )?;
            Some(collect_added_node_world_inputs(&target.input_refs))
        }
        _ => None,
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
    targets: &mut TargetSelector<'_>,
    rng: &mut impl Rng,
    config: &MutationConfig,
    food_type_count: usize,
) -> Result<TargetReachability, MutationSkipReason> {
    let snapshot = genome.clone();
    match TopologyMutator::apply_with_food_type_count(
        genome,
        op,
        targets,
        rng,
        config,
        food_type_count,
    ) {
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
    targets: &mut TargetSelector<'_>,
    rng: &mut impl Rng,
    config: &MutationConfig,
) -> Result<TargetReachability, MutationSkipReason> {
    let snapshot = genome.clone();
    match VmMutator::apply(genome, op, targets, rng, config) {
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
    targets: &mut TargetSelector<'_>,
    rng: &mut impl Rng,
    config: &MutationConfig,
) -> Result<TargetReachability, MutationSkipReason> {
    let snapshot = genome.clone();
    match GraphMutator::apply(genome, op, targets, rng, config) {
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
    targets: &mut TargetSelector<'_>,
    rng: &mut impl Rng,
    config: &MutationConfig,
    food_type_count: usize,
) -> Result<TargetReachability, MutationSkipReason> {
    let snapshot = genome.clone();
    match InputRefMutator::apply_with_food_type_count(
        genome,
        op,
        targets,
        rng,
        config,
        food_type_count,
    ) {
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

pub(crate) fn topology_operator_key(op: TopologyOperator) -> MutationOperator {
    match op {
        TopologyOperator::AddNode => MutationOperator::TopologyAddNode,
        TopologyOperator::RemoveNode => MutationOperator::TopologyRemoveNode,
        TopologyOperator::RetargetNodeTarget => MutationOperator::TopologyRetargetNodeTarget,
        TopologyOperator::AddRouteTarget => MutationOperator::TopologyAddRouteTarget,
        TopologyOperator::RemoveRouteTarget => MutationOperator::TopologyRemoveRouteTarget,
        TopologyOperator::ChangeEntryNode => MutationOperator::TopologyChangeEntryNode,
        TopologyOperator::SwapNodeBackend => MutationOperator::TopologySwapNodeBackend,
        TopologyOperator::CopyNode => MutationOperator::TopologyCopyNode,
        TopologyOperator::CopyMeshBackwardSlice => MutationOperator::TopologyCopyMeshBackwardSlice,
        TopologyOperator::CopyMeshForwardSlice => MutationOperator::TopologyCopyMeshForwardSlice,
        TopologyOperator::SpliceNode => MutationOperator::TopologySpliceNode,
        TopologyOperator::SwapRouteTargets => MutationOperator::TopologySwapRouteTargets,
        TopologyOperator::MutateGateBias => MutationOperator::TopologyMutateGateBias,
    }
}

pub(crate) fn vm_operator_key(op: VmOperator) -> MutationOperator {
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

pub(crate) fn graph_operator_key(op: GraphOperator) -> MutationOperator {
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

pub(crate) fn input_ref_operator_key(op: InputRefOperator) -> MutationOperator {
    match op {
        InputRefOperator::Add => MutationOperator::InputRefAdd,
        InputRefOperator::Prune => MutationOperator::InputRefPrune,
        InputRefOperator::Swap => MutationOperator::InputRefSwap,
        InputRefOperator::RawFieldMutation => MutationOperator::InputRefRawFieldMutation,
    }
}

#[cfg(test)]
mod tests;
