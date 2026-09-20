use super::super::recruitment::{Module, ModuleBackend, Opportunities, RecruitmentCheckpoint};
use super::super::Signature;
use crate::contracts::{NodeId, Position, WorldAction};
use crate::creature::genome::cgp::{GraphSource, NodeClass};
use crate::creature::genome::{BackendDef, CreatureGenome, NodeGenome};
use crate::mutation::{MutationDomain, MutationEventRecord, MutationOperator};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

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
    /// The T13.F07 S0 panel: 1,769,472 proposals under production supply.
    pub const S0: Self = Self {
        batches: 4,
        lineages: 16,
        discovery: 256,
        followup: 256,
    };
    /// The S0 feasibility pilot: batch 0, lineages 0–1, every arm; its
    /// lineages are a prefix of [`Sizes::S0`] and reproduce byte-identically
    /// inside it.
    pub const S0_PILOT: Self = Self {
        batches: 1,
        lineages: 2,
        discovery: 256,
        followup: 256,
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
    /// Whether every dimension fits within `cap`.
    #[must_use]
    pub fn fits(self, cap: Self) -> bool {
        self.batches > 0
            && self.lineages > 0
            && self.discovery > 0
            && self.followup > 0
            && self.batches <= cap.batches
            && self.lineages <= cap.lineages
            && self.discovery <= cap.discovery
            && self.followup <= cap.followup
    }
}

/// Which mutation supply rule the proposals draw under.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Supply {
    /// `MutationConfig::default().with_legacy_supply()`: the fixed per-birth
    /// count the recorded T13.F02/F06 baselines were taken on.
    Legacy,
    /// `MutationConfig::default()`: the per-unit draw on the child's own
    /// `genome_size()` production runs.
    Production,
}

impl Supply {
    #[must_use]
    pub fn mutation_config(self) -> crate::config::MutationConfig {
        let config = crate::config::MutationConfig::default();
        match self {
            Self::Legacy => config.with_legacy_supply(),
            Self::Production => config,
        }
    }

    /// The `supply_rule` string a record carries.
    #[must_use]
    pub const fn rule(self) -> &'static str {
        match self {
            Self::Legacy => "legacy per-birth supply (per_unit_supply_enabled forced false)",
            Self::Production => "production per-unit supply on the child's own genome_size()",
        }
    }
}

/// One observation panel: its supply rule and sizes, checked against the
/// panel's fixed cap at construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Panel {
    pub supply: Supply,
    pub sizes: Sizes,
}

impl Panel {
    /// The fixed legacy panel (T13.F06's 82,944 proposals at production
    /// sizes); the sizes may not exceed [`Sizes::PRODUCTION`].
    ///
    /// # Panics
    /// When `sizes` has a zero dimension or exceeds the legacy cap.
    #[must_use]
    pub fn legacy(sizes: Sizes) -> Self {
        assert!(
            sizes.fits(Sizes::PRODUCTION),
            "legacy panel sizes exceed {:?}: {sizes:?}",
            Sizes::PRODUCTION
        );
        Self {
            supply: Supply::Legacy,
            sizes,
        }
    }

    /// The S0 panel under production supply; the sizes may not exceed
    /// [`Sizes::S0`].
    ///
    /// # Panics
    /// When `sizes` has a zero dimension or exceeds the S0 cap.
    #[must_use]
    pub fn s0(sizes: Sizes) -> Self {
        assert!(
            sizes.fits(Sizes::S0),
            "S0 panel sizes exceed {:?}: {sizes:?}",
            Sizes::S0
        );
        Self {
            supply: Supply::Production,
            sizes,
        }
    }

    /// The record version this panel writes.
    #[must_use]
    pub const fn version(self) -> &'static str {
        match self.supply {
            Supply::Legacy => super::VERSION,
            Supply::Production => S0_VERSION,
        }
    }
}

/// Version of the S0 panel's compact record.
pub const S0_VERSION: &str = "recruitment-transitions-s0-v1";

/// Which retention horizons (generations after the specialized retained
/// discovery) the assay reads; the primary is 64.
pub const HORIZONS: [u32; 3] = [16, 64, 256];
pub const PRIMARY_HORIZON: u32 = 64;

/// What one module's payload reads as under T13.F08's lens, read only: VM,
/// or a graph that is stateful (a stateful compute kind, a self/forward
/// compute reference, a previous-tick memory read, or plasticity), pure
/// without any wired effect surface, or pure with one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DestinationKind {
    Vm,
    GraphStateful,
    GraphPureNoEffect,
    GraphPureWithEffect,
}

impl DestinationKind {
    #[must_use]
    pub fn of(payload: &BackendDef) -> Self {
        let BackendDef::Graph(graph) = payload else {
            return Self::Vm;
        };
        let previous_memory = |source: &GraphSource| {
            matches!(source, GraphSource::SharedMemory { previous: true, .. })
        };
        let stateful = graph.compute_nodes.iter().enumerate().any(|(index, node)| {
            node.kind.class() == NodeClass::Stateful
                || node.plasticity.is_some()
                || node.inputs.iter().any(|edge| match edge.source {
                    GraphSource::ComputeNode(source) => usize::from(source) >= index,
                    ref source => previous_memory(source),
                })
        }) || graph
            .output_sinks
            .iter()
            .flat_map(|sink| &sink.inputs)
            .chain(graph.action_bank.iter().flat_map(|slot| slot.edges()))
            .chain(&graph.execute_gate.inputs)
            .any(|edge| previous_memory(&edge.source));
        if stateful {
            return Self::GraphStateful;
        }
        let effect = graph.action_bank.iter().any(|slot| slot.is_wired())
            || !graph.execute_gate.inputs.is_empty()
            || graph
                .output_sinks
                .iter()
                .any(|sink| !sink.inputs.is_empty());
        if effect {
            Self::GraphPureWithEffect
        } else {
            Self::GraphPureNoEffect
        }
    }
}

/// Cohort modules by destination kind at one checkpoint.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DestinationKindCounts {
    pub vm: u64,
    pub graph_stateful: u64,
    pub graph_pure_no_effect: u64,
    pub graph_pure_with_effect: u64,
}

impl DestinationKindCounts {
    pub fn record(&mut self, kind: DestinationKind) {
        match kind {
            DestinationKind::Vm => self.vm += 1,
            DestinationKind::GraphStateful => self.graph_stateful += 1,
            DestinationKind::GraphPureNoEffect => self.graph_pure_no_effect += 1,
            DestinationKind::GraphPureWithEffect => self.graph_pure_with_effect += 1,
        }
    }
    pub fn merge(&mut self, other: Self) {
        self.vm += other.vm;
        self.graph_stateful += other.graph_stateful;
        self.graph_pure_no_effect += other.graph_pure_no_effect;
        self.graph_pure_with_effect += other.graph_pure_with_effect;
    }
    #[must_use]
    pub const fn total(self) -> u64 {
        self.vm + self.graph_stateful + self.graph_pure_no_effect + self.graph_pure_with_effect
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
    /// Applied routes: the routing node, the winning target position and the
    /// node it named.
    pub routing: Vec<(NodeId, usize, NodeId)>,
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
    /// How many scenes dispatched each node (0..=8).
    pub fn scenes_dispatched(&self) -> BTreeMap<NodeId, u8> {
        let mut counts = BTreeMap::new();
        for scene in &self.scenes {
            let once: BTreeSet<_> = scene.dispatched.iter().copied().collect();
            for node in once {
                *counts.entry(node).or_insert(0u8) += 1;
            }
        }
        counts
    }
    /// Whether some node applied two different target positions across the
    /// eight scenes (the T11.F14 reading on the task scenes).
    pub fn route_position_varies(&self) -> bool {
        self.route_varies(|&(_, position, _)| position)
    }
    /// Whether some node applied routes to two different nodes across the
    /// eight scenes (T13.F07).
    pub fn route_destination_varies(&self) -> bool {
        self.route_varies(|&(_, _, destination)| destination)
    }
    fn route_varies<T: Ord>(&self, key: impl Fn(&(NodeId, usize, NodeId)) -> T) -> bool {
        let snapshots: Vec<BTreeMap<NodeId, BTreeSet<T>>> = self
            .scenes
            .iter()
            .map(|scene| {
                let mut routes: BTreeMap<NodeId, BTreeSet<T>> = BTreeMap::new();
                for route in &scene.routing {
                    routes.entry(route.0).or_default().insert(key(route));
                }
                routes
            })
            .collect();
        super::super::mesh_execution::route_varies_with_input(&snapshots)
    }
    /// Whether every scene `baseline` got right on `task` is still right here.
    pub fn preserves_correct_scenes(&self, baseline: &Self, task: Task) -> bool {
        baseline
            .scenes
            .iter()
            .zip(&self.scenes)
            .all(|(before, after)| {
                let correct = |scene: &Scene| match task {
                    Task::A => scene.correct_a,
                    Task::B => scene.correct_b,
                };
                !correct(before) || correct(after)
            })
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

/// The five specialization components (T13.F07), stored separately so a
/// reader sees which failed. `holds` is their conjunction.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Specialization {
    pub task_live: bool,
    /// Score at least the starting score plus one.
    pub score_gain: bool,
    /// Static-successor bypass loses at least one scene.
    pub bypass_loss: bool,
    /// Replacing the payload with its birth payload on the same route loses
    /// at least one scene; copied or prepared computation alone never scores.
    pub ancestral_loss: bool,
    /// Every scene correct at generation 0 is still correct.
    pub incumbents_preserved: bool,
}

impl Specialization {
    #[must_use]
    pub const fn holds(self) -> bool {
        self.task_live
            && self.score_gain
            && self.bypass_loss
            && self.ancestral_loss
            && self.incumbents_preserved
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    /// Whether the current payload differs from the birth payload.
    pub payload_changed: bool,
    /// score(current) − score(birth payload on the same route); `Some(0)`
    /// without evaluation for an unchanged payload, `None` when the module
    /// was not dispatched or its bypass loss is below one.
    pub ancestral_loss: Option<i16>,
    pub current_ending_energy_sum: f64,
    pub ancestral_ending_energy_sum: Option<f64>,
    pub destination_kind: DestinationKind,
    pub specialization: Specialization,
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
    /// Some cohort module meets every specialization component (T13.F07).
    pub specialized: bool,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Discovery {
    pub generation: u32,
    pub sibling: u8,
    pub module: ModuleUse,
}

/// What became of the specialized recruit at one retention horizon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HorizonOutcome {
    Retained,
    TaskDead,
    Deleted,
    NoLongerUseful,
    Despecialized,
}

impl HorizonOutcome {
    /// Read the recruit at a horizon: task death first, then absence, then
    /// the bypass loss, then the remaining components.
    #[must_use]
    pub fn of(live: bool, module: Option<&ModuleUse>) -> Self {
        match module {
            _ if !live => Self::TaskDead,
            None => Self::Deleted,
            Some(module) if module.score_loss < 1 => Self::NoLongerUseful,
            Some(module) if !module.specialization.holds() => Self::Despecialized,
            Some(_) => Self::Retained,
        }
    }
}

/// The specialized recruit re-read `offset` generations after its retained
/// discovery.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Horizon {
    pub offset: u32,
    pub at_generation: u32,
    pub outcome: HorizonOutcome,
    pub live: bool,
    pub score: u8,
    pub module: Option<ModuleUse>,
}

/// The ladder stages one lineage reached on its retained chain.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ladder {
    /// Some cohort module was statically reachable from the entry node.
    pub eligibility: bool,
    /// Some applied event targeted a reachable cohort module.
    pub local_edit: bool,
    /// Some cohort module dispatched in at least one scene.
    pub expression: bool,
    /// A specialized recruit was retained within the discovery horizon.
    pub specialized: bool,
    /// Some proposal (retained or not) carried a specialized recruit.
    pub proposal_specialized: bool,
    /// Some retained proposal had a bypass loss of one or more with an
    /// ancestral loss of zero.
    pub bypass_only: bool,
    /// The recruit's reading at the primary horizon, when observable.
    pub at_primary_horizon: Option<HorizonOutcome>,
}

/// Why a lineage produced no retained specialized recruit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LossKind {
    /// A specialized proposal existed but none was retained.
    NotSelected,
    Deleted,
    Despecialized,
    TaskDead,
    NoLongerUseful,
}

/// One lineage's exclusive classification: retained, the first failing
/// ladder stage, or the loss kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "class", content = "loss", rename_all = "snake_case")]
pub enum LineageClass {
    Retained,
    NoEligibility,
    NoEdit,
    NoExpression,
    NoBenefit,
    Loss(LossKind),
    /// The primary horizon lies beyond the run (the legacy panel).
    CensoredAt64,
}

impl LineageClass {
    /// The first failing stage of the ladder, in order.
    #[must_use]
    pub fn of(ladder: Ladder) -> Self {
        if !ladder.eligibility {
            Self::NoEligibility
        } else if !ladder.local_edit {
            Self::NoEdit
        } else if !ladder.expression {
            Self::NoExpression
        } else if !ladder.specialized {
            if ladder.proposal_specialized {
                Self::Loss(LossKind::NotSelected)
            } else {
                Self::NoBenefit
            }
        } else {
            match ladder.at_primary_horizon {
                None => Self::CensoredAt64,
                Some(HorizonOutcome::Retained) => Self::Retained,
                Some(HorizonOutcome::TaskDead) => Self::Loss(LossKind::TaskDead),
                Some(HorizonOutcome::Deleted) => Self::Loss(LossKind::Deleted),
                Some(HorizonOutcome::NoLongerUseful) => Self::Loss(LossKind::NoLongerUseful),
                Some(HorizonOutcome::Despecialized) => Self::Loss(LossKind::Despecialized),
            }
        }
    }
}

/// Per-arm ladder and classification counts (T13.F07).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Transitions {
    pub lineages: u32,
    pub eligibility: u32,
    pub local_edit: u32,
    pub expression: u32,
    pub specialized: u32,
    pub proposal_specialized_lineages: u32,
    /// Proposals (both siblings, every generation) carrying a specialized
    /// recruit.
    pub specialized_proposals: u64,
    pub retained_at: BTreeMap<u32, u32>,
    pub classes: BTreeMap<String, u32>,
    pub bypass_only: u32,
    /// Cohort modules with an applied site over cohort modules, pooled at
    /// the final checkpoint.
    pub eligible_site_fraction: Estimate,
    pub destination_kinds: DestinationKindCounts,
}

/// The key a class is counted under.
#[must_use]
pub fn class_key(class: LineageClass) -> String {
    match class {
        LineageClass::Retained => "retained".into(),
        LineageClass::NoEligibility => "no_eligibility".into(),
        LineageClass::NoEdit => "no_edit".into(),
        LineageClass::NoExpression => "no_expression".into(),
        LineageClass::NoBenefit => "no_benefit".into(),
        LineageClass::Loss(kind) => format!(
            "loss_{}",
            match kind {
                LossKind::NotSelected => "not_selected",
                LossKind::Deleted => "deleted",
                LossKind::Despecialized => "despecialized",
                LossKind::TaskDead => "task_dead",
                LossKind::NoLongerUseful => "no_longer_useful",
            }
        ),
        LineageClass::CensoredAt64 => "censored_at_64".into(),
    }
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

/// One tracker module with its birth payload, stored whole only for modules
/// that reached `Dispatch`; every module carries the payload hash.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleRecord {
    pub module: Module,
    pub birth_payload: Option<BackendDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub generation: u32,
    pub genome: CreatureGenome,
    pub task: TaskReading,
    pub battery: Signature,
    pub battery_class: String,
    pub cohort: RecruitmentCheckpoint,
    pub modules: Vec<ModuleRecord>,
    pub task_use: Vec<ModuleUse>,
    pub route_position_varies: bool,
    pub route_destination_varies: bool,
    pub destination_kinds: DestinationKindCounts,
    pub eligible_site_fraction: Estimate,
}

impl Checkpoint {
    /// The retained-parent cost scalars the arm summary spreads.
    #[must_use]
    pub fn cost(&self) -> CheckpointScalars {
        let summary = self.task.summary();
        CheckpointScalars {
            generation: self.generation,
            genome_size: f64::from(self.genome.genome_size()),
            modules: self.genome.nodes.len() as f64,
            carrying_sum: summary.carrying_sum,
            ending_energy_sum: summary.ending_energy_sum,
        }
    }
}

/// The scalars of one checkpoint an arm summary needs.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CheckpointScalars {
    pub generation: u32,
    pub genome_size: f64,
    pub modules: f64,
    pub carrying_sum: f64,
    pub ending_energy_sum: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lineage {
    pub batch: u32,
    pub lineage: u32,
    pub task: Task,
    pub proposals: Vec<Proposal>,
    pub checkpoints: Vec<Checkpoint>,
    pub proposal_discovery: Option<Discovery>,
    pub retained_discovery: Option<Discovery>,
    pub viable_retained_discovery: bool,
    pub retention: Option<Retention>,
    pub first_successful_path: Vec<ReplayStep>,
    /// First retained proposal within the discovery horizon carrying a
    /// specialized recruit (T13.F07).
    pub specialized_discovery: Option<Discovery>,
    /// The retained chain at the specialized discovery generation.
    pub discovery_checkpoint: Option<Checkpoint>,
    /// The recruit re-read at each observable [`HORIZONS`] offset.
    pub horizons: Vec<Horizon>,
    pub ladder: Ladder,
    pub classification: LineageClass,
}

/// One proposal as the compact S0 record keeps it: enough to reconstruct the
/// child from the initial genome, the chosen chain and the seed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompactProposal {
    pub generation: u32,
    pub sibling: u8,
    pub seed: u64,
    pub chosen: bool,
    pub live: bool,
    pub score: u8,
    pub parent_score: u8,
    pub discovery: bool,
    pub specialized: bool,
    /// Applied events only: operator, target, outcome.
    pub events: Vec<(MutationOperator, Option<NodeId>, String)>,
    pub mutation_fingerprint: String,
    /// The chosen child's whole-birth delta; absent for the sibling not taken.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delta: Option<GenomeDelta>,
}

/// One lineage of the compact S0 record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactLineage {
    pub arm: usize,
    pub start: String,
    pub policy: Policy,
    pub batch: u32,
    pub lineage: u32,
    pub proposals: Vec<CompactProposal>,
    pub checkpoints: Vec<Checkpoint>,
    pub discovery_checkpoint: Option<Checkpoint>,
    pub proposal_discovery: Option<Discovery>,
    pub retained_discovery: Option<Discovery>,
    pub viable_retained_discovery: bool,
    pub specialized_discovery: Option<Discovery>,
    pub retention: Option<Retention>,
    pub horizons: Vec<Horizon>,
    pub ladder: Ladder,
    pub classification: LineageClass,
    /// Opportunities pooled over the lineage's proposals.
    pub opportunities: Opportunities,
}

impl CompactLineage {
    #[must_use]
    pub fn of(arm: usize, start: &str, policy: Policy, lineage: &Lineage) -> Self {
        let mut opportunities = Opportunities::default();
        let proposals = lineage
            .proposals
            .iter()
            .map(|proposal| {
                opportunities.merge(&proposal.opportunities);
                CompactProposal {
                    generation: proposal.generation,
                    sibling: proposal.sibling,
                    seed: proposal.seed,
                    chosen: proposal.chosen,
                    live: proposal.outcome.live(),
                    score: proposal.outcome.correct(lineage.task),
                    parent_score: proposal.parent_score,
                    discovery: proposal.discovery,
                    specialized: proposal.specialized,
                    events: proposal
                        .events
                        .iter()
                        .filter_map(|event| {
                            event
                                .operator
                                .filter(|_| event.outcome.starts_with("Applied"))
                                .map(|operator| (operator, event.target, event.outcome.clone()))
                        })
                        .collect(),
                    mutation_fingerprint: proposal.mutation_fingerprint.clone(),
                    delta: None,
                }
            })
            .collect();
        Self {
            arm,
            start: start.into(),
            policy,
            batch: lineage.batch,
            lineage: lineage.lineage,
            proposals,
            checkpoints: lineage.checkpoints.clone(),
            discovery_checkpoint: lineage.discovery_checkpoint.clone(),
            proposal_discovery: lineage.proposal_discovery.clone(),
            retained_discovery: lineage.retained_discovery.clone(),
            viable_retained_discovery: lineage.viable_retained_discovery,
            specialized_discovery: lineage.specialized_discovery.clone(),
            retention: lineage.retention.clone(),
            horizons: lineage.horizons.clone(),
            ladder: lineage.ladder,
            classification: lineage.classification,
            opportunities,
        }
    }
}

/// The per-lineage facts an arm summary folds; built from a full or a
/// compact lineage so both panels summarize through one path.
#[derive(Debug, Clone, PartialEq)]
pub struct LineageFacts {
    pub batch: u32,
    pub lineage: u32,
    /// `(live, score, parent_score)` per proposal.
    pub proposals: Vec<(bool, u8, u8)>,
    pub specialized_proposals: u64,
    pub checkpoints: Vec<CheckpointScalars>,
    pub proposal_discovery: Option<u32>,
    pub retained_discovery: Option<u32>,
    pub specialized_discovery: Option<u32>,
    pub viable_retained_discovery: bool,
    pub retention: Option<RetentionOutcome>,
    pub horizons: Vec<(u32, HorizonOutcome)>,
    pub ladder: Ladder,
    pub classification: LineageClass,
    pub final_applicable: (u64, u64),
    pub destination_kinds: DestinationKindCounts,
}

/// The retained-chain fields a full and a compact lineage share, borrowed
/// so both fold into [`LineageFacts`] through one path.
struct ChainFacts<'a> {
    batch: u32,
    lineage: u32,
    checkpoints: &'a [Checkpoint],
    proposal_discovery: Option<&'a Discovery>,
    retained_discovery: Option<&'a Discovery>,
    specialized_discovery: Option<&'a Discovery>,
    viable_retained_discovery: bool,
    retention: Option<&'a Retention>,
    horizons: &'a [Horizon],
    ladder: Ladder,
    classification: LineageClass,
}

impl Lineage {
    fn chain(&self) -> ChainFacts<'_> {
        ChainFacts {
            batch: self.batch,
            lineage: self.lineage,
            checkpoints: &self.checkpoints,
            proposal_discovery: self.proposal_discovery.as_ref(),
            retained_discovery: self.retained_discovery.as_ref(),
            specialized_discovery: self.specialized_discovery.as_ref(),
            viable_retained_discovery: self.viable_retained_discovery,
            retention: self.retention.as_ref(),
            horizons: &self.horizons,
            ladder: self.ladder,
            classification: self.classification,
        }
    }
}

impl CompactLineage {
    fn chain(&self) -> ChainFacts<'_> {
        ChainFacts {
            batch: self.batch,
            lineage: self.lineage,
            checkpoints: &self.checkpoints,
            proposal_discovery: self.proposal_discovery.as_ref(),
            retained_discovery: self.retained_discovery.as_ref(),
            specialized_discovery: self.specialized_discovery.as_ref(),
            viable_retained_discovery: self.viable_retained_discovery,
            retention: self.retention.as_ref(),
            horizons: &self.horizons,
            ladder: self.ladder,
            classification: self.classification,
        }
    }
}

impl LineageFacts {
    #[must_use]
    pub fn of(lineage: &Lineage) -> Self {
        Self::assemble(
            lineage.chain(),
            lineage.proposals.iter().map(|proposal| {
                (
                    proposal.outcome.live(),
                    proposal.outcome.correct(lineage.task),
                    proposal.parent_score,
                    proposal.specialized,
                )
            }),
        )
    }

    #[must_use]
    pub fn of_compact(lineage: &CompactLineage) -> Self {
        Self::assemble(
            lineage.chain(),
            lineage.proposals.iter().map(|proposal| {
                (
                    proposal.live,
                    proposal.score,
                    proposal.parent_score,
                    proposal.specialized,
                )
            }),
        )
    }

    /// `proposals` yields `(live, score, parent_score, specialized)`.
    fn assemble(
        chain: ChainFacts<'_>,
        proposals: impl Iterator<Item = (bool, u8, u8, bool)>,
    ) -> Self {
        let mut specialized_proposals = 0;
        let proposals = proposals
            .map(|(live, score, parent_score, specialized)| {
                specialized_proposals += u64::from(specialized);
                (live, score, parent_score)
            })
            .collect();
        let last = chain.checkpoints.last();
        Self {
            batch: chain.batch,
            lineage: chain.lineage,
            proposals,
            specialized_proposals,
            checkpoints: chain.checkpoints.iter().map(Checkpoint::cost).collect(),
            proposal_discovery: chain.proposal_discovery.map(|d| d.generation),
            retained_discovery: chain.retained_discovery.map(|d| d.generation),
            specialized_discovery: chain.specialized_discovery.map(|d| d.generation),
            viable_retained_discovery: chain.viable_retained_discovery,
            retention: chain.retention.map(|r| r.outcome),
            horizons: chain
                .horizons
                .iter()
                .map(|horizon| (horizon.offset, horizon.outcome))
                .collect(),
            ladder: chain.ladder,
            classification: chain.classification,
            final_applicable: last.map_or((0, 0), |checkpoint| {
                (
                    checkpoint.eligible_site_fraction.numerator.into(),
                    checkpoint.eligible_site_fraction.denominator.into(),
                )
            }),
            destination_kinds: last.map_or_else(Default::default, |c| c.destination_kinds),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
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
    pub transitions: Transitions,
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
    pub supply: Supply,
    pub supply_rule: String,
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
