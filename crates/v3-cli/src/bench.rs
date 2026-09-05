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
use v3_core::config::SimulationConfig;
use v3_core::creature::genome::analysis::functional_complexity;
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

// ── Profile parameters ──────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct ProfileParams {
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
        name: "gate".to_string(),
        width: 128,
        height: 128,
        founders: 256,
        seeds: vec![11, 22, 33],
        ticks: 75,
        food_coverage: Some(1.0),
    }
}

/// Predeclared minutes-scale goal profile constants (T01.F12).
pub fn goal_profile_params() -> ProfileParams {
    ProfileParams {
        name: "goal".to_string(),
        width: 1600,
        height: 1600,
        founders: 10_000,
        seeds: vec![11, 22, 33],
        ticks: 2_000,
        food_coverage: None,
    }
}

/// The `profile.food_coverage` report string for a profile that leaves
/// production food coverage untouched.
const DEFAULT_FOOD_COVERAGE: &str = "default";

pub fn build_config(params: &ProfileParams) -> SimulationConfig {
    let mut config = SimulationConfig::default();
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
    config
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deterministic {
    pub profile: ProfileBlock,
    pub per_seed: Vec<PerSeed>,
    pub totals: Totals,
    pub per_creature_tick: PerCreatureTick,
    pub goal_indicators: GoalIndicators,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileBlock {
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
pub struct GoalIndicators {
    pub population_persistence: PopulationPersistence,
    pub births_per_100_ticks: String,
    pub reachable_structure_size_distribution: StructureSizeDistribution,
    #[serde(default = "undefined_lineage_diversity")]
    pub lineage_diversity: Indicator<LineageDiversity>,
    #[serde(default = "undefined_memory_sensitivity")]
    pub memory_sensitivity: Indicator<MemorySensitivity>,
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

const UNDEFINED: &str = "Undefined";

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

/// One persistence sample, taken after `run_tick` on every executed tick that
/// is a multiple of [`SAMPLE_EVERY_TICKS`] and on the last executed tick.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceSample {
    pub tick: u64,
    pub population: u64,
    pub mean_energy: Option<String>,
    /// Cumulative `reproduction_actions_spawned_total` at this tick.
    pub births_total: u64,
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
    pub wall_clock: Option<WallClockComparison>,
    pub severe: bool,
}

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

fn six(x: f64) -> String {
    format!("{x:.6}")
}

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

    /// Record one executed tick. `mean_energy` is evaluated only on sampled
    /// ticks that still have creatures, keeping the `O(population)` energy sum
    /// to the predeclared cadence and leaving `null` at extinction.
    fn observe(
        &mut self,
        tick: u64,
        population: u64,
        births_total: u64,
        mean_energy: impl FnOnce() -> f64,
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
    complexities: Vec<u32>,
    goal_observation: Option<GoalObservation>,
    wall_clock_ms: f64,
    phase_wall_clock: SeedPhaseWallClock,
    throughput: SeedThroughput,
}

struct GoalObservation {
    lineage_diversity: LineageDiversitySeed,
    memory_sensitivity: MemorySensitivitySeed,
    wall_clock_ms: f64,
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
}

fn run_one_seed(
    config: &SimulationConfig,
    seed: u64,
    horizon: u64,
    observe_goal_indicators: bool,
) -> SeedRun {
    let start = Instant::now();
    let mut sim = seed_simulation(config.clone(), seed);
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
        );
        if population == 0 {
            break;
        }
    }
    let wall_clock_ms = millis(start.elapsed());

    let persistence = persistence.finish(seed);
    let complexities: Vec<u32> = sim
        .creatures
        .values()
        .map(|c| functional_complexity(&c.genome))
        .collect();
    let goal_observation = observe_goal_indicators.then(|| {
        let observation_started = Instant::now();
        let actions = observe_final_actions(&sim);
        GoalObservation {
            lineage_diversity: lineage_diversity(
                seed,
                sim.creatures
                    .values()
                    .map(|creature| creature.identity.lineage_id),
            ),
            memory_sensitivity: memory_sensitivity(seed, &actions),
            wall_clock_ms: millis(observation_started.elapsed()),
        }
    });

    let per_seed = PerSeed {
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
    let fraction = |count| {
        (final_creature_count > 0)
            .then(|| six(count as f64 / final_creature_count as f64))
            .unwrap_or_else(|| UNDEFINED.to_string())
    };
    MemorySensitivitySeed {
        seed,
        final_creature_count,
        different_from_zeroed_count,
        different_from_scrambled_count,
        different_from_either_count,
        different_from_zeroed_fraction: fraction(different_from_zeroed_count),
        different_from_scrambled_fraction: fraction(different_from_scrambled_count),
        different_from_either_fraction: fraction(different_from_either_count),
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

/// Run the deterministic profile (no host/timestamp data) and return the
/// `Deterministic` block plus the run's wall-clock observations for the
/// caller to fold into the `environment` block.
pub fn run_deterministic(params: &ProfileParams) -> (Deterministic, RunTimings) {
    let config = build_config(params);

    let mut per_seed = Vec::with_capacity(params.seeds.len());
    let mut wall_clock = Vec::with_capacity(params.seeds.len());
    let mut phase_wall_clock = Vec::with_capacity(params.seeds.len());
    let mut throughput_per_seed = Vec::with_capacity(params.seeds.len());
    let mut pooled_complexities: Vec<u32> = Vec::new();
    let mut totals = Totals::default();
    let mut population_persistence_per_seed = Vec::with_capacity(params.seeds.len());
    let mut lineage_diversity_per_seed = Vec::with_capacity(params.seeds.len());
    let mut memory_sensitivity_per_seed = Vec::with_capacity(params.seeds.len());
    let mut final_state_observation_ms_per_seed = Vec::with_capacity(params.seeds.len());
    let observe_goal_indicators = params.name == "goal";

    for &seed in &params.seeds {
        let run = run_one_seed(&config, seed, params.ticks, observe_goal_indicators);
        totals.ticks += run.per_seed.ticks;
        totals.creature_ticks += run.per_seed.creature_ticks;
        totals.mesh_hops += run.per_seed.mesh_hops;
        totals.vm_steps += run.per_seed.vm_steps;
        totals.graph_relax_iters += run.per_seed.graph_relax_iters;
        totals.plasticity_updates += run.per_seed.plasticity_updates;
        totals.actions_applied += run.per_seed.actions_applied;
        totals.births += run.per_seed.births;
        pooled_complexities.extend(run.complexities.iter().copied());
        wall_clock.push(SeedWallClock {
            seed,
            wall_clock_ms: run.wall_clock_ms,
        });
        throughput_per_seed.push(run.throughput);
        phase_wall_clock.push(run.phase_wall_clock);
        population_persistence_per_seed.push(run.persistence);
        if let Some(observation) = run.goal_observation {
            lineage_diversity_per_seed.push(observation.lineage_diversity);
            memory_sensitivity_per_seed.push(observation.memory_sensitivity);
            final_state_observation_ms_per_seed.push(SeedFinalStateObservation {
                seed,
                wall_clock_ms: observation.wall_clock_ms,
            });
        }
        per_seed.push(run.per_seed);
    }

    let per_creature_tick = PerCreatureTick {
        mesh_hops: Some(ratio(totals.mesh_hops, totals.creature_ticks)),
        vm_steps: Some(ratio(totals.vm_steps, totals.creature_ticks)),
        graph_relax_iters: Some(ratio(totals.graph_relax_iters, totals.creature_ticks)),
        plasticity_updates: Some(ratio(totals.plasticity_updates, totals.creature_ticks)),
        actions_applied: Some(ratio(totals.actions_applied, totals.creature_ticks)),
        births: Some(ratio(totals.births, totals.creature_ticks)),
    };

    let population_persistence = PopulationPersistence {
        per_seed: population_persistence_per_seed,
    };

    let births_per_100_ticks = if totals.ticks == 0 {
        six(0.0)
    } else {
        six(totals.births as f64 / totals.ticks as f64 * 100.0)
    };

    let goal_indicators = GoalIndicators {
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
        strategy_count: UNDEFINED.to_string(),
        strategy_causal_distinctness: UNDEFINED.to_string(),
        evolutionary_activity: UNDEFINED.to_string(),
        adaptive_novelty: UNDEFINED.to_string(),
        memory_dependence: UNDEFINED.to_string(),
        learning_dependence: UNDEFINED.to_string(),
        prediction_dependence: UNDEFINED.to_string(),
        information_integration: UNDEFINED.to_string(),
        reciprocal_interaction: UNDEFINED.to_string(),
    };

    let deterministic = Deterministic {
        profile: ProfileBlock {
            name: params.name.clone(),
            world_width: params.width,
            world_height: params.height,
            founders: params.founders,
            seeds: params.seeds.clone(),
            ticks: params.ticks,
            food_coverage: params.food_coverage.map_or_else(
                || DEFAULT_FOOD_COVERAGE.to_string(),
                |coverage| six(f64::from(coverage)),
            ),
        },
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
    };

    (deterministic, timings)
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
    Environment {
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
    }
}

// ── Report assembly ─────────────────────────────────────────────────────────

/// Build a full report on the rayon global thread pool.
pub fn build_report(params: &ProfileParams, feature: &str) -> Report {
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
) -> Report {
    let run = || (run_deterministic(params), rayon::current_num_threads());
    let ((deterministic, timings), threads_used) = match threads {
        Some(threads) => rayon::ThreadPoolBuilder::new()
            .num_threads(threads.get())
            .build()
            .expect("a rayon pool of at least one thread can always be built")
            .install(run),
        None => run(),
    };
    let environment = build_environment(timings, &deterministic.totals, threads_used);
    Report {
        schema_version: SCHEMA_VERSION,
        feature: feature.to_string(),
        deterministic,
        environment,
        comparison: Comparison::default(),
    }
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
        wall_clock,
        severe: any_severe,
    }
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
    if reference.deterministic.profile != current.deterministic.profile {
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
    let baseline = PathBuf::from(&index.goal.epoch_baseline);
    if !baseline.exists() {
        return Ok(Vec::new());
    }
    references_from_series(&index.goal)
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

// ── Persistence accumulator unit tests ──────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use v3_core::contracts::{Direction, WorldAction};
    use v3_core::simulation::seed_simulation;

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
            accumulator.observe(tick, population, tick, || energy);
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
            name: "sweep".to_string(),
            width: 32,
            height: 32,
            founders: 8,
            seeds: vec![1],
            ticks: 10,
            food_coverage: None,
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
        assert!(
            coverages.iter().all(|c| (c - 0.27).abs() < 1e-6),
            "production coverage is 0.27 for every default food type, got {coverages:?}"
        );
        assert!(
            (config.world.food.shared.initial_coverage - 0.27).abs() < 1e-6,
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
}
