//! The `GiveEnergy` action (T27.F03): trophallaxis and parental feeding, part
//! of the giver's own store passed to the neighbour in a chosen direction.

use super::ActionInputs;
use crate::contracts::{ActionOutcome, ActionResult, ActionType, CreatureId, Direction};
use crate::ledger::DeathCause;

/// Result of an attempted `GiveEnergy` action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GiveActionResult {
    /// The gift moved; the recipient was credited up to its cap.
    Transferred,
    /// No creature on the target cell (empty, barrier, or off-world).
    RejectedNoRecipient,
    /// The gift moved nothing: the giver held nothing after its effort, or
    /// the amount was below its energy's `f32` resolution.
    RejectedNothingToGive,
}

impl GiveActionResult {
    /// Every result, in declaration order.
    pub const ALL: [Self; 3] = [
        Self::Transferred,
        Self::RejectedNoRecipient,
        Self::RejectedNothingToGive,
    ];

    /// The run-total key of a give that ended in `outcome`; `None` for an
    /// outcome no give produces.
    #[must_use]
    pub const fn from_outcome(outcome: ActionOutcome) -> Option<Self> {
        match outcome {
            ActionOutcome::Success => Some(Self::Transferred),
            ActionOutcome::NoRecipient => Some(Self::RejectedNoRecipient),
            ActionOutcome::NothingToGive => Some(Self::RejectedNothingToGive),
            _ => None,
        }
    }

    /// Stable string key used by transport/API boundaries.
    #[must_use]
    pub const fn as_key(self) -> &'static str {
        match self {
            Self::Transferred => "Transferred",
            Self::RejectedNoRecipient => "RejectedNoRecipient",
            Self::RejectedNothingToGive => "RejectedNothingToGive",
        }
    }
}

/// Apply a GiveEnergy action: pass part of the giver's store to the
/// neighbour in `dir`.
///
/// The intensity is `i = min(1, give_share × effort)` with the giver's own
/// heritable share and the decoded modifier `effort`. The giver first pays
/// the effort of base `exchange.effort_rate × i × max(giver energy, 0)`,
/// floored at `NoOp`'s, whatever the outcome. The recipient is the creature
/// on the neighbour cell, never the giver; none is `NoRecipient`. The gift
/// is `i × giver.energy` after the effort when that energy is positive,
/// else nothing; a gift is `Success` only when the giver's stored energy
/// fell, else `NothingToGive`. Each side books its own applied change
/// (`gift_debit`, `gift_credit`); the recipient is capped, the excess booked
/// as clamp loss, and a recipient at its cap still receives the gift. A
/// giver left at or below zero is removed by the commit step, with pending
/// cause `Gift`. Returns the result with the recipient, `None` when the
/// target cell held none.
pub(crate) fn apply_give_energy(
    inputs: &mut ActionInputs<'_>,
    giver_id: CreatureId,
    dir: Direction,
    effort: f32,
) -> (ActionResult, Option<CreatureId>) {
    let giver = &mut inputs.creatures[giver_id];
    let intensity = super::exchange_intensity(giver.body.give_share, effort);
    let effort = super::pay_exchange_effort(
        giver,
        inputs.config,
        intensity,
        DeathCause::ActionGiveEnergy,
    );
    inputs.flows.action_charges[ActionType::GiveEnergy] += effort;

    let Some(recipient_id) = super::neighbour_creature(inputs, giver_id, dir) else {
        return (ActionResult::new(ActionOutcome::NoRecipient, effort), None);
    };

    // A gift moves only when the giver's stored energy falls.
    let before = inputs.creatures[giver_id].energy;
    let Some(amount) = super::exchange_amount(intensity, before) else {
        return (
            ActionResult::new(ActionOutcome::NothingToGive, effort),
            Some(recipient_id),
        );
    };

    let giver = &mut inputs.creatures[giver_id];
    giver.energy -= amount;
    inputs.flows.gift_debit += giver.observe_energy(before, DeathCause::Gift);
    let recipient = &mut inputs.creatures[recipient_id];
    let recipient_before = recipient.energy;
    recipient.energy += amount;
    inputs.flows.gift_credit -= recipient.observe_energy(recipient_before, DeathCause::Gift);
    inputs.flows.maximum_energy_clamp_loss +=
        super::cap_energy(&mut inputs.creatures[recipient_id]);
    (
        ActionResult::new(ActionOutcome::Success, effort).with_amount(amount),
        Some(recipient_id),
    )
}
