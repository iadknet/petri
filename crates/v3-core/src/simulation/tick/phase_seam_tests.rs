//! The tick-phase seam (T21.F03): what `run_tick` records for tracing.

use std::time::Duration;

use super::run_tick;
use crate::config::SimulationConfig;
use crate::simulation::seed_simulation;
use crate::simulation::stats::PhaseWallClock;

fn increments(before: PhaseWallClock, after: PhaseWallClock) -> [Duration; 5] {
    [
        after.world_update - before.world_update,
        after.sensor_assembly - before.sensor_assembly,
        after.cognition - before.cognition,
        after.actions - before.actions,
        after.reward_learning - before.reward_learning,
    ]
}

#[test]
fn a_tick_records_its_phase_timings_as_phase_wall_clock_adds_them() {
    let mut config = SimulationConfig::default();
    config.world.width = 32;
    config.world.height = 32;
    config.population.initial_creatures = 16;
    let mut sim = seed_simulation(config, 3);
    assert!(sim.stats.last_tick_phases.is_none());
    for _ in 0..3 {
        let before = sim.stats.phase_wall_clock;
        let population = sim.creatures.len();
        run_tick(&mut sim, &mut None);
        let seam = sim
            .stats
            .last_tick_phases
            .expect("the tick records its phases");
        assert_eq!(seam.tick, sim.tick);
        assert_eq!(seam.population_start, population);
        assert_eq!(seam.threads, rayon::current_num_threads());
        let increments = increments(before, sim.stats.phase_wall_clock);
        for (index, phase) in seam.phases.iter().enumerate() {
            assert_eq!(phase.elapsed, increments[index], "phase {index}");
        }
        assert_eq!(seam.phases[0].offset, Duration::ZERO);
        for pair in seam.phases.windows(2) {
            assert!(pair[0].offset + pair[0].elapsed <= pair[1].offset);
        }
        let last = seam.phases[4];
        assert!(last.offset + last.elapsed <= seam.elapsed);
    }
}
