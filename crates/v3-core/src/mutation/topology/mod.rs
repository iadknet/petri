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
    CopyNode,
    CopyMeshBackwardSlice,
    CopyMeshForwardSlice,
    SpliceNode,
    SwapRouteTargets,
    MutateGateBias,
}

impl TopologyOperator {
    pub const ALL: [Self; 13] = [
        Self::AddNode,
        Self::RemoveNode,
        Self::RetargetNodeTarget,
        Self::AddRouteTarget,
        Self::RemoveRouteTarget,
        Self::ChangeEntryNode,
        Self::SwapNodeBackend,
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
            Self::ChangeEntryNode => 1,
            Self::SwapNodeBackend => 1,
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
            Self::ALL.len() == 13,
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
            | Self::SwapNodeBackend
            | Self::AddRouteTarget => ComplexityEffect::Increasing,
            Self::RemoveNode | Self::RemoveRouteTarget => ComplexityEffect::Decreasing,
            Self::RetargetNodeTarget
            | Self::ChangeEntryNode
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
    /// applicable target exists. The exempt operator ChangeEntryNode returns
    /// `NotApplicable` since they don't select a target node.
    #[cfg(test)]
    pub fn apply(
        genome: &mut CreatureGenome,
        op: TopologyOperator,
        reachable_nodes: &[usize],
        bias: f64,
        rng: &mut impl Rng,
        config: &MutationConfig,
    ) -> Result<TargetReachability, MutationSkipReason> {
        Self::apply_with_food_type_count(genome, op, reachable_nodes, bias, rng, config, 1)
    }

    /// Apply a topology operator with typed-food mutation context.
    pub fn apply_with_food_type_count(
        genome: &mut CreatureGenome,
        op: TopologyOperator,
        reachable_nodes: &[usize],
        bias: f64,
        rng: &mut impl Rng,
        config: &MutationConfig,
        _food_type_count: usize,
    ) -> Result<TargetReachability, MutationSkipReason> {
        match op {
            TopologyOperator::AddNode => {
                structural::apply_splice_node(genome, reachable_nodes, bias, rng)
            }
            TopologyOperator::ChangeEntryNode => structural::apply_change_entry_node(genome, rng)
                .map(|()| TargetReachability::NotApplicable),
            // Biased structural operators:
            TopologyOperator::RemoveNode => {
                structural::apply_remove_node(genome, reachable_nodes, bias, rng)
            }
            TopologyOperator::SwapNodeBackend => {
                structural::apply_swap_node_backend(genome, reachable_nodes, bias, rng, config)
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
                structural::apply_splice_node(genome, reachable_nodes, bias, rng)
            }
            // Biased routing operators:
            TopologyOperator::RetargetNodeTarget => {
                routing::apply_retarget_node_target(genome, reachable_nodes, bias, rng)
            }
            TopologyOperator::AddRouteTarget => {
                routing::apply_add_route_target(genome, reachable_nodes, bias, rng, config)
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

#[cfg(test)]
mod f15_tests;

#[cfg(test)]
mod f08_tests;
