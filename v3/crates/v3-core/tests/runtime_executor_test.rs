#[test]
fn heuristic_eats_when_food_present() {
    use rand::SeedableRng;
    use v3_core::contracts::inputs::CreatureInputs;
    use v3_core::contracts::outputs::WorldAction;
    use v3_core::runtime::executor::execute_heuristic;

    let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
    let mut inputs = CreatureInputs::default();
    inputs.environmental.food_density_self = 50;

    let outputs = execute_heuristic(&inputs, &mut rng);
    assert!(matches!(outputs.world_action, WorldAction::Eat));
}

#[test]
fn heuristic_moves_toward_food() {
    use rand::SeedableRng;
    use v3_core::contracts::inputs::{CreatureInputs, NeighborSense};
    use v3_core::contracts::outputs::WorldAction;
    use v3_core::kernel::types::Direction;
    use v3_core::runtime::executor::execute_heuristic;

    let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
    let mut inputs = CreatureInputs::default();
    // All neighbors passable
    for n in inputs.environmental.neighbors.iter_mut() {
        n.passable = true;
    }
    // East (index 2) has the most food
    inputs.environmental.neighbors[2] = NeighborSense {
        food_density: 100,
        passable: true,
    };

    let outputs = execute_heuristic(&inputs, &mut rng);
    assert!(matches!(
        outputs.world_action,
        WorldAction::Move {
            direction: Direction::E
        }
    ));
}

#[test]
fn heuristic_moves_randomly_when_no_food() {
    use rand::SeedableRng;
    use v3_core::contracts::inputs::CreatureInputs;
    use v3_core::contracts::outputs::WorldAction;
    use v3_core::runtime::executor::execute_heuristic;

    let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
    let mut inputs = CreatureInputs::default();
    // Make some neighbors passable, but no food anywhere
    for n in inputs.environmental.neighbors.iter_mut() {
        n.passable = true;
    }

    let outputs = execute_heuristic(&inputs, &mut rng);
    assert!(matches!(
        outputs.world_action,
        WorldAction::Move { direction: _ }
    ));
}

#[test]
fn heuristic_returns_noop_when_trapped() {
    use rand::SeedableRng;
    use v3_core::contracts::inputs::CreatureInputs;
    use v3_core::contracts::outputs::WorldAction;
    use v3_core::runtime::executor::execute_heuristic;

    let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
    // Default: all neighbors have passable=false, no food
    let inputs = CreatureInputs::default();

    let outputs = execute_heuristic(&inputs, &mut rng);
    assert!(matches!(outputs.world_action, WorldAction::NoOp));
}

#[test]
fn gather_inputs_returns_creature_energy_and_food() {
    use v3_core::creature::state::CreatureState;
    use v3_core::kernel::types::Position;
    use v3_core::kernel::world_state::WorldState;
    use v3_core::sensors::gather_inputs;

    let creature = CreatureState::new(Position { x: 5, y: 5 }, 42, 0, [100, 100, 100]);
    let mut world = WorldState::new(10, 10, true);
    world.set_food_density(Position { x: 5, y: 5 }, 120);

    let inputs = gather_inputs(&creature, &world);

    assert_eq!(inputs.introspection.energy, 42);
    assert_eq!(inputs.introspection.position, Position { x: 5, y: 5 });
    assert_eq!(inputs.environmental.food_density_self, 120);
}

#[test]
fn gather_inputs_reads_neighbor_food_density() {
    use v3_core::creature::state::CreatureState;
    use v3_core::kernel::types::Position;
    use v3_core::kernel::world_state::WorldState;
    use v3_core::sensors::gather_inputs;

    let creature = CreatureState::new(Position { x: 5, y: 5 }, 10, 0, [100, 100, 100]);
    let mut world = WorldState::new(10, 10, true);
    // Place food to the north (Direction::ALL[0] = N, offset (0,-1))
    world.set_food_density(Position { x: 5, y: 4 }, 50);
    // Place food to the east (Direction::ALL[2] = E, offset (1,0))
    world.set_food_density(Position { x: 6, y: 5 }, 75);

    let inputs = gather_inputs(&creature, &world);

    // Index 0 = N
    assert_eq!(inputs.environmental.neighbors[0].food_density, 50);
    assert!(inputs.environmental.neighbors[0].passable);
    // Index 2 = E
    assert_eq!(inputs.environmental.neighbors[2].food_density, 75);
    assert!(inputs.environmental.neighbors[2].passable);
    // Index 4 = S (no food)
    assert_eq!(inputs.environmental.neighbors[4].food_density, 0);
    assert!(inputs.environmental.neighbors[4].passable);
}

#[test]
fn gather_inputs_detects_barriers_and_occupied() {
    use slotmap::SlotMap;
    use v3_core::creature::state::CreatureState;
    use v3_core::kernel::types::{CreatureId, Position};
    use v3_core::kernel::world_state::WorldState;
    use v3_core::sensors::gather_inputs;

    let creature = CreatureState::new(Position { x: 5, y: 5 }, 10, 0, [100, 100, 100]);
    let mut world = WorldState::new(10, 10, true);

    // Place barrier to the north
    world.set_barrier(Position { x: 5, y: 4 }, true);
    // Place a creature to the east
    let mut slots: SlotMap<CreatureId, ()> = SlotMap::with_key();
    let other_id = slots.insert(());
    world.place_creature(Position { x: 6, y: 5 }, other_id);

    let inputs = gather_inputs(&creature, &world);

    // North is barrier — not passable
    assert!(!inputs.environmental.neighbors[0].passable);
    // East is occupied — not passable
    assert!(!inputs.environmental.neighbors[2].passable);
    // South is clear — passable
    assert!(inputs.environmental.neighbors[4].passable);
}
