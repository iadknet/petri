// Heuristic executor tests removed in Stage 3C: the heuristic brain was
// replaced by the VM executor. VM unit tests live in src/runtime/vm.rs.
// Sensor tests (gather_inputs) remain here since they exercise the sensing
// layer independently of the executor.

#[test]
fn gather_inputs_returns_creature_energy_and_food() {
    use v3_core::creature::founders;
    use v3_core::creature::state::CreatureState;
    use v3_core::kernel::types::Position;
    use v3_core::kernel::world_state::WorldState;
    use v3_core::sensors::gather_inputs;

    let creature = CreatureState::new(
        Position { x: 5, y: 5 },
        42,
        0,
        [100, 100, 100],
        founders::get("simple"),
    );
    let mut world = WorldState::new(10, 10, true);
    world.set_food_density(Position { x: 5, y: 5 }, 120);

    let inputs = gather_inputs(&creature, &world);

    assert_eq!(inputs.introspection.energy, 42);
    assert_eq!(inputs.introspection.position, Position { x: 5, y: 5 });
    assert_eq!(inputs.environmental.food_density_self, 120);
}

#[test]
fn gather_inputs_reads_neighbor_food_density() {
    use v3_core::creature::founders;
    use v3_core::creature::state::CreatureState;
    use v3_core::kernel::types::Position;
    use v3_core::kernel::world_state::WorldState;
    use v3_core::sensors::gather_inputs;

    let creature = CreatureState::new(
        Position { x: 5, y: 5 },
        10,
        0,
        [100, 100, 100],
        founders::get("simple"),
    );
    let mut world = WorldState::new(10, 10, true);
    // Place food to the north (Direction::ALL[0] = N, offset (0,-1))
    world.set_food_density(Position { x: 5, y: 4 }, 50);
    // Place food to the east (Direction::ALL[2] = E, offset (1,0))
    world.set_food_density(Position { x: 6, y: 5 }, 75);

    let inputs = gather_inputs(&creature, &world);

    // Index 0 = N
    assert_eq!(inputs.environmental.neighbors[0].food_density, 50);
    assert!(inputs.environmental.neighbors[0].passable());
    // Index 2 = E
    assert_eq!(inputs.environmental.neighbors[2].food_density, 75);
    assert!(inputs.environmental.neighbors[2].passable());
    // Index 4 = S (no food)
    assert_eq!(inputs.environmental.neighbors[4].food_density, 0);
    assert!(inputs.environmental.neighbors[4].passable());
}

#[test]
fn gather_inputs_detects_barriers_and_occupied() {
    use slotmap::SlotMap;
    use v3_core::creature::founders;
    use v3_core::creature::state::CreatureState;
    use v3_core::kernel::types::{CreatureId, Position};
    use v3_core::kernel::world_state::WorldState;
    use v3_core::sensors::gather_inputs;

    let creature = CreatureState::new(
        Position { x: 5, y: 5 },
        10,
        0,
        [100, 100, 100],
        founders::get("simple"),
    );
    let mut world = WorldState::new(10, 10, true);

    // Place barrier to the north
    world.set_barrier(Position { x: 5, y: 4 }, true);
    // Place a creature to the east
    let mut slots: SlotMap<CreatureId, ()> = SlotMap::with_key();
    let other_id = slots.insert(());
    world.place_creature(Position { x: 6, y: 5 }, other_id);

    let inputs = gather_inputs(&creature, &world);

    // North is barrier — not passable
    assert!(!inputs.environmental.neighbors[0].passable());
    // East is occupied — not passable
    assert!(!inputs.environmental.neighbors[2].passable());
    // South is clear — passable
    assert!(inputs.environmental.neighbors[4].passable());
}
