//! The tick-phase seam (T21.F03): what `run_tick` records for tracing.

use std::time::Duration;

use super::run_tick;
use crate::config::SimulationConfig;
use crate::kernel::FoodPass;
use crate::simulation::seed_simulation;
use crate::simulation::stats::PhaseWallClock;

fn increments(before: PhaseWallClock, after: PhaseWallClock) -> [Duration; 6] {
    [
        after.food_growth - before.food_growth,
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
        let last = seam.phases[5];
        assert!(last.offset + last.elapsed <= seam.elapsed);
    }
}

/// The food passes of the default two-type world sit inside the
/// `food_growth` phase, in execution order, one `growth_spread` and one
/// `write_back` per food type.
#[test]
fn a_tick_records_its_food_passes_inside_the_food_growth_phase() {
    let mut config = SimulationConfig::default();
    config.world.width = 32;
    config.world.height = 32;
    config.population.initial_creatures = 16;
    let food_types = config.world.food.types.len();
    let mut sim = seed_simulation(config, 3);
    run_tick(&mut sim, &mut None);
    let seam = sim
        .stats
        .last_tick_phases
        .expect("the tick records its phases");
    let food_growth = seam.phases[0];
    let passes = &sim.stats.last_tick_food_passes;
    assert_eq!(passes[0].pass, FoodPass::DepletionRecovery);
    assert_eq!(passes[passes.len() - 1].pass, FoodPass::InhibitedCount);
    for pass in [FoodPass::GrowthSpread, FoodPass::WriteBack] {
        let types: Vec<_> = passes
            .iter()
            .filter(|span| span.pass == pass)
            .map(|span| span.food_type.map(|id| usize::from(id.get())))
            .collect();
        assert_eq!(types, (0..food_types).map(Some).collect::<Vec<_>>());
    }
    assert!(food_growth.offset <= passes[0].timing.offset);
    for pair in passes.windows(2) {
        assert!(pair[0].timing.offset + pair[0].timing.elapsed <= pair[1].timing.offset);
    }
    let last = passes[passes.len() - 1].timing;
    assert!(last.offset + last.elapsed <= food_growth.offset + food_growth.elapsed);
}
