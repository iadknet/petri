use rand::rngs::SmallRng;
use rand::SeedableRng;
use v3_core::config::SimulationConfig;
use v3_core::creature::state::CreatureState;
use v3_core::kernel::types::Position;
use v3_core::kernel::world_state::WorldState;
use v3_core::SimulationState;

fn default_config() -> SimulationConfig {
    let mut config = SimulationConfig::default();
    // Most integration tests in this file validate Stage 2 behavior.
    // Disable reproduction by default and opt-in only in the dedicated test.
    config.energy.lifecycle.min_reproduce_energy = u32::MAX;
    config
}

#[test]
fn tick_advances_and_creatures_persist() {
    let config = default_config();
    let mut state = SimulationState::new(WorldState::new(10, 10, true));
    let mut rng = SmallRng::seed_from_u64(42);

    // Use enough energy to survive tick's energy decay
    state.spawn_creature(CreatureState::new(
        Position { x: 1, y: 1 },
        50,
        0,
        [255, 0, 0],
    ));
    state.spawn_creature(CreatureState::new(
        Position { x: 2, y: 2 },
        50,
        0,
        [0, 255, 0],
    ));
    state.spawn_creature(CreatureState::new(
        Position { x: 3, y: 3 },
        50,
        0,
        [0, 0, 255],
    ));
    assert_eq!(state.creatures.len(), 3);

    state.tick(&config, &mut rng);

    assert_eq!(state.tick_number, 1);
    assert_eq!(state.creatures.len(), 3);
    for (_, creature) in state.creatures.iter() {
        assert_eq!(creature.age, 1);
    }
}

#[test]
fn tick_removes_dead_creatures_and_frees_positions() {
    let config = default_config();
    let mut state = SimulationState::new(WorldState::new(10, 10, true));
    let mut rng = SmallRng::seed_from_u64(42);

    let pos_alive = Position { x: 1, y: 1 };
    let pos_dead = Position { x: 2, y: 2 };

    state.spawn_creature(CreatureState::new(pos_alive, 50, 0, [255, 0, 0]));
    state.spawn_creature(CreatureState::new(pos_dead, 0, 0, [0, 255, 0])); // energy 0 = dead
    assert_eq!(state.creatures.len(), 2);

    state.tick(&config, &mut rng);

    // Dead creature removed from SlotMap and spatial index
    assert_eq!(state.creatures.len(), 1);
    assert!(!state.world.is_occupied(pos_dead));
}

#[test]
fn tick_returns_stats_with_death_count() {
    let config = default_config();
    let mut state = SimulationState::new(WorldState::new(10, 10, true));
    let mut rng = SmallRng::seed_from_u64(42);

    state.spawn_creature(CreatureState::new(
        Position { x: 1, y: 1 },
        50,
        0,
        [255, 0, 0],
    ));
    state.spawn_creature(CreatureState::new(
        Position { x: 2, y: 2 },
        0,
        0,
        [0, 255, 0],
    )); // dead
    state.spawn_creature(CreatureState::new(
        Position { x: 3, y: 3 },
        0,
        0,
        [0, 0, 255],
    )); // dead

    let stats = state.tick(&config, &mut rng);

    assert_eq!(stats.deaths, 2);
    assert_eq!(stats.births, 0);
}

#[test]
fn multiple_ticks_accumulate_age() {
    let config = default_config();
    let mut state = SimulationState::new(WorldState::new(10, 10, true));
    let mut rng = SmallRng::seed_from_u64(42);

    state.spawn_creature(CreatureState::new(
        Position { x: 5, y: 5 },
        100,
        0,
        [128, 128, 128],
    ));

    for _ in 0..10 {
        state.tick(&config, &mut rng);
    }

    assert_eq!(state.tick_number, 10);
    for (_, creature) in state.creatures.iter() {
        assert_eq!(creature.age, 10);
    }
}

#[test]
fn creatures_eat_food_and_gain_energy() {
    let config = default_config();
    let mut state = SimulationState::new(WorldState::new(10, 10, true));
    let mut rng = SmallRng::seed_from_u64(42);

    let pos = Position { x: 5, y: 5 };
    state.spawn_creature(CreatureState::new(pos, 20, 0, [204, 61, 61]));
    state.world.set_food_density(pos, 50);

    // Run a few ticks — creature should eat and gain energy
    let initial_energy = 20u32;
    state.tick(&config, &mut rng);

    // After eating 50 food (reward 1 per food), creature gains 50 energy.
    // But also loses 1 decay + possible move cost. Check creature still alive with more energy.
    let creature = state.creatures.iter().next().unwrap().1;
    // Creature should have eaten (energy > initial - decay - move_cost)
    assert!(
        creature.energy.value() > initial_energy - 5,
        "creature should have eaten food and gained energy, got {}",
        creature.energy.value()
    );
}

#[test]
fn creatures_die_without_food() {
    let mut config = default_config();
    config.energy.lifecycle.energy_decay_per_tick = 2;
    config.world.food.growth_rate = 0.0; // no food growth
    let mut state = SimulationState::new(WorldState::new(10, 10, true));
    let mut rng = SmallRng::seed_from_u64(42);

    state.spawn_creature(CreatureState::new(
        Position { x: 5, y: 5 },
        10,
        0,
        [204, 61, 61],
    ));

    // With decay=2 and no food, creature should die within 10 ticks
    for _ in 0..10 {
        state.tick(&config, &mut rng);
    }

    assert_eq!(
        state.creatures.len(),
        0,
        "creature should have died from starvation"
    );
}

#[test]
fn tick_stats_show_nonzero_actions() {
    let config = default_config();
    let mut state = SimulationState::new(WorldState::new(10, 10, true));
    let mut rng = SmallRng::seed_from_u64(42);

    state.spawn_creature(CreatureState::new(
        Position { x: 5, y: 5 },
        50,
        0,
        [204, 61, 61],
    ));

    let stats = state.tick(&config, &mut rng);

    assert!(
        stats.actions_attempted > 0,
        "should have attempted at least one action"
    );
}

#[test]
fn tick_reproduction_adds_offspring_and_reports_births() {
    let mut config = default_config();
    config.energy.lifecycle.min_reproduce_energy = 24;
    config.world.food.growth_rate = 0.0;

    let mut state = SimulationState::new(WorldState::new(10, 10, true));
    let mut rng = SmallRng::seed_from_u64(99);

    state.spawn_creature(CreatureState::new(
        Position { x: 5, y: 5 },
        40,
        0,
        [204, 61, 61],
    ));

    let stats = state.tick(&config, &mut rng);

    assert_eq!(stats.births, 1, "tick should report one birth");
    assert_eq!(state.creatures.len(), 2, "offspring should be inserted");
    assert!(
        state.creatures.iter().any(|(_, c)| c.generation == 1),
        "one offspring should have generation=1"
    );
}
