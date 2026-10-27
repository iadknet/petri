//! One action as one transaction (T30.F06): the mechanics resolve an action
//! over narrow inputs, settle the actor's failed effort, floor and
//! overdraft, and report what the scheduler commits: the applied result, the
//! actor's energy, the removals the action caused and a conception born at
//! once. The transaction reads and writes no run total, pushes no log entry
//! and removes no creature.

use slotmap::SlotMap;

use super::{
    apply_give_energy, apply_move, apply_noop, apply_reproduce, apply_steal_energy,
    apply_typed_eat, neighbour_creature,
};
use crate::config::SimulationConfig;
use crate::contracts::{ActionResult, ActionType, CreatureId, Position, WorldAction};
use crate::creature::state::CreatureState;
use crate::kernel::WorldState;
use crate::ledger::{applied_debit, ActionCharges, EnergyFlows, FailedActionEffort};

/// What one action reads and writes: the creatures, the world, the config,
/// the energy ledger and the tick.
pub(crate) struct ActionInputs<'a> {
    pub(crate) creatures: &'a mut SlotMap<CreatureId, CreatureState>,
    pub(crate) world: &'a mut WorldState,
    pub(crate) config: &'a SimulationConfig,
    pub(crate) flows: &'a mut EnergyFlows,
    pub(crate) tick: u64,
}

/// A litter conceived with a gestation span of 0 (T03.F12): the caller
/// delivers it on `target` at once.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Conception {
    pub(crate) target: Position,
    pub(crate) litter: f32,
}

/// One resolved action.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Applied {
    pub(crate) result: ActionResult,
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

/// Resolve `action` for the living creature `id`, then settle it.
pub(crate) fn resolve(
    inputs: &mut ActionInputs<'_>,
    id: CreatureId,
    action: &WorldAction,
) -> Applied {
    let charged_before = (
        inputs.flows.action_charges,
        inputs.flows.failed_action_effort,
    );
    let mut counterpart = None;
    let mut conceived = None;
    let result = match *action {
        WorldAction::NoOp => apply_noop(&mut inputs.creatures[id], inputs.config, inputs.flows),
        WorldAction::Eat { type_idx, leave } => apply_typed_eat(
            &mut inputs.creatures[id],
            inputs.world,
            inputs.config,
            type_idx,
            leave,
            inputs.flows,
        ),
        WorldAction::Move(dir) => apply_move(
            id,
            &mut inputs.creatures[id],
            inputs.world,
            dir,
            inputs.config,
            inputs.flows,
        ),
        WorldAction::Reproduce {
            direction,
            energy_transfer_fraction,
        } => {
            let (result, conception) =
                apply_reproduce(inputs, id, direction, energy_transfer_fraction);
            conceived = conception;
            result
        }
        WorldAction::StealEnergy { direction, effort } => {
            let result = apply_steal_energy(inputs, id, direction, effort);
            counterpart = neighbour_creature(inputs, id, direction);
            result
        }
        WorldAction::GiveEnergy { direction, effort } => {
            let result = apply_give_energy(inputs, id, direction, effort);
            counterpart = neighbour_creature(inputs, id, direction);
            result
        }
    };
    let (energy_after, actor_removed) =
        settle(inputs, id, action.action_type(), &result, charged_before);
    Applied {
        result,
        energy_after,
        actor_removed,
        counterpart,
        conceived,
    }
}

/// Book a failed attempt's effort as failed effort (the one site), then
/// floor the actor at `0.0`: the action's own overdraft nets out of its
/// charge, only a deficit it inherited is a floor credit. Returns the energy
/// before the floor and whether the actor is at or below zero after it.
fn settle(
    inputs: &mut ActionInputs<'_>,
    id: CreatureId,
    kind: ActionType,
    result: &ActionResult,
    charged_before: (ActionCharges, FailedActionEffort),
) -> (f32, bool) {
    let flows = &mut *inputs.flows;
    if !result.outcome.succeeded() {
        flows.failed_action_effort[kind] += result.attempt_effort;
    }
    let creature = &mut inputs.creatures[id];
    let before = creature.energy;
    creature.energy = creature.energy.max(0.0);
    let overshoot = applied_debit(creature.energy, before);
    flows.zero_floor_credit += flows.net_action_overdraft(charged_before, overshoot);
    (before, creature.energy <= 0.0)
}

#[cfg(test)]
#[path = "transaction_tests.rs"]
mod tests;
