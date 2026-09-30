//! The process's run telemetry (T21.F01), with the run's snapshots (T21.F02):
//! interval snapshots after ticks and one at completion, each with the trace
//! of the tick that just ran (T21.F03), and creature windows recorded through
//! the run's window slot (T21.F04).
//!
//! `main` installs it once when `--telemetry` or `PETRI_TELEMETRY` resolves on;
//! `run_simulation` reads it, so its public signature does not depend on the
//! `telemetry` feature. Nothing installed means off.

use std::sync::OnceLock;

use v3_core::config::SimulationConfig;
use v3_core::runtime::trace::recording::ActiveTrace;
use v3_core::simulation::Simulation;
use v3_telemetry::{EndStatus, Flush, RunHandle, RunStart, Telemetry, TickSample};

struct Installed {
    telemetry: Telemetry,
    recipe: Option<String>,
}

static INSTALLED: OnceLock<Installed> = OnceLock::new();

/// Installs the process's telemetry and the `--config` path its runs name.
/// Only the first call takes effect.
pub fn install(telemetry: Telemetry, recipe: Option<String>) {
    let _ = INSTALLED.set(Installed { telemetry, recipe });
}

/// Emits `run.started` for a freshly seeded run when telemetry is on.
pub(crate) fn begin_run(
    config: &SimulationConfig,
    seed: u64,
    ticks: u64,
    sample_every: u16,
) -> Option<RunHandle> {
    let installed = INSTALLED.get()?;
    installed.telemetry.begin_run(RunStart {
        seed,
        config,
        recipe: installed.recipe.as_deref(),
        tick: 0,
        ticks_requested: Some(ticks),
        sample_every: Some(u64::from(sample_every)),
    })
}

/// Starts a creature window into `window` before a tick when one is due.
pub(crate) fn before_tick(
    run: Option<&mut RunHandle>,
    sim: &Simulation,
    window: &mut Option<ActiveTrace>,
) {
    if let (Some(installed), Some(run)) = (INSTALLED.get(), run) {
        installed.telemetry.before_tick(run, sim, window, false);
    }
}

/// Takes an interval snapshot and its tick trace after a tick when one is
/// due, and exports a creature window that ended.
pub(crate) fn after_tick(
    run: Option<&mut RunHandle>,
    sim: &Simulation,
    window: &mut Option<ActiveTrace>,
) {
    if let (Some(installed), Some(run)) = (INSTALLED.get(), run) {
        installed.telemetry.after_tick(run, sim, window, None);
        installed
            .telemetry
            .tick_snapshot(run, sim, TickSample::Interval);
    }
}

/// Exports an open creature window as `run_end`, takes the run-end snapshot
/// and the last tick's trace unless the final tick has a snapshot, emits
/// `run.ended` (`completed`) and waits for the bounded end-of-run flush.
pub(crate) fn end_run(run: Option<RunHandle>, sim: &Simulation, window: &mut Option<ActiveTrace>) {
    if let (Some(installed), Some(mut run)) = (INSTALLED.get(), run) {
        installed.telemetry.end_samples(&mut run, sim, window, None);
        installed
            .telemetry
            .tick_snapshot(&mut run, sim, TickSample::RunEnd);
        installed
            .telemetry
            .end_run(run, EndStatus::Completed, sim.tick, Flush::Wait);
    }
}
