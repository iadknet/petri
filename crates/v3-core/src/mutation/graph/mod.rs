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
    BirthEdgeClass, BundleEdits, EventReport, MutationSkipReason, TargetReachability,
};

/// Graph mutation operator variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

    /// The operator's stable key, `<Domain>.<Name>`: benchmark summary
    /// keys, server payloads and the serde form of `MutationOperator`.
    #[must_use]
    pub const fn as_key(self) -> &'static str {
        match self {
            Self::AlterGraphEdgeWeight => "Graph.AlterGraphEdgeWeight",
            Self::SwapGraphOperator => "Graph.SwapGraphOperator",
            Self::MutateGraphOperatorParam => "Graph.MutateGraphOperatorParam",
            Self::AddInternalGraphNode => "Graph.AddInternalGraphNode",
            Self::RemoveInternalGraphNode => "Graph.RemoveInternalGraphNode",
            Self::AddGraphEdge => "Graph.AddGraphEdge",
            Self::RetargetGraphEdge => "Graph.RetargetGraphEdge",
            Self::RemoveGraphEdge => "Graph.RemoveGraphEdge",
            Self::GraphRawFieldMutation => "Graph.GraphRawFieldMutation",
            Self::CopyInternalNode => "Graph.CopyInternalNode",
            Self::CopySubgraph => "Graph.CopySubgraph",
            Self::CopyNodeInputs => "Graph.CopyNodeInputs",
            Self::EnableHebbian => "Graph.EnableHebbian",
            Self::DisableHebbian => "Graph.DisableHebbian",
            Self::MutateHebbianRule => "Graph.MutateHebbianRule",
            Self::MutateHebbianRate => "Graph.MutateHebbianRate",
            Self::ToggleHebbianLamarckian => "Graph.ToggleHebbianLamarckian",
            Self::EnableRewardModulation => "Graph.EnableRewardModulation",
            Self::DisableRewardModulation => "Graph.DisableRewardModulation",
            Self::MutateRewardSource => "Graph.MutateRewardSource",
            Self::MutateTraceDecay => "Graph.MutateTraceDecay",
            Self::AddBundleLive => "Graph.AddBundleLive",
            Self::AddBundleSilent => "Graph.AddBundleSilent",
            Self::AlterBundleScale => "Graph.AlterBundleScale",
            Self::ShiftBundleOffset => "Graph.ShiftBundleOffset",
            Self::RetargetBundle => "Graph.RetargetBundle",
            Self::RemoveBundle => "Graph.RemoveBundle",
            Self::SetScope => "Graph.SetScope",
            Self::DetachGraphEdge => "Graph.DetachGraphEdge",
            Self::EnableRetention => "Graph.EnableRetention",
            Self::DisableRetention => "Graph.DisableRetention",
            Self::MutateRetention => "Graph.MutateRetention",
            Self::EnableBundleRetention => "Graph.EnableBundleRetention",
            Self::DisableBundleRetention => "Graph.DisableBundleRetention",
            Self::MutateBundleRetention => "Graph.MutateBundleRetention",
        }
    }

    /// Whether this operator increases, decreases, or preserves genome complexity.
    #[must_use]
    pub const fn complexity_effect(self) -> crate::mutation::types::ComplexityEffect {
        use crate::mutation::types::ComplexityEffect;
        match self {
            Self::AddInternalGraphNode
            | Self::AddGraphEdge
            | Self::CopyInternalNode
            | Self::CopySubgraph
            | Self::CopyNodeInputs
            | Self::EnableHebbian
            | Self::EnableRewardModulation
            | Self::AddBundleLive
            | Self::AddBundleSilent => ComplexityEffect::Increasing,
            Self::RemoveInternalGraphNode
            | Self::RemoveGraphEdge
            | Self::DisableHebbian
            | Self::DisableRewardModulation
            | Self::RemoveBundle => ComplexityEffect::Decreasing,
            Self::AlterGraphEdgeWeight
            | Self::SwapGraphOperator
            | Self::MutateGraphOperatorParam
            | Self::RetargetGraphEdge
            | Self::GraphRawFieldMutation
            | Self::MutateHebbianRule
            | Self::MutateHebbianRate
            | Self::ToggleHebbianLamarckian
            | Self::MutateRewardSource
            | Self::MutateTraceDecay
            | Self::AlterBundleScale
            | Self::ShiftBundleOffset
            | Self::RetargetBundle
            | Self::SetScope
            | Self::DetachGraphEdge
            | Self::EnableRetention
            | Self::DisableRetention
            | Self::MutateRetention
            | Self::EnableBundleRetention
            | Self::DisableBundleRetention
            | Self::MutateBundleRetention => ComplexityEffect::Neutral,
        }
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

    /// [`Self::apply_classified`] reachability only: the tests' entry.
    #[cfg(test)]
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

    /// Apply a graph operator to the genome under the world's sensor
    /// context, with what the event reports beside its reachability
    /// ([`EventReport`]).
    ///
    /// # Errors
    /// `MutationSkipReason::NoApplicableTarget` if no Graph-backend node has
    /// a site this operator can apply to. The skip happens before the draw,
    /// so the engine records it as no-eligible-node with no selected target.
    pub(crate) fn apply_classified(
        genome: &mut CreatureGenome,
        op: GraphOperator,
        targets: &mut TargetSelector<'_>,
        rng: &mut impl Rng,
        config: &MutationConfig,
        context: &SensorContext,
    ) -> Result<(TargetReachability, EventReport), MutationSkipReason> {
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
    ) -> Result<EventReport, MutationSkipReason> {
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
            GraphOperator::AddGraphEdge => Ok(EventReport {
                class: operators::add_graph_edge(genome, node_idx, context, rng)?,
                ..EventReport::default()
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
fn unreported(result: Result<(), MutationSkipReason>) -> Result<EventReport, MutationSkipReason> {
    result.map(|()| EventReport::default())
}

/// An attribute change reporting the scope it landed on.
fn scoped(result: Result<Scope, MutationSkipReason>) -> Result<EventReport, MutationSkipReason> {
    result.map(|scope| EventReport {
        attribute_scope: Some(scope),
        ..EventReport::default()
    })
}

/// A bundle-touching event reporting its edits.
fn edited(
    result: Result<BundleEdits, MutationSkipReason>,
) -> Result<EventReport, MutationSkipReason> {
    result.map(|bundle_edits| EventReport {
        bundle_edits,
        ..EventReport::default()
    })
}

/// A creation reporting its bundle edits and the class of its one `Plain`
/// bundle's sink, if any (T25.F11 invariant 8).
fn created(
    result: Result<(BundleEdits, Option<BirthEdgeClass>), MutationSkipReason>,
) -> Result<EventReport, MutationSkipReason> {
    result.map(|(bundle_edits, class)| EventReport {
        class,
        bundle_edits,
        ..EventReport::default()
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
