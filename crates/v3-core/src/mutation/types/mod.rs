use std::collections::HashMap;

use crate::contracts::votes::VoteSink;
use crate::contracts::{InputReference, NodeId, WorldInputKey};
use crate::genome::cgp::Scope;

/// Whether a mutation operator increases, decreases, or preserves genome complexity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComplexityEffect {
    Increasing,
    Decreasing,
    Neutral,
}

impl ComplexityEffect {
    /// Returns true if this effect is `Decreasing`.
    #[must_use]
    pub const fn is_decreasing(self) -> bool {
        matches!(self, Self::Decreasing)
    }
}

/// What one committed event did to bundles (T25.F11), reported by the
/// operator from the container's return values: the bundles it created by
/// wiring (`plain`, `ring`, `pair`) and creation form (`uniform`,
/// `randomized`), the members it detached and the records it removed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BundleEdits {
    pub created: [[u8; 2]; 3],
    pub detached: u8,
    pub removed: u8,
}

impl BundleEdits {
    /// One member detach, which removed its record when it was the last.
    #[must_use]
    pub const fn detach(removed: bool) -> Self {
        Self {
            created: [[0; 2]; 3],
            detached: 1,
            removed: removed as u8,
        }
    }

    /// A removal that took a record when `removed`.
    #[must_use]
    pub const fn removal(removed: bool) -> Self {
        Self {
            created: [[0; 2]; 3],
            detached: 0,
            removed: removed as u8,
        }
    }

    /// One bundle created with `wiring` in `form`.
    #[must_use]
    pub fn creation(wiring: &crate::genome::cgp::Wiring, randomized: bool) -> Self {
        let mut edits = Self::default();
        edits.created[wiring_index(wiring)][usize::from(randomized)] = 1;
        edits
    }
}

/// A wiring's row in [`BundleEdits::created`]: `plain`, `ring`, `pair`.
#[must_use]
pub const fn wiring_index(wiring: &crate::genome::cgp::Wiring) -> usize {
    use crate::genome::cgp::Wiring;
    match wiring {
        Wiring::Plain { .. } => 0,
        Wiring::Ring { .. } => 1,
        Wiring::DirectionPair { .. } => 2,
    }
}

/// Reason a mutation event was skipped.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum MutationSkipReason {
    ParseabilityViolation,
    NumericProposalRejected,
    NoApplicableTarget,
    /// The event left a Graph backend breaking a graph invariant
    /// (`validate_graph`, T25.F03); the draft was rolled back.
    GraphInvariantViolation,
}

impl MutationSkipReason {
    /// Stable string key used by transport/API boundaries.
    #[must_use]
    pub const fn as_key(self) -> &'static str {
        match self {
            Self::ParseabilityViolation => "ParseabilityViolation",
            Self::NumericProposalRejected => "NumericProposalRejected",
            Self::NoApplicableTarget => "NoApplicableTarget",
            Self::GraphInvariantViolation => "GraphInvariantViolation",
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

/// The class of the vote sink a born edge lands on (T11.F30): a kind's
/// `Drive` or one of its direction sinks. `AddGraphEdge` and `InputRef.Add`
/// report it at the push; the `drive_births` reading keys carrier outcomes
/// by it.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum BirthEdgeClass {
    Drive,
    Direction,
}

impl BirthEdgeClass {
    pub const ALL: [Self; 2] = [Self::Drive, Self::Direction];

    #[must_use]
    pub const fn as_key(self) -> &'static str {
        match self {
            Self::Drive => "drive",
            Self::Direction => "direction",
        }
    }

    /// The class of `sink`: `Drive` for a drive, `Direction` for a directed
    /// kind's direction sink, `None` for `Eat`, `Terminate` and `Decide`.
    #[must_use]
    pub const fn of_sink(sink: VoteSink) -> Option<Self> {
        match sink {
            VoteSink::Drive(_) => Some(Self::Drive),
            VoteSink::Directed(..) => Some(Self::Direction),
            VoteSink::Eat | VoteSink::Terminate | VoteSink::Decide => None,
        }
    }
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
    Decision,
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
            Self::Decision => "decision",
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
                WorldInputKey::NearbyCreature => Self::Neighbor,
            },
            InputReference::StaticIntrospection(_)
            | InputReference::DynamicIntrospection(_)
            | InputReference::PreviousOutcome => Self::Introspection,
            InputReference::ActionVotes
            | InputReference::PreviousPassVotes
            | InputReference::CommitCounts => Self::Decision,
            InputReference::UpstreamSlot(_) | InputReference::UpstreamBus => Self::Upstream,
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
    TopologyAddFeedbackRoute,
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
    GraphAddInternalGraphNode,
    GraphRemoveInternalGraphNode,
    GraphAddGraphEdge,
    GraphRetargetGraphEdge,
    GraphRemoveGraphEdge,
    GraphRawFieldMutation,
    GraphCopyInternalNode,
    GraphCopySubgraph,
    GraphCopyNodeInputs,
    GraphEnableHebbian,
    GraphDisableHebbian,
    GraphMutateHebbianRule,
    GraphMutateHebbianRate,
    GraphToggleHebbianLamarckian,
    GraphEnableRewardModulation,
    GraphDisableRewardModulation,
    GraphMutateRewardSource,
    GraphMutateTraceDecay,
    GraphRecruitNeutralInput,
    GraphAddRingBundle,
    GraphAddDirectionPairBundle,
    GraphAlterBundleScale,
    GraphShiftBundleOffset,
    GraphRetargetBundleKind,
    GraphRemoveBundle,
    GraphSetScope,
    GraphAddRandomizedRingBundle,
    GraphAddRandomizedDirectionPairBundle,
    GraphDetachGraphEdge,
    GraphEnableRetention,
    GraphDisableRetention,
    GraphMutateRetention,
    GraphEnableBundleRetention,
    GraphDisableBundleRetention,
    GraphMutateBundleRetention,
    // InputRef
    InputRefAdd,
    InputRefPrune,
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
            Self::TopologyAddFeedbackRoute => "Topology.AddFeedbackRoute",
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
            Self::GraphAddInternalGraphNode => "Graph.AddInternalGraphNode",
            Self::GraphRemoveInternalGraphNode => "Graph.RemoveInternalGraphNode",
            Self::GraphAddGraphEdge => "Graph.AddGraphEdge",
            Self::GraphRetargetGraphEdge => "Graph.RetargetGraphEdge",
            Self::GraphRemoveGraphEdge => "Graph.RemoveGraphEdge",
            Self::GraphRawFieldMutation => "Graph.GraphRawFieldMutation",
            Self::GraphCopyInternalNode => "Graph.CopyInternalNode",
            Self::GraphCopySubgraph => "Graph.CopySubgraph",
            Self::GraphCopyNodeInputs => "Graph.CopyNodeInputs",
            Self::GraphEnableHebbian => "Graph.EnableHebbian",
            Self::GraphDisableHebbian => "Graph.DisableHebbian",
            Self::GraphMutateHebbianRule => "Graph.MutateHebbianRule",
            Self::GraphMutateHebbianRate => "Graph.MutateHebbianRate",
            Self::GraphToggleHebbianLamarckian => "Graph.ToggleHebbianLamarckian",
            Self::GraphEnableRewardModulation => "Graph.EnableRewardModulation",
            Self::GraphDisableRewardModulation => "Graph.DisableRewardModulation",
            Self::GraphMutateRewardSource => "Graph.MutateRewardSource",
            Self::GraphMutateTraceDecay => "Graph.MutateTraceDecay",
            Self::GraphRecruitNeutralInput => "Graph.RecruitNeutralInput",
            Self::GraphAddRingBundle => "Graph.AddRingBundle",
            Self::GraphAddDirectionPairBundle => "Graph.AddDirectionPairBundle",
            Self::GraphAlterBundleScale => "Graph.AlterBundleScale",
            Self::GraphShiftBundleOffset => "Graph.ShiftBundleOffset",
            Self::GraphRetargetBundleKind => "Graph.RetargetBundleKind",
            Self::GraphRemoveBundle => "Graph.RemoveBundle",
            Self::GraphSetScope => "Graph.SetScope",
            Self::GraphAddRandomizedRingBundle => "Graph.AddRandomizedRingBundle",
            Self::GraphAddRandomizedDirectionPairBundle => "Graph.AddRandomizedDirectionPairBundle",
            Self::GraphDetachGraphEdge => "Graph.DetachGraphEdge",
            Self::GraphEnableRetention => "Graph.EnableRetention",
            Self::GraphDisableRetention => "Graph.DisableRetention",
            Self::GraphMutateRetention => "Graph.MutateRetention",
            Self::GraphEnableBundleRetention => "Graph.EnableBundleRetention",
            Self::GraphDisableBundleRetention => "Graph.DisableBundleRetention",
            Self::GraphMutateBundleRetention => "Graph.MutateBundleRetention",
            Self::InputRefAdd => "InputRef.Add",
            Self::InputRefPrune => "InputRef.Prune",
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
            | Self::TopologyAddFeedbackRoute
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
            | Self::GraphAddInternalGraphNode
            | Self::GraphRemoveInternalGraphNode
            | Self::GraphAddGraphEdge
            | Self::GraphRetargetGraphEdge
            | Self::GraphRemoveGraphEdge
            | Self::GraphRawFieldMutation
            | Self::GraphCopyInternalNode
            | Self::GraphCopySubgraph
            | Self::GraphCopyNodeInputs
            | Self::GraphEnableHebbian
            | Self::GraphDisableHebbian
            | Self::GraphMutateHebbianRule
            | Self::GraphMutateHebbianRate
            | Self::GraphToggleHebbianLamarckian
            | Self::GraphEnableRewardModulation
            | Self::GraphDisableRewardModulation
            | Self::GraphMutateRewardSource
            | Self::GraphMutateTraceDecay
            | Self::GraphRecruitNeutralInput
            | Self::GraphAddRingBundle
            | Self::GraphAddDirectionPairBundle
            | Self::GraphAlterBundleScale
            | Self::GraphShiftBundleOffset
            | Self::GraphRetargetBundleKind
            | Self::GraphRemoveBundle
            | Self::GraphSetScope
            | Self::GraphAddRandomizedRingBundle
            | Self::GraphAddRandomizedDirectionPairBundle
            | Self::GraphDetachGraphEdge
            | Self::GraphEnableRetention
            | Self::GraphDisableRetention
            | Self::GraphMutateRetention
            | Self::GraphEnableBundleRetention
            | Self::GraphDisableBundleRetention
            | Self::GraphMutateBundleRetention => MutationDomain::Graph,
            Self::InputRefAdd
            | Self::InputRefPrune
            | Self::InputRefSwap
            | Self::InputRefRawFieldMutation => MutationDomain::InputRef,
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
            | Self::TopologySwapNodeBackend
            | Self::TopologyAddRouteTarget
            | Self::TopologyAddFeedbackRoute => ComplexityEffect::Increasing,
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
            | Self::GraphCopyNodeInputs
            | Self::GraphEnableHebbian
            | Self::GraphEnableRewardModulation
            | Self::GraphRecruitNeutralInput
            | Self::GraphAddRingBundle
            | Self::GraphAddDirectionPairBundle
            | Self::GraphAddRandomizedRingBundle
            | Self::GraphAddRandomizedDirectionPairBundle => ComplexityEffect::Increasing,
            // Graph: structural removals
            Self::GraphRemoveInternalGraphNode
            | Self::GraphRemoveGraphEdge
            | Self::GraphDisableHebbian
            | Self::GraphDisableRewardModulation
            | Self::GraphRemoveBundle => ComplexityEffect::Decreasing,
            // Graph: rewiring / neutral
            Self::GraphAlterGraphEdgeWeight
            | Self::GraphSwapGraphOperator
            | Self::GraphMutateGraphOperatorParam
            | Self::GraphRetargetGraphEdge
            | Self::GraphRawFieldMutation
            | Self::GraphMutateHebbianRule
            | Self::GraphMutateHebbianRate
            | Self::GraphToggleHebbianLamarckian
            | Self::GraphMutateRewardSource
            | Self::GraphMutateTraceDecay
            | Self::GraphAlterBundleScale
            | Self::GraphShiftBundleOffset
            | Self::GraphRetargetBundleKind
            | Self::GraphSetScope
            | Self::GraphDetachGraphEdge
            | Self::GraphEnableRetention
            | Self::GraphDisableRetention
            | Self::GraphMutateRetention
            | Self::GraphEnableBundleRetention
            | Self::GraphDisableBundleRetention
            | Self::GraphMutateBundleRetention => ComplexityEffect::Neutral,
            // InputRef: add / remove / neutral
            Self::InputRefAdd => ComplexityEffect::Increasing,
            Self::InputRefPrune => ComplexityEffect::Decreasing,
            Self::InputRefSwap | Self::InputRefRawFieldMutation => ComplexityEffect::Neutral,
        }
    }

    #[must_use]
    pub const fn all() -> [Self; 71] {
        [
            Self::TopologyAddNode,
            Self::TopologyRemoveNode,
            Self::TopologyRetargetNodeTarget,
            Self::TopologyAddRouteTarget,
            Self::TopologyAddFeedbackRoute,
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
            Self::GraphAddInternalGraphNode,
            Self::GraphRemoveInternalGraphNode,
            Self::GraphAddGraphEdge,
            Self::GraphRetargetGraphEdge,
            Self::GraphRemoveGraphEdge,
            Self::GraphRawFieldMutation,
            Self::GraphCopyInternalNode,
            Self::GraphCopySubgraph,
            Self::GraphCopyNodeInputs,
            Self::GraphEnableHebbian,
            Self::GraphDisableHebbian,
            Self::GraphMutateHebbianRule,
            Self::GraphMutateHebbianRate,
            Self::GraphToggleHebbianLamarckian,
            Self::GraphEnableRewardModulation,
            Self::GraphDisableRewardModulation,
            Self::GraphMutateRewardSource,
            Self::GraphMutateTraceDecay,
            Self::GraphRecruitNeutralInput,
            Self::GraphAddRingBundle,
            Self::GraphAddDirectionPairBundle,
            Self::GraphAlterBundleScale,
            Self::GraphShiftBundleOffset,
            Self::GraphRetargetBundleKind,
            Self::GraphRemoveBundle,
            Self::GraphSetScope,
            Self::GraphAddRandomizedRingBundle,
            Self::GraphAddRandomizedDirectionPairBundle,
            Self::GraphDetachGraphEdge,
            Self::GraphEnableRetention,
            Self::GraphDisableRetention,
            Self::GraphMutateRetention,
            Self::GraphEnableBundleRetention,
            Self::GraphDisableBundleRetention,
            Self::GraphMutateBundleRetention,
            Self::InputRefAdd,
            Self::InputRefPrune,
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
/// `discarded` lists, in draw order, every operator the event tried and threw
/// away because it reported `NoApplicableTarget`, with the id of the first node
/// that operator selected (`None` when it selected nothing). A discard naming a
/// node is the *selected but no applicable site* fact, and it stays visible
/// when a later operator of the same domain applied. For a domain-exhausted
/// event `discarded` holds every operator of the domain and `target` is the
/// first node any of them selected.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MutationEventRecord {
    pub domain: MutationDomain,
    pub operator: Option<MutationOperator>,
    pub target: Option<NodeId>,
    pub outcome: MutationEventOutcome,
    /// Empty for an event whose first operator was accepted; an empty `Vec`
    /// allocates nothing.
    pub discarded: Vec<(MutationOperator, Option<NodeId>)>,
}

/// Per-operator funnel counters for mutation-event staging.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MutationOperatorFunnel {
    pub attempted: u64,
    pub applicable: u64,
    pub structurally_valid: u64,
    pub applied: u64,
    pub skipped: u64,
}

/// Summary returned by `MutationEngine` for every offspring.
///
/// Accounting invariant: `attempted_events == applied_events + skipped_events`.
#[derive(Debug, Clone, PartialEq)]
pub struct MutationSummary {
    /// Applied native event snapshots for the bounded engineering test panel.
    /// Absent from production builds and transports; recording draws no RNG.
    #[cfg(test)]
    pub(crate) recruitment_deltas:
        Vec<(crate::genome::CreatureGenome, crate::genome::CreatureGenome)>,
    pub attempted_events: u32,
    pub applied_events: u32,
    pub skipped_events: u32,
    pub skip_reasons: HashMap<MutationSkipReason, u32>,
    pub skipped_by_operator: HashMap<MutationOperator, u32>,
    pub attempted_by_domain: HashMap<MutationDomain, u32>,
    pub applied_by_domain: HashMap<MutationDomain, u32>,
    pub attempted_by_operator: HashMap<MutationOperator, u32>,
    pub applied_by_operator: HashMap<MutationOperator, u32>,
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
    /// The class of each vote-sink edge an applied `AddGraphEdge` or
    /// `InputRef.Add` pushed, in event order (T11.F30).
    pub vote_edge_classes: Vec<BirthEdgeClass>,
    /// Bundle members committed events detached (T25.F03): retarget,
    /// raw-field step or split of a member.
    pub graph_edges_detached: u32,
    /// Edge bundles committed events removed (T25.F03): `RemoveBundle`, and a
    /// last member's removal or detach.
    pub bundles_removed: u32,
    /// Committed attribute changes (the nine plasticity operators and
    /// `SetScope`) by the scope they landed on, indexed by
    /// [`Scope::index`] (T25.F08).
    pub attribute_changes_by_scope: [u32; 2],
}

impl MutationSummary {
    /// Return a summary with all counts zero and an empty skip-reason map.
    pub fn zero() -> Self {
        Self {
            #[cfg(test)]
            recruitment_deltas: Vec::new(),
            attempted_events: 0,
            applied_events: 0,
            skipped_events: 0,
            skip_reasons: HashMap::new(),
            skipped_by_operator: HashMap::new(),
            attempted_by_domain: HashMap::new(),
            applied_by_domain: HashMap::new(),
            attempted_by_operator: HashMap::new(),
            applied_by_operator: HashMap::new(),
            operator_funnel_by_operator: HashMap::new(),
            skip_reasons_by_operator: HashMap::new(),
            added_node_input_classes_by_operator: HashMap::new(),
            added_node_world_inputs_by_operator: HashMap::new(),
            reachable_target_events: 0,
            unreachable_target_events: 0,
            executed_target_events: 0,
            not_applicable_events: 0,
            events: Vec::new(),
            vote_edge_classes: Vec::new(),
            graph_edges_detached: 0,
            bundles_removed: 0,
            attribute_changes_by_scope: [0; 2],
        }
    }

    /// Record the detaches and bundle removals of a committed event.
    pub fn record_bundle_edits(&mut self, edits: BundleEdits) {
        self.graph_edges_detached += u32::from(edits.detached);
        self.bundles_removed += u32::from(edits.removed);
    }

    /// Record one committed attribute change at the scope it landed on.
    pub fn record_attribute_scope(&mut self, scope: Scope) {
        self.attribute_changes_by_scope[scope.index()] += 1;
    }

    /// Record the class of a vote-sink edge an applied event pushed.
    pub fn record_vote_edge_class(&mut self, class: BirthEdgeClass) {
        self.vote_edge_classes.push(class);
    }

    /// The birth's edge class (T11.F30): defined only when the birth applied
    /// exactly one event, that event was one `AddGraphEdge` or one
    /// `InputRef.Add`, and it pushed an edge onto a drive or direction sink.
    #[must_use]
    pub fn birth_edge_class(&self) -> Option<BirthEdgeClass> {
        let single = self.applied_events == 1
            && [
                MutationOperator::GraphAddGraphEdge,
                MutationOperator::InputRefAdd,
            ]
            .iter()
            .any(|operator| self.applied_by_operator.get(operator) == Some(&1));
        match self.vote_edge_classes.as_slice() {
            [class] if single => Some(*class),
            _ => None,
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

    pub fn record_applied(&mut self, domain: MutationDomain, operator: MutationOperator) {
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
