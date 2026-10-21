//! T27.F03 invariant 5, the give column: a committed give moves one bite
//! from the giver to the creature on the target cell through the ledger.

use super::support::*;
use crate::contracts::{ActionType, CreatureId, Direction, WorldAction};
use crate::creature::action_log::{ActionLog, ActionLogEntry, ActionResult};
use crate::simulation::actions::GiveActionResult;
use crate::simulation::energy_accounting::{applied_debit, DeathCause};
use crate::simulation::Simulation;

const GIVE_NORTH: WorldAction = WorldAction::GiveEnergy {
    direction: Direction::N,
};

/// Run `actions` for `id` with a fresh action log and return its entries.
fn logged(sim: &mut Simulation, id: CreatureId, actions: Vec<WorldAction>) -> Vec<ActionLogEntry> {
    sim.action_logs.insert(id, ActionLog::new(8));
    run_actions(sim, id, actions);
    sim.action_logs
        .get(id)
        .map(|log| log.entries().iter().copied().collect())
        .unwrap_or_default()
}

/// The giver's effort: the `NoOp` floor, adjusted for its complexity and age.
fn floor(sim: &Simulation, id: CreatureId) -> f32 {
    let creature = &sim.creatures[id];
    sim.config.energy.adjusted_action_cost(
        sim.config.energy.costs.noop_cost,
        creature.cached_complexity,
        creature.age,
    )
}

#[test]
fn a_default_give_moves_one_bite_and_books_each_side() {
    let (mut sim, giver, recipient) = make_sim_two_creatures(50.0, 30.0);
    let effort = floor(&sim, giver);

    let entries = logged(&mut sim, giver, vec![GIVE_NORTH]);

    assert_eq!(sim.creatures[recipient].energy, 38.0);
    assert_eq!(
        sim.creatures[giver].energy.to_bits(),
        ((50.0f32 - effort) - 8.0).to_bits()
    );
    let flows = &sim.stats.energy_flows;
    assert_eq!(flows.gift_debit, 8.0);
    assert_eq!(flows.gift_credit, 8.0);
    assert_eq!(flows.maximum_energy_clamp_loss, 0.0);
    assert_eq!(
        flows.action_charges[ActionType::GiveEnergy],
        applied_debit(50.0, 50.0 - effort)
    );
    assert_eq!(flows.failed_action_effort[ActionType::GiveEnergy], 0.0);
    assert_eq!(sim.stats.give_actions_attempted_total, 1);
    assert_eq!(sim.stats.give_actions_transferred_total, 1);
    assert_eq!(sim.stats.give_actions_rejected_total, 0);
    assert_eq!(
        sim.stats
            .give_actions_by_result
            .get(&GiveActionResult::Transferred),
        Some(&1)
    );
    assert_eq!(sim.stats.last_tick_give, 1);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].action_type, ActionType::GiveEnergy);
    assert_eq!(entries[0].result, ActionResult::Success);
    assert_eq!(entries[0].amount, 8.0);
    assert_eq!(entries[0].direction, Direction::N.to_index() as u8);
}

/// A recipient at its cap still receives a `Transferred` gift; its credit is
/// booked, then the excess is clamp loss.
#[test]
fn a_gift_to_a_capped_recipient_is_transferred_and_the_excess_is_clamp_loss() {
    let (mut sim, giver, recipient) = make_sim_two_creatures(50.0, 0.0);
    let cap = sim.creatures[recipient].energy_cap(&sim.config.energy.lifecycle);
    sim.creatures[recipient].energy = cap;

    let entries = logged(&mut sim, giver, vec![GIVE_NORTH]);

    assert_eq!(entries[0].result, ActionResult::Success);
    assert_eq!(sim.creatures[recipient].energy, cap);
    let flows = &sim.stats.energy_flows;
    assert_eq!(flows.gift_debit, 8.0);
    assert_eq!(flows.gift_credit, 8.0);
    assert_eq!(flows.maximum_energy_clamp_loss, 8.0);
}

#[test]
fn a_give_with_no_recipient_logs_no_recipient_and_pays_only_its_effort() {
    let (mut sim, giver) = make_sim_with_one_creature(50.0);
    let effort = floor(&sim, giver);

    let entries = logged(&mut sim, giver, vec![GIVE_NORTH]);

    assert_eq!(entries[0].result, ActionResult::NoRecipient);
    assert_eq!(entries[0].amount, 0.0);
    assert_eq!(sim.creatures[giver].energy, 50.0 - effort);
    let flows = &sim.stats.energy_flows;
    assert_eq!(flows.gift_debit, 0.0);
    assert_eq!(flows.gift_credit, 0.0);
    assert_eq!(
        flows.failed_action_effort[ActionType::GiveEnergy],
        flows.action_charges[ActionType::GiveEnergy]
    );
    assert_eq!(sim.stats.give_actions_rejected_total, 1);
    assert_eq!(
        sim.stats
            .give_actions_by_result
            .get(&GiveActionResult::RejectedNoRecipient),
        Some(&1)
    );
}

/// Nothing to give is `RejectedNothingToGive`, logged `EnergyConstraints`: a
/// giver its effort exhausted, or a bite its energy's `f32` resolution
/// absorbs. (An exhausted giver is removed with its log, so only the
/// surviving case reads the log entry.)
#[test]
fn a_give_with_nothing_to_give_logs_energy_constraints() {
    for (giver_energy, bite) in [(0.1, 8.0), (1.0e8, 1.0e-3)] {
        let (mut sim, giver, recipient) = make_sim_two_creatures(giver_energy, 30.0);
        sim.config.exchange.bite = bite;

        let entries = logged(&mut sim, giver, vec![GIVE_NORTH]);

        let label = format!("giver {giver_energy} bite {bite}");
        if let Some(entry) = entries.first() {
            assert_eq!(entry.result, ActionResult::EnergyConstraints, "{label}");
            assert_eq!(entry.amount, 0.0, "{label}");
        } else {
            assert!(!sim.creatures.contains_key(giver), "{label}");
        }
        assert_eq!(sim.creatures[recipient].energy, 30.0, "{label}");
        let flows = &sim.stats.energy_flows;
        assert_eq!(flows.gift_debit, 0.0, "{label}");
        assert_eq!(flows.gift_credit, 0.0, "{label}");
        assert_eq!(
            flows.failed_action_effort[ActionType::GiveEnergy],
            flows.action_charges[ActionType::GiveEnergy],
            "{label}"
        );
        assert_eq!(
            sim.stats
                .give_actions_by_result
                .get(&GiveActionResult::RejectedNothingToGive),
            Some(&1),
            "{label}"
        );
    }
    let (mut sim, giver, _recipient) = make_sim_two_creatures(1.0e8, 30.0);
    sim.config.exchange.bite = 1.0e-3;
    let entries = logged(&mut sim, giver, vec![GIVE_NORTH]);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].result, ActionResult::EnergyConstraints);
    // The exhausted giver's death is its effort's, not a gift's.
    let (mut sim, giver, _recipient) = make_sim_two_creatures(0.1, 30.0);
    run_actions(&mut sim, giver, vec![GIVE_NORTH]);
    assert!(!sim.creatures.contains_key(giver));
    assert_eq!(sim.stats.mortality.count(DeathCause::ActionGiveEnergy), 1);
}

/// A giver may exhaust itself giving: it is removed right after the
/// dispatch, its remaining queue dropped and its cell freed this tick, with
/// pending cause `Gift`.
#[test]
fn a_giver_exhausted_by_its_gift_is_removed_after_its_dispatch() {
    let (mut sim, giver, recipient) = make_sim_two_creatures(5.0, 30.0);
    let giver_cell = sim.creatures[giver].position;

    run_actions(
        &mut sim,
        giver,
        vec![GIVE_NORTH, WorldAction::Move(Direction::S)],
    );

    assert_eq!(sim.stats.give_actions_transferred_total, 1);
    assert_eq!(
        sim.stats.last_tick_move, 0,
        "the queue after the gift is dropped"
    );
    assert!(!sim.creatures.contains_key(giver));
    assert!(sim.world.creature_at(giver_cell).is_none());
    assert_eq!(sim.stats.mortality.count(DeathCause::Gift), 1);
    // The whole post-effort balance moved: each side's applied change.
    let flows = &sim.stats.energy_flows;
    let charged = flows.action_charges[ActionType::GiveEnergy];
    assert!((flows.gift_debit + charged - 5.0).abs() < 1e-5);
    assert_eq!(
        flows.gift_credit,
        f64::from(sim.creatures[recipient].energy) - 30.0
    );
}
