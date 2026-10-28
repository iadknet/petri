use std::collections::HashMap;
use std::fmt;

use crate::config::{MutationConfig, ReachableBiasConfig};
use crate::contracts::votes::VoteSink;
use crate::contracts::{InputReference, NodeId, WorldInputKey};
use crate::genome::cgp::Scope;
use crate::mutation::graph::GraphOperator;
use crate::mutation::input_ref::InputRefOperator;
use crate::mutation::topology::TopologyOperator;
use crate::mutation::vm::VmOperator;

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

    /// One bundle created with `wiring` in `form` (a test expectation).
    #[cfg(test)]
    #[must_use]
    pub(crate) fn creation(wiring: &crate::genome::cgp::Wiring, randomized: bool) -> Self {
        let mut edits = Self::default();
        edits.created[wiring_index(wiring)][usize::from(randomized)] = 1;
        edits
    }
}

/// The report keys of [`BundleEdits::created`]'s rows, by [`wiring_index`].
pub const WIRING_KEYS: [&str; 3] = ["plain", "ring", "pair"];

/// The report keys of [`BundleEdits::created`]'s columns: the creation
/// form, uniform (one learning draw for the bundle) or randomized (one per
/// member).
pub const FORM_KEYS: [&str; 2] = ["uniform", "randomized"];

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

    /// This domain's operators: its slice of [`MutationOperator::ALL`], in
    /// the domain enum's `ALL` order.
    #[must_use]
    pub fn catalog(self) -> &'static [MutationOperator] {
        const TOPOLOGY: usize = TopologyOperator::ALL.len();
        const VM: usize = TOPOLOGY + VmOperator::ALL.len();
        const GRAPH: usize = VM + GraphOperator::ALL.len();
        let all = &MutationOperator::ALL;
        match self {
            Self::Topology => &all[..TOPOLOGY],
            Self::Vm => &all[TOPOLOGY..VM],
            Self::Graph => &all[VM..GRAPH],
            Self::InputRef => &all[GRAPH..],
        }
    }

    /// The domain's production reachable bias.
    #[must_use]
    pub const fn reachable_bias(self, config: &ReachableBiasConfig) -> f64 {
        match self {
            Self::Topology => config.topology,
            Self::Vm => config.vm,
            Self::Graph => config.graph,
            Self::InputRef => config.input_ref,
        }
    }
}

/// Mutation operator selected for one attempted mutation event: one of the
/// four domain enums' operators (T30.F10). The domain enum owns every fact
/// about its operators (key, weight, complexity effect); this enum derives
/// them from the variant. Its derived order is today's flat order: the
/// domains in variant order, then each domain enum's declaration order.
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(into = "&'static str", try_from = "String")]
pub enum MutationOperator {
    Topology(TopologyOperator),
    Vm(VmOperator),
    Graph(GraphOperator),
    InputRef(InputRefOperator),
}

/// Concatenates the four domain `ALL` lists in domain order.
const fn catalog_of() -> [MutationOperator; 69] {
    let mut all = [MutationOperator::Topology(TopologyOperator::AddNode); 69];
    let mut i = 0;
    let mut j = 0;
    while j < TopologyOperator::ALL.len() {
        all[i] = MutationOperator::Topology(TopologyOperator::ALL[j]);
        i += 1;
        j += 1;
    }
    j = 0;
    while j < VmOperator::ALL.len() {
        all[i] = MutationOperator::Vm(VmOperator::ALL[j]);
        i += 1;
        j += 1;
    }
    j = 0;
    while j < GraphOperator::ALL.len() {
        all[i] = MutationOperator::Graph(GraphOperator::ALL[j]);
        i += 1;
        j += 1;
    }
    j = 0;
    while j < InputRefOperator::ALL.len() {
        all[i] = MutationOperator::InputRef(InputRefOperator::ALL[j]);
        i += 1;
        j += 1;
    }
    assert!(i == 69, "ALL holds every domain operator once");
    all
}

impl MutationOperator {
    /// Every operator, the four domain `ALL` lists in domain order.
    pub const ALL: [Self; 69] = catalog_of();

    /// The domain's own operator, as its `Debug` names it.
    fn inner(&self) -> &dyn fmt::Debug {
        match self {
            Self::Topology(op) => op,
            Self::Vm(op) => op,
            Self::Graph(op) => op,
            Self::InputRef(op) => op,
        }
    }

    /// The domain enum's own `Debug` name (`AddNode`, `VmConstantMutation`).
    #[must_use]
    pub fn inner_name(self) -> String {
        format!("{:?}", self.inner())
    }

    #[must_use]
    pub const fn as_key(self) -> &'static str {
        match self {
            Self::Topology(op) => op.as_key(),
            Self::Vm(op) => op.as_key(),
            Self::Graph(op) => op.as_key(),
            Self::InputRef(op) => op.as_key(),
        }
    }

    #[must_use]
    pub const fn domain(self) -> MutationDomain {
        match self {
            Self::Topology(_) => MutationDomain::Topology,
            Self::Vm(_) => MutationDomain::Vm,
            Self::Graph(_) => MutationDomain::Graph,
            Self::InputRef(_) => MutationDomain::InputRef,
        }
    }

    #[must_use]
    pub const fn complexity_effect(self) -> ComplexityEffect {
        match self {
            Self::Topology(op) => op.complexity_effect(),
            Self::Vm(op) => op.complexity_effect(),
            Self::Graph(op) => op.complexity_effect(),
            Self::InputRef(op) => op.complexity_effect(),
        }
    }

    /// The operator's selection weight within its domain: Topology's in
    /// hundredths through the large-copy tuning, the other three their
    /// own tier weights, so each domain's total keeps its units.
    #[must_use]
    pub fn weight(self, config: &MutationConfig) -> u16 {
        match self {
            Self::Topology(op) => op.tuned_weight(config.large_copy_weight_percent),
            Self::Vm(op) => u16::from(op.weight()),
            Self::Graph(op) => u16::from(op.weight()),
            Self::InputRef(op) => u16::from(op.weight()),
        }
    }
}

/// Today's flat name: the domain, then the inner variant's name, the domain
/// omitted when the inner name already starts with it (`TopologyAddNode`,
/// `VmConstantMutation`, `GraphRawFieldMutation`). The draw-stream pin
/// hashes this text.
impl fmt::Debug for MutationOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let domain = self.domain().as_key();
        let inner = self.inner_name();
        if !inner.starts_with(domain) {
            f.write_str(domain)?;
        }
        f.write_str(&inner)
    }
}

impl From<MutationOperator> for &'static str {
    fn from(operator: MutationOperator) -> Self {
        operator.as_key()
    }
}

impl TryFrom<String> for MutationOperator {
    type Error = String;

    fn try_from(key: String) -> Result<Self, Self::Error> {
        Self::ALL
            .into_iter()
            .find(|operator| operator.as_key() == key)
            .ok_or_else(|| format!("unknown mutation operator key {key:?}"))
    }
}

impl From<TopologyOperator> for MutationOperator {
    fn from(op: TopologyOperator) -> Self {
        Self::Topology(op)
    }
}

impl From<VmOperator> for MutationOperator {
    fn from(op: VmOperator) -> Self {
        Self::Vm(op)
    }
}

impl From<GraphOperator> for MutationOperator {
    fn from(op: GraphOperator) -> Self {
        Self::Graph(op)
    }
}

impl From<InputRefOperator> for MutationOperator {
    fn from(op: InputRefOperator) -> Self {
        Self::InputRef(op)
    }
}

/// What one committed event reports beside its reachability (T30.F10): the
/// class of the vote-sink edge an `AddGraphEdge` or a creation pushed
/// (T11.F30, T25.F11), the scope an attribute operator's change landed on
/// (T25.F08, T25.F09) and what the event did to bundles (T25.F11).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EventReport {
    /// The class of the vote-sink edge the event pushed.
    pub class: Option<BirthEdgeClass>,
    /// The scope the change of one of the thirteen attribute operators (the
    /// nine plasticity operators, `SetScope` and the three edge retention
    /// operators) landed on.
    pub attribute_scope: Option<Scope>,
    /// What the event did to bundles, from the container's return values.
    pub bundle_edits: BundleEdits,
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
    /// The class of the vote sink each applied `AddGraphEdge` pushed onto,
    /// or each creation's one `Plain` bundle wires, in event order (T11.F30,
    /// T25.F11).
    pub vote_edge_classes: Vec<BirthEdgeClass>,
    /// Bundle members committed events detached (T25.F03): retarget,
    /// raw-field step or split of a member.
    pub graph_edges_detached: u32,
    /// Edge bundles committed events removed (T25.F03): `RemoveBundle`, and a
    /// last member's removal or detach.
    pub bundles_removed: u32,
    /// Bundles committed creation events made (T25.F11), by operator, then
    /// wiring (`plain`, `ring`, `pair`) and creation form (`uniform`,
    /// `randomized`), so a wiring is attributed to the operator that built
    /// it when one event builds several.
    pub bundles_created_by_operator: HashMap<MutationOperator, [[u32; 2]; 3]>,
    /// Committed attribute changes (the nine plasticity operators and
    /// `SetScope`) by the scope they landed on, indexed by
    /// [`Scope::index`] (T25.F08).
    pub attribute_changes_by_scope: [u32; 2],
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
            bundles_created_by_operator: HashMap::new(),
            attribute_changes_by_scope: [0; 2],
        }
    }

    /// Record the creations, detaches and bundle removals of a committed
    /// event of `operator`.
    pub fn record_bundle_edits(&mut self, operator: MutationOperator, edits: BundleEdits) {
        if edits.created != [[0; 2]; 3] {
            let created = self
                .bundles_created_by_operator
                .entry(operator)
                .or_default();
            for (row, add) in created.iter_mut().zip(edits.created) {
                for (count, add) in row.iter_mut().zip(add) {
                    *count += u32::from(add);
                }
            }
        }
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
    /// exactly one event, that event was one `AddGraphEdge` or one of the
    /// four creation operators, and it pushed one edge or made one `Plain`
    /// bundle on a drive or direction sink (T25.F11 invariant 8).
    #[must_use]
    pub fn birth_edge_class(&self) -> Option<BirthEdgeClass> {
        let single = self.applied_events == 1
            && [
                MutationOperator::Graph(GraphOperator::AddGraphEdge),
                MutationOperator::Graph(GraphOperator::AddBundleLive),
                MutationOperator::Graph(GraphOperator::AddBundleSilent),
                MutationOperator::InputRef(InputRefOperator::AddSensorLive),
                MutationOperator::InputRef(InputRefOperator::AddSensorSilent),
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
