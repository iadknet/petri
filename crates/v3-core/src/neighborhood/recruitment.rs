//! Module recruitment observation for the mutation-only drift walk (T13.F01).
//!
//! A *module* is one mesh node in one lineage, identified by
//! `(lineage, node id, creation depth)`. The tracker below is fed the facts
//! the walk already produces — the node ids and node genomes on either side of
//! a birth, that birth's [`MutationEventRecord`]s, and the battery's executed
//! and contributing node ids at each reading — and answers, per checkpoint,
//! how far each cohort of new modules advanced.
//!
//! Everything here is observation: no RNG, no genome is changed, and no
//! battery is executed. Dispatch is not an effect, a contribution is battery
//! sensitivity only, and usefulness is unmeasured.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::hash::Hash;

use crate::contracts::NodeId;
use crate::creature::genome::{BackendDef, NodeGenome};
use crate::mutation::{
    MutationDomain, MutationEventOutcome, MutationEventRecord, MutationOperator,
    MutationSkipReason, MutationSummary,
};

/// Version of the recruitment observation contract.
pub const RECRUITMENT_VERSION: &str = "module-recruitment-v1";
/// How a module is identified along the walk.
pub const MODULE_IDENTITY: &str =
    "one mesh node in one lineage, keyed (lineage, node id, creation depth); \
     an id that vanishes and reappears is two modules";
/// How a newly created module's provenance is decided.
pub const PROVENANCE_RULE: &str =
    "copy when the same birth applied a Topology.CopyNode event and the new node's \
     backend_def equals some pre-birth node's, otherwise new";

/// The backend a module carries when it is created.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum ModuleBackend {
    Graph,
    Vm,
}

impl ModuleBackend {
    #[must_use]
    pub const fn as_key(self) -> &'static str {
        match self {
            Self::Graph => "graph",
            Self::Vm => "vm",
        }
    }
}

impl From<&BackendDef> for ModuleBackend {
    fn from(backend: &BackendDef) -> Self {
        match backend {
            BackendDef::Graph(_) => Self::Graph,
            BackendDef::Vm(_) => Self::Vm,
        }
    }
}

/// Where a module came from.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum Provenance {
    /// Present in the founder genome at depth 0.
    Founder,
    /// Created by a birth that applied no `CopyNode` event, or whose content
    /// matched no pre-birth node.
    New,
    /// Created by a birth that applied a `CopyNode` event, carrying the
    /// backend definition of some node the parent already had.
    Copy,
}

impl Provenance {
    #[must_use]
    pub const fn as_key(self) -> &'static str {
        match self {
            Self::Founder => "founder",
            Self::New => "new",
            Self::Copy => "copy",
        }
    }

    /// Whether this module belongs to the recruited cohort (`new` + `copy`).
    #[must_use]
    pub const fn is_cohort(self) -> bool {
        matches!(self, Self::New | Self::Copy)
    }
}

/// The pre-birth module a copy was read from (T13.F07): the first pre-birth
/// node whose `backend_def` matched, `ambiguous` when several matched.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CopySource {
    pub node: NodeId,
    pub created_depth: u64,
    pub ambiguous: bool,
}

/// Which part of a module a mutation event reached (T13.F07's target-local
/// exposure): its route entries (`Split`), its `backend_def` (`Payload`), or
/// neither (`Other`: input references, creation, deletion and copy sources).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EditSurface {
    Split,
    Payload,
    Other,
}

impl EditSurface {
    /// The surface an operator edits on the node it selected.
    #[must_use]
    pub const fn of(operator: MutationOperator) -> Self {
        match operator {
            MutationOperator::TopologyRetargetNodeTarget
            | MutationOperator::TopologyAddRouteTarget
            | MutationOperator::TopologyRemoveRouteTarget
            | MutationOperator::TopologySwapRouteTargets
            | MutationOperator::TopologyMutateGateBias
            | MutationOperator::TopologySpliceNode
            | MutationOperator::TopologyChangeEntryNode => Self::Split,
            MutationOperator::TopologySwapNodeBackend => Self::Payload,
            MutationOperator::TopologyAddNode
            | MutationOperator::TopologyRemoveNode
            | MutationOperator::TopologyCopyNode
            | MutationOperator::TopologyCopyMeshBackwardSlice
            | MutationOperator::TopologyCopyMeshForwardSlice
            | MutationOperator::InputRefAdd
            | MutationOperator::InputRefPrune
            | MutationOperator::InputRefSwap
            | MutationOperator::InputRefRawFieldMutation => Self::Other,
            _ => match operator.domain() {
                MutationDomain::Vm | MutationDomain::Graph => Self::Payload,
                MutationDomain::Topology | MutationDomain::InputRef => Self::Other,
            },
        }
    }
}

/// Counts of the events that selected one module on one surface, and the
/// depth of the first of each kind.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SurfaceExposure {
    pub applied: u64,
    pub discarded: u64,
    pub first_applied: Option<u64>,
    pub first_discarded: Option<u64>,
}

impl SurfaceExposure {
    fn record(&mut self, applied: bool, depth: u64) {
        let (count, first) = if applied {
            (&mut self.applied, &mut self.first_applied)
        } else {
            (&mut self.discarded, &mut self.first_discarded)
        };
        *count += 1;
        first.get_or_insert(depth);
    }
}

/// Target-local exposure of one module (T13.F07): every event that selected
/// it, applied or discarded, by the surface the operator edits. A new route
/// entry on another node naming this module counts as one applied split
/// exposure per birth.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Exposure {
    pub split: SurfaceExposure,
    pub payload: SurfaceExposure,
    pub other: SurfaceExposure,
}

impl Exposure {
    fn surface(&mut self, surface: EditSurface) -> &mut SurfaceExposure {
        match surface {
            EditSurface::Split => &mut self.split,
            EditSurface::Payload => &mut self.payload,
            EditSurface::Other => &mut self.other,
        }
    }

    /// Applied events over every surface.
    #[must_use]
    pub const fn applied(&self) -> u64 {
        self.split.applied + self.payload.applied + self.other.applied
    }

    /// Discarded events over every surface.
    #[must_use]
    pub const fn discarded(&self) -> u64 {
        self.split.discarded + self.payload.discarded + self.other.discarded
    }
}

/// One module and the depth at which it first reached each cohort fact.
///
/// The facts are separate and never merged: a selection is not an applicable
/// selection, an applicable selection is not an internal change, dispatch is
/// not an effect, and a contribution is battery sensitivity only.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Module {
    pub lineage: u32,
    pub node: NodeId,
    pub created_depth: u64,
    pub backend: ModuleBackend,
    pub provenance: Provenance,
    pub deleted_depth: Option<u64>,
    /// Which pre-birth module a `Copy` was read from; `None` for `New` and
    /// founder modules.
    #[serde(default)]
    pub copy_source: Option<CopySource>,
    /// SHA-256 of the `backend_def` this module was created with; the payload
    /// itself is read through [`RecruitmentTracker::birth_payload`].
    #[serde(default)]
    pub birth_payload_hash: String,
    /// Copies later read from this module.
    #[serde(default)]
    pub later_copies: u64,
    #[serde(default)]
    pub exposure: Exposure,
    /// Scenes of the latest task reading that dispatched this module (0..=8);
    /// contextual when 1..=7, unconditional when 8.
    #[serde(default)]
    pub scenes_dispatched: u8,
    /// The depth at which this module first reached each [`CohortFact`],
    /// indexed by `fact as usize`; read it through [`Module::first`].
    first: [Option<u64>; CohortFact::COUNT],
    dispatched_now: bool,
    contributing_now: bool,
}

/// SHA-256 hex of a value's JSON form: payload hashes and proposal
/// fingerprints alike.
#[must_use]
pub fn json_sha256(value: &impl serde::Serialize) -> String {
    use sha2::Digest;
    hex::encode(sha2::Sha256::digest(
        serde_json::to_vec(value).expect("observation values serialize"),
    ))
}

/// SHA-256 hex of a payload's JSON form.
#[must_use]
pub fn payload_hash(payload: &BackendDef) -> String {
    json_sha256(payload)
}

impl Module {
    fn new(
        lineage: u32,
        node: &NodeGenome,
        created_depth: u64,
        provenance: Provenance,
        copy_source: Option<CopySource>,
    ) -> Self {
        Self {
            lineage,
            node: node.node_id,
            created_depth,
            backend: ModuleBackend::from(&node.backend_def),
            provenance,
            deleted_depth: None,
            copy_source,
            birth_payload_hash: payload_hash(&node.backend_def),
            later_copies: 0,
            exposure: Exposure::default(),
            scenes_dispatched: 0,
            first: [None; CohortFact::COUNT],
            dispatched_now: false,
            contributing_now: false,
        }
    }

    /// Whether this module still exists.
    #[must_use]
    pub const fn is_present(&self) -> bool {
        self.deleted_depth.is_none()
    }

    /// Whether the latest reading dispatched this module.
    #[must_use]
    pub const fn is_dispatched(&self) -> bool {
        self.dispatched_now
    }

    /// Whether the latest checkpoint's knockout made this module contribute.
    #[must_use]
    pub const fn is_contributing(&self) -> bool {
        self.contributing_now
    }

    /// The rung this module occupies in the exclusive state ladder.
    #[must_use]
    pub fn rung(&self) -> Rung {
        if self.contributing_now {
            Rung::Contributing
        } else if self.dispatched_now {
            Rung::DispatchedNotContributing
        } else if self.first(CohortFact::InternalChange).is_some() {
            Rung::ChangedOnly
        } else if self.first(CohortFact::ApplicableSelection).is_some() {
            Rung::AppliedOnly
        } else if self.first(CohortFact::Selection).is_some() {
            Rung::SelectedOnly
        } else {
            Rung::NeverSelected
        }
    }

    /// The depth at which this module first reached `fact`, if it has.
    #[must_use]
    pub fn first(&self, fact: CohortFact) -> Option<u64> {
        self.first[fact as usize]
    }

    fn first_mut(&mut self, fact: CohortFact) -> &mut Option<u64> {
        &mut self.first[fact as usize]
    }
}

/// A rung of the exclusive state ladder; a module sits in the highest rung it
/// has reached.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Rung {
    NeverSelected,
    SelectedOnly,
    AppliedOnly,
    ChangedOnly,
    DispatchedNotContributing,
    Contributing,
}

/// One of the cohort facts a module can reach. The discriminants index
/// [`Module`]'s and [`RecruitmentCheckpoint`]'s per-fact arrays, so adding a
/// fact here is the only edit a new fact needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CohortFact {
    Selection,
    ApplicableSelection,
    InternalChange,
    Dispatch,
    Contribution,
}

impl CohortFact {
    /// Every fact, in ladder order.
    pub const ALL: [Self; Self::COUNT] = [
        Self::Selection,
        Self::ApplicableSelection,
        Self::InternalChange,
        Self::Dispatch,
        Self::Contribution,
    ];
    /// How many facts there are.
    pub const COUNT: usize = 5;

    /// The stable key this fact is reported under.
    #[must_use]
    pub const fn as_key(self) -> &'static str {
        match self {
            Self::Selection => "selection",
            Self::ApplicableSelection => "applicable_selection",
            Self::InternalChange => "internal_change",
            Self::Dispatch => "dispatch",
            Self::Contribution => "contribution",
        }
    }
}

/// The mutation opportunities production offered one lineage, pooled from
/// depth 0 to the current depth.
///
/// `selected_inapplicable` means a node was selected and carried no applicable
/// site; `no_eligible_node` means no node of the required kind existed to
/// select at all. The split is reported twice. The `_by_domain` maps count
/// whole events, and only events whose domain exhausted every operator can
/// count there, because the engine retries the other operators of a domain
/// before recording the skip. The `discarded_*_by_operator` maps count each
/// operator the engine threw away for reporting no applicable site, whether or
/// not a later operator of the same domain then applied, so a module selected
/// by an operator that found no site is visible in them alone.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Opportunities {
    pub births: u64,
    pub zero_event_births: u64,
    pub attempted: u64,
    pub applied: u64,
    pub skipped: u64,
    pub reachable_target_events: u64,
    pub unreachable_target_events: u64,
    pub executed_target_events: u64,
    pub attempted_by_domain: BTreeMap<MutationDomain, u64>,
    pub applied_by_domain: BTreeMap<MutationDomain, u64>,
    pub attempted_by_operator: BTreeMap<MutationOperator, u64>,
    pub applied_by_operator: BTreeMap<MutationOperator, u64>,
    pub skipped_by_operator_reason: BTreeMap<MutationOperator, BTreeMap<MutationSkipReason, u64>>,
    pub selected_inapplicable_by_domain: BTreeMap<MutationDomain, u64>,
    pub no_eligible_node_by_domain: BTreeMap<MutationDomain, u64>,
    pub discarded_selected_inapplicable_by_operator: BTreeMap<MutationOperator, u64>,
    pub discarded_no_eligible_node_by_operator: BTreeMap<MutationOperator, u64>,
}

fn bump<K: Ord>(map: &mut BTreeMap<K, u64>, key: K, count: u64) {
    *map.entry(key).or_default() += count;
}

fn merge_map<K: Ord + Copy>(into: &mut BTreeMap<K, u64>, from: &BTreeMap<K, u64>) {
    for (&key, &count) in from {
        bump(into, key, count);
    }
}

/// Fold one of [`MutationSummary`]'s `u32` tallies into a pooled `u64` one.
fn absorb<K: Ord + Copy + Hash>(into: &mut BTreeMap<K, u64>, from: &HashMap<K, u32>) {
    for (&key, &count) in from {
        bump(into, key, u64::from(count));
    }
}

impl Opportunities {
    /// Pool one birth's summary.
    pub fn record(&mut self, summary: &MutationSummary) {
        self.births += 1;
        self.zero_event_births += u64::from(summary.attempted_events == 0);
        self.attempted += u64::from(summary.attempted_events);
        self.applied += u64::from(summary.applied_events);
        self.skipped += u64::from(summary.skipped_events);
        self.reachable_target_events += u64::from(summary.reachable_target_events);
        self.unreachable_target_events += u64::from(summary.unreachable_target_events);
        self.executed_target_events += u64::from(summary.executed_target_events);
        absorb(&mut self.attempted_by_domain, &summary.attempted_by_domain);
        absorb(&mut self.applied_by_domain, &summary.applied_by_domain);
        absorb(
            &mut self.attempted_by_operator,
            &summary.attempted_by_operator,
        );
        absorb(&mut self.applied_by_operator, &summary.applied_by_operator);
        for (&operator, reasons) in &summary.skip_reasons_by_operator {
            absorb(
                self.skipped_by_operator_reason.entry(operator).or_default(),
                reasons,
            );
        }
        for event in &summary.events {
            if event.outcome
                == MutationEventOutcome::Skipped(MutationSkipReason::NoApplicableTarget)
            {
                let split = if event.target.is_some() {
                    &mut self.selected_inapplicable_by_domain
                } else {
                    &mut self.no_eligible_node_by_domain
                };
                bump(split, event.domain, 1);
            }
            for &(operator, pick) in &event.discarded {
                let split = if pick.is_some() {
                    &mut self.discarded_selected_inapplicable_by_operator
                } else {
                    &mut self.discarded_no_eligible_node_by_operator
                };
                bump(split, operator, 1);
            }
        }
    }

    /// Pool another lineage's totals into these.
    pub fn merge(&mut self, other: &Self) {
        self.births += other.births;
        self.zero_event_births += other.zero_event_births;
        self.attempted += other.attempted;
        self.applied += other.applied;
        self.skipped += other.skipped;
        self.reachable_target_events += other.reachable_target_events;
        self.unreachable_target_events += other.unreachable_target_events;
        self.executed_target_events += other.executed_target_events;
        merge_map(&mut self.attempted_by_domain, &other.attempted_by_domain);
        merge_map(&mut self.applied_by_domain, &other.applied_by_domain);
        merge_map(
            &mut self.attempted_by_operator,
            &other.attempted_by_operator,
        );
        merge_map(&mut self.applied_by_operator, &other.applied_by_operator);
        for (&operator, reasons) in &other.skipped_by_operator_reason {
            let entry = self.skipped_by_operator_reason.entry(operator).or_default();
            merge_map(entry, reasons);
        }
        merge_map(
            &mut self.selected_inapplicable_by_domain,
            &other.selected_inapplicable_by_domain,
        );
        merge_map(
            &mut self.no_eligible_node_by_domain,
            &other.no_eligible_node_by_domain,
        );
        merge_map(
            &mut self.discarded_selected_inapplicable_by_operator,
            &other.discarded_selected_inapplicable_by_operator,
        );
        merge_map(
            &mut self.discarded_no_eligible_node_by_operator,
            &other.discarded_no_eligible_node_by_operator,
        );
    }
}

/// Cohort counts: creation, loss, and the exclusive state ladder over the
/// modules still present. `created == deleted + present`, and the six ladder
/// rungs partition `present`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CohortCounts {
    pub created: u64,
    pub deleted: u64,
    pub present: u64,
    pub never_selected: u64,
    pub selected_only: u64,
    pub applied_only: u64,
    pub changed_only: u64,
    pub dispatched_not_contributing: u64,
    pub contributing: u64,
}

impl CohortCounts {
    fn record(&mut self, module: &Module) {
        self.created += 1;
        if !module.is_present() {
            self.deleted += 1;
            return;
        }
        self.present += 1;
        match module.rung() {
            Rung::NeverSelected => self.never_selected += 1,
            Rung::SelectedOnly => self.selected_only += 1,
            Rung::AppliedOnly => self.applied_only += 1,
            Rung::ChangedOnly => self.changed_only += 1,
            Rung::DispatchedNotContributing => self.dispatched_not_contributing += 1,
            Rung::Contributing => self.contributing += 1,
        }
    }

    /// Modules present and dispatched at the latest reading.
    #[must_use]
    pub const fn dispatched(&self) -> u64 {
        self.dispatched_not_contributing + self.contributing
    }
}

/// Founder modules as a reference row beside the recruited cohort.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FounderCounts {
    pub created: u64,
    pub deleted: u64,
    pub present: u64,
    pub dispatched: u64,
    pub contributing: u64,
}

/// For one cohort fact: how many modules reached it, the integer median
/// generations from creation among those, and the two censoring counts.
///
/// `reached + censored_deleted + censored_present == created`: no module is
/// dropped from the denominator.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TimeToFirst {
    pub reached: u64,
    pub median_generations: Option<u64>,
    pub censored_deleted: u64,
    pub censored_present: u64,
}

/// What became of the *cohort* modules contributing at the previous
/// checkpoint. Founder modules are outside the cohort and never counted here,
/// so `contributing_before` equals the earlier checkpoint's
/// [`CohortCounts::contributing`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Retention {
    pub from_depth: u64,
    pub contributing_before: u64,
    pub still_contributing: u64,
    pub present_not_contributing: u64,
    pub deleted: u64,
}

/// One lineage's cohort row at a checkpoint.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LineageRow {
    pub lineage: u32,
    pub created: u64,
    pub present: u64,
    pub dispatched: u64,
    pub contributing: u64,
    /// Cohort modules that reached `ApplicableSelection` (at least one
    /// applied event found a site on them); over `created`, this is
    /// T13.F07's `eligible_site_fraction`.
    #[serde(default)]
    pub applicable: u64,
}

/// Everything the recruitment observation reports at one checkpoint.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RecruitmentCheckpoint {
    pub depth: u64,
    /// `new` + `copy` modules created at or before this checkpoint, pooled.
    pub cohort: CohortCounts,
    pub graph: CohortCounts,
    pub vm: CohortCounts,
    pub founders: FounderCounts,
    /// Time-to-first readings indexed by [`CohortFact`]; read them through
    /// [`RecruitmentCheckpoint::time_to_first`].
    time_to_first: [TimeToFirst; CohortFact::COUNT],
    /// Absent at the first checkpoint: retention needs an earlier one.
    pub retention: Option<Retention>,
    pub lineage_rows: Vec<LineageRow>,
    /// Cumulative mutation opportunities pooled over every lineage.
    pub opportunities: Opportunities,
    pub lineage_opportunities: Vec<Opportunities>,
}

/// One lineage's live modules, its module table, and its pooled opportunities.
#[derive(Debug, Clone, Default)]
struct LineageState {
    opportunities: Opportunities,
    live: BTreeMap<NodeId, usize>,
    modules: Vec<Module>,
    /// Each module's `backend_def` at creation, aligned with `modules`.
    payloads: Vec<BackendDef>,
    /// The nodes this lineage carried before the next birth, retained so a
    /// generation is diffed in place. Only the nodes a birth changed or added
    /// are cloned into it, so a birth that changed nothing clones nothing.
    previous: BTreeMap<NodeId, NodeGenome>,
}

/// The facts one birth of one lineage offers the tracker.
///
/// The pre-birth nodes are the tracker's own retained snapshot, seeded by
/// [`RecruitmentTracker::seed_founder`], so the caller hands over only the
/// genome the birth produced.
#[derive(Debug, Clone, Copy)]
pub struct BirthObservation<'a> {
    pub lineage: u32,
    /// The depth of the generation this birth produced.
    pub depth: u64,
    pub after: &'a [NodeGenome],
    pub summary: &'a MutationSummary,
}

/// The module table and cohort facts for a whole walk.
#[derive(Debug, Clone)]
pub struct RecruitmentTracker {
    lineages: Vec<LineageState>,
    last_contributors: BTreeSet<(u32, usize)>,
    last_checkpoint_depth: Option<u64>,
}

impl RecruitmentTracker {
    /// A tracker for `lineages` lineages, with no modules yet.
    #[must_use]
    pub fn new(lineages: u32) -> Self {
        Self {
            lineages: vec![LineageState::default(); lineages as usize],
            last_contributors: BTreeSet::new(),
            last_checkpoint_depth: None,
        }
    }

    /// Record a lineage's founder modules, created at depth 0, and seed the
    /// snapshot every later birth is diffed against.
    pub fn seed_founder(&mut self, lineage: u32, nodes: &[NodeGenome]) {
        for node in nodes {
            self.create(lineage, node, 0, Provenance::Founder, None);
        }
        let state = &mut self.lineages[lineage as usize];
        state.previous = nodes
            .iter()
            .map(|node| (node.node_id, node.clone()))
            .collect();
    }

    /// Every module of every lineage, in creation order per lineage.
    pub fn modules(&self) -> impl Iterator<Item = &Module> {
        self.lineages.iter().flat_map(|state| state.modules.iter())
    }

    /// Re-base every live module's birth payload on `nodes`: a constructed
    /// starting form's history is authored construction, not lineage
    /// mutation, so the payload a form starts with is its birth payload.
    pub fn rebase_birth_payloads(&mut self, lineage: u32, nodes: &[NodeGenome]) {
        let state = &mut self.lineages[lineage as usize];
        for node in nodes {
            if let Some(&index) = state.live.get(&node.node_id) {
                state.payloads[index].clone_from(&node.backend_def);
                state.modules[index].birth_payload_hash = payload_hash(&node.backend_def);
            }
        }
    }

    /// The `backend_def` a module was created with.
    #[must_use]
    pub fn birth_payload(&self, module: &Module) -> Option<&BackendDef> {
        let state = self.lineages.get(module.lineage as usize)?;
        state
            .modules
            .iter()
            .position(|candidate| {
                candidate.node == module.node && candidate.created_depth == module.created_depth
            })
            .map(|index| &state.payloads[index])
    }

    fn create(
        &mut self,
        lineage: u32,
        node: &NodeGenome,
        depth: u64,
        provenance: Provenance,
        copy_source: Option<CopySource>,
    ) {
        let state = &mut self.lineages[lineage as usize];
        let index = state.modules.len();
        if let Some(source) = copy_source {
            if let Some(&source_index) = state.live.get(&source.node) {
                state.modules[source_index].later_copies += 1;
            }
        }
        state
            .modules
            .push(Module::new(lineage, node, depth, provenance, copy_source));
        state.payloads.push(node.backend_def.clone());
        state.live.insert(node.node_id, index);
    }

    /// Pool one birth's opportunities and fold its genome diff into the
    /// module table: deletions, creations with provenance, selections from the
    /// event records, and internal change for the ids that survived. The diff
    /// runs against the lineage's retained snapshot, which this call updates.
    pub fn record_birth(&mut self, observation: BirthObservation<'_>) {
        let BirthObservation {
            lineage,
            depth,
            after,
            summary,
        } = observation;
        let copy_applied = summary.events.iter().any(|event| {
            event.outcome.is_applied() && event.operator == Some(MutationOperator::TopologyCopyNode)
        });

        let state = &mut self.lineages[lineage as usize];
        state.opportunities.record(summary);
        let mut previous = std::mem::take(&mut state.previous);
        let mut changed: Vec<usize> = Vec::new();
        let mut created: Vec<(usize, Provenance, Option<CopySource>)> = Vec::new();
        // Route entries this birth added, by the module they name (T13.F07
        // split-gate exposure); at most once per module per birth.
        let mut named: BTreeSet<NodeId> = BTreeSet::new();
        // Classify against the untouched snapshot: provenance asks whether
        // some *pre-birth* node carried this content, so nothing is written
        // back until the whole birth has been read.
        for (index, node) in after.iter().enumerate() {
            match previous.get(&node.node_id) {
                Some(slot) if slot == node => {}
                Some(slot) => {
                    named.extend(
                        node.targets
                            .iter()
                            .map(|target| target.target_id)
                            .filter(|id| !slot.targets.iter().any(|old| old.target_id == *id)),
                    );
                    changed.push(index);
                }
                None => {
                    named.extend(node.targets.iter().map(|target| target.target_id));
                    let mut matches = previous
                        .values()
                        .filter(|existing| existing.backend_def == node.backend_def);
                    let source = copy_applied.then(|| matches.next()).flatten();
                    let (provenance, copy_source) = match source {
                        Some(existing) => {
                            let ambiguous = matches.next().is_some();
                            let created_depth = state
                                .live
                                .get(&existing.node_id)
                                .map_or(0, |&index| state.modules[index].created_depth);
                            (
                                Provenance::Copy,
                                Some(CopySource {
                                    node: existing.node_id,
                                    created_depth,
                                    ambiguous,
                                }),
                            )
                        }
                        None => (Provenance::New, None),
                    };
                    created.push((index, provenance, copy_source));
                }
            }
        }
        for &index in &changed {
            if let Some(slot) = previous.get_mut(&after[index].node_id) {
                slot.clone_from(&after[index]);
            }
        }
        for &(index, _, _) in &created {
            previous.insert(after[index].node_id, after[index].clone());
        }
        // Every id of `after` is now in the snapshot, so a longer snapshot is
        // the only way this birth deleted anything.
        let deleted: Vec<NodeId> = if previous.len() == after.len() {
            Vec::new()
        } else {
            let live: BTreeSet<NodeId> = after.iter().map(|node| node.node_id).collect();
            let gone: Vec<NodeId> = previous
                .keys()
                .copied()
                .filter(|id| !live.contains(id))
                .collect();
            previous.retain(|id, _| live.contains(id));
            gone
        };
        self.lineages[lineage as usize].previous = previous;

        self.record_selections(lineage, depth, &summary.events);
        for node in named {
            self.expose(lineage, node, EditSurface::Split, true, depth);
        }
        for node in deleted {
            self.delete(lineage, node, depth);
        }
        for index in changed {
            let node = after[index].node_id;
            self.fact_reached(lineage, node, CohortFact::InternalChange, depth);
        }
        for (index, provenance, copy_source) in created {
            self.create(lineage, &after[index], depth, provenance, copy_source);
        }
    }

    fn record_selections(&mut self, lineage: u32, depth: u64, events: &[MutationEventRecord]) {
        for event in events {
            // A discarded operator's pick is a selection too: the module was
            // named and carried no applicable site.
            for (operator, target) in event
                .discarded
                .iter()
                .filter_map(|&(operator, pick)| pick.map(|pick| (operator, pick)))
            {
                self.fact_reached(lineage, target, CohortFact::Selection, depth);
                self.expose(lineage, target, EditSurface::of(operator), false, depth);
            }
            let Some(target) = event.target else {
                continue;
            };
            self.fact_reached(lineage, target, CohortFact::Selection, depth);
            if event.outcome.is_applied() {
                self.fact_reached(lineage, target, CohortFact::ApplicableSelection, depth);
            }
            if let Some(operator) = event.operator {
                self.expose(
                    lineage,
                    target,
                    EditSurface::of(operator),
                    event.outcome.is_applied(),
                    depth,
                );
            }
        }
    }

    fn expose(
        &mut self,
        lineage: u32,
        node: NodeId,
        surface: EditSurface,
        applied: bool,
        depth: u64,
    ) {
        let state = &mut self.lineages[lineage as usize];
        if let Some(&index) = state.live.get(&node) {
            state.modules[index]
                .exposure
                .surface(surface)
                .record(applied, depth);
        }
    }

    /// Fold one task reading's per-scene dispatch counts in (T13.F07): each
    /// live module's `scenes_dispatched` becomes its count, zero when absent.
    pub fn record_scene_dispatch(&mut self, lineage: u32, scenes: &BTreeMap<NodeId, u8>) {
        let state = &mut self.lineages[lineage as usize];
        for (&node, &index) in &state.live {
            state.modules[index].scenes_dispatched = scenes.get(&node).copied().unwrap_or(0);
        }
    }

    fn delete(&mut self, lineage: u32, node: NodeId, depth: u64) {
        let state = &mut self.lineages[lineage as usize];
        if let Some(index) = state.live.remove(&node) {
            let module = &mut state.modules[index];
            module.deleted_depth = Some(depth);
            module.dispatched_now = false;
            module.contributing_now = false;
            module.scenes_dispatched = 0;
        }
    }

    fn fact_reached(&mut self, lineage: u32, node: NodeId, fact: CohortFact, depth: u64) {
        let state = &mut self.lineages[lineage as usize];
        let Some(&index) = state.live.get(&node) else {
            return;
        };
        state.modules[index].first_mut(fact).get_or_insert(depth);
    }

    /// Fold one battery reading of one lineage in: which modules the battery
    /// dispatched, and — at a checkpoint only — which of those contributed.
    ///
    /// `contributing` must be a subset of `executed`.
    pub fn record_reading(
        &mut self,
        lineage: u32,
        depth: u64,
        executed: &BTreeSet<NodeId>,
        contributing: Option<&BTreeSet<NodeId>>,
    ) {
        let state = &mut self.lineages[lineage as usize];
        for (&node, &index) in &state.live {
            let module = &mut state.modules[index];
            module.dispatched_now = executed.contains(&node);
            // Contribution is only defined at a checkpoint's knockout, so a
            // plain refresh reading clears it rather than carrying it forward.
            module.contributing_now =
                contributing.is_some_and(|set| module.dispatched_now && set.contains(&node));
        }
        for &node in executed {
            self.fact_reached(lineage, node, CohortFact::Dispatch, depth);
        }
        for &node in contributing.into_iter().flatten() {
            self.fact_reached(lineage, node, CohortFact::Contribution, depth);
        }
    }

    /// The checkpoint reading over every module recorded so far.
    ///
    /// Call after every lineage's checkpoint reading has been folded in, so
    /// the ladder's dispatch and contribution rungs describe this checkpoint.
    /// A [`RecruitmentTracker::record_reading`] without `contributing` in
    /// between resets contribution, so a refresh reading must never sit
    /// between a checkpoint's readings and this call.
    pub fn checkpoint(&mut self, depth: u64) -> RecruitmentCheckpoint {
        let mut reading = RecruitmentCheckpoint {
            depth,
            cohort: CohortCounts::default(),
            graph: CohortCounts::default(),
            vm: CohortCounts::default(),
            founders: FounderCounts::default(),
            time_to_first: [TimeToFirst::default(); CohortFact::COUNT],
            retention: self.last_checkpoint_depth.map(|from_depth| Retention {
                from_depth,
                contributing_before: self.last_contributors.len() as u64,
                ..Retention::default()
            }),
            lineage_rows: Vec::with_capacity(self.lineages.len()),
            opportunities: Opportunities::default(),
            lineage_opportunities: Vec::with_capacity(self.lineages.len()),
        };
        let mut contributors = BTreeSet::new();
        let mut generations: BTreeMap<CohortFact, Vec<u64>> = BTreeMap::new();

        for (lineage, state) in self.lineages.iter().enumerate() {
            let mut row = LineageRow {
                lineage: lineage as u32,
                ..LineageRow::default()
            };
            for (index, module) in state.modules.iter().enumerate() {
                if !module.provenance.is_cohort() {
                    let founders = &mut reading.founders;
                    founders.created += 1;
                    if module.is_present() {
                        founders.present += 1;
                        founders.dispatched += u64::from(module.dispatched_now);
                        founders.contributing += u64::from(module.contributing_now);
                    } else {
                        founders.deleted += 1;
                    }
                    continue;
                }
                // Retention is a cohort reading, so only `new`/`copy` modules
                // enter the next checkpoint's denominator.
                if module.contributing_now {
                    contributors.insert((module.lineage, index));
                }
                reading.cohort.record(module);
                match module.backend {
                    ModuleBackend::Graph => reading.graph.record(module),
                    ModuleBackend::Vm => reading.vm.record(module),
                }
                row.created += 1;
                row.applicable +=
                    u64::from(module.first(CohortFact::ApplicableSelection).is_some());
                if module.is_present() {
                    row.present += 1;
                    row.dispatched += u64::from(module.dispatched_now);
                    row.contributing += u64::from(module.contributing_now);
                }
                for fact in CohortFact::ALL {
                    match module.first(fact) {
                        Some(reached) => generations
                            .entry(fact)
                            .or_default()
                            .push(reached.saturating_sub(module.created_depth)),
                        None => {
                            let censored = reading.time_to_first_mut(fact);
                            if module.is_present() {
                                censored.censored_present += 1;
                            } else {
                                censored.censored_deleted += 1;
                            }
                        }
                    }
                }
            }
            reading.lineage_rows.push(row);
            reading.opportunities.merge(&state.opportunities);
            reading
                .lineage_opportunities
                .push(state.opportunities.clone());
        }

        for fact in CohortFact::ALL {
            let mut samples = generations.remove(&fact).unwrap_or_default();
            samples.sort_unstable();
            let slot = reading.time_to_first_mut(fact);
            slot.reached = samples.len() as u64;
            slot.median_generations = samples.get(samples.len() / 2).copied();
        }

        if let Some(retention) = reading.retention.as_mut() {
            for &(lineage, index) in &self.last_contributors {
                let module = &self.lineages[lineage as usize].modules[index];
                if !module.is_present() {
                    retention.deleted += 1;
                } else if module.contributing_now {
                    retention.still_contributing += 1;
                } else {
                    retention.present_not_contributing += 1;
                }
            }
        }
        self.last_contributors = contributors;
        self.last_checkpoint_depth = Some(depth);
        reading
    }
}

impl RecruitmentCheckpoint {
    fn time_to_first_mut(&mut self, fact: CohortFact) -> &mut TimeToFirst {
        &mut self.time_to_first[fact as usize]
    }

    /// The time-to-first reading for one fact.
    #[must_use]
    pub const fn time_to_first(&self, fact: CohortFact) -> &TimeToFirst {
        &self.time_to_first[fact as usize]
    }
}

#[cfg(test)]
mod tests;
