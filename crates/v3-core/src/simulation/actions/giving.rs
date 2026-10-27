//! The `GiveEnergy` action (T27.F03): trophallaxis and parental feeding, part
//! of the giver's own store passed to the neighbour in a chosen direction.

use super::ActionInputs;
use super::TransferOutcome;
use crate::contracts::{ActionType, CreatureId, Direction};
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
/// on the neighbour cell, never the giver; none is `RejectedNoRecipient`.
/// The gift is `i × giver.energy` after the effort when that energy is
/// positive, else nothing; a gift is `Transferred` only when the giver's
/// stored energy fell, else `RejectedNothingToGive`. Each side books its own
/// applied change (`gift_debit`, `gift_credit`); the recipient is capped,
/// the excess booked as clamp loss, and a recipient at its cap still
/// receives a `Transferred` gift. A giver left at or below zero is removed
/// by the tick's post-action removal, with pending cause `Gift`.
pub(crate) fn apply_give_energy(
    inputs: &mut ActionInputs<'_>,
    giver_id: CreatureId,
    dir: Direction,
    effort: f32,
) -> TransferOutcome<GiveActionResult> {
    inputs.stats.give_actions_attempted_total += 1;
    inputs.stats.last_tick_give += 1;

    let giver = &mut inputs.creatures[giver_id];
    let intensity = super::exchange_intensity(giver.body.give_share, effort);
    let effort = super::pay_exchange_effort(
        giver,
        inputs.config,
        intensity,
        DeathCause::ActionGiveEnergy,
    );
    inputs.stats.energy_flows.action_charges[ActionType::GiveEnergy] += effort;

    let Some(recipient_id) = super::neighbour_creature(inputs, giver_id, dir) else {
        return reject(inputs, GiveActionResult::RejectedNoRecipient, effort, None);
    };

    // A gift moves only when the giver's stored energy falls.
    let before = inputs.creatures[giver_id].energy;
    let Some(amount) = super::exchange_amount(intensity, before) else {
        return reject(
            inputs,
            GiveActionResult::RejectedNothingToGive,
            effort,
            Some(recipient_id),
        );
    };

    let giver = &mut inputs.creatures[giver_id];
    giver.energy -= amount;
    inputs.stats.energy_flows.gift_debit += giver.observe_energy(before, DeathCause::Gift);
    let recipient = &mut inputs.creatures[recipient_id];
    let recipient_before = recipient.energy;
    recipient.energy += amount;
    inputs.stats.energy_flows.gift_credit -=
        recipient.observe_energy(recipient_before, DeathCause::Gift);
    inputs.stats.energy_flows.maximum_energy_clamp_loss +=
        super::cap_energy(&mut inputs.creatures[recipient_id]);

    inputs.stats.give_actions_transferred_total += 1;
    *inputs
        .stats
        .give_actions_by_result
        .entry(GiveActionResult::Transferred)
        .or_insert(0) += 1;
    TransferOutcome {
        result: GiveActionResult::Transferred,
        amount,
        counterpart: Some(recipient_id),
    }
}

/// Books a give that moved nothing: one rejection by `result`, its effort as
/// failed.
fn reject(
    inputs: &mut ActionInputs<'_>,
    result: GiveActionResult,
    effort: f64,
    counterpart: Option<CreatureId>,
) -> TransferOutcome<GiveActionResult> {
    inputs.stats.give_actions_rejected_total += 1;
    *inputs
        .stats
        .give_actions_by_result
        .entry(result)
        .or_insert(0) += 1;
    inputs.stats.energy_flows.failed_action_effort[ActionType::GiveEnergy] += effort;
    TransferOutcome {
        result,
        amount: 0.0,
        counterpart,
    }
}
