pub(crate) mod group;
pub mod hebbian;
pub(crate) mod operators;
pub(crate) mod recruitment;

use rand::Rng;

use crate::config::{MutationConfig, NeutralInputRecruitment};
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
    CopyEdgeBundle,
    EnableHebbian,
    DisableHebbian,
    MutateHebbianRule,
    MutateHebbianRate,
    ToggleHebbianLamarckian,
    EnableRewardModulation,
    DisableRewardModulation,
    MutateRewardSource,
    MutateTraceDecay,
    RecruitNeutralInput,
    AddRingGroup,
    AddDirectionPairGroup,
    AlterGroupScale,
    ShiftGroupOffset,
    RetargetGroupKind,
    RemoveGroup,
    SetScope,
    AddRandomizedRingGroup,
    AddRandomizedDirectionPairGroup,
    DetachGraphEdge,
    EnableRetention,
    DisableRetention,
    MutateRetention,
    EnableGroupRetention,
    DisableGroupRetention,
    MutateGroupRetention,
}

impl GraphOperator {
    pub const ALL: [Self; 38] = [
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
        Self::CopyEdgeBundle,
        Self::EnableHebbian,
        Self::DisableHebbian,
        Self::MutateHebbianRule,
        Self::MutateHebbianRate,
        Self::ToggleHebbianLamarckian,
        Self::EnableRewardModulation,
        Self::DisableRewardModulation,
        Self::MutateRewardSource,
        Self::MutateTraceDecay,
        Self::RecruitNeutralInput,
        Self::AddRingGroup,
        Self::AddDirectionPairGroup,
        Self::AlterGroupScale,
        Self::ShiftGroupOffset,
        Self::RetargetGroupKind,
        Self::RemoveGroup,
        Self::SetScope,
        Self::AddRandomizedRingGroup,
        Self::AddRandomizedDirectionPairGroup,
        Self::DetachGraphEdge,
        Self::EnableRetention,
        Self::DisableRetention,
        Self::MutateRetention,
        Self::EnableGroupRetention,
        Self::DisableGroupRetention,
        Self::MutateGroupRetention,
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
            Self::CopyEdgeBundle => 2,
            Self::EnableHebbian => 1,
            Self::DisableHebbian => 1,
            Self::MutateHebbianRule => 2,
            Self::MutateHebbianRate => 4,
            Self::ToggleHebbianLamarckian => 2,
            Self::EnableRewardModulation => 1,
            Self::DisableRewardModulation => 1,
            Self::MutateRewardSource => 2,
            Self::MutateTraceDecay => 4,
            Self::RecruitNeutralInput => 1,
            // Edge groups (T25.F03, at the catalog positions and weights of
            // the T11.F31 ring projection operators): each the tier of the
            // edge operator it parallels.
            Self::AddRingGroup => 2,
            // T25.F04: beside `AddRingGroup`, at its tier.
            Self::AddDirectionPairGroup => 2,
            Self::AlterGroupScale => 4,
            Self::ShiftGroupOffset => 2,
            Self::RetargetGroupKind => 2,
            Self::RemoveGroup => 2,
            // T25.F07: after `RemoveGroup`, at the tier of a moderate edit.
            Self::SetScope => 2,
            // T25.F08: after `SetScope`, each at its uniform twin's tier and
            // detach at a moderate edit's.
            Self::AddRandomizedRingGroup => 2,
            Self::AddRandomizedDirectionPairGroup => 2,
            Self::DetachGraphEdge => 2,
            // T25.F09: after `DetachGraphEdge`, the enable and disable twins
            // at a structural tier and the steps at a refinement's, as
            // their trace-decay analogs.
            Self::EnableRetention | Self::DisableRetention => 1,
            Self::MutateRetention => 4,
            Self::EnableGroupRetention | Self::DisableGroupRetention => 1,
            Self::MutateGroupRetention => 4,
        }
    }

    const TOTAL_WEIGHT: u16 = {
        assert!(
            Self::ALL.len() == 38,
            "ALL must cover every GraphOperator variant"
        );
        let mut sum = 0u16;
        let mut i = 0;
        while i < Self::ALL.len() {
            if !matches!(Self::ALL[i], Self::RecruitNeutralInput) {
                sum += Self::ALL[i].weight() as u16;
            }
            i += 1;
        }
        sum
    };

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
            Self::CopyEdgeBundle => MutationOperator::GraphCopyEdgeBundle,
            Self::EnableHebbian => MutationOperator::GraphEnableHebbian,
            Self::DisableHebbian => MutationOperator::GraphDisableHebbian,
            Self::MutateHebbianRule => MutationOperator::GraphMutateHebbianRule,
            Self::MutateHebbianRate => MutationOperator::GraphMutateHebbianRate,
            Self::ToggleHebbianLamarckian => MutationOperator::GraphToggleHebbianLamarckian,
            Self::EnableRewardModulation => MutationOperator::GraphEnableRewardModulation,
            Self::DisableRewardModulation => MutationOperator::GraphDisableRewardModulation,
            Self::MutateRewardSource => MutationOperator::GraphMutateRewardSource,
            Self::MutateTraceDecay => MutationOperator::GraphMutateTraceDecay,
            Self::RecruitNeutralInput => MutationOperator::GraphRecruitNeutralInput,
            Self::AddRingGroup => MutationOperator::GraphAddRingGroup,
            Self::AddDirectionPairGroup => MutationOperator::GraphAddDirectionPairGroup,
            Self::AlterGroupScale => MutationOperator::GraphAlterGroupScale,
            Self::ShiftGroupOffset => MutationOperator::GraphShiftGroupOffset,
            Self::RetargetGroupKind => MutationOperator::GraphRetargetGroupKind,
            Self::RemoveGroup => MutationOperator::GraphRemoveGroup,
            Self::SetScope => MutationOperator::GraphSetScope,
            Self::AddRandomizedRingGroup => MutationOperator::GraphAddRandomizedRingGroup,
            Self::AddRandomizedDirectionPairGroup => {
                MutationOperator::GraphAddRandomizedDirectionPairGroup
            }
            Self::DetachGraphEdge => MutationOperator::GraphDetachGraphEdge,
            Self::EnableRetention => MutationOperator::GraphEnableRetention,
            Self::DisableRetention => MutationOperator::GraphDisableRetention,
            Self::MutateRetention => MutationOperator::GraphMutateRetention,
            Self::EnableGroupRetention => MutationOperator::GraphEnableGroupRetention,
            Self::DisableGroupRetention => MutationOperator::GraphDisableGroupRetention,
            Self::MutateGroupRetention => MutationOperator::GraphMutateGroupRetention,
        }
    }

    /// Whether this operator increases, decreases, or preserves genome
    /// complexity, as its [`MutationOperator`] classifies it.
    #[must_use]
    pub const fn complexity_effect(self) -> crate::mutation::types::ComplexityEffect {
        self.operator().complexity_effect()
    }

    /// Whether configuration admits this operator before any sampling.
    pub fn enabled(self, config: &MutationConfig) -> bool {
        match self {
            Self::RecruitNeutralInput => {
                config.neutral_input_recruitment != NeutralInputRecruitment::Off
            }
            _ => true,
        }
    }

    /// Pick a default-arm graph operator weighted by impact tier.
    // WORKAROUND(T25.F08): a test-only sampler beside the engine's
    // `select_weighted_index` over the enabled catalog; the four domains'
    // `random` and `TOTAL_WEIGHT` go in one area change and the catalog pins
    // assert `ALL.len()` and the engine's enabled weight sum.
    pub fn random(rng: &mut impl Rng) -> Self {
        let mut r = rng.gen_range(0..Self::TOTAL_WEIGHT);
        for &op in &Self::ALL {
            if op == Self::RecruitNeutralInput {
                continue;
            }
            let w = op.weight() as u16;
            if r < w {
                return op;
            }
            r -= w;
        }
        unreachable!()
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
            Self::RecruitNeutralInput => !recruitment::destinations(def).is_empty(),
            Self::GraphRawFieldMutation => operators::has_raw_field_site(def, inputs),
            Self::CopyInternalNode => operators::can_copy_compute_node(def),
            Self::CopySubgraph => operators::can_copy_subgraph(def),
            Self::CopyEdgeBundle => operators::can_copy_edge_bundle(def),
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
            Self::AddRingGroup | Self::AddRandomizedRingGroup => group::can_add(def, inputs.refs),
            Self::AddDirectionPairGroup | Self::AddRandomizedDirectionPairGroup => {
                group::can_add_pair(def, inputs.refs)
            }
            Self::AlterGroupScale
            | Self::ShiftGroupOffset
            | Self::RetargetGroupKind
            | Self::RemoveGroup => group::has_group(def),
            Self::SetScope | Self::DetachGraphEdge => hebbian::any_edge(def, hebbian::is_member),
            Self::EnableRetention => hebbian::any_edge(def, hebbian::can_enable_retention),
            Self::DisableRetention | Self::MutateRetention => {
                hebbian::any_edge(def, hebbian::is_decaying)
            }
            Self::EnableGroupRetention => group::any_group(def, group::retains),
            Self::DisableGroupRetention | Self::MutateGroupRetention => {
                group::any_group(def, group::is_decaying_group)
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

    /// Apply a graph operator to the genome.
    ///
    /// Returns `Ok(TargetReachability)` on success, or `Err(MutationSkipReason::NoApplicableTarget)`
    /// if no Graph-backend node has a site this operator can apply to. The
    /// skip happens before the draw, so the engine records it as
    /// no-eligible-node with no selected target.
    pub fn apply(
        genome: &mut CreatureGenome,
        op: GraphOperator,
        targets: &mut TargetSelector<'_>,
        rng: &mut impl Rng,
        config: &MutationConfig,
    ) -> Result<TargetReachability, MutationSkipReason> {
        Self::apply_in(
            genome,
            op,
            targets,
            rng,
            config,
            &crate::config::SimulationConfig::default().sensor_context(1),
        )
    }

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
        if !op.enabled(config) {
            return Err(MutationSkipReason::NoApplicableTarget);
        }
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
        config: &MutationConfig,
        context: &SensorContext,
    ) -> Result<GraphEventReport, MutationSkipReason> {
        match op {
            GraphOperator::RecruitNeutralInput => unreported(recruitment::recruit(
                node_mut(genome, node_idx)?,
                config.neutral_input_recruitment,
                context,
                rng,
            )),
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
            GraphOperator::CopyEdgeBundle => {
                unreported(operators::apply_copy_edge_bundle(genome, node_idx, rng))
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
            GraphOperator::AlterGroupScale => unreported(group::alter_scale(
                operators::graph_def_mut(genome, node_idx)?,
                rng,
            )),
            GraphOperator::ShiftGroupOffset => unreported(group::shift_offset(
                operators::graph_def_mut(genome, node_idx)?,
                rng,
            )),
            GraphOperator::RetargetGroupKind => unreported(group::retarget_kind(
                operators::graph_def_mut(genome, node_idx)?,
                rng,
            )),
            GraphOperator::RemoveGroup => edited(group::remove(
                operators::graph_def_mut(genome, node_idx)?,
                rng,
            )),
            GraphOperator::SetScope => scoped(hebbian::set_scope_in_def(
                operators::graph_def_mut(genome, node_idx)?,
                rng,
            )),
            GraphOperator::AddRingGroup => edited(group::add_in_form(
                node_mut(genome, node_idx)?,
                group::CreationForm::Uniform,
                context,
                rng,
            )),
            GraphOperator::AddRandomizedRingGroup => edited(group::add_in_form(
                node_mut(genome, node_idx)?,
                group::CreationForm::Randomized,
                context,
                rng,
            )),
            GraphOperator::AddDirectionPairGroup => edited(group::add_pair_in_form(
                node_mut(genome, node_idx)?,
                group::CreationForm::Uniform,
                context,
                rng,
            )),
            GraphOperator::AddRandomizedDirectionPairGroup => edited(group::add_pair_in_form(
                node_mut(genome, node_idx)?,
                group::CreationForm::Randomized,
                context,
                rng,
            )),
            GraphOperator::DetachGraphEdge => {
                edited(group::detach_member(node_mut(genome, node_idx)?, rng))
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
            GraphOperator::EnableGroupRetention => unreported(group::enable_retention(
                operators::graph_def_mut(genome, node_idx)?,
                rng,
            )),
            GraphOperator::DisableGroupRetention => unreported(group::disable_retention(
                operators::graph_def_mut(genome, node_idx)?,
                rng,
            )),
            GraphOperator::MutateGroupRetention => unreported(group::mutate_retention(
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
