use std::collections::HashMap;

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
            | Self::TopologyCopyNode => MutationDomain::Topology,
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
            | Self::GraphCopyEdgeBundle => MutationDomain::Graph,
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
    pub const fn all() -> [Self; 34] {
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
                | MutationOperator::TopologyCopyNode => {
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
                | MutationOperator::GraphCopyEdgeBundle => {
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
}
