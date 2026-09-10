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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

/// One module and the depth at which it first reached each cohort fact.
///
/// The facts are separate and never merged: a selection is not an applicable
/// selection, an applicable selection is not an internal change, dispatch is
/// not an effect, and a contribution is battery sensitivity only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
    pub lineage: u32,
    pub node: NodeId,
    pub created_depth: u64,
    pub backend: ModuleBackend,
    pub provenance: Provenance,
    pub deleted_depth: Option<u64>,
    pub first_selection: Option<u64>,
    pub first_applicable_selection: Option<u64>,
    pub first_internal_change: Option<u64>,
    pub first_dispatch: Option<u64>,
    pub first_contribution: Option<u64>,
    dispatched_now: bool,
    contributing_now: bool,
}

impl Module {
    fn new(
        lineage: u32,
        node: NodeId,
        created_depth: u64,
        backend: ModuleBackend,
        provenance: Provenance,
    ) -> Self {
        Self {
            lineage,
            node,
            created_depth,
            backend,
            provenance,
            deleted_depth: None,
            first_selection: None,
            first_applicable_selection: None,
            first_internal_change: None,
            first_dispatch: None,
            first_contribution: None,
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
    pub const fn rung(&self) -> Rung {
        if self.contributing_now {
            Rung::Contributing
        } else if self.dispatched_now {
            Rung::DispatchedNotContributing
        } else if self.first_internal_change.is_some() {
            Rung::ChangedOnly
        } else if self.first_applicable_selection.is_some() {
            Rung::AppliedOnly
        } else if self.first_selection.is_some() {
            Rung::SelectedOnly
        } else {
            Rung::NeverSelected
        }
    }

    fn fact(&self, fact: CohortFact) -> Option<u64> {
        match fact {
            CohortFact::Selection => self.first_selection,
            CohortFact::ApplicableSelection => self.first_applicable_selection,
            CohortFact::InternalChange => self.first_internal_change,
            CohortFact::Dispatch => self.first_dispatch,
            CohortFact::Contribution => self.first_contribution,
        }
    }

    fn fact_mut(&mut self, fact: CohortFact) -> &mut Option<u64> {
        match fact {
            CohortFact::Selection => &mut self.first_selection,
            CohortFact::ApplicableSelection => &mut self.first_applicable_selection,
            CohortFact::InternalChange => &mut self.first_internal_change,
            CohortFact::Dispatch => &mut self.first_dispatch,
            CohortFact::Contribution => &mut self.first_contribution,
        }
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

/// One of the cohort facts a module can reach.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CohortFact {
    Selection,
    ApplicableSelection,
    InternalChange,
    Dispatch,
    Contribution,
}

/// The mutation opportunities production offered one lineage, pooled from
/// depth 0 to the current depth.
///
/// The two `NoApplicableTarget` splits are keyed by domain because the engine
/// retries every operator of a domain before recording the skip: an
/// operator-level `NoApplicableTarget` never reaches the summary, so only the
/// domain owns the attempt. `selected_inapplicable` means a node was selected
/// and carried no applicable site; `no_eligible_node` means no node of the
/// required kind existed to select at all.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
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
    }
}

/// Cohort counts: creation, loss, and the exclusive state ladder over the
/// modules still present. `created == deleted + present`, and the six ladder
/// rungs partition `present`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
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
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
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
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TimeToFirst {
    pub reached: u64,
    pub median_generations: Option<u64>,
    pub censored_deleted: u64,
    pub censored_present: u64,
}

/// Time-to-first readings for every cohort fact.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TimeToFirstReading {
    pub selection: TimeToFirst,
    pub applicable_selection: TimeToFirst,
    pub internal_change: TimeToFirst,
    pub dispatch: TimeToFirst,
    pub contribution: TimeToFirst,
}

/// What became of the modules contributing at the previous checkpoint.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Retention {
    pub from_depth: u64,
    pub contributing_before: u64,
    pub still_contributing: u64,
    pub present_not_contributing: u64,
    pub deleted: u64,
}

/// One lineage's cohort row at a checkpoint.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LineageRow {
    pub lineage: u32,
    pub created: u64,
    pub present: u64,
    pub dispatched: u64,
    pub contributing: u64,
}

/// Everything the recruitment observation reports at one checkpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecruitmentCheckpoint {
    pub depth: u64,
    /// `new` + `copy` modules created at or before this checkpoint, pooled.
    pub cohort: CohortCounts,
    pub graph: CohortCounts,
    pub vm: CohortCounts,
    pub founders: FounderCounts,
    pub time_to_first: TimeToFirstReading,
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
}

/// The facts one birth of one lineage offers the tracker.
#[derive(Debug, Clone, Copy)]
pub struct BirthObservation<'a> {
    pub lineage: u32,
    /// The depth of the generation this birth produced.
    pub depth: u64,
    pub before: &'a [NodeGenome],
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

    /// Record a lineage's founder modules, created at depth 0.
    pub fn seed_founder(&mut self, lineage: u32, nodes: &[NodeGenome]) {
        for node in nodes {
            self.create(
                lineage,
                node.node_id,
                0,
                ModuleBackend::from(&node.backend_def),
                Provenance::Founder,
            );
        }
    }

    /// Every module of every lineage, in creation order per lineage.
    pub fn modules(&self) -> impl Iterator<Item = &Module> {
        self.lineages.iter().flat_map(|state| state.modules.iter())
    }

    fn create(
        &mut self,
        lineage: u32,
        node: NodeId,
        depth: u64,
        backend: ModuleBackend,
        provenance: Provenance,
    ) {
        let state = &mut self.lineages[lineage as usize];
        let index = state.modules.len();
        state
            .modules
            .push(Module::new(lineage, node, depth, backend, provenance));
        state.live.insert(node, index);
    }

    /// Pool one birth's opportunities and fold its genome diff into the
    /// module table: deletions, creations with provenance, selections from the
    /// event records, and internal change for the ids that survived.
    pub fn record_birth(&mut self, observation: BirthObservation<'_>) {
        let BirthObservation {
            lineage,
            depth,
            before,
            after,
            summary,
        } = observation;
        self.lineages[lineage as usize]
            .opportunities
            .record(summary);
        self.record_selections(lineage, depth, &summary.events);

        let after_by_id: BTreeMap<NodeId, &NodeGenome> =
            after.iter().map(|node| (node.node_id, node)).collect();
        for node in before {
            match after_by_id.get(&node.node_id) {
                None => self.delete(lineage, node.node_id, depth),
                Some(current) if *current != node => {
                    self.fact_reached(lineage, node.node_id, CohortFact::InternalChange, depth);
                }
                Some(_) => {}
            }
        }

        let copy_applied = summary.events.iter().any(|event| {
            event.outcome.is_applied() && event.operator == Some(MutationOperator::TopologyCopyNode)
        });
        let before_ids: BTreeSet<NodeId> = before.iter().map(|node| node.node_id).collect();
        for node in after {
            if before_ids.contains(&node.node_id) {
                continue;
            }
            let copied = copy_applied
                && before
                    .iter()
                    .any(|existing| existing.backend_def == node.backend_def);
            self.create(
                lineage,
                node.node_id,
                depth,
                ModuleBackend::from(&node.backend_def),
                if copied {
                    Provenance::Copy
                } else {
                    Provenance::New
                },
            );
        }
    }

    fn record_selections(&mut self, lineage: u32, depth: u64, events: &[MutationEventRecord]) {
        for event in events {
            let Some(target) = event.target else {
                continue;
            };
            self.fact_reached(lineage, target, CohortFact::Selection, depth);
            if event.outcome.is_applied() {
                self.fact_reached(lineage, target, CohortFact::ApplicableSelection, depth);
            }
        }
    }

    fn delete(&mut self, lineage: u32, node: NodeId, depth: u64) {
        let state = &mut self.lineages[lineage as usize];
        if let Some(index) = state.live.remove(&node) {
            let module = &mut state.modules[index];
            module.deleted_depth = Some(depth);
            module.dispatched_now = false;
            module.contributing_now = false;
        }
    }

    fn fact_reached(&mut self, lineage: u32, node: NodeId, fact: CohortFact, depth: u64) {
        let state = &mut self.lineages[lineage as usize];
        let Some(&index) = state.live.get(&node) else {
            return;
        };
        state.modules[index].fact_mut(fact).get_or_insert(depth);
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
            time_to_first: TimeToFirstReading::default(),
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
                if module.contributing_now {
                    contributors.insert((module.lineage, index));
                }
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
                reading.cohort.record(module);
                match module.backend {
                    ModuleBackend::Graph => reading.graph.record(module),
                    ModuleBackend::Vm => reading.vm.record(module),
                }
                row.created += 1;
                if module.is_present() {
                    row.present += 1;
                    row.dispatched += u64::from(module.dispatched_now);
                    row.contributing += u64::from(module.contributing_now);
                }
                for fact in FACTS {
                    match module.fact(fact) {
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

        for fact in FACTS {
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

const FACTS: [CohortFact; 5] = [
    CohortFact::Selection,
    CohortFact::ApplicableSelection,
    CohortFact::InternalChange,
    CohortFact::Dispatch,
    CohortFact::Contribution,
];

impl RecruitmentCheckpoint {
    fn time_to_first_mut(&mut self, fact: CohortFact) -> &mut TimeToFirst {
        match fact {
            CohortFact::Selection => &mut self.time_to_first.selection,
            CohortFact::ApplicableSelection => &mut self.time_to_first.applicable_selection,
            CohortFact::InternalChange => &mut self.time_to_first.internal_change,
            CohortFact::Dispatch => &mut self.time_to_first.dispatch,
            CohortFact::Contribution => &mut self.time_to_first.contribution,
        }
    }

    /// The time-to-first reading for one fact.
    #[must_use]
    pub const fn time_to_first(&self, fact: CohortFact) -> &TimeToFirst {
        match fact {
            CohortFact::Selection => &self.time_to_first.selection,
            CohortFact::ApplicableSelection => &self.time_to_first.applicable_selection,
            CohortFact::InternalChange => &self.time_to_first.internal_change,
            CohortFact::Dispatch => &self.time_to_first.dispatch,
            CohortFact::Contribution => &self.time_to_first.contribution,
        }
    }
}

#[cfg(test)]
mod tests;
