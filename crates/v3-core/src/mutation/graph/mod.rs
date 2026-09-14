pub mod hebbian;
pub(crate) mod operators;

use rand::Rng;

use crate::config::MutationConfig;
use crate::contracts::InputReference;
use crate::creature::genome::cgp::CgpGraphBackendDef;
use crate::creature::genome::{BackendDef, CreatureGenome};
use crate::mutation::reachability::TargetSelector;
use crate::mutation::types::{MutationSkipReason, TargetReachability};

/// Graph mutation operator variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GraphOperator {
    AlterGraphEdgeWeight,
    SwapGraphOperator,
    MutateGraphOperatorParam,
    MutateActionSlotBehavior,
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
}

impl GraphOperator {
    pub const ALL: [Self; 22] = [
        Self::AlterGraphEdgeWeight,
        Self::SwapGraphOperator,
        Self::MutateGraphOperatorParam,
        Self::MutateActionSlotBehavior,
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
    ];

    /// Per-operator weight reflecting impact tier.
    /// 4 = refinement, 2 = moderate, 1 = structural.
    #[must_use]
    pub const fn weight(self) -> u8 {
        match self {
            Self::AlterGraphEdgeWeight => 4,
            Self::SwapGraphOperator => 2,
            Self::MutateGraphOperatorParam => 4,
            Self::MutateActionSlotBehavior => 2,
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
        }
    }

    const TOTAL_WEIGHT: u16 = {
        assert!(
            Self::ALL.len() == 22,
            "ALL must cover every GraphOperator variant"
        );
        let mut sum = 0u16;
        let mut i = 0;
        while i < Self::ALL.len() {
            sum += Self::ALL[i].weight() as u16;
            i += 1;
        }
        sum
    };

    /// Whether this operator increases, decreases, or preserves genome complexity.
    #[must_use]
    pub const fn complexity_effect(self) -> crate::mutation::types::ComplexityEffect {
        use crate::mutation::types::ComplexityEffect;
        match self {
            Self::AddInternalGraphNode
            | Self::AddGraphEdge
            | Self::CopyInternalNode
            | Self::CopySubgraph
            | Self::CopyEdgeBundle
            | Self::EnableHebbian
            | Self::EnableRewardModulation => ComplexityEffect::Increasing,
            Self::RemoveInternalGraphNode
            | Self::RemoveGraphEdge
            | Self::DisableHebbian
            | Self::DisableRewardModulation => ComplexityEffect::Decreasing,
            Self::AlterGraphEdgeWeight
            | Self::SwapGraphOperator
            | Self::MutateGraphOperatorParam
            | Self::MutateActionSlotBehavior
            | Self::RetargetGraphEdge
            | Self::GraphRawFieldMutation
            | Self::MutateHebbianRule
            | Self::MutateHebbianRate
            | Self::ToggleHebbianLamarckian
            | Self::MutateRewardSource
            | Self::MutateTraceDecay => ComplexityEffect::Neutral,
        }
    }

    /// Pick a random graph operator weighted by impact tier.
    pub fn random(rng: &mut impl Rng) -> Self {
        let mut r = rng.gen_range(0..Self::TOTAL_WEIGHT);
        for &op in &Self::ALL {
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
    /// its target from, so a node this accepts never skips at application.
    fn applies_to(
        self,
        def: &CgpGraphBackendDef,
        input_refs: &[InputReference],
        config: &MutationConfig,
    ) -> bool {
        match self {
            Self::AlterGraphEdgeWeight | Self::RetargetGraphEdge | Self::RemoveGraphEdge => {
                operators::has_edge_site(def)
            }
            Self::SwapGraphOperator | Self::RemoveInternalGraphNode => {
                operators::has_compute_node(def)
            }
            Self::MutateGraphOperatorParam => operators::has_parameterized_compute_node(def),
            Self::MutateActionSlotBehavior => operators::has_action_slot(def),
            Self::AddInternalGraphNode => operators::can_add_compute_node(def),
            Self::AddGraphEdge => operators::can_add_edge(def),
            Self::GraphRawFieldMutation => operators::has_raw_field_site(def, input_refs, config),
            Self::CopyInternalNode => operators::can_copy_compute_node(def),
            Self::CopySubgraph => operators::can_copy_subgraph(def),
            Self::CopyEdgeBundle => operators::can_copy_edge_bundle(def),
            Self::EnableHebbian => hebbian::any_node(def, hebbian::can_enable_hebbian),
            Self::DisableHebbian
            | Self::MutateHebbianRule
            | Self::MutateHebbianRate
            | Self::ToggleHebbianLamarckian => hebbian::any_node(def, hebbian::is_plastic),
            Self::EnableRewardModulation => {
                hebbian::any_node(def, hebbian::can_enable_reward_modulation)
            }
            Self::DisableRewardModulation | Self::MutateRewardSource | Self::MutateTraceDecay => {
                hebbian::any_node(def, hebbian::is_reward_modulated)
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
        config: &MutationConfig,
    ) -> Vec<usize> {
        genome
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| match &n.backend_def {
                BackendDef::Graph(def) => op.applies_to(def, &n.input_refs, config),
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
        let applicable = Self::applicable_indices(genome, op, config);
        if applicable.is_empty() {
            return Err(MutationSkipReason::NoApplicableTarget);
        }

        let (node_idx, reachability) = targets
            .select(&applicable, rng)
            .ok_or(MutationSkipReason::NoApplicableTarget)?;
        Self::apply_to_node(genome, op, node_idx, rng, config).map(|()| reachability)
    }

    /// Dispatch `op` onto one already-selected node.
    pub(crate) fn apply_to_node(
        genome: &mut CreatureGenome,
        op: GraphOperator,
        node_idx: usize,
        rng: &mut impl Rng,
        config: &MutationConfig,
    ) -> Result<(), MutationSkipReason> {
        match op {
            GraphOperator::AlterGraphEdgeWeight => {
                operators::alter_edge_weight(genome, node_idx, rng)
            }
            GraphOperator::SwapGraphOperator => operators::swap_operator(genome, node_idx, rng),
            GraphOperator::MutateGraphOperatorParam => {
                operators::mutate_operator_param(genome, node_idx, rng)
            }
            GraphOperator::MutateActionSlotBehavior => {
                operators::mutate_action_slot_behavior(genome, node_idx, rng)
            }
            GraphOperator::AddInternalGraphNode => {
                operators::add_internal_node(genome, node_idx, rng, config)
            }
            GraphOperator::RemoveInternalGraphNode => {
                operators::remove_internal_node(genome, node_idx, rng)
            }
            GraphOperator::AddGraphEdge => operators::add_graph_edge(genome, node_idx, rng, config),
            GraphOperator::RetargetGraphEdge => {
                operators::retarget_graph_edge(genome, node_idx, rng, config)
            }
            GraphOperator::RemoveGraphEdge => operators::remove_graph_edge(genome, node_idx, rng),
            GraphOperator::GraphRawFieldMutation => {
                operators::apply_graph_raw_field_mutation(genome, node_idx, rng, config)
            }
            GraphOperator::CopyInternalNode => {
                operators::apply_copy_internal_node(genome, node_idx, rng)
            }
            GraphOperator::CopySubgraph => operators::apply_copy_subgraph(genome, node_idx, rng),
            GraphOperator::CopyEdgeBundle => {
                operators::apply_copy_edge_bundle(genome, node_idx, rng)
            }
            GraphOperator::EnableHebbian => hebbian::enable_hebbian(genome, node_idx, rng),
            GraphOperator::DisableHebbian => hebbian::disable_hebbian(genome, node_idx, rng),
            GraphOperator::MutateHebbianRule => hebbian::mutate_hebbian_rule(genome, node_idx, rng),
            GraphOperator::MutateHebbianRate => hebbian::mutate_hebbian_rate(genome, node_idx, rng),
            GraphOperator::ToggleHebbianLamarckian => {
                hebbian::toggle_hebbian_lamarckian(genome, node_idx, rng)
            }
            GraphOperator::EnableRewardModulation => {
                hebbian::enable_reward_modulation(genome, node_idx, rng)
            }
            GraphOperator::DisableRewardModulation => {
                hebbian::disable_reward_modulation(genome, node_idx, rng)
            }
            GraphOperator::MutateRewardSource => {
                hebbian::mutate_reward_source(genome, node_idx, rng)
            }
            GraphOperator::MutateTraceDecay => hebbian::mutate_trace_decay(genome, node_idx, rng),
        }
    }
}

#[cfg(test)]
mod tests;
