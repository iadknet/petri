//! Seed execution, environment detection, and report assembly.

use super::comparison::{locked_rand_version, Comparison};
use super::indicators::{
    assemble_goal_indicators, build_mutational_neighborhood_indicator,
    compute_founder_neighborhood, evolved_neighborhood_for_seed, lineage_diversity,
    memory_sensitivity, structural_companions_census, structure_size_distribution,
    temporal_memory_sensitivity, timed_drift_depth, GoalIndicatorInputs,
};
use super::profiles::{
    build_config, goal_case, goal_recipes_for, profile_block, GoalCase, GoalRecipe,
    NeighborhoodSizes, ProfileParams, GOAL_WORLD_SET, SCHEMA_VERSION,
};
use super::schema::{
    ratio, timed_recruitment_paths, Deterministic, DriftDepth, Environment, GoalCaseObservation,
    Host, Indicator, LineageDiversitySeed, MemorySensitivitySeed, NeighborhoodEvolvedSeed,
    NeighborhoodFounderHalf, PerCreatureTick, PerSeed, PopulationPersistenceSeed, Report,
    SeedFinalStateObservation, SeedPhaseWallClock, SeedThroughput, SeedWallClock,
    StructuralCompanionsSeed, TemporalMemorySensitivitySeed, Throughput, ThroughputRates, Totals,
};
use super::tracking::{PersistenceAccumulator, PopulationReadings, WorldTracking};
use std::num::NonZeroUsize;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use v3_core::config::SimulationConfig;
use v3_core::creature::genome::analysis::functional_complexity;
use v3_core::neighborhood::{Battery, EvalContext};
use v3_core::simulation::{observe_final_actions, run_tick, seed_simulation};

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

pub(super) struct SeedRun {
    per_seed: PerSeed,
    pub(super) persistence: PopulationPersistenceSeed,
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
    structural_companions: StructuralCompanionsSeed,
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
pub(super) fn millis(duration: std::time::Duration) -> f64 {
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
    pub recruitment_paths_wall_clock_ms: Option<f64>,
    /// Evolved-half neighborhood wall time per seed (goal profile only).
    pub neighborhood_evolved_wall_clock_ms_per_seed: Vec<SeedFinalStateObservation>,
}

pub(super) fn run_one_seed(
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
            || PopulationReadings::observe(&sim),
            || WorldTracking::observe(&sim),
        );
        if population == 0 {
            break;
        }
    }
    let wall_clock_ms = millis(start.elapsed().saturating_sub(connectivity_duration));

    let tracking = WorldTracking::observe(&sim).with_transferred_counters(&sim);
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
        let structural_companions = structural_companions_census(
            seed,
            sim.creatures.values().map(|creature| &creature.genome),
        );
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
            structural_companions,
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
    let mut structural_companions_per_seed = Vec::with_capacity(params.seeds.len());
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
            structural_companions_per_seed.push(observation.structural_companions);
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

    let mut goal_indicators = assemble_goal_indicators(
        params,
        &totals,
        GoalIndicatorInputs {
            population_persistence_per_seed,
            lineage_diversity_per_seed,
            memory_sensitivity_per_seed,
            structural_companions_per_seed,
            temporal_memory_sensitivity_per_seed,
            evolved_neighborhood_per_seed,
            pooled_complexities,
            neighborhood_founder,
            drift_depth,
            case_observations,
        },
    );

    let (recruitment_paths, recruitment_paths_wall_clock_ms) =
        timed_recruitment_paths(observe_goal_indicators);
    goal_indicators.recruitment_paths = recruitment_paths;

    let deterministic = Deterministic {
        graph_work_definition: "graph_relax_iters: entered single-evaluation visits, entered when the graph has a compute node or a wired effect surface (T13.F04), including unaffordable visits (T11.F06); historical deltas cross definitions".to_string(),
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
        recruitment_paths_wall_clock_ms,
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
pub fn rfc3339_now() -> String {
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
        recruitment_paths_wall_clock_ms,
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
        recruitment_paths_wall_clock_ms,
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

#[cfg(test)]
mod tests;
