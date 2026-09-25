//! Mutation-effect attribution and observation coverage (T11.F26).
//!
//! `mutation-effects-v1` runs single-event diagnostic proposals from three
//! parent cohorts (the canonical founder, the drift walk's birth lineages at
//! its last checkpoint, and a draw of a world's selected genomes) and
//! attributes each applied proposal by comparing traced `neighborhood-v1`
//! executions of parent and child. `neighborhood-coverage-v1` re-reads a
//! fixed sample of battery-silent pairs on contexts and histories the
//! battery lacks, beside three controls.
//!
//! Observation only: every function runs on clones after the last tick,
//! draws from its own seeded RNGs, and folds integers in a fixed order.

pub mod contexts;
mod controls;
mod records;
#[cfg(test)]
mod tests;

use std::collections::{BTreeMap, BTreeSet};

use rand::rngs::SmallRng;
use rand::SeedableRng;
use rayon::prelude::*;

use crate::config::MutationConfig;
use crate::contracts::{NodeId, WorldAction};
use crate::creature::genome::analysis::mesh_reachable_nodes;
use crate::creature::genome::CreatureGenome;
use crate::mutation::reachability::ParentExecuted;
use crate::mutation::{MutationDomain, MutationEngine, MutationOperator};
use crate::simulation::Simulation;

use super::classify::{classify, Class};
use super::mesh_execution::indices_for_node_ids;
use super::{Battery, EvalContext, Signature};
pub use contexts::Group;
use contexts::{recorded_contexts, Extension};
use records::{run_panel_executions, PanelExecution, TracedBattery};

pub const VERSION: &str = "mutation-effects-v1";
pub const COVERAGE_VERSION: &str = "neighborhood-coverage-v1";
/// Proposal seed: `PROPOSAL_SEED_BASE + COHORT_SEED_MULTIPLIER * cohort +
/// PARENT_SEED_MULTIPLIER * (parent_index + 1) + proposal_index`, where
/// `parent_index` is the parent's zero-based ordinal within its cohort, not
/// the stored `parents[].index` (for `selected`, the original sample position).
pub const PROPOSAL_SEED_BASE: u64 = 20_000_000;
pub const COHORT_SEED_MULTIPLIER: u64 = 1_000_000;
pub const PARENT_SEED_MULTIPLIER: u64 = 1_000;
/// Selected-cohort draw seed: `SELECTED_SEED_BASE + world_seed`.
pub const SELECTED_SEED_BASE: u64 = 24_000_000;

/// The genome identity every T11.F26 comparison uses: the `Debug`
/// rendering, which distinguishes `-0.0`, renders every NaN alike, and omits
/// the birth-only `birth_weights` (see `CgpGraphBackendDef`'s `Debug`).
#[must_use]
pub fn genome_identity(genome: &CreatureGenome) -> String {
    format!("{genome:?}")
}

/// Predeclared sizes. Production profiles run [`Sizes::PRODUCTION`]; the
/// small default keeps fixtures cheap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sizes {
    /// Selected-cohort parents requested per world.
    pub parents: u32,
    /// Single-event proposals per parent.
    pub proposals: u32,
    /// Coverage pairs sampled per parent.
    pub pairs_per_parent: u32,
    /// Recorded contexts requested per world.
    pub recorded_contexts: u32,
}

impl Sizes {
    pub const PRODUCTION: Self = Self {
        parents: 20,
        proposals: 100,
        pairs_per_parent: 10,
        recorded_contexts: 32,
    };
}

impl Default for Sizes {
    fn default() -> Self {
        Self {
            parents: 2,
            proposals: 6,
            pairs_per_parent: 2,
            recorded_contexts: 4,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cohort {
    Founder,
    Drift,
    Selected,
}

impl Cohort {
    #[must_use]
    pub const fn index(self) -> u64 {
        self as u64
    }

    #[must_use]
    pub const fn as_key(self) -> &'static str {
        match self {
            Self::Founder => "founder",
            Self::Drift => "drift",
            Self::Selected => "selected",
        }
    }
}

/// The partition of applied proposals, in precedence order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    ActionChanged,
    ActionDead,
    GenomeIdentical,
    UnexecutedEdit,
    MaskedBeforeSelection,
    StateOrCostOnly,
    Unresolved,
}

impl Category {
    pub const ALL: [Self; 7] = [
        Self::ActionChanged,
        Self::ActionDead,
        Self::GenomeIdentical,
        Self::UnexecutedEdit,
        Self::MaskedBeforeSelection,
        Self::StateOrCostOnly,
        Self::Unresolved,
    ];

    #[must_use]
    pub const fn index(self) -> usize {
        self as usize
    }

    #[must_use]
    pub const fn as_key(self) -> &'static str {
        match self {
            Self::ActionChanged => "action_changed",
            Self::ActionDead => "action_dead",
            Self::GenomeIdentical => "genome_identical",
            Self::UnexecutedEdit => "unexecuted_edit",
            Self::MaskedBeforeSelection => "masked_before_selection",
            Self::StateOrCostOnly => "state_or_cost_only",
            Self::Unresolved => "unresolved",
        }
    }
}

/// Where a proposal's first recorded target sat in the parent: a stratum
/// label, never a cause.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetClass {
    KnockoutContributing,
    ExecutedNotContributing,
    ReachableNotExecuted,
    Unreachable,
    NoNodeTarget,
}

impl TargetClass {
    pub const ALL: [Self; 5] = [
        Self::KnockoutContributing,
        Self::ExecutedNotContributing,
        Self::ReachableNotExecuted,
        Self::Unreachable,
        Self::NoNodeTarget,
    ];

    #[must_use]
    pub const fn index(self) -> usize {
        self as usize
    }

    #[must_use]
    pub const fn as_key(self) -> &'static str {
        match self {
            Self::KnockoutContributing => "knockout_contributing",
            Self::ExecutedNotContributing => "executed_not_contributing",
            Self::ReachableNotExecuted => "reachable_not_executed",
            Self::Unreachable => "unreachable",
            Self::NoNodeTarget => "no_node_target",
        }
    }
}

/// The accepted operator of a proposal's event, or the domain whose every
/// operator found no applicable site.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OperatorKey {
    Operator(MutationOperator),
    DomainExhausted(MutationDomain),
}

impl OperatorKey {
    #[must_use]
    pub fn as_key(self) -> String {
        match self {
            Self::Operator(operator) => operator.as_key().to_string(),
            Self::DomainExhausted(domain) => format!("{domain:?}.domain_exhausted"),
        }
    }
}

/// One applied proposal's attribution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Attribution {
    pub category: Category,
    /// Action-silent with a differing state-and-cost record, any category.
    pub silent_with_state_or_cost: bool,
    /// `UnexecutedEdit` whose records differ anyway.
    pub consistency_violation: bool,
    /// The existing classifier's `Silent`.
    pub action_silent: bool,
}

/// Integer counts over a set of proposals. `categories` is indexed by
/// [`Category::index`] and sums to `proposals - skipped`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EffectCounts {
    pub proposals: u32,
    pub skipped: u32,
    pub categories: [u32; 7],
    pub silent_with_state_or_cost: u32,
    pub consistency_violations: u32,
}

impl EffectCounts {
    #[must_use]
    pub fn applied(&self) -> u32 {
        self.proposals - self.skipped
    }

    fn record(&mut self, attribution: Option<Attribution>) {
        self.proposals += 1;
        let Some(attribution) = attribution else {
            self.skipped += 1;
            return;
        };
        self.categories[attribution.category.index()] += 1;
        self.silent_with_state_or_cost += u32::from(attribution.silent_with_state_or_cost);
        self.consistency_violations += u32::from(attribution.consistency_violation);
    }

    #[must_use]
    pub fn merge(mut self, other: &Self) -> Self {
        self.proposals += other.proposals;
        self.skipped += other.skipped;
        for (total, count) in self.categories.iter_mut().zip(other.categories) {
            *total += count;
        }
        self.silent_with_state_or_cost += other.silent_with_state_or_cost;
        self.consistency_violations += other.consistency_violations;
        self
    }

    /// The category holding the most proposals; ties resolve to the earlier
    /// category in precedence order. `None` with nothing applied.
    #[must_use]
    pub fn plurality(&self) -> Option<Category> {
        let max = *self.categories.iter().max()?;
        (max > 0).then(|| {
            Category::ALL
                .into_iter()
                .find(|category| self.categories[category.index()] == max)
                .expect("some category holds the maximum")
        })
    }
}

/// One cohort parent: its index in the source reading (drift lineage or
/// selected sample position), depth or generation, and genome.
#[derive(Debug, Clone)]
pub struct CohortParent {
    pub index: u64,
    pub depth_or_generation: u64,
    pub genome: CreatureGenome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParentRow {
    pub index: u64,
    pub depth_or_generation: u64,
    pub genome_size: u32,
    pub total_nodes: u32,
    pub reachable_nodes: u32,
    pub executed_nodes: u32,
    pub contributing_nodes: u32,
    pub all_noop: bool,
    pub distinct_queues: u32,
    pub counts: EffectCounts,
}

/// One cohort's coverage-extension reading. `differ_by_group` is indexed by
/// [`Group::index`]; its recorded entry is zero when the world recorded no
/// context.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Coverage {
    pub parents_evaluated: u32,
    pub pairs_requested: u32,
    pub pairs_sampled: u32,
    pub differ_by_group: [u32; 4],
    /// Pairs whose actions differ in some group.
    pub differ_any: u32,
    /// Pairs with identical actions everywhere and a differing state-and-cost record.
    pub state_or_cost_only: u32,
    /// Parents all-`NoOp` on `neighborhood-v1`, and those that act somewhere
    /// on the extension.
    pub all_noop_parents: u32,
    pub all_noop_parents_acting: u32,
}

impl Coverage {
    /// Count one pair: each group whose actions differ, the pair once in the
    /// union when any does, and state-or-cost-only when none does but the
    /// state-and-cost record differs.
    fn record_pair(&mut self, differs_by_group: [bool; 4], state_differs: bool) {
        for (total, differs) in self.differ_by_group.iter_mut().zip(differs_by_group) {
            *total += u32::from(differs);
        }
        let any = differs_by_group.contains(&true);
        self.differ_any += u32::from(any);
        self.state_or_cost_only += u32::from(!any && state_differs);
    }

    #[must_use]
    fn merge(mut self, other: &Self) -> Self {
        self.parents_evaluated += other.parents_evaluated;
        self.pairs_requested += other.pairs_requested;
        self.pairs_sampled += other.pairs_sampled;
        for (total, count) in self.differ_by_group.iter_mut().zip(other.differ_by_group) {
            *total += count;
        }
        self.differ_any += other.differ_any;
        self.state_or_cost_only += other.state_or_cost_only;
        self.all_noop_parents += other.all_noop_parents;
        self.all_noop_parents_acting += other.all_noop_parents_acting;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CohortReading {
    pub cohort: Cohort,
    pub parents_requested: u32,
    pub parents: Vec<ParentRow>,
    pub totals: EffectCounts,
    pub operators: BTreeMap<OperatorKey, EffectCounts>,
    /// Indexed by [`TargetClass::index`].
    pub targets: [EffectCounts; 5],
    pub coverage: Coverage,
}

/// One control's outcome on this report's extension.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlOutcome {
    pub name: &'static str,
    pub expectation: &'static str,
    pub silent_on_original: bool,
    /// Action differences by [`Group::index`].
    pub differs_by_group: [bool; 4],
    pub state_differs: bool,
    pub passed: bool,
}

/// One world's complete reading.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reading {
    pub founder: CohortReading,
    pub drift: CohortReading,
    pub selected: Result<CohortReading, String>,
    pub recorded_requested: u32,
    pub recorded_contexts: Result<u32, String>,
    /// `"recorded"`, or `"authored"` when fewer than four contexts were recorded.
    pub sequence_source: &'static str,
    pub controls: Vec<ControlOutcome>,
}

/// Sample positions of the selected cohort: a uniform draw without
/// replacement of `min(requested, sample_size)` positions from
/// `SmallRng::seed_from_u64(SELECTED_SEED_BASE + world_seed)`, ascending.
#[must_use]
pub fn selected_positions(sample_size: usize, requested: usize, world_seed: u64) -> Vec<usize> {
    let amount = requested.min(sample_size);
    if amount == 0 {
        return Vec::new();
    }
    let mut rng = SmallRng::seed_from_u64(SELECTED_SEED_BASE.wrapping_add(world_seed));
    let mut positions = rand::seq::index::sample(&mut rng, sample_size, amount).into_vec();
    positions.sort_unstable();
    positions
}

/// The node ids whose genome differs between `parent` and `child`: added,
/// removed, or edited under [`genome_identity`], and both entry nodes when
/// the entry changed.
#[must_use]
pub fn edited_node_ids(parent: &CreatureGenome, child: &CreatureGenome) -> BTreeSet<NodeId> {
    let render = |genome: &CreatureGenome| -> BTreeMap<NodeId, String> {
        genome
            .nodes
            .iter()
            .map(|node| (node.node_id, format!("{node:?}")))
            .collect()
    };
    let (before, after) = (render(parent), render(child));
    let mut edited: BTreeSet<NodeId> = before
        .iter()
        .filter(|(id, node)| after.get(id) != Some(node))
        .map(|(id, _)| *id)
        .chain(after.keys().filter(|id| !before.contains_key(id)).copied())
        .collect();
    if parent.entry_node_id != child.entry_node_id {
        edited.insert(parent.entry_node_id);
        edited.insert(child.entry_node_id);
    }
    edited
}

/// A parent read once for attribution.
struct ParentTrace<'a> {
    genome: &'a CreatureGenome,
    identity: String,
    traced: TracedBattery,
    signature: Signature,
}

impl<'a> ParentTrace<'a> {
    fn new(genome: &'a CreatureGenome, battery: &Battery, context: &EvalContext) -> Self {
        let traced = TracedBattery::run(
            battery,
            genome,
            context.runtime,
            context.shared_memory_decay_rate,
        );
        let signature = traced.signature(battery);
        Self {
            genome,
            identity: genome_identity(genome),
            traced,
            signature,
        }
    }
}

/// Attribute one applied child against its parent, in category precedence.
fn attribute(
    parent: &ParentTrace<'_>,
    child: &CreatureGenome,
    battery: &Battery,
    context: &EvalContext,
) -> Attribution {
    // Identity ignores NaN payloads while the records keep bits, so an
    // identical child still runs: the independent state count reads it.
    let identical = genome_identity(child) == parent.identity;
    let traced = TracedBattery::run(
        battery,
        child,
        context.runtime,
        context.shared_memory_decay_rate,
    );
    let class = classify(&parent.signature, &traced.signature(battery)).class;
    let computation = parent.traced.computation_differs(&traced);
    let state = parent.traced.state_differs(&traced);
    let dispatched_edit = edited_node_ids(parent.genome, child)
        .iter()
        .any(|id| parent.traced.dispatched.contains(id) || traced.dispatched.contains(id));
    let category = match class {
        Class::Changed => Category::ActionChanged,
        Class::Dead => Category::ActionDead,
        Class::Silent if identical => Category::GenomeIdentical,
        Class::Silent if !dispatched_edit => Category::UnexecutedEdit,
        Class::Silent if computation => Category::MaskedBeforeSelection,
        Class::Silent if state => Category::StateOrCostOnly,
        Class::Silent => Category::Unresolved,
    };
    let action_silent = class == Class::Silent;
    Attribution {
        category,
        silent_with_state_or_cost: action_silent && state,
        consistency_violation: category == Category::UnexecutedEdit && (computation || state),
        action_silent,
    }
}

/// One proposal's outcome; `attribution` is `None` for a skipped proposal.
#[derive(Debug, Clone, Copy)]
struct ProposalOutcome {
    operator: OperatorKey,
    target: TargetClass,
    attribution: Option<Attribution>,
}

/// One evaluated parent: its row, its proposals' outcomes, and its first
/// coverage pairs in proposal order.
struct ParentEvaluation {
    row: ParentRow,
    outcomes: Vec<ProposalOutcome>,
    pairs: Vec<CreatureGenome>,
}

fn evaluate_parent(
    cohort: Cohort,
    position: usize,
    parent: &CohortParent,
    battery: &Battery,
    one_event: &MutationConfig,
    context: &EvalContext,
    sizes: Sizes,
) -> ParentEvaluation {
    let genome = &parent.genome;
    let decay = context.shared_memory_decay_rate;
    let trace = ParentTrace::new(genome, battery, context);
    let sets = battery.mesh_execution_sets(genome, context.runtime, decay);
    let reachable = mesh_reachable_nodes(genome);
    let reachable_ids: BTreeSet<NodeId> = reachable
        .iter()
        .map(|&index| genome.nodes[index].node_id)
        .collect();
    let executed = indices_for_node_ids(genome, &sets.executed);
    let target_class = |target: Option<NodeId>| match target {
        None => TargetClass::NoNodeTarget,
        Some(id) if sets.contributing.contains(&id) => TargetClass::KnockoutContributing,
        Some(id) if sets.executed.contains(&id) => TargetClass::ExecutedNotContributing,
        Some(id) if reachable_ids.contains(&id) => TargetClass::ReachableNotExecuted,
        Some(_) => TargetClass::Unreachable,
    };
    let seed_base = PROPOSAL_SEED_BASE
        + COHORT_SEED_MULTIPLIER * cohort.index()
        + PARENT_SEED_MULTIPLIER * (position as u64 + 1);

    let proposals: Vec<(ProposalOutcome, Option<CreatureGenome>)> = (0..sizes.proposals)
        .into_par_iter()
        .map(|proposal| {
            let mut child = genome.clone();
            let mut rng = SmallRng::seed_from_u64(seed_base + u64::from(proposal));
            let summary = MutationEngine::apply_mutations_on_units(
                &mut child,
                1,
                one_event,
                &reachable,
                ParentExecuted::Indices(&executed),
                &mut rng,
                context.food_type_count,
            );
            let event = summary
                .events
                .first()
                .expect("one unit at rate 1.0 requests exactly one event");
            let operator = event.operator.map_or(
                OperatorKey::DomainExhausted(event.domain),
                OperatorKey::Operator,
            );
            let attribution =
                (summary.applied_events > 0).then(|| attribute(&trace, &child, battery, context));
            let pair = attribution.and_then(|attribution| {
                (attribution.action_silent && attribution.category != Category::GenomeIdentical)
                    .then_some(child)
            });
            let outcome = ProposalOutcome {
                operator,
                target: target_class(event.target),
                attribution,
            };
            (outcome, pair)
        })
        .collect();

    let mut counts = EffectCounts::default();
    let mut outcomes = Vec::with_capacity(proposals.len());
    let mut pairs = Vec::new();
    for (outcome, pair) in proposals {
        counts.record(outcome.attribution);
        outcomes.push(outcome);
        if let Some(child) = pair {
            if pairs.len() < sizes.pairs_per_parent as usize {
                pairs.push(child);
            }
        }
    }
    ParentEvaluation {
        row: ParentRow {
            index: parent.index,
            depth_or_generation: parent.depth_or_generation,
            genome_size: genome.genome_size(),
            total_nodes: sets.reading.total_node_count as u32,
            reachable_nodes: sets.reading.reachable_node_count as u32,
            executed_nodes: sets.reading.executed_node_count as u32,
            contributing_nodes: sets.contributing.len() as u32,
            all_noop: trace.signature.all_noop(),
            distinct_queues: trace.signature.distinct_queue_count() as u32,
            counts,
        },
        outcomes,
        pairs,
    }
}

/// A genome's executions on the extension panel.
fn run_extension(
    genome: &CreatureGenome,
    extension: &Extension,
    context: &EvalContext,
) -> Vec<PanelExecution> {
    run_panel_executions(
        genome,
        &extension.singles,
        &extension.sequences,
        context.runtime,
        context.shared_memory_decay_rate,
    )
}

/// Per-group action differences and whether any state record differs.
fn compare_on_extension(
    parent: &[PanelExecution],
    child: &[PanelExecution],
    extension: &Extension,
) -> ([bool; 4], bool) {
    let mut differs = [false; 4];
    let mut state = false;
    for (index, (a, b)) in parent.iter().zip(child).enumerate() {
        differs[extension.group(index).index()] |= a.actions != b.actions;
        state |= a.state != b.state;
    }
    (differs, state)
}

fn acts(executions: &[PanelExecution]) -> bool {
    executions.iter().any(|execution| {
        execution
            .actions
            .iter()
            .any(|action| *action != WorldAction::NoOp)
    })
}

#[allow(
    clippy::too_many_arguments,
    reason = "the cohort, its parents and request, beside the shared panel and configs"
)]
fn observe_cohort(
    cohort: Cohort,
    parents: &[CohortParent],
    parents_requested: u32,
    extension: &Extension,
    battery: &Battery,
    one_event: &MutationConfig,
    context: &EvalContext,
    sizes: Sizes,
) -> CohortReading {
    let evaluations: Vec<(ParentEvaluation, Coverage)> = parents
        .par_iter()
        .enumerate()
        .map(|(position, parent)| {
            let evaluation =
                evaluate_parent(cohort, position, parent, battery, one_event, context, sizes);
            let coverage = parent_coverage(parent, &evaluation, extension, context, sizes);
            (evaluation, coverage)
        })
        .collect();

    let mut reading = CohortReading {
        cohort,
        parents_requested,
        parents: Vec::with_capacity(evaluations.len()),
        totals: EffectCounts::default(),
        operators: BTreeMap::new(),
        targets: [EffectCounts::default(); 5],
        coverage: Coverage::default(),
    };
    for (evaluation, coverage) in evaluations {
        reading.totals = reading.totals.merge(&evaluation.row.counts);
        for outcome in &evaluation.outcomes {
            reading
                .operators
                .entry(outcome.operator)
                .or_default()
                .record(outcome.attribution);
            reading.targets[outcome.target.index()].record(outcome.attribution);
        }
        reading.coverage = reading.coverage.merge(&coverage);
        reading.parents.push(evaluation.row);
    }
    reading
}

/// One parent's coverage row: its all-`NoOp` capability on both panels and
/// its sampled pairs re-read on the extension.
fn parent_coverage(
    parent: &CohortParent,
    evaluation: &ParentEvaluation,
    extension: &Extension,
    context: &EvalContext,
    sizes: Sizes,
) -> Coverage {
    let base = run_extension(&parent.genome, extension, context);
    let all_noop = evaluation.row.all_noop;
    let mut coverage = Coverage {
        parents_evaluated: 1,
        pairs_requested: sizes.pairs_per_parent,
        pairs_sampled: evaluation.pairs.len() as u32,
        all_noop_parents: u32::from(all_noop),
        all_noop_parents_acting: u32::from(all_noop && acts(&base)),
        ..Coverage::default()
    };
    for child in &evaluation.pairs {
        let (differs, state) =
            compare_on_extension(&base, &run_extension(child, extension, context), extension);
        coverage.record_pair(differs, state);
    }
    coverage
}

/// Run one control pair on `neighborhood-v1` and on the extension.
fn run_control(
    name: &'static str,
    expectation: &'static str,
    (parent, child): (CreatureGenome, CreatureGenome),
    expect_difference: bool,
    extension: &Extension,
    battery: &Battery,
    context: &EvalContext,
) -> ControlOutcome {
    let decay = context.shared_memory_decay_rate;
    let silent_on_original = classify(
        &battery.signature(&parent, context.runtime, decay),
        &battery.signature(&child, context.runtime, decay),
    )
    .class
        == Class::Silent;
    let (differs_by_group, state_differs) = compare_on_extension(
        &run_extension(&parent, extension, context),
        &run_extension(&child, extension, context),
        extension,
    );
    let differs = differs_by_group.contains(&true);
    let passed = if expect_difference {
        silent_on_original && differs
    } else {
        silent_on_original && !differs && !state_differs
    };
    ControlOutcome {
        name,
        expectation,
        silent_on_original,
        differs_by_group,
        state_differs,
        passed,
    }
}

fn run_controls(
    founder: &CreatureGenome,
    extension: &Extension,
    battery: &Battery,
    context: &EvalContext,
) -> Vec<ControlOutcome> {
    const DIFFERS: &str = "silent on neighborhood-v1, actions differ on the extension";
    let control = |name, expectation, pair, expect_difference| {
        run_control(
            name,
            expectation,
            pair,
            expect_difference,
            extension,
            battery,
            context,
        )
    };
    vec![
        control(
            "barrier_dependent_action_edit",
            DIFFERS,
            controls::barrier_pair(),
            true,
        ),
        control(
            "slow_integrator_after_tick_4",
            DIFFERS,
            controls::integrator_pair(),
            true,
        ),
        control(
            "same_genome",
            "no difference in any group",
            (founder.clone(), founder.clone()),
            false,
        ),
    ]
}

/// The inputs one world's reading needs beyond the battery and configs.
#[derive(Clone, Copy)]
pub struct WorldInputs<'a> {
    pub founder: &'a CreatureGenome,
    /// The drift walk's birth lineages at its last checkpoint.
    pub drift: &'a [CohortParent],
    /// The selected cohort, or why it is undefined.
    pub selected: Result<&'a [CohortParent], &'a str>,
    /// The world's terminal state, for the recorded contexts.
    pub sim: &'a Simulation,
    pub world_seed: u64,
}

/// One world's complete `mutation-effects-v1` and `neighborhood-coverage-v1`
/// reading.
#[must_use]
pub fn observe(
    inputs: WorldInputs<'_>,
    battery: &Battery,
    mutation: &MutationConfig,
    context: &EvalContext,
    sizes: Sizes,
) -> Reading {
    let one_event = MutationConfig {
        per_unit_rate: 1.0,
        ..mutation.clone()
    };
    let recorded = recorded_contexts(
        inputs.sim,
        inputs.world_seed,
        sizes.recorded_contexts as usize,
    );
    let recorded_contexts = if recorded.is_empty() {
        Err("extinct: no living creature at the terminal tick".to_string())
    } else {
        Ok(recorded.len() as u32)
    };
    let extension = Extension::new(recorded, battery);
    let cohort = |cohort, parents: &[CohortParent], requested| {
        observe_cohort(
            cohort, parents, requested, &extension, battery, &one_event, context, sizes,
        )
    };
    let founder = [CohortParent {
        index: 0,
        depth_or_generation: 0,
        genome: inputs.founder.clone(),
    }];
    Reading {
        founder: cohort(Cohort::Founder, &founder, 1),
        drift: cohort(Cohort::Drift, inputs.drift, inputs.drift.len() as u32),
        selected: inputs
            .selected
            .map(|parents| cohort(Cohort::Selected, parents, sizes.parents))
            .map_err(str::to_string),
        recorded_requested: sizes.recorded_contexts,
        recorded_contexts,
        sequence_source: extension.sequence_source(),
        controls: run_controls(inputs.founder, &extension, battery, context),
    }
}
