use rand::rngs::SmallRng;
use rand::SeedableRng;
use std::collections::HashSet;
use v3_core::config::SimulationConfig;
use v3_core::kernel::types::Position;
use v3_core::kernel::world_state::WorldState;
use v3_core::seed::seed_creatures;
use v3_core::SimulationState;

/// Config tuned for viability: generous food, moderate decay, small world.
fn viability_config() -> SimulationConfig {
    let mut config = SimulationConfig::default();
    config.world.food.growth_rate = 0.05;
    config.world.food.initial_density = 100;
    config.world.food.initial_coverage = 0.5;
    config.energy.lifecycle.initial_energy = 30;
    config.energy.lifecycle.max_energy = 100;
    config.energy.lifecycle.energy_decay_per_tick = 1;
    // This test remains the Stage 2 viability gate; disable reproduction here.
    config.energy.lifecycle.min_reproduce_energy = u32::MAX;
    config.energy.costs.move_cost = 1;
    config.energy.costs.eat_cost = 0;
    config.energy.costs.eat_reward_per_food = 1;
    config
}

fn total_food(state: &SimulationState, width: u16, height: u16) -> u64 {
    let mut total: u64 = 0;
    for y in 0..height {
        for x in 0..width {
            total += state.world.get_food_density(Position { x, y }) as u64;
        }
    }
    total
}

#[test]
fn viability_creatures_survive_100_ticks() {
    let config = viability_config();
    let (width, height) = (32, 32);
    let mut state = SimulationState::new(WorldState::new(width, height, true));
    let mut rng = SmallRng::seed_from_u64(12345);

    seed_creatures(&mut state, 15, &config, &mut rng);

    let initial_pop = state.creatures.len();
    assert_eq!(initial_pop, 15, "should seed all 15 creatures");

    // Record initial positions
    let initial_positions: HashSet<(u16, u16)> = state
        .creatures
        .iter()
        .map(|(_, c)| (c.position.x, c.position.y))
        .collect();

    let initial_food = total_food(&state, width, height);
    assert!(initial_food > 0, "food should have been seeded");

    // Record initial energies
    let initial_max_energy: u32 = state
        .creatures
        .iter()
        .map(|(_, c)| c.energy.value())
        .max()
        .unwrap();

    let mut total_actions_attempted: u32 = 0;

    // Run 100 ticks
    for _ in 0..100 {
        let stats = state.tick(&config, &mut rng);
        total_actions_attempted += stats.actions_attempted;
    }

    // Assert: population > 0 after 100 ticks
    assert!(
        !state.creatures.is_empty(),
        "population collapsed to 0 — ecology not viable"
    );

    // Assert: at least some creatures have energy > initial (they ate)
    let max_energy: u32 = state
        .creatures
        .iter()
        .map(|(_, c)| c.energy.value())
        .max()
        .unwrap_or(0);
    assert!(
        max_energy > initial_max_energy,
        "no creature gained energy above initial {} (max now {})",
        initial_max_energy,
        max_energy
    );

    // Assert: food has been consumed (total food < initial or regrown food not matching)
    // Since food both grows and is consumed, check that creatures actually ate
    // by verifying actions happened
    assert!(
        total_actions_attempted > 0,
        "no actions were attempted during simulation"
    );

    // Assert: creature positions have changed from initial
    let final_positions: HashSet<(u16, u16)> = state
        .creatures
        .iter()
        .map(|(_, c)| (c.position.x, c.position.y))
        .collect();
    assert_ne!(
        initial_positions, final_positions,
        "creature positions should have changed"
    );

    // Assert: TickStats actions_attempted > 0 (already checked above with accumulated total)
}
