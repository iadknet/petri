use crate::contracts::{CreatureId, Direction};
use crate::creature::action_log::ActionType;
use crate::simulation::simulation::Simulation;

/// Spatial record of a single predation event for frontend visualization.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PredationEventRecord {
    pub attacker_x: u16,
    pub attacker_y: u16,
    pub victim_x: u16,
    pub victim_y: u16,
    pub energy_stolen: f32,
    pub killed: bool,
}

/// Result of an attempted predation (StealEnergy) action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PredationActionResult {
    /// Energy was stolen; victim survived.
    Transferred,
    /// Victim drained to zero or below; killed and removed.
    TransferredAndKilled,
    /// No victim at target (empty cell, barrier, or off-world).
    RejectedNoVictim,
}

impl PredationActionResult {
    /// Stable string key used by transport/API boundaries.
    #[must_use]
    pub const fn as_key(self) -> &'static str {
        match self {
            Self::Transferred => "Transferred",
            Self::TransferredAndKilled => "TransferredAndKilled",
            Self::RejectedNoVictim => "RejectedNoVictim",
        }
    }
}

/// What a transfer action moved: its result, the energy debited from the
/// side that gives it up, and the counterpart creature, `None` when the target
/// cell held none.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransferOutcome<R> {
    pub result: R,
    pub amount: f32,
    pub counterpart: Option<CreatureId>,
}

/// Apply a StealEnergy action: take one bite of energy from the neighbour in
/// `dir` (T27.F03).
///
/// The attacker always pays the effort of `steal_cost_rate × exchange.bite`,
/// floored at `NoOp`'s (T16.F02), regardless of outcome, and nothing more.
/// The transfer is `min(bite, victim.energy)` when the victim's energy is
/// positive, else nothing (a comparison, so a `NaN` energy transfers
/// nothing). A steal is `Transferred` only when the victim's stored energy
/// changed; no victim, an exhausted victim or a bite the victim's `f32`
/// resolution absorbs is `RejectedNoVictim`. Each side books its own applied
/// change. If the victim's energy drops to zero or below, the victim is
/// killed and removed, and the attacker receives a complexity bonus.
pub fn apply_steal_energy(
    attacker_id: CreatureId,
    sim: &mut Simulation,
    dir: Direction,
) -> TransferOutcome<PredationActionResult> {
    use crate::simulation::energy_accounting::DeathCause;
    let bite = sim.config.exchange.bite;

    // Step 1: Stats — count attempt and per-tick.
    sim.stats.predation_actions_attempted_total += 1;
    sim.stats.last_tick_steal += 1;

    // Step 2: Deduct the effort (based on the bite, not the transfer;
    // floored at NoOp's; scaled by genome complexity and age).
    let effort = super::pay_effort(
        &mut sim.creatures[attacker_id],
        &sim.config.energy,
        sim.config.predation.steal_cost_rate * bite,
        DeathCause::ActionStealEnergy,
    );
    sim.stats.energy_flows.action_charges[ActionType::StealEnergy] += effort;

    // Steps 3 and 4: resolve the target cell and its victim, guarding
    // against self-predation in degenerate worlds.
    let attacker_pos = sim.creatures[attacker_id].position;
    let victim_id = sim
        .world
        .resolve_neighbor(attacker_pos, dir)
        .and_then(|target_pos| sim.world.creature_at(target_pos))
        .filter(|&id| id != attacker_id);

    // Step 5: the bite, capped at what the victim holds; the debit applies
    // only when it changes the victim's stored energy.
    let debit = victim_id.and_then(|victim_id| {
        let before = sim.creatures[victim_id].energy;
        let amount = if before > 0.0 { bite.min(before) } else { 0.0 };
        let after = before - amount;
        (after.to_bits() != before.to_bits()).then_some((victim_id, amount, before, after))
    });
    let Some((victim_id, amount, before, after)) = debit else {
        sim.stats.predation_actions_rejected_total += 1;
        *sim.stats
            .predation_actions_by_result
            .entry(PredationActionResult::RejectedNoVictim)
            .or_insert(0) += 1;
        sim.stats.energy_flows.failed_action_effort[ActionType::StealEnergy] += effort;
        return TransferOutcome {
            result: PredationActionResult::RejectedNoVictim,
            amount: 0.0,
            counterpart: victim_id,
        };
    };
    let moved = before - after;

    // Step 6: Transfer energy; each side books its own applied change.
    let victim = &mut sim.creatures[victim_id];
    victim.energy = after;
    sim.stats.energy_flows.predation_victim_debit +=
        victim.observe_energy(before, DeathCause::Predation);
    #[cfg(feature = "telemetry-seams")]
    if sim.stats.observed_creature == Some(victim_id) {
        sim.stats.observed_damage += moved;
    }
    let attacker = &mut sim.creatures[attacker_id];
    let before = attacker.energy;
    attacker.energy += amount;
    sim.stats.energy_flows.predation_attacker_credit -=
        attacker.observe_energy(before, DeathCause::Predation);

    // Step 7: Cap attacker energy at its body's cap.
    super::cap_energy(
        &mut sim.creatures[attacker_id],
        &sim.config.energy.lifecycle,
        &mut sim.stats.energy_flows,
    );

    // Step 8: Check kill.
    if sim.creatures[victim_id].energy <= 0.0 {
        // Snapshot victim data before removal.
        let victim_complexity = sim.creatures[victim_id].cached_complexity;
        let victim_pos = sim.creatures[victim_id].position;

        // Compute and award complexity bonus.
        let bonus =
            victim_complexity as f32 * sim.config.predation.kill_complexity_bonus_multiplier;
        let attacker = &mut sim.creatures[attacker_id];
        let before = attacker.energy;
        attacker.energy += bonus;
        sim.stats.energy_flows.predation_kill_bonus_credit -=
            attacker.observe_energy(before, DeathCause::Predation);
        super::cap_energy(
            &mut sim.creatures[attacker_id],
            &sim.config.energy.lifecycle,
            &mut sim.stats.energy_flows,
        );

        // Remove victim from world, slotmap, and action logs.
        sim.remove_creature(victim_id);

        sim.creatures[attacker_id].lifetime_predation_kills_count += 1;
        sim.stats.predation_kills_total += 1;
        sim.stats.last_tick_predation_kills += 1;
        sim.stats.predation_actions_transferred_total += 1;
        *sim.stats
            .predation_actions_by_result
            .entry(PredationActionResult::TransferredAndKilled)
            .or_insert(0) += 1;
        sim.stats
            .last_tick_predation_events
            .push(PredationEventRecord {
                attacker_x: attacker_pos.x,
                attacker_y: attacker_pos.y,
                victim_x: victim_pos.x,
                victim_y: victim_pos.y,
                energy_stolen: moved,
                killed: true,
            });
        return TransferOutcome {
            result: PredationActionResult::TransferredAndKilled,
            amount: moved,
            counterpart: Some(victim_id),
        };
    }

    // Step 9: Victim survived.
    sim.creatures[victim_id].lifetime_predation_hits_taken_count += 1;
    sim.stats.predation_actions_transferred_total += 1;
    *sim.stats
        .predation_actions_by_result
        .entry(PredationActionResult::Transferred)
        .or_insert(0) += 1;
    let victim_pos = sim.creatures[victim_id].position;
    sim.stats
        .last_tick_predation_events
        .push(PredationEventRecord {
            attacker_x: attacker_pos.x,
            attacker_y: attacker_pos.y,
            victim_x: victim_pos.x,
            victim_y: victim_pos.y,
            energy_stolen: moved,
            killed: false,
        });
    TransferOutcome {
        result: PredationActionResult::Transferred,
        amount: moved,
        counterpart: Some(victim_id),
    }
}

/// A steal with the exchange bite set to `bite` first.
#[cfg(test)]
pub(crate) fn steal_with_bite(
    attacker_id: CreatureId,
    sim: &mut Simulation,
    dir: Direction,
    bite: f32,
) -> TransferOutcome<PredationActionResult> {
    sim.config.exchange.bite = bite;
    apply_steal_energy(attacker_id, sim, dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SimulationConfig;
    use crate::contracts::{ActionType, CreatureId, Position};
    use crate::creature::body::HeritableBody;
    use crate::creature::founder::v3alpha1_founder_genome;
    use crate::creature::identity::CreatureIdentityState;
    use crate::creature::phenotype::Phenotype;
    use crate::creature::state::CreatureState;
    use crate::kernel::WorldState;
    use crate::simulation::simulation::Simulation;
    use rand::SeedableRng;
    use slotmap::SlotMap;

    /// A founder-phenotype body with the default identity.
    fn founder_body() -> HeritableBody {
        HeritableBody {
            phenotype: Phenotype::FOUNDER,
            identity: CreatureIdentityState::default(),
        }
    }

    /// Create a simulation with two creatures: attacker at `pos_a` and victim at `pos_v`.
    fn make_sim_two_creatures(
        pos_a: Position,
        energy_a: f32,
        pos_v: Position,
        energy_v: f32,
    ) -> (Simulation, CreatureId, CreatureId) {
        let mut cfg = SimulationConfig::default();
        cfg.world.width = 10;
        cfg.world.height = 10;
        cfg.world.food.growth_rate = 0.0;

        let mut world = WorldState::new(cfg.world.width, cfg.world.height, cfg.world.edge_mode);
        let mut creatures: SlotMap<CreatureId, CreatureState> = SlotMap::with_key();

        let attacker_id = creatures.insert_with_key(|id| {
            CreatureState::new(
                id,
                v3alpha1_founder_genome(),
                pos_a,
                energy_a,
                0,
                founder_body(),
                [0.0; 16],
            )
        });
        world.place_creature(pos_a, attacker_id);

        let victim_id = creatures.insert_with_key(|id| {
            CreatureState::new(
                id,
                v3alpha1_founder_genome(),
                pos_v,
                energy_v,
                0,
                founder_body(),
                [0.0; 16],
            )
        });
        world.place_creature(pos_v, victim_id);

        let sim = Simulation {
            world,
            creatures,
            action_logs: slotmap::SecondaryMap::new(),
            tick: 0,
            config: cfg,
            stats: crate::simulation::stats::SimStats::default(),
            rng: rand::rngs::SmallRng::seed_from_u64(42),
            food_stream_key: crate::kernel::FoodStreamKey::from_run_seed(42),
        };
        (sim, attacker_id, victim_id)
    }

    /// Create a simulation with one creature (attacker only, no victim).
    fn make_sim_one_creature(pos: Position, energy: f32) -> (Simulation, CreatureId) {
        let mut cfg = SimulationConfig::default();
        cfg.world.width = 10;
        cfg.world.height = 10;
        cfg.world.food.growth_rate = 0.0;

        let mut world = WorldState::new(cfg.world.width, cfg.world.height, cfg.world.edge_mode);
        let mut creatures: SlotMap<CreatureId, CreatureState> = SlotMap::with_key();

        let id = creatures.insert_with_key(|id| {
            CreatureState::new(
                id,
                v3alpha1_founder_genome(),
                pos,
                energy,
                0,
                founder_body(),
                [0.0; 16],
            )
        });
        world.place_creature(pos, id);

        let sim = Simulation {
            world,
            creatures,
            action_logs: slotmap::SecondaryMap::new(),
            tick: 0,
            config: cfg,
            stats: crate::simulation::stats::SimStats::default(),
            rng: rand::rngs::SmallRng::seed_from_u64(42),
            food_stream_key: crate::kernel::FoodStreamKey::from_run_seed(42),
        };
        (sim, id)
    }

    #[test]
    fn steal_from_occupied_cell_transfers_energy() {
        // Attacker at (5,5), victim at (5,4) = N of attacker
        let (mut sim, attacker_id, victim_id) =
            make_sim_two_creatures(Position::new(5, 5), 50.0, Position::new(5, 4), 30.0);
        sim.config.predation.steal_cost_rate = 0.0; // only the NoOp floor (T16.F02)

        let result = steal_with_bite(attacker_id, &mut sim, Direction::N, 10.0).result;

        assert_eq!(result, PredationActionResult::Transferred);
        assert!(
            (sim.creatures[attacker_id].energy - (60.0 - 0.4)).abs() < 1e-5,
            "attacker should gain 10 less the 0.4 effort floor (effort-cost defaults, 2026-10-10)"
        );
        assert!(
            (sim.creatures[victim_id].energy - 20.0).abs() < 1e-6,
            "victim should lose 10"
        );
    }

    #[test]
    fn a_survived_transfer_counts_one_hit_on_the_victim_and_no_kill() {
        let (mut sim, attacker_id, victim_id) =
            make_sim_two_creatures(Position::new(5, 5), 50.0, Position::new(5, 4), 30.0);
        sim.config.predation.steal_cost_rate = 0.0;

        let result = steal_with_bite(attacker_id, &mut sim, Direction::N, 10.0).result;

        assert_eq!(result, PredationActionResult::Transferred);
        assert_eq!(
            sim.creatures[victim_id].lifetime_predation_hits_taken_count,
            1
        );
        assert_eq!(sim.creatures[attacker_id].lifetime_predation_kills_count, 0);
        assert_eq!(
            sim.creatures[attacker_id].lifetime_predation_hits_taken_count,
            0
        );
        assert_eq!(sim.creatures[victim_id].lifetime_predation_kills_count, 0);
    }

    #[test]
    fn a_kill_counts_on_the_attacker_beside_the_population_total() {
        let (mut sim, attacker_id, victim_id) =
            make_sim_two_creatures(Position::new(5, 5), 50.0, Position::new(5, 4), 5.0);
        sim.config.predation.steal_cost_rate = 0.0;

        let result = steal_with_bite(attacker_id, &mut sim, Direction::N, 20.0).result;

        assert_eq!(result, PredationActionResult::TransferredAndKilled);
        assert!(!sim.creatures.contains_key(victim_id));
        assert_eq!(sim.creatures[attacker_id].lifetime_predation_kills_count, 1);
        assert_eq!(
            sim.creatures[attacker_id].lifetime_predation_kills_count,
            sim.stats.predation_kills_total
        );
        assert_eq!(
            sim.creatures[attacker_id].lifetime_predation_hits_taken_count,
            0
        );
    }

    #[test]
    fn a_rejected_steal_counts_no_kill_and_no_hit() {
        let (mut sim, attacker_id) = make_sim_one_creature(Position::new(5, 5), 50.0);
        sim.config.predation.steal_cost_rate = 0.0;

        let result = steal_with_bite(attacker_id, &mut sim, Direction::N, 10.0).result;

        assert_eq!(result, PredationActionResult::RejectedNoVictim);
        assert_eq!(sim.creatures[attacker_id].lifetime_predation_kills_count, 0);
        assert_eq!(
            sim.creatures[attacker_id].lifetime_predation_hits_taken_count,
            0
        );
    }

    #[test]
    fn steal_cost_deducted_from_attacker() {
        let (mut sim, attacker_id, _victim_id) =
            make_sim_two_creatures(Position::new(5, 5), 50.0, Position::new(5, 4), 30.0);
        sim.config.predation.steal_cost_rate = 0.2;
        let adjusted_cost = sim.config.energy.adjusted_action_cost(
            0.2 * 10.0,
            sim.creatures[attacker_id].cached_complexity,
            sim.creatures[attacker_id].age,
        );

        let result = steal_with_bite(attacker_id, &mut sim, Direction::N, 10.0).result;

        assert_eq!(result, PredationActionResult::Transferred);
        // attacker: 50 - adjusted_cost + gain(10)
        let expected = 50.0 - adjusted_cost + 10.0;
        assert!(
            (sim.creatures[attacker_id].energy - expected).abs() < 1e-4,
            "attacker energy {} should be {}",
            sim.creatures[attacker_id].energy,
            expected
        );
    }

    #[test]
    fn cost_based_on_attempted_not_actual() {
        // Victim has less energy than requested
        let (mut sim, attacker_id, victim_id) =
            make_sim_two_creatures(Position::new(5, 5), 50.0, Position::new(5, 4), 5.0);
        sim.config.predation.steal_cost_rate = 0.2;
        let adjusted_cost = sim.config.energy.adjusted_action_cost(
            0.2 * 20.0,
            sim.creatures[attacker_id].cached_complexity,
            sim.creatures[attacker_id].age,
        );

        // Capture victim complexity before kill removes it from slotmap.
        let victim_complexity = sim.creatures[victim_id].cached_complexity;
        let result = steal_with_bite(attacker_id, &mut sim, Direction::N, 20.0).result;

        // Cost = adjusted(0.2 * 20 (attempted)) (not 0.2 * 5)
        // Actual transfer = min(20, 5) = 5.0
        // Victim killed: energy goes to 0 → TransferredAndKilled
        assert_eq!(result, PredationActionResult::TransferredAndKilled);
        // attacker: 50 - adjusted_cost + 5 (actual) + bonus
        let bonus =
            victim_complexity as f32 * sim.config.predation.kill_complexity_bonus_multiplier;
        let expected = 50.0 - adjusted_cost + 5.0 + bonus;
        assert!(
            (sim.creatures[attacker_id].energy - expected).abs() < 1e-4,
            "attacker energy {} should be ~{}, cost on attempted not actual",
            sim.creatures[attacker_id].energy,
            expected
        );
        // Victim should be removed
        assert!(!sim.creatures.contains_key(victim_id));
    }

    #[test]
    fn steal_from_empty_cell_still_costs_energy() {
        // Only attacker, no victim to the north
        let (mut sim, attacker_id) = make_sim_one_creature(Position::new(5, 5), 50.0);
        sim.config.predation.steal_cost_rate = 0.2;
        let adjusted_cost = sim.config.energy.adjusted_action_cost(
            0.2 * 10.0,
            sim.creatures[attacker_id].cached_complexity,
            sim.creatures[attacker_id].age,
        );

        let result = steal_with_bite(attacker_id, &mut sim, Direction::N, 10.0).result;

        assert_eq!(result, PredationActionResult::RejectedNoVictim);
        // Cost still deducted: 50 - adjusted_cost
        let expected = 50.0 - adjusted_cost;
        assert!(
            (sim.creatures[attacker_id].energy - expected).abs() < 1e-4,
            "cost should still be deducted: got {}",
            sim.creatures[attacker_id].energy
        );
        // The rejection is counted once, in the total and by result.
        assert_eq!(sim.stats.predation_actions_rejected_total, 1);
        assert_eq!(
            sim.stats
                .predation_actions_by_result
                .get(&PredationActionResult::RejectedNoVictim),
            Some(&1)
        );
    }

    #[test]
    fn steal_kills_victim_when_drained_to_zero() {
        let (mut sim, attacker_id, victim_id) =
            make_sim_two_creatures(Position::new(5, 5), 50.0, Position::new(5, 4), 10.0);
        sim.config.predation.steal_cost_rate = 0.0;

        // Request exactly victim's energy
        let result = steal_with_bite(attacker_id, &mut sim, Direction::N, 10.0).result;

        assert_eq!(result, PredationActionResult::TransferredAndKilled);
        assert!(
            !sim.creatures.contains_key(victim_id),
            "victim should be dead"
        );
    }

    #[test]
    fn kill_awards_complexity_bonus() {
        let (mut sim, attacker_id, victim_id) =
            make_sim_two_creatures(Position::new(5, 5), 50.0, Position::new(5, 4), 5.0);
        sim.config.predation.steal_cost_rate = 0.0;
        sim.config.predation.kill_complexity_bonus_multiplier = 1.0; // large multiplier for clarity

        let victim_complexity = sim.creatures[victim_id].cached_complexity;
        assert!(
            victim_complexity > 0,
            "founder genome should have nonzero complexity"
        );

        let result = steal_with_bite(attacker_id, &mut sim, Direction::N, 10.0).result;

        assert_eq!(result, PredationActionResult::TransferredAndKilled);
        // attacker: 50 − 0.4 (effort floor; effort-cost defaults, 2026-10-10) + 5 (actual steal) + complexity bonus
        let expected = 50.0 - 0.4 + 5.0 + victim_complexity as f32 * 1.0;
        assert!(
            (sim.creatures[attacker_id].energy - expected).abs() < 1e-4,
            "attacker energy {} should be {} (with complexity bonus)",
            sim.creatures[attacker_id].energy,
            expected
        );
    }

    #[test]
    fn killed_victim_removed_from_world() {
        let victim_pos = Position::new(5, 4);
        let (mut sim, attacker_id, _victim_id) =
            make_sim_two_creatures(Position::new(5, 5), 50.0, victim_pos, 5.0);
        sim.config.predation.steal_cost_rate = 0.0;

        let result = steal_with_bite(attacker_id, &mut sim, Direction::N, 10.0).result;

        assert_eq!(result, PredationActionResult::TransferredAndKilled);
        assert!(
            sim.world.creature_at(victim_pos).is_none(),
            "victim should be removed from world occupancy"
        );
    }

    #[test]
    fn attacker_energy_capped_at_max() {
        let (mut sim, attacker_id, _victim_id) =
            make_sim_two_creatures(Position::new(5, 5), 195.0, Position::new(5, 4), 50.0);
        sim.config.predation.steal_cost_rate = 0.0;
        let max_energy = sim.config.energy.lifecycle.max_energy; // 200.0

        let result = steal_with_bite(attacker_id, &mut sim, Direction::N, 20.0).result;

        assert_eq!(result, PredationActionResult::Transferred);
        assert!(
            sim.creatures[attacker_id].energy <= max_energy,
            "attacker energy {} should be capped at max {}",
            sim.creatures[attacker_id].energy,
            max_energy
        );
        assert!(
            (sim.creatures[attacker_id].energy - max_energy).abs() < 1e-6,
            "should be exactly at max"
        );
    }

    /// T27.F03 invariant 5: a steal whose transfer is zero is a failure: an
    /// exhausted or `NaN` victim, or a bite the victim's `f32` resolution
    /// absorbs. The effort is paid and booked as failed; no flow moves.
    #[test]
    fn a_zero_transfer_is_rejected_and_its_effort_is_failed_effort() {
        for (victim_energy, bite) in [(0.0, 8.0), (-3.0, 8.0), (f32::NAN, 8.0), (1.0e8, 1.0e-3)] {
            let (mut sim, attacker_id, victim_id) = make_sim_two_creatures(
                Position::new(5, 5),
                50.0,
                Position::new(5, 4),
                victim_energy,
            );
            let victim_before = sim.creatures[victim_id].energy.to_bits();

            let outcome = steal_with_bite(attacker_id, &mut sim, Direction::N, bite);

            let label = format!("victim {victim_energy} bite {bite}");
            assert_eq!(
                outcome.result,
                PredationActionResult::RejectedNoVictim,
                "{label}"
            );
            assert_eq!(outcome.amount, 0.0, "{label}");
            assert_eq!(outcome.counterpart, Some(victim_id), "{label}");
            assert_eq!(
                sim.creatures[victim_id].energy.to_bits(),
                victim_before,
                "{label}"
            );
            let flows = &sim.stats.energy_flows;
            assert!(
                flows.action_charges[ActionType::StealEnergy] > 0.0,
                "{label}"
            );
            assert_eq!(
                flows.failed_action_effort[ActionType::StealEnergy],
                flows.action_charges[ActionType::StealEnergy],
                "{label}"
            );
            assert_eq!(flows.predation_victim_debit, 0.0, "{label}");
            assert_eq!(flows.predation_attacker_credit, 0.0, "{label}");
            assert_eq!(sim.stats.predation_actions_rejected_total, 1, "{label}");
            assert_eq!(sim.stats.predation_actions_transferred_total, 0, "{label}");
            assert!(sim.stats.last_tick_predation_events.is_empty(), "{label}");
            assert_eq!(
                sim.creatures[victim_id].lifetime_predation_hits_taken_count, 0,
                "{label}"
            );
        }
    }

    /// T27.F03: at the defaults a steal takes one bite of 8.0, pays the
    /// effort `steal_cost_rate × bite` (the floor), and books the pair of
    /// applied changes.
    #[test]
    fn a_default_steal_takes_one_bite_through_the_ledger() {
        let (mut sim, attacker_id, victim_id) =
            make_sim_two_creatures(Position::new(5, 5), 50.0, Position::new(5, 4), 30.0);
        assert_eq!(sim.config.exchange.bite, 8.0);
        let effort = sim.config.energy.adjusted_action_cost(
            sim.config.predation.steal_cost_rate * 8.0,
            sim.creatures[attacker_id].cached_complexity,
            sim.creatures[attacker_id].age,
        );

        let outcome = apply_steal_energy(attacker_id, &mut sim, Direction::N);

        assert_eq!(outcome.result, PredationActionResult::Transferred);
        assert_eq!(outcome.amount, 8.0);
        assert_eq!(outcome.counterpart, Some(victim_id));
        assert_eq!(sim.creatures[victim_id].energy, 22.0);
        assert_eq!(
            sim.creatures[attacker_id].energy.to_bits(),
            ((50.0f32 - effort) + 8.0).to_bits()
        );
        let flows = &sim.stats.energy_flows;
        assert_eq!(flows.predation_victim_debit, 8.0);
        assert_eq!(flows.predation_attacker_credit, 8.0);
        assert_eq!(flows.failed_action_effort[ActionType::StealEnergy], 0.0);
        assert_eq!(sim.stats.last_tick_predation_events[0].energy_stolen, 8.0);
    }

    /// T27.F03: each side books its own applied `f32` change; a credit the
    /// attacker's resolution absorbs is still a `Transferred` steal.
    #[test]
    fn a_credit_rounded_away_on_the_attacker_is_still_transferred() {
        let (mut sim, attacker_id, victim_id) =
            make_sim_two_creatures(Position::new(5, 5), 6.0e7, Position::new(5, 4), 30.0);
        sim.creatures[attacker_id].energy = 6.0e7;
        let outcome = steal_with_bite(attacker_id, &mut sim, Direction::N, 1.0);
        assert_eq!(outcome.result, PredationActionResult::Transferred);
        assert_eq!(sim.creatures[victim_id].energy, 29.0);
        let flows = &sim.stats.energy_flows;
        assert_eq!(flows.predation_victim_debit, 1.0);
        assert_eq!(flows.predation_attacker_credit, 0.0);
    }

    #[test]
    fn event_record_pushed_on_successful_transfer() {
        let (mut sim, attacker_id, _victim_id) =
            make_sim_two_creatures(Position::new(5, 5), 50.0, Position::new(5, 4), 30.0);
        sim.config.predation.steal_cost_rate = 0.0;

        let result = steal_with_bite(attacker_id, &mut sim, Direction::N, 10.0).result;

        assert_eq!(result, PredationActionResult::Transferred);
        assert_eq!(sim.stats.last_tick_predation_events.len(), 1);
        let event = &sim.stats.last_tick_predation_events[0];
        assert_eq!(event.attacker_x, 5);
        assert_eq!(event.attacker_y, 5);
        assert_eq!(event.victim_x, 5);
        assert_eq!(event.victim_y, 4);
        assert!((event.energy_stolen - 10.0).abs() < 1e-6);
        assert!(!event.killed);
    }

    #[test]
    fn event_record_pushed_on_kill() {
        let (mut sim, attacker_id, _victim_id) =
            make_sim_two_creatures(Position::new(5, 5), 50.0, Position::new(5, 4), 5.0);
        sim.config.predation.steal_cost_rate = 0.0;

        let result = steal_with_bite(attacker_id, &mut sim, Direction::N, 10.0).result;

        assert_eq!(result, PredationActionResult::TransferredAndKilled);
        assert_eq!(sim.stats.last_tick_predation_events.len(), 1);
        let event = &sim.stats.last_tick_predation_events[0];
        assert_eq!(event.attacker_x, 5);
        assert_eq!(event.attacker_y, 5);
        assert_eq!(event.victim_x, 5);
        assert_eq!(event.victim_y, 4);
        assert!((event.energy_stolen - 5.0).abs() < 1e-6);
        assert!(event.killed);
        assert_eq!(sim.stats.last_tick_predation_kills, 1);
    }

    #[test]
    fn no_event_record_on_rejection() {
        let (mut sim, attacker_id) = make_sim_one_creature(Position::new(5, 5), 50.0);
        sim.config.predation.steal_cost_rate = 0.0;

        let result = steal_with_bite(attacker_id, &mut sim, Direction::N, 10.0).result;

        assert_eq!(result, PredationActionResult::RejectedNoVictim);
        assert!(sim.stats.last_tick_predation_events.is_empty());
        assert_eq!(sim.stats.last_tick_predation_kills, 0);
    }
}
