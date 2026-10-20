use super::*;

#[test]
fn mutation_summary_zero_has_zero_counts() {
    let s = MutationSummary::zero();
    assert_eq!(s.attempted_events, 0);
    assert_eq!(s.applied_events, 0);
    assert_eq!(s.skipped_events, 0);
    assert!(s.skip_reasons.is_empty());
    assert!(s.skipped_by_operator.is_empty());
    assert!(s.attempted_by_domain.is_empty());
    assert!(s.applied_by_domain.is_empty());
    assert!(s.attempted_by_operator.is_empty());
    assert!(s.applied_by_operator.is_empty());
    assert!(s.operator_funnel_by_operator.is_empty());
    assert!(s.skip_reasons_by_operator.is_empty());
    assert!(s.added_node_input_classes_by_operator.is_empty());
    assert!(s.added_node_world_inputs_by_operator.is_empty());
    assert_eq!(s.reachable_target_events, 0);
    assert_eq!(s.unreachable_target_events, 0);
    assert_eq!(s.not_applicable_events, 0);
}

#[test]
fn added_node_input_class_keys_are_stable() {
    assert_eq!(MutationAddedNodeInputClass::None.as_key(), "none");
    assert_eq!(MutationAddedNodeInputClass::Food.as_key(), "food");
    assert_eq!(MutationAddedNodeInputClass::Barrier.as_key(), "barrier");
    assert_eq!(
        MutationAddedNodeInputClass::ActionQueue.as_key(),
        "action_queue"
    );
}

#[test]
fn mutation_skip_reason_keys_are_stable() {
    assert_eq!(
        MutationSkipReason::ParseabilityViolation.as_key(),
        "ParseabilityViolation"
    );
    assert_eq!(
        MutationSkipReason::NumericProposalRejected.as_key(),
        "NumericProposalRejected"
    );
    assert_eq!(
        MutationSkipReason::NoApplicableTarget.as_key(),
        "NoApplicableTarget"
    );
}

#[test]
fn world_input_key_keys_are_stable() {
    assert_eq!(
        crate::contracts::WorldInputKey::FoodHere {
            type_idx: crate::config::OrdinaryFoodTypeId::default()
        }
        .as_key(),
        "FoodHere"
    );
    assert_eq!(
        crate::contracts::WorldInputKey::AreaFoodSummary {
            type_idx: crate::config::OrdinaryFoodTypeId::default()
        }
        .as_key(),
        "AreaFoodSummary"
    );
    assert_eq!(
        crate::contracts::WorldInputKey::NearbyCreature.as_key(),
        "NearbyCreature"
    );
}

#[test]
fn record_skipped_tracks_skip_reason_and_operator() {
    let mut s = MutationSummary::zero();
    s.record_skipped(
        MutationOperator::VmInstructionMutation,
        MutationSkipReason::NoApplicableTarget,
    );
    assert_eq!(s.skipped_events, 1);
    assert_eq!(
        s.skip_reasons.get(&MutationSkipReason::NoApplicableTarget),
        Some(&1)
    );
    assert_eq!(
        s.skipped_by_operator
            .get(&MutationOperator::VmInstructionMutation),
        Some(&1)
    );
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
        MutationOperator::InputRefPrune.complexity_effect(),
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
        MutationOperator::VmDeleteInstruction.complexity_effect(),
        ComplexityEffect::Decreasing
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
            | MutationOperator::TopologyAddFeedbackRoute
            | MutationOperator::TopologyRemoveRouteTarget
            | MutationOperator::TopologyChangeEntryNode
            | MutationOperator::TopologySwapNodeBackend
            | MutationOperator::TopologyCopyNode
            | MutationOperator::TopologyCopyMeshBackwardSlice
            | MutationOperator::TopologyCopyMeshForwardSlice
            | MutationOperator::TopologySpliceNode
            | MutationOperator::TopologySwapRouteTargets
            | MutationOperator::TopologyMutateGateBias => {
                assert_eq!(operator.domain(), MutationDomain::Topology)
            }
            MutationOperator::VmConstantMutation
            | MutationOperator::VmInstructionMutation
            | MutationOperator::VmDeleteInstruction
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
            | MutationOperator::GraphMutateTraceDecay
            | MutationOperator::GraphRecruitNeutralInput
            | MutationOperator::GraphAddRingGroup
            | MutationOperator::GraphAddDirectionPairGroup
            | MutationOperator::GraphAlterGroupScale
            | MutationOperator::GraphShiftGroupOffset
            | MutationOperator::GraphRetargetGroupKind
            | MutationOperator::GraphRemoveGroup
            | MutationOperator::GraphSetScope
            | MutationOperator::GraphAddRandomizedRingGroup
            | MutationOperator::GraphAddRandomizedDirectionPairGroup
            | MutationOperator::GraphDetachGraphEdge => {
                assert_eq!(operator.domain(), MutationDomain::Graph)
            }
            MutationOperator::InputRefAdd
            | MutationOperator::InputRefPrune
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
            TopologyOperator::AddFeedbackRoute => MutationOperator::TopologyAddFeedbackRoute,
            TopologyOperator::RemoveRouteTarget => MutationOperator::TopologyRemoveRouteTarget,
            TopologyOperator::ChangeEntryNode => MutationOperator::TopologyChangeEntryNode,
            TopologyOperator::SwapNodeBackend => MutationOperator::TopologySwapNodeBackend,
            TopologyOperator::CopyNode => MutationOperator::TopologyCopyNode,
            TopologyOperator::CopyMeshBackwardSlice => {
                MutationOperator::TopologyCopyMeshBackwardSlice
            }
            TopologyOperator::CopyMeshForwardSlice => {
                MutationOperator::TopologyCopyMeshForwardSlice
            }
            TopologyOperator::SpliceNode => MutationOperator::TopologySpliceNode,
            TopologyOperator::SwapRouteTargets => MutationOperator::TopologySwapRouteTargets,
            TopologyOperator::MutateGateBias => MutationOperator::TopologyMutateGateBias,
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
            VmOperator::VmDeleteInstruction => MutationOperator::VmDeleteInstruction,
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
            GraphOperator::RecruitNeutralInput => MutationOperator::GraphRecruitNeutralInput,
            GraphOperator::AddRingGroup => MutationOperator::GraphAddRingGroup,
            GraphOperator::AddDirectionPairGroup => MutationOperator::GraphAddDirectionPairGroup,
            GraphOperator::AlterGroupScale => MutationOperator::GraphAlterGroupScale,
            GraphOperator::ShiftGroupOffset => MutationOperator::GraphShiftGroupOffset,
            GraphOperator::RetargetGroupKind => MutationOperator::GraphRetargetGroupKind,
            GraphOperator::RemoveGroup => MutationOperator::GraphRemoveGroup,
            GraphOperator::SetScope => MutationOperator::GraphSetScope,
            GraphOperator::AddRandomizedRingGroup => MutationOperator::GraphAddRandomizedRingGroup,
            GraphOperator::AddRandomizedDirectionPairGroup => {
                MutationOperator::GraphAddRandomizedDirectionPairGroup
            }
            GraphOperator::DetachGraphEdge => MutationOperator::GraphDetachGraphEdge,
        };
        // `complexity_effect` delegates to `operator()`, so pinning the
        // mapping pins the classification.
        assert_eq!(
            mo,
            graph.operator(),
            "GraphOperator maps {:?} to the wrong MutationOperator",
            graph
        );
    }

    for &ir in &InputRefOperator::ALL {
        let mo = match ir {
            InputRefOperator::Add => MutationOperator::InputRefAdd,
            InputRefOperator::Prune => MutationOperator::InputRefPrune,
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

/// The Graph catalog is one consistent set (T25.F08 invariant 1): `ALL`
/// covers every variant once, `operator()` is injective onto exactly the
/// Graph-domain members of `MutationOperator::all()`, every key is unique
/// and `Graph.`-prefixed, the two enums agree on complexity, and `all()` is
/// the four domains' catalogs. The hand-written map above stays the
/// identity oracle.
#[test]
fn the_graph_catalog_is_a_bijection_onto_its_engine_operators() {
    use crate::mutation::graph::GraphOperator;
    use crate::mutation::input_ref::InputRefOperator;
    use crate::mutation::topology::TopologyOperator;
    use crate::mutation::vm::VmOperator;
    use std::collections::HashSet;

    let variants: HashSet<GraphOperator> = GraphOperator::ALL.into_iter().collect();
    assert_eq!(
        variants.len(),
        GraphOperator::ALL.len(),
        "ALL lists each once"
    );
    // A variant missing from `ALL` fails to compile here.
    for &op in &GraphOperator::ALL {
        match op {
            GraphOperator::AlterGraphEdgeWeight
            | GraphOperator::SwapGraphOperator
            | GraphOperator::MutateGraphOperatorParam
            | GraphOperator::AddInternalGraphNode
            | GraphOperator::RemoveInternalGraphNode
            | GraphOperator::AddGraphEdge
            | GraphOperator::RetargetGraphEdge
            | GraphOperator::RemoveGraphEdge
            | GraphOperator::GraphRawFieldMutation
            | GraphOperator::CopyInternalNode
            | GraphOperator::CopySubgraph
            | GraphOperator::CopyEdgeBundle
            | GraphOperator::EnableHebbian
            | GraphOperator::DisableHebbian
            | GraphOperator::MutateHebbianRule
            | GraphOperator::MutateHebbianRate
            | GraphOperator::ToggleHebbianLamarckian
            | GraphOperator::EnableRewardModulation
            | GraphOperator::DisableRewardModulation
            | GraphOperator::MutateRewardSource
            | GraphOperator::MutateTraceDecay
            | GraphOperator::RecruitNeutralInput
            | GraphOperator::AddRingGroup
            | GraphOperator::AddDirectionPairGroup
            | GraphOperator::AlterGroupScale
            | GraphOperator::ShiftGroupOffset
            | GraphOperator::RetargetGroupKind
            | GraphOperator::RemoveGroup
            | GraphOperator::SetScope
            | GraphOperator::AddRandomizedRingGroup
            | GraphOperator::AddRandomizedDirectionPairGroup
            | GraphOperator::DetachGraphEdge => {}
        }
    }
    assert_eq!(variants.len(), 32);

    let image: HashSet<MutationOperator> = GraphOperator::ALL
        .into_iter()
        .map(GraphOperator::operator)
        .collect();
    assert_eq!(
        image.len(),
        GraphOperator::ALL.len(),
        "operator() is injective"
    );
    let graph_domain: HashSet<MutationOperator> = MutationOperator::all()
        .into_iter()
        .filter(|op| op.domain() == MutationDomain::Graph)
        .collect();
    assert_eq!(image, graph_domain, "the image is the Graph domain");

    let keys: HashSet<&str> = GraphOperator::ALL
        .into_iter()
        .map(|op| op.operator().as_key())
        .collect();
    assert_eq!(keys.len(), GraphOperator::ALL.len(), "keys are unique");
    assert!(keys.iter().all(|key| key.starts_with("Graph.")));
    for &op in &GraphOperator::ALL {
        assert_eq!(op.complexity_effect(), op.operator().complexity_effect());
    }

    assert_eq!(
        MutationOperator::all().len(),
        TopologyOperator::ALL.len()
            + VmOperator::ALL.len()
            + GraphOperator::ALL.len()
            + InputRefOperator::ALL.len()
    );
    let all_keys: HashSet<&str> = MutationOperator::all()
        .into_iter()
        .map(MutationOperator::as_key)
        .collect();
    assert_eq!(all_keys.len(), MutationOperator::all().len());
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
    s.record_applied(MutationDomain::Topology, MutationOperator::TopologyAddNode);
    s.record_reachability(TargetReachability::Reachable);

    s.record_applied(MutationDomain::Vm, MutationOperator::VmConstantMutation);
    s.record_reachability(TargetReachability::Unreachable);

    s.record_applied(MutationDomain::Topology, MutationOperator::TopologyAddNode);
    s.record_reachability(TargetReachability::NotApplicable);

    let reachability_total =
        s.reachable_target_events + s.unreachable_target_events + s.not_applicable_events;
    assert_eq!(reachability_total, s.applied_events);
}

#[test]
fn operator_funnel_tracks_stage_counts_and_skip_reason_breakdown() {
    let mut s = MutationSummary::zero();
    let operator = MutationOperator::VmInstructionMutation;
    let domain = MutationDomain::Vm;

    s.record_attempt(domain, operator);
    s.record_applied(domain, operator);

    s.record_attempt(domain, operator);
    s.record_skipped(operator, MutationSkipReason::NoApplicableTarget);

    s.record_attempt(domain, operator);
    s.record_skipped(operator, MutationSkipReason::ParseabilityViolation);

    let funnel = s
        .operator_funnel_by_operator
        .get(&operator)
        .expect("operator funnel should exist");
    assert_eq!(funnel.attempted, 3);
    assert_eq!(funnel.applicable, 2);
    assert_eq!(funnel.structurally_valid, 1);
    assert_eq!(funnel.applied, 1);
    assert_eq!(funnel.skipped, 2);

    let by_reason = s
        .skip_reasons_by_operator
        .get(&operator)
        .expect("operator skip reasons should exist");
    assert_eq!(
        by_reason.get(&MutationSkipReason::NoApplicableTarget),
        Some(&1)
    );
    assert_eq!(
        by_reason.get(&MutationSkipReason::ParseabilityViolation),
        Some(&1)
    );
}

#[test]
fn operator_keys_are_unique_and_domain_qualified() {
    let operators = MutationOperator::all();
    let keys: std::collections::HashSet<_> =
        operators.iter().map(|operator| operator.as_key()).collect();
    assert_eq!(keys.len(), operators.len());
    for operator in operators {
        assert!(operator.as_key().contains('.'));
    }
}
