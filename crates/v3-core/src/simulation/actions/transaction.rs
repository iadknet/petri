//! One action as one transaction (T30.F06): the mechanics resolve an action
//! over narrow inputs, settle the actor's floor and overdraft, and report what
//! the scheduler commits: the applied result, the actor's energy, the
//! removals the action caused and a conception born at once. The transaction
//! pushes no log entry and removes no creature.

use slotmap::SlotMap;

use super::{
    apply_give_energy, apply_move, apply_noop, apply_reproduce, apply_steal_energy,
    apply_typed_eat, GiveActionResult, PredationActionResult, ReproductionActionResult,
};
use crate::config::{OrdinaryFoodTypeId, SimulationConfig};
use crate::contracts::{CreatureId, Direction, Position, WorldAction};
use crate::creature::action_log::{ActionResult, ActionType, NO_DIRECTION};
use crate::creature::state::CreatureState;
use crate::kernel::WorldState;
use crate::ledger::{applied_debit, ActionCharges, FailedActionEffort};
use crate::simulation::stats::SimStats;

/// What one action reads and writes: the creatures, the world, the config,
/// the run totals the executors still write, and the tick.
pub(crate) struct ActionInputs<'a> {
    pub(crate) creatures: &'a mut SlotMap<CreatureId, CreatureState>,
    pub(crate) world: &'a mut WorldState,
    pub(crate) config: &'a SimulationConfig,
    pub(crate) stats: &'a mut SimStats,
    pub(crate) tick: u64,
}

/// A litter conceived with a gestation span of 0 (T03.F12): the caller
/// delivers it on `target` at once.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Conception {
    pub(crate) target: Position,
    pub(crate) litter: f32,
}

/// The action-log fields an applied action determines.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct LoggedResult {
    pub(crate) action_type: ActionType,
    pub(crate) result: ActionResult,
    pub(crate) direction: u8,
    pub(crate) amount: f32,
    pub(crate) food_type: Option<OrdinaryFoodTypeId>,
}

impl LoggedResult {
    fn new(action_type: ActionType, result: ActionResult, direction: Direction) -> Self {
        Self {
            action_type,
            result,
            direction: direction.to_index() as u8,
            amount: 0.0,
            food_type: None,
        }
    }

    /// Whether the action counts as a success for the reward count: every
    /// applied outcome, a kill included.
    #[must_use]
    pub(crate) fn succeeded(&self) -> bool {
        matches!(
            self.result,
            ActionResult::Success | ActionResult::TransferredAndKilled
        )
    }
}

/// One resolved action.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Applied {
    pub(crate) result: LoggedResult,
    /// The actor's energy after the action, before the zero floor.
    pub(crate) energy_after: f32,
    /// The actor is at or below zero after the floor; the caller removes it.
    pub(crate) actor_removed: bool,
    /// The creature on the target cell of a steal or give; a
    /// `TransferredAndKilled` steal's counterpart is the caller's to remove.
    pub(crate) counterpart: Option<CreatureId>,
    /// A span-0 conception the caller delivers.
    pub(crate) conceived: Option<Conception>,
}

/// Resolve `action` for the living creature `id`, then settle it: floor the
/// actor at zero and net its own overdraft out of the action's charge.
pub(crate) fn resolve(
    inputs: &mut ActionInputs<'_>,
    id: CreatureId,
    action: &WorldAction,
) -> Applied {
    let flows = &inputs.stats.energy_flows;
    let charged_before = (flows.action_charges, flows.failed_action_effort);
    let mut counterpart = None;
    let mut conceived = None;
    let result = match *action {
        WorldAction::NoOp => noop(inputs, id),
        WorldAction::Eat { type_idx, leave } => eat(inputs, id, type_idx, leave),
        WorldAction::Move(dir) => step(inputs, id, dir),
        WorldAction::Reproduce {
            direction,
            energy_transfer_fraction,
        } => {
            let (result, conception) = reproduce(inputs, id, direction, energy_transfer_fraction);
            conceived = conception;
            result
        }
        WorldAction::StealEnergy { direction, effort } => {
            let (result, victim) = steal(inputs, id, direction, effort);
            counterpart = victim;
            result
        }
        WorldAction::GiveEnergy { direction, effort } => {
            let (result, recipient) = give(inputs, id, direction, effort);
            counterpart = recipient;
            result
        }
    };
    let (energy_after, actor_removed) = settle(inputs, id, charged_before);
    Applied {
        result,
        energy_after,
        actor_removed,
        counterpart,
        conceived,
    }
}

/// Floor the actor at `0.0`: the action's own overdraft nets out of its
/// charge, only a deficit it inherited is a floor credit. Returns the energy
/// before the floor and whether the actor is at or below zero after it.
fn settle(
    inputs: &mut ActionInputs<'_>,
    id: CreatureId,
    charged_before: (ActionCharges, FailedActionEffort),
) -> (f32, bool) {
    let creature = &mut inputs.creatures[id];
    let before = creature.energy;
    creature.energy = creature.energy.max(0.0);
    let overshoot = applied_debit(creature.energy, before);
    let removed = creature.energy <= 0.0;
    let flows = &mut inputs.stats.energy_flows;
    flows.zero_floor_credit += flows.net_action_overdraft(charged_before, overshoot);
    (before, removed)
}

/// NoOp cannot fail; its effort is the floor every action's effort is held to.
fn noop(inputs: &mut ActionInputs<'_>, id: CreatureId) -> LoggedResult {
    let creature = &mut inputs.creatures[id];
    creature.record_action_attempt(ActionType::NoOp);
    apply_noop(creature, inputs.config, &mut inputs.stats.energy_flows);
    inputs.stats.last_tick_noop += 1;
    LoggedResult {
        direction: NO_DIRECTION,
        ..LoggedResult::new(ActionType::NoOp, ActionResult::Success, Direction::N)
    }
}

fn eat(
    inputs: &mut ActionInputs<'_>,
    id: CreatureId,
    type_idx: OrdinaryFoodTypeId,
    leave: f32,
) -> LoggedResult {
    let stats = &mut *inputs.stats;
    let creature = &mut inputs.creatures[id];
    creature.record_action_attempt(ActionType::Eat);
    let food_before = inputs.world.food_at_type(creature.position, type_idx);
    let eaten = apply_typed_eat(
        creature,
        inputs.world,
        inputs.config,
        type_idx,
        leave,
        &mut stats.energy_flows,
    );
    stats.last_tick_eat += 1;
    let mut logged = LoggedResult {
        direction: NO_DIRECTION,
        food_type: Some(type_idx),
        ..LoggedResult::new(ActionType::Eat, ActionResult::Success, Direction::N)
    };
    if eaten > 0.0 {
        logged.amount = eaten;
        creature.record_applied_eat(type_idx);
        *stats
            .eat_actions_applied_total_by_type
            .entry(type_idx)
            .or_insert(0) += 1;
        if eaten < food_before {
            *stats
                .eat_actions_partial_total_by_type
                .entry(type_idx)
                .or_insert(0) += 1;
        }
    } else {
        logged.result = ActionResult::NoFood;
        *stats
            .eat_actions_failed_total_by_type
            .entry(type_idx)
            .or_insert(0) += 1;
    }
    logged
}

fn step(inputs: &mut ActionInputs<'_>, id: CreatureId, dir: Direction) -> LoggedResult {
    let creature = &mut inputs.creatures[id];
    creature.record_action_attempt(ActionType::Move);
    let moved = apply_move(
        id,
        creature,
        inputs.world,
        dir,
        inputs.config,
        &mut inputs.stats.energy_flows,
    );
    inputs.stats.last_tick_move += 1;
    inputs.stats.move_actions_attempted_total += 1;
    if moved {
        LoggedResult::new(ActionType::Move, ActionResult::Success, dir)
    } else {
        creature.lifetime_blocked_move_count += 1;
        LoggedResult::new(ActionType::Move, ActionResult::Blocked, dir)
    }
}

fn reproduce(
    inputs: &mut ActionInputs<'_>,
    id: CreatureId,
    direction: Direction,
    energy_transfer_fraction: f32,
) -> (LoggedResult, Option<Conception>) {
    let (result, conception) = apply_reproduce(inputs, id, direction, energy_transfer_fraction);
    let creature = &mut inputs.creatures[id];
    creature.record_action_attempt(ActionType::Reproduce);
    if result == ReproductionActionResult::RejectedInvalidTarget {
        creature.lifetime_invalid_reproduce_count += 1;
    }
    let logged = match result {
        ReproductionActionResult::Conceived => ActionResult::Success,
        ReproductionActionResult::RejectedInvalidTarget => ActionResult::InvalidTarget,
        ReproductionActionResult::RejectedAgeConstraints => ActionResult::AgeConstraints,
        ReproductionActionResult::RejectedEnergyConstraints => ActionResult::EnergyConstraints,
        ReproductionActionResult::RejectedPopulationCap => ActionResult::PopulationCap,
    };
    let logged = LoggedResult {
        amount: energy_transfer_fraction,
        ..LoggedResult::new(ActionType::Reproduce, logged, direction)
    };
    (logged, conception)
}

fn steal(
    inputs: &mut ActionInputs<'_>,
    id: CreatureId,
    direction: Direction,
    effort: f32,
) -> (LoggedResult, Option<CreatureId>) {
    inputs.creatures[id].record_action_attempt(ActionType::StealEnergy);
    let outcome = apply_steal_energy(inputs, id, direction, effort);
    let result = match outcome.result {
        PredationActionResult::Transferred => ActionResult::Success,
        PredationActionResult::TransferredAndKilled => ActionResult::TransferredAndKilled,
        PredationActionResult::RejectedNoVictim | PredationActionResult::RejectedNothingToTake => {
            ActionResult::NoVictim
        }
    };
    let logged = LoggedResult {
        amount: outcome.amount,
        ..LoggedResult::new(ActionType::StealEnergy, result, direction)
    };
    (logged, outcome.counterpart)
}

fn give(
    inputs: &mut ActionInputs<'_>,
    id: CreatureId,
    direction: Direction,
    effort: f32,
) -> (LoggedResult, Option<CreatureId>) {
    inputs.creatures[id].record_action_attempt(ActionType::GiveEnergy);
    let outcome = apply_give_energy(inputs, id, direction, effort);
    let result = match outcome.result {
        GiveActionResult::Transferred => ActionResult::Success,
        GiveActionResult::RejectedNoRecipient => ActionResult::NoRecipient,
        GiveActionResult::RejectedNothingToGive => ActionResult::EnergyConstraints,
    };
    let logged = LoggedResult {
        amount: outcome.amount,
        ..LoggedResult::new(ActionType::GiveEnergy, result, direction)
    };
    (logged, outcome.counterpart)
}

#[cfg(test)]
#[path = "transaction_tests.rs"]
mod tests;
