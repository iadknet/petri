use rand::SeedableRng;
use v3_core::config::SimulationConfig;
use v3_core::contracts::outputs::WorldAction;
use v3_core::creature::state::CreatureState;
use v3_core::kernel::types::{CreatureId, Direction, Position};
use v3_core::kernel::world_state::WorldState;
use v3_core::tick::actions::{execute_action, ActionStatus, FailureReason};

fn setup() -> (
    SimulationConfig,
    WorldState,
    CreatureState,
    CreatureId,
    rand::rngs::SmallRng,
) {
    let config = SimulationConfig::default();
    let mut world = WorldState::new(10, 10, false);
    let pos = Position { x: 5, y: 5 };
    let creature = CreatureState::new(
        pos,
        20,
        0,
        [204, 61, 61],
        v3_core::creature::founders::get("simple"),
    );

    // Create a CreatureId via a temporary slot
    let mut slots = slotmap::SlotMap::<CreatureId, ()>::with_key();
    let id = slots.insert(());
    world.place_creature(pos, id);

    let rng = rand::rngs::SmallRng::seed_from_u64(42);
    (config, world, creature, id, rng)
}

#[test]
fn eat_consumes_food_and_gains_energy() {
    let (config, mut world, mut creature, id, mut rng) = setup();
    world.set_food_density(creature.position, 50);

    let initial_energy = creature.energy.value();
    let result = execute_action(
        &WorldAction::Eat,
        id,
        &mut creature,
        &mut world,
        &config,
        &mut rng,
    );

    assert!(result.succeeded());
    // Food consumed from world
    assert_eq!(world.get_food_density(Position { x: 5, y: 5 }), 0);
    // Energy gained: 50 food * 1 reward_per_food = 50 energy
    assert_eq!(creature.energy.value(), initial_energy + 50);
}

#[test]
fn move_updates_position_and_spatial_index() {
    let (config, mut world, mut creature, id, mut rng) = setup();
    let old_pos = creature.position;

    let result = execute_action(
        &WorldAction::Move {
            direction: Direction::E,
        },
        id,
        &mut creature,
        &mut world,
        &config,
        &mut rng,
    );

    assert!(result.succeeded());
    let new_pos = Position { x: 6, y: 5 };
    assert_eq!(creature.position, new_pos);
    assert!(!world.is_occupied(old_pos));
    assert!(world.is_occupied(new_pos));
    assert_eq!(world.creature_at(new_pos), Some(id));
}

#[test]
fn move_into_barrier_fails_but_costs_energy() {
    let (mut config, mut world, mut creature, id, mut rng) = setup();
    config.energy.costs.move_cost = 2;

    // Place barrier to the east
    world.set_barrier(Position { x: 6, y: 5 }, true);
    let initial_energy = creature.energy.value();

    let result = execute_action(
        &WorldAction::Move {
            direction: Direction::E,
        },
        id,
        &mut creature,
        &mut world,
        &config,
        &mut rng,
    );

    assert!(!result.succeeded());
    assert_eq!(
        result.status,
        ActionStatus::Failed(FailureReason::CellBlocked)
    );
    // Energy still drained
    assert_eq!(creature.energy.value(), initial_energy - 2);
    // Position unchanged
    assert_eq!(creature.position, Position { x: 5, y: 5 });
}

#[test]
fn eat_with_no_food_fails() {
    let (mut config, mut world, mut creature, id, mut rng) = setup();
    config.energy.costs.eat_cost = 1;
    // No food at creature's position (default is 0)

    let initial_energy = creature.energy.value();
    let result = execute_action(
        &WorldAction::Eat,
        id,
        &mut creature,
        &mut world,
        &config,
        &mut rng,
    );

    assert!(!result.succeeded());
    assert_eq!(result.status, ActionStatus::Failed(FailureReason::NoFood));
    // Eat cost still deducted
    assert_eq!(creature.energy.value(), initial_energy - 1);
}

#[test]
fn reproduce_spawns_offspring_and_transfers_energy() {
    let (config, mut world, mut creature, id, mut rng) = setup();
    creature
        .energy
        .charge(20, config.energy.lifecycle.max_energy); // 40 total
    creature.memory[0] = 77;
    creature.memory[1023] = 5;

    let initial_energy = creature.energy.value();
    let result = execute_action(
        &WorldAction::Reproduce {
            direction: Direction::E,
            energy_amount: 12,
        },
        id,
        &mut creature,
        &mut world,
        &config,
        &mut rng,
    );

    assert!(result.succeeded());
    let child = result
        .offspring
        .expect("reproduction should return offspring");
    assert_eq!(child.position, Position { x: 6, y: 5 });
    assert_eq!(child.energy.value(), 12);
    assert_eq!(child.generation, creature.generation + 1);
    assert_eq!(child.memory, creature.memory);
    assert_eq!(
        creature.energy.value(),
        initial_energy - config.energy.costs.reproduce_cost - 12
    );
}

#[test]
fn reproduce_into_barrier_fails_and_only_cost_is_charged() {
    let (config, mut world, mut creature, id, mut rng) = setup();
    creature
        .energy
        .charge(20, config.energy.lifecycle.max_energy); // 40 total
    world.set_barrier(Position { x: 6, y: 5 }, true);

    let initial_energy = creature.energy.value();
    let result = execute_action(
        &WorldAction::Reproduce {
            direction: Direction::E,
            energy_amount: 12,
        },
        id,
        &mut creature,
        &mut world,
        &config,
        &mut rng,
    );

    assert_eq!(
        result.status,
        ActionStatus::Failed(FailureReason::CellBlocked)
    );
    assert!(result.offspring.is_none());
    assert_eq!(
        creature.energy.value(),
        initial_energy - config.energy.costs.reproduce_cost
    );
}

#[test]
fn reproduce_fails_when_parent_energy_is_below_minimum() {
    let (config, mut world, mut creature, id, mut rng) = setup();
    // Keep default energy at 20, below default min_reproduce_energy (24).
    let initial_energy = creature.energy.value();
    let result = execute_action(
        &WorldAction::Reproduce {
            direction: Direction::E,
            energy_amount: 12,
        },
        id,
        &mut creature,
        &mut world,
        &config,
        &mut rng,
    );

    assert_eq!(
        result.status,
        ActionStatus::Failed(FailureReason::InsufficientEnergy)
    );
    assert!(result.offspring.is_none());
    // Action cost is still charged even on failure.
    assert_eq!(
        creature.energy.value(),
        initial_energy - config.energy.costs.reproduce_cost
    );
}
