use rand::Rng;

use crate::config::MutationConfig;
use crate::creature::genome::CreatureGenome;
use crate::mutation::reachability::TargetSelector;
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
    /// 40 = refinement, 20 = moderate, 10 = structural; `ChangeEntryNode`
    /// stays at 1 so the whole-brain macro is a rare event (1 in 211) rather
    /// than 1 in 22: on a chain-shaped founder an entry moved past the
    /// action nodes queues nothing, the one topology edit that still reads
    /// dead after T11.F15.
    #[must_use]
    pub const fn weight(self) -> u8 {
        match self {
            Self::AddNode => 10,
            Self::RemoveNode => 10,
            Self::RetargetNodeTarget => 20,
            Self::AddRouteTarget => 20,
            Self::RemoveRouteTarget => 20,
            Self::ChangeEntryNode => 1,
            Self::SwapNodeBackend => 10,
            Self::CopyNode => 10,
            Self::CopyMeshBackwardSlice => 10,
            Self::CopyMeshForwardSlice => 10,
            Self::SpliceNode => 10,
            Self::SwapRouteTargets => 40,
            Self::MutateGateBias => 40,
        }
    }

    /// Integer weights in hundredths keep small weights exact, including at 1%.
    /// The base total is 211, so the tuned total never exceeds 21,100.
    #[must_use]
    pub fn tuned_weight(self, large_copy_weight_percent: u8) -> u16 {
        let factor = match self {
            Self::CopyNode | Self::CopyMeshBackwardSlice | Self::CopyMeshForwardSlice => {
                u16::from(large_copy_weight_percent.min(100))
            }
            _ => 100,
        };
        u16::from(self.weight()) * factor
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
        targets: &mut TargetSelector<'_>,
        rng: &mut impl Rng,
        config: &MutationConfig,
    ) -> Result<TargetReachability, MutationSkipReason> {
        Self::apply_with_food_type_count(genome, op, targets, rng, config, 1)
    }

    /// Apply a topology operator with typed-food mutation context.
    pub fn apply_with_food_type_count(
        genome: &mut CreatureGenome,
        op: TopologyOperator,
        targets: &mut TargetSelector<'_>,
        rng: &mut impl Rng,
        _config: &MutationConfig,
        _food_type_count: usize,
    ) -> Result<TargetReachability, MutationSkipReason> {
        match op {
            TopologyOperator::AddNode => structural::apply_splice_node(genome, targets, rng),
            TopologyOperator::ChangeEntryNode => structural::apply_change_entry_node(genome, rng)
                .map(|()| TargetReachability::NotApplicable),
            // Biased structural operators:
            TopologyOperator::RemoveNode => structural::apply_remove_node(genome, targets, rng),
            TopologyOperator::SwapNodeBackend => {
                structural::apply_swap_node_backend(genome, targets, rng)
            }
            TopologyOperator::CopyNode => structural::apply_copy_node(genome, targets, rng),
            TopologyOperator::CopyMeshBackwardSlice => {
                structural::apply_copy_mesh_backward_slice(genome, targets, rng)
            }
            TopologyOperator::CopyMeshForwardSlice => {
                structural::apply_copy_mesh_forward_slice(genome, targets, rng)
            }
            TopologyOperator::SpliceNode => structural::apply_splice_node(genome, targets, rng),
            // Biased routing operators:
            TopologyOperator::RetargetNodeTarget => {
                routing::apply_retarget_node_target(genome, targets, rng)
            }
            TopologyOperator::AddRouteTarget => {
                routing::apply_add_route_target(genome, targets, rng)
            }
            TopologyOperator::RemoveRouteTarget => {
                routing::apply_remove_route_target(genome, targets, rng)
            }
            TopologyOperator::SwapRouteTargets => {
                routing::apply_swap_route_targets(genome, targets, rng)
            }
            TopologyOperator::MutateGateBias => {
                routing::apply_mutate_gate_bias(genome, targets, rng)
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
