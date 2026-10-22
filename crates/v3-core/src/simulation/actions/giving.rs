//! The `GiveEnergy` action (T27.F03): trophallaxis and parental feeding, part
//! of the giver's own store passed to the neighbour in a chosen direction.

use super::TransferOutcome;
use crate::contracts::{ActionType, CreatureId, Direction};
use crate::simulation::energy_accounting::DeathCause;
use crate::simulation::simulation::Simulation;

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
pub fn apply_give_energy(
    giver_id: CreatureId,
    sim: &mut Simulation,
    dir: Direction,
    effort: f32,
) -> TransferOutcome<GiveActionResult> {
    sim.stats.give_actions_attempted_total += 1;
    sim.stats.last_tick_give += 1;

    let giver = &mut sim.creatures[giver_id];
    let intensity = super::exchange_intensity(giver.body.give_share, effort);
    let effort =
        super::pay_exchange_effort(giver, &sim.config, intensity, DeathCause::ActionGiveEnergy);
    sim.stats.energy_flows.action_charges[ActionType::GiveEnergy] += effort;

    let Some(recipient_id) = super::neighbour_creature(sim, giver_id, dir) else {
        return reject(sim, GiveActionResult::RejectedNoRecipient, effort, None);
    };

    // A gift moves only when the giver's stored energy falls.
    let before = sim.creatures[giver_id].energy;
    let Some(amount) = super::exchange_amount(intensity, before) else {
        return reject(
            sim,
            GiveActionResult::RejectedNothingToGive,
            effort,
            Some(recipient_id),
        );
    };

    let giver = &mut sim.creatures[giver_id];
    giver.energy -= amount;
    sim.stats.energy_flows.gift_debit += giver.observe_energy(before, DeathCause::Gift);
    let recipient = &mut sim.creatures[recipient_id];
    let recipient_before = recipient.energy;
    recipient.energy += amount;
    sim.stats.energy_flows.gift_credit -=
        recipient.observe_energy(recipient_before, DeathCause::Gift);
    sim.stats.energy_flows.maximum_energy_clamp_loss +=
        super::cap_energy(&mut sim.creatures[recipient_id]);

    sim.stats.give_actions_transferred_total += 1;
    *sim.stats
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
    sim: &mut Simulation,
    result: GiveActionResult,
    effort: f64,
    counterpart: Option<CreatureId>,
) -> TransferOutcome<GiveActionResult> {
    sim.stats.give_actions_rejected_total += 1;
    *sim.stats.give_actions_by_result.entry(result).or_insert(0) += 1;
    sim.stats.energy_flows.failed_action_effort[ActionType::GiveEnergy] += effort;
    TransferOutcome {
        result,
        amount: 0.0,
        counterpart,
    }
}
