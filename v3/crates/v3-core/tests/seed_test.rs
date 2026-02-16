use rand::rngs::SmallRng;
use rand::SeedableRng;
use v3_core::kernel::world_state::WorldState;
use v3_core::seed::seed_creatures;
use v3_core::SimulationState;

#[test]
fn seed_creatures_places_creatures_at_unique_positions() {
    let mut state = SimulationState::new(WorldState::new(10, 10, true));
    let mut rng = SmallRng::seed_from_u64(42);

    seed_creatures(&mut state, 5, 20, &mut rng);

    assert_eq!(state.creatures.len(), 5);

    // All creatures should be at unique, occupied positions
    for (id, creature) in state.creatures.iter() {
        assert!(state.world.creature_at(creature.position).is_some());
        assert_eq!(state.world.creature_at(creature.position), Some(id));
    }
}

#[test]
fn seed_creatures_respects_world_capacity() {
    // 2x2 world = 4 cells, request 10 creatures
    let mut state = SimulationState::new(WorldState::new(2, 2, true));
    let mut rng = SmallRng::seed_from_u64(42);

    seed_creatures(&mut state, 10, 20, &mut rng);

    assert!(state.creatures.len() <= 4); // Best-effort, bounded by capacity
}

#[test]
fn seed_creatures_initializes_energy_and_generation() {
    let mut state = SimulationState::new(WorldState::new(10, 10, true));
    let mut rng = SmallRng::seed_from_u64(99);

    seed_creatures(&mut state, 3, 50, &mut rng);

    for (_, creature) in state.creatures.iter() {
        assert_eq!(creature.energy.value(), 50);
        assert_eq!(creature.generation, 0);
        assert_eq!(creature.age, 0);
    }
}
