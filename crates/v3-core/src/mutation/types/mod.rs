use std::collections::HashMap;

use crate::contracts::{InputReference, NodeId, WorldInputKey};

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

    /// Returns true if this effect is `Decreasing`.
    #[must_use]
    pub const fn is_decreasing(self) -> bool {
        matches!(self, Self::Decreasing)
    }
}

/// Reason a mutation event was skipped.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
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
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum MutationDomain {
    Topology,
    Vm,
    Graph,
    InputRef,
}

/// Input class attached or wired when a mutation adds a new node.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum MutationAddedNodeInputClass {
    None,
    Food,
    Neighbor,
    Barrier,
    Occupancy,
    Introspection,
    Upstream,
    ActionQueue,
}

impl MutationAddedNodeInputClass {
    #[must_use]
    pub const fn as_key(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Food => "food",
            Self::Neighbor => "neighbor",
            Self::Barrier => "barrier",
            Self::Occupancy => "occupancy",
            Self::Introspection => "introspection",
            Self::Upstream => "upstream",
            Self::ActionQueue => "action_queue",
        }
    }
}

impl From<&InputReference> for MutationAddedNodeInputClass {
    fn from(input_ref: &InputReference) -> Self {
        match input_ref {
            InputReference::World(key) => match key {
                WorldInputKey::FoodHere { .. }
                | WorldInputKey::NeighborFoodRing { .. }
                | WorldInputKey::AreaFoodSummary { .. } => Self::Food,
                WorldInputKey::NeighborBarrierRing | WorldInputKey::AreaBarrierSummary => {
                    Self::Barrier
                }
                WorldInputKey::NeighborOccupiedRing | WorldInputKey::AreaOccupancySummary => {
                    Self::Occupancy
                }
                WorldInputKey::NearbyCreatureCore
                | WorldInputKey::NearbyCreatureVitals
                | WorldInputKey::NearbyCreatureIdentity => Self::Neighbor,
            },
            InputReference::StaticIntrospection(_) | InputReference::DynamicIntrospection(_) => {
                Self::Introspection
            }
            InputReference::UpstreamSlot(_) => Self::Upstream,
            InputReference::ActionQueue => Self::ActionQueue,
        }
    }
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
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum MutationOperator {
    // Topology
    TopologyAddNode,
    TopologyRemoveNode,
    TopologyRetargetNodeTarget,
    TopologyAddRouteTarget,
    TopologyRemoveRouteTarget,
    TopologyChangeEntryNode,
    TopologySwapNodeBackend,
    TopologyCopyNode,
    TopologyCopyMeshBackwardSlice,
    TopologyCopyMeshForwardSlice,
    TopologySpliceNode,
    TopologySwapRouteTargets,
    TopologyMutateGateBias,
    // VM
    VmConstantMutation,
    VmInstructionMutation,
    VmDeleteInstruction,
    VmRegisterCountMutation,
    VmInstructionRawFieldMutation,
    VmCopyInstructionBlock,
    VmCopyInstructionBlockRemapped,
    VmCopyConstantBlock,
    VmCopyGeneBackwardSlice,
    VmCopyGeneForwardSlice,
    VmInsertReadStoreMotif,
    VmInsertReadBidMotif,
    VmInsertLoadCompareMotif,
    VmMutateSlotAddress,
    VmMutatePairedSlotAddress,
    // Graph
    GraphAlterGraphEdgeWeight,
    GraphSwapGraphOperator,
    GraphMutateGraphOperatorParam,
    GraphMutateActionSlotBehavior,
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
    GraphEnableRewardModulation,
    GraphDisableRewardModulation,
    GraphMutateRewardSource,
    GraphMutateTraceDecay,
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
            Self::TopologyCopyNode => "Topology.CopyNode",
            Self::TopologyCopyMeshBackwardSlice => "Topology.CopyMeshBackwardSlice",
            Self::TopologyCopyMeshForwardSlice => "Topology.CopyMeshForwardSlice",
            Self::TopologySpliceNode => "Topology.SpliceNode",
            Self::TopologySwapRouteTargets => "Topology.SwapRouteTargets",
            Self::TopologyMutateGateBias => "Topology.MutateGateBias",
            Self::VmConstantMutation => "Vm.VmConstantMutation",
            Self::VmInstructionMutation => "Vm.VmInstructionMutation",
            Self::VmDeleteInstruction => "Vm.VmDeleteInstruction",
            Self::VmRegisterCountMutation => "Vm.VmRegisterCountMutation",
            Self::VmInstructionRawFieldMutation => "Vm.VmInstructionRawFieldMutation",
            Self::VmCopyInstructionBlock => "Vm.CopyInstructionBlock",
            Self::VmCopyInstructionBlockRemapped => "Vm.CopyInstructionBlockRemapped",
            Self::VmCopyConstantBlock => "Vm.CopyConstantBlock",
            Self::VmCopyGeneBackwardSlice => "Vm.CopyGeneBackwardSlice",
            Self::VmCopyGeneForwardSlice => "Vm.CopyGeneForwardSlice",
            Self::VmInsertReadStoreMotif => "Vm.InsertReadStoreMotif",
            Self::VmInsertReadBidMotif => "Vm.InsertReadBidMotif",
            Self::VmInsertLoadCompareMotif => "Vm.InsertLoadCompareMotif",
            Self::VmMutateSlotAddress => "Vm.MutateSlotAddress",
            Self::VmMutatePairedSlotAddress => "Vm.MutatePairedSlotAddress",
            Self::GraphAlterGraphEdgeWeight => "Graph.AlterGraphEdgeWeight",
            Self::GraphSwapGraphOperator => "Graph.SwapGraphOperator",
            Self::GraphMutateGraphOperatorParam => "Graph.MutateGraphOperatorParam",
            Self::GraphMutateActionSlotBehavior => "Graph.MutateActionSlotBehavior",
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
            Self::GraphEnableRewardModulation => "Graph.EnableRewardModulation",
            Self::GraphDisableRewardModulation => "Graph.DisableRewardModulation",
            Self::GraphMutateRewardSource => "Graph.MutateRewardSource",
            Self::GraphMutateTraceDecay => "Graph.MutateTraceDecay",
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
            | Self::TopologyCopyNode
            | Self::TopologyCopyMeshBackwardSlice
            | Self::TopologyCopyMeshForwardSlice
            | Self::TopologySpliceNode
            | Self::TopologySwapRouteTargets
            | Self::TopologyMutateGateBias => MutationDomain::Topology,
            Self::VmConstantMutation
            | Self::VmInstructionMutation
            | Self::VmDeleteInstruction
            | Self::VmRegisterCountMutation
            | Self::VmInstructionRawFieldMutation
            | Self::VmCopyInstructionBlock
            | Self::VmCopyInstructionBlockRemapped
            | Self::VmCopyConstantBlock
            | Self::VmCopyGeneBackwardSlice
            | Self::VmCopyGeneForwardSlice
            | Self::VmInsertReadStoreMotif
            | Self::VmInsertReadBidMotif
            | Self::VmInsertLoadCompareMotif
            | Self::VmMutateSlotAddress
            | Self::VmMutatePairedSlotAddress => MutationDomain::Vm,
            Self::GraphAlterGraphEdgeWeight
            | Self::GraphSwapGraphOperator
            | Self::GraphMutateGraphOperatorParam
            | Self::GraphMutateActionSlotBehavior
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
            | Self::GraphToggleHebbianLamarckian
            | Self::GraphEnableRewardModulation
            | Self::GraphDisableRewardModulation
            | Self::GraphMutateRewardSource
            | Self::GraphMutateTraceDecay => MutationDomain::Graph,
            Self::InputRefAdd
            | Self::InputRefRemove
            | Self::InputRefSwap
            | Self::InputRefRawFieldMutation => MutationDomain::InputRef,
        }
    }

    #[must_use]
    pub const fn semantic_category(self) -> MutationSemanticCategory {
        MutationSemanticCategory::SemanticChange
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
            | Self::TopologySwapNodeBackend
            | Self::TopologyAddRouteTarget => ComplexityEffect::Increasing,
            // Topology: structural removals
            Self::TopologyRemoveNode | Self::TopologyRemoveRouteTarget => {
                ComplexityEffect::Decreasing
            }
            // Topology: rewiring / neutral
            Self::TopologyRetargetNodeTarget
            | Self::TopologyChangeEntryNode
            | Self::TopologySwapRouteTargets
            | Self::TopologyMutateGateBias => ComplexityEffect::Neutral,
            // VM: copy/motif-insert operators are increasing
            Self::VmCopyInstructionBlock
            | Self::VmCopyInstructionBlockRemapped
            | Self::VmCopyConstantBlock
            | Self::VmCopyGeneBackwardSlice
            | Self::VmCopyGeneForwardSlice
            | Self::VmInsertReadStoreMotif
            | Self::VmInsertReadBidMotif
            | Self::VmInsertLoadCompareMotif => ComplexityEffect::Increasing,
            Self::VmDeleteInstruction => ComplexityEffect::Decreasing,
            // VM: all others neutral (mutate existing content, no structural growth)
            Self::VmConstantMutation
            | Self::VmInstructionMutation
            | Self::VmRegisterCountMutation
            | Self::VmInstructionRawFieldMutation
            | Self::VmMutateSlotAddress
            | Self::VmMutatePairedSlotAddress => ComplexityEffect::Neutral,
            // Graph: structural additions
            Self::GraphAddInternalGraphNode
            | Self::GraphAddGraphEdge
            | Self::GraphCopyInternalNode
            | Self::GraphCopySubgraph
            | Self::GraphCopyEdgeBundle
            | Self::GraphEnableHebbian
            | Self::GraphEnableRewardModulation => ComplexityEffect::Increasing,
            // Graph: structural removals
            Self::GraphRemoveInternalGraphNode
            | Self::GraphRemoveGraphEdge
            | Self::GraphDisableHebbian
            | Self::GraphDisableRewardModulation => ComplexityEffect::Decreasing,
            // Graph: rewiring / neutral
            Self::GraphAlterGraphEdgeWeight
            | Self::GraphSwapGraphOperator
            | Self::GraphMutateGraphOperatorParam
            | Self::GraphMutateActionSlotBehavior
            | Self::GraphRetargetGraphEdge
            | Self::GraphRawFieldMutation
            | Self::GraphMutateHebbianRule
            | Self::GraphMutateHebbianRate
            | Self::GraphToggleHebbianLamarckian
            | Self::GraphMutateRewardSource
            | Self::GraphMutateTraceDecay => ComplexityEffect::Neutral,
            // InputRef: add / remove / neutral
            Self::InputRefAdd => ComplexityEffect::Increasing,
            Self::InputRefRemove => ComplexityEffect::Decreasing,
            Self::InputRefSwap | Self::InputRefRawFieldMutation => ComplexityEffect::Neutral,
        }
    }

    #[must_use]
    pub const fn all() -> [Self; 54] {
        [
            Self::TopologyAddNode,
            Self::TopologyRemoveNode,
            Self::TopologyRetargetNodeTarget,
            Self::TopologyAddRouteTarget,
            Self::TopologyRemoveRouteTarget,
            Self::TopologyChangeEntryNode,
            Self::TopologySwapNodeBackend,
            Self::TopologyCopyNode,
            Self::TopologyCopyMeshBackwardSlice,
            Self::TopologyCopyMeshForwardSlice,
            Self::TopologySpliceNode,
            Self::TopologySwapRouteTargets,
            Self::TopologyMutateGateBias,
            Self::VmConstantMutation,
            Self::VmInstructionMutation,
            Self::VmDeleteInstruction,
            Self::VmRegisterCountMutation,
            Self::VmInstructionRawFieldMutation,
            Self::VmCopyInstructionBlock,
            Self::VmCopyInstructionBlockRemapped,
            Self::VmCopyConstantBlock,
            Self::VmCopyGeneBackwardSlice,
            Self::VmCopyGeneForwardSlice,
            Self::VmInsertReadStoreMotif,
            Self::VmInsertReadBidMotif,
            Self::VmInsertLoadCompareMotif,
            Self::VmMutateSlotAddress,
            Self::VmMutatePairedSlotAddress,
            Self::GraphAlterGraphEdgeWeight,
            Self::GraphSwapGraphOperator,
            Self::GraphMutateGraphOperatorParam,
            Self::GraphMutateActionSlotBehavior,
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
            Self::GraphEnableRewardModulation,
            Self::GraphDisableRewardModulation,
            Self::GraphMutateRewardSource,
            Self::GraphMutateTraceDecay,
            Self::InputRefAdd,
            Self::InputRefRemove,
            Self::InputRefSwap,
            Self::InputRefRawFieldMutation,
        ]
    }
}

/// Whether a mutation target node is reachable from the mesh entry node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TargetReachability {
    Reachable,
    Unreachable,
    NotApplicable,
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

/// What one attempted mutation event did.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MutationEventOutcome {
    /// The event changed the genome; the classification is its first target's.
    Applied(TargetReachability),
    /// The event left the genome as it found it.
    Skipped(MutationSkipReason),
}

impl MutationEventOutcome {
    /// Whether this event changed the genome.
    #[must_use]
    pub const fn is_applied(self) -> bool {
        matches!(self, Self::Applied(_))
    }
}

/// One attempted mutation event, in draw order (T13.F01).
///
/// `operator` is `None` for a domain-exhausted event: every operator of the
/// domain reported `NoApplicableTarget`, so no single operator owns the
/// attempt (see [`MutationSummary::record_domain_skip`]).
///
/// `target` is the id the genome carried, before this event, for the first
/// node the event's target selector returned. It is `None` when nothing was
/// ever selected, which distinguishes *no eligible node of the required kind
/// exists* from *a node was selected but carried no applicable site*.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MutationEventRecord {
    pub domain: MutationDomain,
    pub operator: Option<MutationOperator>,
    pub target: Option<NodeId>,
    pub outcome: MutationEventOutcome,
}

/// Per-operator funnel counters for mutation-event staging.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MutationOperatorFunnel {
    pub attempted: u64,
    pub applicable: u64,
    pub structurally_valid: u64,
    pub applied: u64,
    pub semantic_change: u64,
    pub skipped: u64,
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
    pub skipped_by_operator: HashMap<MutationOperator, u32>,
    pub attempted_by_domain: HashMap<MutationDomain, u32>,
    pub applied_by_domain: HashMap<MutationDomain, u32>,
    pub attempted_by_operator: HashMap<MutationOperator, u32>,
    pub applied_by_operator: HashMap<MutationOperator, u32>,
    pub applied_semantic_noop_events: u32,
    pub applied_semantic_change_events: u32,
    pub operator_funnel_by_operator: HashMap<MutationOperator, MutationOperatorFunnel>,
    pub skip_reasons_by_operator: HashMap<MutationOperator, HashMap<MutationSkipReason, u32>>,
    pub added_node_input_classes_by_operator:
        HashMap<MutationOperator, HashMap<MutationAddedNodeInputClass, u32>>,
    pub added_node_world_inputs_by_operator: HashMap<MutationOperator, HashMap<WorldInputKey, u32>>,
    pub reachable_target_events: u32,
    pub unreachable_target_events: u32,
    /// Applied events whose target was a node the parent executed recently
    /// (T11.F17). Counted by membership, like the reachable classification,
    /// so short-circuited draws on an all-executed eligible set count too.
    pub executed_target_events: u32,
    pub not_applicable_events: u32,
    /// One record per attempted event, in draw order (T13.F01). Purely
    /// observational: recording consumes no RNG and changes no selection.
    pub events: Vec<MutationEventRecord>,
}

impl MutationSummary {
    /// Return a summary with all counts zero and an empty skip-reason map.
    pub fn zero() -> Self {
        Self {
            attempted_events: 0,
            applied_events: 0,
            skipped_events: 0,
            skip_reasons: HashMap::new(),
            skipped_by_operator: HashMap::new(),
            attempted_by_domain: HashMap::new(),
            applied_by_domain: HashMap::new(),
            attempted_by_operator: HashMap::new(),
            applied_by_operator: HashMap::new(),
            applied_semantic_noop_events: 0,
            applied_semantic_change_events: 0,
            operator_funnel_by_operator: HashMap::new(),
            skip_reasons_by_operator: HashMap::new(),
            added_node_input_classes_by_operator: HashMap::new(),
            added_node_world_inputs_by_operator: HashMap::new(),
            reachable_target_events: 0,
            unreachable_target_events: 0,
            executed_target_events: 0,
            not_applicable_events: 0,
            events: Vec::new(),
        }
    }

    /// Append one attempted event's record, keeping
    /// `events.len() == attempted_events`.
    pub fn record_event(&mut self, event: MutationEventRecord) {
        self.events.push(event);
    }

    pub fn record_attempt(&mut self, domain: MutationDomain, operator: MutationOperator) {
        self.attempted_events += 1;
        *self.attempted_by_domain.entry(domain).or_insert(0) += 1;
        *self.attempted_by_operator.entry(operator).or_insert(0) += 1;
        self.operator_funnel_by_operator
            .entry(operator)
            .or_default()
            .attempted += 1;
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
        let funnel = self
            .operator_funnel_by_operator
            .entry(operator)
            .or_default();
        funnel.applicable += 1;
        funnel.structurally_valid += 1;
        funnel.applied += 1;
        match semantic {
            MutationSemanticCategory::SemanticNoop => self.applied_semantic_noop_events += 1,
            MutationSemanticCategory::SemanticChange => {
                self.applied_semantic_change_events += 1;
                funnel.semantic_change += 1;
            }
        }
    }

    pub fn record_skipped(&mut self, operator: MutationOperator, reason: MutationSkipReason) {
        self.skipped_events += 1;
        *self.skip_reasons.entry(reason).or_insert(0) += 1;
        *self.skipped_by_operator.entry(operator).or_insert(0) += 1;
        *self
            .skip_reasons_by_operator
            .entry(operator)
            .or_default()
            .entry(reason)
            .or_insert(0) += 1;
        let funnel = self
            .operator_funnel_by_operator
            .entry(operator)
            .or_default();
        if !matches!(reason, MutationSkipReason::NoApplicableTarget) {
            funnel.applicable += 1;
        }
        funnel.skipped += 1;
    }

    pub fn record_added_node_input_classes(
        &mut self,
        operator: MutationOperator,
        classes: &[MutationAddedNodeInputClass],
    ) {
        let entry = self
            .added_node_input_classes_by_operator
            .entry(operator)
            .or_default();
        for class in classes {
            *entry.entry(*class).or_insert(0) += 1;
        }
    }

    pub fn record_added_node_world_inputs(
        &mut self,
        operator: MutationOperator,
        keys: &[WorldInputKey],
    ) {
        let entry = self
            .added_node_world_inputs_by_operator
            .entry(operator)
            .or_default();
        for key in keys {
            *entry.entry(*key).or_insert(0) += 1;
        }
    }

    /// Record a skip where no operator could be selected for a domain.
    ///
    /// Increments attempted_events, attempted_by_domain, and skipped_events
    /// without recording an operator-level attempt (since none was selected).
    pub fn record_domain_skip(&mut self, domain: MutationDomain, reason: MutationSkipReason) {
        self.attempted_events += 1;
        *self.attempted_by_domain.entry(domain).or_insert(0) += 1;
        self.skipped_events += 1;
        *self.skip_reasons.entry(reason).or_insert(0) += 1;
    }

    pub fn record_reachability(&mut self, target: TargetReachability) {
        match target {
            TargetReachability::Reachable => self.reachable_target_events += 1,
            TargetReachability::Unreachable => self.unreachable_target_events += 1,
            TargetReachability::NotApplicable => self.not_applicable_events += 1,
        }
    }

    /// Record how many of an applied event's targets were recently executed.
    pub fn record_executed_targets(&mut self, count: u32) {
        self.executed_target_events += count;
    }
}

#[cfg(test)]
mod tests;
