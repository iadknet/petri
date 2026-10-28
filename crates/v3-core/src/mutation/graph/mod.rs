pub(crate) mod bundle;
pub mod hebbian;
pub(crate) mod operators;

use rand::Rng;

use crate::config::MutationConfig;
use crate::contracts::SensorContext;
use crate::genome::cgp::{CgpGraphBackendDef, Scope};
use crate::genome::{BackendDef, CreatureGenome, NodeGenome};
use crate::mutation::reachability::TargetSelector;
use crate::mutation::types::{
    BirthEdgeClass, BundleEdits, MutationOperator, MutationSkipReason, TargetReachability,
};

/// What an applied Graph event reports beside its reachability.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct GraphEventReport {
    /// The class of the vote-sink edge an `AddGraphEdge` pushed (T11.F30).
    pub class: Option<BirthEdgeClass>,
    /// The scope the change of one of the thirteen attribute operators (the
    /// nine plasticity operators, `SetScope` and the three edge retention
    /// operators) landed on (T25.F08, T25.F09).
    pub attribute_scope: Option<Scope>,
    /// What the event did to bundles (T25.F11), from the container's
    /// return values.
    pub bundle_edits: BundleEdits,
}

/// Graph mutation operator variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GraphOperator {
    AlterGraphEdgeWeight,
    SwapGraphOperator,
    MutateGraphOperatorParam,
    AddInternalGraphNode,
    RemoveInternalGraphNode,
    AddGraphEdge,
    RetargetGraphEdge,
    RemoveGraphEdge,
    GraphRawFieldMutation,
    CopyInternalNode,
    CopySubgraph,
    CopyNodeInputs,
    EnableHebbian,
    DisableHebbian,
    MutateHebbianRule,
    MutateHebbianRate,
    ToggleHebbianLamarckian,
    EnableRewardModulation,
    DisableRewardModulation,
    MutateRewardSource,
    MutateTraceDecay,
    AddBundleLive,
    AddBundleSilent,
    AlterBundleScale,
    ShiftBundleOffset,
    RetargetBundle,
    RemoveBundle,
    SetScope,
    DetachGraphEdge,
    EnableRetention,
    DisableRetention,
    MutateRetention,
    EnableBundleRetention,
    DisableBundleRetention,
    MutateBundleRetention,
}

impl GraphOperator {
    pub const ALL: [Self; 35] = [
        Self::AlterGraphEdgeWeight,
        Self::SwapGraphOperator,
        Self::MutateGraphOperatorParam,
        Self::AddInternalGraphNode,
        Self::RemoveInternalGraphNode,
        Self::AddGraphEdge,
        Self::RetargetGraphEdge,
        Self::RemoveGraphEdge,
        Self::GraphRawFieldMutation,
        Self::CopyInternalNode,
        Self::CopySubgraph,
        Self::CopyNodeInputs,
        Self::EnableHebbian,
        Self::DisableHebbian,
        Self::MutateHebbianRule,
        Self::MutateHebbianRate,
        Self::ToggleHebbianLamarckian,
        Self::EnableRewardModulation,
        Self::DisableRewardModulation,
        Self::MutateRewardSource,
        Self::MutateTraceDecay,
        Self::AddBundleLive,
        Self::AddBundleSilent,
        Self::AlterBundleScale,
        Self::ShiftBundleOffset,
        Self::RetargetBundle,
        Self::RemoveBundle,
        Self::SetScope,
        Self::DetachGraphEdge,
        Self::EnableRetention,
        Self::DisableRetention,
        Self::MutateRetention,
        Self::EnableBundleRetention,
        Self::DisableBundleRetention,
        Self::MutateBundleRetention,
    ];

    /// Per-operator weight reflecting impact tier.
    /// 4 = refinement, 2 = moderate, 1 = structural.
    #[must_use]
    pub const fn weight(self) -> u8 {
        match self {
            Self::AlterGraphEdgeWeight => 4,
            Self::SwapGraphOperator => 2,
            Self::MutateGraphOperatorParam => 4,
            Self::AddInternalGraphNode => 1,
            Self::RemoveInternalGraphNode => 1,
            Self::AddGraphEdge => 2,
            Self::RetargetGraphEdge => 2,
            Self::RemoveGraphEdge => 2,
            Self::GraphRawFieldMutation => 4,
            Self::CopyInternalNode => 1,
            Self::CopySubgraph => 1,
            Self::CopyNodeInputs => 2,
            Self::EnableHebbian => 1,
            Self::DisableHebbian => 1,
            Self::MutateHebbianRule => 2,
            Self::MutateHebbianRate => 4,
            Self::ToggleHebbianLamarckian => 2,
            Self::EnableRewardModulation => 1,
            Self::DisableRewardModulation => 1,
            Self::MutateRewardSource => 2,
            Self::MutateTraceDecay => 4,
            // T25.F11: the two creation operators split the four retired
            // `Add*Group` operators' weight 8 evenly.
            Self::AddBundleLive | Self::AddBundleSilent => 4,
            Self::AlterBundleScale => 4,
            Self::ShiftBundleOffset => 2,
            Self::RetargetBundle => 2,
            Self::RemoveBundle => 2,
            // T25.F07: after `RemoveBundle`, at the tier of a moderate edit.
            Self::SetScope => 2,
            // T25.F08: after `SetScope`, detach at a moderate edit's tier.
            Self::DetachGraphEdge => 2,
            // T25.F09: after `DetachGraphEdge`, the enable and disable twins
            // at a structural tier and the steps at a refinement's, as
            // their trace-decay analogs.
            Self::EnableRetention | Self::DisableRetention => 1,
            Self::MutateRetention => 4,
            Self::EnableBundleRetention | Self::DisableBundleRetention => 1,
            Self::MutateBundleRetention => 4,
        }
    }

    /// The engine's operator key for this operator: the one table every
    /// per-operator classification of a Graph operator reads.
    #[must_use]
    pub const fn operator(self) -> MutationOperator {
        match self {
            Self::AlterGraphEdgeWeight => MutationOperator::GraphAlterGraphEdgeWeight,
            Self::SwapGraphOperator => MutationOperator::GraphSwapGraphOperator,
            Self::MutateGraphOperatorParam => MutationOperator::GraphMutateGraphOperatorParam,
            Self::AddInternalGraphNode => MutationOperator::GraphAddInternalGraphNode,
            Self::RemoveInternalGraphNode => MutationOperator::GraphRemoveInternalGraphNode,
            Self::AddGraphEdge => MutationOperator::GraphAddGraphEdge,
            Self::RetargetGraphEdge => MutationOperator::GraphRetargetGraphEdge,
            Self::RemoveGraphEdge => MutationOperator::GraphRemoveGraphEdge,
            Self::GraphRawFieldMutation => MutationOperator::GraphRawFieldMutation,
            Self::CopyInternalNode => MutationOperator::GraphCopyInternalNode,
            Self::CopySubgraph => MutationOperator::GraphCopySubgraph,
            Self::CopyNodeInputs => MutationOperator::GraphCopyNodeInputs,
            Self::EnableHebbian => MutationOperator::GraphEnableHebbian,
            Self::DisableHebbian => MutationOperator::GraphDisableHebbian,
            Self::MutateHebbianRule => MutationOperator::GraphMutateHebbianRule,
            Self::MutateHebbianRate => MutationOperator::GraphMutateHebbianRate,
            Self::ToggleHebbianLamarckian => MutationOperator::GraphToggleHebbianLamarckian,
            Self::EnableRewardModulation => MutationOperator::GraphEnableRewardModulation,
            Self::DisableRewardModulation => MutationOperator::GraphDisableRewardModulation,
            Self::MutateRewardSource => MutationOperator::GraphMutateRewardSource,
            Self::MutateTraceDecay => MutationOperator::GraphMutateTraceDecay,
            Self::AddBundleLive => MutationOperator::GraphAddBundleLive,
            Self::AddBundleSilent => MutationOperator::GraphAddBundleSilent,
            Self::AlterBundleScale => MutationOperator::GraphAlterBundleScale,
            Self::ShiftBundleOffset => MutationOperator::GraphShiftBundleOffset,
            Self::RetargetBundle => MutationOperator::GraphRetargetBundle,
            Self::RemoveBundle => MutationOperator::GraphRemoveBundle,
            Self::SetScope => MutationOperator::GraphSetScope,
            Self::DetachGraphEdge => MutationOperator::GraphDetachGraphEdge,
            Self::EnableRetention => MutationOperator::GraphEnableRetention,
            Self::DisableRetention => MutationOperator::GraphDisableRetention,
            Self::MutateRetention => MutationOperator::GraphMutateRetention,
            Self::EnableBundleRetention => MutationOperator::GraphEnableBundleRetention,
            Self::DisableBundleRetention => MutationOperator::GraphDisableBundleRetention,
            Self::MutateBundleRetention => MutationOperator::GraphMutateBundleRetention,
        }
    }

    /// Whether this operator increases, decreases, or preserves genome
    /// complexity, as its [`MutationOperator`] classifies it.
    #[must_use]
    pub const fn complexity_effect(self) -> crate::mutation::types::ComplexityEffect {
        self.operator().complexity_effect()
    }

    /// Whether this operator has a site to apply to on one Graph-backend
    /// node. Each arm delegates to the same enumeration the operator draws
    /// its target from. Numeric proposal rejection can still skip atomically.
    fn applies_to(self, def: &CgpGraphBackendDef, inputs: operators::NodeInputs<'_>) -> bool {
        match self {
            Self::AlterGraphEdgeWeight | Self::RetargetGraphEdge | Self::RemoveGraphEdge => {
                operators::has_edge_site(def)
            }
            Self::SwapGraphOperator | Self::RemoveInternalGraphNode => {
                operators::has_compute_node(def)
            }
            Self::MutateGraphOperatorParam => operators::has_parameterized_compute_node(def),
            Self::AddInternalGraphNode => operators::can_add_compute_node(def),
            Self::AddGraphEdge => operators::can_add_edge(def),
            Self::GraphRawFieldMutation => operators::has_raw_field_site(def, inputs),
            Self::CopyInternalNode => operators::can_copy_compute_node(def),
            Self::CopySubgraph => operators::can_copy_subgraph(def),
            Self::CopyNodeInputs => operators::can_copy_node_inputs(def),
            Self::EnableHebbian => hebbian::any_edge(def, hebbian::can_enable_hebbian),
            Self::DisableHebbian
            | Self::MutateHebbianRule
            | Self::MutateHebbianRate
            | Self::ToggleHebbianLamarckian => hebbian::any_edge(def, hebbian::is_plastic),
            Self::EnableRewardModulation => {
                hebbian::any_edge(def, hebbian::can_enable_reward_modulation)
            }
            Self::DisableRewardModulation | Self::MutateRewardSource | Self::MutateTraceDecay => {
                hebbian::any_edge(def, hebbian::is_reward_modulated)
            }
            Self::AddBundleLive | Self::AddBundleSilent => {
                bundle::bundle_sources(def, inputs.refs, inputs.context)
                    .next()
                    .is_some()
            }
            Self::AlterBundleScale | Self::RetargetBundle | Self::RemoveBundle => {
                bundle::has_bundle(def)
            }
            Self::ShiftBundleOffset => bundle::any_bundle(def, bundle::is_geometric),
            Self::SetScope | Self::DetachGraphEdge => hebbian::any_edge(def, hebbian::is_member),
            Self::EnableRetention => hebbian::any_edge(def, hebbian::can_enable_retention),
            Self::DisableRetention | Self::MutateRetention => {
                hebbian::any_edge(def, hebbian::is_decaying)
            }
            Self::EnableBundleRetention => bundle::any_bundle(def, bundle::retains),
            Self::DisableBundleRetention | Self::MutateBundleRetention => {
                bundle::any_bundle(def, bundle::is_decaying_bundle)
            }
        }
    }
}

/// Graph domain mutator.
pub struct GraphMutator;

impl GraphMutator {
    /// The Graph-backend node indices `op` can apply to, ascending.
    ///
    /// Selection draws from this set rather than from every Graph-backend
    /// node, so a refinement operator reaches a module that has a suitable
    /// site instead of landing on one that does not (T13.F03).
    pub(crate) fn applicable_indices(
        genome: &CreatureGenome,
        op: GraphOperator,
        context: &SensorContext,
    ) -> Vec<usize> {
        genome
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| match &n.backend_def {
                BackendDef::Graph(def) => op.applies_to(
                    def,
                    operators::NodeInputs {
                        refs: &n.input_refs,
                        context,
                    },
                ),
                BackendDef::Vm(_) => false,
            })
            .map(|(i, _)| i)
            .collect()
    }

    /// Apply a graph operator to the genome under the world's sensor
    /// context.
    ///
    /// Returns `Ok(TargetReachability)` on success, or `Err(MutationSkipReason::NoApplicableTarget)`
    /// if no Graph-backend node has a site this operator can apply to. The
    /// skip happens before the draw, so the engine records it as
    /// no-eligible-node with no selected target.
    pub fn apply_in(
        genome: &mut CreatureGenome,
        op: GraphOperator,
        targets: &mut TargetSelector<'_>,
        rng: &mut impl Rng,
        config: &MutationConfig,
        context: &SensorContext,
    ) -> Result<TargetReachability, MutationSkipReason> {
        Self::apply_classified(genome, op, targets, rng, config, context)
            .map(|(reachability, _)| reachability)
    }

    /// [`Self::apply_in`] with what the event reports
    /// beside its reachability ([`GraphEventReport`]); the same draws.
    pub(crate) fn apply_classified(
        genome: &mut CreatureGenome,
        op: GraphOperator,
        targets: &mut TargetSelector<'_>,
        rng: &mut impl Rng,
        config: &MutationConfig,
        context: &SensorContext,
    ) -> Result<(TargetReachability, GraphEventReport), MutationSkipReason> {
        let applicable = Self::applicable_indices(genome, op, context);
        if applicable.is_empty() {
            return Err(MutationSkipReason::NoApplicableTarget);
        }

        let (node_idx, reachability) = targets
            .select(&applicable, rng)
            .ok_or(MutationSkipReason::NoApplicableTarget)?;
        let report = Self::apply_to_node(genome, op, node_idx, rng, config, context)?;
        Ok((reachability, report))
    }

    /// Dispatch `op` onto one already-selected node, with what the event
    /// reports: the class of the vote-sink edge an `AddGraphEdge` pushed
    /// (T11.F30) and the scope an attribute operator's change landed on
    /// (T25.F08 invariant 8).
    pub(crate) fn apply_to_node(
        genome: &mut CreatureGenome,
        op: GraphOperator,
        node_idx: usize,
        rng: &mut impl Rng,
        _config: &MutationConfig,
        context: &SensorContext,
    ) -> Result<GraphEventReport, MutationSkipReason> {
        match op {
            GraphOperator::AlterGraphEdgeWeight => {
                unreported(operators::alter_edge_weight(genome, node_idx, rng))
            }
            GraphOperator::SwapGraphOperator => {
                unreported(operators::swap_operator(genome, node_idx, rng))
            }
            GraphOperator::MutateGraphOperatorParam => {
                unreported(operators::mutate_operator_param(genome, node_idx, rng))
            }
            GraphOperator::AddInternalGraphNode => {
                edited(operators::add_internal_node(genome, node_idx, context, rng))
            }
            GraphOperator::RemoveInternalGraphNode => {
                unreported(operators::remove_internal_node(genome, node_idx, rng))
            }
            GraphOperator::AddGraphEdge => Ok(GraphEventReport {
                class: operators::add_graph_edge(genome, node_idx, context, rng)?,
                ..GraphEventReport::default()
            }),
            GraphOperator::RetargetGraphEdge => edited(operators::retarget_graph_edge(
                genome, node_idx, context, rng,
            )),
            GraphOperator::RemoveGraphEdge => {
                edited(operators::remove_graph_edge(genome, node_idx, rng))
            }
            GraphOperator::GraphRawFieldMutation => edited(
                operators::apply_graph_raw_field_mutation(genome, node_idx, context, rng),
            ),
            GraphOperator::CopyInternalNode => {
                unreported(operators::apply_copy_internal_node(genome, node_idx, rng))
            }
            GraphOperator::CopySubgraph => {
                unreported(operators::apply_copy_subgraph(genome, node_idx, rng))
            }
            GraphOperator::CopyNodeInputs => {
                unreported(operators::apply_copy_node_inputs(genome, node_idx, rng))
            }
            GraphOperator::EnableHebbian => scoped(hebbian::enable_hebbian_in_def(
                operators::graph_def_mut(genome, node_idx)?,
                rng,
            )),
            GraphOperator::DisableHebbian => scoped(hebbian::disable_hebbian_in_def(
                operators::graph_def_mut(genome, node_idx)?,
                rng,
            )),
            GraphOperator::MutateHebbianRule => scoped(hebbian::mutate_hebbian_rule_in_def(
                operators::graph_def_mut(genome, node_idx)?,
                rng,
            )),
            GraphOperator::MutateHebbianRate => scoped(hebbian::mutate_hebbian_rate_in_def(
                operators::graph_def_mut(genome, node_idx)?,
                rng,
            )),
            GraphOperator::ToggleHebbianLamarckian => {
                scoped(hebbian::toggle_hebbian_lamarckian_in_def(
                    operators::graph_def_mut(genome, node_idx)?,
                    rng,
                ))
            }
            GraphOperator::EnableRewardModulation => {
                scoped(hebbian::enable_reward_modulation_in_def(
                    operators::graph_def_mut(genome, node_idx)?,
                    rng,
                ))
            }
            GraphOperator::DisableRewardModulation => {
                scoped(hebbian::disable_reward_modulation_in_def(
                    operators::graph_def_mut(genome, node_idx)?,
                    rng,
                ))
            }
            GraphOperator::MutateRewardSource => scoped(hebbian::mutate_reward_source_in_def(
                operators::graph_def_mut(genome, node_idx)?,
                rng,
            )),
            GraphOperator::MutateTraceDecay => scoped(hebbian::mutate_trace_decay_in_def(
                operators::graph_def_mut(genome, node_idx)?,
                rng,
            )),
            GraphOperator::AlterBundleScale => unreported(bundle::alter_scale(
                operators::graph_def_mut(genome, node_idx)?,
                rng,
            )),
            GraphOperator::ShiftBundleOffset => unreported(bundle::shift_offset(
                operators::graph_def_mut(genome, node_idx)?,
                rng,
            )),
            GraphOperator::RetargetBundle => unreported(bundle::retarget(
                operators::graph_def_mut(genome, node_idx)?,
                rng,
            )),
            GraphOperator::RemoveBundle => edited(bundle::remove(
                operators::graph_def_mut(genome, node_idx)?,
                rng,
            )),
            GraphOperator::SetScope => scoped(hebbian::set_scope_in_def(
                operators::graph_def_mut(genome, node_idx)?,
                rng,
            )),
            GraphOperator::AddBundleLive => created(bundle::add_bundle_by_law(
                node_mut(genome, node_idx)?,
                bundle::BirthLaw::Live,
                context,
                rng,
            )),
            GraphOperator::AddBundleSilent => created(bundle::add_bundle_by_law(
                node_mut(genome, node_idx)?,
                bundle::BirthLaw::Silent,
                context,
                rng,
            )),
            GraphOperator::DetachGraphEdge => {
                edited(bundle::detach_member(node_mut(genome, node_idx)?, rng))
            }
            GraphOperator::EnableRetention => scoped(hebbian::enable_retention_in_def(
                operators::graph_def_mut(genome, node_idx)?,
                rng,
            )),
            GraphOperator::DisableRetention => scoped(hebbian::disable_retention_in_def(
                operators::graph_def_mut(genome, node_idx)?,
                rng,
            )),
            GraphOperator::MutateRetention => scoped(hebbian::mutate_retention_in_def(
                operators::graph_def_mut(genome, node_idx)?,
                rng,
            )),
            GraphOperator::EnableBundleRetention => unreported(bundle::enable_retention(
                operators::graph_def_mut(genome, node_idx)?,
                rng,
            )),
            GraphOperator::DisableBundleRetention => unreported(bundle::disable_retention(
                operators::graph_def_mut(genome, node_idx)?,
                rng,
            )),
            GraphOperator::MutateBundleRetention => unreported(bundle::mutate_retention(
                operators::graph_def_mut(genome, node_idx)?,
                rng,
            )),
        }
    }
}

/// An event that reports nothing beside its reachability.
fn unreported(
    result: Result<(), MutationSkipReason>,
) -> Result<GraphEventReport, MutationSkipReason> {
    result.map(|()| GraphEventReport::default())
}

/// An attribute change reporting the scope it landed on.
fn scoped(
    result: Result<Scope, MutationSkipReason>,
) -> Result<GraphEventReport, MutationSkipReason> {
    result.map(|scope| GraphEventReport {
        attribute_scope: Some(scope),
        ..GraphEventReport::default()
    })
}

/// A bundle-touching event reporting its edits.
fn edited(
    result: Result<BundleEdits, MutationSkipReason>,
) -> Result<GraphEventReport, MutationSkipReason> {
    result.map(|bundle_edits| GraphEventReport {
        bundle_edits,
        ..GraphEventReport::default()
    })
}

/// A creation reporting its bundle edits and the class of its one `Plain`
/// bundle's sink, if any (T25.F11 invariant 8).
fn created(
    result: Result<(BundleEdits, Option<BirthEdgeClass>), MutationSkipReason>,
) -> Result<GraphEventReport, MutationSkipReason> {
    result.map(|(bundle_edits, class)| GraphEventReport {
        class,
        bundle_edits,
        ..GraphEventReport::default()
    })
}

/// The node at `node_idx`, or `NoApplicableTarget` past the end.
fn node_mut(
    genome: &mut CreatureGenome,
    node_idx: usize,
) -> Result<&mut NodeGenome, MutationSkipReason> {
    genome
        .nodes
        .get_mut(node_idx)
        .ok_or(MutationSkipReason::NoApplicableTarget)
}

#[cfg(test)]
mod tests;
