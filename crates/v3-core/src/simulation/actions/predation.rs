use crate::contracts::{CreatureId, Direction};
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

/// Apply a StealEnergy action: attempt to steal energy from a neighbor.
///
/// The attacker always pays `steal_cost_rate * requested_amount` regardless of outcome.
/// If a victim exists, the actual transfer is `min(requested_amount, victim.energy)`.
/// If the victim's energy drops to zero or below, the victim is killed and removed,
/// and the attacker receives a complexity bonus.
pub fn apply_steal_energy(
    attacker_id: CreatureId,
    sim: &mut Simulation,
    dir: Direction,
    requested_amount: f32,
) -> PredationActionResult {
    use crate::simulation::energy_accounting::DeathCause;
    // Sanitize requested amount: NaN/Inf/negative → 0.0.
    let requested_amount = if requested_amount.is_finite() && requested_amount > 0.0 {
        requested_amount
    } else {
        0.0
    };

    // Step 1: Stats — count attempt and per-tick.
    sim.stats.predation_actions_attempted_total += 1;
    sim.stats.last_tick_steal += 1;

    // Step 2: Deduct cost (based on attempted amount, not actual; scaled by genome complexity and age).
    let cost = sim.config.energy.adjusted_action_cost(
        sim.config.predation.steal_cost_rate * requested_amount,
        sim.creatures[attacker_id].cached_complexity,
        sim.creatures[attacker_id].age,
    );
    let attacker = &mut sim.creatures[attacker_id];
    let before = attacker.energy;
    attacker.energy -= cost;
    sim.stats.energy_flows.action_charges.steal_energy +=
        attacker.observe_energy(before, DeathCause::ActionStealEnergy);

    // Step 3: Resolve target cell.
    let attacker_pos = sim.creatures[attacker_id].position;
    let target_pos = match sim.world.resolve_neighbor(attacker_pos, dir) {
        Some(p) => p,
        None => {
            sim.stats.predation_actions_rejected_total += 1;
            *sim.stats
                .predation_actions_by_result
                .entry(PredationActionResult::RejectedNoVictim)
                .or_insert(0) += 1;
            return PredationActionResult::RejectedNoVictim;
        }
    };

    // Step 4: Look up victim (guard against self-predation in degenerate worlds).
    let victim_id = match sim.world.creature_at(target_pos) {
        Some(id) if id != attacker_id => id,
        _ => {
            sim.stats.predation_actions_rejected_total += 1;
            *sim.stats
                .predation_actions_by_result
                .entry(PredationActionResult::RejectedNoVictim)
                .or_insert(0) += 1;
            return PredationActionResult::RejectedNoVictim;
        }
    };

    // Step 5: Compute actual steal (capped at victim's current energy).
    let actual = requested_amount.min(sim.creatures[victim_id].energy);

    // Step 6: Transfer energy.
    let victim = &mut sim.creatures[victim_id];
    let before = victim.energy;
    victim.energy -= actual;
    sim.stats.energy_flows.predation_victim_debit +=
        victim.observe_energy(before, DeathCause::Predation);
    let attacker = &mut sim.creatures[attacker_id];
    let before = attacker.energy;
    attacker.energy += actual;
    sim.stats.energy_flows.predation_attacker_credit -=
        attacker.observe_energy(before, DeathCause::Predation);

    // Step 7: Cap attacker energy at max.
    let max_energy = sim.config.energy.lifecycle.max_energy;
    if sim.creatures[attacker_id].energy > max_energy {
        let before = sim.creatures[attacker_id].energy;
        sim.creatures[attacker_id].energy = max_energy;
        sim.stats.energy_flows.maximum_energy_clamp_loss +=
            crate::simulation::energy_accounting::applied_debit(before, max_energy);
    }

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
        if sim.creatures[attacker_id].energy > max_energy {
            let before = sim.creatures[attacker_id].energy;
            sim.creatures[attacker_id].energy = max_energy;
            sim.stats.energy_flows.maximum_energy_clamp_loss +=
                crate::simulation::energy_accounting::applied_debit(before, max_energy);
        }

        // Remove victim from world, slotmap, and action logs.
        sim.remove_creature(victim_id);

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
        return PredationActionResult::TransferredAndKilled;
    }

    // Step 9: Victim survived.
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
    PredationActionResult::Transferred
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SimulationConfig;
    use crate::contracts::{CreatureId, Position};
    use crate::creature::founder::v3alpha1_founder_genome;
    use crate::creature::identity::CreatureIdentityState;
    use crate::creature::state::CreatureState;
    use crate::kernel::WorldState;
    use crate::simulation::simulation::Simulation;
    use rand::SeedableRng;
    use slotmap::SlotMap;

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
        cfg.world.food.initial_coverage = 0.0;
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
                [0, 0, 92, 92, 138, 138],
                0,
                [true; 6],
                CreatureIdentityState::default(),
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
                [0, 0, 92, 92, 138, 138],
                0,
                [true; 6],
                CreatureIdentityState::default(),
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
        };
        (sim, attacker_id, victim_id)
    }

    /// Create a simulation with one creature (attacker only, no victim).
    fn make_sim_one_creature(pos: Position, energy: f32) -> (Simulation, CreatureId) {
        let mut cfg = SimulationConfig::default();
        cfg.world.width = 10;
        cfg.world.height = 10;
        cfg.world.food.initial_coverage = 0.0;
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
                [0, 0, 92, 92, 138, 138],
                0,
                [true; 6],
                CreatureIdentityState::default(),
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
        };
        (sim, id)
    }

    #[test]
    fn steal_from_occupied_cell_transfers_energy() {
        // Attacker at (5,5), victim at (5,4) = N of attacker
        let (mut sim, attacker_id, victim_id) =
            make_sim_two_creatures(Position::new(5, 5), 50.0, Position::new(5, 4), 30.0);
        sim.config.predation.steal_cost_rate = 0.0; // no cost for clarity

        let result = apply_steal_energy(attacker_id, &mut sim, Direction::N, 10.0);

        assert_eq!(result, PredationActionResult::Transferred);
        assert!(
            (sim.creatures[attacker_id].energy - 60.0).abs() < 1e-6,
            "attacker should gain 10"
        );
        assert!(
            (sim.creatures[victim_id].energy - 20.0).abs() < 1e-6,
            "victim should lose 10"
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

        let result = apply_steal_energy(attacker_id, &mut sim, Direction::N, 10.0);

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
        let result = apply_steal_energy(attacker_id, &mut sim, Direction::N, 20.0);

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

        let result = apply_steal_energy(attacker_id, &mut sim, Direction::N, 10.0);

        assert_eq!(result, PredationActionResult::RejectedNoVictim);
        // Cost still deducted: 50 - adjusted_cost
        let expected = 50.0 - adjusted_cost;
        assert!(
            (sim.creatures[attacker_id].energy - expected).abs() < 1e-4,
            "cost should still be deducted: got {}",
            sim.creatures[attacker_id].energy
        );
    }

    #[test]
    fn steal_kills_victim_when_drained_to_zero() {
        let (mut sim, attacker_id, victim_id) =
            make_sim_two_creatures(Position::new(5, 5), 50.0, Position::new(5, 4), 10.0);
        sim.config.predation.steal_cost_rate = 0.0;

        // Request exactly victim's energy
        let result = apply_steal_energy(attacker_id, &mut sim, Direction::N, 10.0);

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

        let result = apply_steal_energy(attacker_id, &mut sim, Direction::N, 10.0);

        assert_eq!(result, PredationActionResult::TransferredAndKilled);
        // attacker: 50 + 5 (actual steal) + complexity bonus
        let expected = 50.0 + 5.0 + victim_complexity as f32 * 1.0;
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

        let result = apply_steal_energy(attacker_id, &mut sim, Direction::N, 10.0);

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

        let result = apply_steal_energy(attacker_id, &mut sim, Direction::N, 20.0);

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
    fn steal_zero_amount_only_costs_zero() {
        let (mut sim, attacker_id, victim_id) =
            make_sim_two_creatures(Position::new(5, 5), 50.0, Position::new(5, 4), 30.0);
        sim.config.predation.steal_cost_rate = 0.2;

        let result = apply_steal_energy(attacker_id, &mut sim, Direction::N, 0.0);

        assert_eq!(result, PredationActionResult::Transferred);
        // cost = 0.2 * 0 = 0, transfer = 0
        assert!(
            (sim.creatures[attacker_id].energy - 50.0).abs() < 1e-6,
            "no energy change for zero steal"
        );
        assert!(
            (sim.creatures[victim_id].energy - 30.0).abs() < 1e-6,
            "victim unchanged"
        );
    }

    #[test]
    fn event_record_pushed_on_successful_transfer() {
        let (mut sim, attacker_id, _victim_id) =
            make_sim_two_creatures(Position::new(5, 5), 50.0, Position::new(5, 4), 30.0);
        sim.config.predation.steal_cost_rate = 0.0;

        let result = apply_steal_energy(attacker_id, &mut sim, Direction::N, 10.0);

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

        let result = apply_steal_energy(attacker_id, &mut sim, Direction::N, 10.0);

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

        let result = apply_steal_energy(attacker_id, &mut sim, Direction::N, 10.0);

        assert_eq!(result, PredationActionResult::RejectedNoVictim);
        assert!(sim.stats.last_tick_predation_events.is_empty());
        assert_eq!(sim.stats.last_tick_predation_kills, 0);
    }
}
