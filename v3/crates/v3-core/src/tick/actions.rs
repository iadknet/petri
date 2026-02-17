use crate::config::SimulationConfig;
use crate::contracts::outputs::WorldAction;
use crate::creature::reproduction::create_offspring;
use crate::creature::state::CreatureState;
use crate::kernel::types::CreatureId;
use crate::kernel::world_state::WorldState;
use rand::Rng;

/// Outcome of attempting a world action.
#[derive(Clone, Debug)]
pub struct ActionResult {
    pub status: ActionStatus,
    pub offspring: Option<CreatureState>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActionStatus {
    Success,
    Failed(FailureReason),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FailureReason {
    NoFood,
    CellBlocked,
    OutOfBounds,
    InsufficientEnergy,
}

impl ActionResult {
    pub fn success() -> Self {
        Self {
            status: ActionStatus::Success,
            offspring: None,
        }
    }

    pub fn failed(reason: FailureReason) -> Self {
        Self {
            status: ActionStatus::Failed(reason),
            offspring: None,
        }
    }

    pub fn success_with_offspring(offspring: CreatureState) -> Self {
        Self {
            status: ActionStatus::Success,
            offspring: Some(offspring),
        }
    }

    pub fn succeeded(&self) -> bool {
        self.status == ActionStatus::Success
    }
}

/// Dispatch and execute a world action. Energy cost is always paid via
/// `drain_saturating`, regardless of whether the action succeeds.
/// `rng` included for forward compatibility with Stage 3 (reproduce).
pub fn execute_action(
    action: &WorldAction,
    creature_id: CreatureId,
    creature: &mut CreatureState,
    world: &mut WorldState,
    config: &SimulationConfig,
    rng: &mut impl Rng,
) -> ActionResult {
    match action {
        WorldAction::NoOp => {
            creature
                .energy
                .drain_saturating(config.energy.costs.noop_cost);
            ActionResult::success()
        }
        WorldAction::Eat => execute_eat(creature, world, config),
        WorldAction::Move { direction } => {
            execute_move(creature_id, creature, world, config, *direction)
        }
        WorldAction::Reproduce {
            direction,
            energy_amount,
        } => execute_reproduce(creature, world, config, *direction, *energy_amount, rng),
    }
}

fn execute_eat(
    creature: &mut CreatureState,
    world: &mut WorldState,
    config: &SimulationConfig,
) -> ActionResult {
    creature
        .energy
        .drain_saturating(config.energy.costs.eat_cost);

    let food = world.consume_food(creature.position);
    if food == 0 {
        return ActionResult::failed(FailureReason::NoFood);
    }

    let energy_gain = (food as u32) * config.energy.costs.eat_reward_per_food;
    creature
        .energy
        .charge(energy_gain, config.energy.lifecycle.max_energy);
    ActionResult::success()
}

fn execute_move(
    creature_id: CreatureId,
    creature: &mut CreatureState,
    world: &mut WorldState,
    config: &SimulationConfig,
    direction: crate::kernel::types::Direction,
) -> ActionResult {
    creature
        .energy
        .drain_saturating(config.energy.costs.move_cost);

    let target = world.resolve_neighbor(creature.position, direction);
    let target = match target {
        Some(pos) => pos,
        None => return ActionResult::failed(FailureReason::OutOfBounds),
    };

    if world.is_barrier(target) || world.is_occupied(target) {
        return ActionResult::failed(FailureReason::CellBlocked);
    }

    // Update spatial index and creature position
    world.remove_creature(creature.position);
    world.place_creature(target, creature_id);
    creature.position = target;

    ActionResult::success()
}

fn execute_reproduce(
    creature: &mut CreatureState,
    world: &mut WorldState,
    config: &SimulationConfig,
    direction: crate::kernel::types::Direction,
    energy_amount: u32,
    rng: &mut impl Rng,
) -> ActionResult {
    creature
        .energy
        .drain_saturating(config.energy.costs.reproduce_cost);

    if creature.energy.value() < config.energy.lifecycle.min_reproduce_energy {
        return ActionResult::failed(FailureReason::InsufficientEnergy);
    }

    let target = match world.resolve_neighbor(creature.position, direction) {
        Some(pos) => pos,
        None => return ActionResult::failed(FailureReason::OutOfBounds),
    };

    if world.is_barrier(target) || world.is_occupied(target) {
        return ActionResult::failed(FailureReason::CellBlocked);
    }

    let transfer = energy_amount.min(config.energy.lifecycle.default_offspring_energy);
    if transfer == 0 || creature.energy.value() < transfer {
        return ActionResult::failed(FailureReason::InsufficientEnergy);
    }

    creature.energy.drain_saturating(transfer);
    let child = create_offspring(creature, target, transfer, config, rng);
    ActionResult::success_with_offspring(child)
}
