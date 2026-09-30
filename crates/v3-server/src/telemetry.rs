//! The server's run telemetry (T21.F01): one run from each seeding (the
//! initial seed-0 run and every `startup`), a `run.state` per status change, a
//! `run.config` per accepted patch, and `run.ended` at the next reset or at
//! process shutdown. Reset never waits for the previous run's export.
//!
//! Snapshots (T21.F02): an interval snapshot after each tick when due, one
//! at a transition when due, and one from the ending run's simulation at
//! reset and at shutdown, each taken before the record it accompanies. Only
//! the interval snapshot carries a tick trace (T21.F03): a config patch
//! accepted while paused may postdate the tick the others would trace.
//!
//! Creature windows (T21.F04) record into the handle's window slot while no
//! manual sample is active; the manual Execution Sampler exports its sample
//! at hand-over, or when a new sample replaces it or the run ends.

use std::sync::{Arc, Mutex, MutexGuard};

use v3_core::config::SimulationConfig;
use v3_core::runtime::trace::recording::ActiveTrace;
use v3_core::simulation::Simulation;
use v3_telemetry::{
    EndStatus, Flush, RunHandle, RunStart, RunState, Telemetry, TickSample, Trigger,
};

use crate::state::SimulationStatus;

/// A handle's two recording slots: the window slot and the manual sampler's.
pub struct Samples<'a> {
    pub window: &'a mut Option<ActiveTrace>,
    pub manual: Option<&'a ActiveTrace>,
}

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

    /// A `startup`: exports the ending run's open window and unfetched
    /// manual sample as `run_end`, ends the run as `reset` at `old`'s tick,
    /// after its run-end snapshot of `old`, without waiting for its export,
    /// and begins the run of `new`. A status that changes to idle is
    /// reported on the new run.
    pub(crate) fn reset(
        &self,
        old: &Simulation,
        old_status: SimulationStatus,
        new: &Simulation,
        seed: u64,
        samples: Samples<'_>,
    ) {
        self.end_samples(old, samples);
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

    /// Starts a creature window into `window` when one is due and no manual
    /// sample is active.
    pub(crate) fn before_tick(
        &self,
        sim: &Simulation,
        window: &mut Option<ActiveTrace>,
        manual_active: bool,
    ) {
        if let Some(run) = self.current().as_mut() {
            self.telemetry.before_tick(run, sim, window, manual_active);
        }
    }

    /// Exports a window that ended, then takes an interval snapshot of `sim`
    /// and its tick trace after a tick when one is due.
    pub(crate) fn after_tick(
        &self,
        sim: &Simulation,
        window: &mut Option<ActiveTrace>,
        manual: Option<&ActiveTrace>,
    ) {
        if let Some(run) = self.current().as_mut() {
            self.telemetry.after_tick(run, sim, window, manual);
            self.telemetry.tick_snapshot(run, sim, TickSample::Interval);
        }
    }

    /// `start_sample`: ends an open window as `manual`, exports the unfetched
    /// sample `replaced` as `replaced`, and admits `sample` for export.
    pub(crate) fn start_sample(
        &self,
        sim: &Simulation,
        window: &mut Option<ActiveTrace>,
        replaced: Option<&ActiveTrace>,
        sample: &ActiveTrace,
    ) {
        if let Some(run) = self.current().as_mut() {
            self.telemetry
                .start_manual(run, sim, window, replaced, sample);
        }
    }

    /// `get_sample` hands a completed sample over: it exports once.
    pub(crate) fn hand_over(&self, sim: &Simulation, sample: &ActiveTrace) {
        if let Some(run) = self.current().as_mut() {
            self.telemetry.hand_over(run, sim, sample);
        }
    }

    fn end_samples(&self, sim: &Simulation, samples: Samples<'_>) {
        if let Some(run) = self.current().as_mut() {
            self.telemetry
                .end_samples(run, sim, samples.window, samples.manual);
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

    /// An accepted config patch, already applied to `sim`: ends an open
    /// window as `config_change`, marks an admitted manual sample, and emits
    /// `run.config`.
    pub(crate) fn config(&self, sim: &Simulation, window: &mut Option<ActiveTrace>) {
        if let Some(run) = self.current().as_mut() {
            self.telemetry.config_patched(run, sim, window);
            self.telemetry.run_config(run, &sim.config, sim.tick);
        }
    }

    /// Exports the open window and unfetched manual sample as `run_end` and
    /// ends the current run as `shutdown` after its run-end snapshot of
    /// `sim`; returns at once. [`ServerTelemetry::flush`] follows it.
    pub fn end_for_shutdown(&self, sim: &Simulation, samples: Samples<'_>) {
        self.end_samples(sim, samples);
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
            tick_traces: v3_telemetry::Switch::On,
            windows: v3_telemetry::WindowSettings::default(),
        }));
        let mut first = seed_simulation(config(), 1);
        telemetry.begin(&first.config, 1);
        telemetry.transition(SimulationStatus::Running, &first);
        telemetry.transition(SimulationStatus::Paused, &first);
        ticks(&mut first, 2);
        std::thread::sleep(Duration::from_millis(15));
        telemetry.transition(SimulationStatus::Running, &first);
        ticks(&mut first, 1);
        telemetry.after_tick(&first, &mut None, None);
        let mut second = seed_simulation(config(), 2);
        let (mut first_window, mut second_window) = (None, None);
        let first_samples = Samples {
            window: &mut first_window,
            manual: None,
        };
        telemetry.reset(&first, SimulationStatus::Running, &second, 2, first_samples);
        ticks(&mut second, 2);
        let second_samples = Samples {
            window: &mut second_window,
            manual: None,
        };
        telemetry.end_for_shutdown(&second, second_samples);
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
