use rand::Rng;

use crate::config::MutationConfig;
use crate::creature::genome::CreatureGenome;
use crate::mutation::types::{MutationSkipReason, TargetReachability};

// Re-exported for use via `use super::*` in tests.
#[cfg(test)]
use crate::contracts::NodeId;
#[cfg(test)]
use crate::creature::genome::{BackendDef, NodeGenome};

mod birth;
mod routing;
mod structural;

/// Topology mutation operator variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TopologyOperator {
    AddNode,
    RemoveNode,
    RetargetNodeTarget,
    AddRouteTarget,
    RemoveRouteTarget,
    ChangeEntryNode,
    SwapNodeBackend,
    RewriteNodeId,
    CopyNode,
    CopyMeshBackwardSlice,
    CopyMeshForwardSlice,
    SpliceNode,
    SwapRouteTargets,
    MutateGateBias,
}

impl TopologyOperator {
    pub const ALL: [Self; 14] = [
        Self::AddNode,
        Self::RemoveNode,
        Self::RetargetNodeTarget,
        Self::AddRouteTarget,
        Self::RemoveRouteTarget,
        Self::ChangeEntryNode,
        Self::SwapNodeBackend,
        Self::RewriteNodeId,
        Self::CopyNode,
        Self::CopyMeshBackwardSlice,
        Self::CopyMeshForwardSlice,
        Self::SpliceNode,
        Self::SwapRouteTargets,
        Self::MutateGateBias,
    ];

    /// Per-operator weight reflecting impact tier.
    /// 4 = refinement, 2 = moderate, 1 = structural.
    #[must_use]
    pub const fn weight(self) -> u8 {
        match self {
            Self::AddNode => 1,
            Self::RemoveNode => 1,
            Self::RetargetNodeTarget => 2,
            Self::AddRouteTarget => 2,
            Self::RemoveRouteTarget => 2,
            Self::ChangeEntryNode => 2,
            Self::SwapNodeBackend => 1,
            Self::RewriteNodeId => 4,
            Self::CopyNode => 1,
            Self::CopyMeshBackwardSlice => 1,
            Self::CopyMeshForwardSlice => 1,
            Self::SpliceNode => 1,
            Self::SwapRouteTargets => 4,
            Self::MutateGateBias => 4,
        }
    }

    const TOTAL_WEIGHT: u16 = {
        // Compile-time guard: if a variant is added to the enum but not to ALL,
        // weight() will still compile (exhaustive match), but ALL will be incomplete.
        // This assertion catches that at compile time.
        assert!(
            Self::ALL.len() == 14,
            "ALL must cover every TopologyOperator variant"
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
            Self::AddNode
            | Self::CopyNode
            | Self::CopyMeshBackwardSlice
            | Self::CopyMeshForwardSlice
            | Self::SpliceNode
            | Self::AddRouteTarget => ComplexityEffect::Increasing,
            Self::RemoveNode | Self::RemoveRouteTarget => ComplexityEffect::Decreasing,
            Self::RetargetNodeTarget
            | Self::ChangeEntryNode
            | Self::SwapNodeBackend
            | Self::RewriteNodeId
            | Self::SwapRouteTargets
            | Self::MutateGateBias => ComplexityEffect::Neutral,
        }
    }

    /// Pick a random topology operator weighted by impact tier.
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
}

/// Topology domain mutator.
pub struct TopologyMutator;

impl TopologyMutator {
    /// Apply a topology operator to the genome with reachability-biased target selection.
    ///
    /// Returns `Ok(TargetReachability)` on success, or `Err(MutationSkipReason)` if no
    /// applicable target exists. Exempt operators (AddNode, ChangeEntryNode) return
    /// `NotApplicable` since they don't select a target node.
    pub fn apply(
        genome: &mut CreatureGenome,
        op: TopologyOperator,
        reachable_nodes: &[usize],
        bias: f64,
        rng: &mut impl Rng,
        config: &MutationConfig,
    ) -> Result<TargetReachability, MutationSkipReason> {
        match op {
            // Exempt: these don't select a target node for mutation.
            TopologyOperator::AddNode => structural::apply_add_node(genome, config, rng)
                .map(|()| TargetReachability::NotApplicable),
            TopologyOperator::ChangeEntryNode => structural::apply_change_entry_node(genome, rng)
                .map(|()| TargetReachability::NotApplicable),
            // Biased structural operators:
            TopologyOperator::RemoveNode => {
                structural::apply_remove_node(genome, reachable_nodes, bias, rng)
            }
            TopologyOperator::SwapNodeBackend => {
                structural::apply_swap_node_backend(genome, reachable_nodes, bias, rng, config)
            }
            TopologyOperator::RewriteNodeId => {
                structural::apply_rewrite_node_id(genome, reachable_nodes, bias, rng)
            }
            TopologyOperator::CopyNode => {
                structural::apply_copy_node(genome, reachable_nodes, bias, rng)
            }
            TopologyOperator::CopyMeshBackwardSlice => {
                structural::apply_copy_mesh_backward_slice(genome, reachable_nodes, bias, rng)
            }
            TopologyOperator::CopyMeshForwardSlice => {
                structural::apply_copy_mesh_forward_slice(genome, reachable_nodes, bias, rng)
            }
            TopologyOperator::SpliceNode => {
                structural::apply_splice_node(genome, reachable_nodes, bias, rng, config)
            }
            // Biased routing operators:
            TopologyOperator::RetargetNodeTarget => {
                routing::apply_retarget_node_target(genome, reachable_nodes, bias, rng)
            }
            TopologyOperator::AddRouteTarget => {
                routing::apply_add_route_target(genome, reachable_nodes, bias, rng)
            }
            TopologyOperator::RemoveRouteTarget => {
                routing::apply_remove_route_target(genome, reachable_nodes, bias, rng)
            }
            TopologyOperator::SwapRouteTargets => {
                routing::apply_swap_route_targets(genome, reachable_nodes, bias, rng)
            }
            TopologyOperator::MutateGateBias => {
                routing::apply_mutate_gate_bias(genome, reachable_nodes, bias, rng)
            }
        }
    }
}

#[cfg(test)]
mod tests;
