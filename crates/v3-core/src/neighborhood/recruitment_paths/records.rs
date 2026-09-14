use super::super::recruitment::{Module, ModuleBackend, Opportunities, RecruitmentCheckpoint};
use super::super::Signature;
use crate::contracts::{NodeId, Position, WorldAction};
use crate::creature::genome::{CreatureGenome, NodeGenome};
use crate::mutation::{MutationDomain, MutationEventRecord, MutationOperator};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Task {
    A,
    B,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Policy {
    Drift,
    Selection,
    /// Score first, then the summed ending energy the production charges leave.
    CostSelection,
}

impl Policy {
    /// The two T13.F02 policies; their eighteen arms lead the report.
    pub const F02: [Self; 2] = [Self::Drift, Self::Selection];
    pub const COUNT: usize = Self::F02.len() + 1;
}

/// Starting forms in the fixed family.
pub const STARTS: usize = 9;
/// Arms per observation: every starting form under every policy.
pub const ARMS: usize = STARTS * Policy::COUNT;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sizes {
    pub batches: u32,
    pub lineages: u32,
    pub discovery: u32,
    pub followup: u32,
}

impl Sizes {
    pub const PRODUCTION: Self = Self {
        batches: 4,
        lineages: 8,
        discovery: 32,
        followup: 16,
    };
    pub const TEST: Self = Self {
        batches: 1,
        lineages: 1,
        discovery: 2,
        followup: 1,
    };
    pub fn proposals(self) -> u64 {
        u64::from(self.batches)
            * u64::from(self.lineages)
            * u64::from(self.discovery + self.followup)
            * 2
            * ARMS as u64
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Work {
    pub mesh_hops: u64,
    pub vm_steps: u64,
    pub graph_visits: u64,
    pub plasticity: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Scene {
    pub food: [bool; 3],
    pub correct_a: bool,
    pub correct_b: bool,
    pub survived: bool,
    /// Absent after the production tick removes the subject.
    pub energy: Option<f32>,
    pub maintenance: f64,
    pub carrying: f64,
    pub work: Work,
    pub actions: Vec<WorldAction>,
    pub position: Option<Position>,
    pub dispatched: Vec<NodeId>,
    pub routing: Vec<(NodeId, NodeId)>,
    pub output_slots: Vec<(NodeId, [f32; 24])>,
    /// Observed zero is distinct from state unavailable after subject removal.
    pub shared_memory: Option<[f32; 16]>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskReading {
    pub scenes: Vec<Scene>,
}

impl TaskReading {
    pub(super) fn memory_effect(&self, control: &Self) -> Option<bool> {
        if self.scenes.len() != control.scenes.len() {
            return None;
        }
        self.scenes
            .iter()
            .zip(&control.scenes)
            .try_fold(false, |effect, (left, right)| {
                let differs = left.shared_memory? != right.shared_memory?;
                Some(effect || differs)
            })
    }

    pub fn live(&self) -> bool {
        self.scenes.iter().all(|scene| scene.survived)
    }
    pub fn correct(&self, task: Task) -> u8 {
        self.scenes
            .iter()
            .filter(|scene| match task {
                Task::A => scene.correct_a,
                Task::B => scene.correct_b,
            })
            .count() as u8
    }
    pub fn dispatched(&self) -> BTreeSet<NodeId> {
        self.scenes
            .iter()
            .flat_map(|scene| scene.dispatched.iter().copied())
            .collect()
    }
    pub fn summary(&self) -> TaskSummary {
        let mut work = Work::default();
        for scene in &self.scenes {
            work.mesh_hops += scene.work.mesh_hops;
            work.vm_steps += scene.work.vm_steps;
            work.graph_visits += scene.work.graph_visits;
            work.plasticity += scene.work.plasticity;
        }
        TaskSummary {
            correct_a: self.correct(Task::A),
            correct_b: self.correct(Task::B),
            surviving_scenes: self.scenes.iter().filter(|scene| scene.survived).count() as u8,
            ending_energy_sum: self
                .scenes
                .iter()
                .filter_map(|scene| scene.energy.map(f64::from))
                .sum(),
            maintenance_sum: self.scenes.iter().map(|scene| scene.maintenance).sum(),
            carrying_sum: self.scenes.iter().map(|scene| scene.carrying).sum(),
            work,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskSummary {
    pub correct_a: u8,
    pub correct_b: u8,
    pub surviving_scenes: u8,
    /// Sum of remaining subject states; read with `surviving_scenes`.
    pub ending_energy_sum: f64,
    pub maintenance_sum: f64,
    pub carrying_sum: f64,
    pub work: Work,
}

impl TaskSummary {
    /// Every scene survived the eight production ticks.
    #[must_use]
    pub fn live(&self) -> bool {
        self.surviving_scenes == 8
    }
    #[must_use]
    pub fn correct(&self, task: Task) -> u8 {
        match task {
            Task::A => self.correct_a,
            Task::B => self.correct_b,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeChange {
    pub node: NodeId,
    pub before: Option<NodeGenome>,
    pub after: Option<NodeGenome>,
}

/// Exact whole-birth resolution, including node order. Events do not attribute
/// individual fields when multiple events touch a node in the same birth.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenomeDelta {
    pub entry_before: NodeId,
    pub entry_after: NodeId,
    pub order_before: Vec<NodeId>,
    pub order_after: Vec<NodeId>,
    pub nodes: Vec<NodeChange>,
}

impl GenomeDelta {
    pub fn between(before: &CreatureGenome, after: &CreatureGenome) -> Self {
        let ids: BTreeSet<_> = before
            .nodes
            .iter()
            .chain(&after.nodes)
            .map(|node| node.node_id)
            .collect();
        let nodes = ids
            .into_iter()
            .filter_map(|id| {
                let left = before.nodes.iter().find(|node| node.node_id == id);
                let right = after.nodes.iter().find(|node| node.node_id == id);
                (left != right).then(|| NodeChange {
                    node: id,
                    before: left.cloned(),
                    after: right.cloned(),
                })
            })
            .collect();
        Self {
            entry_before: before.entry_node_id,
            entry_after: after.entry_node_id,
            order_before: before.nodes.iter().map(|node| node.node_id).collect(),
            order_after: after.nodes.iter().map(|node| node.node_id).collect(),
            nodes,
        }
    }

    /// Refuse a delta whose observed before-values do not match the replay.
    pub fn apply(&self, genome: &CreatureGenome) -> Option<CreatureGenome> {
        if genome.entry_node_id != self.entry_before
            || genome
                .nodes
                .iter()
                .map(|node| node.node_id)
                .collect::<Vec<_>>()
                != self.order_before
            || self.nodes.iter().any(|change| {
                genome.nodes.iter().find(|node| node.node_id == change.node)
                    != change.before.as_ref()
            })
        {
            return None;
        }
        let nodes: Option<Vec<_>> = self
            .order_after
            .iter()
            .map(|id| {
                self.nodes
                    .iter()
                    .find(|change| change.node == *id)
                    .map_or_else(
                        || genome.nodes.iter().find(|node| node.node_id == *id),
                        |change| change.after.as_ref(),
                    )
                    .cloned()
            })
            .collect();
        Some(CreatureGenome {
            entry_node_id: self.entry_after,
            nodes: nodes?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event {
    pub domain: MutationDomain,
    pub operator: Option<MutationOperator>,
    pub target: Option<NodeId>,
    pub outcome: String,
    pub discarded: Vec<(MutationOperator, Option<NodeId>)>,
}

impl From<&MutationEventRecord> for Event {
    fn from(event: &MutationEventRecord) -> Self {
        Self {
            domain: event.domain,
            operator: event.operator,
            target: event.target,
            outcome: format!("{:?}", event.outcome),
            discarded: event.discarded.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleUse {
    pub node: NodeId,
    pub created_depth: u64,
    pub created_backend: ModuleBackend,
    pub current_backend: ModuleBackend,
    pub dispatched: bool,
    pub score_loss: i16,
    pub queue_effect: bool,
    /// Unmeasured if either paired subject lacks a required memory observation.
    pub memory_effect: Option<bool>,
    pub output_effect: bool,
    pub routing_effect: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub generation: u32,
    pub sibling: u8,
    pub seed: u64,
    pub parent_live: bool,
    pub parent_score: u8,
    pub chosen: bool,
    pub outcome: TaskSummary,
    pub modules: usize,
    pub genome_size: u32,
    pub opportunities: Opportunities,
    /// Selected-but-inapplicable discards; backend comes from the event's domain,
    /// not from the possibly deleted or replaced post-birth target.
    pub events: Vec<Event>,
    pub useful_modules: Vec<ModuleUse>,
    pub discovery: bool,
    pub viable_path: bool,
    pub mutation_fingerprint: String,
    pub parent_fingerprint: String,
    pub rng_after: u64,
    pub selected_inapplicable_by_backend_operator: std::collections::BTreeMap<
        ModuleBackend,
        std::collections::BTreeMap<MutationOperator, u64>,
    >,
    pub selected_inapplicable_backend_unresolved: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Discovery {
    pub generation: u32,
    pub sibling: u8,
    pub module: ModuleUse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetentionOutcome {
    TaskDead,
    Deleted,
    Useful,
    NoLongerUseful,
}

pub fn classify_retention(live: bool, score_loss: Option<i16>) -> RetentionOutcome {
    match (live, score_loss) {
        (false, _) => RetentionOutcome::TaskDead,
        (true, None) => RetentionOutcome::Deleted,
        (true, Some(loss)) if loss >= 1 => RetentionOutcome::Useful,
        (true, Some(_)) => RetentionOutcome::NoLongerUseful,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Retention {
    pub discovery: Discovery,
    pub at_generation: u32,
    pub outcome: RetentionOutcome,
    pub score: u8,
    pub score_loss: Option<i16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayStep {
    pub generation: u32,
    pub sibling: Option<u8>,
    pub seed: Option<u64>,
    pub retained: bool,
    pub events: Vec<Event>,
    pub delta: GenomeDelta,
    pub outcome: TaskReading,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub generation: u32,
    pub genome: CreatureGenome,
    pub task: TaskReading,
    pub battery: Signature,
    pub battery_class: String,
    pub cohort: RecruitmentCheckpoint,
    pub modules: Vec<Module>,
    pub task_use: Vec<ModuleUse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lineage {
    pub batch: u32,
    pub lineage: u32,
    pub proposals: Vec<Proposal>,
    pub checkpoints: Vec<Checkpoint>,
    pub proposal_discovery: Option<Discovery>,
    pub retained_discovery: Option<Discovery>,
    pub viable_retained_discovery: bool,
    pub retention: Option<Retention>,
    pub first_successful_path: Vec<ReplayStep>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Estimate {
    pub numerator: u32,
    pub denominator: u32,
    pub fraction: Option<f64>,
    pub wilson_95: Option<[f64; 2]>,
}

pub fn estimate(numerator: u32, denominator: u32) -> Estimate {
    assert!(numerator <= denominator);
    let interval = (denominator > 0).then(|| {
        let n = f64::from(denominator);
        let p = f64::from(numerator) / n;
        let z2 = 1.959_963_984_540_054_f64.powi(2);
        let centre = (p + z2 / (2.0 * n)) / (1.0 + z2 / n);
        let half = (z2 * (p * (1.0 - p) / n + z2 / (4.0 * n * n))).sqrt() / (1.0 + z2 / n);
        [(centre - half).max(0.0), (centre + half).min(1.0)]
    });
    Estimate {
        numerator,
        denominator,
        fraction: (denominator > 0).then(|| f64::from(numerator) / f64::from(denominator)),
        wilson_95: interval,
    }
}

/// Median of a non-empty sorted sample; an even count averages the two
/// middle values.
fn median<T: Copy + Into<f64>>(sorted: &[T]) -> Option<f64> {
    let middle = sorted.len() / 2;
    match sorted.len() {
        0 => None,
        len if len % 2 == 1 => Some(sorted[middle].into()),
        _ => Some((sorted[middle - 1].into() + sorted[middle].into()) / 2.0),
    }
}

/// Time to first discovery over an arm's lineages, censored at the discovery
/// horizon for lineages that never discover.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimeToFirst {
    /// Sorted first-discovery generations of the discovering lineages.
    pub generations: Vec<u32>,
    pub median: Option<f64>,
    /// Lineages without a discovery by the discovery horizon.
    pub censored: u32,
}

impl TimeToFirst {
    #[must_use]
    pub fn of(first: impl IntoIterator<Item = Option<u32>>) -> Self {
        let mut censored = 0;
        let mut generations = Vec::new();
        for generation in first {
            match generation {
                Some(generation) => generations.push(generation),
                None => censored += 1,
            }
        }
        generations.sort_unstable();
        Self {
            median: median(&generations),
            generations,
            censored,
        }
    }
}

/// Retention outcomes at discovery + follow-up among retained discoverers.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetentionOutcomes {
    pub useful: u32,
    pub no_longer_useful: u32,
    pub deleted: u32,
    pub task_dead: u32,
}

impl RetentionOutcomes {
    pub fn record(&mut self, outcome: RetentionOutcome) {
        match outcome {
            RetentionOutcome::Useful => self.useful += 1,
            RetentionOutcome::NoLongerUseful => self.no_longer_useful += 1,
            RetentionOutcome::Deleted => self.deleted += 1,
            RetentionOutcome::TaskDead => self.task_dead += 1,
        }
    }
    #[must_use]
    pub fn total(&self) -> u32 {
        self.useful + self.no_longer_useful + self.deleted + self.task_dead
    }
}

/// Proposal damage: task death over all proposals, and a loss of at least the
/// 1/8 margin against the parent over task-live proposals.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Damage {
    pub task_dead: Estimate,
    pub task_live_loss: Estimate,
}

/// Min / median / max over an arm's lineages.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Spread {
    pub min: f64,
    pub median: f64,
    pub max: f64,
}

impl Spread {
    #[must_use]
    pub fn of(mut values: Vec<f64>) -> Option<Self> {
        values.sort_unstable_by(f64::total_cmp);
        Some(Self {
            min: *values.first()?,
            median: median(&values)?,
            max: *values.last()?,
        })
    }
}

/// Cost of the retained parent at one checkpoint generation. `modules` is the
/// genome's node count, the same count `Proposal::modules` records.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckpointCost {
    pub generation: u32,
    pub genome_size: Spread,
    pub modules: Spread,
    pub carrying_sum: Spread,
    pub ending_energy_sum: Spread,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    pub proposal_discovery: Estimate,
    pub retained_discovery: Estimate,
    pub viable_retained_discovery: Estimate,
    pub retained_useful: Estimate,
    pub retention_among_discoverers: Estimate,
    pub discovery_depth_range: Option<[u32; 2]>,
    pub time_to_first_retained: TimeToFirst,
    pub time_to_first_proposal: TimeToFirst,
    pub retention_outcomes: RetentionOutcomes,
    pub damage: Damage,
    pub checkpoint_cost: Vec<CheckpointCost>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Arm {
    pub start: String,
    pub task: Task,
    pub policy: Policy,
    pub lineages: Vec<Lineage>,
    pub summary: Summary,
    pub batches: Vec<Summary>,
    pub opportunities: Opportunities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairedLineage {
    pub batch: u32,
    pub lineage: u32,
    pub proposal_discovery_difference: i8,
    pub retained_discovery_difference: i8,
    pub useful_retention_difference: i8,
    pub first_parent_divergence: Option<u32>,
    pub first_mutation_divergence: Option<(u32, u8)>,
    pub first_rng_divergence: Option<(u32, u8)>,
    pub matched_proposals_before_divergence: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pair {
    pub left_arm: usize,
    pub right_arm: usize,
    pub lineages: Vec<PairedLineage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub version: String,
    pub config: crate::config::SimulationConfig,
    pub config_digest: String,
    pub sizes: Sizes,
    pub task_definition: String,
    pub mutation_context: String,
    pub construction_resolution: String,
    pub observation_resolution: String,
    pub rng_control: String,
    pub limitations: Vec<String>,
    pub constructed: Vec<super::ConstructedPath>,
    pub starts: Vec<super::Start>,
    pub arms: Vec<Arm>,
    pub pairs: Vec<Pair>,
    pub total_proposals: u64,
    pub opportunities: Opportunities,
}

/// One selection candidate: task-live, exact correct-scene count on the arm's
/// task, and the energy the eight production ticks left
/// (`TaskSummary::ending_energy_sum`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Candidate {
    pub live: bool,
    pub score: u8,
    pub ending_energy_sum: f64,
}

impl Candidate {
    #[must_use]
    pub fn new(reading: &TaskReading, task: Task) -> Self {
        Self {
            live: reading.live(),
            score: reading.correct(task),
            ending_energy_sum: reading.summary().ending_energy_sum,
        }
    }
}

/// Sibling preference is explicit; selection's score comparison uses exact
/// correct-scene counts, so one point is the predeclared 1/8 margin.
/// `Selection` never reads energy. `CostSelection` orders by score, then by
/// strictly higher ending energy, then sibling 0, sibling 1, parent, so a
/// neutral child that carries or runs more than its parent is not retained.
pub fn choose(policy: Policy, parent: Candidate, children: [Candidate; 2]) -> Option<usize> {
    if policy == Policy::Drift {
        return Some(0);
    }
    // Candidates in tie order: sibling 0, sibling 1, then the parent (index 2).
    let mut best: Option<(usize, Candidate)> = None;
    for (index, candidate) in children.into_iter().chain([parent]).enumerate() {
        if index < 2 && (!candidate.live || candidate.score < parent.score) {
            continue;
        }
        let beats = best.is_none_or(|(_, current)| {
            candidate.score > current.score
                || (policy == Policy::CostSelection
                    && candidate.score == current.score
                    && candidate.ending_energy_sum > current.ending_energy_sum)
        });
        if beats {
            best = Some((index, candidate));
        }
    }
    best.and_then(|(index, _)| (index < 2).then_some(index))
}
