//! Serialised report types and their `undefined_*` constructors.

use super::comparison::{Comparison, MeasuredIdentity};
use super::profiles::GoalCase;
use super::run::millis;
use super::tracking::{PersistenceSample, WorldTracking};
use crate::{fraction_or_undefined, six, UNDEFINED};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::Instant;
use v3_core::neighborhood;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub schema_version: u32,
    pub feature: String,
    pub deterministic: Deterministic,
    pub environment: Environment,
    pub comparison: Comparison,
}

fn legacy_graph_work_definition() -> String {
    "graph_relax_iters: entered relaxation passes (before T11.F06)".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deterministic {
    #[serde(default = "legacy_graph_work_definition")]
    pub graph_work_definition: String,
    pub profile: ProfileBlock,
    pub per_seed: Vec<PerSeed>,
    pub totals: Totals,
    pub per_creature_tick: PerCreatureTick,
    pub goal_indicators: GoalIndicators,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileBlock {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cases: Vec<GoalCase>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recipe_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config_digest: Option<String>,
    pub name: String,
    pub world_width: u16,
    pub world_height: u16,
    pub founders: u32,
    pub seeds: Vec<u64>,
    pub ticks: u64,
    pub food_coverage: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerSeed {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tick_zero_connectivity: Option<v3_core::kernel::PassableConnectivity>,
    pub seed: u64,
    pub ticks: u64,
    pub creature_ticks: u64,
    pub mesh_hops: u64,
    pub vm_steps: u64,
    pub graph_relax_iters: u64,
    pub plasticity_updates: u64,
    pub actions_applied: u64,
    pub births: u64,
    pub final_population: u64,
    pub extinction_tick: Option<u64>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Totals {
    pub ticks: u64,
    pub creature_ticks: u64,
    pub mesh_hops: u64,
    pub vm_steps: u64,
    pub graph_relax_iters: u64,
    pub plasticity_updates: u64,
    pub actions_applied: u64,
    pub births: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PerCreatureTick {
    #[serde(default)]
    pub mesh_hops: Option<String>,
    #[serde(default)]
    pub vm_steps: Option<String>,
    #[serde(default)]
    pub graph_relax_iters: Option<String>,
    #[serde(default)]
    pub plasticity_updates: Option<String>,
    #[serde(default)]
    pub actions_applied: Option<String>,
    #[serde(default)]
    pub births: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalCaseObservation {
    pub case: GoalCase,
    /// Complete final population for this case; absent in the initial F04 report.
    #[serde(default)]
    pub reachable_structure_size_distribution: Option<StructureSizeDistribution>,
    /// End-of-run per-world behavior for this case.
    #[serde(flatten)]
    pub tracking: WorldTracking,
    /// The fractions derived from `tracking`; every field is empty in reports
    /// stored before T12.F04 measured them.
    #[serde(flatten)]
    pub fractions: TrackedFractions,
    pub mutational_neighborhood: Indicator<MutationalNeighborhood>,
    pub drift_depth: Indicator<DriftDepth>,
    /// The seeded births-only read of this world's terminal population
    /// (T14.F12); unmeasured, never zero, in reports stored before it.
    #[serde(default = "undefined_neighborhood_read")]
    pub neighborhood_read: Indicator<NeighborhoodRead>,
}

/// The rates [`WorldTracking`] implies, derived once so a total and its
/// fraction can never disagree.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrackedFractions {
    /// Each food type's share of every applied Eat.
    #[serde(default)]
    pub typed_eat_share: Vec<String>,
    /// Barrier-blocked moves against every move attempted, over the whole
    /// population. A property of the map, not of any genome.
    #[serde(default)]
    pub blocked_move_fraction: String,
    /// The barrier-awareness reading: how often a state's moves *made beside a
    /// barrier* were blocked by one, denominator
    /// `move_attempts_with_barrier_neighbor_by_reader_state`. This is a rate
    /// per reader state, so the two states are comparable to each other, and
    /// it is `Undefined` for a state that never moved beside a barrier — a
    /// barrier-free world reports no rate rather than zero.
    #[serde(default)]
    pub barrier_blocked_fraction_by_reader_state: ByReaderState<String>,
    /// Avoidable blocked moves of any cause, including occupancy and edges,
    /// against every move attempted by *every* genome. The denominator is the
    /// whole population's moves, so this is a share of all moves rather than a
    /// per-state rate, and the two states are not comparable to each other.
    #[serde(default)]
    pub avoidable_blocked_share_of_all_moves_by_reader_state: ByReaderState<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalIndicators {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cases: Vec<GoalCaseObservation>,
    pub population_persistence: PopulationPersistence,
    pub births_per_100_ticks: String,
    /// Pooled complete final populations across every seed/case.
    pub reachable_structure_size_distribution: StructureSizeDistribution,
    #[serde(default = "undefined_lineage_diversity")]
    pub lineage_diversity: Indicator<LineageDiversity>,
    #[serde(default = "undefined_memory_sensitivity")]
    pub memory_sensitivity: Indicator<MemorySensitivity>,
    /// Structural exposure in the same final populations; not a capability indicator.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub structural_companions: Option<StructuralCompanionsCensus>,
    #[serde(default = "undefined_temporal_memory_sensitivity")]
    pub temporal_memory_sensitivity: Indicator<TemporalMemorySensitivity>,
    #[serde(default = "undefined_mutational_neighborhood")]
    pub mutational_neighborhood: Indicator<MutationalNeighborhood>,
    #[serde(default = "undefined_drift_depth")]
    pub drift_depth: Indicator<DriftDepth>,
    /// One explicit default-task experiment per goal report, outside its worlds.
    #[serde(default = "undefined_recruitment_paths")]
    pub recruitment_paths: Indicator<neighborhood::recruitment_paths::Report>,
    pub strategy_count: String,
    pub strategy_causal_distinctness: String,
    pub evolutionary_activity: String,
    pub adaptive_novelty: String,
    pub memory_dependence: String,
    pub learning_dependence: String,
    pub prediction_dependence: String,
    pub information_integration: String,
    pub reciprocal_interaction: String,
}

/// A goal-only indicator is either unavailable for a profile or carries its
/// versioned per-seed observations. Keeping `Undefined` as the wire value
/// preserves the established report vocabulary for deferred measurements.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Indicator<T> {
    Undefined(String),
    Defined(T),
}

impl<T> Indicator<T> {
    /// The measured reading, or `None` when the indicator is undefined.
    pub fn defined(&self) -> Option<&T> {
        match self {
            Indicator::Defined(reading) => Some(reading),
            Indicator::Undefined(_) => None,
        }
    }
}

pub(super) fn undefined_lineage_diversity() -> Indicator<LineageDiversity> {
    Indicator::Undefined(UNDEFINED.to_string())
}

pub(super) fn undefined_memory_sensitivity() -> Indicator<MemorySensitivity> {
    Indicator::Undefined(UNDEFINED.to_string())
}

/// Definition tokens for the indicators that carry one. A definition change
/// moves the token here, so a stored report names the definition it measured.
pub const LINEAGE_DIVERSITY_VERSION: &str = "lineage-diversity-v1";
pub const MEMORY_SENSITIVITY_VERSION: &str = "memory-sensitivity-v1";
pub const REACHABLE_STRUCTURE_VERSION: &str = "reachable-structure-v1";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LineageDiversity {
    /// Absent in reports stored before T14.F01: the token was not measured.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    pub per_seed: Vec<LineageDiversitySeed>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LineageDiversitySeed {
    pub seed: u64,
    pub surviving_founder_clade_count: u64,
    pub shannon_entropy_nats: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemorySensitivity {
    /// Absent in reports stored before T14.F01: the token was not measured.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    pub snapshot_timing: String,
    pub scramble_algorithm: String,
    pub per_seed: Vec<MemorySensitivitySeed>,
}

/// Four overlapping carrier counts alongside shared-memory sensitivity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructuralCompanionsCensus {
    pub per_seed: Vec<StructuralCompanionsSeed>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructuralCompanionsSeed {
    pub seed: u64,
    pub final_creature_count: u64,
    pub reads_shared_memory: u64,
    pub writes_shared_memory: u64,
    pub has_stateful_compute_node: u64,
    pub has_plasticity: u64,
}

pub(super) fn undefined_temporal_memory_sensitivity() -> Indicator<TemporalMemorySensitivity> {
    Indicator::Undefined(UNDEFINED.to_string())
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemporalMemorySensitivity {
    pub version: String,
    pub snapshot_timing: String,
    pub scramble_algorithm: String,
    pub per_seed: Vec<TemporalMemorySensitivitySeed>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemporalMemorySensitivitySeed {
    pub seed: u64,
    pub previous_slots: MemorySensitivitySeed,
    pub persisted_outputs: MemorySensitivitySeed,
    pub operator_state: MemorySensitivitySeed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemorySensitivitySeed {
    pub seed: u64,
    pub final_creature_count: u64,
    pub different_from_zeroed_count: u64,
    pub different_from_scrambled_count: u64,
    pub different_from_either_count: u64,
    pub different_from_zeroed_fraction: String,
    pub different_from_scrambled_fraction: String,
    pub different_from_either_fraction: String,
}

pub(super) fn undefined_mutational_neighborhood() -> Indicator<MutationalNeighborhood> {
    Indicator::Undefined(UNDEFINED.to_string())
}

pub(super) fn undefined_evolved_neighborhood() -> Indicator<EvolvedNeighborhoodHalf> {
    Indicator::Undefined(UNDEFINED.to_string())
}

pub(super) fn undefined_recruitment_paths() -> Indicator<neighborhood::recruitment_paths::Report> {
    Indicator::Undefined("unmeasured; recruitment-paths-v1 is goal-only".into())
}

pub(super) fn timed_recruitment_paths(
    goal: bool,
    sizes: neighborhood::recruitment_paths::Sizes,
) -> (
    Indicator<neighborhood::recruitment_paths::Report>,
    Option<f64>,
) {
    if !goal {
        return (undefined_recruitment_paths(), None);
    }
    let start = Instant::now();
    let reading = neighborhood::recruitment_paths::observe(sizes);
    (Indicator::Defined(reading), Some(millis(start.elapsed())))
}

pub(super) fn undefined_drift_depth() -> Indicator<DriftDepth> {
    Indicator::Undefined(UNDEFINED.to_string())
}

pub(super) fn undefined_neighborhood_read() -> Indicator<NeighborhoodRead> {
    Indicator::Undefined(UNDEFINED.to_string())
}

pub const NEIGHBORHOOD_READ_VERSION: &str = "neighborhood-read-v1";

/// The neighborhood read (T14.F12): a seeded sample of the id-sorted living
/// population at the end of a goal-profile world, each genome's production
/// births classified by the `neighborhood-v1` battery, pooled over the
/// sample. Every fraction divides by `births.births_total`, all births, as
/// the drift checkpoint's do.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NeighborhoodRead {
    pub version: String,
    pub battery_version: String,
    pub sample_seed_formula: String,
    pub birth_seed_formula: String,
    pub population_size: u64,
    pub sample_size_requested: u32,
    pub sample_size: u32,
    pub birth_trials: u32,
    pub births: NeighborhoodBirths,
    pub silent_per_all_births: String,
    pub changed_per_all_births: String,
    pub dead_per_all_births: String,
    pub generation_sum: u64,
    pub mean_generation: String,
    pub genome_size_sum: u64,
    pub mean_genome_size: String,
    pub total_nodes: u64,
    pub mean_total_nodes: String,
    pub reachable_nodes: u64,
    pub mean_reachable_nodes: String,
    pub executed_nodes: u64,
    pub mean_executed_nodes: String,
    pub genomes: Vec<NeighborhoodReadGenome>,
}

/// One sampled genome's identity, structure, and integer birth tallies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NeighborhoodReadGenome {
    pub rank: u64,
    pub creature_id: String,
    pub lineage_id: u32,
    pub generation: u64,
    pub genome_size: u32,
    pub total_nodes: u64,
    pub reachable_nodes: u64,
    pub executed_nodes: u64,
    pub births_total: u32,
    pub zero_event_births: u32,
    pub silent: u32,
    pub changed: u32,
    pub dead: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DriftDepth {
    pub version: String,
    pub founder: String,
    pub lineages: u32,
    pub birth_lineages: u32,
    pub birth_subset: String,
    pub birth_trials: u32,
    pub checkpoints: Vec<u64>,
    pub walk_seed_formula: String,
    pub birth_seed_formula: String,
    pub battery_version: String,
    pub mesh_version: String,
    pub knockout_method: String,
    /// Where the walk's executed node sets come from, and how often they are
    /// refreshed (T11.F17). Serde-defaulted so historical v1 reports load.
    #[serde(default)]
    pub executed_source: String,
    #[serde(default)]
    pub executed_refresh: String,
    /// New in `drift-depth-v3`: how modules are identified and where their
    /// provenance comes from. Empty in earlier reports.
    #[serde(default)]
    pub recruitment_version: String,
    #[serde(default)]
    pub module_identity: String,
    #[serde(default)]
    pub provenance_rule: String,
    /// The mutation supply rule and values the walk ran (T11.F19): always the
    /// legacy per-birth rule, whatever production selects. Empty in earlier
    /// reports, whose walks ran the same rule as production's default.
    #[serde(default)]
    pub supply_rule: String,
    pub executions_per_genome: u32,
    pub snapshot_count: u32,
    pub sequence_count: u32,
    pub sequence_len: u32,
    pub readings: Vec<DriftDepthCheckpoint>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DriftDepthCheckpoint {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backends: Option<neighborhood::mesh_execution::MeshBackendCounts>,
    /// New in `drift-depth-v3`; absent, never zero, in earlier reports.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recruitment: Option<ModuleRecruitment>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opportunities: Option<MutationOpportunities>,
    pub depth: u64,
    pub lineages: u32,
    pub total_nodes: u64,
    pub reachable_nodes: u64,
    pub executed_nodes: u64,
    pub knockout_nodes: u64,
    pub mean_total_nodes: String,
    pub mean_reachable_nodes: String,
    pub mean_executed_nodes: String,
    pub mean_knockout_nodes: String,
    pub route_varying_lineages: u32,
    pub route_varying_fraction: String,
    pub hop_cap_hits: u64,
    pub battery_executions: u64,
    pub hop_cap_fraction: String,
    pub births: NeighborhoodBirths,
    pub silent_per_all_births: String,
    pub changed_per_all_births: String,
    pub dead_per_all_births: String,
}

/// Cohort counts for one backend split, with fractions against the pooled
/// integers beside them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CohortLadder {
    pub created: u64,
    pub deleted: u64,
    pub present: u64,
    pub never_selected: u64,
    pub selected_only: u64,
    pub applied_only: u64,
    pub changed_only: u64,
    pub dispatched_not_contributing: u64,
    pub contributing: u64,
    /// `present / created`.
    pub present_fraction: String,
    /// `(dispatched_not_contributing + contributing) / present`.
    pub dispatched_fraction: String,
    /// `contributing / present`.
    pub contributing_fraction: String,
}

pub(super) fn cohort_ladder(counts: neighborhood::recruitment::CohortCounts) -> CohortLadder {
    CohortLadder {
        created: counts.created,
        deleted: counts.deleted,
        present: counts.present,
        never_selected: counts.never_selected,
        selected_only: counts.selected_only,
        applied_only: counts.applied_only,
        changed_only: counts.changed_only,
        dispatched_not_contributing: counts.dispatched_not_contributing,
        contributing: counts.contributing,
        present_fraction: fraction_or_undefined(counts.present, counts.created),
        dispatched_fraction: fraction_or_undefined(counts.dispatched(), counts.present),
        contributing_fraction: fraction_or_undefined(counts.contributing, counts.present),
    }
}

/// Founder modules as a reference row beside the recruited cohort.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FounderRow {
    pub created: u64,
    pub deleted: u64,
    pub present: u64,
    pub dispatched: u64,
    pub contributing: u64,
    pub contributing_fraction: String,
}

/// How long a cohort took to first reach one fact, with both censoring counts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimeToFirstRow {
    pub fact: String,
    pub reached: u64,
    pub reached_fraction: String,
    pub median_generations: Option<u64>,
    pub censored_deleted: u64,
    pub censored_present: u64,
}

/// What became of the modules contributing at the previous checkpoint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetentionRow {
    pub from_depth: u64,
    pub contributing_before: u64,
    pub still_contributing: u64,
    pub present_not_contributing: u64,
    pub deleted: u64,
    pub retained_fraction: String,
}

/// One lineage's cohort row at a checkpoint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CohortLineageRow {
    pub lineage: u32,
    pub created: u64,
    pub present: u64,
    pub dispatched: u64,
    pub contributing: u64,
}

/// The module recruitment reading at one checkpoint (T13.F01). Absent from
/// `drift-depth-v2` and earlier reports, never zero there.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleRecruitment {
    pub cohort: CohortLadder,
    pub graph: CohortLadder,
    pub vm: CohortLadder,
    pub founders: FounderRow,
    pub time_to_first: Vec<TimeToFirstRow>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retention: Option<RetentionRow>,
    pub lineages: Vec<CohortLineageRow>,
}

/// One operator's cumulative opportunities across the walk.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OperatorOpportunityRow {
    pub operator: String,
    pub attempted: u64,
    pub applied: u64,
    pub applied_fraction: String,
    pub skipped_by_reason: BTreeMap<String, u64>,
}

/// One lineage's cumulative opportunities across the walk.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LineageOpportunityRow {
    pub lineage: u32,
    pub births: u64,
    pub attempted: u64,
    pub applied: u64,
    pub skipped: u64,
    /// Operators this lineage's births discarded after selecting a node that
    /// carried no applicable site, summed over
    /// `discarded_selected_inapplicable_by_operator`. These rows sum to the
    /// pooled map's total.
    pub discarded_selected_inapplicable: u64,
    pub applied_by_domain: BTreeMap<String, u64>,
}

/// The mutation opportunities production offered, pooled from depth 0 to this
/// checkpoint over every lineage (T13.F01).
///
/// `selected_inapplicable` counts a node that was selected and carried no
/// applicable site; `no_eligible_node` counts finding no node of the required
/// kind at all. The `_by_domain` maps count whole events, and only an event
/// whose domain exhausted every operator can land there, because the engine
/// retries the other operators of a domain before recording the skip. The
/// `discarded_*_by_operator` maps count each operator the engine threw away
/// for reporting no applicable site, whether or not a later operator of the
/// same domain then applied, so an operator that selected a module and found
/// no site is visible there alone.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MutationOpportunities {
    pub births: u64,
    pub zero_event_births: u64,
    pub zero_event_fraction: String,
    pub attempted: u64,
    pub applied: u64,
    pub skipped: u64,
    pub applied_fraction: String,
    pub reachable_target_events: u64,
    pub unreachable_target_events: u64,
    pub executed_target_events: u64,
    pub executed_target_fraction: String,
    pub attempted_by_domain: BTreeMap<String, u64>,
    pub applied_by_domain: BTreeMap<String, u64>,
    pub selected_inapplicable_by_domain: BTreeMap<String, u64>,
    pub no_eligible_node_by_domain: BTreeMap<String, u64>,
    #[serde(default)]
    pub discarded_selected_inapplicable_by_operator: BTreeMap<String, u64>,
    #[serde(default)]
    pub discarded_no_eligible_node_by_operator: BTreeMap<String, u64>,
    pub operators: Vec<OperatorOpportunityRow>,
    pub lineages: Vec<LineageOpportunityRow>,
}

/// One class's count and, against the applied count, its six-decimal
/// fraction (`Undefined` when nothing applied). `mean_fraction_differing` is
/// the mean fraction of a signature's executions that differ, among changed
/// and dead trials only.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NeighborhoodTally {
    pub trials: u32,
    pub skipped: u32,
    pub applied: u32,
    pub silent: u32,
    pub changed: u32,
    pub dead: u32,
    pub changed_only_in_sequences: u32,
    pub silent_fraction: String,
    pub changed_fraction: String,
    pub dead_fraction: String,
    pub mean_fraction_differing: String,
}

/// One operator's family, name, and tally.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NeighborhoodOperatorRow {
    pub family: String,
    pub operator: String,
    pub tally: NeighborhoodTally,
}

/// One applied-event-count bucket's tally.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NeighborhoodBirthBucket {
    pub applied_events: u32,
    pub tally: NeighborhoodTally,
}

/// One requested-event-count bucket, ordered by count in reports.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NeighborhoodRequestedBirthBucket {
    pub requested_events: u32,
    pub births: u32,
}

/// The per-birth reading: total births attempted, how many drew zero applied
/// events (counted, not evaluated, since they are identical to the base by
/// construction), the pooled "any events" tally, and the same trials
/// bucketed by their exact `applied_events` count, ascending.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NeighborhoodBirths {
    pub births_total: u32,
    /// Requested-event histogram; absent in reports before T11.F04.
    #[serde(default)]
    pub by_requested_events: Vec<NeighborhoodRequestedBirthBucket>,
    pub zero_event_births: u32,
    pub any_events: NeighborhoodTally,
    pub by_events: Vec<NeighborhoodBirthBucket>,
}

/// Structural facts about a sampled genome's reachable mesh (T11.F01
/// "Structural companions"), so a zero reading is attributed to absent
/// structure, inert structure, or a probe blind spot rather than assumed.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NeighborhoodCompanions {
    pub functional_complexity: u32,
    pub reachable_node_count: u64,
    pub reads_shared_memory: bool,
    pub writes_shared_memory: bool,
    pub has_stateful_compute_node: bool,
    pub has_plasticity: bool,
}

/// The predeclared `neighborhood-v1` battery: version, seeds, sizes, and the
/// battery's total execution count per genome (48 snapshots + 32 sequence
/// ticks at the predeclared sizes), recorded here rather than restated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NeighborhoodBattery {
    pub version: String,
    pub snapshot_seed: u64,
    pub sequence_seed: u64,
    pub snapshot_count: u32,
    pub sequence_count: u32,
    pub sequence_len: u32,
    pub executions_per_genome: u32,
    pub founder_operator_trials: u32,
    pub founder_birth_count: u32,
    pub evolved_operator_trials: u32,
    pub evolved_birth_count: u32,
    pub evolved_sample_size: u32,
}

/// Mesh measurements on the existing neighborhood battery, never a fitness signal.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeshExecution {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backends: Option<neighborhood::mesh_execution::MeshBackendCounts>,
    pub version: String,
    pub executions_per_genome: u32,
    pub snapshot_route_probes: u32,
    pub knockout_method: String,
    pub total_node_count: u64,
    pub reachable_node_count: u64,
    pub executed_node_count: u64,
    pub knockout_count: u64,
    pub route_varies_with_input: bool,
    pub hop_cap_hits: u64,
}

fn undefined_mesh_execution() -> Indicator<MeshExecution> {
    Indicator::Undefined(UNDEFINED.to_string())
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenerationDistribution {
    pub median: u64,
    pub max: u64,
}

pub(super) fn undefined_generation_distribution() -> Indicator<GenerationDistribution> {
    Indicator::Undefined(UNDEFINED.to_string())
}

/// The founder half: always present when `mutational_neighborhood` is
/// defined (gate and goal).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NeighborhoodFounderHalf {
    #[serde(default)]
    pub generation: Option<u64>,
    #[serde(default = "undefined_mesh_execution")]
    pub mesh_execution: Indicator<MeshExecution>,
    pub reachable_node_count: u64,
    pub operator_rows: Vec<NeighborhoodOperatorRow>,
    pub births: NeighborhoodBirths,
}

/// One sampled evolved genome's rank, id, and complete neighborhood reading.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NeighborhoodSampledGenome {
    #[serde(default)]
    pub generation: Option<u64>,
    #[serde(default = "undefined_mesh_execution")]
    pub mesh_execution: Indicator<MeshExecution>,
    pub rank: u64,
    pub creature_id: String,
    pub operator_rows: Vec<NeighborhoodOperatorRow>,
    pub births: NeighborhoodBirths,
    pub companions: NeighborhoodCompanions,
}

/// One seed's evolved-sample reading: the sampled genomes and the pooled
/// per-operator and per-birth tallies across the sample.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NeighborhoodEvolvedSeed {
    #[serde(default = "undefined_generation_distribution")]
    pub generation_distribution: Indicator<GenerationDistribution>,
    pub seed: u64,
    pub final_population_size: u64,
    pub sampled_genomes: Vec<NeighborhoodSampledGenome>,
    pub pooled_operator_rows: Vec<NeighborhoodOperatorRow>,
    pub pooled_births: NeighborhoodBirths,
}

/// The evolved half: `Undefined` outside the goal profile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvolvedNeighborhoodHalf {
    pub per_seed: Vec<NeighborhoodEvolvedSeed>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MutationalNeighborhood {
    pub battery: NeighborhoodBattery,
    pub founder: NeighborhoodFounderHalf,
    #[serde(default = "undefined_evolved_neighborhood")]
    pub evolved: Indicator<EvolvedNeighborhoodHalf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopulationPersistence {
    pub per_seed: Vec<PopulationPersistenceSeed>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopulationPersistenceSeed {
    pub seed: u64,
    pub extinction_tick: Option<u64>,
    pub minimum_population: u64,
    pub final_population: u64,
    /// Maximum population observed from seeding (tick 0, founders) through
    /// the last executed tick, and the first tick at which it occurred.
    #[serde(default)]
    pub peak_population: u64,
    #[serde(default)]
    pub peak_tick: u64,
    /// Mean population over the ticks executed strictly after
    /// `0.75 x horizon`; `null` when the run went extinct before that window.
    #[serde(default)]
    pub plateau_population: Option<String>,
    /// Mean creature energy at the last executed tick; `null` at extinction.
    #[serde(default)]
    pub mean_energy: Option<String>,
    #[serde(default)]
    pub samples: Vec<PersistenceSample>,
}

/// A reading split by whether the acting genome reads the barrier ring.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ByReaderState<T: Default> {
    pub has_barrier_reader: T,
    pub no_barrier_reader: T,
}

/// Blocked move actions split by what blocked them, one field per
/// [`v3_core::simulation::actions::MoveBlockedCause`] variant.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MovesBlockedByCause {
    pub barrier: u64,
    pub occupied: u64,
    pub out_of_bounds: u64,
}

/// The mutation supply one case produced and where its events aimed
/// (T14.F02), read from `SimStats`. `attempted = applied + skipped`, and the
/// four-way target split is T11.F17's own delivery instrument.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MutationSupply {
    pub events_attempted_total: u64,
    pub events_applied_total: u64,
    pub events_skipped_total: u64,
    /// Applied events whose target was a node the parent executed recently.
    pub executed_target_total: u64,
    pub reachable_target_total: u64,
    pub unreachable_target_total: u64,
    pub not_applicable_target_total: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructureSizeDistribution {
    /// Absent in reports stored before T14.F01: the token was not measured.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    pub min: u32,
    pub p25: u32,
    pub median: u32,
    pub p75: u32,
    pub max: u32,
    pub mean: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    /// Locked SmallRng dependency version; absent in historical reports means unmeasured.
    #[serde(default)]
    pub rand_version: Option<String>,
    pub generated_at: String,
    pub host: Host,
    pub build_profile: String,
    pub git_revision: String,
    pub wall_clock_ms_per_seed: Vec<SeedWallClock>,
    pub wall_clock_ms_total: f64,
    pub wall_clock_ms_per_creature_tick: f64,
    /// The rayon thread count in effect during the run, read from inside the
    /// pool that ran it. Absent from reports stored before T10.F09.
    #[serde(default)]
    pub threads: Option<usize>,
    /// Per-seed cumulative wall-clock by tick phase (T10.F09).
    #[serde(default)]
    pub phase_wall_clock_ms_per_seed: Vec<SeedPhaseWallClock>,
    /// Per-seed and total throughput derived from this report's own counters
    /// and wall-clock (T10.F09).
    #[serde(default)]
    pub throughput: Throughput,
    /// Goal-only final-state observation cost, kept outside the existing timed
    /// tick phases and empty for gate and sweep profiles.
    #[serde(default)]
    pub final_state_observation_ms_per_seed: Vec<SeedFinalStateObservation>,
    #[serde(default)]
    pub final_state_observation_ms_total: f64,
    /// Founder-half mutational-neighborhood wall time (T11.F01), computed
    /// once per report, kept outside every simulation and final-state
    /// observation timing above. Zero for the sweep profile.
    #[serde(default)]
    pub neighborhood_founder_wall_clock_ms: f64,
    /// Complete goal-only mutation walk and readings; absent when unrun.
    #[serde(default)]
    pub drift_depth_wall_clock_ms: Option<f64>,
    /// Entire report-level task experiment, outside all ecological observations.
    #[serde(default)]
    pub recruitment_paths_wall_clock_ms: Option<f64>,
    /// Evolved-half neighborhood wall time per seed (goal profile only).
    #[serde(default)]
    pub neighborhood_evolved_wall_clock_ms_per_seed: Vec<SeedFinalStateObservation>,
    #[serde(default)]
    pub neighborhood_evolved_wall_clock_ms_total: f64,
    /// Neighborhood-read wall time per seed (goal world set only).
    #[serde(default)]
    pub neighborhood_read_wall_clock_ms_per_seed: Vec<SeedFinalStateObservation>,
    #[serde(default)]
    pub neighborhood_read_wall_clock_ms_total: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeedFinalStateObservation {
    pub seed: u64,
    pub wall_clock_ms: f64,
}

/// One seed's cumulative wall-clock split across the five timed tick phases.
/// The turn-queue build and the priority sort are untimed: they are the
/// remainder against that seed's `wall_clock_ms`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SeedPhaseWallClock {
    pub seed: u64,
    pub world_update_ms: f64,
    pub sensor_assembly_ms: f64,
    pub cognition_ms: f64,
    pub actions_ms: f64,
    pub reward_learning_ms: f64,
}

/// Throughput rates for one run, per seed and in total.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Throughput {
    pub per_seed: Vec<SeedThroughput>,
    pub total: ThroughputRates,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SeedThroughput {
    pub seed: u64,
    #[serde(flatten)]
    pub rates: ThroughputRates,
}

/// Wall-clock rates. `ticks_per_hour` and `births_per_hour` are the per-core
/// budget when they come from a `--threads 1` report.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct ThroughputRates {
    pub ticks_per_second: f64,
    pub creature_ticks_per_second: f64,
    pub ticks_per_hour: f64,
    pub births_per_hour: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Host {
    pub hostname: String,
    pub os: String,
    pub arch: String,
    pub cpu_model: Option<String>,
    pub logical_cores: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedWallClock {
    pub seed: u64,
    pub wall_clock_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceComparison {
    pub path: String,
    /// Original measurement identity, unavailable in historical comparisons.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub measured_identity: Option<MeasuredIdentity>,
    pub counters: Vec<CounterComparison>,
    /// One entry per world in the `goal-worlds-v1` profile; empty for the gate
    /// and single-config profiles, which have no cases to compare.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cases: Vec<CaseComparison>,
    pub wall_clock: Option<WallClockComparison>,
    pub severe: bool,
}

/// One world's reading against the same world in a reference report. Per-case
/// entries carry no severity level: a world set follows each environment's
/// trajectory, and the profile-total work counters above own the regression
/// rule.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CaseComparison {
    pub case: String,
    /// The reference ran this case from different inputs — an edited recipe or
    /// a reassigned run seed. Labeled, never an error: a recipe edit is the
    /// point of a world-set closure.
    pub inputs_changed: bool,
    /// The reference report has no case by this name at all.
    pub absent_in_reference: bool,
    pub current_digest: String,
    pub reference_digest: Option<String>,
    pub readings: Vec<CaseReadingComparison>,
}

/// One per-case reading, as six-decimal strings so it reads like every other
/// comparison value in the report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CaseReadingComparison {
    pub name: String,
    pub current: Option<String>,
    pub reference: Option<String>,
    pub percent_delta: Option<String>,
}

/// Readings whose difference is not a rate: a percent delta on them would be
/// meaningless, so only the values are reported.
pub(super) const VALUE_ONLY_CASE_READINGS: [&str; 1] = ["extinction_tick"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterComparison {
    pub name: String,
    pub current: String,
    pub reference: Option<String>,
    pub percent_delta: Option<String>,
    pub level: ComparisonLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallClockComparison {
    pub current_ms_per_creature_tick: f64,
    pub reference_ms_per_creature_tick: f64,
    pub percent_delta: f64,
    pub level: ComparisonLevel,
}

/// How one comparison against a stored reference read. Serializes as the
/// lowercase strings the stored reports already carry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ComparisonLevel {
    /// Within threshold, or no comparable reference value.
    Ok,
    Flag,
    Severe,
    /// The counter is absent from the reference report (an older schema);
    /// never treated as a zero-value regression.
    New,
}

impl std::fmt::Display for ComparisonLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Ok => "ok",
            Self::Flag => "flag",
            Self::Severe => "severe",
            Self::New => "new",
        };
        f.write_str(name)
    }
}

// ── Formatting helpers ───────────────────────────────────────────────────────

pub(super) fn ratio(numerator: u64, denominator: u64) -> String {
    if denominator == 0 {
        return six(0.0);
    }
    six(numerator as f64 / denominator as f64)
}

// ── Per-seed persistence tracking (T01.F11) ─────────────────────────────────

#[cfg(test)]
mod tests;
