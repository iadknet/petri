//! The `GiveEnergy` action (T27.F03): trophallaxis and parental feeding, one
//! exchange bite passed to the neighbour in a chosen direction.

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
    /// The giver held nothing to give after its effort, or the bite was
    /// below its energy's `f32` resolution.
    RejectedNothingToGive,
}

impl GiveActionResult {
    /// Every result, in report order.
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

/// Apply a GiveEnergy action: pass one bite of energy to the neighbour in
/// `dir`.
///
/// The giver pays the `NoOp` effort floor first, whatever the outcome. The
/// recipient is the creature on the neighbour cell, never the giver; none is
/// `RejectedNoRecipient`. The gift is `min(bite, giver.energy)` after the
/// effort when the giver's energy is positive, else nothing; a gift is
/// `Transferred` only when the giver's stored energy changed, else
/// `RejectedNothingToGive`. Each side books its own applied change
/// (`gift_debit`, `gift_credit`); the recipient is capped, the excess booked
/// as clamp loss. A giver left at or below zero is removed by the tick's
/// post-action removal, with pending cause `Gift`.
pub fn apply_give_energy(
    giver_id: CreatureId,
    sim: &mut Simulation,
    dir: Direction,
) -> TransferOutcome<GiveActionResult> {
    let bite = sim.config.exchange.bite;
    sim.stats.give_actions_attempted_total += 1;
    sim.stats.last_tick_give += 1;

    let effort = super::pay_effort(
        &mut sim.creatures[giver_id],
        &sim.config.energy,
        sim.config.energy.costs.noop_cost,
        DeathCause::ActionGiveEnergy,
    );
    sim.stats.energy_flows.action_charges[ActionType::GiveEnergy] += effort;

    let giver_pos = sim.creatures[giver_id].position;
    let recipient_id = sim
        .world
        .resolve_neighbor(giver_pos, dir)
        .and_then(|target_pos| sim.world.creature_at(target_pos))
        .filter(|&id| id != giver_id);
    let Some(recipient_id) = recipient_id else {
        return reject(sim, effort, GiveActionResult::RejectedNoRecipient, None);
    };

    let before = sim.creatures[giver_id].energy;
    let amount = if before > 0.0 { bite.min(before) } else { 0.0 };
    let after = before - amount;
    if after.to_bits() == before.to_bits() {
        return reject(
            sim,
            effort,
            GiveActionResult::RejectedNothingToGive,
            Some(recipient_id),
        );
    }

    let giver = &mut sim.creatures[giver_id];
    giver.energy = after;
    sim.stats.energy_flows.gift_debit += giver.observe_energy(before, DeathCause::Gift);
    let recipient = &mut sim.creatures[recipient_id];
    let recipient_before = recipient.energy;
    recipient.energy += amount;
    sim.stats.energy_flows.gift_credit -=
        recipient.observe_energy(recipient_before, DeathCause::Gift);
    super::cap_energy(
        &mut sim.creatures[recipient_id],
        &sim.config.energy.lifecycle,
        &mut sim.stats.energy_flows,
    );

    sim.stats.give_actions_transferred_total += 1;
    *sim.stats
        .give_actions_by_result
        .entry(GiveActionResult::Transferred)
        .or_insert(0) += 1;
    TransferOutcome {
        result: GiveActionResult::Transferred,
        amount: before - after,
        counterpart: Some(recipient_id),
    }
}

/// Book a failed give: its effort is failed effort and nothing moves.
fn reject(
    sim: &mut Simulation,
    effort: f64,
    result: GiveActionResult,
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
