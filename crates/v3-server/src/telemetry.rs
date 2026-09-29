//! The server's run telemetry (T21.F01): one run from each seeding (the
//! initial seed-0 run and every `startup`), a `run.state` per status change, a
//! `run.config` per accepted patch, and `run.ended` at the next reset or at
//! process shutdown. Reset never waits for the previous run's export.

use std::sync::{Arc, Mutex, MutexGuard};

use v3_core::config::SimulationConfig;
use v3_telemetry::{EndStatus, Flush, RunHandle, RunStart, RunState, Telemetry};

use crate::state::SimulationStatus;

/// The process's telemetry and its current run; off by default.
#[derive(Clone, Debug, Default)]
pub struct ServerTelemetry {
    telemetry: Telemetry,
    current: Arc<Mutex<Option<RunHandle>>>,
}

fn run_state(status: SimulationStatus) -> RunState {
    match status {
        SimulationStatus::Idle => RunState::Idle,
        SimulationStatus::Running => RunState::Running,
        SimulationStatus::Paused => RunState::Paused,
    }
}

impl ServerTelemetry {
    pub fn new(telemetry: Telemetry) -> Self {
        Self {
            telemetry,
            current: Arc::default(),
        }
    }

    fn current(&self) -> MutexGuard<'_, Option<RunHandle>> {
        self.current
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Emits `run.started` for a freshly seeded run and makes it current.
    pub(crate) fn begin(&self, config: &SimulationConfig, seed: u64) {
        *self.current() = self.telemetry.begin_run(RunStart {
            seed,
            config,
            recipe: None,
            tick: 0,
            ticks_requested: None,
            sample_every: None,
        });
    }

    /// A `startup`: ends the current run as `reset` at `old_tick` without
    /// waiting for its export, and begins the new one. A status that changes
    /// to idle is reported on the new run.
    pub(crate) fn reset(
        &self,
        old_tick: u64,
        old_status: SimulationStatus,
        config: &SimulationConfig,
        seed: u64,
    ) {
        if let Some(run) = self.current().take() {
            self.telemetry
                .end_run(run, EndStatus::Reset, old_tick, Flush::Background);
        }
        self.begin(config, seed);
        if old_status != SimulationStatus::Idle {
            self.transition(SimulationStatus::Idle, 0);
        }
    }

    /// Emits `run.state` for a status change.
    pub(crate) fn transition(&self, status: SimulationStatus, tick: u64) {
        if let Some(run) = self.current().as_ref() {
            self.telemetry.run_state(run, run_state(status), tick);
        }
    }

    /// Emits `run.config` for an accepted config patch.
    pub(crate) fn config(&self, config: &SimulationConfig, tick: u64) {
        if let Some(run) = self.current().as_mut() {
            self.telemetry.run_config(run, config, tick);
        }
    }

    /// Ends the current run as `shutdown` and runs the bounded shutdown flush,
    /// which blocks for up to its flush bound.
    pub fn shutdown(&self, tick: u64) {
        if let Some(run) = self.current().take() {
            self.telemetry
                .end_run(run, EndStatus::Shutdown, tick, Flush::Background);
        }
        self.telemetry.shutdown();
    }
}
