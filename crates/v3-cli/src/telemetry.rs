//! The process's run telemetry (T21.F01).
//!
//! `main` installs it once when `--telemetry` or `PETRI_TELEMETRY` resolves on;
//! `run_simulation` reads it, so its public signature does not depend on the
//! `telemetry` feature. Nothing installed means off.

use std::sync::OnceLock;

use v3_core::config::SimulationConfig;
use v3_telemetry::{EndStatus, Flush, RunHandle, RunStart, Telemetry};

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

/// Emits `run.ended` (`completed`) and waits for the bounded end-of-run flush.
pub(crate) fn end_run(run: Option<RunHandle>, tick: u64) {
    if let (Some(installed), Some(run)) = (INSTALLED.get(), run) {
        installed
            .telemetry
            .end_run(run, EndStatus::Completed, tick, Flush::Wait);
    }
}
