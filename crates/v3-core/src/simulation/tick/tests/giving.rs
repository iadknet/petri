//! T27.F03 invariant 6, the give column: a committed give moves
//! `min(1, give_share × effort) × giver energy` from the giver to the
//! creature on the target cell through the ledger.

use super::support::*;
use crate::contracts::{ActionType, CreatureId, Direction, WorldAction};
use crate::creature::action_log::{ActionLog, ActionLogEntry, ActionResult};
use crate::simulation::actions::{apply_give_energy, GiveActionResult};
use crate::simulation::energy_accounting::{applied_debit, DeathCause};
use crate::simulation::Simulation;

/// A give north at effort modifier `effort`.
const fn give_north(effort: f32) -> WorldAction {
    WorldAction::GiveEnergy {
        direction: Direction::N,
        effort,
    }
}

/// Run `actions` for `id` with a fresh action log and return its entries.
fn logged(sim: &mut Simulation, id: CreatureId, actions: Vec<WorldAction>) -> Vec<ActionLogEntry> {
    sim.action_logs.insert(id, ActionLog::new(8));
    run_actions(sim, id, actions);
    sim.action_logs
        .get(id)
        .map(|log| log.entries().iter().copied().collect())
        .unwrap_or_default()
}

/// The effort creature `id` pays for an exchange act at `intensity`:
/// `effort_rate × intensity × energy` floored at `NoOp`'s, adjusted for its
/// complexity and age.
fn exchange_effort(sim: &Simulation, id: CreatureId, intensity: f32) -> f32 {
    let creature = &sim.creatures[id];
    let base = sim.config.exchange.effort_rate * intensity * creature.energy;
    sim.config
        .energy
        .action_effort(base, creature.cached_complexity, creature.age)
}

#[test]
fn a_give_moves_the_share_of_the_giver_store_and_books_each_side() {
    let (mut sim, giver, recipient) = make_sim_two_creatures(50.0, 30.0);
    // Intensity 0.2 (the founder share at modifier 1): base 0.05 × 0.2 × 50.
    let effort = exchange_effort(&sim, giver, 0.2);
    let after_effort = 50.0f32 - effort;
    let gift = 0.2f32 * after_effort;

    let entries = logged(&mut sim, giver, vec![give_north(1.0)]);

    assert_eq!(sim.creatures[giver].energy, after_effort - gift);
    assert_eq!(sim.creatures[recipient].energy, 30.0 + gift);
    let flows = &sim.stats.energy_flows;
    assert_eq!(
        flows.gift_debit,
        applied_debit(after_effort, after_effort - gift)
    );
    assert_eq!(flows.gift_credit, applied_debit(30.0 + gift, 30.0));
    assert_eq!(flows.maximum_energy_clamp_loss, 0.0);
    assert_eq!(
        flows.action_charges[ActionType::GiveEnergy],
        applied_debit(50.0, after_effort)
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
    assert_eq!(entries[0].amount, gift);
    assert_eq!(entries[0].direction, Direction::N.to_index() as u8);
}

#[test]
fn the_give_intensity_scales_the_share_by_the_effort_up_to_one() {
    for (share, effort, intensity) in [(0.2, 2.0, 0.4), (0.5, 0.25, 0.125), (0.5, 4.0, 1.0)] {
        let (mut sim, giver, recipient) = make_sim_two_creatures(50.0, 30.0);
        sim.creatures[giver].body.give_share = share;
        let after_effort = 50.0f32 - exchange_effort(&sim, giver, intensity);
        let outcome = apply_give_energy(giver, &mut sim, Direction::N, effort);
        assert_eq!(outcome.result, GiveActionResult::Transferred);
        assert_eq!(
            outcome.amount,
            intensity * after_effort,
            "{share} × {effort}"
        );
        assert_eq!(outcome.counterpart, Some(recipient));
    }
}

/// A recipient at its cap still receives a `Transferred` gift; its credit is
/// booked, then the excess is clamp loss.
#[test]
fn a_gift_to_a_capped_recipient_is_transferred_and_the_excess_is_clamp_loss() {
    let (mut sim, giver, recipient) = make_sim_two_creatures(50.0, 0.0);
    let cap = sim.creatures[recipient].energy_cap();
    sim.creatures[recipient].energy = cap;
    let after_effort = 50.0f32 - exchange_effort(&sim, giver, 0.2);
    let gift = 0.2f32 * after_effort;

    let entries = logged(&mut sim, giver, vec![give_north(1.0)]);

    assert_eq!(entries[0].result, ActionResult::Success);
    assert_eq!(sim.creatures[recipient].energy, cap);
    let flows = &sim.stats.energy_flows;
    assert_eq!(flows.gift_credit, applied_debit(cap + gift, cap));
    assert_eq!(flows.maximum_energy_clamp_loss, flows.gift_credit);
}

#[test]
fn a_give_with_no_recipient_logs_no_recipient_and_pays_only_its_effort() {
    let (mut sim, giver) = make_sim_with_one_creature(50.0);
    let effort = exchange_effort(&sim, giver, 0.2);

    let entries = logged(&mut sim, giver, vec![give_north(1.0)]);

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
/// sub-resolution amount (`share_min` against the smallest positive store,
/// with no effort to pay) keeps the giver alive and its log; a giver its
/// effort exhausted is removed with its log, its death its effort's.
#[test]
fn a_give_with_nothing_to_give_logs_energy_constraints() {
    let tiny = f32::from_bits(1);
    let (mut sim, giver, recipient) = make_sim_two_creatures(tiny, 30.0);
    sim.config.exchange.effort_rate = 0.0;
    sim.config.energy.costs.noop_cost = 0.0; // no T16.F02 effort floor
    sim.creatures[giver].body.give_share = sim.config.exchange.share_min;

    let entries = logged(&mut sim, giver, vec![give_north(1.0)]);

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].result, ActionResult::EnergyConstraints);
    assert_eq!(entries[0].amount, 0.0);
    assert_eq!(sim.creatures[giver].energy, tiny);
    assert_eq!(sim.creatures[recipient].energy, 30.0);
    let flows = &sim.stats.energy_flows;
    assert_eq!(flows.gift_debit, 0.0);
    assert_eq!(flows.gift_credit, 0.0);
    assert_eq!(
        sim.stats
            .give_actions_by_result
            .get(&GiveActionResult::RejectedNothingToGive),
        Some(&1)
    );

    let (mut sim, giver, recipient) = make_sim_two_creatures(0.1, 30.0);
    run_actions(&mut sim, giver, vec![give_north(1.0)]);
    assert!(!sim.creatures.contains_key(giver));
    assert_eq!(sim.creatures[recipient].energy, 30.0);
    assert_eq!(sim.stats.mortality.count(DeathCause::ActionGiveEnergy), 1);
    let flows = &sim.stats.energy_flows;
    assert_eq!(
        flows.failed_action_effort[ActionType::GiveEnergy],
        flows.action_charges[ActionType::GiveEnergy]
    );
}

/// A giver may exhaust itself giving: at intensity 1 it passes its whole
/// post-effort store and is removed right after the dispatch, its remaining
/// queue dropped and its cell freed this tick, with pending cause `Gift`.
#[test]
fn a_giver_exhausted_by_its_gift_is_removed_after_its_dispatch() {
    let (mut sim, giver, recipient) = make_sim_two_creatures(5.0, 30.0);
    sim.creatures[giver].body.give_share = 1.0;
    let giver_cell = sim.creatures[giver].position;
    let after_effort = 5.0f32 - exchange_effort(&sim, giver, 1.0);

    run_actions(
        &mut sim,
        giver,
        vec![give_north(1.0), WorldAction::Move(Direction::S)],
    );

    assert_eq!(sim.stats.give_actions_transferred_total, 1);
    assert_eq!(
        sim.stats.last_tick_move, 0,
        "the queue after the gift is dropped"
    );
    assert!(!sim.creatures.contains_key(giver));
    assert!(sim.world.creature_at(giver_cell).is_none());
    assert_eq!(sim.stats.mortality.count(DeathCause::Gift), 1);
    assert_eq!(sim.creatures[recipient].energy, 30.0 + after_effort);
    // Each side's applied change; the pair agrees up to rounding.
    let flows = &sim.stats.energy_flows;
    assert_eq!(flows.gift_debit, f64::from(after_effort));
    assert_eq!(flows.gift_credit, applied_debit(30.0 + after_effort, 30.0));
}

proptest::proptest! {
    /// A give is `Transferred` only when it moved a positive amount that
    /// lowered the giver's stored energy, for any `f32` bit pattern of the
    /// giver's store and the effort, and never credits a rejected give.
    #[test]
    fn a_gift_always_lowers_the_giver_store(
        store_bits in proptest::num::u32::ANY,
        effort_bits in proptest::num::u32::ANY,
    ) {
        let store = f32::from_bits(store_bits);
        let (mut sim, giver, recipient) = make_sim_two_creatures(50.0, 30.0);
        sim.config.exchange.effort_rate = 0.0;
        sim.config.energy.costs.noop_cost = 0.0; // no T16.F02 effort floor
        sim.creatures[giver].energy = store;

        let outcome = apply_give_energy(giver, &mut sim, Direction::N, f32::from_bits(effort_bits));

        match outcome.result {
            GiveActionResult::Transferred => {
                proptest::prop_assert!(outcome.amount > 0.0);
                proptest::prop_assert!(sim.creatures[giver].energy < store);
            }
            GiveActionResult::RejectedNothingToGive => {
                proptest::prop_assert_eq!(outcome.amount, 0.0);
                proptest::prop_assert_eq!(sim.creatures[recipient].energy, 30.0);
                proptest::prop_assert_eq!(sim.stats.energy_flows.gift_credit, 0.0);
            }
            GiveActionResult::RejectedNoRecipient => {
                proptest::prop_assert!(false, "the recipient is adjacent");
            }
        }
    }
}
