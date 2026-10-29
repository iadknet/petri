use std::collections::HashMap;
use std::fmt;

use crate::config::{MutationConfig, ReachableBiasConfig};
use crate::contracts::{InputReference, NodeId, WorldInputKey};
use crate::mutation::body::BodyOperator;
use crate::mutation::graph::GraphOperator;
use crate::mutation::input_ref::InputRefOperator;
use crate::mutation::topology::TopologyOperator;
use crate::mutation::vm::VmOperator;

pub use crate::mutation::event::{
    wiring_index, BirthEdgeClass, Broadcast, BundleEdits, ComplexityEffect, EventReport,
    MutationSkipReason, TargetReachability, FORM_KEYS, WIRING_KEYS,
};

/// Domain of a mutation event: the four brain domains and the body
/// chromosome (T29.F02).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum MutationDomain {
    Topology,
    Vm,
    Graph,
    InputRef,
    Body,
}

/// A brain domain: what the two-layer dispatch draws for a brain event and
/// what carries the facts only the brain has (a target selector's reachable
/// bias, operator weights).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BrainDomain {
    Topology,
    Vm,
    Graph,
    InputRef,
}

impl BrainDomain {
    /// The four brain domains, in domain order.
    pub const ALL: [Self; 4] = [Self::Topology, Self::Vm, Self::Graph, Self::InputRef];

    /// This domain's operators: its slice of [`BrainOperator::ALL`].
    #[must_use]
    pub fn catalog(self) -> &'static [BrainOperator] {
        const TOPOLOGY: usize = TopologyOperator::ALL.len();
        const VM: usize = TOPOLOGY + VmOperator::ALL.len();
        const GRAPH: usize = VM + GraphOperator::ALL.len();
        let all = &BrainOperator::ALL;
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

impl From<BrainDomain> for MutationDomain {
    fn from(domain: BrainDomain) -> Self {
        match domain {
            BrainDomain::Topology => Self::Topology,
            BrainDomain::Vm => Self::Vm,
            BrainDomain::Graph => Self::Graph,
            BrainDomain::InputRef => Self::InputRef,
        }
    }
}

/// A brain operator: what a birth's brain events select by weight and
/// dispatch to a target (T29.F02 invariant 4). A body event draws its gene by
/// no weight and never dispatches, so the brain path holds this type and
/// widens it into a [`MutationOperator`] only to report.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BrainOperator {
    Topology(TopologyOperator),
    Vm(VmOperator),
    Graph(GraphOperator),
    InputRef(InputRefOperator),
}

impl BrainOperator {
    /// How many operators the four brain domain enums declare together.
    pub const COUNT: usize = TopologyOperator::ALL.len()
        + VmOperator::ALL.len()
        + GraphOperator::ALL.len()
        + InputRefOperator::ALL.len();

    /// Every brain operator, the four brain domain `ALL` lists in domain order.
    pub const ALL: [Self; Self::COUNT] = brain_catalog();

    /// This operator as a [`MutationOperator`]; `From` in const context.
    #[must_use]
    pub const fn widen(self) -> MutationOperator {
        match self {
            Self::Topology(op) => MutationOperator::Topology(op),
            Self::Vm(op) => MutationOperator::Vm(op),
            Self::Graph(op) => MutationOperator::Graph(op),
            Self::InputRef(op) => MutationOperator::InputRef(op),
        }
    }

    /// The operator's selection weight within its brain domain: Topology's
    /// in hundredths through the large-copy tuning, the other three their
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

    #[must_use]
    pub const fn complexity_effect(self) -> ComplexityEffect {
        self.widen().complexity_effect()
    }
}

/// Concatenates the four brain domain `ALL` lists in domain order.
const fn brain_catalog() -> [BrainOperator; BrainOperator::COUNT] {
    let mut all = [BrainOperator::Topology(TopologyOperator::AddNode); BrainOperator::COUNT];
    let mut i = 0;
    let mut j = 0;
    while j < TopologyOperator::ALL.len() {
        all[i] = BrainOperator::Topology(TopologyOperator::ALL[j]);
        i += 1;
        j += 1;
    }
    j = 0;
    while j < VmOperator::ALL.len() {
        all[i] = BrainOperator::Vm(VmOperator::ALL[j]);
        i += 1;
        j += 1;
    }
    j = 0;
    while j < GraphOperator::ALL.len() {
        all[i] = BrainOperator::Graph(GraphOperator::ALL[j]);
        i += 1;
        j += 1;
    }
    j = 0;
    while j < InputRefOperator::ALL.len() {
        all[i] = BrainOperator::InputRef(InputRefOperator::ALL[j]);
        i += 1;
        j += 1;
    }
    assert!(
        i == BrainOperator::COUNT,
        "ALL holds every brain operator once"
    );
    all
}

impl From<BrainOperator> for MutationOperator {
    fn from(op: BrainOperator) -> Self {
        op.widen()
    }
}

impl From<TopologyOperator> for BrainOperator {
    fn from(op: TopologyOperator) -> Self {
        Self::Topology(op)
    }
}

impl From<VmOperator> for BrainOperator {
    fn from(op: VmOperator) -> Self {
        Self::Vm(op)
    }
}

impl From<GraphOperator> for BrainOperator {
    fn from(op: GraphOperator) -> Self {
        Self::Graph(op)
    }
}

impl From<InputRefOperator> for BrainOperator {
    fn from(op: InputRefOperator) -> Self {
        Self::InputRef(op)
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
            Self::Body => "Body",
        }
    }

    #[must_use]
    pub const fn all() -> [Self; 5] {
        [
            Self::Topology,
            Self::Vm,
            Self::Graph,
            Self::InputRef,
            Self::Body,
        ]
    }
}

/// Mutation operator selected for one attempted mutation event: one of the
/// four brain domain enums' operators (T30.F10) or a body gene's (T29.F02). The domain enum owns every fact
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
    Body(BodyOperator),
}

/// [`BrainOperator::ALL`], then the body's operators.
const fn catalog_of() -> [MutationOperator; MutationOperator::COUNT] {
    let mut all = [MutationOperator::Topology(TopologyOperator::AddNode); MutationOperator::COUNT];
    let mut i = 0;
    while i < BrainOperator::COUNT {
        all[i] = BrainOperator::ALL[i].widen();
        i += 1;
    }
    let mut j = 0;
    while j < BodyOperator::ALL.len() {
        all[i] = MutationOperator::Body(BodyOperator::ALL[j]);
        i += 1;
        j += 1;
    }
    all
}

impl MutationOperator {
    /// How many operators the five domain enums declare together.
    pub const COUNT: usize = BrainOperator::COUNT + BodyOperator::ALL.len();

    /// Every operator, the five domain `ALL` lists in domain order: the
    /// brain's [`BrainOperator::ALL`], then the body's.
    pub const ALL: [Self; Self::COUNT] = catalog_of();

    /// The domain's own operator, as its `Debug` names it.
    fn inner(&self) -> &dyn fmt::Debug {
        match self {
            Self::Topology(op) => op,
            Self::Vm(op) => op,
            Self::Graph(op) => op,
            Self::InputRef(op) => op,
            Self::Body(op) => op,
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
            Self::Body(op) => op.as_key(),
        }
    }

    #[must_use]
    pub const fn domain(self) -> MutationDomain {
        match self {
            Self::Topology(_) => MutationDomain::Topology,
            Self::Vm(_) => MutationDomain::Vm,
            Self::Graph(_) => MutationDomain::Graph,
            Self::InputRef(_) => MutationDomain::InputRef,
            Self::Body(_) => MutationDomain::Body,
        }
    }

    #[must_use]
    pub const fn complexity_effect(self) -> ComplexityEffect {
        match self {
            Self::Topology(op) => op.complexity_effect(),
            Self::Vm(op) => op.complexity_effect(),
            Self::Graph(op) => op.complexity_effect(),
            Self::InputRef(op) => op.complexity_effect(),
            Self::Body(op) => op.complexity_effect(),
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

/// The flat name of the [`MutationOperator`] it widens to.
impl fmt::Debug for BrainOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.widen(), f)
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

impl From<BodyOperator> for MutationOperator {
    fn from(op: BodyOperator) -> Self {
        Self::Body(op)
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
    /// Committed learning edits (the fourteen learning operators) by how
    /// far they reached, indexed by [`Broadcast::index`] (T25.F12).
    pub attribute_changes_by_broadcast: [u32; 2],
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
            attribute_changes_by_broadcast: [0; 2],
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

    /// Record one committed learning edit at its reach.
    pub fn record_broadcast(&mut self, broadcast: Broadcast) {
        self.attribute_changes_by_broadcast[broadcast.index()] += 1;
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

    /// Record one body event of `operator` (T29.F02 invariant 4): attempted
    /// and applied, with no target, no discards and
    /// [`TargetReachability::NotApplicable`].
    pub fn record_body_event(&mut self, operator: crate::mutation::body::BodyOperator) {
        let operator = MutationOperator::Body(operator);
        self.record_attempt(MutationDomain::Body, operator);
        self.record_event(MutationEventRecord {
            domain: MutationDomain::Body,
            operator: Some(operator),
            target: None,
            outcome: MutationEventOutcome::Applied(TargetReachability::NotApplicable),
            discarded: Vec::new(),
        });
        self.record_applied(MutationDomain::Body, operator);
        self.record_reachability(TargetReachability::NotApplicable);
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
