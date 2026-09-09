//! Deterministic benchmark harness (T10.F10).
//!
//! Runs a fixed set of seeded simulations and emits one compact JSON report
//! per feature. The report's `deterministic` block is byte-identical for the
//! same commit and inputs; the `environment` block records host identity and
//! wall-clock as an unasserted secondary signal.

use std::collections::BTreeMap;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use v3_core::config::{MutationConfig, SimulationConfig};

use crate::{fraction_or_undefined, six, UNDEFINED};
use v3_core::creature::founder::founder_genome;
use v3_core::creature::genome::analysis::functional_complexity;
use v3_core::neighborhood::{
    self, evaluate_genome, evolved_sample_ranks, structural_companions, Battery, BirthResult,
    EvalContext, GenomeEvaluation, OperatorRow, StructuralCompanions, Tally, BATTERY_VERSION,
};
use v3_core::simulation::{
    observe_final_actions, run_tick, seed_simulation, FinalActionObservation,
};

pub const SCHEMA_VERSION: u32 = 1;

/// The six deterministic work counters normalized by creature-tick.
pub const COUNTER_NAMES: [&str; 6] = [
    "mesh_hops",
    "vm_steps",
    "graph_relax_iters",
    "plasticity_updates",
    "actions_applied",
    "births",
];

/// Deterministic work-counter regression thresholds against each reference,
/// per the T10.F10 spec.
const FLAG_PERCENT: f64 = 10.0;
const SEVERE_PERCENT: f64 = 50.0;

/// Wall-clock regression thresholds against a host-matching reference. They
/// are looser than the work-counter ones because wall-clock is a noisy
/// secondary signal, and a wall-clock level never fails a comparison.
const WALL_CLOCK_FLAG_PERCENT: f64 = 25.0;
const WALL_CLOCK_SEVERE_PERCENT: f64 = 100.0;

/// Persistence sampling cadence (T01.F11): every executed tick that is a
/// multiple of this constant is sampled, plus the last executed tick.
pub const SAMPLE_EVERY_TICKS: u64 = 100;

/// Mutational-neighborhood (T11.F01) battery trial and birth counts. Carried
/// on `ProfileParams` rather than fixed constants, so a tiny test fixture can
/// exercise the same goal/gate observation code path as production at a
/// fraction of the cost (the T01.F12 precedent for the lineage and memory
/// indicators). Never written to `ProfileBlock` — `compare_against_path`
/// hard-fails on `ProfileBlock` inequality, and only `gate_profile_params()`
/// and `goal_profile_params()` carry the production reading the floors and
/// the no-regression rule depend on. The actual sizes used are recorded
/// truthfully in the report's `NeighborhoodBattery` block instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NeighborhoodSizes {
    pub founder_operator_trials: u32,
    pub founder_births: u32,
    pub evolved_operator_trials: u32,
    pub evolved_births: u32,
}

impl NeighborhoodSizes {
    /// The production sizes, halved once from the originally predeclared
    /// 100 operator trials / 1,000 births after a debug-build timing
    /// measurement showed the founder half's two full-report calls inside
    /// `gate_profile_deterministic_block_is_byte_identical_across_two_runs`
    /// growing that test well past its 10-second budget (see the T11.F01
    /// spec's Verification section for the measured numbers). Release wall
    /// time at the original sizes was ~0.8s against the 10s release budget,
    /// so the halving is driven by the debug-build constraint alone. The
    /// evolved-half sizes are unaffected by that halving, since the evolved
    /// half never runs inside a debug-build gate test.
    pub const PRODUCTION: Self = Self {
        founder_operator_trials: 50,
        founder_births: 500,
        evolved_operator_trials: 20,
        evolved_births: 200,
    };
}

impl Default for NeighborhoodSizes {
    /// Small sizes for test fixtures and synthetic profiles: fast, and never
    /// mistaken for a production reading.
    fn default() -> Self {
        Self {
            founder_operator_trials: 2,
            founder_births: 5,
            evolved_operator_trials: 2,
            evolved_births: 5,
        }
    }
}

// ── Profile parameters ──────────────────────────────────────────────────────

/// A complete resolved recipe and its supplied source path.
#[derive(Debug, Clone)]
pub struct Recipe {
    pub path: String,
    pub config: SimulationConfig,
}

impl PartialEq for Recipe {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
            && serde_json::to_value(&self.config).expect("config serializes")
                == serde_json::to_value(&other.config).expect("config serializes")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProfileParams {
    pub recipe: Option<Recipe>,
    pub name: String,
    pub width: u16,
    pub height: u16,
    pub founders: u32,
    pub seeds: Vec<u64>,
    pub ticks: u64,
    /// `None` leaves `SimulationConfig::default()`'s per-food-type coverage
    /// untouched (production defaults) and serializes as the profile string
    /// `default`; `Some(x)` forces `x` onto every food type.
    pub food_coverage: Option<f32>,
    pub neighborhood: NeighborhoodSizes,
    pub drift: neighborhood::drift::DriftSizes,
}

/// Predeclared gate profile constants (T10.F10 Inputs and Invariants).
///
/// The horizon was reduced from the originally predeclared 300 ticks to 75
/// ticks after a timing probe on the recording host measured the two fast
/// `make check` tests (nine seed-runs total, debug build) would otherwise
/// take roughly 75 seconds, well over the 30-second budget. World size,
/// founder count, seeds, and food coverage are unchanged from the
/// predeclaration.
pub fn gate_profile_params() -> ProfileParams {
    ProfileParams {
        recipe: None,
        name: "gate".to_string(),
        width: 128,
        height: 128,
        founders: 256,
        seeds: vec![11, 22, 33],
        ticks: 75,
        food_coverage: Some(1.0),
        neighborhood: NeighborhoodSizes::PRODUCTION,
        drift: neighborhood::drift::DriftSizes::PRODUCTION,
    }
}

/// Predeclared minutes-scale goal profile constants (T01.F12).
pub fn goal_profile_params() -> ProfileParams {
    ProfileParams {
        recipe: None,
        name: GOAL_WORLD_SET.to_string(),
        width: 1600,
        height: 1600,
        founders: 10_000,
        seeds: goal_recipe_seeds(),
        ticks: 2_000,
        food_coverage: None,
        neighborhood: NeighborhoodSizes::PRODUCTION,
        drift: neighborhood::drift::DriftSizes::PRODUCTION,
    }
}

pub const GOAL_WORLD_SET: &str = "goal-worlds-v1";

/// One checked-in baseline world: the recipe the world-set profile runs and
/// the seed it runs it at. Adding a world to the set is one entry here — the
/// profile's seed list is derived from these seeds, and a profile that names
/// any other list is rejected rather than silently mis-paired.
struct GoalRecipe {
    name: &'static str,
    path: &'static str,
    source: &'static str,
    seed: u64,
}

const GOAL_RECIPES: [GoalRecipe; 3] = [
    GoalRecipe {
        name: "Orchards in grassland",
        path: "experiments/worlds/orchards-in-grassland.json",
        source: include_str!("../../../experiments/worlds/orchards-in-grassland.json"),
        seed: 11,
    },
    GoalRecipe {
        name: "Canyon country",
        path: "experiments/worlds/canyon-country.json",
        source: include_str!("../../../experiments/worlds/canyon-country.json"),
        seed: 22,
    },
    GoalRecipe {
        name: "Confluence",
        path: "experiments/worlds/confluence.json",
        source: include_str!("../../../experiments/worlds/confluence.json"),
        seed: 33,
    },
];

/// The seeds the checked-in world set runs, in report order.
fn goal_recipe_seeds() -> Vec<u64> {
    GOAL_RECIPES.iter().map(|recipe| recipe.seed).collect()
}

/// The recipe each of a profile's seeds runs: the checked-in world set for
/// [`GOAL_WORLD_SET`], and nothing at all for every other profile, whose seeds
/// are replicates of one config.
///
/// # Errors
///
/// When a world-set profile's seed list is not exactly the seeds its recipes
/// carry, in their order. Pairing by position would then run a world under
/// another world's recipe, drop a world the set gained, or index past the
/// recipe list, so this is a hard error before any run starts.
fn goal_recipes_for(params: &ProfileParams) -> Result<&'static [GoalRecipe], String> {
    if params.name != GOAL_WORLD_SET {
        return Ok(&[]);
    }
    let seeds = goal_recipe_seeds();
    if params.seeds != seeds {
        return Err(format!(
            "profile {GOAL_WORLD_SET} runs one checked-in recipe per seed and must name their \
             seeds in order {seeds:?}, not {:?}",
            params.seeds
        ));
    }
    Ok(&GOAL_RECIPES)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoalCase {
    pub name: String,
    pub seed: u64,
    pub recipe_path: String,
    pub config_digest: String,
    pub food_type_count: usize,
}

fn goal_case(params: &ProfileParams, recipe: &GoalRecipe) -> (GoalCase, SimulationConfig) {
    let config = v3_core::config::resolve_config(
        &SimulationConfig::default(),
        serde_json::from_str(recipe.source).expect("checked-in recipe JSON"),
    )
    .expect("checked-in goal recipe");
    let mut case_params = params.clone();
    case_params.recipe = Some(Recipe {
        path: recipe.path.to_string(),
        config,
    });
    let config = build_config(&case_params);
    (
        GoalCase {
            name: recipe.name.to_string(),
            seed: recipe.seed,
            recipe_path: recipe.path.to_string(),
            config_digest: v3_core::config::config_digest(&config),
            food_type_count: config.world.food.types.len(),
        },
        config,
    )
}

/// The `profile.food_coverage` report string for a profile that leaves
/// production food coverage untouched.
const DEFAULT_FOOD_COVERAGE: &str = "default";

pub fn build_config(params: &ProfileParams) -> SimulationConfig {
    let mut config = params
        .recipe
        .as_ref()
        .map_or_else(SimulationConfig::default, |recipe| recipe.config.clone());
    config.world.width = params.width;
    config.world.height = params.height;
    config.population.initial_creatures = params.founders;
    if let Some(coverage) = params.food_coverage {
        config.world.food.shared.initial_coverage = coverage;
        for food_type in &mut config.world.food.types {
            food_type.initial_coverage = coverage;
        }
    }
    config.normalize();
    if params.recipe.is_some() {
        config.apply_startup_overrides();
    }
    config
}

fn profile_block(
    params: &ProfileParams,
    config: &SimulationConfig,
    recipes: &[GoalRecipe],
) -> ProfileBlock {
    ProfileBlock {
        cases: recipes
            .iter()
            .map(|recipe| goal_case(params, recipe).0)
            .collect(),
        recipe_path: params.recipe.as_ref().map(|recipe| recipe.path.clone()),
        config_digest: params
            .recipe
            .as_ref()
            .map(|_| v3_core::config::config_digest(config)),
        name: params.name.clone(),
        world_width: config.world.width,
        world_height: config.world.height,
        founders: config.population.initial_creatures,
        seeds: params.seeds.clone(),
        ticks: params.ticks,
        food_coverage: params.food_coverage.map_or_else(
            || {
                if params.name == GOAL_WORLD_SET {
                    "per-case recipe".to_string()
                } else if params.recipe.is_some() {
                    "recipe".to_string()
                } else {
                    DEFAULT_FOOD_COVERAGE.to_string()
                }
            },
            |coverage| {
                six(f64::from(if params.recipe.is_some() {
                    config.world.food.types[0].initial_coverage
                } else {
                    coverage
                }))
            },
        ),
    }
}

// ── Report schema ────────────────────────────────────────────────────────────

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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    #[serde(default = "undefined_temporal_memory_sensitivity")]
    pub temporal_memory_sensitivity: Indicator<TemporalMemorySensitivity>,
    #[serde(default = "undefined_mutational_neighborhood")]
    pub mutational_neighborhood: Indicator<MutationalNeighborhood>,
    #[serde(default = "undefined_drift_depth")]
    pub drift_depth: Indicator<DriftDepth>,
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

fn undefined_lineage_diversity() -> Indicator<LineageDiversity> {
    Indicator::Undefined(UNDEFINED.to_string())
}

fn undefined_memory_sensitivity() -> Indicator<MemorySensitivity> {
    Indicator::Undefined(UNDEFINED.to_string())
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LineageDiversity {
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
    pub snapshot_timing: String,
    pub scramble_algorithm: String,
    pub per_seed: Vec<MemorySensitivitySeed>,
}

fn undefined_temporal_memory_sensitivity() -> Indicator<TemporalMemorySensitivity> {
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

fn undefined_mutational_neighborhood() -> Indicator<MutationalNeighborhood> {
    Indicator::Undefined(UNDEFINED.to_string())
}

fn undefined_evolved_neighborhood() -> Indicator<EvolvedNeighborhoodHalf> {
    Indicator::Undefined(UNDEFINED.to_string())
}

fn undefined_drift_depth() -> Indicator<DriftDepth> {
    Indicator::Undefined(UNDEFINED.to_string())
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

fn drift_checkpoint(row: neighborhood::drift::Checkpoint) -> DriftDepthCheckpoint {
    let mesh = row.mesh;
    let denominator = u64::from(mesh.lineages);
    let battery_executions = denominator * u64::from(neighborhood_battery_execution_count());
    DriftDepthCheckpoint {
        backends: Some(mesh.backends),
        depth: row.depth,
        lineages: mesh.lineages,
        total_nodes: mesh.total_nodes,
        reachable_nodes: mesh.reachable_nodes,
        executed_nodes: mesh.executed_nodes,
        knockout_nodes: mesh.knockout_nodes,
        mean_total_nodes: fraction_or_undefined(mesh.total_nodes, denominator),
        mean_reachable_nodes: fraction_or_undefined(mesh.reachable_nodes, denominator),
        mean_executed_nodes: fraction_or_undefined(mesh.executed_nodes, denominator),
        mean_knockout_nodes: fraction_or_undefined(mesh.knockout_nodes, denominator),
        route_varying_lineages: mesh.route_varying_lineages,
        route_varying_fraction: fraction_or_undefined(
            u64::from(mesh.route_varying_lineages),
            denominator,
        ),
        hop_cap_hits: mesh.hop_cap_hits,
        battery_executions,
        hop_cap_fraction: fraction_or_undefined(mesh.hop_cap_hits, battery_executions),
        silent_per_all_births: fraction_or_undefined(
            u64::from(row.births.any_events.silent),
            u64::from(row.births.births_total),
        ),
        changed_per_all_births: fraction_or_undefined(
            u64::from(row.births.any_events.changed),
            u64::from(row.births.births_total),
        ),
        dead_per_all_births: fraction_or_undefined(
            u64::from(row.births.any_events.dead),
            u64::from(row.births.births_total),
        ),
        births: to_neighborhood_births(&row.births),
    }
}

fn timed_drift_depth(
    params: &ProfileParams,
    config: &SimulationConfig,
    battery: Option<&Battery>,
) -> (Indicator<DriftDepth>, Option<f64>) {
    if params.name != "goal" && params.name != GOAL_WORLD_SET {
        return (undefined_drift_depth(), None);
    }
    let started = Instant::now();
    let reading = compute_drift_depth(config, battery.expect("goal battery"), params.drift);
    (Indicator::Defined(reading), Some(millis(started.elapsed())))
}

fn compute_drift_depth(
    config: &SimulationConfig,
    battery: &Battery,
    sizes: neighborhood::drift::DriftSizes,
) -> DriftDepth {
    use neighborhood::{battery as fixed_battery, drift, mesh_execution};
    let founder = founder_genome(v3_core::config::FounderProfile::V3Alpha1);
    let readings = drift::observe(
        &founder,
        battery,
        &config.mutation,
        &EvalContext::from_config(config),
        sizes,
    );
    DriftDepth {
        version: drift::VERSION.to_string(),
        founder: "V3Alpha1".to_string(),
        lineages: sizes.lineages,
        birth_lineages: sizes.birth_lineages,
        birth_subset: "first lineage indices in ascending order".to_string(),
        birth_trials: sizes.births,
        checkpoints: sizes.checkpoints.to_vec(),
        walk_seed_formula: format!("{} + lineage_index", drift::WALK_SEED_BASE),
        birth_seed_formula: format!(
            "{} + {} * (lineage_index + 1) + checkpoint + {} + trial_index",
            drift::BIRTH_OFFSET_BASE,
            drift::BIRTH_LINEAGE_MULTIPLIER,
            neighborhood::births::BIRTH_SEED_BASE
        ),
        battery_version: BATTERY_VERSION.to_string(),
        mesh_version: mesh_execution::MESH_EXECUTION_VERSION.to_string(),
        knockout_method: mesh_execution::KNOCKOUT_METHOD.to_string(),
        executed_source: drift::EXECUTED_SOURCE.to_string(),
        executed_refresh: drift::EXECUTED_REFRESH.to_string(),
        executions_per_genome: neighborhood_battery_execution_count(),
        snapshot_count: fixed_battery::SNAPSHOT_COUNT as u32,
        sequence_count: fixed_battery::SEQUENCE_COUNT as u32,
        sequence_len: fixed_battery::SEQUENCE_LEN as u32,
        readings: readings.into_iter().map(drift_checkpoint).collect(),
    }
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

fn undefined_generation_distribution() -> Indicator<GenerationDistribution> {
    Indicator::Undefined(UNDEFINED.to_string())
}

fn generation_distribution(mut generations: Vec<u64>) -> Indicator<GenerationDistribution> {
    if generations.is_empty() {
        return undefined_generation_distribution();
    }
    generations.sort_unstable();
    Indicator::Defined(GenerationDistribution {
        median: generations[generations.len() / 2],
        max: generations[generations.len() - 1],
    })
}

fn mesh_execution(
    battery: &Battery,
    genome: &v3_core::creature::genome::CreatureGenome,
    context: &EvalContext,
) -> Indicator<MeshExecution> {
    use v3_core::neighborhood::mesh_execution::{KNOCKOUT_METHOD, MESH_EXECUTION_VERSION};
    let reading = battery.mesh_execution(genome, context.runtime, context.shared_memory_decay_rate);
    Indicator::Defined(MeshExecution {
        backends: Some(reading.backends),
        version: MESH_EXECUTION_VERSION.to_string(),
        executions_per_genome: neighborhood_battery_execution_count(),
        snapshot_route_probes: neighborhood::battery::SNAPSHOT_COUNT as u32,
        knockout_method: KNOCKOUT_METHOD.to_string(),
        total_node_count: reading.total_node_count as u64,
        reachable_node_count: reading.reachable_node_count as u64,
        executed_node_count: reading.executed_node_count as u64,
        knockout_count: reading.knockout_count as u64,
        route_varies_with_input: reading.route_varies_with_input,
        hop_cap_hits: reading.hop_cap_hits as u64,
    })
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

/// Cumulative per-world behavior a baseline world is followed by (T12.F04):
/// what the population ate, how much food stood, and how often it walked into
/// something. Every field comes from applied simulation behavior, and every one
/// is serde-defaulted so reports stored before T12.F04 still parse.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldTracking {
    /// Applied Eat actions by the food type they consumed.
    #[serde(default)]
    pub typed_eats_total: Vec<u64>,
    /// Standing density per food type after the last executed tick's growth.
    #[serde(default)]
    pub food_density_total: Vec<String>,
    /// Move actions executed, blocked or not.
    #[serde(default)]
    pub moves_attempted_total: u64,
    /// Move actions a barrier blocked. The `barrier` entry of
    /// [`WorldTracking::moves_blocked_total_by_cause`], kept under its own key
    /// because reports stored before that map existed carry only this one.
    #[serde(default)]
    pub moves_blocked_barrier_total: u64,
    /// Every blocked move by what blocked it, straight from
    /// `SimStats::move_actions_blocked_total_by_cause`: barriers, an occupied
    /// target, and the world edge. Their sum is every move the population
    /// attempted and did not make. Absent — not zeroed — in a report stored
    /// before these totals were carried.
    #[serde(default)]
    pub moves_blocked_total_by_cause: Option<MovesBlockedByCause>,
    /// Blocked moves of any cause — barrier, occupancy, or edge — that had a
    /// valid alternative target, by reader state. Counted over every move the
    /// population made, not only the ones made beside a barrier.
    #[serde(default)]
    pub moves_blocked_avoidable_by_reader_state: ByReaderState<u64>,
    /// Move attempts made from a cell that had at least one neighboring
    /// barrier, by reader state: the only moves at which reading the barrier
    /// ring could have changed anything, and the denominator of
    /// [`TrackedFractions::barrier_blocked_fraction_by_reader_state`].
    #[serde(default)]
    pub move_attempts_with_barrier_neighbor_by_reader_state: ByReaderState<u64>,
    /// Of those attempts, the ones a barrier blocked: the numerator of the
    /// same fraction.
    #[serde(default)]
    pub moves_blocked_barrier_with_barrier_neighbor_by_reader_state: ByReaderState<u64>,
}

impl WorldTracking {
    /// Read the cumulative counters out of a running simulation.
    fn observe(sim: &v3_core::simulation::Simulation) -> Self {
        use std::collections::HashMap;
        use v3_core::config::OrdinaryFoodTypeId;
        use v3_core::simulation::actions::{BarrierReaderState, MoveBlockedCause};
        let stats = &sim.stats;
        let by_reader_state = |counts: &HashMap<BarrierReaderState, u64>| {
            let count = |state: BarrierReaderState| counts.get(&state).copied().unwrap_or_default();
            ByReaderState {
                has_barrier_reader: count(BarrierReaderState::HasBarrierReader),
                no_barrier_reader: count(BarrierReaderState::NoBarrierReader),
            }
        };
        let blocked = |cause: MoveBlockedCause| {
            stats
                .move_actions_blocked_total_by_cause
                .get(&cause)
                .copied()
                .unwrap_or_default()
        };
        let by_cause = MovesBlockedByCause {
            barrier: blocked(MoveBlockedCause::Barrier),
            occupied: blocked(MoveBlockedCause::Occupied),
            out_of_bounds: blocked(MoveBlockedCause::OutOfBounds),
        };
        Self {
            typed_eats_total: (0..sim.config.world.food.types.len())
                .map(|index| {
                    stats
                        .eat_actions_applied_total_by_type
                        .get(&OrdinaryFoodTypeId::new(index as u16))
                        .copied()
                        .unwrap_or(0)
                })
                .collect(),
            food_density_total: stats
                .last_tick_food_total_density_by_type
                .iter()
                .map(|density| six(f64::from(*density)))
                .collect(),
            moves_attempted_total: stats.move_actions_attempted_total,
            moves_blocked_barrier_total: by_cause.barrier,
            moves_blocked_total_by_cause: Some(by_cause),
            moves_blocked_avoidable_by_reader_state: by_reader_state(
                &stats.move_actions_blocked_avoidable_total_by_reader_state,
            ),
            move_attempts_with_barrier_neighbor_by_reader_state: by_reader_state(
                &stats.move_attempts_with_barrier_neighbor_total_by_reader_state,
            ),
            moves_blocked_barrier_with_barrier_neighbor_by_reader_state: by_reader_state(
                &stats.move_blocked_barrier_with_barrier_neighbor_total_by_reader_state,
            ),
        }
    }

    /// The rates these totals imply, each against its own denominator: a
    /// type's eat share against every applied Eat, `blocked_move_fraction` and
    /// the avoidable share against every move attempted, and the
    /// barrier-blocked fraction against that reader state's own attempts made
    /// beside a barrier. Only the last is a per-state rate.
    fn fractions(&self) -> TrackedFractions {
        let eats: u64 = self.typed_eats_total.iter().sum();
        let attempted = self.moves_attempted_total;
        let avoidable = &self.moves_blocked_avoidable_by_reader_state;
        let beside = &self.move_attempts_with_barrier_neighbor_by_reader_state;
        let blocked_beside = &self.moves_blocked_barrier_with_barrier_neighbor_by_reader_state;
        TrackedFractions {
            typed_eat_share: self
                .typed_eats_total
                .iter()
                .map(|typed| fraction_or_undefined(*typed, eats))
                .collect(),
            blocked_move_fraction: fraction_or_undefined(
                self.moves_blocked_barrier_total,
                attempted,
            ),
            barrier_blocked_fraction_by_reader_state: ByReaderState {
                has_barrier_reader: fraction_or_undefined(
                    blocked_beside.has_barrier_reader,
                    beside.has_barrier_reader,
                ),
                no_barrier_reader: fraction_or_undefined(
                    blocked_beside.no_barrier_reader,
                    beside.no_barrier_reader,
                ),
            },
            avoidable_blocked_share_of_all_moves_by_reader_state: ByReaderState {
                has_barrier_reader: fraction_or_undefined(avoidable.has_barrier_reader, attempted),
                no_barrier_reader: fraction_or_undefined(avoidable.no_barrier_reader, attempted),
            },
        }
    }
}

/// One persistence sample, taken after `run_tick` on every executed tick that
/// is a multiple of [`SAMPLE_EVERY_TICKS`] and on the last executed tick.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceSample {
    pub tick: u64,
    pub population: u64,
    pub mean_energy: Option<String>,
    /// Cumulative `reproduction_actions_spawned_total` at this tick.
    pub births_total: u64,
    #[serde(flatten)]
    pub tracking: WorldTracking,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructureSizeDistribution {
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
    /// Evolved-half neighborhood wall time per seed (goal profile only).
    #[serde(default)]
    pub neighborhood_evolved_wall_clock_ms_per_seed: Vec<SeedFinalStateObservation>,
    #[serde(default)]
    pub neighborhood_evolved_wall_clock_ms_total: f64,
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

/// Derive throughput rates from one run's own counters and its own elapsed
/// wall-clock. A non-positive elapsed time yields zero rates rather than
/// infinity or NaN, so a report can always be serialized as JSON.
#[must_use]
pub fn throughput_rates(
    ticks: u64,
    creature_ticks: u64,
    births: u64,
    wall_clock_ms: f64,
) -> ThroughputRates {
    let seconds = wall_clock_ms / 1000.0;
    let per_second = |count: u64| {
        if seconds > 0.0 {
            count as f64 / seconds
        } else {
            0.0
        }
    };
    ThroughputRates {
        ticks_per_second: per_second(ticks),
        creature_ticks_per_second: per_second(creature_ticks),
        ticks_per_hour: per_second(ticks) * 3600.0,
        births_per_hour: per_second(births) * 3600.0,
    }
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Comparison {
    pub references: Vec<ReferenceComparison>,
    pub severe: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceComparison {
    pub path: String,
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
const VALUE_ONLY_CASE_READINGS: [&str; 1] = ["extinction_tick"];

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

fn ratio(numerator: u64, denominator: u64) -> String {
    if denominator == 0 {
        return six(0.0);
    }
    six(numerator as f64 / denominator as f64)
}

// ── Per-seed persistence tracking (T01.F11) ─────────────────────────────────

/// Accumulates the per-seed persistence observations defined by the T01.F11
/// spec. It never touches the simulation, so it is unit-testable from
/// synthetic observations alone.
#[derive(Debug)]
struct PersistenceAccumulator {
    horizon: u64,
    population: u64,
    minimum_population: u64,
    peak_population: u64,
    peak_tick: u64,
    extinction_tick: Option<u64>,
    plateau_sum: u64,
    plateau_ticks: u64,
    samples: Vec<PersistenceSample>,
}

impl PersistenceAccumulator {
    /// `seeded_population` is the founder count observed at tick 0, before
    /// any tick has run: the first peak and minimum candidate.
    fn new(horizon: u64, seeded_population: u64) -> Self {
        Self {
            horizon,
            population: seeded_population,
            minimum_population: seeded_population,
            peak_population: seeded_population,
            peak_tick: 0,
            extinction_tick: None,
            plateau_sum: 0,
            plateau_ticks: 0,
            samples: Vec::new(),
        }
    }

    /// The plateau window is the ticks strictly after `0.75 x horizon`,
    /// compared in exact integer arithmetic.
    fn in_plateau_window(&self, tick: u64) -> bool {
        tick * 4 > self.horizon * 3
    }

    /// A tick is sampled when it is a multiple of [`SAMPLE_EVERY_TICKS`] or is
    /// the last executed tick — the horizon, or the extinction tick that ends
    /// the run early. One condition, so the final tick is never duplicated.
    fn is_sampled(&self, tick: u64, population: u64) -> bool {
        tick.is_multiple_of(SAMPLE_EVERY_TICKS) || tick == self.horizon || population == 0
    }

    /// Record one executed tick. `mean_energy` and `tracking` are evaluated
    /// only on sampled ticks — the first keeps the `O(population)` energy sum
    /// to the predeclared cadence and leaves `null` at extinction, the second
    /// keeps the per-world counter reads there too.
    fn observe(
        &mut self,
        tick: u64,
        population: u64,
        births_total: u64,
        mean_energy: impl FnOnce() -> f64,
        tracking: impl FnOnce() -> WorldTracking,
    ) {
        self.population = population;
        self.minimum_population = self.minimum_population.min(population);
        if population > self.peak_population {
            self.peak_population = population;
            self.peak_tick = tick;
        }
        if population == 0 && self.extinction_tick.is_none() {
            self.extinction_tick = Some(tick);
        }
        if self.in_plateau_window(tick) {
            self.plateau_sum += population;
            self.plateau_ticks += 1;
        }
        if self.is_sampled(tick, population) {
            self.samples.push(PersistenceSample {
                tick,
                population,
                mean_energy: (population > 0).then(|| six(mean_energy())),
                births_total,
                tracking: tracking(),
            });
        }
    }

    fn finish(self, seed: u64) -> PopulationPersistenceSeed {
        PopulationPersistenceSeed {
            seed,
            extinction_tick: self.extinction_tick,
            minimum_population: self.minimum_population,
            final_population: self.population,
            peak_population: self.peak_population,
            peak_tick: self.peak_tick,
            plateau_population: (self.plateau_ticks > 0)
                .then(|| ratio(self.plateau_sum, self.plateau_ticks)),
            // The last executed tick is always sampled, so the run's final
            // mean energy is the last sample's.
            mean_energy: self.samples.last().and_then(|s| s.mean_energy.clone()),
            samples: self.samples,
        }
    }
}

// ── Seed execution ───────────────────────────────────────────────────────────

struct SeedRun {
    per_seed: PerSeed,
    persistence: PopulationPersistenceSeed,
    /// The run's final cumulative per-world behavior.
    tracking: WorldTracking,
    complexities: Vec<u32>,
    goal_observation: Option<GoalObservation>,
    wall_clock_ms: f64,
    phase_wall_clock: SeedPhaseWallClock,
    throughput: SeedThroughput,
}

struct GoalObservation {
    lineage_diversity: LineageDiversitySeed,
    memory_sensitivity: MemorySensitivitySeed,
    temporal_memory_sensitivity: TemporalMemorySensitivitySeed,
    wall_clock_ms: f64,
    /// `Some` only in the goal profile, where the evolved-genome half of the
    /// mutational-neighborhood indicator runs.
    evolved_neighborhood: Option<NeighborhoodEvolvedSeed>,
    evolved_neighborhood_wall_clock_ms: f64,
}

/// `Duration` as fractional milliseconds, the unit every wall-clock field in
/// the `environment` block uses. `Duration::as_millis_f64` is still unstable
/// on the pinned toolchain.
fn millis(duration: std::time::Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}

/// Every wall-clock observation of a run, for the caller to fold into the
/// `environment` block. Nothing here may enter the `deterministic` block.
pub struct RunTimings {
    pub wall_clock_ms_per_seed: Vec<SeedWallClock>,
    pub phase_wall_clock_ms_per_seed: Vec<SeedPhaseWallClock>,
    pub throughput_per_seed: Vec<SeedThroughput>,
    pub final_state_observation_ms_per_seed: Vec<SeedFinalStateObservation>,
    /// Founder-half neighborhood wall time (T11.F01), computed once per
    /// report outside every per-seed timing above.
    pub neighborhood_founder_wall_clock_ms: f64,
    /// Complete goal-only mutation walk and readings; absent when unrun.
    pub drift_depth_wall_clock_ms: Option<f64>,
    /// Evolved-half neighborhood wall time per seed (goal profile only).
    pub neighborhood_evolved_wall_clock_ms_per_seed: Vec<SeedFinalStateObservation>,
}

fn run_one_seed(
    config: &SimulationConfig,
    seed: u64,
    horizon: u64,
    observe_goal_indicators: bool,
    neighborhood_battery: Option<&Battery>,
    neighborhood_sizes: NeighborhoodSizes,
) -> SeedRun {
    let start = Instant::now();
    let mut sim = seed_simulation(config.clone(), seed);
    let observation_start = Instant::now();
    let tick_zero_connectivity = sim.world.passable_connectivity();
    let connectivity_duration = observation_start.elapsed();
    let mut persistence = PersistenceAccumulator::new(horizon, sim.creatures.len() as u64);

    let mut ticks_executed: u64 = 0;
    for _ in 0..horizon {
        run_tick(&mut sim, &mut None);
        ticks_executed += 1;
        let population = sim.creatures.len() as u64;
        persistence.observe(
            sim.tick,
            population,
            sim.stats.reproduction_actions_spawned_total,
            || {
                let total: f64 = sim.creatures.values().map(|c| f64::from(c.energy)).sum();
                total / population as f64
            },
            || WorldTracking::observe(&sim),
        );
        if population == 0 {
            break;
        }
    }
    let wall_clock_ms = millis(start.elapsed().saturating_sub(connectivity_duration));

    let tracking = WorldTracking::observe(&sim);
    let persistence = persistence.finish(seed);
    let complexities: Vec<u32> = sim
        .creatures
        .values()
        .map(|c| functional_complexity(&c.genome))
        .collect();
    let goal_observation = observe_goal_indicators.then(|| {
        let observation_started = Instant::now();
        let actions = observe_final_actions(&sim);
        let lineage_diversity_seed = lineage_diversity(
            seed,
            sim.creatures
                .values()
                .map(|creature| creature.identity.lineage_id),
        );
        let memory_sensitivity_seed = memory_sensitivity(seed, &actions);
        let temporal_memory_sensitivity = temporal_memory_sensitivity(seed, &sim);
        let wall_clock_ms = millis(observation_started.elapsed());

        let evolved_neighborhood_started = Instant::now();
        let evolved_neighborhood = neighborhood_battery.map(|battery| {
            let context = EvalContext::from_config(config);
            evolved_neighborhood_for_seed(
                seed,
                &sim,
                battery,
                &config.mutation,
                &context,
                neighborhood_sizes,
            )
        });
        let evolved_neighborhood_wall_clock_ms = millis(evolved_neighborhood_started.elapsed());

        GoalObservation {
            lineage_diversity: lineage_diversity_seed,
            memory_sensitivity: memory_sensitivity_seed,
            temporal_memory_sensitivity,
            wall_clock_ms,
            evolved_neighborhood,
            evolved_neighborhood_wall_clock_ms,
        }
    });

    let per_seed = PerSeed {
        tick_zero_connectivity: Some(tick_zero_connectivity),
        seed,
        ticks: ticks_executed,
        creature_ticks: sim.stats.creature_ticks_total,
        mesh_hops: sim.stats.mesh_hops_total,
        vm_steps: sim.stats.vm_steps_total,
        graph_relax_iters: sim.stats.graph_relax_iters_total,
        plasticity_updates: sim.stats.plasticity_updates_total,
        actions_applied: sim.stats.actions_applied_total,
        births: sim.stats.reproduction_actions_spawned_total,
        final_population: persistence.final_population,
        extinction_tick: persistence.extinction_tick,
    };

    let phases = sim.stats.phase_wall_clock;
    let throughput = SeedThroughput {
        seed,
        rates: throughput_rates(
            per_seed.ticks,
            per_seed.creature_ticks,
            per_seed.births,
            wall_clock_ms,
        ),
    };
    SeedRun {
        per_seed,
        persistence,
        tracking,
        complexities,
        goal_observation,
        wall_clock_ms,
        throughput,
        phase_wall_clock: SeedPhaseWallClock {
            seed,
            world_update_ms: millis(phases.world_update),
            sensor_assembly_ms: millis(phases.sensor_assembly),
            cognition_ms: millis(phases.cognition),
            actions_ms: millis(phases.actions),
            reward_learning_ms: millis(phases.reward_learning),
        },
    }
}

fn lineage_diversity(
    seed: u64,
    lineage_ids: impl IntoIterator<Item = u32>,
) -> LineageDiversitySeed {
    let mut counts = BTreeMap::<u32, u64>::new();
    for lineage_id in lineage_ids {
        *counts.entry(lineage_id).or_default() += 1;
    }
    let total: u64 = counts.values().sum();
    let shannon_entropy_nats = if total == 0 {
        UNDEFINED.to_string()
    } else {
        let total = total as f64;
        let entropy = counts.values().fold(0.0, |acc, &count| {
            let probability = count as f64 / total;
            acc - probability * probability.ln()
        });
        six(entropy)
    };
    LineageDiversitySeed {
        seed,
        surviving_founder_clade_count: counts.len() as u64,
        shannon_entropy_nats,
    }
}

fn memory_sensitivity(seed: u64, observations: &[FinalActionObservation]) -> MemorySensitivitySeed {
    let final_creature_count = observations.len() as u64;
    let (different_from_zeroed_count, different_from_scrambled_count, different_from_either_count) =
        observations.iter().fold((0, 0, 0), |counts, observation| {
            let different_from_zeroed = observation.intact != observation.zeroed;
            let different_from_scrambled = observation.intact != observation.scrambled;
            (
                counts.0 + u64::from(different_from_zeroed),
                counts.1 + u64::from(different_from_scrambled),
                counts.2 + u64::from(different_from_zeroed || different_from_scrambled),
            )
        });
    MemorySensitivitySeed {
        seed,
        final_creature_count,
        different_from_zeroed_count,
        different_from_scrambled_count,
        different_from_either_count,
        different_from_zeroed_fraction: fraction_or_undefined(
            different_from_zeroed_count,
            final_creature_count,
        ),
        different_from_scrambled_fraction: fraction_or_undefined(
            different_from_scrambled_count,
            final_creature_count,
        ),
        different_from_either_fraction: fraction_or_undefined(
            different_from_either_count,
            final_creature_count,
        ),
    }
}

fn temporal_memory_sensitivity(
    seed: u64,
    sim: &v3_core::simulation::Simulation,
) -> TemporalMemorySensitivitySeed {
    use v3_core::simulation::{observe_temporal_actions, TemporalMemorySubstrate};
    let observations = observe_temporal_actions(sim);
    let component = |substrate| {
        let actions: Vec<_> = observations
            .iter()
            .filter(|observation| observation.substrate == substrate)
            .map(|observation| observation.actions.clone())
            .collect();
        memory_sensitivity(seed, &actions)
    };
    TemporalMemorySensitivitySeed {
        seed,
        previous_slots: component(TemporalMemorySubstrate::PreviousSlots),
        persisted_outputs: component(TemporalMemorySubstrate::PersistedOutputs),
        operator_state: component(TemporalMemorySubstrate::OperatorState),
    }
}

// ── Mutational neighborhood (T11.F01) ───────────────────────────────────────

/// The battery's total executions per genome: every single-tick snapshot
/// plus every sequence's ticks.
fn neighborhood_battery_execution_count() -> u32 {
    (neighborhood::battery::SNAPSHOT_COUNT
        + neighborhood::battery::SEQUENCE_COUNT * neighborhood::battery::SEQUENCE_LEN) as u32
}

fn to_neighborhood_tally(tally: &Tally) -> NeighborhoodTally {
    let applied = tally.applied();
    NeighborhoodTally {
        trials: tally.trials,
        skipped: tally.skipped,
        applied,
        silent: tally.silent,
        changed: tally.changed,
        dead: tally.dead,
        changed_only_in_sequences: tally.changed_only_in_sequences,
        silent_fraction: fraction_or_undefined(tally.silent.into(), applied.into()),
        changed_fraction: fraction_or_undefined(tally.changed.into(), applied.into()),
        dead_fraction: fraction_or_undefined(tally.dead.into(), applied.into()),
        mean_fraction_differing: six(tally.mean_fraction_differing()),
    }
}

fn to_neighborhood_operator_rows(rows: &[OperatorRow]) -> Vec<NeighborhoodOperatorRow> {
    rows.iter()
        .map(|row| NeighborhoodOperatorRow {
            family: row.family.to_string(),
            operator: row.operator.clone(),
            tally: to_neighborhood_tally(&row.tally),
        })
        .collect()
}

fn to_neighborhood_births(result: &BirthResult) -> NeighborhoodBirths {
    NeighborhoodBirths {
        births_total: result.births_total,
        by_requested_events: result
            .by_requested_events
            .iter()
            .map(
                |(&requested_events, &births)| NeighborhoodRequestedBirthBucket {
                    requested_events,
                    births,
                },
            )
            .collect(),
        zero_event_births: result.zero_event_births,
        any_events: to_neighborhood_tally(&result.any_events),
        by_events: result
            .by_events
            .iter()
            .map(|(&applied_events, tally)| NeighborhoodBirthBucket {
                applied_events,
                tally: to_neighborhood_tally(tally),
            })
            .collect(),
    }
}

fn to_neighborhood_companions(companions: &StructuralCompanions) -> NeighborhoodCompanions {
    NeighborhoodCompanions {
        functional_complexity: companions.functional_complexity,
        reachable_node_count: companions.reachable_node_count as u64,
        reads_shared_memory: companions.reads_shared_memory,
        writes_shared_memory: companions.writes_shared_memory,
        has_stateful_compute_node: companions.has_stateful_compute_node,
        has_plasticity: companions.has_plasticity,
    }
}

/// The founder half: always computed when `mutational_neighborhood` is
/// defined (gate and goal), once per report — outside the per-seed loop,
/// since it depends only on the founder genome and the production mutation
/// config, never on a world trajectory.
fn compute_founder_neighborhood(
    config: &SimulationConfig,
    battery: &Battery,
    sizes: NeighborhoodSizes,
) -> NeighborhoodFounderHalf {
    let subject = founder_genome(config.population.founder_profile);
    let context = EvalContext::from_config(config);
    let evaluation: GenomeEvaluation = evaluate_genome(
        &subject,
        battery,
        &config.mutation,
        &context,
        sizes.founder_operator_trials,
        sizes.founder_births,
        0,
    );
    NeighborhoodFounderHalf {
        generation: Some(0),
        mesh_execution: mesh_execution(battery, &subject, &context),
        reachable_node_count: structural_companions(&subject).reachable_node_count as u64,
        operator_rows: to_neighborhood_operator_rows(&evaluation.operator_rows),
        births: to_neighborhood_births(&evaluation.births),
    }
}

/// The evolved half for one seed's final living population: the predeclared
/// rank sample, each sampled genome's full reading and structural
/// companions, and the pooled per-operator and per-birth tallies across the
/// sample.
fn evolved_neighborhood_for_seed(
    seed: u64,
    sim: &v3_core::simulation::Simulation,
    battery: &Battery,
    mutation_config: &MutationConfig,
    context: &EvalContext,
    sizes: NeighborhoodSizes,
) -> NeighborhoodEvolvedSeed {
    let mut creature_ids: Vec<_> = sim.creatures.keys().collect();
    creature_ids.sort();
    let population_size = creature_ids.len();
    let ranks = evolved_sample_ranks(population_size);
    let catalog = v3_core::neighborhood::operator_catalog();

    let mut sampled_genomes = Vec::with_capacity(ranks.len());
    let mut pooled_operator_tallies: Vec<Tally> = vec![Tally::default(); catalog.len()];
    let mut pooled_births = BirthResult::default();

    for (genome_index, &rank) in ranks.iter().enumerate() {
        let creature_id = creature_ids[rank];
        let creature = &sim.creatures[creature_id];
        let seed_offset =
            v3_core::neighborhood::EVOLVED_SEED_MULTIPLIER * (genome_index as u64 + 1);
        let evaluation = evaluate_genome(
            &creature.genome,
            battery,
            mutation_config,
            context,
            sizes.evolved_operator_trials,
            sizes.evolved_births,
            seed_offset,
        );
        let companions = structural_companions(&creature.genome);

        for (pooled, row) in pooled_operator_tallies
            .iter_mut()
            .zip(&evaluation.operator_rows)
        {
            *pooled = pooled.merge(row.tally);
        }
        pooled_births = pooled_births.merge(&evaluation.births);

        sampled_genomes.push(NeighborhoodSampledGenome {
            generation: Some(creature.generation),
            mesh_execution: mesh_execution(battery, &creature.genome, context),
            rank: rank as u64,
            creature_id: format!("{creature_id:?}"),
            operator_rows: to_neighborhood_operator_rows(&evaluation.operator_rows),
            births: to_neighborhood_births(&evaluation.births),
            companions: to_neighborhood_companions(&companions),
        });
    }

    let pooled_operator_rows = catalog
        .iter()
        .zip(pooled_operator_tallies.iter())
        .map(|((family, name), tally)| NeighborhoodOperatorRow {
            family: (*family).to_string(),
            operator: name.clone(),
            tally: to_neighborhood_tally(tally),
        })
        .collect();

    NeighborhoodEvolvedSeed {
        generation_distribution: generation_distribution(
            sim.creatures
                .values()
                .map(|creature| creature.generation)
                .collect(),
        ),
        seed,
        final_population_size: population_size as u64,
        sampled_genomes,
        pooled_operator_rows,
        pooled_births: to_neighborhood_births(&pooled_births),
    }
}

fn percentile(sorted: &[u32], p: f64) -> u32 {
    if sorted.is_empty() {
        return 0;
    }
    let idx = (p / 100.0 * (sorted.len() - 1) as f64).round();
    let idx = idx.clamp(0.0, (sorted.len() - 1) as f64) as usize;
    sorted[idx]
}

fn structure_size_distribution(mut pooled: Vec<u32>) -> StructureSizeDistribution {
    if pooled.is_empty() {
        return StructureSizeDistribution {
            min: 0,
            p25: 0,
            median: 0,
            p75: 0,
            max: 0,
            mean: six(0.0),
        };
    }
    pooled.sort_unstable();
    let sum: u64 = pooled.iter().map(|&v| u64::from(v)).sum();
    let mean = sum as f64 / pooled.len() as f64;
    StructureSizeDistribution {
        min: pooled[0],
        p25: percentile(&pooled, 25.0),
        median: percentile(&pooled, 50.0),
        p75: percentile(&pooled, 75.0),
        max: pooled[pooled.len() - 1],
        mean: six(mean),
    }
}

/// Assemble the report's `mutational_neighborhood` indicator (T11.F01):
/// `Undefined` when the founder half did not run (the sweep and synthetic
/// profiles), otherwise the battery block, the founder half, and the evolved
/// half (only defined for the goal profile).
fn build_mutational_neighborhood_indicator(
    neighborhood_founder: Option<NeighborhoodFounderHalf>,
    params: &ProfileParams,
    observe_goal_indicators: bool,
    evolved_neighborhood_per_seed: Vec<NeighborhoodEvolvedSeed>,
) -> Indicator<MutationalNeighborhood> {
    let Some(founder) = neighborhood_founder else {
        return undefined_mutational_neighborhood();
    };
    Indicator::Defined(MutationalNeighborhood {
        battery: NeighborhoodBattery {
            version: BATTERY_VERSION.to_string(),
            snapshot_seed: neighborhood::battery::SNAPSHOT_SEED,
            sequence_seed: neighborhood::battery::SEQUENCE_SEED,
            snapshot_count: neighborhood::battery::SNAPSHOT_COUNT as u32,
            sequence_count: neighborhood::battery::SEQUENCE_COUNT as u32,
            sequence_len: neighborhood::battery::SEQUENCE_LEN as u32,
            executions_per_genome: neighborhood_battery_execution_count(),
            founder_operator_trials: params.neighborhood.founder_operator_trials,
            founder_birth_count: params.neighborhood.founder_births,
            evolved_operator_trials: params.neighborhood.evolved_operator_trials,
            evolved_birth_count: params.neighborhood.evolved_births,
            evolved_sample_size: neighborhood::SAMPLE_SIZE as u32,
        },
        founder,
        evolved: if observe_goal_indicators {
            Indicator::Defined(EvolvedNeighborhoodHalf {
                per_seed: evolved_neighborhood_per_seed,
            })
        } else {
            undefined_evolved_neighborhood()
        },
    })
}

struct PreparedGoalCase {
    case: GoalCase,
    config: SimulationConfig,
    battery: Battery,
    founder: NeighborhoodFounderHalf,
    drift: Indicator<DriftDepth>,
}

fn prepare_goal_case(
    params: &ProfileParams,
    recipe: &GoalRecipe,
    founder_ms: &mut f64,
    drift_ms: &mut Option<f64>,
) -> PreparedGoalCase {
    let (case, config) = goal_case(params, recipe);
    let battery = Battery::generate(config.world.food.types.len());
    let start = Instant::now();
    let founder = compute_founder_neighborhood(&config, &battery, params.neighborhood);
    *founder_ms += millis(start.elapsed());
    let (drift, duration) = timed_drift_depth(params, &config, Some(&battery));
    *drift_ms.as_mut().expect("world-set timing") += duration.expect("goal timing");
    PreparedGoalCase {
        case,
        config,
        battery,
        founder,
        drift,
    }
}

/// Sum every executed case or seed run into the profile totals. Each per-case
/// row is one world in the world-set profile and one seed replicate elsewhere,
/// so the profile total is always the field-wise sum of the rows the report
/// carries.
fn accumulate_totals(per_seed: &[PerSeed]) -> Totals {
    per_seed.iter().fold(Totals::default(), |mut totals, row| {
        totals.ticks += row.ticks;
        totals.creature_ticks += row.creature_ticks;
        totals.mesh_hops += row.mesh_hops;
        totals.vm_steps += row.vm_steps;
        totals.graph_relax_iters += row.graph_relax_iters;
        totals.plasticity_updates += row.plasticity_updates;
        totals.actions_applied += row.actions_applied;
        totals.births += row.births;
        totals
    })
}

fn normalized_totals(totals: &Totals) -> PerCreatureTick {
    PerCreatureTick {
        mesh_hops: Some(ratio(totals.mesh_hops, totals.creature_ticks)),
        vm_steps: Some(ratio(totals.vm_steps, totals.creature_ticks)),
        graph_relax_iters: Some(ratio(totals.graph_relax_iters, totals.creature_ticks)),
        plasticity_updates: Some(ratio(totals.plasticity_updates, totals.creature_ticks)),
        actions_applied: Some(ratio(totals.actions_applied, totals.creature_ticks)),
        births: Some(ratio(totals.births, totals.creature_ticks)),
    }
}

struct GoalIndicatorInputs {
    population_persistence_per_seed: Vec<PopulationPersistenceSeed>,
    lineage_diversity_per_seed: Vec<LineageDiversitySeed>,
    memory_sensitivity_per_seed: Vec<MemorySensitivitySeed>,
    temporal_memory_sensitivity_per_seed: Vec<TemporalMemorySensitivitySeed>,
    evolved_neighborhood_per_seed: Vec<NeighborhoodEvolvedSeed>,
    pooled_complexities: Vec<u32>,
    neighborhood_founder: Option<NeighborhoodFounderHalf>,
    drift_depth: Indicator<DriftDepth>,
    case_observations: Vec<GoalCaseObservation>,
}

fn assemble_goal_indicators(
    params: &ProfileParams,
    totals: &Totals,
    inputs: GoalIndicatorInputs,
) -> GoalIndicators {
    let GoalIndicatorInputs {
        population_persistence_per_seed,
        lineage_diversity_per_seed,
        memory_sensitivity_per_seed,
        temporal_memory_sensitivity_per_seed,
        evolved_neighborhood_per_seed,
        pooled_complexities,
        neighborhood_founder,
        drift_depth,
        case_observations,
    } = inputs;
    let world_set = params.name == GOAL_WORLD_SET;
    let observe_goal_indicators = params.name == "goal" || world_set;
    let population_persistence = PopulationPersistence {
        per_seed: population_persistence_per_seed,
    };

    let births_per_100_ticks = if totals.ticks == 0 {
        six(0.0)
    } else {
        six(totals.births as f64 / totals.ticks as f64 * 100.0)
    };

    GoalIndicators {
        cases: case_observations,
        population_persistence,
        births_per_100_ticks,
        reachable_structure_size_distribution: structure_size_distribution(pooled_complexities),
        lineage_diversity: if observe_goal_indicators {
            Indicator::Defined(LineageDiversity {
                per_seed: lineage_diversity_per_seed,
            })
        } else {
            undefined_lineage_diversity()
        },
        memory_sensitivity: if observe_goal_indicators {
            Indicator::Defined(MemorySensitivity {
                snapshot_timing: "after the final executed tick, before any observation action"
                    .to_string(),
                scramble_algorithm: "rotate_left(1) across 16 shared-memory slots".to_string(),
                per_seed: memory_sensitivity_per_seed,
            })
        } else {
            undefined_memory_sensitivity()
        },
        temporal_memory_sensitivity: if observe_goal_indicators {
            Indicator::Defined(TemporalMemorySensitivity {
                version: "temporal-memory-v1".to_string(),
                snapshot_timing: "hypothetical cognition from final committed graph state, with final sensors and current/previous shared-memory slots held fixed; no world tick or shared-memory snapshot/decay".to_string(),
                scramble_algorithm: "rotate_left(1) independently within each named slot vector, after graph tick preparation".to_string(),
                per_seed: temporal_memory_sensitivity_per_seed,
            })
        } else {
            undefined_temporal_memory_sensitivity()
        },
        mutational_neighborhood: if world_set {
            Indicator::Undefined("reported per case".to_string())
        } else {
            build_mutational_neighborhood_indicator(
                neighborhood_founder,
                params,
                observe_goal_indicators,
                evolved_neighborhood_per_seed,
            )
        },
        drift_depth,
        strategy_count: UNDEFINED.to_string(),
        strategy_causal_distinctness: UNDEFINED.to_string(),
        evolutionary_activity: UNDEFINED.to_string(),
        adaptive_novelty: UNDEFINED.to_string(),
        memory_dependence: UNDEFINED.to_string(),
        learning_dependence: UNDEFINED.to_string(),
        prediction_dependence: UNDEFINED.to_string(),
        information_integration: UNDEFINED.to_string(),
        reciprocal_interaction: UNDEFINED.to_string(),
    }
}

/// Run the deterministic profile (no host/timestamp data) and return the
/// `Deterministic` block plus the run's wall-clock observations for the
/// caller to fold into the `environment` block.
pub fn run_deterministic(params: &ProfileParams) -> Result<(Deterministic, RunTimings), String> {
    let recipes = goal_recipes_for(params)?;
    let config = build_config(params);

    let mut per_seed = Vec::with_capacity(params.seeds.len());
    let mut wall_clock = Vec::with_capacity(params.seeds.len());
    let mut phase_wall_clock = Vec::with_capacity(params.seeds.len());
    let mut throughput_per_seed = Vec::with_capacity(params.seeds.len());
    let mut pooled_complexities: Vec<u32> = Vec::new();
    let mut population_persistence_per_seed = Vec::with_capacity(params.seeds.len());
    let mut lineage_diversity_per_seed = Vec::with_capacity(params.seeds.len());
    let mut memory_sensitivity_per_seed = Vec::with_capacity(params.seeds.len());
    let mut temporal_memory_sensitivity_per_seed = Vec::with_capacity(params.seeds.len());
    let mut final_state_observation_ms_per_seed = Vec::with_capacity(params.seeds.len());
    let mut evolved_neighborhood_per_seed = Vec::with_capacity(params.seeds.len());
    let mut neighborhood_evolved_wall_clock_ms_per_seed = Vec::with_capacity(params.seeds.len());
    let world_set = params.name == GOAL_WORLD_SET;
    let observe_goal_indicators = params.name == "goal" || world_set;
    let mut case_observations = Vec::new();

    // The founder half runs for exactly the gate and goal profiles (never
    // sweep, and never a test-only or synthetic profile name), and only once
    // per report — it depends only on the founder genome and the production
    // mutation config, never on any seed's world trajectory, so it is
    // computed outside the per-seed loop and timed separately from every
    // per-seed wall-clock field.
    let run_neighborhood = !world_set && (params.name == "gate" || observe_goal_indicators);
    let neighborhood_battery =
        run_neighborhood.then(|| Battery::generate(config.world.food.types.len()));
    let neighborhood_founder_start = Instant::now();
    let neighborhood_founder = neighborhood_battery
        .as_ref()
        .map(|battery| compute_founder_neighborhood(&config, battery, params.neighborhood));
    let mut neighborhood_founder_wall_clock_ms = millis(neighborhood_founder_start.elapsed());
    let (drift_depth, mut drift_depth_wall_clock_ms) = if world_set {
        (
            Indicator::Undefined("reported per case".to_string()),
            Some(0.0),
        )
    } else {
        timed_drift_depth(params, &config, neighborhood_battery.as_ref())
    };

    for (index, &seed) in params.seeds.iter().enumerate() {
        // `recipes` is empty off the world set and exactly one entry per seed
        // on it, so this pairs each seed with its own world and never falls
        // back to the shared config for a world-set case.
        let case = recipes.get(index).map(|recipe| {
            prepare_goal_case(
                params,
                recipe,
                &mut neighborhood_founder_wall_clock_ms,
                &mut drift_depth_wall_clock_ms,
            )
        });
        let case_config = case.as_ref().map_or(&config, |case| &case.config);
        let battery = case
            .as_ref()
            .map(|case| &case.battery)
            .or(neighborhood_battery.as_ref());
        // `run_one_seed` only reads `neighborhood_battery` inside its own
        // `observe_goal_indicators`-gated closure, so passing it unconditionally
        // here is equivalent to nulling it out for non-goal profiles and one
        // branch simpler.
        let run = run_one_seed(
            case_config,
            seed,
            params.ticks,
            observe_goal_indicators,
            battery,
            params.neighborhood,
        );
        pooled_complexities.extend(run.complexities.iter().copied());
        wall_clock.push(SeedWallClock {
            seed,
            wall_clock_ms: run.wall_clock_ms,
        });
        throughput_per_seed.push(run.throughput);
        phase_wall_clock.push(run.phase_wall_clock);
        population_persistence_per_seed.push(run.persistence);
        let mut case_evolved = Vec::new();
        if let Some(observation) = run.goal_observation {
            lineage_diversity_per_seed.push(observation.lineage_diversity);
            memory_sensitivity_per_seed.push(observation.memory_sensitivity);
            temporal_memory_sensitivity_per_seed.push(observation.temporal_memory_sensitivity);
            final_state_observation_ms_per_seed.push(SeedFinalStateObservation {
                seed,
                wall_clock_ms: observation.wall_clock_ms,
            });
            if let Some(evolved) = observation.evolved_neighborhood {
                if world_set {
                    case_evolved.push(evolved);
                } else {
                    evolved_neighborhood_per_seed.push(evolved);
                }
                neighborhood_evolved_wall_clock_ms_per_seed.push(SeedFinalStateObservation {
                    seed,
                    wall_clock_ms: observation.evolved_neighborhood_wall_clock_ms,
                });
            }
        }
        if let Some(case) = case {
            case_observations.push(GoalCaseObservation {
                case: case.case,
                reachable_structure_size_distribution: Some(structure_size_distribution(
                    run.complexities,
                )),
                fractions: run.tracking.fractions(),
                tracking: run.tracking,
                mutational_neighborhood: build_mutational_neighborhood_indicator(
                    Some(case.founder),
                    params,
                    true,
                    case_evolved,
                ),
                drift_depth: case.drift,
            });
        }
        per_seed.push(run.per_seed);
    }

    let totals = accumulate_totals(&per_seed);
    let per_creature_tick = normalized_totals(&totals);

    let goal_indicators = assemble_goal_indicators(
        params,
        &totals,
        GoalIndicatorInputs {
            population_persistence_per_seed,
            lineage_diversity_per_seed,
            memory_sensitivity_per_seed,
            temporal_memory_sensitivity_per_seed,
            evolved_neighborhood_per_seed,
            pooled_complexities,
            neighborhood_founder,
            drift_depth,
            case_observations,
        },
    );

    let deterministic = Deterministic {
        graph_work_definition: "graph_relax_iters: entered nonempty single-evaluation visits, including unaffordable visits (T11.F06); historical deltas cross definitions".to_string(),
        profile: profile_block(params, &config, recipes),
        per_seed,
        totals,
        per_creature_tick,
        goal_indicators,
    };

    let timings = RunTimings {
        wall_clock_ms_per_seed: wall_clock,
        phase_wall_clock_ms_per_seed: phase_wall_clock,
        throughput_per_seed,
        final_state_observation_ms_per_seed,
        neighborhood_founder_wall_clock_ms,
        drift_depth_wall_clock_ms,
        neighborhood_evolved_wall_clock_ms_per_seed,
    };

    Ok((deterministic, timings))
}

// ── Environment (non-deterministic, informational) ──────────────────────────

fn detect_hostname() -> String {
    std::process::Command::new("hostname")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

fn detect_cpu_model() -> Option<String> {
    if cfg!(target_os = "macos") {
        std::process::Command::new("sysctl")
            .args(["-n", "machdep.cpu.brand_string"])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    } else if cfg!(target_os = "linux") {
        std::fs::read_to_string("/proc/cpuinfo").ok().and_then(|c| {
            c.lines()
                .find(|l| l.starts_with("model name"))
                .and_then(|l| l.split(':').nth(1))
                .map(|s| s.trim().to_string())
        })
    } else {
        None
    }
}

fn detect_host() -> Host {
    Host {
        hostname: detect_hostname(),
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        cpu_model: detect_cpu_model(),
        logical_cores: std::thread::available_parallelism()
            .map(std::num::NonZeroUsize::get)
            .unwrap_or(1),
    }
}

fn detect_git_revision() -> String {
    std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Days since the Unix epoch to a proleptic Gregorian civil (year, month,
/// day) triple. Howard Hinnant's `civil_from_days` algorithm — no external
/// crate needed for the RFC 3339 timestamp.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32; // [1, 12]
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

/// Render the current wall-clock time as an RFC 3339 UTC timestamp without
/// pulling in a date/time dependency.
fn rfc3339_now() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = (secs / 86_400) as i64;
    let time_of_day = secs % 86_400;
    let (year, month, day) = civil_from_days(days);
    let hour = time_of_day / 3600;
    let minute = (time_of_day % 3600) / 60;
    let second = time_of_day % 60;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

fn build_environment(timings: RunTimings, totals: &Totals, threads: usize) -> Environment {
    let RunTimings {
        wall_clock_ms_per_seed,
        phase_wall_clock_ms_per_seed,
        throughput_per_seed,
        final_state_observation_ms_per_seed,
        neighborhood_founder_wall_clock_ms,
        drift_depth_wall_clock_ms,
        neighborhood_evolved_wall_clock_ms_per_seed,
    } = timings;
    let wall_clock_ms_total: f64 = wall_clock_ms_per_seed.iter().map(|s| s.wall_clock_ms).sum();
    let wall_clock_ms_per_creature_tick = if totals.creature_ticks == 0 {
        0.0
    } else {
        wall_clock_ms_total / totals.creature_ticks as f64
    };
    let throughput = Throughput {
        per_seed: throughput_per_seed,
        total: throughput_rates(
            totals.ticks,
            totals.creature_ticks,
            totals.births,
            wall_clock_ms_total,
        ),
    };
    let final_state_observation_ms_total = final_state_observation_ms_per_seed
        .iter()
        .map(|observation| observation.wall_clock_ms)
        .sum();
    let neighborhood_evolved_wall_clock_ms_total = neighborhood_evolved_wall_clock_ms_per_seed
        .iter()
        .map(|observation| observation.wall_clock_ms)
        .sum();
    Environment {
        rand_version: Some(locked_rand_version().to_string()),
        generated_at: rfc3339_now(),
        host: detect_host(),
        build_profile: if cfg!(debug_assertions) {
            "debug".to_string()
        } else {
            "release".to_string()
        },
        git_revision: detect_git_revision(),
        wall_clock_ms_per_seed,
        wall_clock_ms_total,
        wall_clock_ms_per_creature_tick,
        threads: Some(threads),
        phase_wall_clock_ms_per_seed,
        throughput,
        final_state_observation_ms_per_seed,
        final_state_observation_ms_total,
        neighborhood_founder_wall_clock_ms,
        drift_depth_wall_clock_ms,
        neighborhood_evolved_wall_clock_ms_per_seed,
        neighborhood_evolved_wall_clock_ms_total,
    }
}

// ── Report assembly ─────────────────────────────────────────────────────────

/// Build a full report on the rayon global thread pool.
pub fn build_report(params: &ProfileParams, feature: &str) -> Result<Report, String> {
    build_report_with_threads(params, feature, None)
}

/// Build a full report (deterministic block, environment, and an empty
/// comparison) for the given profile.
///
/// `threads` runs the profile inside a private rayon pool of that many
/// threads via `ThreadPool::install`, which scopes Phase 1b's `par_iter_mut`
/// to that pool; `None` uses the global pool. `install` rather than
/// `build_global` so one process can run a profile at several thread counts.
/// The recorded thread count is read from inside whichever pool ran the
/// profile.
pub fn build_report_with_threads(
    params: &ProfileParams,
    feature: &str,
    threads: Option<NonZeroUsize>,
) -> Result<Report, String> {
    let run = || (run_deterministic(params), rayon::current_num_threads());
    let (profile_run, threads_used) = match threads {
        Some(threads) => rayon::ThreadPoolBuilder::new()
            .num_threads(threads.get())
            .build()
            .expect("a rayon pool of at least one thread can always be built")
            .install(run),
        None => run(),
    };
    let (deterministic, timings) = profile_run?;
    let environment = build_environment(timings, &deterministic.totals, threads_used);
    Ok(Report {
        schema_version: SCHEMA_VERSION,
        feature: feature.to_string(),
        deterministic,
        environment,
        comparison: Comparison::default(),
    })
}

/// Serialize just the `deterministic` block, with keys sorted (via
/// `serde_json::Value`'s default `BTreeMap`-backed `Map`) and no timestamp,
/// hostname, duration, or path — suitable for the byte-identity test.
pub fn deterministic_block_json(report: &Report) -> String {
    let value = serde_json::to_value(&report.deterministic)
        .expect("Deterministic always serializes to a JSON value");
    serde_json::to_string(&value).expect("JSON value always serializes to a string")
}

/// Serialize the full report with sorted keys throughout.
pub fn report_json_pretty(report: &Report) -> String {
    let value = serde_json::to_value(report).expect("Report always serializes to a JSON value");
    serde_json::to_string_pretty(&value).expect("JSON value always serializes to a string")
}

// ── Comparison against stored references ────────────────────────────────────

fn percent_delta(current: f64, reference: f64) -> Option<f64> {
    if reference == 0.0 {
        return None;
    }
    Some((current - reference) / reference * 100.0)
}

fn counter_level(delta_percent: Option<f64>) -> ComparisonLevel {
    match delta_percent {
        None => ComparisonLevel::Ok,
        Some(d) if d > SEVERE_PERCENT => ComparisonLevel::Severe,
        Some(d) if d > FLAG_PERCENT => ComparisonLevel::Flag,
        _ => ComparisonLevel::Ok,
    }
}

/// Look up one counter's `per_creature_tick` value. Returns `None` when the
/// counter is absent from this report (an older schema), which the caller
/// must surface as `level: "new"` rather than treating as zero.
fn per_creature_tick_value(pct: &PerCreatureTick, name: &str) -> Option<f64> {
    let raw = match name {
        "mesh_hops" => &pct.mesh_hops,
        "vm_steps" => &pct.vm_steps,
        "graph_relax_iters" => &pct.graph_relax_iters,
        "plasticity_updates" => &pct.plasticity_updates,
        "actions_applied" => &pct.actions_applied,
        "births" => &pct.births,
        _ => unreachable!("unknown counter name: {name}"),
    };
    raw.as_deref().and_then(|s| s.parse::<f64>().ok())
}

/// Parse a six-decimal report string; `Undefined` reads as unmeasured.
fn parse_reading(value: &str) -> Option<f64> {
    value.parse::<f64>().ok()
}

/// A reader-state fraction is followed as one reading per state, named for the
/// state, so each is compared against its own history.
fn by_reader_state_readings(
    name: &str,
    fractions: &ByReaderState<String>,
) -> [(String, Option<f64>); 2] {
    [
        (
            format!("{name}_has_barrier_reader"),
            parse_reading(&fractions.has_barrier_reader),
        ),
        (
            format!("{name}_no_barrier_reader"),
            parse_reading(&fractions.no_barrier_reader),
        ),
    ]
}

/// Blocked moves are followed as one total per cause, named for the cause, so
/// a world that trades barrier blocks for crowding is readable as such.
fn by_cause_readings(blocked: Option<&MovesBlockedByCause>) -> [(String, Option<f64>); 3] {
    [
        ("moves_blocked_barrier_total", blocked.map(|by| by.barrier)),
        (
            "moves_blocked_occupied_total",
            blocked.map(|by| by.occupied),
        ),
        (
            "moves_blocked_out_of_bounds_total",
            blocked.map(|by| by.out_of_bounds),
        ),
    ]
    .map(|(name, count)| (name.to_string(), count.map(|count| count as f64)))
}

/// Every reading a world-set comparison follows for one case, in report order.
/// An absent case, an unmeasured indicator, and a zero denominator all read as
/// `None` rather than as a zero the delta would then compare against.
fn case_readings(report: &Report, case_name: &str) -> Vec<(String, Option<f64>)> {
    let indicators = &report.deterministic.goal_indicators;
    let Some(observation) = indicators
        .cases
        .iter()
        .find(|entry| entry.case.name == case_name)
    else {
        return Vec::new();
    };
    let seed = observation.case.seed;
    let persistence = indicators
        .population_persistence
        .per_seed
        .iter()
        .find(|row| row.seed == seed);
    let run = report
        .deterministic
        .per_seed
        .iter()
        .find(|row| row.seed == seed);
    let lineage = match &indicators.lineage_diversity {
        Indicator::Defined(reading) => reading.per_seed.iter().find(|row| row.seed == seed),
        Indicator::Undefined(_) => None,
    };
    let memory = match &indicators.memory_sensitivity {
        Indicator::Defined(reading) => reading.per_seed.iter().find(|row| row.seed == seed),
        Indicator::Undefined(_) => None,
    };
    let neighborhood = match &observation.mutational_neighborhood {
        Indicator::Defined(reading) => Some(reading),
        Indicator::Undefined(_) => None,
    };
    let evolved = neighborhood.and_then(|reading| match &reading.evolved {
        Indicator::Defined(half) => half.per_seed.first(),
        Indicator::Undefined(_) => None,
    });
    let per_creature_tick = |count: fn(&PerSeed) -> u64| {
        run.and_then(|row| {
            (row.creature_ticks > 0).then(|| count(row) as f64 / row.creature_ticks as f64)
        })
    };

    let mut readings: Vec<(String, Option<f64>)> = vec![
        (
            "final_population".into(),
            persistence.map(|row| row.final_population as f64),
        ),
        (
            "minimum_population".into(),
            persistence.map(|row| row.minimum_population as f64),
        ),
        (
            "peak_population".into(),
            persistence.map(|row| row.peak_population as f64),
        ),
        (
            "plateau_population".into(),
            persistence
                .and_then(|row| row.plateau_population.as_deref())
                .and_then(parse_reading),
        ),
        ("births".into(), run.map(|row| row.births as f64)),
        (
            "mean_energy".into(),
            persistence
                .and_then(|row| row.mean_energy.as_deref())
                .and_then(parse_reading),
        ),
        (
            "extinction_tick".into(),
            persistence
                .and_then(|row| row.extinction_tick)
                .map(|tick| tick as f64),
        ),
    ];
    // The same six counters the profile totals normalize, in the same order, so
    // a per-case row can never drift from the profile-level counter list.
    let counts: [fn(&PerSeed) -> u64; COUNTER_NAMES.len()] = [
        |row| row.mesh_hops,
        |row| row.vm_steps,
        |row| row.graph_relax_iters,
        |row| row.plasticity_updates,
        |row| row.actions_applied,
        |row| row.births,
    ];
    for (name, count) in COUNTER_NAMES.iter().zip(counts) {
        readings.push((
            format!("{name}_per_creature_tick"),
            per_creature_tick(count),
        ));
    }
    for (index, share) in observation.fractions.typed_eat_share.iter().enumerate() {
        readings.push((
            format!("typed_eat_share_type_{index}"),
            parse_reading(share),
        ));
    }
    readings.extend(by_reader_state_readings(
        "barrier_blocked_fraction",
        &observation
            .fractions
            .barrier_blocked_fraction_by_reader_state,
    ));
    readings.extend(by_reader_state_readings(
        "avoidable_blocked_share_of_all_moves",
        &observation
            .fractions
            .avoidable_blocked_share_of_all_moves_by_reader_state,
    ));
    readings.extend(by_cause_readings(
        observation.tracking.moves_blocked_total_by_cause.as_ref(),
    ));
    readings.extend([
        (
            "blocked_move_fraction".to_string(),
            parse_reading(&observation.fractions.blocked_move_fraction),
        ),
        (
            "lineage_shannon_entropy_nats".to_string(),
            lineage.and_then(|row| parse_reading(&row.shannon_entropy_nats)),
        ),
        (
            "surviving_founder_clade_count".to_string(),
            lineage.map(|row| row.surviving_founder_clade_count as f64),
        ),
        (
            "memory_different_from_either_fraction".to_string(),
            memory.and_then(|row| parse_reading(&row.different_from_either_fraction)),
        ),
        (
            "drift_changed_per_all_births_at_2000".to_string(),
            match &observation.drift_depth {
                Indicator::Defined(drift) => drift
                    .readings
                    .iter()
                    .find(|row| row.depth == 2_000)
                    .and_then(|row| parse_reading(&row.changed_per_all_births)),
                Indicator::Undefined(_) => None,
            },
        ),
        (
            "founder_changed_per_all_births".to_string(),
            neighborhood.and_then(|reading| {
                parse_reading(&reading.founder.births.any_events.changed_fraction)
            }),
        ),
        (
            "founder_dead_per_all_births".to_string(),
            neighborhood.and_then(|reading| {
                parse_reading(&reading.founder.births.any_events.dead_fraction)
            }),
        ),
        (
            "evolved_changed_per_all_births".to_string(),
            evolved.and_then(|row| parse_reading(&row.pooled_births.any_events.changed_fraction)),
        ),
        (
            "evolved_dead_per_all_births".to_string(),
            evolved.and_then(|row| parse_reading(&row.pooled_births.any_events.dead_fraction)),
        ),
        (
            "reachable_structure_size_median".to_string(),
            observation
                .reachable_structure_size_distribution
                .as_ref()
                .map(|distribution| f64::from(distribution.median)),
        ),
    ]);
    readings
}

/// Compare every world in `current` against the same-named world in
/// `reference`. Empty outside the world-set profile.
fn compare_cases(current: &Report, reference: &Report) -> Vec<CaseComparison> {
    if current.deterministic.profile.name != GOAL_WORLD_SET {
        return Vec::new();
    }
    current
        .deterministic
        .profile
        .cases
        .iter()
        .map(|case| {
            let reference_case = reference
                .deterministic
                .profile
                .cases
                .iter()
                .find(|other| other.name == case.name);
            let reference_readings = case_readings(reference, &case.name);
            let readings = case_readings(current, &case.name)
                .into_iter()
                .map(|(name, current_value)| {
                    let reference_value = reference_readings
                        .iter()
                        .find(|(other, _)| *other == name)
                        .and_then(|(_, value)| *value);
                    let percent_delta = (!VALUE_ONLY_CASE_READINGS.contains(&name.as_str()))
                        .then(|| current_value.zip(reference_value))
                        .flatten()
                        .and_then(|(current, reference)| percent_delta(current, reference));
                    CaseReadingComparison {
                        name,
                        current: current_value.map(six),
                        reference: reference_value.map(six),
                        percent_delta: percent_delta.map(six),
                    }
                })
                .collect();
            CaseComparison {
                case: case.name.clone(),
                inputs_changed: reference_case.is_some_and(|other| {
                    other.config_digest != case.config_digest || other.seed != case.seed
                }),
                absent_in_reference: reference_case.is_none(),
                current_digest: case.config_digest.clone(),
                reference_digest: reference_case.map(|other| other.config_digest.clone()),
                readings,
            }
        })
        .collect()
}

/// Compare `current` against one stored reference report, returning a
/// `ReferenceComparison`. Only integer work counters (never wall-clock) can
/// mark a comparison `severe`.
pub fn compare_against(
    current: &Report,
    reference_path: &Path,
    reference: &Report,
) -> ReferenceComparison {
    let mut counters = Vec::with_capacity(COUNTER_NAMES.len());
    let mut any_severe = false;

    for &name in &COUNTER_NAMES {
        let current_value = per_creature_tick_value(&current.deterministic.per_creature_tick, name)
            .expect("a freshly built report always populates every per_creature_tick counter");
        let reference_value =
            per_creature_tick_value(&reference.deterministic.per_creature_tick, name);

        let (level, reference_str, delta_str) = match reference_value {
            None => (ComparisonLevel::New, None, None),
            // The reference recorded no work for this counter but the
            // current run does: the ratio is unbounded (division by zero),
            // so treat it as severe rather than silently reporting "ok"
            // with a null delta. This is the only way a feature that
            // introduces the first nonzero reading of a counter (e.g. the
            // first plastic founder genome) can trip a regression at all.
            Some(reference_value) if reference_value == 0.0 && current_value > 0.0 => {
                any_severe = true;
                (ComparisonLevel::Severe, Some(six(reference_value)), None)
            }
            Some(reference_value) => {
                let delta = percent_delta(current_value, reference_value);
                let level = counter_level(delta);
                if level == ComparisonLevel::Severe {
                    any_severe = true;
                }
                (level, Some(six(reference_value)), delta.map(six))
            }
        };

        counters.push(CounterComparison {
            name: name.to_string(),
            current: six(current_value),
            reference: reference_str,
            percent_delta: delta_str,
            level,
        });
    }

    let wall_clock = if current.environment.host == reference.environment.host {
        let current_ms = current.environment.wall_clock_ms_per_creature_tick;
        let reference_ms = reference.environment.wall_clock_ms_per_creature_tick;
        let delta = percent_delta(current_ms, reference_ms).unwrap_or(0.0);
        let level = if delta > WALL_CLOCK_SEVERE_PERCENT {
            ComparisonLevel::Severe
        } else if delta > WALL_CLOCK_FLAG_PERCENT {
            ComparisonLevel::Flag
        } else {
            ComparisonLevel::Ok
        };
        Some(WallClockComparison {
            current_ms_per_creature_tick: current_ms,
            reference_ms_per_creature_tick: reference_ms,
            percent_delta: delta,
            level,
        })
    } else {
        None
    };

    ReferenceComparison {
        path: reference_path.display().to_string(),
        counters,
        cases: compare_cases(current, reference),
        wall_clock,
        severe: any_severe,
    }
}

/// The profile block two reports must agree on to be comparable at all.
///
/// For the world set the per-case block is excluded: every closure that edits a
/// recipe changes that case's `config_digest`, and the standard-baseline
/// contract expects exactly that. Hard-failing on it would discard a ten-minute
/// run, so a digest change is reported per case as `inputs_changed` and a case
/// the reference never ran is reported as `absent_in_reference`. World size,
/// founder count, seeds, ticks, and food coverage still have to match.
fn comparable_profile(profile: &ProfileBlock) -> ProfileBlock {
    let mut profile = profile.clone();
    if profile.name == GOAL_WORLD_SET {
        profile.cases.clear();
    }
    profile
}

/// Load a reference report from disk and compare `current` against it.
/// Returns `Err` when the reference file cannot be read or parsed — a
/// missing declared reference is a hard error, not a silent skip.
pub fn compare_against_path(
    current: &Report,
    reference_path: &Path,
) -> Result<ReferenceComparison, String> {
    let content = std::fs::read_to_string(reference_path)
        .map_err(|e| format!("failed to read reference {}: {e}", reference_path.display()))?;
    let reference: Report = serde_json::from_str(&content).map_err(|e| {
        format!(
            "failed to parse reference {}: {e}",
            reference_path.display()
        )
    })?;
    if comparable_profile(&reference.deterministic.profile)
        != comparable_profile(&current.deterministic.profile)
    {
        return Err(format!(
            "reference {} was generated with a different profile ({:?}) than the \
             current run ({:?}); a work-counter comparison across different world \
             size, founder count, seeds, ticks, or food coverage is meaningless. \
             Re-pin the reference or exclude it.",
            reference_path.display(),
            reference.deterministic.profile,
            current.deterministic.profile
        ));
    }
    Ok(compare_against(current, reference_path, &reference))
}

/// Compare `current` against every reference path, folding the results into
/// `current.comparison`, and return whether any reference was severe.
pub fn apply_comparisons(
    current: &mut Report,
    reference_paths: &[PathBuf],
) -> Result<bool, String> {
    let mut references = Vec::with_capacity(reference_paths.len());
    let mut overall_severe = false;
    for path in reference_paths {
        let entry = compare_against_path(current, path)?;
        if entry.severe {
            overall_severe = true;
        }
        references.push(entry);
    }
    current.comparison = Comparison {
        references,
        severe: overall_severe,
    };
    Ok(overall_severe)
}

// ── Series index (docs/progress/benchmark-series.json) ──────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesIndex {
    pub series: String,
    pub epoch_baseline: String,
    pub closed: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkSeriesIndex {
    pub gate: SeriesIndex,
    pub goal: SeriesIndex,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub goal_worlds: Option<SeriesIndex>,
}

/// Resolve the gate profile's default comparison references from the series
/// index: the epoch baseline and the last closed report (if different).
/// Returns an empty list when the series index does not exist yet (this
/// feature's own first report, which becomes the epoch baseline).
pub fn default_gate_references(series_index_path: &Path) -> Result<Vec<PathBuf>, String> {
    if !series_index_path.exists() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(series_index_path)
        .map_err(|e| format!("failed to read {}: {e}", series_index_path.display()))?;
    let index: BenchmarkSeriesIndex = serde_json::from_str(&content)
        .map_err(|e| format!("failed to parse {}: {e}", series_index_path.display()))?;
    references_from_series(&index.gate)
}

/// Resolve the goal profile's comparison references from its distinct series.
/// Its initial baseline does not exist until that first goal report is written,
/// so the first invocation deliberately has no comparison.
pub fn default_goal_references(series_index_path: &Path) -> Result<Vec<PathBuf>, String> {
    if !series_index_path.exists() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(series_index_path)
        .map_err(|e| format!("failed to read {}: {e}", series_index_path.display()))?;
    let index: BenchmarkSeriesIndex = serde_json::from_str(&content)
        .map_err(|e| format!("failed to parse {}: {e}", series_index_path.display()))?;
    let Some(series) = index.goal_worlds.as_ref() else {
        return Ok(Vec::new());
    };
    let baseline = PathBuf::from(&series.epoch_baseline);
    if !baseline.exists() {
        return Ok(Vec::new());
    }
    references_from_series(series)
}

fn references_from_series(index: &SeriesIndex) -> Result<Vec<PathBuf>, String> {
    let mut paths = vec![PathBuf::from(&index.epoch_baseline)];
    if let Some(last) = index.closed.last() {
        if last != &index.epoch_baseline {
            paths.push(PathBuf::from(last));
        }
    }
    Ok(paths)
}

/// Read the simulation crate's dependency identity, not lockfile package order.
fn locked_rand_version() -> &'static str {
    rand_version_from_lock(include_str!("../../../Cargo.lock"))
        .expect("workspace lockfile identifies v3-core's rand version")
}

fn rand_version_from_lock(lockfile: &str) -> Option<&str> {
    let packages = lockfile.split("[[package]]");
    let core = packages
        .clone()
        .find(|package| package.lines().any(|line| line == "name = \"v3-core\""))?;
    if let Some(version) = core.lines().find_map(|line| {
        line.trim()
            .strip_prefix("\"rand ")
            .and_then(|dependency| dependency.strip_suffix("\","))
    }) {
        return Some(version);
    }
    // Cargo omits the version in dependency identities when the name is unique.
    if !core.lines().any(|line| line.trim() == "\"rand\",") {
        return None;
    }
    let mut versions = packages
        .filter(|package| package.lines().any(|line| line == "name = \"rand\""))
        .filter_map(|package| {
            package.lines().find_map(|line| {
                line.strip_prefix("version = \"")
                    .and_then(|version| version.strip_suffix('"'))
            })
        });
    let version = versions.next()?;
    versions.next().is_none().then_some(version)
}

// ── Persistence accumulator unit tests ──────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use v3_core::contracts::{Direction, WorldAction};
    use v3_core::simulation::seed_simulation;

    proptest! {
        #[test]
        fn births_rate_uses_complete_profile_totals(births in 0u64..1_000_000, ticks in 0u64..100_000) {
            for ticks in [0, ticks.max(1)] {
                let inputs = GoalIndicatorInputs {
                    population_persistence_per_seed: vec![],
                    lineage_diversity_per_seed: vec![],
                    memory_sensitivity_per_seed: vec![],
                    temporal_memory_sensitivity_per_seed: vec![],
                    evolved_neighborhood_per_seed: vec![],
                    pooled_complexities: vec![],
                    neighborhood_founder: None,
                    drift_depth: Indicator::Undefined("test profile".into()),
                    case_observations: vec![],
                };
                let totals = Totals { births, ticks, ..Totals::default() };
                let indicators = assemble_goal_indicators(&small_profile("synthetic"), &totals, inputs);
                let expected = if ticks == 0 { 0.0 } else { births as f64 * 100.0 / ticks as f64 };
                let actual: f64 = indicators.births_per_100_ticks.parse().unwrap();
                prop_assert!((actual - expected).abs() <= 0.000_001);
            }
        }
    }

    #[test]
    fn goal_case_adds_observation_time_to_zero_accumulators() {
        let mut params = small_profile(GOAL_WORLD_SET);
        params.seeds = goal_recipe_seeds();
        let mut founder_ms = 0.0;
        let mut drift_ms = Some(0.0);
        let _case = prepare_goal_case(&params, &GOAL_RECIPES[0], &mut founder_ms, &mut drift_ms);
        assert!(founder_ms > 0.0);
        assert!(drift_ms.unwrap() > 0.0);
    }

    #[test]
    fn goal_cases_keep_distinct_full_population_structure_distributions() {
        let mut params = goal_profile_params();
        params.width = 32;
        params.height = 32;
        params.founders = 32;
        params.ticks = 60;
        params.neighborhood = NeighborhoodSizes::default();
        params.drift = Default::default();
        let (report, _) = run_deterministic(&params).expect("a valid profile");
        let mut expected_cases = Vec::new();
        let mut pooled = Vec::new();
        for (index, case) in report.goal_indicators.cases.iter().enumerate() {
            let (_, config) = goal_case(&params, &GOAL_RECIPES[index]);
            let run = run_one_seed(
                &config,
                params.seeds[index],
                params.ticks,
                false,
                None,
                params.neighborhood,
            );
            assert_eq!(run.complexities.len() as u64, run.per_seed.final_population);
            assert!(run.complexities.len() > neighborhood::SAMPLE_SIZE);
            pooled.extend_from_slice(&run.complexities);
            let expected =
                serde_json::to_value(structure_size_distribution(run.complexities)).unwrap();
            assert_eq!(
                serde_json::to_value(case.reachable_structure_size_distribution.as_ref().unwrap())
                    .unwrap(),
                expected
            );
            expected_cases.push(expected);
            let mut historical = serde_json::to_value(case).unwrap();
            historical
                .as_object_mut()
                .unwrap()
                .remove("reachable_structure_size_distribution");
            let historical: GoalCaseObservation = serde_json::from_value(historical).unwrap();
            assert!(historical.reachable_structure_size_distribution.is_none());
        }
        assert!(expected_cases.windows(2).any(|pair| pair[0] != pair[1]));
        assert_eq!(
            serde_json::to_value(report.goal_indicators.reachable_structure_size_distribution)
                .unwrap(),
            serde_json::to_value(structure_size_distribution(pooled)).unwrap()
        );
    }

    #[test]
    fn goal_world_set_executes_three_named_configs_with_case_observations() {
        let mut params = goal_profile_params();
        params.width = 16;
        params.height = 16;
        params.founders = 4;
        params.ticks = 1;
        params.neighborhood = NeighborhoodSizes::default();
        params.drift = Default::default();
        let (report, timings) = run_deterministic(&params).expect("a valid profile");
        assert_eq!(report.profile.name, "goal-worlds-v1");
        assert_eq!(report.per_seed.len(), 3);
        assert_eq!(
            report
                .profile
                .cases
                .iter()
                .map(|case| case.seed)
                .collect::<Vec<_>>(),
            vec![11, 22, 33]
        );
        assert_eq!(
            report
                .profile
                .cases
                .iter()
                .map(|case| case.food_type_count)
                .collect::<Vec<_>>(),
            vec![2, 1, 2]
        );
        assert_eq!(report.goal_indicators.cases.len(), 3);
        assert!(
            matches!(report.goal_indicators.mutational_neighborhood, Indicator::Undefined(ref reason) if reason == "reported per case")
        );
        assert!(
            matches!(report.goal_indicators.drift_depth, Indicator::Undefined(ref reason) if reason == "reported per case")
        );
        for (index, case) in report.goal_indicators.cases.iter().enumerate() {
            let (expected, config) = goal_case(&params, &GOAL_RECIPES[index]);
            assert_eq!(case.case, expected);
            let seeded = seed_simulation(config.clone(), expected.seed);
            assert_eq!(
                report.per_seed[index].tick_zero_connectivity,
                Some(seeded.world.passable_connectivity())
            );
            let Indicator::Defined(neighborhood) = &case.mutational_neighborhood else {
                panic!("case neighborhood missing")
            };
            let battery = Battery::generate(config.world.food.types.len());
            let expected_founder =
                compute_founder_neighborhood(&config, &battery, params.neighborhood);
            assert_eq!(
                serde_json::to_value(&neighborhood.founder).unwrap(),
                serde_json::to_value(expected_founder).unwrap()
            );
            let Indicator::Defined(evolved) = &neighborhood.evolved else {
                panic!("case evolved reading missing")
            };
            assert_eq!(evolved.per_seed.len(), 1);
            assert_eq!(evolved.per_seed[0].seed, expected.seed);
            assert!(matches!(case.drift_depth, Indicator::Defined(_)));
        }
        assert!(timings.neighborhood_founder_wall_clock_ms > 0.0);
        assert!(timings.drift_depth_wall_clock_ms.unwrap() > 0.0);
        assert_eq!(timings.neighborhood_evolved_wall_clock_ms_per_seed.len(), 3);
    }

    #[test]
    fn lockfile_identity_selects_core_dependency_among_reordered_versions() {
        let core = "[[package]]\nname = \"v3-core\"\nversion = \"0.1.0\"\ndependencies = [\n \"rand 0.8.6\",\n]\n";
        let old = "[[package]]\nname = \"rand\"\nversion = \"0.8.6\"\n";
        let new = "[[package]]\nname = \"rand\"\nversion = \"0.9.5\"\n";
        for lock in [format!("{new}{old}{core}"), format!("{core}{old}{new}")] {
            assert_eq!(rand_version_from_lock(&lock), Some("0.8.6"));
        }
        let upgraded = format!("{old}{new}{}", core.replace("rand 0.8.6", "rand 0.9.5"));
        assert_eq!(rand_version_from_lock(&upgraded), Some("0.9.5"));
        let no_rand = core.replace("rand 0.8.6", "serde");
        assert_eq!(rand_version_from_lock(&format!("{old}{no_rand}")), None);
        let unversioned = core.replace("rand 0.8.6", "rand");
        assert_eq!(
            rand_version_from_lock(&format!("{old}{unversioned}")),
            Some("0.8.6")
        );
        assert_eq!(
            rand_version_from_lock(&format!("{old}{new}{unversioned}")),
            None
        );
    }

    #[test]
    fn drift_checkpoint_uses_pooled_lineage_execution_and_all_birth_denominators() {
        let row = neighborhood::drift::Checkpoint {
            depth: 250,
            mesh: neighborhood::drift::MeshTotals {
                backends: neighborhood::mesh_execution::MeshBackendCounts {
                    graph: neighborhood::mesh_execution::BackendNodeCounts {
                        total: 8,
                        executed: 3,
                        contributing: 1,
                    },
                    vm: neighborhood::mesh_execution::BackendNodeCounts {
                        total: 12,
                        executed: 4,
                        contributing: 3,
                    },
                },
                lineages: 4,
                total_nodes: 20,
                reachable_nodes: 10,
                executed_nodes: 7,
                knockout_nodes: 3,
                route_varying_lineages: 1,
                hop_cap_hits: 8,
            },
            births: BirthResult {
                births_total: 20,
                zero_event_births: 10,
                any_events: Tally {
                    trials: 10,
                    silent: 5,
                    changed: 3,
                    dead: 2,
                    ..Tally::default()
                },
                ..BirthResult::default()
            },
        };
        let report = drift_checkpoint(row);
        let backends = report.backends.expect("measured backend totals");
        assert_eq!(
            (
                backends.graph.total,
                backends.graph.executed,
                backends.graph.contributing
            ),
            (8, 3, 1)
        );
        assert_eq!(
            (
                backends.vm.total,
                backends.vm.executed,
                backends.vm.contributing
            ),
            (12, 4, 3)
        );
        let mut historical = serde_json::to_value(&report).unwrap();
        historical.as_object_mut().unwrap().remove("backends");
        let historical: DriftDepthCheckpoint = serde_json::from_value(historical).unwrap();
        assert_eq!(historical.backends, None);
        assert_eq!(historical.total_nodes, 20);
        assert_eq!(report.depth, 250);
        assert_eq!(report.mean_total_nodes, "5.000000");
        assert_eq!(report.mean_reachable_nodes, "2.500000");
        assert_eq!(report.mean_executed_nodes, "1.750000");
        assert_eq!(report.mean_knockout_nodes, "0.750000");
        assert_eq!(report.route_varying_fraction, "0.250000");
        assert_eq!(report.battery_executions, 320);
        assert_eq!(report.hop_cap_fraction, "0.025000");
        assert_eq!(report.silent_per_all_births, "0.250000");
        assert_eq!(report.changed_per_all_births, "0.150000");
        assert_eq!(report.dead_per_all_births, "0.100000");
        assert_eq!(report.births.any_events.changed_fraction, "0.300000");
        let empty = drift_checkpoint(neighborhood::drift::Checkpoint::default());
        assert_eq!(empty.mean_total_nodes, UNDEFINED);
        assert_eq!(empty.hop_cap_fraction, UNDEFINED);
        assert_eq!(empty.changed_per_all_births, UNDEFINED);
    }

    #[test]
    fn reduced_drift_deterministic_output_matches_across_thread_counts_and_seed_counts() {
        let mut params = small_profile("goal");
        let one = rayon::ThreadPoolBuilder::new()
            .num_threads(1)
            .build()
            .unwrap()
            .install(|| run_deterministic(&params).expect("a valid profile").0);
        let two = rayon::ThreadPoolBuilder::new()
            .num_threads(2)
            .build()
            .unwrap()
            .install(|| run_deterministic(&params).expect("a valid profile").0);
        assert_eq!(
            serde_json::to_vec(&one).unwrap(),
            serde_json::to_vec(&two).unwrap()
        );
        params.seeds.push(99);
        let more = run_deterministic(&params).expect("a valid profile").0;
        assert_eq!(
            one.goal_indicators.drift_depth,
            more.goal_indicators.drift_depth
        );
    }

    #[test]
    fn drift_is_goal_only_once_and_historical_fields_are_unavailable() {
        let goal = build_report(&small_profile("goal"), "test").expect("a valid profile");
        let Indicator::Defined(drift) = &goal.deterministic.goal_indicators.drift_depth else {
            panic!("goal drift missing")
        };
        assert_eq!(drift.version, "drift-depth-v2");
        assert_eq!(drift.founder, "V3Alpha1");
        assert_eq!(
            drift.birth_subset,
            "first lineage indices in ascending order"
        );
        assert_eq!(drift.walk_seed_formula, "90000 + lineage_index");
        assert_eq!(
            drift.birth_seed_formula,
            "7000000 + 1000 * (lineage_index + 1) + checkpoint + 9000 + trial_index"
        );
        assert_eq!(drift.battery_version, "neighborhood-v1");
        assert_eq!(drift.mesh_version, "mesh-execution-v1");
        assert_eq!(drift.knockout_method, "static-successor-bypass-v1");
        assert_eq!(
            drift.executed_source,
            "battery hop records (mesh-execution-v1), node ids"
        );
        assert_eq!(
            drift.executed_refresh,
            "walk: depth 0 and every 10 generations; births: derived at each checkpoint"
        );
        assert_eq!(
            (
                drift.executions_per_genome,
                drift.snapshot_count,
                drift.sequence_count,
                drift.sequence_len
            ),
            (80, 48, 8, 4)
        );
        let production = goal_profile_params().drift;
        assert_eq!(
            (
                production.lineages,
                production.birth_lineages,
                production.births
            ),
            (50, 20, 100)
        );
        assert_eq!(production.checkpoints, &[0, 22, 250, 1000, 2000]);
        assert_eq!(drift.lineages, 2);
        assert_eq!(drift.birth_lineages, 1);
        assert_eq!(drift.birth_trials, 2);
        assert_eq!(drift.checkpoints, vec![0, 2]);
        assert_eq!(drift.readings.len(), 2);
        assert_eq!(drift.readings[0].births.births_total, 2);
        assert_eq!(drift.readings[0].battery_executions, 160);
        assert!(goal.environment.drift_depth_wall_clock_ms.is_some());
        for name in ["gate", "sweep", "synthetic"] {
            let report = build_report(&small_profile(name), "test").expect("a valid profile");
            assert!(matches!(
                report.deterministic.goal_indicators.drift_depth,
                Indicator::Undefined(_)
            ));
            assert_eq!(report.environment.drift_depth_wall_clock_ms, None);
        }
        let mut historical = serde_json::to_value(&goal).unwrap();
        historical["deterministic"]["goal_indicators"]
            .as_object_mut()
            .unwrap()
            .remove("drift_depth");
        historical["environment"]
            .as_object_mut()
            .unwrap()
            .remove("drift_depth_wall_clock_ms");
        let historical: Report = serde_json::from_value(historical).unwrap();
        assert!(matches!(
            historical.deterministic.goal_indicators.drift_depth,
            Indicator::Undefined(_)
        ));
        assert_eq!(historical.environment.drift_depth_wall_clock_ms, None);
    }

    /// Feed the accumulator one observation per tick from a population
    /// series (index 0 is tick 1), with a constant mean creature energy and
    /// a cumulative birth count equal to the tick.
    fn observe_series(
        horizon: u64,
        seeded_population: u64,
        populations: &[u64],
        energy: f64,
    ) -> PopulationPersistenceSeed {
        let mut accumulator = PersistenceAccumulator::new(horizon, seeded_population);
        for (index, &population) in populations.iter().enumerate() {
            let tick = index as u64 + 1;
            accumulator.observe(tick, population, tick, || energy, WorldTracking::default);
        }
        accumulator.finish(7)
    }

    /// `millis` converts to fractional milliseconds. A pure unit conversion,
    /// not a wall-clock magnitude: the input duration is synthetic.
    #[test]
    fn millis_converts_a_duration_to_fractional_milliseconds() {
        use std::time::Duration;

        assert_eq!(millis(Duration::from_millis(1500)), 1500.0);
        assert_eq!(millis(Duration::from_micros(1)), 0.001);
        assert_eq!(millis(Duration::ZERO), 0.0);
    }

    fn synthetic_timings(seed_wall_clock_ms: &[f64]) -> RunTimings {
        RunTimings {
            wall_clock_ms_per_seed: seed_wall_clock_ms
                .iter()
                .enumerate()
                .map(|(index, &wall_clock_ms)| SeedWallClock {
                    seed: index as u64,
                    wall_clock_ms,
                })
                .collect(),
            phase_wall_clock_ms_per_seed: Vec::new(),
            throughput_per_seed: Vec::new(),
            final_state_observation_ms_per_seed: Vec::new(),
            neighborhood_founder_wall_clock_ms: 0.0,
            drift_depth_wall_clock_ms: None,
            neighborhood_evolved_wall_clock_ms_per_seed: Vec::new(),
        }
    }

    /// `build_environment` sums the per-seed wall-clock, divides it by the
    /// run's creature-ticks, and records the thread count it was given. The
    /// inputs are synthetic, so this asserts the arithmetic, never a timing.
    #[test]
    fn build_environment_derives_the_per_creature_tick_cost_and_throughput() {
        let totals = Totals {
            ticks: 200,
            creature_ticks: 4_000,
            births: 50,
            ..Totals::default()
        };

        let environment = build_environment(synthetic_timings(&[300.0, 500.0]), &totals, 4);

        assert_eq!(environment.wall_clock_ms_total, 800.0);
        assert_eq!(environment.wall_clock_ms_per_creature_tick, 0.2);
        assert_eq!(environment.threads, Some(4));
        assert_eq!(environment.rand_version.as_deref(), Some("0.8.6"));
        let mut historical = serde_json::to_value(&environment).unwrap();
        historical.as_object_mut().unwrap().remove("rand_version");
        assert_eq!(
            serde_json::from_value::<Environment>(historical)
                .unwrap()
                .rand_version,
            None
        );
        assert_eq!(
            environment.throughput.total,
            throughput_rates(200, 4_000, 50, 800.0)
        );
        assert_eq!(environment.throughput.total.ticks_per_second, 250.0);
    }

    /// A run with no creature-ticks reports a zero per-creature-tick cost
    /// rather than dividing by zero.
    #[test]
    fn build_environment_reports_zero_cost_when_no_creature_ran() {
        let environment = build_environment(synthetic_timings(&[12.5]), &Totals::default(), 1);

        assert_eq!(environment.wall_clock_ms_total, 12.5);
        assert_eq!(environment.wall_clock_ms_per_creature_tick, 0.0);
    }

    /// Every level's `Display` string is the string it serializes to, which
    /// is what `run_bench` prints and what the stored reports carry.
    #[test]
    fn comparison_level_displays_as_its_serialized_string() {
        for level in [
            ComparisonLevel::Ok,
            ComparisonLevel::Flag,
            ComparisonLevel::Severe,
            ComparisonLevel::New,
        ] {
            let serialized = serde_json::to_value(level).expect("a level always serializes");
            assert_eq!(
                level.to_string(),
                serialized.as_str().expect("levels serialize as strings")
            );
        }
        assert_eq!(ComparisonLevel::Severe.to_string(), "severe");
    }

    /// The work-counter thresholds are strict: a delta exactly at 10 percent
    /// is `ok` and one exactly at 50 percent is `flag`, not the next level up.
    #[test]
    fn counter_levels_are_strict_at_their_thresholds() {
        assert_eq!(counter_level(None), ComparisonLevel::Ok);
        assert_eq!(counter_level(Some(0.0)), ComparisonLevel::Ok);
        assert_eq!(counter_level(Some(-80.0)), ComparisonLevel::Ok);
        assert_eq!(
            counter_level(percent_delta(11.0, 10.0)),
            ComparisonLevel::Ok,
            "exactly +10 percent is inside the flag threshold"
        );
        assert_eq!(counter_level(Some(10.5)), ComparisonLevel::Flag);
        assert_eq!(
            counter_level(percent_delta(1.5, 1.0)),
            ComparisonLevel::Flag,
            "exactly +50 percent is inside the severe threshold"
        );
        assert_eq!(counter_level(Some(60.0)), ComparisonLevel::Severe);
    }

    #[test]
    fn peak_records_the_maximum_population_and_the_first_tick_reaching_it() {
        let summary = observe_series(8, 10, &[12, 20, 15, 20, 18, 9, 9, 9], 1.0);

        assert_eq!(summary.peak_population, 20);
        assert_eq!(summary.peak_tick, 2, "the first tick at the peak wins");
        assert_eq!(summary.minimum_population, 9);
    }

    #[test]
    fn seeding_population_is_the_peak_when_no_tick_exceeds_it() {
        let summary = observe_series(4, 50, &[40, 30, 20, 10], 1.0);

        assert_eq!(summary.peak_population, 50);
        assert_eq!(summary.peak_tick, 0, "tick 0 is the founder population");
    }

    #[test]
    fn plateau_is_null_when_the_run_goes_extinct_before_the_window() {
        // Horizon 100: the plateau window is ticks 76..=100. Extinction at
        // tick 3 means no observation ever lands in the window.
        let summary = observe_series(100, 4, &[3, 1, 0], 1.0);

        assert_eq!(summary.extinction_tick, Some(3));
        assert_eq!(summary.plateau_population, None);
        assert_eq!(summary.final_population, 0);
    }

    #[test]
    fn plateau_averages_only_the_ticks_executed_inside_a_partial_window() {
        // Horizon 8: the window is the ticks strictly after 6, so 7 and 8.
        // The run goes extinct at tick 8, so the window holds 10 and 0.
        let summary = observe_series(8, 4, &[4, 4, 4, 4, 4, 4, 10, 0], 1.0);

        assert_eq!(summary.extinction_tick, Some(8));
        assert_eq!(summary.plateau_population.as_deref(), Some("5.000000"));
    }

    #[test]
    fn mean_energy_is_null_at_extinction_and_the_final_value_otherwise() {
        let extinct = observe_series(4, 4, &[4, 2, 0], 2.5);
        assert_eq!(extinct.mean_energy, None);
        assert_eq!(
            extinct
                .samples
                .last()
                .expect("the extinction tick is sampled")
                .mean_energy,
            None
        );

        let survived = observe_series(4, 4, &[4, 4, 4, 4], 2.5);
        assert_eq!(survived.mean_energy.as_deref(), Some("2.500000"));
    }

    #[test]
    fn samples_cover_every_hundredth_tick_plus_a_deduplicated_final_tick() {
        let populations = [5_u64; 250];
        let summary = observe_series(250, 5, &populations, 1.0);

        let ticks: Vec<u64> = summary.samples.iter().map(|s| s.tick).collect();
        assert_eq!(ticks, vec![100, 200, 250], "tick 0 is never sampled");
        assert_eq!(summary.samples[0].births_total, 100);

        // A horizon that is itself a multiple of the cadence yields one
        // sample for the final tick, not two.
        let exact = observe_series(200, 5, &populations[..200], 1.0);
        let exact_ticks: Vec<u64> = exact.samples.iter().map(|s| s.tick).collect();
        assert_eq!(exact_ticks, vec![100, 200]);
    }

    #[test]
    fn a_horizon_with_no_executed_ticks_reports_the_seeded_population() {
        let summary = observe_series(0, 6, &[], 1.0);

        assert_eq!(summary.peak_population, 6);
        assert_eq!(summary.final_population, 6);
        assert_eq!(summary.plateau_population, None);
        assert_eq!(summary.mean_energy, None);
        assert!(summary.samples.is_empty());
    }

    #[test]
    fn omitting_food_coverage_leaves_production_coverage_untouched() {
        let params = ProfileParams {
            recipe: None,
            name: "sweep".to_string(),
            width: 32,
            height: 32,
            founders: 8,
            seeds: vec![1],
            ticks: 10,
            food_coverage: None,
            neighborhood: NeighborhoodSizes::default(),
            drift: Default::default(),
        };
        let config = build_config(&params);

        let default_config = SimulationConfig::default();
        let coverages: Vec<f32> = config
            .world
            .food
            .types
            .iter()
            .map(|t| t.initial_coverage)
            .collect();
        let default_coverages: Vec<f32> = default_config
            .world
            .food
            .types
            .iter()
            .map(|t| t.initial_coverage)
            .collect();

        assert_eq!(coverages, default_coverages);
        assert_eq!(coverages, vec![0.54]);
        assert!(
            (config.world.food.shared.initial_coverage - 0.54).abs() < 1e-6,
            "normalize() makes the shared coverage follow the primary food type"
        );
    }

    #[test]
    fn forcing_food_coverage_applies_it_to_every_food_type() {
        let config = build_config(&gate_profile_params());

        for food_type in &config.world.food.types {
            assert!((food_type.initial_coverage - 1.0).abs() < 1e-6);
        }
    }

    #[test]
    fn lineage_diversity_handles_empty_one_balanced_and_unequal_populations() {
        assert_eq!(
            lineage_diversity(11, []).shannon_entropy_nats,
            UNDEFINED,
            "empty populations have undefined entropy"
        );
        assert_eq!(lineage_diversity(11, [7]).shannon_entropy_nats, "0.000000");
        assert_eq!(
            lineage_diversity(11, [3, 9]).shannon_entropy_nats,
            "0.693147"
        );
        assert_eq!(
            lineage_diversity(11, [3, 3, 3, 9]).shannon_entropy_nats,
            "0.562335"
        );
    }

    #[test]
    fn temporal_report_keeps_substrate_counts_separate() {
        use v3_core::contracts::NodeId;
        use v3_core::creature::genome::cgp::{
            ActionSlot, ActionSlotBehavior, CgpGraphBackendDef, ComputeNode, ComputeNodeKind,
            ExecuteGate, GraphEdge, GraphSource, WorldActionKind,
        };
        use v3_core::creature::genome::{BackendDef, CreatureGenome, NodeGenome};
        let mut config = SimulationConfig::default();
        config.population.initial_creatures = 1;
        let mut sim = seed_simulation(config, 11);
        let id = sim.creatures.keys().next().unwrap();
        let edge = GraphEdge {
            source: GraphSource::SharedMemory {
                slot: 0,
                previous: true,
            },
            weight: 1.0,
        };
        sim.creatures[id].genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![],
                targets: vec![],
                backend_def: BackendDef::Graph(CgpGraphBackendDef {
                    compute_nodes: vec![ComputeNode {
                        kind: ComputeNodeKind::Constant(0.0),
                        inputs: vec![],
                        plasticity: None,
                    }],
                    output_sinks: vec![],
                    action_bank: vec![ActionSlot {
                        behavior: ActionSlotBehavior::Emit(WorldActionKind::Eat),
                        gate_inputs: vec![edge],
                        param_inputs: vec![],
                    }],
                    execute_gate: ExecuteGate { inputs: vec![edge] },
                }),
            }],
        };
        sim.creatures[id].prev_shared_memory[0] = 1.0;
        let report = temporal_memory_sensitivity(11, &sim);
        assert_eq!(report.previous_slots.final_creature_count, 1);
        assert_eq!(report.previous_slots.different_from_zeroed_count, 1);
        assert_eq!(report.previous_slots.different_from_scrambled_count, 1);
        for component in [report.persisted_outputs, report.operator_state] {
            assert_eq!(component.final_creature_count, 1);
            assert_eq!(component.different_from_either_count, 0);
        }
    }

    #[test]
    fn historical_goal_indicators_default_temporal_memory_to_undefined() {
        let report: Report = serde_json::from_str(include_str!(
            "../../../docs/progress/features/t11-f04-mutation-supply-and-neutral-scaffold.json"
        ))
        .unwrap();
        assert_eq!(
            report
                .deterministic
                .goal_indicators
                .temporal_memory_sensitivity,
            undefined_temporal_memory_sensitivity()
        );
        assert_eq!(
            report.deterministic.graph_work_definition,
            "graph_relax_iters: entered relaxation passes (before T11.F06)"
        );
    }

    #[test]
    fn memory_sensitivity_counts_full_action_differences_and_union_once() {
        let mut config = SimulationConfig::default();
        config.world.width = 16;
        config.world.height = 16;
        config.population.initial_creatures = 3;
        let sim = seed_simulation(config, 11);
        let ids: Vec<_> = sim.creatures.keys().collect();
        let intact = vec![WorldAction::Reproduce {
            direction: Direction::N,
            energy_transfer: 1.0,
        }];
        let payload_changed = vec![WorldAction::Reproduce {
            direction: Direction::N,
            energy_transfer: 2.0,
        }];
        let direction_changed = vec![WorldAction::Reproduce {
            direction: Direction::E,
            energy_transfer: 1.0,
        }];
        let readings = vec![
            FinalActionObservation {
                creature_id: ids[0],
                intact: intact.clone(),
                zeroed: payload_changed.clone(),
                scrambled: intact.clone(),
            },
            FinalActionObservation {
                creature_id: ids[1],
                intact: intact.clone(),
                zeroed: intact.clone(),
                scrambled: direction_changed.clone(),
            },
            FinalActionObservation {
                creature_id: ids[2],
                intact: intact.clone(),
                zeroed: payload_changed,
                scrambled: direction_changed,
            },
        ];

        let indicator = memory_sensitivity(11, &readings);
        assert_eq!(indicator.different_from_zeroed_count, 2);
        assert_eq!(indicator.different_from_scrambled_count, 2);
        assert_eq!(indicator.different_from_either_count, 3);
        assert_eq!(indicator.different_from_either_fraction, "1.000000");
        let empty = memory_sensitivity(11, &[]);
        assert_eq!(empty.final_creature_count, 0);
        assert_eq!(empty.different_from_either_fraction, UNDEFINED);
    }

    proptest! {
        #[test]
        fn lineage_diversity_count_is_the_number_of_distinct_surviving_lineages(
            lineage_ids in proptest::collection::vec(0_u32..32, 0..128),
        ) {
            let result = lineage_diversity(11, lineage_ids.iter().copied());
            let distinct: std::collections::BTreeSet<_> = lineage_ids.iter().copied().collect();
            prop_assert_eq!(result.surviving_founder_clade_count, distinct.len() as u64);
            if lineage_ids.is_empty() {
                prop_assert_eq!(result.shannon_entropy_nats, UNDEFINED);
            } else {
                prop_assert_ne!(result.shannon_entropy_nats, UNDEFINED);
            }
        }

        #[test]
        fn memory_sensitivity_union_and_fractions_match_generated_differences(
            differences in proptest::collection::vec((any::<bool>(), any::<bool>()), 0..64),
        ) {
            let mut config = SimulationConfig::default();
            config.world.width = 16;
            config.world.height = 16;
            config.population.initial_creatures = 1;
            let sim = seed_simulation(config, 11);
            let creature_id = sim.creatures.keys().next().expect("one founder");
            let intact = vec![WorldAction::NoOp];
            let changed = vec![WorldAction::Move(Direction::N)];
            let observations: Vec<_> = differences
                .iter()
                .map(|&(zeroed_differs, scrambled_differs)| FinalActionObservation {
                    creature_id,
                    intact: intact.clone(),
                    zeroed: if zeroed_differs { changed.clone() } else { intact.clone() },
                    scrambled: if scrambled_differs { changed.clone() } else { intact.clone() },
                })
                .collect();

            let result = memory_sensitivity(11, &observations);
            let zeroed_count = differences.iter().filter(|(zeroed, _)| *zeroed).count() as u64;
            let scrambled_count = differences.iter().filter(|(_, scrambled)| *scrambled).count() as u64;
            let both_count = differences.iter().filter(|(zeroed, scrambled)| *zeroed && *scrambled).count() as u64;
            let union_count = zeroed_count + scrambled_count - both_count;

            prop_assert_eq!(result.different_from_zeroed_count, zeroed_count);
            prop_assert_eq!(result.different_from_scrambled_count, scrambled_count);
            prop_assert_eq!(result.different_from_either_count, union_count);
            prop_assert!(union_count >= zeroed_count && union_count >= scrambled_count);
            if differences.is_empty() {
                prop_assert_eq!(result.different_from_zeroed_fraction, UNDEFINED);
                prop_assert_eq!(result.different_from_scrambled_fraction, UNDEFINED);
                prop_assert_eq!(result.different_from_either_fraction, UNDEFINED);
            } else {
                let total = differences.len() as f64;
                prop_assert_eq!(result.different_from_zeroed_fraction, six(zeroed_count as f64 / total));
                prop_assert_eq!(result.different_from_scrambled_fraction, six(scrambled_count as f64 / total));
                prop_assert_eq!(result.different_from_either_fraction, six(union_count as f64 / total));
            }
        }
    }

    // ── Mutational neighborhood (T11.F01) ───────────────────────────────

    fn small_profile(name: &str) -> ProfileParams {
        ProfileParams {
            recipe: None,
            name: name.to_string(),
            width: 8,
            height: 8,
            founders: 4,
            seeds: vec![1],
            ticks: 2,
            food_coverage: Some(1.0),
            neighborhood: NeighborhoodSizes::default(),
            drift: Default::default(),
        }
    }

    #[test]
    fn neighborhood_tally_conversion_computes_fractions_against_applied_not_trials() {
        let tally = Tally {
            trials: 10,
            skipped: 2,
            silent: 3,
            changed: 4,
            dead: 1,
            changed_only_in_sequences: 1,
            differing_executions_total: 40,
            total_executions_total: 400,
        };
        let report = to_neighborhood_tally(&tally);
        assert_eq!(report.applied, 8);
        assert_eq!(report.silent_fraction, "0.375000");
        assert_eq!(report.changed_fraction, "0.500000");
        assert_eq!(report.dead_fraction, "0.125000");
    }

    #[test]
    fn neighborhood_tally_conversion_reports_undefined_fractions_when_nothing_applied() {
        let tally = Tally {
            trials: 5,
            skipped: 5,
            ..Tally::default()
        };
        let report = to_neighborhood_tally(&tally);
        assert_eq!(report.applied, 0);
        assert_eq!(report.silent_fraction, UNDEFINED);
        assert_eq!(report.changed_fraction, UNDEFINED);
        assert_eq!(report.dead_fraction, UNDEFINED);
    }

    #[test]
    fn neighborhood_battery_execution_count_is_48_snapshots_plus_32_sequence_ticks() {
        assert_eq!(neighborhood_battery_execution_count(), 80);
    }

    #[test]
    fn to_neighborhood_operator_rows_preserves_every_row_in_order() {
        let rows = vec![
            OperatorRow {
                family: "vm",
                operator: "Example".to_string(),
                tally: Tally::default(),
            },
            OperatorRow {
                family: "graph",
                operator: "Other".to_string(),
                tally: Tally::default(),
            },
        ];
        let converted = to_neighborhood_operator_rows(&rows);
        assert_eq!(converted.len(), 2);
        assert_eq!(converted[0].family, "vm");
        assert_eq!(converted[0].operator, "Example");
        assert_eq!(converted[1].family, "graph");
        assert_eq!(converted[1].operator, "Other");
    }

    /// `evolved_neighborhood_for_seed` seeds sampled genome `i` (in rank
    /// order) by `EVOLVED_SEED_MULTIPLIER * (i + 1)`. This reconstructs the
    /// same per-genome reading independently, via the same public
    /// `evaluate_genome` seam, and checks it against the function's own
    /// output: a `+`, `/`, or a `*` swapped for the `+` inside
    /// `(genome_index + 1)` would draw a different seed offset and, with
    /// overwhelming likelihood, a different reading.
    #[test]
    fn evolved_neighborhood_for_seed_offsets_each_sampled_genome_by_its_rank_order() {
        let mut config = SimulationConfig::default();
        config.world.width = 16;
        config.world.height = 16;
        config.population.initial_creatures = 3;
        let sim = seed_simulation(config.clone(), 11);
        let battery = Battery::generate(config.world.food.types.len());
        let context = EvalContext::from_config(&config);
        // Larger than `NeighborhoodSizes::default()`'s fixture sizes: at the
        // production ~8.8% mutated birth rate and near-deterministic
        // per-operator classes, a handful of trials can coincidentally match
        // under a wrong seed offset. 60 births and 3 operator trials make
        // that implausible while staying well under a second for 3 genomes.
        let sizes = NeighborhoodSizes {
            evolved_operator_trials: 3,
            evolved_births: 60,
            ..NeighborhoodSizes::default()
        };

        let actual =
            evolved_neighborhood_for_seed(11, &sim, &battery, &config.mutation, &context, sizes);

        let mut creature_ids: Vec<_> = sim.creatures.keys().collect();
        creature_ids.sort();
        assert_eq!(
            actual.sampled_genomes.len(),
            creature_ids.len(),
            "a population below the sample size takes every rank"
        );

        for (genome_index, &creature_id) in creature_ids.iter().enumerate() {
            let creature = &sim.creatures[creature_id];
            let seed_offset =
                v3_core::neighborhood::EVOLVED_SEED_MULTIPLIER * (genome_index as u64 + 1);
            let expected = evaluate_genome(
                &creature.genome,
                &battery,
                &config.mutation,
                &context,
                sizes.evolved_operator_trials,
                sizes.evolved_births,
                seed_offset,
            );
            assert_eq!(
                actual.sampled_genomes[genome_index].operator_rows,
                to_neighborhood_operator_rows(&expected.operator_rows),
                "genome_index {genome_index}"
            );
            assert_eq!(
                actual.sampled_genomes[genome_index].births,
                to_neighborhood_births(&expected.births),
                "genome_index {genome_index}"
            );
        }
    }

    #[test]
    fn generated_report_with_requested_births_roundtrips_through_outer_indicator() {
        let report = build_report(&small_profile("gate"), "t11-f04-report-roundtrip")
            .expect("a valid profile");
        let encoded = serde_json::to_string(&report).unwrap();
        let decoded: Report = serde_json::from_str(&encoded).unwrap();
        assert_eq!(
            deterministic_block_json(&decoded),
            deterministic_block_json(&report)
        );
    }

    proptest::proptest! {
        #[test]
        fn requested_birth_histogram_roundtrips(
            histogram in proptest::collection::btree_map(0u32..10, 1u32..100, 0..10),
        ) {
            let result = BirthResult {
                births_total: histogram.values().sum(),
                by_requested_events: histogram,
                ..BirthResult::default()
            };
            let report = to_neighborhood_births(&result);
            let encoded = serde_json::to_value(&report).unwrap();
            let decoded: NeighborhoodBirths = serde_json::from_value(encoded.clone()).unwrap();
            proptest::prop_assert_eq!(&decoded, &report);
            let wrapped = Indicator::Defined(report.clone());
            let wrapped_encoded = serde_json::to_string(&wrapped).unwrap();
            let decoded: Indicator<NeighborhoodBirths> = serde_json::from_str(&wrapped_encoded).unwrap();
            proptest::prop_assert_eq!(decoded, wrapped);
            let mut historical = encoded;
            historical.as_object_mut().unwrap().remove("by_requested_events");
            let decoded: NeighborhoodBirths = serde_json::from_value(historical).unwrap();
            proptest::prop_assert!(decoded.by_requested_events.is_empty());
        }
    }

    #[test]
    fn merge_birth_results_sums_totals_and_merges_matching_buckets() {
        let mut a = BirthResult {
            births_total: 10,
            zero_event_births: 4,
            ..BirthResult::default()
        };
        a.any_events = Tally {
            trials: 6,
            silent: 2,
            changed: 3,
            dead: 1,
            ..Tally::default()
        };
        a.by_events.insert(1, a.any_events);
        a.by_requested_events.insert(0, 4);
        a.by_requested_events.insert(2, 6);

        let mut b = BirthResult {
            births_total: 5,
            zero_event_births: 1,
            ..BirthResult::default()
        };
        b.any_events = Tally {
            trials: 4,
            silent: 1,
            changed: 3,
            ..Tally::default()
        };
        b.by_events.insert(1, b.any_events);
        b.by_requested_events.insert(0, 1);
        b.by_requested_events.insert(2, 4);

        let merged = a.merge(&b);
        assert_eq!(merged.births_total, 15);
        assert_eq!(merged.zero_event_births, 5);
        assert_eq!(merged.any_events.trials, 10);
        assert_eq!(merged.by_events[&1].trials, 10);
        assert_eq!(
            merged.by_requested_events,
            BTreeMap::from([(0, 5), (2, 10)])
        );
        assert_eq!(
            to_neighborhood_births(&merged).by_requested_events,
            vec![
                NeighborhoodRequestedBirthBucket {
                    requested_events: 0,
                    births: 5
                },
                NeighborhoodRequestedBirthBucket {
                    requested_events: 2,
                    births: 10
                },
            ]
        );
    }

    #[test]
    fn goal_indicators_defaults_mutational_neighborhood_to_undefined_when_the_field_is_absent() {
        let report = build_report(&small_profile("synthetic"), "t11-f01-serde-default-check")
            .expect("a valid profile");
        let mut value = serde_json::to_value(&report).expect("a report always serializes");
        value["deterministic"]["goal_indicators"]
            .as_object_mut()
            .expect("goal_indicators is an object")
            .remove("mutational_neighborhood");

        let reparsed: Report =
            serde_json::from_value(value).expect("a missing field falls back to the serde default");
        assert!(matches!(
            reparsed.deterministic.goal_indicators.mutational_neighborhood,
            Indicator::Undefined(ref value) if value == "Undefined"
        ));
    }

    #[test]
    fn evolved_depth_uses_whole_population_and_actual_sample_generation() {
        let mut config = SimulationConfig::default();
        config.world.width = 16;
        config.world.height = 16;
        config.population.initial_creatures = 25;
        let mut sim = seed_simulation(config.clone(), 11);
        let mut ids: Vec<_> = sim.creatures.keys().collect();
        ids.sort();
        let ranks = evolved_sample_ranks(ids.len());
        let high = u64::from(u32::MAX) + 12;
        for (rank, id) in ids.iter().enumerate() {
            sim.creatures.get_mut(*id).unwrap().generation = if rank == ranks[0] {
                high + 1
            } else if ranks.contains(&rank) {
                3
            } else {
                high
            };
        }
        let battery = Battery::generate(config.world.food.types.len());
        let reading = evolved_neighborhood_for_seed(
            11,
            &sim,
            &battery,
            &config.mutation,
            &EvalContext::from_config(&config),
            NeighborhoodSizes::default(),
        );
        assert_eq!(reading.final_population_size, 25);
        assert_eq!(
            reading.generation_distribution,
            Indicator::Defined(GenerationDistribution {
                median: high,
                max: high + 1
            })
        );
        for sample in reading.sampled_genomes {
            assert_eq!(
                sample.generation,
                Some(sim.creatures[ids[sample.rank as usize]].generation)
            );
            assert_eq!(
                sample.generation,
                Some(if sample.rank == ranks[0] as u64 {
                    high + 1
                } else {
                    3
                })
            );
            assert!(matches!(sample.mesh_execution, Indicator::Defined(_)));
        }
    }

    #[test]
    fn mesh_report_deterministic_fields_match_across_runs_and_threads() {
        for profile in ["gate", "goal"] {
            let params = small_profile(profile);
            let run = |threads| {
                rayon::ThreadPoolBuilder::new()
                    .num_threads(threads)
                    .build()
                    .unwrap()
                    .install(|| {
                        serde_json::to_vec(&run_deterministic(&params).expect("a valid profile").0)
                            .unwrap()
                    })
            };
            assert_eq!(run(1), run(2));
        }
    }

    #[test]
    fn population_depth_uses_u64_upper_median_and_extinction_is_undefined() {
        assert!(matches!(
            generation_distribution(vec![]),
            Indicator::Undefined(_)
        ));
        assert_eq!(
            generation_distribution(vec![9, 1, 5]),
            Indicator::Defined(GenerationDistribution { median: 5, max: 9 })
        );
        let large = u64::from(u32::MAX) + 9;
        assert_eq!(
            generation_distribution(vec![large, 0, 2, 1]),
            Indicator::Defined(GenerationDistribution {
                median: 2,
                max: large
            })
        );
    }

    #[test]
    fn historical_mesh_fields_default_to_unmeasured() {
        let (det, _) = run_deterministic(&small_profile("goal")).expect("a valid profile");
        let Indicator::Defined(neighborhood) = det.goal_indicators.mutational_neighborhood else {
            panic!("goal reading");
        };
        let mut founder = serde_json::to_value(&neighborhood.founder).unwrap();
        founder.as_object_mut().unwrap().remove("mesh_execution");
        founder.as_object_mut().unwrap().remove("generation");
        let founder: NeighborhoodFounderHalf = serde_json::from_value(founder).unwrap();
        assert!(matches!(founder.mesh_execution, Indicator::Undefined(_)));
        assert_eq!(founder.generation, None);
        let Indicator::Defined(evolved) = neighborhood.evolved else {
            panic!("evolved reading");
        };
        let mut seed = serde_json::to_value(&evolved.per_seed[0]).unwrap();
        seed.as_object_mut()
            .unwrap()
            .remove("generation_distribution");
        for sample in seed["sampled_genomes"].as_array_mut().unwrap() {
            sample.as_object_mut().unwrap().remove("mesh_execution");
            sample.as_object_mut().unwrap().remove("generation");
        }
        let seed: NeighborhoodEvolvedSeed = serde_json::from_value(seed).unwrap();
        assert!(matches!(
            seed.generation_distribution,
            Indicator::Undefined(_)
        ));
        for sample in seed.sampled_genomes {
            assert_eq!(sample.generation, None);
            assert!(matches!(sample.mesh_execution, Indicator::Undefined(_)));
        }
    }

    #[test]
    fn mutational_neighborhood_is_defined_only_for_the_gate_and_goal_profile_names() {
        let (gate_det, _) = run_deterministic(&small_profile("gate")).expect("a valid profile");
        let Indicator::Defined(gate_neighborhood) =
            gate_det.goal_indicators.mutational_neighborhood
        else {
            panic!("the gate profile must define mutational_neighborhood");
        };
        assert_eq!(gate_neighborhood.founder.generation, Some(0));
        let Indicator::Defined(mesh) = &gate_neighborhood.founder.mesh_execution else {
            panic!("founder mesh reading");
        };
        let counts = mesh.backends.expect("present backend measurement");
        assert_eq!(counts.graph.total + counts.vm.total, mesh.total_node_count);
        assert_eq!(
            counts.graph.executed + counts.vm.executed,
            mesh.executed_node_count
        );
        assert_eq!(
            counts.graph.contributing + counts.vm.contributing,
            mesh.executed_node_count - mesh.knockout_count
        );
        let mut historical = serde_json::to_value(mesh).unwrap();
        historical.as_object_mut().unwrap().remove("backends");
        let historical: MeshExecution = serde_json::from_value(historical).unwrap();
        assert_eq!(historical.backends, None);
        assert_eq!(historical.total_node_count, mesh.total_node_count);
        assert_eq!(mesh.version, "mesh-execution-v1");
        assert_eq!(mesh.executions_per_genome, 80);
        assert_eq!(mesh.snapshot_route_probes, 48);
        assert_eq!(mesh.knockout_method, "static-successor-bypass-v1");
        assert!(
            matches!(gate_neighborhood.evolved, Indicator::Undefined(_)),
            "the evolved half never runs in the gate profile"
        );

        let (goal_det, _) = run_deterministic(&small_profile("goal")).expect("a valid profile");
        let Indicator::Defined(goal_neighborhood) =
            goal_det.goal_indicators.mutational_neighborhood
        else {
            panic!("the goal profile must define mutational_neighborhood");
        };
        assert!(
            matches!(goal_neighborhood.evolved, Indicator::Defined(_)),
            "the evolved half runs in the goal profile"
        );

        let (synthetic_det, _) =
            run_deterministic(&small_profile("synthetic")).expect("a valid profile");
        assert!(matches!(
            synthetic_det.goal_indicators.mutational_neighborhood,
            Indicator::Undefined(_)
        ));

        let (sweep_det, _) = run_deterministic(&small_profile("sweep")).expect("a valid profile");
        assert!(matches!(
            sweep_det.goal_indicators.mutational_neighborhood,
            Indicator::Undefined(_)
        ));
    }
    #[test]
    fn recipe_profile_identifies_final_config_without_fabricating_legacy_metadata() {
        let mut params = small_profile("sweep");
        let legacy =
            serde_json::to_value(profile_block(&params, &build_config(&params), &[])).unwrap();
        assert!(legacy.get("recipe_path").is_none());
        assert!(legacy.get("config_digest").is_none());
        let old: ProfileBlock = serde_json::from_value(legacy).unwrap();
        assert!(old.config_digest.is_none());
        let config = v3_core::config::resolve_config(&SimulationConfig::default(), serde_json::json!({"world":{"world_seed":18446744073709551615u64},"energy":{"costs":{"move_cost":0.75}}})).unwrap();
        params.recipe = Some(Recipe {
            path: "world.json".into(),
            config,
        });
        params.food_coverage = None;
        let applied = build_config(&params);
        let profile = profile_block(&params, &applied, &[]);
        assert_eq!(profile.recipe_path.as_deref(), Some("world.json"));
        assert_eq!(
            profile.config_digest,
            Some(v3_core::config::config_digest(&applied))
        );
        assert_eq!(profile.food_coverage, "recipe");
        assert_eq!(profile.world_width, applied.world.width);
        assert_eq!(profile.founders, applied.population.initial_creatures);
        let sim = seed_simulation(applied.clone(), 1);
        assert_eq!(sim.config.world.world_seed, Some(u64::MAX));
        assert_eq!(sim.config.energy.costs.move_cost, 0.75);
        let (observed, _) = run_deterministic(&params).expect("a valid profile");
        assert_eq!(observed.profile, profile);
        params
            .recipe
            .as_mut()
            .unwrap()
            .config
            .energy
            .costs
            .move_cost = 0.5;
        assert_ne!(profile_block(&params, &build_config(&params), &[]), profile);
        params.food_coverage = Some(2.0);
        assert_eq!(
            profile_block(&params, &build_config(&params), &[]).food_coverage,
            "1.000000"
        );
    }
    #[test]
    fn recipe_equality_requires_both_source_path_and_complete_config() {
        let recipe = Recipe {
            path: "world.json".into(),
            config: SimulationConfig::default(),
        };
        assert_eq!(recipe, recipe.clone());
        let mut changed = recipe.clone();
        changed.path = "other.json".into();
        assert_ne!(recipe, changed);
        changed = recipe.clone();
        changed.config.world.world_seed = Some(u64::MAX);
        assert_ne!(recipe, changed);
    }

    /// Each world-set case reports the same cumulative behavior a replay of
    /// that case's own config and seed produces, and every fraction is derived
    /// from those totals rather than measured separately.
    #[test]
    fn world_set_case_tracking_matches_a_replayed_run() {
        let mut params = goal_profile_params();
        params.width = 24;
        params.height = 24;
        params.founders = 16;
        params.ticks = 4;
        params.neighborhood = NeighborhoodSizes::default();
        params.drift = Default::default();
        let (report, _) = run_deterministic(&params).expect("a valid profile");

        for (index, case) in report.goal_indicators.cases.iter().enumerate() {
            let (_, config) = goal_case(&params, &GOAL_RECIPES[index]);
            let mut sim = seed_simulation(config, case.case.seed);
            for _ in 0..params.ticks {
                run_tick(&mut sim, &mut None);
                if sim.creatures.is_empty() {
                    break;
                }
            }
            let expected = WorldTracking::observe(&sim);
            assert_eq!(case.tracking, expected, "case {}", case.case.name);
            assert_eq!(
                case.tracking.typed_eats_total.len(),
                case.case.food_type_count,
                "one eat counter per configured food type"
            );
            assert_eq!(
                case.tracking.food_density_total.len(),
                case.case.food_type_count
            );
            assert_eq!(case.fractions, expected.fractions());
        }
    }

    /// The world-set tracking fractions read against the totals they came
    /// from, and the two reader-state fractions use their own denominators:
    /// the barrier-block rate is per state, against that state's own attempts
    /// beside a barrier; the avoidable share is against every move the whole
    /// population attempted.
    #[test]
    fn tracking_fractions_divide_each_reading_by_its_own_denominator() {
        let tracking = WorldTracking {
            typed_eats_total: vec![3, 1],
            food_density_total: vec![six(1.5)],
            moves_attempted_total: 8,
            moves_blocked_barrier_total: 2,
            moves_blocked_total_by_cause: Some(MovesBlockedByCause {
                barrier: 2,
                occupied: 3,
                out_of_bounds: 1,
            }),
            moves_blocked_avoidable_by_reader_state: ByReaderState {
                has_barrier_reader: 1,
                no_barrier_reader: 3,
            },
            move_attempts_with_barrier_neighbor_by_reader_state: ByReaderState {
                has_barrier_reader: 4,
                no_barrier_reader: 16,
            },
            moves_blocked_barrier_with_barrier_neighbor_by_reader_state: ByReaderState {
                has_barrier_reader: 1,
                no_barrier_reader: 10,
            },
        };
        let fractions = tracking.fractions();
        assert_eq!(fractions.typed_eat_share, vec![six(0.75), six(0.25)]);
        assert_eq!(fractions.blocked_move_fraction, six(0.25));
        assert_eq!(
            fractions.barrier_blocked_fraction_by_reader_state,
            ByReaderState {
                has_barrier_reader: six(0.25),
                no_barrier_reader: six(0.625),
            },
            "each state's barrier blocks against that same state's attempts beside a barrier"
        );
        assert_eq!(
            fractions.avoidable_blocked_share_of_all_moves_by_reader_state,
            ByReaderState {
                has_barrier_reader: six(0.125),
                no_barrier_reader: six(0.375),
            }
        );

        let empty = WorldTracking {
            typed_eats_total: vec![0, 0],
            ..WorldTracking::default()
        }
        .fractions();
        assert_eq!(empty.typed_eat_share, vec![UNDEFINED, UNDEFINED]);
        assert_eq!(empty.blocked_move_fraction, UNDEFINED);
        assert_eq!(
            empty.barrier_blocked_fraction_by_reader_state,
            ByReaderState {
                has_barrier_reader: UNDEFINED.to_string(),
                no_barrier_reader: UNDEFINED.to_string(),
            },
            "a state that never moved beside a barrier is unmeasured, not zero"
        );
        assert_eq!(
            empty.avoidable_blocked_share_of_all_moves_by_reader_state,
            ByReaderState {
                has_barrier_reader: UNDEFINED.to_string(),
                no_barrier_reader: UNDEFINED.to_string(),
            }
        );

        let barrier_free = WorldTracking {
            moves_attempted_total: 10,
            moves_blocked_avoidable_by_reader_state: ByReaderState {
                has_barrier_reader: 0,
                no_barrier_reader: 2,
            },
            ..WorldTracking::default()
        }
        .fractions();
        assert_eq!(
            barrier_free.barrier_blocked_fraction_by_reader_state,
            ByReaderState {
                has_barrier_reader: UNDEFINED.to_string(),
                no_barrier_reader: UNDEFINED.to_string(),
            },
            "a world without barriers reports no barrier-block rate at all"
        );
        assert_eq!(
            barrier_free
                .avoidable_blocked_share_of_all_moves_by_reader_state
                .no_barrier_reader,
            six(0.2),
            "the avoidable share still counts blocks of every other cause"
        );
    }

    /// Every blocked-move cause is read from its own key of
    /// `move_actions_blocked_total_by_cause`, and the kept
    /// `moves_blocked_barrier_total` is exactly that map's barrier entry, so
    /// the two can never disagree.
    #[test]
    fn observe_reads_each_blocked_move_cause_from_its_own_stats_key() {
        use v3_core::simulation::actions::MoveBlockedCause::{Barrier, Occupied, OutOfBounds};
        let params = small_profile("sweep");
        let mut sim = seed_simulation(build_config(&params), 1);
        assert_eq!(
            WorldTracking::observe(&sim).moves_blocked_total_by_cause,
            Some(MovesBlockedByCause::default()),
            "a measured run whose creatures never hit a cause reads zero, not absent"
        );
        for (cause, count) in [(Barrier, 5), (Occupied, 9), (OutOfBounds, 13)] {
            sim.stats
                .move_actions_blocked_total_by_cause
                .insert(cause, count);
        }

        let tracking = WorldTracking::observe(&sim);
        assert_eq!(
            tracking.moves_blocked_total_by_cause,
            Some(MovesBlockedByCause {
                barrier: 5,
                occupied: 9,
                out_of_bounds: 13,
            })
        );
        assert_eq!(
            tracking.moves_blocked_barrier_total, 5,
            "the kept barrier total is the map's barrier entry"
        );
    }

    /// Each barrier counter comes from its own stats map under its own
    /// reader-state key. The barrier-block rate's numerator and denominator
    /// are separate measurements, so reading either from the other's map, or
    /// under the other state's key, would misreport barrier awareness.
    #[test]
    fn observe_reads_each_barrier_counter_from_its_own_stats_map() {
        use v3_core::simulation::actions::BarrierReaderState::{HasBarrierReader, NoBarrierReader};
        let params = small_profile("sweep");
        let mut sim = seed_simulation(build_config(&params), 1);
        for (state, attempts, blocked, avoidable) in
            [(HasBarrierReader, 7, 3, 5), (NoBarrierReader, 11, 2, 13)]
        {
            sim.stats
                .move_attempts_with_barrier_neighbor_total_by_reader_state
                .insert(state, attempts);
            sim.stats
                .move_blocked_barrier_with_barrier_neighbor_total_by_reader_state
                .insert(state, blocked);
            sim.stats
                .move_actions_blocked_avoidable_total_by_reader_state
                .insert(state, avoidable);
        }

        let tracking = WorldTracking::observe(&sim);
        assert_eq!(
            tracking.move_attempts_with_barrier_neighbor_by_reader_state,
            ByReaderState {
                has_barrier_reader: 7,
                no_barrier_reader: 11,
            }
        );
        assert_eq!(
            tracking.moves_blocked_barrier_with_barrier_neighbor_by_reader_state,
            ByReaderState {
                has_barrier_reader: 3,
                no_barrier_reader: 2,
            }
        );
        assert_eq!(
            tracking.moves_blocked_avoidable_by_reader_state,
            ByReaderState {
                has_barrier_reader: 5,
                no_barrier_reader: 13,
            }
        );
    }

    /// Every persistence sample carries the tracking counters as of its tick,
    /// on the same cadence `births_total` uses, and they never go backwards.
    #[test]
    fn persistence_samples_carry_world_tracking_on_the_births_cadence() {
        let mut params = small_profile("sweep");
        params.width = 24;
        params.height = 24;
        params.founders = 16;
        params.ticks = SAMPLE_EVERY_TICKS + 5;
        let (report, _) = run_deterministic(&params).expect("a valid profile");
        let seed = &report.goal_indicators.population_persistence.per_seed[0];
        assert_eq!(
            seed.samples.iter().map(|s| s.tick).collect::<Vec<_>>(),
            vec![SAMPLE_EVERY_TICKS, params.ticks],
            "the cadence is every hundredth tick plus the final tick"
        );
        let mut previous = 0;
        for sample in &seed.samples {
            assert_eq!(sample.tracking.typed_eats_total.len(), 1);
            assert_eq!(sample.tracking.food_density_total.len(), 1);
            assert!(
                sample.tracking.moves_attempted_total >= previous,
                "move attempts are cumulative"
            );
            previous = sample.tracking.moves_attempted_total;
            assert!(
                sample.tracking.moves_blocked_barrier_total
                    <= sample.tracking.moves_attempted_total
            );
            let by_cause = sample
                .tracking
                .moves_blocked_total_by_cause
                .as_ref()
                .expect("a measured sample carries every blocked-move cause");
            assert_eq!(
                by_cause.barrier, sample.tracking.moves_blocked_barrier_total,
                "the kept barrier total and the by-cause barrier entry are one measurement"
            );
            assert!(
                by_cause.barrier + by_cause.occupied + by_cause.out_of_bounds
                    <= sample.tracking.moves_attempted_total,
                "every blocked move, of any cause, is one of the moves attempted"
            );
            let beside = &sample
                .tracking
                .move_attempts_with_barrier_neighbor_by_reader_state;
            let blocked_beside = &sample
                .tracking
                .moves_blocked_barrier_with_barrier_neighbor_by_reader_state;
            assert!(
                blocked_beside.has_barrier_reader <= beside.has_barrier_reader
                    && blocked_beside.no_barrier_reader <= beside.no_barrier_reader,
                "a barrier block beside a barrier is one of that state's attempts beside one"
            );
            assert!(
                beside.has_barrier_reader + beside.no_barrier_reader
                    <= sample.tracking.moves_attempted_total,
                "attempts beside a barrier are a subset of every move attempted"
            );
        }
        assert!(previous > 0, "a moving population must attempt moves");
    }

    /// Historical reports predate every tracking field, and a round trip of a
    /// current report keeps them flat on the wire.
    #[test]
    fn tracking_fields_default_when_absent_and_survive_a_round_trip() {
        let legacy: PersistenceSample = serde_json::from_value(serde_json::json!({
            "tick": 100, "population": 5, "mean_energy": "1.000000", "births_total": 2
        }))
        .expect("a pre-T12.F04 sample must still parse");
        assert_eq!(legacy.tracking, WorldTracking::default());

        let sample = PersistenceSample {
            tick: 100,
            population: 5,
            mean_energy: None,
            births_total: 2,
            tracking: WorldTracking {
                typed_eats_total: vec![7],
                food_density_total: vec![six(2.0)],
                moves_attempted_total: 9,
                moves_blocked_barrier_total: 1,
                moves_blocked_total_by_cause: Some(MovesBlockedByCause {
                    barrier: 1,
                    occupied: 2,
                    out_of_bounds: 3,
                }),
                moves_blocked_avoidable_by_reader_state: ByReaderState {
                    has_barrier_reader: 0,
                    no_barrier_reader: 1,
                },
                move_attempts_with_barrier_neighbor_by_reader_state: ByReaderState {
                    has_barrier_reader: 4,
                    no_barrier_reader: 3,
                },
                moves_blocked_barrier_with_barrier_neighbor_by_reader_state: ByReaderState {
                    has_barrier_reader: 2,
                    no_barrier_reader: 1,
                },
            },
        };
        let wire = serde_json::to_value(&sample).unwrap();
        assert_eq!(
            wire["moves_attempted_total"], 9,
            "tracking stays flat: {wire}"
        );
        assert_eq!(
            wire["move_attempts_with_barrier_neighbor_by_reader_state"]["has_barrier_reader"], 4,
            "the barrier-block denominator is on the wire under its own key: {wire}"
        );
        assert_eq!(
            wire["moves_blocked_total_by_cause"]["occupied"], 2,
            "each blocked-move cause is on the wire under its own key: {wire}"
        );
        let decoded: PersistenceSample = serde_json::from_value(wire).unwrap();
        assert_eq!(decoded.tracking, sample.tracking);
    }

    proptest! {
        /// The profile totals are exactly the field-wise sum of the per-case
        /// (world-set) or per-seed rows the same report carries.
        #[test]
        fn profile_totals_are_the_field_wise_sum_of_every_case_row(
            rows in proptest::collection::vec(
                (0u64..1000, 0u64..1000, 0u64..1000, 0u64..1000, 0u64..1000, 0u64..1000, 0u64..1000, 0u64..1000),
                0..6usize,
            )
        ) {
            let per_seed: Vec<PerSeed> = rows
                .iter()
                .enumerate()
                .map(|(index, row)| PerSeed {
                    tick_zero_connectivity: None,
                    seed: index as u64,
                    ticks: row.0,
                    creature_ticks: row.1,
                    mesh_hops: row.2,
                    vm_steps: row.3,
                    graph_relax_iters: row.4,
                    plasticity_updates: row.5,
                    actions_applied: row.6,
                    births: row.7,
                    final_population: 0,
                    extinction_tick: None,
                })
                .collect();
            let totals = accumulate_totals(&per_seed);
            prop_assert_eq!(totals.ticks, rows.iter().map(|r| r.0).sum::<u64>());
            prop_assert_eq!(totals.creature_ticks, rows.iter().map(|r| r.1).sum::<u64>());
            prop_assert_eq!(totals.mesh_hops, rows.iter().map(|r| r.2).sum::<u64>());
            prop_assert_eq!(totals.vm_steps, rows.iter().map(|r| r.3).sum::<u64>());
            prop_assert_eq!(totals.graph_relax_iters, rows.iter().map(|r| r.4).sum::<u64>());
            prop_assert_eq!(totals.plasticity_updates, rows.iter().map(|r| r.5).sum::<u64>());
            prop_assert_eq!(totals.actions_applied, rows.iter().map(|r| r.6).sum::<u64>());
            prop_assert_eq!(totals.births, rows.iter().map(|r| r.7).sum::<u64>());
        }
    }

    /// The world-set profile shrunk to test size: the same recipes, seeds, and
    /// per-case machinery, on a world small enough to run in a test.
    fn small_world_set_params() -> ProfileParams {
        let mut params = goal_profile_params();
        params.width = 16;
        params.height = 16;
        params.founders = 4;
        params.ticks = 1;
        params.neighborhood = NeighborhoodSizes::default();
        params.drift = Default::default();
        params
    }

    fn small_world_set_report() -> Report {
        build_report(&small_world_set_params(), "t12-f04-world-set-check").expect("a valid profile")
    }

    fn case_reading<'a>(comparison: &'a CaseComparison, name: &str) -> &'a CaseReadingComparison {
        comparison
            .readings
            .iter()
            .find(|reading| reading.name == name)
            .unwrap_or_else(|| panic!("reading {name} must be compared"))
    }

    /// A world-set reference whose recipes were edited still compares: the
    /// changed case is labeled `inputs_changed`, its deltas are computed, and a
    /// case the reference never ran is recorded absent rather than dropped.
    #[test]
    fn world_set_comparison_labels_changed_inputs_and_records_absent_cases() {
        let mut current = small_world_set_report();
        let mut reference = current.clone();

        let edited_seed = current.deterministic.profile.cases[1].seed;
        reference.deterministic.profile.cases[1].config_digest = "sha256:edited".to_string();
        let population = |report: &mut Report, seed: u64, value: u64, extinction: Option<u64>| {
            for row in &mut report
                .deterministic
                .goal_indicators
                .population_persistence
                .per_seed
            {
                if row.seed == seed {
                    row.final_population = value;
                    row.extinction_tick = extinction;
                }
            }
        };
        population(&mut current, edited_seed, 200, Some(7));
        population(&mut reference, edited_seed, 100, Some(3));
        let dropped = current.deterministic.profile.cases[2].name.clone();
        reference.deterministic.profile.cases.remove(2);
        reference.deterministic.goal_indicators.cases.remove(2);

        let path = std::env::temp_dir().join(format!(
            "t12-f04-world-set-reference-{}.json",
            std::process::id()
        ));
        std::fs::write(&path, report_json_pretty(&reference)).expect("write the reference");
        let comparison = compare_against_path(&current, &path)
            .expect("a world-set reference whose case digests differ must still be comparable");
        let _ = std::fs::remove_file(&path);

        assert_eq!(comparison.cases.len(), 3);
        assert!(
            !comparison.cases[0].inputs_changed,
            "an untouched recipe is not an input change"
        );
        assert_eq!(
            comparison.cases[0].reference_digest.as_deref(),
            Some(comparison.cases[0].current_digest.as_str())
        );
        assert!(!comparison.cases[0].absent_in_reference);

        let edited = &comparison.cases[1];
        assert!(edited.inputs_changed);
        assert!(!edited.absent_in_reference);
        assert_eq!(edited.reference_digest.as_deref(), Some("sha256:edited"));
        let final_population = case_reading(edited, "final_population");
        assert_eq!(final_population.current.as_deref(), Some("200.000000"));
        assert_eq!(final_population.reference.as_deref(), Some("100.000000"));
        assert_eq!(
            final_population.percent_delta.as_deref(),
            Some("100.000000")
        );
        let extinction = case_reading(edited, "extinction_tick");
        assert_eq!(extinction.current.as_deref(), Some("7.000000"));
        assert_eq!(extinction.reference.as_deref(), Some("3.000000"));
        assert_eq!(
            extinction.percent_delta, None,
            "an extinction tick is a value, not a rate"
        );

        let absent = &comparison.cases[2];
        assert_eq!(absent.case, dropped);
        assert!(absent.absent_in_reference);
        assert!(!absent.inputs_changed);
        assert_eq!(absent.reference_digest, None);
        let reading = case_reading(absent, "final_population");
        assert!(reading.current.is_some());
        assert_eq!(reading.reference, None);
        assert_eq!(reading.percent_delta, None);
        assert!(
            !comparison.severe,
            "a per-case difference never makes a comparison severe by itself"
        );
    }

    /// A profile difference that is not the per-case block is still a hard
    /// error, and a single-config profile records no cases at all.
    #[test]
    fn comparison_still_rejects_a_different_world_and_records_no_cases_off_the_world_set() {
        let current = small_world_set_report();
        let mut reference = current.clone();
        reference.deterministic.profile.ticks += 1;
        let path = std::env::temp_dir().join(format!(
            "t12-f04-world-set-mismatch-{}.json",
            std::process::id()
        ));
        std::fs::write(&path, report_json_pretty(&reference)).expect("write the reference");
        let error = compare_against_path(&current, &path).unwrap_err();
        let _ = std::fs::remove_file(&path);
        assert!(error.contains("different profile"), "{error}");

        let single = build_report(&small_profile("sweep"), "t12-f04-single-config-check")
            .expect("a valid profile");
        assert!(
            compare_against(&single, std::path::Path::new("reference.json"), &single)
                .cases
                .is_empty(),
            "only the world set has cases to compare"
        );
    }

    /// Every per-case reading comes from that case's own seed and its own
    /// observation, is parsed from the report's own strings, and is
    /// unmeasured — never zero — where its denominator is.
    #[test]
    fn case_readings_follow_the_case_seed_and_observation() {
        let mut report = small_world_set_report();
        let cases: Vec<(String, u64)> = report
            .deterministic
            .profile
            .cases
            .iter()
            .map(|case| (case.name.clone(), case.seed))
            .collect();
        let first_seed = cases[0].1;

        // Stamp every row with a value derived from its own seed, so a lookup
        // that takes some other case's row reads a different number. The first
        // case runs no creature-ticks, so its normalized counters are
        // unmeasured rather than a division by zero.
        for row in &mut report.deterministic.per_seed {
            row.creature_ticks = if row.seed == first_seed { 0 } else { 10 };
            row.mesh_hops = row.seed;
            row.births = row.seed * 2;
        }
        for row in &mut report
            .deterministic
            .goal_indicators
            .population_persistence
            .per_seed
        {
            row.final_population = row.seed + 1;
            row.minimum_population = row.seed;
            row.peak_population = row.seed + 2;
            row.plateau_population = Some(six(f64::from(row.seed as u32) / 2.0));
            row.mean_energy = Some(six(f64::from(row.seed as u32) / 4.0));
            row.extinction_tick = Some(row.seed + 3);
        }
        if let Indicator::Defined(lineage) =
            &mut report.deterministic.goal_indicators.lineage_diversity
        {
            for row in &mut lineage.per_seed {
                row.surviving_founder_clade_count = row.seed;
                row.shannon_entropy_nats = six(f64::from(row.seed as u32));
            }
        }
        if let Indicator::Defined(memory) =
            &mut report.deterministic.goal_indicators.memory_sensitivity
        {
            for row in &mut memory.per_seed {
                row.different_from_either_fraction = six(f64::from(row.seed as u32) / 100.0);
            }
        }
        for observation in &mut report.deterministic.goal_indicators.cases {
            let seed = observation.case.seed;
            // Three different multiples of the seed, so reading one cause's
            // total under another cause's name is visible. The first case
            // carries none at all, as a report stored before these totals did.
            observation.tracking.moves_blocked_total_by_cause =
                (seed != first_seed).then(|| MovesBlockedByCause {
                    barrier: seed,
                    occupied: seed * 3,
                    out_of_bounds: seed * 5,
                });
            observation.fractions.typed_eat_share = vec![six(f64::from(seed as u32) / 50.0)];
            observation.fractions.blocked_move_fraction = six(f64::from(seed as u32) / 200.0);
            observation
                .fractions
                .barrier_blocked_fraction_by_reader_state = ByReaderState {
                has_barrier_reader: six(f64::from(seed as u32) / 400.0),
                no_barrier_reader: six(f64::from(seed as u32) / 500.0),
            };
            observation
                .fractions
                .avoidable_blocked_share_of_all_moves_by_reader_state = ByReaderState {
                has_barrier_reader: six(f64::from(seed as u32) / 800.0),
                no_barrier_reader: six(f64::from(seed as u32) / 1_100.0),
            };
            if let Indicator::Defined(drift) = &mut observation.drift_depth {
                // Two checkpoints with different readings, so selecting the
                // wrong depth is visible.
                let mut shallow = drift.readings[0].clone();
                shallow.depth = 1_000;
                shallow.changed_per_all_births = six(0.125);
                let mut deep = drift.readings[0].clone();
                deep.depth = 2_000;
                deep.changed_per_all_births = six(f64::from(seed as u32) / 400.0);
                drift.readings = vec![shallow, deep];
            }
        }

        let value = |case: &str, name: &str| {
            case_readings(&report, case)
                .into_iter()
                .find(|(reading, _)| reading == name)
                .unwrap_or_else(|| panic!("{name} must be a compared reading"))
                .1
        };
        let (second, seed) = (&cases[1].0, cases[1].1);
        let seeded = f64::from(seed as u32);
        assert_eq!(value(second, "final_population"), Some(seeded + 1.0));
        assert_eq!(value(second, "minimum_population"), Some(seeded));
        assert_eq!(value(second, "peak_population"), Some(seeded + 2.0));
        assert_eq!(value(second, "plateau_population"), Some(seeded / 2.0));
        assert_eq!(value(second, "mean_energy"), Some(seeded / 4.0));
        assert_eq!(value(second, "extinction_tick"), Some(seeded + 3.0));
        assert_eq!(value(second, "births"), Some(seeded * 2.0));
        assert_eq!(
            value(second, "mesh_hops_per_creature_tick"),
            Some(seeded / 10.0),
            "a counter is normalized by that case's own creature-ticks"
        );
        assert_eq!(
            value(&cases[0].0, "mesh_hops_per_creature_tick"),
            None,
            "no creature-ticks makes a normalized counter unmeasured, not zero"
        );
        assert_eq!(value(second, "surviving_founder_clade_count"), Some(seeded));
        assert_eq!(value(second, "lineage_shannon_entropy_nats"), Some(seeded));
        assert_eq!(
            value(second, "memory_different_from_either_fraction"),
            Some(seeded / 100.0)
        );
        assert_eq!(value(second, "typed_eat_share_type_0"), Some(seeded / 50.0));
        assert_eq!(value(second, "blocked_move_fraction"), Some(seeded / 200.0));
        assert_eq!(
            value(second, "barrier_blocked_fraction_has_barrier_reader"),
            Some(seeded / 400.0)
        );
        assert_eq!(
            value(second, "barrier_blocked_fraction_no_barrier_reader"),
            Some(seeded / 500.0)
        );
        assert_eq!(
            value(
                second,
                "avoidable_blocked_share_of_all_moves_has_barrier_reader"
            ),
            Some(seeded / 800.0)
        );
        assert_eq!(
            value(
                second,
                "avoidable_blocked_share_of_all_moves_no_barrier_reader"
            ),
            Some(seeded / 1_100.0)
        );
        assert_eq!(value(second, "moves_blocked_barrier_total"), Some(seeded));
        assert_eq!(
            value(second, "moves_blocked_occupied_total"),
            Some(seeded * 3.0)
        );
        assert_eq!(
            value(second, "moves_blocked_out_of_bounds_total"),
            Some(seeded * 5.0)
        );
        assert_eq!(
            value(&cases[0].0, "moves_blocked_occupied_total"),
            None,
            "a report that never carried the by-cause totals is unmeasured, not zero"
        );
        assert_eq!(
            value(second, "drift_changed_per_all_births_at_2000"),
            Some(seeded / 400.0),
            "the depth-2,000 checkpoint, not the depth-1,000 one"
        );
        assert!(
            case_readings(&report, "a world no report ran").is_empty(),
            "an unknown case has no readings at all"
        );
    }

    /// The world set runs exactly the seeds its recipes carry, in their order,
    /// so adding a world is one entry in `GOAL_RECIPES`. A profile that names
    /// other seeds, or the same seeds in another order, is a hard error rather
    /// than a world silently paired with another world's recipe or dropped.
    #[test]
    fn a_world_set_profile_must_name_the_seeds_its_recipes_carry() {
        assert_eq!(
            goal_profile_params().seeds,
            vec![11, 22, 33],
            "the checked-in case order the stored reports were recorded in"
        );

        // A test-sized world set, so a guard that fails to fire finishes and
        // fails an assertion instead of running the real 1600² profile.
        let mut reordered = small_world_set_params();
        reordered.seeds = vec![22, 11, 33];
        let error = run_deterministic(&reordered)
            .err()
            .expect("a reordered seed list pairs each world with another world's recipe");
        assert!(error.contains(GOAL_WORLD_SET), "{error}");
        assert!(error.contains("22, 11, 33"), "{error}");
        assert!(error.contains("11, 22, 33"), "{error}");

        let mut short = small_world_set_params();
        short.seeds.pop();
        assert!(
            run_deterministic(&short).is_err(),
            "a missing seed drops a world instead of running it"
        );

        let mut other = small_profile("sweep");
        other.seeds = vec![7, 9];
        assert!(
            run_deterministic(&other).is_ok(),
            "only the world set is tied to the checked-in recipes"
        );
    }

    /// `Undefined` is the report's no-denominator value and must not parse as
    /// a number a delta would then compare against.
    #[test]
    fn an_undefined_reading_parses_as_unmeasured() {
        assert_eq!(parse_reading(&six(0.25)), Some(0.25));
        assert_eq!(parse_reading(UNDEFINED), None);
        assert_eq!(parse_reading(""), None);
    }
}
