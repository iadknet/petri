//! Deterministic benchmark harness (T10.F10).
//!
//! Runs a fixed set of seeded simulations and emits one compact JSON report
//! per feature. The report's `deterministic` block is byte-identical for the
//! same commit and inputs; the `environment` block records host identity and
//! wall-clock as an unasserted secondary signal.

use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use v3_core::config::SimulationConfig;
use v3_core::creature::genome::analysis::functional_complexity;
use v3_core::simulation::{run_tick, seed_simulation};

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

/// Regression thresholds against each reference, per the T10.F10 spec.
const FLAG_PERCENT: f64 = 10.0;
const SEVERE_PERCENT: f64 = 50.0;

// ── Profile parameters ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ProfileParams {
    pub name: String,
    pub width: u16,
    pub height: u16,
    pub founders: u32,
    pub seeds: Vec<u64>,
    pub ticks: u64,
    pub food_coverage: f32,
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
        food_coverage: 1.0,
    }
}

pub fn build_config(params: &ProfileParams) -> SimulationConfig {
    let mut config = SimulationConfig::default();
    config.world.width = params.width;
    config.world.height = params.height;
    config.population.initial_creatures = params.founders;
    config.world.food.shared.initial_coverage = params.food_coverage;
    for food_type in &mut config.world.food.types {
        food_type.initial_coverage = params.food_coverage;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallClockComparison {
    pub current_ms_per_creature_tick: f64,
    pub reference_ms_per_creature_tick: f64,
    pub percent_delta: f64,
    pub level: String,
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

// ── Seed execution ───────────────────────────────────────────────────────────

struct SeedRun {
    per_seed: PerSeed,
    minimum_population: u64,
    complexities: Vec<u32>,
    wall_clock_ms: f64,
}

fn run_one_seed(config: &SimulationConfig, seed: u64, horizon: u64) -> SeedRun {
    let start = Instant::now();
    let mut sim = seed_simulation(config.clone(), seed);
    let mut minimum_population = sim.creatures.len() as u64;
    let mut extinction_tick: Option<u64> = None;

    let mut ticks_executed: u64 = 0;
    for _ in 0..horizon {
        run_tick(&mut sim, &mut None);
        ticks_executed += 1;
        let population = sim.creatures.len() as u64;
        if population < minimum_population {
            minimum_population = population;
        }
        if population == 0 {
            extinction_tick = Some(sim.tick);
            break;
        }
    }
    let wall_clock_ms = start.elapsed().as_secs_f64() * 1000.0;

    let final_population = sim.creatures.len() as u64;
    let complexities: Vec<u32> = sim
        .creatures
        .values()
        .map(|c| functional_complexity(&c.genome))
        .collect();

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
        final_population,
        extinction_tick,
    };

    SeedRun {
        per_seed,
        minimum_population,
        complexities,
        wall_clock_ms,
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
/// `Deterministic` block plus per-seed wall-clock timings for the caller to
/// fold into the `environment` block.
pub fn run_deterministic(params: &ProfileParams) -> (Deterministic, Vec<SeedWallClock>) {
    let config = build_config(params);

    let mut per_seed = Vec::with_capacity(params.seeds.len());
    let mut wall_clock = Vec::with_capacity(params.seeds.len());
    let mut pooled_complexities: Vec<u32> = Vec::new();
    let mut totals = Totals::default();
    let mut population_persistence_per_seed = Vec::with_capacity(params.seeds.len());

    for &seed in &params.seeds {
        let run = run_one_seed(&config, seed, params.ticks);
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
        population_persistence_per_seed.push(PopulationPersistenceSeed {
            seed,
            extinction_tick: run.per_seed.extinction_tick,
            minimum_population: run.minimum_population,
            final_population: run.per_seed.final_population,
        });
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
            food_coverage: six(f64::from(params.food_coverage)),
        },
        per_seed,
        totals,
        per_creature_tick,
        goal_indicators,
    };

    (deterministic, wall_clock)
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

fn build_environment(
    wall_clock_ms_per_seed: Vec<SeedWallClock>,
    creature_ticks: u64,
) -> Environment {
    let wall_clock_ms_total: f64 = wall_clock_ms_per_seed.iter().map(|s| s.wall_clock_ms).sum();
    let wall_clock_ms_per_creature_tick = if creature_ticks == 0 {
        0.0
    } else {
        wall_clock_ms_total / creature_ticks as f64
    };
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
    }
}

// ── Report assembly ─────────────────────────────────────────────────────────

/// Build a full report (deterministic block, environment, and an empty
/// comparison) for the given profile.
pub fn build_report(params: &ProfileParams, feature: &str) -> Report {
    let (deterministic, wall_clock) = run_deterministic(params);
    let environment = build_environment(wall_clock, deterministic.totals.creature_ticks);
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

fn counter_level(delta_percent: Option<f64>) -> &'static str {
    match delta_percent {
        None => "ok",
        Some(d) if d > SEVERE_PERCENT => "severe",
        Some(d) if d > FLAG_PERCENT => "flag",
        _ => "ok",
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
            None => ("new".to_string(), None, None),
            Some(reference_value) => {
                let delta = percent_delta(current_value, reference_value);
                let level = counter_level(delta);
                if level == "severe" {
                    any_severe = true;
                }
                (
                    level.to_string(),
                    Some(six(reference_value)),
                    delta.map(six),
                )
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
        let level = if delta > 100.0 {
            "severe"
        } else if delta > 25.0 {
            "flag"
        } else {
            "ok"
        };
        Some(WallClockComparison {
            current_ms_per_creature_tick: current_ms,
            reference_ms_per_creature_tick: reference_ms,
            percent_delta: delta,
            level: level.to_string(),
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
    let index: SeriesIndex = serde_json::from_str(&content)
        .map_err(|e| format!("failed to parse {}: {e}", series_index_path.display()))?;

    let mut paths = vec![PathBuf::from(&index.epoch_baseline)];
    if let Some(last) = index.closed.last() {
        if last != &index.epoch_baseline {
            paths.push(PathBuf::from(last));
        }
    }
    Ok(paths)
}
