//! The server's run telemetry (T21.F01): one run from each seeding (the
//! initial seed-0 run and every `startup`), a `run.state` per status change, a
//! `run.config` per accepted patch, and `run.ended` at the next reset or at
//! process shutdown. Reset never waits for the previous run's export.
//!
//! Snapshots (T21.F02): an interval snapshot after each tick when due, one
//! at a transition when due, and one from the ending run's simulation at
//! reset and at shutdown, each taken before the record it accompanies.

use std::sync::{Arc, Mutex, MutexGuard};

use v3_core::config::SimulationConfig;
use v3_core::simulation::Simulation;
use v3_telemetry::{EndStatus, Flush, RunHandle, RunStart, RunState, Telemetry, Trigger};

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

    /// A `startup`: ends the current run as `reset` at `old`'s tick, after its
    /// run-end snapshot of `old`, without waiting for its export, and begins
    /// the run of `new`. A status that changes to idle is reported on the new
    /// run.
    pub(crate) fn reset(
        &self,
        old: &Simulation,
        old_status: SimulationStatus,
        new: &Simulation,
        seed: u64,
    ) {
        self.end(old, EndStatus::Reset);
        self.begin(&new.config, seed);
        if old_status != SimulationStatus::Idle {
            self.transition(SimulationStatus::Idle, new);
        }
    }

    /// Takes the current run's run-end snapshot of `sim` and emits
    /// `run.ended` at its tick without waiting for the export.
    fn end(&self, sim: &Simulation, status: EndStatus) {
        if let Some(mut run) = self.current().take() {
            self.telemetry.snapshot(&mut run, sim, Trigger::RunEnd);
            self.telemetry
                .end_run(run, status, sim.tick, Flush::Background);
        }
    }

    /// Takes an interval snapshot of `sim` after a tick when one is due.
    pub(crate) fn after_tick(&self, sim: &Simulation) {
        if let Some(run) = self.current().as_mut() {
            self.telemetry.snapshot(run, sim, Trigger::Interval);
        }
    }

    /// Emits `run.state` for a status change, after a transition snapshot of
    /// `sim` when one is due.
    pub(crate) fn transition(&self, status: SimulationStatus, sim: &Simulation) {
        if let Some(run) = self.current().as_mut() {
            self.telemetry.snapshot(run, sim, Trigger::Transition);
            self.telemetry.run_state(run, run_state(status), sim.tick);
        }
    }

    /// Emits `run.config` for an accepted config patch.
    pub(crate) fn config(&self, config: &SimulationConfig, tick: u64) {
        if let Some(run) = self.current().as_mut() {
            self.telemetry.run_config(run, config, tick);
        }
    }

    /// Ends the current run as `shutdown` after its run-end snapshot of
    /// `sim`; returns at once. [`ServerTelemetry::flush`] follows it.
    pub fn end_for_shutdown(&self, sim: &Simulation) {
        self.end(sim, EndStatus::Shutdown);
    }

    /// The bounded shutdown flush, which blocks for up to its flush bound.
    pub fn flush(&self) {
        self.telemetry.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use v3_core::simulation::{run_tick, seed_simulation};
    use v3_telemetry::testing::Receiver;
    use v3_telemetry::{Limits, Options, ReportSink, Service};

    use super::*;

    fn config() -> SimulationConfig {
        let mut config = SimulationConfig::default();
        config.world.width = 32;
        config.world.height = 32;
        config.population.initial_creatures = 16;
        config
    }

    fn ticks(sim: &mut Simulation, count: usize) {
        for _ in 0..count {
            run_tick(sim, &mut None);
        }
    }

    #[test]
    fn transitions_resets_and_shutdown_take_snapshots_by_the_cadence_rules() {
        let receiver = Receiver::start();
        let telemetry = ServerTelemetry::new(Telemetry::start_with(Options {
            service: Service::Server,
            endpoint: receiver.endpoint().to_owned(),
            limits: Limits::default(),
            reports: ReportSink::capture(),
            metrics_interval: Duration::from_secs(3_600),
        }));
        let mut first = seed_simulation(config(), 1);
        telemetry.begin(&first.config, 1);
        telemetry.transition(SimulationStatus::Running, &first);
        telemetry.transition(SimulationStatus::Paused, &first);
        ticks(&mut first, 2);
        std::thread::sleep(Duration::from_millis(15));
        telemetry.transition(SimulationStatus::Running, &first);
        ticks(&mut first, 1);
        telemetry.after_tick(&first);
        let mut second = seed_simulation(config(), 2);
        telemetry.reset(&first, SimulationStatus::Running, &second, 2);
        ticks(&mut second, 2);
        telemetry.end_for_shutdown(&second);
        telemetry.flush();

        let records = receiver.records();
        let started: Vec<&str> = records
            .iter()
            .filter(|record| record.event_name == "run.started")
            .map(|record| record.attribute("petri.run_id").unwrap())
            .collect();
        assert_eq!(started.len(), 2);
        let snapshots = receiver.snapshots();
        let ticks_of = |run: &str| -> Vec<u64> {
            snapshots
                .iter()
                .filter(|snapshot| snapshot.run_id() == Some(run))
                .map(|snapshot| snapshot.tick().unwrap())
                .collect()
        };
        let ended_tick = |run: &str| -> u64 {
            records
                .iter()
                .find(|record| {
                    record.event_name == "run.ended"
                        && record.attribute("petri.run_id") == Some(run)
                })
                .and_then(|record| record.attribute("petri.tick"))
                .unwrap()
                .parse()
                .unwrap()
        };
        // First run: one at `running`, none at `paused` in the same tick, one
        // at `running` two ticks and 15 ms later, none after the tick inside
        // the interval, and the run-end one at reset from the ending run.
        assert_eq!(ticks_of(started[0]), [0, 2, 3]);
        assert_eq!(ended_tick(started[0]), 3);
        // Second run: one at `idle` after the reset, and one at shutdown.
        assert_eq!(ticks_of(started[1]), [0, 2]);
        assert_eq!(ended_tick(started[1]), 2);
    }
}
