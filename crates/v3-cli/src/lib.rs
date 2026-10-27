use std::collections::HashMap;

use serde::Serialize;
use v3_core::config::SimulationConfig;
use v3_core::simulation::{run_tick, seed_simulation};

pub const PROTOCOL_VERSION: &str = "v3alpha1";

// ── Report formatting ────────────────────────────────────────────────────────

/// The report vocabulary's value for a reading whose denominator is zero:
/// unmeasurable, and deliberately not the zero a delta would compare against.
pub(crate) const UNDEFINED: &str = "Undefined";

/// Every fractional reading in a report and in `world inspect` is six decimals.
pub(crate) fn six(x: f64) -> String {
    format!("{x:.6}")
}

/// A six-decimal fraction against `denominator`, or [`UNDEFINED`] when the
/// denominator is zero.
pub(crate) fn fraction_or_undefined(count: u64, denominator: u64) -> String {
    mean_or_undefined(count as f64, denominator)
}

/// [`fraction_or_undefined`] for a sum that is already a float — a mean over
/// `count` observations rather than a count against a total.
pub(crate) fn mean_or_undefined(sum: f64, count: u64) -> String {
    if count == 0 {
        UNDEFINED.to_string()
    } else {
        six(sum / count as f64)
    }
}

/// Error type for CLI simulation runs.
#[derive(Debug)]
pub enum RunError {
    ValidationError(String),
    RuntimeError(String),
}

impl std::fmt::Display for RunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RunError::ValidationError(msg) => write!(f, "validation error: {msg}"),
            RunError::RuntimeError(msg) => write!(f, "runtime error: {msg}"),
        }
    }
}

// ── Event types ──────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct RunStartedEvent {
    pub protocol_version: &'static str,
    pub event_type: &'static str,
    pub seed: u64,
    pub ticks_requested: u64,
    pub sample_every: u16,
    pub config_digest: String,
}

#[derive(Debug, Serialize)]
pub struct TickSampleEvent {
    pub protocol_version: &'static str,
    pub event_type: &'static str,
    pub tick: u64,
    pub population: usize,
    pub mean_energy: f32,
    /// Mean total genome size (junk included) over the living population.
    pub mean_genome_size: f64,
    /// Mean mesh node count over the living population.
    pub mean_mesh_nodes: f64,
    /// Mean lineage depth over the living population.
    pub mean_generation: f64,
    pub reproduction_actions_attempted_total: u64,
    pub reproduction_actions_spawned_total: u64,
    pub reproduction_actions_rejected_total: u64,
    pub mutation_events_attempted_total: u64,
    pub mutation_events_applied_total: u64,
    pub mutation_events_skipped_total: u64,
    pub mutation_events_attempted_total_by_domain: HashMap<String, u64>,
    pub mutation_events_applied_total_by_domain: HashMap<String, u64>,
    pub mutation_events_attempted_total_by_operator: HashMap<String, u64>,
    pub mutation_events_applied_total_by_operator: HashMap<String, u64>,
}

#[derive(Debug, Serialize)]
pub struct RunCompletedEvent {
    pub protocol_version: &'static str,
    pub event_type: &'static str,
    pub ticks_executed: u64,
    pub final_population: usize,
    pub final_mean_energy: f32,
}

// ── run_simulation ────────────────────────────────────────────────────────────

/// Run a simulation and emit NDJSON events to `out`.
///
/// Events emitted (in order):
/// 1. `run_started` — once at the start.
/// 2. `tick_sample` — after every tick where `tick % sample_every == 0`, and
///    always after the final tick (even if already emitted for that tick).
/// 3. `run_completed` — once at the end.
pub fn run_simulation<W: std::io::Write>(
    config: SimulationConfig,
    seed: u64,
    ticks: u64,
    sample_every: u16,
    out: &mut W,
) -> Result<(), RunError> {
    let mut sim = seed_simulation(config, seed);
    #[cfg(feature = "telemetry")]
    let mut telemetry_run = telemetry::begin_run(&sim.config, seed, ticks, sample_every);

    let started = RunStartedEvent {
        protocol_version: PROTOCOL_VERSION,
        event_type: "run_started",
        seed,
        ticks_requested: ticks,
        sample_every,
        config_digest: v3_core::config::config_digest(&sim.config),
    };
    emit(out, &started)?;

    let sample_every = sample_every as u64;
    let mut last_sampled_tick: Option<u64> = None;

    // The creature window slot (T21.F04); telemetry off leaves it empty.
    let mut window: Option<v3_core::simulation::recording::ActiveTrace> = None;
    for _ in 0..ticks {
        #[cfg(feature = "telemetry")]
        telemetry::before_tick(telemetry_run.as_mut(), &sim, &mut window);
        run_tick(&mut sim, &mut window);
        #[cfg(feature = "telemetry")]
        telemetry::after_tick(telemetry_run.as_mut(), &sim, &mut window);
        let current_tick = sim.tick;

        let should_sample = current_tick.is_multiple_of(sample_every) || current_tick == ticks;

        if should_sample && last_sampled_tick != Some(current_tick) {
            let sample = build_tick_sample(&sim, current_tick);
            emit(out, &sample)?;
            last_sampled_tick = Some(current_tick);
        }
    }

    // Ensure final tick is always sampled.
    if last_sampled_tick != Some(ticks) {
        let sample = build_tick_sample(&sim, sim.tick);
        emit(out, &sample)?;
    }

    let completed = RunCompletedEvent {
        protocol_version: PROTOCOL_VERSION,
        event_type: "run_completed",
        ticks_executed: ticks,
        final_population: sim.creatures.len(),
        final_mean_energy: sim.mean_energy(),
    };
    emit(out, &completed)?;
    #[cfg(feature = "telemetry")]
    telemetry::end_run(telemetry_run, &sim, &mut window);

    Ok(())
}

/// Means of the living population's structure and lineage depth: total
/// genome size (junk included), mesh node count, generation, wired output
/// sinks (sinks with at least one edge). All `0.0` for
/// an empty population, like `Simulation::mean_energy`.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct StructureMeans {
    pub(crate) genome_size: f64,
    pub(crate) mesh_nodes: f64,
    pub(crate) generation: f64,
    /// T25.F02: the sinks the unit rule stopped charging a unit for.
    pub(crate) wired_sinks: f64,
}

pub(crate) fn structure_means(sim: &v3_core::simulation::Simulation) -> StructureMeans {
    use v3_core::genome::BackendDef;

    let population = sim.creatures.len();
    if population == 0 {
        return StructureMeans::default();
    }
    let [mut genome_size, mut mesh_nodes, mut generation, mut wired_sinks] = [0u64; 4];
    for creature in sim.creatures.values() {
        genome_size += u64::from(creature.genome_size());
        mesh_nodes += creature.genome().nodes.len() as u64;
        generation += creature.generation;
        for node in &creature.genome().nodes {
            if let BackendDef::Graph(graph) = &node.backend_def {
                wired_sinks += graph
                    .output_sinks()
                    .iter()
                    .filter(|sink| !sink.inputs.is_empty())
                    .count() as u64;
            }
        }
    }
    let mean = |total: u64| total as f64 / population as f64;
    StructureMeans {
        genome_size: mean(genome_size),
        mesh_nodes: mean(mesh_nodes),
        generation: mean(generation),
        wired_sinks: mean(wired_sinks),
    }
}

fn build_tick_sample(sim: &v3_core::simulation::Simulation, tick: u64) -> TickSampleEvent {
    let means = structure_means(sim);
    TickSampleEvent {
        protocol_version: PROTOCOL_VERSION,
        event_type: "tick_sample",
        tick,
        population: sim.creatures.len(),
        mean_energy: sim.mean_energy(),
        mean_genome_size: means.genome_size,
        mean_mesh_nodes: means.mesh_nodes,
        mean_generation: means.generation,
        reproduction_actions_attempted_total: sim.stats.reproduction_actions_attempted_total,
        reproduction_actions_spawned_total: sim.stats.reproduction_actions_spawned_total,
        reproduction_actions_rejected_total: sim.stats.reproduction_actions_rejected_total,
        mutation_events_attempted_total: sim.stats.mutation_events_attempted_total,
        mutation_events_applied_total: sim.stats.mutation_events_applied_total,
        mutation_events_skipped_total: sim.stats.mutation_events_skipped_total,
        mutation_events_attempted_total_by_domain: sim
            .stats
            .mutation_events_attempted_total_by_domain
            .iter()
            .map(|(domain, count)| (domain.as_key().to_string(), *count))
            .collect(),
        mutation_events_applied_total_by_domain: sim
            .stats
            .mutation_events_applied_total_by_domain
            .iter()
            .map(|(domain, count)| (domain.as_key().to_string(), *count))
            .collect(),
        mutation_events_attempted_total_by_operator: sim
            .stats
            .mutation_events_attempted_total_by_operator
            .iter()
            .map(|(operator, count)| (operator.as_key().to_string(), *count))
            .collect(),
        mutation_events_applied_total_by_operator: sim
            .stats
            .mutation_events_applied_total_by_operator
            .iter()
            .map(|(operator, count)| (operator.as_key().to_string(), *count))
            .collect(),
    }
}

fn emit<W: std::io::Write, T: Serialize>(out: &mut W, event: &T) -> Result<(), RunError> {
    let json = serde_json::to_string(event)
        .map_err(|e| RunError::RuntimeError(format!("serialization error: {e}")))?;
    writeln!(out, "{json}").map_err(|e| RunError::RuntimeError(format!("IO error: {e}")))
}

pub mod bench;
pub mod inspect;
pub mod opportunity;
pub mod profile;
pub mod recruitment;
#[cfg(feature = "telemetry")]
pub mod telemetry;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn structure_means_are_zero_for_an_empty_population() {
        let mut cfg = SimulationConfig::default();
        cfg.world.width = 32;
        cfg.world.height = 32;
        cfg.population.initial_creatures = 0;
        let sim = seed_simulation(cfg, 42);
        assert_eq!(sim.creatures.len(), 0);
        assert_eq!(
            structure_means(&sim),
            StructureMeans {
                genome_size: 0.0,
                mesh_nodes: 0.0,
                generation: 0.0,
                wired_sinks: 0.0,
            }
        );
    }

    #[test]
    fn structure_means_average_the_living_population() {
        let mut cfg = SimulationConfig::default();
        cfg.world.width = 32;
        cfg.world.height = 32;
        cfg.population.initial_creatures = 4;
        let sim = seed_simulation(cfg, 42);
        assert_eq!(sim.creatures.len(), 4);

        let expected_genome_size =
            f64::from(sim.creatures.values().map(|c| c.genome_size()).sum::<u32>()) / 4.0;
        assert_eq!(
            structure_means(&sim),
            StructureMeans {
                genome_size: expected_genome_size,
                mesh_nodes: 2.0,
                generation: 0.0,
                wired_sinks: 17.0,
            },
            "founders carry two mesh nodes, generation zero and 6 + 11 wired sinks"
        );
    }
}
