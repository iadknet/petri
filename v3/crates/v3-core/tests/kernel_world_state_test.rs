#[test]
fn position_equality_works() {
    use v3_core::kernel::types::Position;

    let pos1 = Position { x: 5, y: 10 };
    let pos2 = Position { x: 5, y: 10 };
    let pos3 = Position { x: 6, y: 10 };

    assert_eq!(pos1, pos2);
    assert_ne!(pos1, pos3);
}

#[test]
fn direction_delta_returns_correct_offsets() {
    use v3_core::kernel::types::Direction;

    assert_eq!(Direction::N.delta(), (0, -1));
    assert_eq!(Direction::NE.delta(), (1, -1));
    assert_eq!(Direction::E.delta(), (1, 0));
    assert_eq!(Direction::SE.delta(), (1, 1));
    assert_eq!(Direction::S.delta(), (0, 1));
    assert_eq!(Direction::SW.delta(), (-1, 1));
    assert_eq!(Direction::W.delta(), (-1, 0));
    assert_eq!(Direction::NW.delta(), (-1, -1));
}

#[test]
fn world_state_new_creates_empty_world() {
    use v3_core::kernel::types::Position;
    use v3_core::kernel::world_state::WorldState;

    let world = WorldState::new(10, 10, true);

    assert_eq!(world.width, 10);
    assert_eq!(world.height, 10);
    assert!(world.wrap);
    assert_eq!(world.get_food_density(Position { x: 5, y: 5 }), 0);
    assert!(!world.is_occupied(Position { x: 5, y: 5 }));
    assert!(world.creature_at(Position { x: 5, y: 5 }).is_none());
}

#[test]
fn place_and_remove_creature_updates_spatial_index() {
    use slotmap::SlotMap;
    use v3_core::kernel::types::{CreatureId, Position};
    use v3_core::kernel::world_state::WorldState;

    let mut world = WorldState::new(10, 10, true);
    let pos = Position { x: 3, y: 4 };

    // Create a CreatureId via a temporary SlotMap
    let mut slots: SlotMap<CreatureId, ()> = SlotMap::with_key();
    let id = slots.insert(());

    // Place creature
    world.place_creature(pos, id);
    assert!(world.is_occupied(pos));
    assert_eq!(world.creature_at(pos), Some(id));

    // Remove creature
    world.remove_creature(pos);
    assert!(!world.is_occupied(pos));
    assert_eq!(world.creature_at(pos), None);
}

#[test]
fn resolve_neighbor_wraps_correctly() {
    use v3_core::kernel::types::{Direction, Position};
    use v3_core::kernel::world_state::WorldState;

    let world = WorldState::new(10, 10, true);

    // Normal neighbor
    let pos = Position { x: 5, y: 5 };
    assert_eq!(
        world.resolve_neighbor(pos, Direction::N),
        Some(Position { x: 5, y: 4 })
    );

    // Wrap around top edge
    let top = Position { x: 5, y: 0 };
    assert_eq!(
        world.resolve_neighbor(top, Direction::N),
        Some(Position { x: 5, y: 9 })
    );

    // Wrap around left edge
    let left = Position { x: 0, y: 5 };
    assert_eq!(
        world.resolve_neighbor(left, Direction::W),
        Some(Position { x: 9, y: 5 })
    );
}

#[test]
fn grow_food_probabilistically_capped_at_255() {
    use rand::SeedableRng;
    use v3_core::config::world::FoodConfig;
    use v3_core::kernel::types::Position;
    use v3_core::kernel::world_state::WorldState;

    let mut world = WorldState::new(4, 4, false);
    let mut rng = rand::rngs::SmallRng::seed_from_u64(42);

    // Set growth_rate to 1.0 so every cell grows
    let config = FoodConfig {
        growth_rate: 1.0,
        initial_density: 0,
        initial_coverage: 0.0,
    };

    // Grow once — every cell should gain +1
    world.grow_food(&config, &mut rng);
    assert_eq!(world.get_food_density(Position { x: 0, y: 0 }), 1);
    assert_eq!(world.get_food_density(Position { x: 3, y: 3 }), 1);

    // Set a cell near max and verify cap at 255
    world.set_food_density(Position { x: 1, y: 1 }, 255);
    world.grow_food(&config, &mut rng);
    assert_eq!(world.get_food_density(Position { x: 1, y: 1 }), 255); // capped

    // Barrier cells do not grow
    world.set_barrier(Position { x: 2, y: 2 }, true);
    let before = world.get_food_density(Position { x: 2, y: 2 });
    world.grow_food(&config, &mut rng);
    assert_eq!(world.get_food_density(Position { x: 2, y: 2 }), before);
}

#[test]
fn seed_food_places_initial_food() {
    use rand::SeedableRng;
    use v3_core::config::world::FoodConfig;
    use v3_core::kernel::types::Position;
    use v3_core::kernel::world_state::WorldState;

    let mut world = WorldState::new(10, 10, false);
    let mut rng = rand::rngs::SmallRng::seed_from_u64(42);

    let config = FoodConfig {
        growth_rate: 0.0,
        initial_density: 80,
        initial_coverage: 1.0, // all cells get food
    };

    world.seed_food(&config, &mut rng);
    // With coverage 1.0, every cell should have density 80
    assert_eq!(world.get_food_density(Position { x: 0, y: 0 }), 80);
    assert_eq!(world.get_food_density(Position { x: 9, y: 9 }), 80);
}

#[test]
fn resolve_neighbor_non_wrapping_returns_none_at_edges() {
    use v3_core::kernel::types::{Direction, Position};
    use v3_core::kernel::world_state::WorldState;

    let world = WorldState::new(10, 10, false); // No wrapping

    let top = Position { x: 5, y: 0 };
    assert_eq!(world.resolve_neighbor(top, Direction::N), None);

    let left = Position { x: 0, y: 5 };
    assert_eq!(world.resolve_neighbor(left, Direction::W), None);

    // Interior still works
    let mid = Position { x: 5, y: 5 };
    assert_eq!(
        world.resolve_neighbor(mid, Direction::S),
        Some(Position { x: 5, y: 6 })
    );
}
