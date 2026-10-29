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
        .family_name(),
        "FoodHere"
    );
    assert_eq!(
        crate::contracts::WorldInputKey::AreaFoodSummary {
            type_idx: crate::config::OrdinaryFoodTypeId::default()
        }
        .family_name(),
        "AreaFoodSummary"
    );
    assert_eq!(
        crate::contracts::WorldInputKey::NearbyCreature.family_name(),
        "NearbyCreature"
    );
}

#[test]
fn record_skipped_tracks_skip_reason_and_operator() {
    let mut s = MutationSummary::zero();
    s.record_skipped(
        MutationOperator::Vm(VmOperator::VmInstructionMutation),
        MutationSkipReason::NoApplicableTarget,
    );
    assert_eq!(s.skipped_events, 1);
    assert_eq!(
        s.skip_reasons.get(&MutationSkipReason::NoApplicableTarget),
        Some(&1)
    );
    assert_eq!(
        s.skipped_by_operator
            .get(&MutationOperator::Vm(VmOperator::VmInstructionMutation)),
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
    for op in MutationOperator::ALL {
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
    for op in MutationOperator::ALL {
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
        MutationOperator::Topology(TopologyOperator::AddNode).complexity_effect(),
        ComplexityEffect::Increasing
    );
    assert_eq!(
        MutationOperator::Topology(TopologyOperator::RemoveNode).complexity_effect(),
        ComplexityEffect::Decreasing
    );
    assert_eq!(
        MutationOperator::Topology(TopologyOperator::RetargetNodeTarget).complexity_effect(),
        ComplexityEffect::Neutral
    );
    // The four creation operators are increasing (T25.F11 invariant 7).
    for creation in [
        MutationOperator::InputRef(InputRefOperator::AddSensorLive),
        MutationOperator::InputRef(InputRefOperator::AddSensorSilent),
        MutationOperator::Graph(GraphOperator::AddBundleLive),
        MutationOperator::Graph(GraphOperator::AddBundleSilent),
    ] {
        assert_eq!(
            creation.complexity_effect(),
            ComplexityEffect::Increasing,
            "{creation:?}"
        );
    }
    assert_eq!(
        MutationOperator::InputRef(InputRefOperator::Prune).complexity_effect(),
        ComplexityEffect::Decreasing
    );
    // Copy operators are increasing
    assert_eq!(
        MutationOperator::Topology(TopologyOperator::CopyNode).complexity_effect(),
        ComplexityEffect::Increasing
    );
    assert_eq!(
        MutationOperator::Vm(VmOperator::VmCopyInstructionBlock).complexity_effect(),
        ComplexityEffect::Increasing
    );
    assert_eq!(
        MutationOperator::Vm(VmOperator::VmDeleteInstruction).complexity_effect(),
        ComplexityEffect::Decreasing
    );
    assert_eq!(
        MutationOperator::Graph(GraphOperator::CopySubgraph).complexity_effect(),
        ComplexityEffect::Increasing
    );
    // Hebbian rule/rate mutations are neutral
    assert_eq!(
        MutationOperator::Graph(GraphOperator::MutateHebbianRule).complexity_effect(),
        ComplexityEffect::Neutral
    );
    assert_eq!(
        MutationOperator::Graph(GraphOperator::EnableHebbian).complexity_effect(),
        ComplexityEffect::Increasing
    );
    assert_eq!(
        MutationOperator::Graph(GraphOperator::DisableHebbian).complexity_effect(),
        ComplexityEffect::Decreasing
    );
}

#[test]
fn operator_domain_mapping_is_consistent() {
    for operator in MutationOperator::ALL {
        let domain = match operator {
            MutationOperator::Topology(_) => MutationDomain::Topology,
            MutationOperator::Vm(_) => MutationDomain::Vm,
            MutationOperator::Graph(_) => MutationDomain::Graph,
            MutationOperator::InputRef(_) => MutationDomain::InputRef,
        };
        assert_eq!(operator.domain(), domain);
    }
}

/// Every domain operator maps through `From` to its own nested variant and
/// keeps its domain's complexity effect there.
#[test]
fn complexity_effect_cross_consistency_with_domain_operators() {
    use crate::mutation::graph::GraphOperator;
    use crate::mutation::input_ref::InputRefOperator;
    use crate::mutation::topology::TopologyOperator;
    use crate::mutation::vm::VmOperator;

    for op in TopologyOperator::ALL {
        let mapped = MutationOperator::from(op);
        assert_eq!(mapped, MutationOperator::Topology(op));
        assert_eq!(mapped.complexity_effect(), op.complexity_effect(), "{op:?}");
    }
    for op in VmOperator::ALL {
        let mapped = MutationOperator::from(op);
        assert_eq!(mapped, MutationOperator::Vm(op));
        assert_eq!(mapped.complexity_effect(), op.complexity_effect(), "{op:?}");
    }
    for op in GraphOperator::ALL {
        let mapped = MutationOperator::from(op);
        assert_eq!(mapped, MutationOperator::Graph(op));
        assert_eq!(mapped.complexity_effect(), op.complexity_effect(), "{op:?}");
    }
    for op in InputRefOperator::ALL {
        let mapped = MutationOperator::from(op);
        assert_eq!(mapped, MutationOperator::InputRef(op));
        assert_eq!(mapped.complexity_effect(), op.complexity_effect(), "{op:?}");
    }
}

/// The Graph catalog is one consistent set (T25.F08 invariant 1): `ALL`
/// covers every variant once, `From` is injective onto exactly the
/// Graph-domain members of `MutationOperator::ALL`, every key is unique
/// and `Graph.`-prefixed, and `MutationOperator::ALL` is the four domains'
/// catalogs.
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
            | GraphOperator::CopyNodeInputs
            | GraphOperator::EnableHebbian
            | GraphOperator::DisableHebbian
            | GraphOperator::ClearLearningOverride
            | GraphOperator::MutateHebbianRule
            | GraphOperator::MutateHebbianRate
            | GraphOperator::ToggleHebbianLamarckian
            | GraphOperator::EnableRewardModulation
            | GraphOperator::DisableRewardModulation
            | GraphOperator::MutateRewardSource
            | GraphOperator::MutateTraceDecay
            | GraphOperator::AddBundleLive
            | GraphOperator::AddBundleSilent
            | GraphOperator::AlterBundleScale
            | GraphOperator::ShiftBundleOffset
            | GraphOperator::RetargetBundle
            | GraphOperator::RemoveBundle
            | GraphOperator::ToggleCredits
            | GraphOperator::DetachGraphEdge
            | GraphOperator::EnableRetention
            | GraphOperator::DisableRetention
            | GraphOperator::MutateRetention => {}
        }
    }
    // T25.F12 invariant 9: `SetScope` and the three bundle retention
    // operators (weights 2, 1, 1, 4) leave, `ClearLearningOverride` (1) and
    // `ToggleCredits` (2) join (33 at weight 73); `InputRef.Add` is
    // `AddSensorLive` and `AddSensorSilent` (5 at weight 12, T25.F11).
    assert_eq!(variants.len(), 33);
    let weight = |weights: &mut dyn Iterator<Item = u8>| weights.map(u32::from).sum::<u32>();
    assert_eq!(
        weight(&mut GraphOperator::ALL.into_iter().map(GraphOperator::weight)),
        73
    );
    assert_eq!(
        weight(
            &mut InputRefOperator::ALL
                .into_iter()
                .map(InputRefOperator::weight)
        ),
        12
    );
    assert_eq!(InputRefOperator::ALL.len(), 5);
    assert_eq!(MutationOperator::ALL.len(), 67);

    let image: HashSet<MutationOperator> = GraphOperator::ALL
        .into_iter()
        .map(MutationOperator::from)
        .collect();
    assert_eq!(image.len(), GraphOperator::ALL.len(), "From is injective");
    let graph_domain: HashSet<MutationOperator> = MutationOperator::ALL
        .into_iter()
        .filter(|op| op.domain() == MutationDomain::Graph)
        .collect();
    assert_eq!(image, graph_domain, "the image is the Graph domain");

    let keys: HashSet<&str> = GraphOperator::ALL
        .into_iter()
        .map(|op| MutationOperator::from(op).as_key())
        .collect();
    assert_eq!(keys.len(), GraphOperator::ALL.len(), "keys are unique");
    assert!(keys.iter().all(|key| key.starts_with("Graph.")));

    assert_eq!(
        MutationOperator::ALL.len(),
        TopologyOperator::ALL.len()
            + VmOperator::ALL.len()
            + GraphOperator::ALL.len()
            + InputRefOperator::ALL.len()
    );
    let all_keys: HashSet<&str> = MutationOperator::ALL
        .into_iter()
        .map(MutationOperator::as_key)
        .collect();
    assert_eq!(all_keys.len(), MutationOperator::ALL.len());
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
        MutationOperator::Topology(TopologyOperator::AddNode),
    );
    s.record_reachability(TargetReachability::Reachable);

    s.record_applied(
        MutationDomain::Vm,
        MutationOperator::Vm(VmOperator::VmConstantMutation),
    );
    s.record_reachability(TargetReachability::Unreachable);

    s.record_applied(
        MutationDomain::Topology,
        MutationOperator::Topology(TopologyOperator::AddNode),
    );
    s.record_reachability(TargetReachability::NotApplicable);

    let reachability_total =
        s.reachable_target_events + s.unreachable_target_events + s.not_applicable_events;
    assert_eq!(reachability_total, s.applied_events);
}

#[test]
fn operator_funnel_tracks_stage_counts_and_skip_reason_breakdown() {
    let mut s = MutationSummary::zero();
    let operator = MutationOperator::Vm(VmOperator::VmInstructionMutation);
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
    let operators = MutationOperator::ALL;
    let keys: std::collections::HashSet<_> =
        operators.iter().map(|operator| operator.as_key()).collect();
    assert_eq!(keys.len(), operators.len());
    for operator in operators {
        assert!(operator.as_key().contains('.'));
    }
}

/// Every operator's key in the flat order (T30.F10 invariant 2, T25.F12
/// invariant 9's catalog): the nested catalog reproduces it verbatim.
#[rustfmt::skip]
const KEYS: [&str; 67] = [
    "Topology.AddNode", "Topology.RemoveNode", "Topology.RetargetNodeTarget",
    "Topology.AddRouteTarget", "Topology.AddFeedbackRoute", "Topology.RemoveRouteTarget",
    "Topology.ChangeEntryNode", "Topology.SwapNodeBackend", "Topology.CopyNode",
    "Topology.CopyMeshBackwardSlice", "Topology.CopyMeshForwardSlice", "Topology.SpliceNode",
    "Topology.SwapRouteTargets", "Topology.MutateGateBias",
    "Vm.VmConstantMutation", "Vm.VmInstructionMutation", "Vm.VmDeleteInstruction",
    "Vm.VmRegisterCountMutation", "Vm.VmInstructionRawFieldMutation", "Vm.CopyInstructionBlock",
    "Vm.CopyInstructionBlockRemapped", "Vm.CopyConstantBlock", "Vm.CopyGeneBackwardSlice",
    "Vm.CopyGeneForwardSlice", "Vm.InsertReadStoreMotif", "Vm.InsertReadBidMotif",
    "Vm.InsertLoadCompareMotif", "Vm.MutateSlotAddress", "Vm.MutatePairedSlotAddress",
    "Graph.AlterGraphEdgeWeight", "Graph.SwapGraphOperator", "Graph.MutateGraphOperatorParam",
    "Graph.AddInternalGraphNode", "Graph.RemoveInternalGraphNode", "Graph.AddGraphEdge",
    "Graph.RetargetGraphEdge", "Graph.RemoveGraphEdge", "Graph.GraphRawFieldMutation",
    "Graph.CopyInternalNode", "Graph.CopySubgraph", "Graph.CopyNodeInputs",
    "Graph.EnableHebbian", "Graph.DisableHebbian", "Graph.ClearLearningOverride",
    "Graph.MutateHebbianRule",
    "Graph.MutateHebbianRate", "Graph.ToggleHebbianLamarckian", "Graph.EnableRewardModulation",
    "Graph.DisableRewardModulation", "Graph.MutateRewardSource", "Graph.MutateTraceDecay",
    "Graph.AddBundleLive", "Graph.AddBundleSilent", "Graph.AlterBundleScale",
    "Graph.ShiftBundleOffset", "Graph.RetargetBundle", "Graph.RemoveBundle",
    "Graph.ToggleCredits", "Graph.DetachGraphEdge", "Graph.EnableRetention",
    "Graph.DisableRetention", "Graph.MutateRetention",
    "InputRef.AddSensorLive", "InputRef.AddSensorSilent", "InputRef.Prune", "InputRef.Swap",
    "InputRef.RawFieldMutation",
];

/// T30.F10 invariants 1-3: the nested catalog derives every operator fact
/// from its domain enum, in today's order, keys and serde form.
#[test]
fn the_nested_catalog_derives_every_operator_fact() {
    use crate::config::MutationConfig;
    use crate::mutation::graph::GraphOperator;
    use crate::mutation::input_ref::InputRefOperator;
    use crate::mutation::topology::TopologyOperator;
    use crate::mutation::vm::VmOperator;
    use std::collections::HashSet;

    let keys: Vec<&str> = MutationOperator::ALL.iter().map(|op| op.as_key()).collect();
    assert_eq!(keys, KEYS, "keys and order are today's");
    assert_eq!(
        keys.iter().collect::<HashSet<_>>().len(),
        MutationOperator::ALL.len()
    );
    assert!(
        MutationOperator::ALL.windows(2).all(|w| w[0] < w[1]),
        "the derived nested order is the flat order"
    );

    let expected: Vec<MutationOperator> = TopologyOperator::ALL
        .into_iter()
        .map(MutationOperator::from)
        .chain(VmOperator::ALL.into_iter().map(MutationOperator::from))
        .chain(GraphOperator::ALL.into_iter().map(MutationOperator::from))
        .chain(
            InputRefOperator::ALL
                .into_iter()
                .map(MutationOperator::from),
        )
        .collect();
    assert_eq!(MutationOperator::ALL.to_vec(), expected);
    let catalog: Vec<MutationOperator> = MutationDomain::all()
        .into_iter()
        .flat_map(|domain| domain.catalog().iter().copied())
        .collect();
    assert_eq!(catalog, expected);

    let config = MutationConfig {
        large_copy_weight_percent: 30,
        ..MutationConfig::default()
    };
    for domain in MutationDomain::all() {
        for &op in domain.catalog() {
            assert_eq!(op.domain(), domain);
            assert!(op.as_key().starts_with(&format!("{}.", domain.as_key())));
            let json = serde_json::to_string(&op).unwrap();
            assert_eq!(json, format!("\"{}\"", op.as_key()));
            assert_eq!(serde_json::from_str::<MutationOperator>(&json).unwrap(), op);
        }
    }
    for &op in &MutationOperator::ALL {
        let (effect, weight) = match op {
            MutationOperator::Topology(o) => (o.complexity_effect(), o.tuned_weight(30)),
            MutationOperator::Vm(o) => (o.complexity_effect(), u16::from(o.weight())),
            MutationOperator::Graph(o) => (o.complexity_effect(), u16::from(o.weight())),
            MutationOperator::InputRef(o) => (o.complexity_effect(), u16::from(o.weight())),
        };
        assert_eq!(op.complexity_effect(), effect, "{op:?}");
        assert_eq!(op.weight(&config), weight, "{op:?}");
    }
    assert!(serde_json::from_str::<MutationOperator>("\"GraphAddGraphEdge\"").is_err());

    let debug = |op: MutationOperator| format!("{op:?}");
    assert_eq!(debug(TopologyOperator::AddNode.into()), "TopologyAddNode");
    assert_eq!(
        debug(VmOperator::VmConstantMutation.into()),
        "VmConstantMutation"
    );
    assert_eq!(
        debug(VmOperator::VmCopyInstructionBlock.into()),
        "VmCopyInstructionBlock"
    );
    assert_eq!(
        debug(GraphOperator::GraphRawFieldMutation.into()),
        "GraphRawFieldMutation"
    );
    assert_eq!(
        debug(GraphOperator::AddGraphEdge.into()),
        "GraphAddGraphEdge"
    );
    assert_eq!(debug(InputRefOperator::Prune.into()), "InputRefPrune");
}
