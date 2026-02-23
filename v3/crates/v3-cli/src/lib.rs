use serde::Serialize;
use v3_core::config::SimulationConfig;
use v3_core::simulation::{run_tick, seed_simulation};

pub const PROTOCOL_VERSION: &str = "v3alpha1";

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
    pub initial_population: usize,
}

#[derive(Debug, Serialize)]
pub struct TickSampleEvent {
    pub protocol_version: &'static str,
    pub event_type: &'static str,
    pub tick: u64,
    pub population: usize,
    pub mean_energy: f32,
    pub reproduction_actions_attempted_total: u64,
    pub reproduction_actions_spawned_total: u64,
    pub reproduction_actions_rejected_total: u64,
    pub mutation_events_attempted_total: u64,
    pub mutation_events_applied_total: u64,
    pub mutation_events_skipped_total: u64,
}

#[derive(Debug, Serialize)]
pub struct RunCompletedEvent {
    pub protocol_version: &'static str,
    pub event_type: &'static str,
    pub ticks_executed: u64,
    pub final_population: usize,
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

    let started = RunStartedEvent {
        protocol_version: PROTOCOL_VERSION,
        event_type: "run_started",
        seed,
        ticks_requested: ticks,
        initial_population: sim.creatures.len(),
    };
    emit(out, &started)?;

    let sample_every = sample_every as u64;
    let mut last_sampled_tick: Option<u64> = None;

    for _ in 0..ticks {
        run_tick(&mut sim);
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
    };
    emit(out, &completed)?;

    Ok(())
}

fn build_tick_sample(sim: &v3_core::simulation::Simulation, tick: u64) -> TickSampleEvent {
    TickSampleEvent {
        protocol_version: PROTOCOL_VERSION,
        event_type: "tick_sample",
        tick,
        population: sim.creatures.len(),
        mean_energy: sim.mean_energy(),
        reproduction_actions_attempted_total: sim.stats.reproduction_actions_attempted_total,
        reproduction_actions_spawned_total: sim.stats.reproduction_actions_spawned_total,
        reproduction_actions_rejected_total: sim.stats.reproduction_actions_rejected_total,
        mutation_events_attempted_total: sim.stats.mutation_events_attempted_total,
        mutation_events_applied_total: sim.stats.mutation_events_applied_total,
        mutation_events_skipped_total: sim.stats.mutation_events_skipped_total,
    }
}

fn emit<W: std::io::Write, T: Serialize>(out: &mut W, event: &T) -> Result<(), RunError> {
    let json = serde_json::to_string(event)
        .map_err(|e| RunError::RuntimeError(format!("serialization error: {e}")))?;
    writeln!(out, "{json}").map_err(|e| RunError::RuntimeError(format!("IO error: {e}")))
}
