use rand::rngs::SmallRng;
use rand::SeedableRng;
use v3_core::creature::state::CreatureState;
use v3_core::kernel::types::Position;
use v3_core::kernel::world_state::WorldState;
use v3_core::SimulationState;

#[test]
fn tick_advances_and_creatures_persist() {
    let mut state = SimulationState::new(WorldState::new(10, 10, true));
    let mut rng = SmallRng::seed_from_u64(42);

    state.spawn_creature(CreatureState::new(
        Position { x: 1, y: 1 },
        10,
        0,
        [255, 0, 0],
    ));
    state.spawn_creature(CreatureState::new(
        Position { x: 2, y: 2 },
        10,
        0,
        [0, 255, 0],
    ));
    state.spawn_creature(CreatureState::new(
        Position { x: 3, y: 3 },
        10,
        0,
        [0, 0, 255],
    ));
    assert_eq!(state.creatures.len(), 3);

    state.tick(&mut rng);

    assert_eq!(state.tick_number, 1);
    assert_eq!(state.creatures.len(), 3);
    // All creatures should have aged
    for (_, creature) in state.creatures.iter() {
        assert_eq!(creature.age, 1);
        assert!(state.world.creature_at(creature.position).is_some());
    }
}

#[test]
fn tick_removes_dead_creatures_and_frees_positions() {
    let mut state = SimulationState::new(WorldState::new(10, 10, true));
    let mut rng = SmallRng::seed_from_u64(42);

    let pos_alive = Position { x: 1, y: 1 };
    let pos_dead = Position { x: 2, y: 2 };

    state.spawn_creature(CreatureState::new(pos_alive, 10, 0, [255, 0, 0]));
    state.spawn_creature(CreatureState::new(pos_dead, 0, 0, [0, 255, 0])); // energy 0 = dead
    assert_eq!(state.creatures.len(), 2);

    state.tick(&mut rng);

    // Dead creature removed from SlotMap and spatial index
    assert_eq!(state.creatures.len(), 1);
    assert!(!state.world.is_occupied(pos_dead));
    // Living creature still present
    assert!(state.world.is_occupied(pos_alive));
}

#[test]
fn tick_returns_stats_with_death_count() {
    let mut state = SimulationState::new(WorldState::new(10, 10, true));
    let mut rng = SmallRng::seed_from_u64(42);

    state.spawn_creature(CreatureState::new(
        Position { x: 1, y: 1 },
        10,
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

    let stats = state.tick(&mut rng);

    assert_eq!(stats.deaths, 2);
    assert_eq!(stats.births, 0);
}

#[test]
fn multiple_ticks_accumulate_age() {
    let mut state = SimulationState::new(WorldState::new(10, 10, true));
    let mut rng = SmallRng::seed_from_u64(42);

    state.spawn_creature(CreatureState::new(
        Position { x: 5, y: 5 },
        100,
        0,
        [128, 128, 128],
    ));

    for _ in 0..10 {
        state.tick(&mut rng);
    }

    assert_eq!(state.tick_number, 10);
    for (_, creature) in state.creatures.iter() {
        assert_eq!(creature.age, 10);
    }
}
