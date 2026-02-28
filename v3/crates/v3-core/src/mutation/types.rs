use std::collections::HashMap;

/// Whether a mutation operator increases, decreases, or preserves genome complexity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComplexityEffect {
    Increasing,
    Decreasing,
    Neutral,
}

impl ComplexityEffect {
    /// Returns true if this effect is `Increasing`.
    #[must_use]
    pub const fn is_increasing(self) -> bool {
        matches!(self, Self::Increasing)
    }
}

/// The two architectural layers of genome mutation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum MutationLayer {
    /// Mesh-level: add/remove/rewire nodes in the topology graph.
    Mesh,
    /// Node-internal: modify VM programs, graph weights, input references within a node.
    NodeInternal,
}

/// Reason a mutation event was skipped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum MutationSkipReason {
    ParseabilityViolation,
    NoApplicableTarget,
    BudgetExhausted,
}

impl MutationSkipReason {
    /// Stable string key used by transport/API boundaries.
    #[must_use]
    pub const fn as_key(self) -> &'static str {
        match self {
            Self::ParseabilityViolation => "ParseabilityViolation",
            Self::NoApplicableTarget => "NoApplicableTarget",
            Self::BudgetExhausted => "BudgetExhausted",
        }
    }
}

/// Domain selected for a mutation event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum MutationDomain {
    Topology,
    Vm,
    Graph,
    InputRef,
}

impl MutationDomain {
    #[must_use]
    pub const fn as_key(self) -> &'static str {
        match self {
            Self::Topology => "Topology",
            Self::Vm => "Vm",
            Self::Graph => "Graph",
            Self::InputRef => "InputRef",
        }
    }

    #[must_use]
    pub const fn all() -> [Self; 4] {
        [Self::Topology, Self::Vm, Self::Graph, Self::InputRef]
    }

    #[must_use]
    pub const fn layer(self) -> MutationLayer {
        match self {
            Self::Topology => MutationLayer::Mesh,
            Self::Vm | Self::Graph | Self::InputRef => MutationLayer::NodeInternal,
        }
    }
}

/// Mutation operator selected for one attempted mutation event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum MutationOperator {
    // Topology
    TopologyAddNode,
    TopologyRemoveNode,
    TopologyRetargetNodeTarget,
    TopologyAddRouteTarget,
    TopologyRemoveRouteTarget,
    TopologyChangeEntryNode,
    TopologySwapNodeBackend,
    TopologyRewriteNodeId,
    TopologyCopyNode,
    TopologyCopyMeshBackwardSlice,
    TopologyCopyMeshForwardSlice,
    TopologySpliceNode,
    TopologySwapRouteTargets,
    // VM
    VmConstantMutation,
    VmInstructionMutation,
    VmRegisterCountMutation,
    VmInstructionRawFieldMutation,
    VmCopyInstructionBlock,
    VmCopyInstructionBlockRemapped,
    VmCopyConstantBlock,
    VmCopyGeneBackwardSlice,
    VmCopyGeneForwardSlice,
    // Graph
    GraphAlterGraphEdgeWeight,
    GraphSwapGraphOperator,
    GraphMutateGraphOperatorParam,
    GraphAddInternalGraphNode,
    GraphRemoveInternalGraphNode,
    GraphAddGraphEdge,
    GraphRetargetGraphEdge,
    GraphRemoveGraphEdge,
    GraphRawFieldMutation,
    GraphCopyInternalNode,
    GraphCopySubgraph,
    GraphCopyEdgeBundle,
    GraphEnableHebbian,
    GraphDisableHebbian,
    GraphMutateHebbianRule,
    GraphMutateHebbianRate,
    GraphToggleHebbianLamarckian,
    // InputRef
    InputRefAdd,
    InputRefRemove,
    InputRefSwap,
    InputRefRawFieldMutation,
}

impl MutationOperator {
    #[must_use]
    pub const fn as_key(self) -> &'static str {
        match self {
            Self::TopologyAddNode => "Topology.AddNode",
            Self::TopologyRemoveNode => "Topology.RemoveNode",
            Self::TopologyRetargetNodeTarget => "Topology.RetargetNodeTarget",
            Self::TopologyAddRouteTarget => "Topology.AddRouteTarget",
            Self::TopologyRemoveRouteTarget => "Topology.RemoveRouteTarget",
            Self::TopologyChangeEntryNode => "Topology.ChangeEntryNode",
            Self::TopologySwapNodeBackend => "Topology.SwapNodeBackend",
            Self::TopologyRewriteNodeId => "Topology.RewriteNodeId",
            Self::TopologyCopyNode => "Topology.CopyNode",
            Self::TopologyCopyMeshBackwardSlice => "Topology.CopyMeshBackwardSlice",
            Self::TopologyCopyMeshForwardSlice => "Topology.CopyMeshForwardSlice",
            Self::TopologySpliceNode => "Topology.SpliceNode",
            Self::TopologySwapRouteTargets => "Topology.SwapRouteTargets",
            Self::VmConstantMutation => "Vm.VmConstantMutation",
            Self::VmInstructionMutation => "Vm.VmInstructionMutation",
            Self::VmRegisterCountMutation => "Vm.VmRegisterCountMutation",
            Self::VmInstructionRawFieldMutation => "Vm.VmInstructionRawFieldMutation",
            Self::VmCopyInstructionBlock => "Vm.CopyInstructionBlock",
            Self::VmCopyInstructionBlockRemapped => "Vm.CopyInstructionBlockRemapped",
            Self::VmCopyConstantBlock => "Vm.CopyConstantBlock",
            Self::VmCopyGeneBackwardSlice => "Vm.CopyGeneBackwardSlice",
            Self::VmCopyGeneForwardSlice => "Vm.CopyGeneForwardSlice",
            Self::GraphAlterGraphEdgeWeight => "Graph.AlterGraphEdgeWeight",
            Self::GraphSwapGraphOperator => "Graph.SwapGraphOperator",
            Self::GraphMutateGraphOperatorParam => "Graph.MutateGraphOperatorParam",
            Self::GraphAddInternalGraphNode => "Graph.AddInternalGraphNode",
            Self::GraphRemoveInternalGraphNode => "Graph.RemoveInternalGraphNode",
            Self::GraphAddGraphEdge => "Graph.AddGraphEdge",
            Self::GraphRetargetGraphEdge => "Graph.RetargetGraphEdge",
            Self::GraphRemoveGraphEdge => "Graph.RemoveGraphEdge",
            Self::GraphRawFieldMutation => "Graph.GraphRawFieldMutation",
            Self::GraphCopyInternalNode => "Graph.CopyInternalNode",
            Self::GraphCopySubgraph => "Graph.CopySubgraph",
            Self::GraphCopyEdgeBundle => "Graph.CopyEdgeBundle",
            Self::GraphEnableHebbian => "Graph.EnableHebbian",
            Self::GraphDisableHebbian => "Graph.DisableHebbian",
            Self::GraphMutateHebbianRule => "Graph.MutateHebbianRule",
            Self::GraphMutateHebbianRate => "Graph.MutateHebbianRate",
            Self::GraphToggleHebbianLamarckian => "Graph.ToggleHebbianLamarckian",
            Self::InputRefAdd => "InputRef.Add",
            Self::InputRefRemove => "InputRef.Remove",
            Self::InputRefSwap => "InputRef.Swap",
            Self::InputRefRawFieldMutation => "InputRef.RawFieldMutation",
        }
    }

    #[must_use]
    pub const fn domain(self) -> MutationDomain {
        match self {
            Self::TopologyAddNode
            | Self::TopologyRemoveNode
            | Self::TopologyRetargetNodeTarget
            | Self::TopologyAddRouteTarget
            | Self::TopologyRemoveRouteTarget
            | Self::TopologyChangeEntryNode
            | Self::TopologySwapNodeBackend
            | Self::TopologyRewriteNodeId
            | Self::TopologyCopyNode
            | Self::TopologyCopyMeshBackwardSlice
            | Self::TopologyCopyMeshForwardSlice
            | Self::TopologySpliceNode
            | Self::TopologySwapRouteTargets => MutationDomain::Topology,
            Self::VmConstantMutation
            | Self::VmInstructionMutation
            | Self::VmRegisterCountMutation
            | Self::VmInstructionRawFieldMutation
            | Self::VmCopyInstructionBlock
            | Self::VmCopyInstructionBlockRemapped
            | Self::VmCopyConstantBlock
            | Self::VmCopyGeneBackwardSlice
            | Self::VmCopyGeneForwardSlice => MutationDomain::Vm,
            Self::GraphAlterGraphEdgeWeight
            | Self::GraphSwapGraphOperator
            | Self::GraphMutateGraphOperatorParam
            | Self::GraphAddInternalGraphNode
            | Self::GraphRemoveInternalGraphNode
            | Self::GraphAddGraphEdge
            | Self::GraphRetargetGraphEdge
            | Self::GraphRemoveGraphEdge
            | Self::GraphRawFieldMutation
            | Self::GraphCopyInternalNode
            | Self::GraphCopySubgraph
            | Self::GraphCopyEdgeBundle
            | Self::GraphEnableHebbian
            | Self::GraphDisableHebbian
            | Self::GraphMutateHebbianRule
            | Self::GraphMutateHebbianRate
            | Self::GraphToggleHebbianLamarckian => MutationDomain::Graph,
            Self::InputRefAdd
            | Self::InputRefRemove
            | Self::InputRefSwap
            | Self::InputRefRawFieldMutation => MutationDomain::InputRef,
        }
    }

    #[must_use]
    pub const fn semantic_category(self) -> MutationSemanticCategory {
        match self {
            Self::TopologyRewriteNodeId => MutationSemanticCategory::SemanticNoop,
            _ => MutationSemanticCategory::SemanticChange,
        }
    }

    #[must_use]
    pub const fn complexity_effect(self) -> ComplexityEffect {
        match self {
            // Topology: structural additions
            Self::TopologyAddNode
            | Self::TopologyCopyNode
            | Self::TopologyCopyMeshBackwardSlice
            | Self::TopologyCopyMeshForwardSlice
            | Self::TopologySpliceNode
            | Self::TopologyAddRouteTarget => ComplexityEffect::Increasing,
            // Topology: structural removals
            Self::TopologyRemoveNode | Self::TopologyRemoveRouteTarget => {
                ComplexityEffect::Decreasing
            }
            // Topology: rewiring / neutral
            Self::TopologyRetargetNodeTarget
            | Self::TopologyChangeEntryNode
            | Self::TopologySwapNodeBackend
            | Self::TopologyRewriteNodeId
            | Self::TopologySwapRouteTargets => ComplexityEffect::Neutral,
            // VM: copy operators are increasing
            Self::VmCopyInstructionBlock
            | Self::VmCopyInstructionBlockRemapped
            | Self::VmCopyConstantBlock
            | Self::VmCopyGeneBackwardSlice
            | Self::VmCopyGeneForwardSlice => ComplexityEffect::Increasing,
            // VM: all others neutral (mutate existing content, no structural growth)
            Self::VmConstantMutation
            | Self::VmInstructionMutation
            | Self::VmRegisterCountMutation
            | Self::VmInstructionRawFieldMutation => ComplexityEffect::Neutral,
            // Graph: structural additions
            Self::GraphAddInternalGraphNode
            | Self::GraphAddGraphEdge
            | Self::GraphCopyInternalNode
            | Self::GraphCopySubgraph
            | Self::GraphCopyEdgeBundle
            | Self::GraphEnableHebbian => ComplexityEffect::Increasing,
            // Graph: structural removals
            Self::GraphRemoveInternalGraphNode
            | Self::GraphRemoveGraphEdge
            | Self::GraphDisableHebbian => ComplexityEffect::Decreasing,
            // Graph: rewiring / neutral
            Self::GraphAlterGraphEdgeWeight
            | Self::GraphSwapGraphOperator
            | Self::GraphMutateGraphOperatorParam
            | Self::GraphRetargetGraphEdge
            | Self::GraphRawFieldMutation
            | Self::GraphMutateHebbianRule
            | Self::GraphMutateHebbianRate
            | Self::GraphToggleHebbianLamarckian => ComplexityEffect::Neutral,
            // InputRef: add / remove / neutral
            Self::InputRefAdd => ComplexityEffect::Increasing,
            Self::InputRefRemove => ComplexityEffect::Decreasing,
            Self::InputRefSwap | Self::InputRefRawFieldMutation => ComplexityEffect::Neutral,
        }
    }

    #[must_use]
    pub const fn all() -> [Self; 43] {
        [
            Self::TopologyAddNode,
            Self::TopologyRemoveNode,
            Self::TopologyRetargetNodeTarget,
            Self::TopologyAddRouteTarget,
            Self::TopologyRemoveRouteTarget,
            Self::TopologyChangeEntryNode,
            Self::TopologySwapNodeBackend,
            Self::TopologyRewriteNodeId,
            Self::TopologyCopyNode,
            Self::TopologyCopyMeshBackwardSlice,
            Self::TopologyCopyMeshForwardSlice,
            Self::TopologySpliceNode,
            Self::TopologySwapRouteTargets,
            Self::VmConstantMutation,
            Self::VmInstructionMutation,
            Self::VmRegisterCountMutation,
            Self::VmInstructionRawFieldMutation,
            Self::VmCopyInstructionBlock,
            Self::VmCopyInstructionBlockRemapped,
            Self::VmCopyConstantBlock,
            Self::VmCopyGeneBackwardSlice,
            Self::VmCopyGeneForwardSlice,
            Self::GraphAlterGraphEdgeWeight,
            Self::GraphSwapGraphOperator,
            Self::GraphMutateGraphOperatorParam,
            Self::GraphAddInternalGraphNode,
            Self::GraphRemoveInternalGraphNode,
            Self::GraphAddGraphEdge,
            Self::GraphRetargetGraphEdge,
            Self::GraphRemoveGraphEdge,
            Self::GraphRawFieldMutation,
            Self::GraphCopyInternalNode,
            Self::GraphCopySubgraph,
            Self::GraphCopyEdgeBundle,
            Self::GraphEnableHebbian,
            Self::GraphDisableHebbian,
            Self::GraphMutateHebbianRule,
            Self::GraphMutateHebbianRate,
            Self::GraphToggleHebbianLamarckian,
            Self::InputRefAdd,
            Self::InputRefRemove,
            Self::InputRefSwap,
            Self::InputRefRawFieldMutation,
        ]
    }
}

/// Semantic class for an applied mutation event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum MutationSemanticCategory {
    SemanticNoop,
    SemanticChange,
}

impl MutationSemanticCategory {
    #[must_use]
    pub const fn as_key(self) -> &'static str {
        match self {
            Self::SemanticNoop => "SemanticNoop",
            Self::SemanticChange => "SemanticChange",
        }
    }
}

/// Summary returned by `MutationEngine` for every offspring.
///
/// Accounting invariant: `attempted_events == applied_events + skipped_events`.
#[derive(Debug, Clone)]
pub struct MutationSummary {
    pub attempted_events: u32,
    pub applied_events: u32,
    pub skipped_events: u32,
    pub skip_reasons: HashMap<MutationSkipReason, u32>,
    pub attempted_by_domain: HashMap<MutationDomain, u32>,
    pub applied_by_domain: HashMap<MutationDomain, u32>,
    pub attempted_by_operator: HashMap<MutationOperator, u32>,
    pub applied_by_operator: HashMap<MutationOperator, u32>,
    pub applied_semantic_noop_events: u32,
    pub applied_semantic_change_events: u32,
}

impl MutationSummary {
    /// Return a summary with all counts zero and an empty skip-reason map.
    pub fn zero() -> Self {
        Self {
            attempted_events: 0,
            applied_events: 0,
            skipped_events: 0,
            skip_reasons: HashMap::new(),
            attempted_by_domain: HashMap::new(),
            applied_by_domain: HashMap::new(),
            attempted_by_operator: HashMap::new(),
            applied_by_operator: HashMap::new(),
            applied_semantic_noop_events: 0,
            applied_semantic_change_events: 0,
        }
    }

    pub fn record_attempt(&mut self, domain: MutationDomain, operator: MutationOperator) {
        self.attempted_events += 1;
        *self.attempted_by_domain.entry(domain).or_insert(0) += 1;
        *self.attempted_by_operator.entry(operator).or_insert(0) += 1;
    }

    pub fn record_applied(
        &mut self,
        domain: MutationDomain,
        operator: MutationOperator,
        semantic: MutationSemanticCategory,
    ) {
        self.applied_events += 1;
        *self.applied_by_domain.entry(domain).or_insert(0) += 1;
        *self.applied_by_operator.entry(operator).or_insert(0) += 1;
        match semantic {
            MutationSemanticCategory::SemanticNoop => self.applied_semantic_noop_events += 1,
            MutationSemanticCategory::SemanticChange => self.applied_semantic_change_events += 1,
        }
    }

    pub fn record_skipped(&mut self, reason: MutationSkipReason) {
        self.skipped_events += 1;
        *self.skip_reasons.entry(reason).or_insert(0) += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mutation_summary_zero_has_zero_counts() {
        let s = MutationSummary::zero();
        assert_eq!(s.attempted_events, 0);
        assert_eq!(s.applied_events, 0);
        assert_eq!(s.skipped_events, 0);
        assert!(s.skip_reasons.is_empty());
        assert!(s.attempted_by_domain.is_empty());
        assert!(s.applied_by_domain.is_empty());
        assert!(s.attempted_by_operator.is_empty());
        assert!(s.applied_by_operator.is_empty());
        assert_eq!(s.applied_semantic_noop_events, 0);
        assert_eq!(s.applied_semantic_change_events, 0);
    }

    #[test]
    fn zero_summary_accounting_invariant() {
        let s = MutationSummary::zero();
        assert_eq!(s.attempted_events, s.applied_events + s.skipped_events);
    }

    #[test]
    fn complexity_effect_covers_all_operators() {
        for op in MutationOperator::all() {
            let effect = op.complexity_effect();
            assert!(
                matches!(
                    effect,
                    ComplexityEffect::Increasing
                        | ComplexityEffect::Decreasing
                        | ComplexityEffect::Neutral
                ),
                "complexity_effect must return a valid effect for {:?}",
                op
            );
        }
    }

    #[test]
    fn complexity_effect_has_at_least_one_of_each_kind() {
        let mut has_increasing = false;
        let mut has_decreasing = false;
        let mut has_neutral = false;
        for op in MutationOperator::all() {
            match op.complexity_effect() {
                ComplexityEffect::Increasing => has_increasing = true,
                ComplexityEffect::Decreasing => has_decreasing = true,
                ComplexityEffect::Neutral => has_neutral = true,
            }
        }
        assert!(has_increasing, "must have at least one Increasing operator");
        assert!(has_decreasing, "must have at least one Decreasing operator");
        assert!(has_neutral, "must have at least one Neutral operator");
    }

    #[test]
    fn complexity_effect_known_classifications() {
        // Spot-check known classifications
        assert_eq!(
            MutationOperator::TopologyAddNode.complexity_effect(),
            ComplexityEffect::Increasing
        );
        assert_eq!(
            MutationOperator::TopologyRemoveNode.complexity_effect(),
            ComplexityEffect::Decreasing
        );
        assert_eq!(
            MutationOperator::TopologyRetargetNodeTarget.complexity_effect(),
            ComplexityEffect::Neutral
        );
        assert_eq!(
            MutationOperator::InputRefAdd.complexity_effect(),
            ComplexityEffect::Increasing
        );
        assert_eq!(
            MutationOperator::InputRefRemove.complexity_effect(),
            ComplexityEffect::Decreasing
        );
        // Copy operators are increasing
        assert_eq!(
            MutationOperator::TopologyCopyNode.complexity_effect(),
            ComplexityEffect::Increasing
        );
        assert_eq!(
            MutationOperator::VmCopyInstructionBlock.complexity_effect(),
            ComplexityEffect::Increasing
        );
        assert_eq!(
            MutationOperator::GraphCopySubgraph.complexity_effect(),
            ComplexityEffect::Increasing
        );
        // Hebbian rule/rate mutations are neutral
        assert_eq!(
            MutationOperator::GraphMutateHebbianRule.complexity_effect(),
            ComplexityEffect::Neutral
        );
        assert_eq!(
            MutationOperator::GraphEnableHebbian.complexity_effect(),
            ComplexityEffect::Increasing
        );
        assert_eq!(
            MutationOperator::GraphDisableHebbian.complexity_effect(),
            ComplexityEffect::Decreasing
        );
    }

    #[test]
    fn operator_domain_mapping_is_consistent() {
        for operator in MutationOperator::all() {
            match operator {
                MutationOperator::TopologyAddNode
                | MutationOperator::TopologyRemoveNode
                | MutationOperator::TopologyRetargetNodeTarget
                | MutationOperator::TopologyAddRouteTarget
                | MutationOperator::TopologyRemoveRouteTarget
                | MutationOperator::TopologyChangeEntryNode
                | MutationOperator::TopologySwapNodeBackend
                | MutationOperator::TopologyRewriteNodeId
                | MutationOperator::TopologyCopyNode
                | MutationOperator::TopologyCopyMeshBackwardSlice
                | MutationOperator::TopologyCopyMeshForwardSlice
                | MutationOperator::TopologySpliceNode
                | MutationOperator::TopologySwapRouteTargets => {
                    assert_eq!(operator.domain(), MutationDomain::Topology)
                }
                MutationOperator::VmConstantMutation
                | MutationOperator::VmInstructionMutation
                | MutationOperator::VmRegisterCountMutation
                | MutationOperator::VmInstructionRawFieldMutation
                | MutationOperator::VmCopyInstructionBlock
                | MutationOperator::VmCopyInstructionBlockRemapped
                | MutationOperator::VmCopyConstantBlock
                | MutationOperator::VmCopyGeneBackwardSlice
                | MutationOperator::VmCopyGeneForwardSlice => {
                    assert_eq!(operator.domain(), MutationDomain::Vm)
                }
                MutationOperator::GraphAlterGraphEdgeWeight
                | MutationOperator::GraphSwapGraphOperator
                | MutationOperator::GraphMutateGraphOperatorParam
                | MutationOperator::GraphAddInternalGraphNode
                | MutationOperator::GraphRemoveInternalGraphNode
                | MutationOperator::GraphAddGraphEdge
                | MutationOperator::GraphRetargetGraphEdge
                | MutationOperator::GraphRemoveGraphEdge
                | MutationOperator::GraphRawFieldMutation
                | MutationOperator::GraphCopyInternalNode
                | MutationOperator::GraphCopySubgraph
                | MutationOperator::GraphCopyEdgeBundle
                | MutationOperator::GraphEnableHebbian
                | MutationOperator::GraphDisableHebbian
                | MutationOperator::GraphMutateHebbianRule
                | MutationOperator::GraphMutateHebbianRate
                | MutationOperator::GraphToggleHebbianLamarckian => {
                    assert_eq!(operator.domain(), MutationDomain::Graph)
                }
                MutationOperator::InputRefAdd
                | MutationOperator::InputRefRemove
                | MutationOperator::InputRefSwap
                | MutationOperator::InputRefRawFieldMutation => {
                    assert_eq!(operator.domain(), MutationDomain::InputRef)
                }
            }
        }
    }

    #[test]
    fn complexity_effect_cross_consistency_with_domain_operators() {
        use crate::mutation::graph::GraphOperator;
        use crate::mutation::input_ref::InputRefOperator;
        use crate::mutation::topology::TopologyOperator;
        use crate::mutation::vm::VmOperator;

        for &top in &TopologyOperator::ALL {
            let mo = match top {
                TopologyOperator::AddNode => MutationOperator::TopologyAddNode,
                TopologyOperator::RemoveNode => MutationOperator::TopologyRemoveNode,
                TopologyOperator::RetargetNodeTarget => {
                    MutationOperator::TopologyRetargetNodeTarget
                }
                TopologyOperator::AddRouteTarget => MutationOperator::TopologyAddRouteTarget,
                TopologyOperator::RemoveRouteTarget => MutationOperator::TopologyRemoveRouteTarget,
                TopologyOperator::ChangeEntryNode => MutationOperator::TopologyChangeEntryNode,
                TopologyOperator::SwapNodeBackend => MutationOperator::TopologySwapNodeBackend,
                TopologyOperator::RewriteNodeId => MutationOperator::TopologyRewriteNodeId,
                TopologyOperator::CopyNode => MutationOperator::TopologyCopyNode,
                TopologyOperator::CopyMeshBackwardSlice => {
                    MutationOperator::TopologyCopyMeshBackwardSlice
                }
                TopologyOperator::CopyMeshForwardSlice => {
                    MutationOperator::TopologyCopyMeshForwardSlice
                }
                TopologyOperator::SpliceNode => MutationOperator::TopologySpliceNode,
                TopologyOperator::SwapRouteTargets => MutationOperator::TopologySwapRouteTargets,
            };
            assert_eq!(
                mo.complexity_effect(),
                top.complexity_effect(),
                "MutationOperator and TopologyOperator disagree for {:?}",
                top
            );
        }

        for &vm in &VmOperator::ALL {
            let mo = match vm {
                VmOperator::VmConstantMutation => MutationOperator::VmConstantMutation,
                VmOperator::VmInstructionMutation => MutationOperator::VmInstructionMutation,
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
            };
            assert_eq!(
                mo.complexity_effect(),
                vm.complexity_effect(),
                "MutationOperator and VmOperator disagree for {:?}",
                vm
            );
        }

        for &graph in &GraphOperator::ALL {
            let mo = match graph {
                GraphOperator::AlterGraphEdgeWeight => MutationOperator::GraphAlterGraphEdgeWeight,
                GraphOperator::SwapGraphOperator => MutationOperator::GraphSwapGraphOperator,
                GraphOperator::MutateGraphOperatorParam => {
                    MutationOperator::GraphMutateGraphOperatorParam
                }
                GraphOperator::AddInternalGraphNode => MutationOperator::GraphAddInternalGraphNode,
                GraphOperator::RemoveInternalGraphNode => {
                    MutationOperator::GraphRemoveInternalGraphNode
                }
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
                GraphOperator::ToggleHebbianLamarckian => {
                    MutationOperator::GraphToggleHebbianLamarckian
                }
            };
            assert_eq!(
                mo.complexity_effect(),
                graph.complexity_effect(),
                "MutationOperator and GraphOperator disagree for {:?}",
                graph
            );
        }

        for &ir in &InputRefOperator::ALL {
            let mo = match ir {
                InputRefOperator::Add => MutationOperator::InputRefAdd,
                InputRefOperator::Remove => MutationOperator::InputRefRemove,
                InputRefOperator::Swap => MutationOperator::InputRefSwap,
                InputRefOperator::RawFieldMutation => MutationOperator::InputRefRawFieldMutation,
            };
            assert_eq!(
                mo.complexity_effect(),
                ir.complexity_effect(),
                "MutationOperator and InputRefOperator disagree for {:?}",
                ir
            );
        }
    }

    #[test]
    fn mutation_domain_layer_mapping() {
        assert_eq!(MutationDomain::Topology.layer(), MutationLayer::Mesh);
        assert_eq!(MutationDomain::Vm.layer(), MutationLayer::NodeInternal);
        assert_eq!(MutationDomain::Graph.layer(), MutationLayer::NodeInternal);
        assert_eq!(
            MutationDomain::InputRef.layer(),
            MutationLayer::NodeInternal
        );
    }
}
