//! The profiling harness behind `v3-cli profile-ticks`: a default-config
//! world of the given size and population, seeded with [`SEED`], run for the
//! given ticks with the time spent in the tick loop measured.
//!
//! CPU profiling with samply, on the `profiling` build:
//!
//! ```sh
//! cargo build --profile profiling -p v3-cli
//! samply record ./target/profiling/v3-cli profile-ticks --telemetry off \
//!     [--ticks 500] [--creatures 500] [--world-size 200]
//! ```
//!
//! Heap allocation profiling with dhat, through the `dhat-heap` feature:
//!
//! ```sh
//! cargo run -p v3-cli --features dhat-heap -- profile-ticks --telemetry off [--ticks 500]
//! ```

use std::time::{Duration, Instant};

use v3_core::config::SimulationConfig;
use v3_core::simulation::seeding::seed_simulation;
use v3_core::simulation::tick::run_tick;

/// The seed every profiling run uses.
pub const SEED: u64 = 42;

/// The workload: ticks run, founders seeded, and the square world's side.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Options {
    pub ticks: u64,
    pub creatures: u32,
    pub world_size: u16,
}

impl Options {
    /// The default config with the world size and founder count applied,
    /// normalized.
    #[must_use]
    pub fn config(&self) -> SimulationConfig {
        let mut config = SimulationConfig::default();
        config.world.width = self.world_size;
        config.world.height = self.world_size;
        config.population.initial_creatures = self.creatures;
        config.normalize();
        config
    }
}

/// What one profiling run measured.
#[derive(Debug, Clone, Copy)]
pub struct Outcome {
    /// Wall time of the tick loop alone; seeding is outside it.
    pub elapsed: Duration,
    pub creatures_remaining: usize,
}

/// Seeds the workload's world and runs its ticks.
#[must_use]
pub fn run(options: &Options) -> Outcome {
    let mut sim = seed_simulation(options.config(), SEED);
    let start = Instant::now();
    for _ in 0..options.ticks {
        run_tick(&mut sim, &mut None);
    }
    Outcome {
        elapsed: start.elapsed(),
        creatures_remaining: sim.creatures.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_run_ticks_a_small_world_and_reports_its_population() {
        let options = Options {
            ticks: 2,
            creatures: 10,
            world_size: 20,
        };
        let config = options.config();
        assert_eq!((config.world.width, config.world.height), (20, 20));
        assert_eq!(config.population.initial_creatures, 10);

        let mut sim = seed_simulation(config, SEED);
        for _ in 0..options.ticks {
            run_tick(&mut sim, &mut None);
        }
        let outcome = run(&options);

        assert_eq!(outcome.creatures_remaining, sim.creatures.len());
        assert!(outcome.creatures_remaining > 0);
    }
}
