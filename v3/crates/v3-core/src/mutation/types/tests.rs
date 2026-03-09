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
    assert_eq!(s.reachable_target_events, 0);
    assert_eq!(s.unreachable_target_events, 0);
    assert_eq!(s.not_applicable_events, 0);
}

#[test]
fn zero_summary_accounting_invariant() {
    let s = MutationSummary::zero();
    assert_eq!(s.attempted_events, s.applied_events + s.skipped_events);
}

#[test]
fn is_decreasing_returns_true_only_for_decreasing() {
    assert!(!ComplexityEffect::Increasing.is_decreasing());
    assert!(ComplexityEffect::Decreasing.is_decreasing());
    assert!(!ComplexityEffect::Neutral.is_decreasing());
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
            | MutationOperator::VmCopyGeneForwardSlice
            | MutationOperator::VmInsertReadStoreMotif
            | MutationOperator::VmInsertReadBidMotif
            | MutationOperator::VmInsertLoadCompareMotif
            | MutationOperator::VmMutateSlotAddress
            | MutationOperator::VmMutatePairedSlotAddress => {
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
            | MutationOperator::GraphToggleHebbianLamarckian
            | MutationOperator::GraphEnableRewardModulation
            | MutationOperator::GraphDisableRewardModulation
            | MutationOperator::GraphMutateRewardSource
            | MutationOperator::GraphMutateTraceDecay => {
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
            TopologyOperator::RetargetNodeTarget => MutationOperator::TopologyRetargetNodeTarget,
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
            VmOperator::VmInsertReadStoreMotif => MutationOperator::VmInsertReadStoreMotif,
            VmOperator::VmInsertReadBidMotif => MutationOperator::VmInsertReadBidMotif,
            VmOperator::VmInsertLoadCompareMotif => MutationOperator::VmInsertLoadCompareMotif,
            VmOperator::VmMutateSlotAddress => MutationOperator::VmMutateSlotAddress,
            VmOperator::VmMutatePairedSlotAddress => MutationOperator::VmMutatePairedSlotAddress,
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
            GraphOperator::EnableRewardModulation => MutationOperator::GraphEnableRewardModulation,
            GraphOperator::DisableRewardModulation => {
                MutationOperator::GraphDisableRewardModulation
            }
            GraphOperator::MutateRewardSource => MutationOperator::GraphMutateRewardSource,
            GraphOperator::MutateTraceDecay => MutationOperator::GraphMutateTraceDecay,
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

#[test]
fn record_reachability_increments_correct_counter() {
    let mut s = MutationSummary::zero();
    s.record_reachability(TargetReachability::Reachable);
    s.record_reachability(TargetReachability::Reachable);
    s.record_reachability(TargetReachability::Unreachable);
    s.record_reachability(TargetReachability::NotApplicable);
    assert_eq!(s.reachable_target_events, 2);
    assert_eq!(s.unreachable_target_events, 1);
    assert_eq!(s.not_applicable_events, 1);
}

#[test]
fn reachability_accounting_matches_applied_events() {
    let mut s = MutationSummary::zero();
    // Simulate 3 applied events with reachability tracking
    s.record_applied(
        MutationDomain::Topology,
        MutationOperator::TopologyAddNode,
        MutationSemanticCategory::SemanticChange,
    );
    s.record_reachability(TargetReachability::Reachable);

    s.record_applied(
        MutationDomain::Vm,
        MutationOperator::VmConstantMutation,
        MutationSemanticCategory::SemanticChange,
    );
    s.record_reachability(TargetReachability::Unreachable);

    s.record_applied(
        MutationDomain::Topology,
        MutationOperator::TopologyAddNode,
        MutationSemanticCategory::SemanticChange,
    );
    s.record_reachability(TargetReachability::NotApplicable);

    let reachability_total =
        s.reachable_target_events + s.unreachable_target_events + s.not_applicable_events;
    assert_eq!(reachability_total, s.applied_events);
}
