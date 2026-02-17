use rand::rngs::SmallRng;
use rand::SeedableRng;
use v3_core::config::SimulationConfig;
use v3_core::kernel::world_state::WorldState;
use v3_core::seed::seed_creatures;
use v3_core::SimulationState;

fn default_config() -> SimulationConfig {
    SimulationConfig::default()
}

#[test]
fn seed_creatures_places_creatures_at_unique_positions() {
    let config = default_config();
    let mut state = SimulationState::new(WorldState::new(10, 10, true));
    let mut rng = SmallRng::seed_from_u64(42);

    seed_creatures(&mut state, 5, &config, &mut rng);

    assert_eq!(state.creatures.len(), 5);

    for (id, creature) in state.creatures.iter() {
        assert!(state.world.creature_at(creature.position).is_some());
        assert_eq!(state.world.creature_at(creature.position), Some(id));
    }
}

#[test]
fn seed_creatures_respects_world_capacity() {
    let config = default_config();
    // 2x2 world = 4 cells, request 10 creatures
    let mut state = SimulationState::new(WorldState::new(2, 2, true));
    let mut rng = SmallRng::seed_from_u64(42);

    seed_creatures(&mut state, 10, &config, &mut rng);

    assert!(state.creatures.len() <= 4);
}

#[test]
fn seed_creatures_initializes_energy_and_generation() {
    let mut config = default_config();
    config.energy.lifecycle.initial_energy = 50;
    let mut state = SimulationState::new(WorldState::new(10, 10, true));
    let mut rng = SmallRng::seed_from_u64(99);

    seed_creatures(&mut state, 3, &config, &mut rng);

    for (_, creature) in state.creatures.iter() {
        assert_eq!(creature.energy.value(), 50);
        assert_eq!(creature.generation, 0);
        assert_eq!(creature.age, 0);
        // Fixed phenotype
        assert_eq!(creature.phenotype_r, 204);
        assert_eq!(creature.phenotype_g, 61);
        assert_eq!(creature.phenotype_b, 61);
    }
}

#[test]
fn seed_creatures_seeds_food_on_world() {
    let mut config = default_config();
    config.world.food.initial_coverage = 1.0;
    config.world.food.initial_density = 80;
    let mut state = SimulationState::new(WorldState::new(10, 10, true));
    let mut rng = SmallRng::seed_from_u64(42);

    seed_creatures(&mut state, 1, &config, &mut rng);

    // With 100% coverage, there should be food cells
    let mut total_food: u32 = 0;
    for y in 0..10u16 {
        for x in 0..10u16 {
            total_food += state
                .world
                .get_food_density(v3_core::kernel::types::Position { x, y })
                as u32;
        }
    }
    assert!(total_food > 0, "food should have been seeded");
}
