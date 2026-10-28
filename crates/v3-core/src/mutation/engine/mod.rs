use std::collections::BTreeSet;

use rand::Rng;

use crate::config::MutationConfig;
use crate::contracts::{NodeId, SensorContext};
use crate::creature::parseability::ParseabilityGate;
use crate::genome::cgp::{validate_graph, Scope};
use crate::genome::{BackendDef, CreatureGenome};
use crate::mutation::graph::{GraphMutator, GraphOperator};
use crate::mutation::input_ref::{InputRefMutator, InputRefOperator};
use crate::mutation::pressure;
use crate::mutation::reachability::{BirthMembership, ParentExecuted, TargetSelector};
use crate::mutation::topology::{TopologyMutator, TopologyOperator};
use crate::mutation::types::{
    BirthEdgeClass, BundleEdits, MutationAddedNodeInputClass, MutationDomain, MutationEventOutcome,
    MutationEventRecord, MutationOperator, MutationSkipReason, MutationSummary, TargetReachability,
};
use crate::mutation::vm::{VmMutator, VmOperator};

/// Draw requested supply from normalized config, independently of operator success.
///
/// The one supply rule (T11.F19, T11.F20): one Bernoulli trial per unit at
/// `per_unit_rate`, so the count is `Binomial(units, per_unit_rate)` with no
/// trigger, minimum, maximum, or continuation. The caller supplies `units`.
fn requested_event_count(config: &MutationConfig, units: u32, rng: &mut impl Rng) -> u32 {
    (0..units)
        .map(|_| u32::from(rng.gen_bool(config.per_unit_rate)))
        .sum()
}

/// An event's reachability and, for `AddGraphEdge` and `InputRef.Add`, the
/// class of the vote-sink edge it pushed (T11.F30).
type ClassifiedResult = Result<(TargetReachability, Option<BirthEdgeClass>), MutationSkipReason>;

/// What one attempted mutation event produced, once an operator of the drawn
/// domain accepted the attempt.
struct SelectedEvent {
    operator: MutationOperator,
    // WORKAROUND(T30.F11): operators return the node they added, not a
    // whole-genome clone taken before the event (at each domain's draw) and
    // diffed after it.
    tracked_before: Option<CreatureGenome>,
    /// The reachability and, for `AddGraphEdge` and `InputRef.Add`, the class
    /// of the vote-sink edge the event pushed (T11.F30).
    result: ClassifiedResult,
    /// What the event did to bundles, reported by the operator and counted when it
    /// commits (T25.F03); zero outside the Graph domain.
    bundle_edits: BundleEdits,
    /// The scope an attribute operator's change landed on, when it commits
    /// (T25.F08).
    attribute_scope: Option<Scope>,
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
        Self::apply_mutations_in(
            genome,
            config,
            parent_reachable_nodes,
            ParentExecuted::NONE,
            rng,
            &crate::config::SimulationConfig::default().sensor_context(1),
        )
    }

    /// Apply mutation events to a child genome using the configured number of
    /// available food types for typed input-ref/topology sampling.
    ///
    /// Every production birth and reading comes through here: the supply draw
    /// runs on the genome's own `genome_size()`, read once before any event.
    /// The child starts as the parent's copy, so this is the parent's size.
    pub fn apply_mutations_in(
        genome: &mut CreatureGenome,
        config: &MutationConfig,
        parent_reachable_nodes: &[usize],
        parent_executed: ParentExecuted<'_>,
        rng: &mut impl Rng,
        context: &SensorContext,
    ) -> MutationSummary {
        let units = genome.genome_size();
        Self::apply_mutations_on_units(
            genome,
            units,
            config,
            parent_reachable_nodes,
            parent_executed,
            rng,
            context,
        )
    }

    /// [`Self::apply_mutations_in`] with the supply draw on
    /// a caller-supplied unit count: `Binomial(units, per_unit_rate)`
    /// requested events. The drift walk and `recruitment_paths` pass the
    /// instrument constant `FOUNDER_GENOME_SIZE_UNITS` (T11.F20); engine
    /// tests pass explicit counts (rate 1.0 on N units requests exactly N).
    ///
    /// `parent_reachable_nodes` and `parent_executed` are sorted parent node
    /// indices. Every target draw sees them through [`BirthMembership`]: a
    /// child node counts as a member only when the parent carried it, however
    /// earlier events of this birth removed or added nodes (T11.F24).
    #[allow(
        clippy::too_many_lines,
        reason = "one match arm per mutation event kind; splitting it would spread \
                  the operator dispatch table across several functions"
    )]
    pub fn apply_mutations_on_units(
        genome: &mut CreatureGenome,
        units: u32,
        config: &MutationConfig,
        parent_reachable_nodes: &[usize],
        parent_executed: ParentExecuted<'_>,
        rng: &mut impl Rng,
        context: &SensorContext,
    ) -> MutationSummary {
        let event_count = requested_event_count(config, units, rng);
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
        // The parent's sets are the targeting policy (T11.F17); which current
        // child nodes are their members follows the birth's removals and
        // additions (T11.F24).
        let mut membership = BirthMembership::new(
            parent_reachable_nodes,
            executed.as_ref(),
            genome.nodes.len(),
        );

        let mut summary = MutationSummary::zero();
        // Reused across events: the ids the genome carries before each event,
        // so a selected index can be recorded as the node it named (T13.F01)
        // and membership can follow the event.
        let mut node_ids: Vec<NodeId> = Vec::with_capacity(genome.nodes.len());
        for _ in 0..event_count {
            node_ids.clear();
            node_ids.extend(genome.nodes.iter().map(|node| node.node_id));
            let sets = membership.sets();
            let selector = |domain_bias: f64| {
                sets.selector(
                    pressure_adjusted_bias(domain_bias, restricted),
                    executed_bias,
                )
            };
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
                    .filter(|op| op.tuned_weight(config.large_copy_weight_percent) > 0)
                    .collect();
                let selected = loop {
                    if available.is_empty() {
                        break None;
                    }
                    let idx = select_weighted_index(
                        &available,
                        |op| op.tuned_weight(config.large_copy_weight_percent),
                        rng,
                    );
                    let op = available[idx];
                    let operator = topology_operator_key(op);
                    let tracked_before = if operator_requires_added_node_input_tracking(operator) {
                        Some(genome.clone())
                    } else {
                        None
                    };
                    let mut targets = selector(rb.topology);
                    let result =
                        apply_topology_event(genome, op, &mut targets, rng, config, context)
                            .map(|reachability| (reachability, None));
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
                        bundle_edits: BundleEdits::default(),
                        attribute_scope: None,
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
                            let result = apply_vm_event(genome, op, &mut targets, rng, config)
                                .map(|reachability| (reachability, None));
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
                                bundle_edits: BundleEdits::default(),
                                attribute_scope: None,
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
                            let operator = op.operator();
                            let tracked_before =
                                if operator_requires_added_node_input_tracking(operator) {
                                    Some(genome.clone())
                                } else {
                                    None
                                };
                            let mut targets = selector(rb.graph);
                            let (result, bundle_edits, attribute_scope) =
                                apply_graph_event(genome, op, &mut targets, rng, config, context);
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
                                bundle_edits,
                                attribute_scope,
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
                            let (result, bundle_edits) = apply_input_ref_event(
                                genome,
                                op,
                                &mut targets,
                                rng,
                                config,
                                context,
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
                                bundle_edits,
                                attribute_scope: None,
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
                bundle_edits,
                attribute_scope,
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
                    Ok((reachability, _)) => MutationEventOutcome::Applied(reachability),
                    Err(reason) => MutationEventOutcome::Skipped(reason),
                },
                discarded,
            });

            match result {
                Ok((reachability, edge_class)) => {
                    summary.record_applied(domain, operator);
                    summary.record_bundle_edits(operator, bundle_edits);
                    if let Some(scope) = attribute_scope {
                        summary.record_attribute_scope(scope);
                    }
                    if let Some(class) = edge_class {
                        summary.record_vote_edge_class(class);
                    }
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
                    membership.observe_event(&node_ids, &genome.nodes);
                }
                // A skipped attempt restored the genome, so membership stands.
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
                        if after_graph.compute_nodes().len()
                            == before_graph.compute_nodes().len() + 1 =>
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
                        if after_graph.compute_nodes().len()
                            == before_graph.compute_nodes().len() + 1 =>
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

/// Apply one mutation event with the parseability and graph-invariant gate:
/// snapshot the genome (birth seeds included), apply, and commit when the
/// event applied, the genome still parses and every Graph backend passes
/// `validate_graph` (T25.F03); otherwise restore the snapshot. A rejected
/// event leaves the draft, its seeds included, exactly as it was. Shared by
/// all four domains.
fn apply_gated<T>(
    genome: &mut CreatureGenome,
    apply: impl FnOnce(&mut CreatureGenome) -> Result<T, MutationSkipReason>,
) -> Result<T, MutationSkipReason> {
    let snapshot = genome.clone();
    let result = match apply(genome) {
        Ok(_) if ParseabilityGate::validate(genome).is_err() => {
            Err(MutationSkipReason::ParseabilityViolation)
        }
        Ok(_)
            if genome
                .nodes
                .iter()
                .any(|node| validate_graph(node).is_err()) =>
        {
            Err(MutationSkipReason::GraphInvariantViolation)
        }
        result => result,
    };
    if result.is_err() {
        *genome = snapshot;
    }
    result
}

/// Apply one topology mutation event with parseability gate.
fn apply_topology_event(
    genome: &mut CreatureGenome,
    op: TopologyOperator,
    targets: &mut TargetSelector<'_>,
    rng: &mut impl Rng,
    config: &MutationConfig,
    context: &SensorContext,
) -> Result<TargetReachability, MutationSkipReason> {
    apply_gated(genome, |genome| {
        TopologyMutator::apply_in(genome, op, targets, rng, config, context)
    })
}

/// Apply one VM mutation event with parseability gate.
fn apply_vm_event(
    genome: &mut CreatureGenome,
    op: VmOperator,
    targets: &mut TargetSelector<'_>,
    rng: &mut impl Rng,
    config: &MutationConfig,
) -> Result<TargetReachability, MutationSkipReason> {
    apply_gated(genome, |genome| {
        VmMutator::apply(genome, op, targets, rng, config)
    })
}

/// Apply one graph mutation event with the gate, and the detaches and bundle
/// removals it made, counted only when it commits: a rolled-back event
/// reports none (T25.F03), nor an attribute scope (T25.F08).
fn apply_graph_event(
    genome: &mut CreatureGenome,
    op: GraphOperator,
    targets: &mut TargetSelector<'_>,
    rng: &mut impl Rng,
    config: &MutationConfig,
    context: &SensorContext,
) -> (ClassifiedResult, BundleEdits, Option<Scope>) {
    let result = apply_gated(genome, |genome| {
        GraphMutator::apply_classified(genome, op, targets, rng, config, context)
    });
    let edits = result
        .as_ref()
        .map_or(BundleEdits::default(), |(_, report)| report.bundle_edits);
    let attribute_scope = result
        .as_ref()
        .ok()
        .and_then(|(_, report)| report.attribute_scope);
    (
        result.map(|(reachability, report)| (reachability, report.class)),
        edits,
        attribute_scope,
    )
}

/// Apply one input ref mutation event with parseability gate, and the bundle edits
/// it made, counted only when it commits.
fn apply_input_ref_event(
    genome: &mut CreatureGenome,
    op: InputRefOperator,
    targets: &mut TargetSelector<'_>,
    rng: &mut impl Rng,
    config: &MutationConfig,
    context: &SensorContext,
) -> (ClassifiedResult, BundleEdits) {
    let result = apply_gated(genome, |genome| {
        InputRefMutator::apply_classified(genome, op, targets, rng, config, context)
    });
    let edits = result
        .as_ref()
        .map_or(BundleEdits::default(), |(_, (_, edits))| *edits);
    (
        result.map(|(reachability, (class, _))| (reachability, class)),
        edits,
    )
}

pub(crate) fn topology_operator_key(op: TopologyOperator) -> MutationOperator {
    match op {
        TopologyOperator::AddNode => MutationOperator::TopologyAddNode,
        TopologyOperator::RemoveNode => MutationOperator::TopologyRemoveNode,
        TopologyOperator::RetargetNodeTarget => MutationOperator::TopologyRetargetNodeTarget,
        TopologyOperator::AddRouteTarget => MutationOperator::TopologyAddRouteTarget,
        TopologyOperator::AddFeedbackRoute => MutationOperator::TopologyAddFeedbackRoute,
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

pub(crate) fn input_ref_operator_key(op: InputRefOperator) -> MutationOperator {
    match op {
        InputRefOperator::AddSensorLive => MutationOperator::InputRefAddSensorLive,
        InputRefOperator::AddSensorSilent => MutationOperator::InputRefAddSensorSilent,
        InputRefOperator::Prune => MutationOperator::InputRefPrune,
        InputRefOperator::Swap => MutationOperator::InputRefSwap,
        InputRefOperator::RawFieldMutation => MutationOperator::InputRefRawFieldMutation,
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod membership_tests;

#[cfg(test)]
mod bundle_tests;

#[cfg(test)]
mod draw_stream_tests;
