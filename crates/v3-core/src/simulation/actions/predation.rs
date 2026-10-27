use super::TransferOutcome;
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
    /// A victim, but the steal moved nothing: no positive amount, an
    /// exhausted victim, or an amount `f32` rounding absorbs.
    RejectedNothingToTake,
}

impl PredationActionResult {
    /// Every result, in declaration order.
    pub const ALL: [Self; 4] = [
        Self::Transferred,
        Self::TransferredAndKilled,
        Self::RejectedNoVictim,
        Self::RejectedNothingToTake,
    ];

    /// Whether the steal moved energy.
    #[must_use]
    pub const fn transferred(self) -> bool {
        matches!(self, Self::Transferred | Self::TransferredAndKilled)
    }

    /// Stable string key used by transport/API boundaries.
    #[must_use]
    pub const fn as_key(self) -> &'static str {
        match self {
            Self::Transferred => "Transferred",
            Self::TransferredAndKilled => "TransferredAndKilled",
            Self::RejectedNoVictim => "RejectedNoVictim",
            Self::RejectedNothingToTake => "RejectedNothingToTake",
        }
    }
}

/// Apply a StealEnergy action: attempt to take part of a neighbour's store
/// (T27.F03).
///
/// The intensity is `i = min(1, steal_share × effort)` with the attacker's
/// own heritable share and the decoded modifier `effort`. The attacker
/// always pays the effort of base `exchange.effort_rate × i ×
/// max(attacker energy, 0)`, floored at `NoOp`'s (T16.F02), before the
/// target resolves, and nothing more. If a victim exists, the transfer is
/// `i × victim.energy` when the victim's energy is positive, else nothing; a
/// steal that leaves the victim's stored energy unchanged fails as
/// `RejectedNothingToTake` and kills nothing. If the victim's energy drops
/// to zero or below (at `i = 1`, or by rounding), the victim is killed and
/// removed, and the attacker receives a complexity bonus.
pub fn apply_steal_energy(
    attacker_id: CreatureId,
    sim: &mut Simulation,
    dir: Direction,
    effort: f32,
) -> TransferOutcome<PredationActionResult> {
    use crate::ledger::DeathCause;

    // Step 1: Stats — count attempt and per-tick.
    sim.stats.predation_actions_attempted_total += 1;
    sim.stats.last_tick_steal += 1;

    // Step 2: Deduct the effort, known from the attacker alone.
    let attacker = &mut sim.creatures[attacker_id];
    let intensity = super::exchange_intensity(attacker.body.steal_share, effort);
    let effort = super::pay_exchange_effort(
        attacker,
        &sim.config,
        intensity,
        DeathCause::ActionStealEnergy,
    );
    sim.stats.energy_flows.action_charges[ActionType::StealEnergy] += effort;

    // Steps 3 and 4: resolve the target cell and its victim, guarding
    // against self-predation in degenerate worlds.
    let attacker_pos = sim.creatures[attacker_id].position;
    let Some(victim_id) = super::neighbour_creature(sim, attacker_id, dir) else {
        return reject(sim, PredationActionResult::RejectedNoVictim, effort, None);
    };

    // Step 5: The share of the victim's store the intensity takes; a steal
    // transfers only when the victim's stored energy falls.
    let before = sim.creatures[victim_id].energy;
    let Some(actual) = super::exchange_amount(intensity, before) else {
        return reject(
            sim,
            PredationActionResult::RejectedNothingToTake,
            effort,
            Some(victim_id),
        );
    };

    // Step 6: Transfer energy.
    let victim = &mut sim.creatures[victim_id];
    victim.energy -= actual;
    sim.stats.energy_flows.predation_victim_debit +=
        victim.observe_energy(before, DeathCause::Predation);
    #[cfg(feature = "telemetry-seams")]
    if sim.stats.observed_creature == Some(victim_id) {
        sim.stats.observed_damage += actual;
    }
    let attacker = &mut sim.creatures[attacker_id];
    let before = attacker.energy;
    attacker.energy += actual;
    sim.stats.energy_flows.predation_attacker_credit -=
        attacker.observe_energy(before, DeathCause::Predation);

    // Step 7: Cap attacker energy at its body's cap.
    sim.stats.energy_flows.maximum_energy_clamp_loss +=
        super::cap_energy(&mut sim.creatures[attacker_id]);

    // Step 8: Check kill.
    if sim.creatures[victim_id].energy <= 0.0 {
        // Snapshot victim data before removal.
        let victim_complexity = sim.creatures[victim_id].complexity();
        let victim_pos = sim.creatures[victim_id].position;

        // Compute and award complexity bonus.
        let bonus =
            victim_complexity as f32 * sim.config.predation.kill_complexity_bonus_multiplier;
        let attacker = &mut sim.creatures[attacker_id];
        let before = attacker.energy;
        attacker.energy += bonus;
        sim.stats.energy_flows.predation_kill_bonus_credit -=
            attacker.observe_energy(before, DeathCause::Predation);
        sim.stats.energy_flows.maximum_energy_clamp_loss +=
            super::cap_energy(&mut sim.creatures[attacker_id]);

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
                energy_stolen: actual,
                killed: true,
            });
        return TransferOutcome {
            result: PredationActionResult::TransferredAndKilled,
            amount: actual,
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
            energy_stolen: actual,
            killed: false,
        });
    TransferOutcome {
        result: PredationActionResult::Transferred,
        amount: actual,
        counterpart: Some(victim_id),
    }
}

/// Books a steal that moved nothing: one rejection by `result`, its effort as
/// failed.
fn reject(
    sim: &mut Simulation,
    result: PredationActionResult,
    effort: f64,
    counterpart: Option<CreatureId>,
) -> TransferOutcome<PredationActionResult> {
    sim.stats.predation_actions_rejected_total += 1;
    *sim.stats
        .predation_actions_by_result
        .entry(result)
        .or_insert(0) += 1;
    sim.stats.energy_flows.failed_action_effort[ActionType::StealEnergy] += effort;
    TransferOutcome {
        result,
        amount: 0.0,
        counterpart,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SimulationConfig;
    use crate::contracts::{CreatureId, Position};
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
            capacity: 200.0,
            steal_share: 0.2,
            give_share: 0.2,
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
    fn steal_from_occupied_cell_transfers_the_share_of_the_victim_store() {
        // Attacker at (5,5), victim at (5,4) = N of attacker
        let (mut sim, attacker_id, victim_id) =
            make_sim_two_creatures(Position::new(5, 5), 50.0, Position::new(5, 4), 30.0);
        sim.config.exchange.effort_rate = 0.0; // only the NoOp floor (T16.F02)

        let outcome = apply_steal_energy(attacker_id, &mut sim, Direction::N, 1.0);

        // Intensity 0.2 (the founder share at modifier 1) takes 0.2 × 30.
        assert_eq!(outcome.result, PredationActionResult::Transferred);
        assert_eq!(outcome.amount, 6.0);
        assert_eq!(outcome.counterpart, Some(victim_id));
        assert_eq!(sim.creatures[attacker_id].energy, (50.0 - 0.4) + 6.0);
        assert_eq!(sim.creatures[victim_id].energy, 24.0, "no kill at 0.2");
        let flows = &sim.stats.energy_flows;
        assert_eq!(flows.predation_victim_debit, 6.0);
        assert_eq!(flows.predation_attacker_credit, 6.0);
    }

    #[test]
    fn the_intensity_scales_the_share_by_the_effort_up_to_one() {
        for (share, effort, expected) in [(0.2, 2.0, 12.0), (0.5, 0.25, 3.75), (0.5, 4.0, 30.0)] {
            let (mut sim, attacker_id, victim_id) =
                make_sim_two_creatures(Position::new(5, 5), 50.0, Position::new(5, 4), 30.0);
            sim.creatures[attacker_id].body.steal_share = share;
            let outcome = apply_steal_energy(attacker_id, &mut sim, Direction::N, effort);
            assert_eq!(outcome.amount, expected, "{share} × {effort}");
            assert_eq!(
                sim.creatures.contains_key(victim_id),
                expected < 30.0,
                "a kill only at intensity 1"
            );
        }
    }

    #[test]
    fn a_survived_transfer_counts_one_hit_on_the_victim_and_no_kill() {
        let (mut sim, attacker_id, victim_id) =
            make_sim_two_creatures(Position::new(5, 5), 50.0, Position::new(5, 4), 30.0);
        sim.config.exchange.effort_rate = 0.0;

        let result = apply_steal_energy(attacker_id, &mut sim, Direction::N, 1.0).result;

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
        sim.config.exchange.effort_rate = 0.0;

        let result = apply_steal_energy(attacker_id, &mut sim, Direction::N, 20.0).result;

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
        sim.config.exchange.effort_rate = 0.0;

        let result = apply_steal_energy(attacker_id, &mut sim, Direction::N, 10.0).result;

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
        sim.config.exchange.effort_rate = 0.2;
        // Base: rate 0.2 × intensity 0.2 × the attacker's 50.
        let adjusted_cost = sim.config.energy.adjusted_action_cost(
            0.2 * 0.2 * 50.0,
            sim.creatures[attacker_id].complexity(),
            sim.creatures[attacker_id].age,
        );

        let result = apply_steal_energy(attacker_id, &mut sim, Direction::N, 1.0).result;

        assert_eq!(result, PredationActionResult::Transferred);
        // attacker: 50 - adjusted_cost + gain(0.2 × 30)
        let expected = 50.0 - adjusted_cost + 6.0;
        assert!(
            (sim.creatures[attacker_id].energy - expected).abs() < 1e-4,
            "attacker energy {} should be {}",
            sim.creatures[attacker_id].energy,
            expected
        );
    }

    #[test]
    fn the_charge_base_reads_the_attacker_store_not_the_victim_store() {
        let (mut sim, attacker_id, victim_id) =
            make_sim_two_creatures(Position::new(5, 5), 50.0, Position::new(5, 4), 5.0);
        sim.config.exchange.effort_rate = 0.2;
        // Intensity 1 (0.2 × modifier 5, capped): base 0.2 × 1 × 50.
        let adjusted_cost = sim.config.energy.adjusted_action_cost(
            0.2 * 50.0,
            sim.creatures[attacker_id].complexity(),
            sim.creatures[attacker_id].age,
        );

        // Capture victim complexity before kill removes it from slotmap.
        let victim_complexity = sim.creatures[victim_id].complexity();
        let result = apply_steal_energy(attacker_id, &mut sim, Direction::N, 5.0).result;

        // At intensity 1 the steal takes the whole store and kills.
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
        sim.config.exchange.effort_rate = 0.2;
        let adjusted_cost = sim.config.energy.adjusted_action_cost(
            0.2 * 0.2 * 50.0,
            sim.creatures[attacker_id].complexity(),
            sim.creatures[attacker_id].age,
        );

        let result = apply_steal_energy(attacker_id, &mut sim, Direction::N, 1.0).result;

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
        sim.config.exchange.effort_rate = 0.0;

        // Request exactly victim's energy
        let result = apply_steal_energy(attacker_id, &mut sim, Direction::N, 10.0).result;

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
        sim.config.exchange.effort_rate = 0.0;
        sim.config.predation.kill_complexity_bonus_multiplier = 1.0; // large multiplier for clarity

        let victim_complexity = sim.creatures[victim_id].complexity();
        assert!(
            victim_complexity > 0,
            "founder genome should have nonzero complexity"
        );

        let result = apply_steal_energy(attacker_id, &mut sim, Direction::N, 10.0).result;

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
        sim.config.exchange.effort_rate = 0.0;

        let result = apply_steal_energy(attacker_id, &mut sim, Direction::N, 10.0).result;

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
        sim.config.exchange.effort_rate = 0.0;
        let max_energy = sim.creatures[attacker_id].energy_cap(); // 200.0

        let result = apply_steal_energy(attacker_id, &mut sim, Direction::N, 1.0).result;

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

    #[test]
    fn a_zero_effort_pays_the_noop_floor_and_takes_nothing() {
        let (mut sim, attacker_id, victim_id) =
            make_sim_two_creatures(Position::new(5, 5), 50.0, Position::new(5, 4), 30.0);
        sim.config.exchange.effort_rate = 0.2;

        let outcome = apply_steal_energy(attacker_id, &mut sim, Direction::N, 0.0);

        assert_eq!(outcome.result, PredationActionResult::RejectedNothingToTake);
        assert_eq!(outcome.amount, 0.0);
        assert_eq!(outcome.counterpart, Some(victim_id));
        // cost = max(0.2 * 0, noop_cost) = 0.4 (T16.F02; effort-cost defaults, 2026-10-10)
        assert!(
            (sim.creatures[attacker_id].energy - (50.0 - 0.4)).abs() < 1e-6,
            "a zero steal pays the NoOp floor"
        );
        assert_eq!(sim.creatures[victim_id].energy, 30.0, "victim unchanged");
        assert_nothing_taken(&sim, attacker_id, victim_id);
    }

    /// A steal that moved nothing: rejected once, its effort booked as failed,
    /// no kill, no hit, no `Predation` death and no event record.
    fn assert_nothing_taken(sim: &Simulation, attacker_id: CreatureId, victim_id: CreatureId) {
        assert!(sim.creatures.contains_key(victim_id), "no kill");
        assert_eq!(sim.stats.predation_actions_rejected_total, 1);
        assert_eq!(sim.stats.predation_actions_transferred_total, 0);
        assert_eq!(
            sim.stats
                .predation_actions_by_result
                .get(&PredationActionResult::RejectedNothingToTake),
            Some(&1)
        );
        assert_eq!(sim.stats.predation_kills_total, 0);
        assert!(sim.stats.last_tick_predation_events.is_empty());
        assert_eq!(sim.creatures[victim_id].pending_death_cause, None);
        assert_eq!(
            sim.creatures[victim_id].lifetime_predation_hits_taken_count,
            0
        );
        assert_eq!(sim.creatures[attacker_id].lifetime_predation_kills_count, 0);
        let flows = &sim.stats.energy_flows;
        assert_eq!(flows.predation_victim_debit, 0.0);
        assert_eq!(flows.predation_attacker_credit, 0.0);
        assert!(flows.failed_action_effort[ActionType::StealEnergy] > 0.0);
        assert_eq!(
            flows.failed_action_effort[ActionType::StealEnergy],
            flows.action_charges[ActionType::StealEnergy]
        );
    }

    #[test]
    fn an_exhausted_victim_yields_nothing_to_take_and_no_kill() {
        for store in [0.0, -0.0, -3.0, f32::NAN, f32::NEG_INFINITY] {
            let (mut sim, attacker_id, victim_id) =
                make_sim_two_creatures(Position::new(5, 5), 50.0, Position::new(5, 4), 30.0);
            sim.creatures[victim_id].energy = store;

            let outcome = apply_steal_energy(attacker_id, &mut sim, Direction::N, 10.0);

            assert_eq!(
                outcome.result,
                PredationActionResult::RejectedNothingToTake,
                "store {store}"
            );
            assert_eq!(outcome.amount, 0.0);
            assert_eq!(sim.creatures[victim_id].energy.to_bits(), store.to_bits());
            assert_nothing_taken(&sim, attacker_id, victim_id);
        }
    }

    #[test]
    fn a_sub_resolution_amount_is_nothing_to_take() {
        // `share_min` against the smallest positive store rounds to zero.
        let tiny = f32::from_bits(1);
        let (mut sim, attacker_id, victim_id) =
            make_sim_two_creatures(Position::new(5, 5), 50.0, Position::new(5, 4), tiny);
        sim.creatures[attacker_id].body.steal_share = sim.config.exchange.share_min;

        let outcome = apply_steal_energy(attacker_id, &mut sim, Direction::N, 1.0);

        assert_eq!(outcome.result, PredationActionResult::RejectedNothingToTake);
        assert_eq!(outcome.amount, 0.0);
        assert_eq!(sim.creatures[victim_id].energy, tiny);
        assert_nothing_taken(&sim, attacker_id, victim_id);
    }

    #[test]
    fn a_kill_below_intensity_one_happens_only_by_rounding() {
        // 0.6 × one ulp rounds to the ulp: the steal takes the whole store.
        let tiny = f32::from_bits(1);
        let (mut sim, attacker_id, victim_id) =
            make_sim_two_creatures(Position::new(5, 5), 50.0, Position::new(5, 4), tiny);
        sim.creatures[attacker_id].body.steal_share = 0.6;

        let outcome = apply_steal_energy(attacker_id, &mut sim, Direction::N, 1.0);

        assert_eq!(outcome.result, PredationActionResult::TransferredAndKilled);
        assert_eq!(outcome.amount, tiny);
        assert!(!sim.creatures.contains_key(victim_id));
    }

    proptest::proptest! {
        /// A steal is `Transferred` or `TransferredAndKilled` only when it
        /// moved a positive amount that lowered the victim's stored energy,
        /// for any `f32` bit pattern of the victim's store and the effort.
        #[test]
        fn a_transfer_always_lowers_the_victim_store(
            store_bits in proptest::num::u32::ANY,
            effort_bits in proptest::num::u32::ANY,
        ) {
            let store = f32::from_bits(store_bits);
            let (mut sim, attacker_id, victim_id) =
                make_sim_two_creatures(Position::new(5, 5), 50.0, Position::new(5, 4), 30.0);
            sim.creatures[victim_id].energy = store;

            let outcome =
                apply_steal_energy(attacker_id, &mut sim, Direction::N, f32::from_bits(effort_bits));

            match outcome.result {
                PredationActionResult::Transferred => {
                    proptest::prop_assert!(outcome.amount > 0.0);
                    proptest::prop_assert!(sim.creatures[victim_id].energy < store);
                }
                PredationActionResult::TransferredAndKilled => {
                    proptest::prop_assert!(outcome.amount > 0.0);
                    proptest::prop_assert!(!sim.creatures.contains_key(victim_id));
                }
                PredationActionResult::RejectedNothingToTake => {
                    proptest::prop_assert_eq!(outcome.amount, 0.0);
                    proptest::prop_assert_eq!(
                        sim.creatures[victim_id].energy.to_bits(),
                        store.to_bits()
                    );
                }
                PredationActionResult::RejectedNoVictim => {
                    proptest::prop_assert!(false, "the victim is adjacent");
                }
            }
        }
    }

    #[test]
    fn event_record_pushed_on_successful_transfer() {
        let (mut sim, attacker_id, _victim_id) =
            make_sim_two_creatures(Position::new(5, 5), 50.0, Position::new(5, 4), 30.0);
        sim.config.exchange.effort_rate = 0.0;

        let result = apply_steal_energy(attacker_id, &mut sim, Direction::N, 1.0).result;

        assert_eq!(result, PredationActionResult::Transferred);
        assert_eq!(sim.stats.last_tick_predation_events.len(), 1);
        let event = &sim.stats.last_tick_predation_events[0];
        assert_eq!(event.attacker_x, 5);
        assert_eq!(event.attacker_y, 5);
        assert_eq!(event.victim_x, 5);
        assert_eq!(event.victim_y, 4);
        assert_eq!(event.energy_stolen, 6.0);
        assert!(!event.killed);
    }

    #[test]
    fn event_record_pushed_on_kill() {
        let (mut sim, attacker_id, _victim_id) =
            make_sim_two_creatures(Position::new(5, 5), 50.0, Position::new(5, 4), 5.0);
        sim.config.exchange.effort_rate = 0.0;

        let result = apply_steal_energy(attacker_id, &mut sim, Direction::N, 10.0).result;

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
        sim.config.exchange.effort_rate = 0.0;

        let result = apply_steal_energy(attacker_id, &mut sim, Direction::N, 10.0).result;

        assert_eq!(result, PredationActionResult::RejectedNoVictim);
        assert!(sim.stats.last_tick_predation_events.is_empty());
        assert_eq!(sim.stats.last_tick_predation_kills, 0);
    }

    #[test]
    fn each_result_has_its_stable_key_and_transfer_flag() {
        let expected = [
            (PredationActionResult::Transferred, "Transferred", true),
            (
                PredationActionResult::TransferredAndKilled,
                "TransferredAndKilled",
                true,
            ),
            (
                PredationActionResult::RejectedNoVictim,
                "RejectedNoVictim",
                false,
            ),
            (
                PredationActionResult::RejectedNothingToTake,
                "RejectedNothingToTake",
                false,
            ),
        ];
        assert_eq!(
            PredationActionResult::ALL.to_vec(),
            expected.map(|(result, _, _)| result).to_vec()
        );
        for (result, key, transferred) in expected {
            assert_eq!(result.as_key(), key);
            assert_eq!(result.transferred(), transferred, "{result:?}");
        }
    }
}
