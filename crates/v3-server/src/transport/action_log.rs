//! The server-owned action-log entry (T30.F01 invariant 8, G22): today's
//! fields and variant names, built from `creature::action_log`. T30.F06
//! derives it from the applied action result.

use serde::Serialize;
use v3_core::contracts::{ActionType as CoreActionType, OrdinaryFoodTypeId};
use v3_core::creature::action_log::{ActionLogEntry as CoreEntry, ActionResult as CoreResult};

/// An action's kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub enum ActionType {
    NoOp,
    Eat,
    Move,
    Reproduce,
    StealEnergy,
    GiveEnergy,
}

impl From<CoreActionType> for ActionType {
    fn from(action: CoreActionType) -> Self {
        match action {
            CoreActionType::NoOp => Self::NoOp,
            CoreActionType::Eat => Self::Eat,
            CoreActionType::Move => Self::Move,
            CoreActionType::Reproduce => Self::Reproduce,
            CoreActionType::StealEnergy => Self::StealEnergy,
            CoreActionType::GiveEnergy => Self::GiveEnergy,
        }
    }
}

/// An action's result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub enum ActionResult {
    Success,
    NoFood,
    Blocked,
    InvalidTarget,
    EnergyConstraints,
    PopulationCap,
    TransferredAndKilled,
    NoVictim,
    AgeConstraints,
    NoRecipient,
}

impl From<CoreResult> for ActionResult {
    fn from(result: CoreResult) -> Self {
        match result {
            CoreResult::Success => Self::Success,
            CoreResult::NoFood => Self::NoFood,
            CoreResult::Blocked => Self::Blocked,
            CoreResult::InvalidTarget => Self::InvalidTarget,
            CoreResult::EnergyConstraints => Self::EnergyConstraints,
            CoreResult::PopulationCap => Self::PopulationCap,
            CoreResult::TransferredAndKilled => Self::TransferredAndKilled,
            CoreResult::NoVictim => Self::NoVictim,
            CoreResult::AgeConstraints => Self::AgeConstraints,
            CoreResult::NoRecipient => Self::NoRecipient,
        }
    }
}

/// One action-log record.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct ActionLogEntry {
    pub tick: u64,
    pub action_type: ActionType,
    pub result: ActionResult,
    /// Direction index `0..8`, or `NO_DIRECTION` when the action has none.
    pub direction: u8,
    pub energy_before: f32,
    pub energy_after: f32,
    /// Food eaten, energy taken or given, or the transfer fraction of a
    /// reproduction; 0 for a move or a no-op.
    pub amount: f32,
    /// The food type an Eat selected; `null` for every other action.
    pub food_type: Option<OrdinaryFoodTypeId>,
    pub priority_bid: f32,
}

impl From<&CoreEntry> for ActionLogEntry {
    fn from(entry: &CoreEntry) -> Self {
        Self {
            tick: entry.tick,
            action_type: entry.action_type.into(),
            result: entry.result.into(),
            direction: entry.direction,
            energy_before: entry.energy_before,
            energy_after: entry.energy_after,
            amount: entry.amount,
            food_type: entry.food_type,
            priority_bid: entry.priority_bid,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every core variant has a wire variant of the same JSON name, and the
    /// wire enums have no other variant.
    #[test]
    fn action_type_and_result_are_pinned_to_the_core_enums() {
        for action in CoreActionType::ALL {
            assert_eq!(
                serde_json::to_value(action).unwrap(),
                serde_json::to_value(ActionType::from(action)).unwrap()
            );
        }
        assert_eq!(
            <ActionType as ts_rs::TS>::decl(&ts_rs::Config::new())
                .matches('"')
                .count()
                / 2,
            CoreActionType::ALL.len()
        );
        for result in CoreResult::ALL {
            assert_eq!(
                result.as_key(),
                serde_json::to_value(ActionResult::from(result)).unwrap()
            );
        }
        assert_eq!(
            <ActionResult as ts_rs::TS>::decl(&ts_rs::Config::new())
                .matches('"')
                .count()
                / 2,
            CoreResult::ALL.len()
        );
    }
}
