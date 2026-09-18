pub(crate) mod cgp_reproduction;
mod predation;
mod reproduction;

pub use predation::{apply_steal_energy, PredationActionResult, PredationEventRecord};
pub use reproduction::{apply_reproduce, ReproductionActionResult, ReproductionInvalidTargetCause};

use crate::config::OrdinaryFoodTypeId;
use crate::config::SimulationConfig;
use crate::contracts::{CreatureId, Direction};
use crate::creature::state::CreatureState;
use crate::kernel::WorldState;
use crate::simulation::energy_accounting::{applied_debit, DeathCause, EnergyFlows};

// ─── Action application functions ─────────────────────────────────────────────

/// Classification for rejected move actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MoveBlockedCause {
    Barrier,
    Occupied,
    OutOfBounds,
}

impl MoveBlockedCause {
    /// Stable string key used by transport/API boundaries.
    #[must_use]
    pub const fn as_key(self) -> &'static str {
        match self {
            Self::Barrier => "barrier",
            Self::Occupied => "occupied",
            Self::OutOfBounds => "out_of_bounds",
        }
    }
}

/// Whether the acting creature has any live barrier sensor read in reachable mesh nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BarrierReaderState {
    HasBarrierReader,
    NoBarrierReader,
}

impl BarrierReaderState {
    /// Stable string key used by transport/API boundaries.
    #[must_use]
    pub const fn as_key(self) -> &'static str {
        match self {
            Self::HasBarrierReader => "has_barrier_reader",
            Self::NoBarrierReader => "no_barrier_reader",
        }
    }
}

/// Apply a NoOp action (deduct noop cost, scaled by genome complexity and age).
pub fn apply_noop(
    creature: &mut CreatureState,
    config: &SimulationConfig,
    flows: &mut EnergyFlows,
) {
    let before = creature.energy;
    creature.energy -= config.energy.adjusted_action_cost(
        config.energy.costs.noop_cost,
        creature.cached_complexity,
        creature.age,
    );
    flows.action_charges.noop += creature.observe_energy(before, DeathCause::ActionNoop);
}

/// Apply a typed Eat action using the configured food owner for the cell.
#[must_use]
pub fn apply_typed_eat(
    creature: &mut CreatureState,
    world: &mut WorldState,
    config: &SimulationConfig,
    type_idx: OrdinaryFoodTypeId,
    flows: &mut EnergyFlows,
) -> bool {
    flows
        .food_intake_by_type
        .resize(config.world.food.types.len(), 0.0);
    let food = world.consume_food_type(creature.position, type_idx);
    if food > 0.0 {
        let reward = config
            .world
            .food
            .types
            .get(usize::from(type_idx.get()))
            .and_then(|food_type| food_type.energy_per_unit)
            .unwrap_or(config.energy.costs.eat_reward_per_food);
        let before = creature.energy;
        let credited = creature.energy + food * reward;
        creature.energy = credited.clamp(0.0, config.energy.lifecycle.max_energy);
        flows.food_intake_by_type[usize::from(type_idx.get())] += applied_debit(credited, before);
        flows.maximum_energy_clamp_loss += applied_debit(credited, creature.energy).max(0.0);
        flows.zero_floor_credit += applied_debit(creature.energy, credited).max(0.0);
        creature.observe_energy(before, DeathCause::ActionEat);
    }
    let before = creature.energy;
    creature.energy -= config.energy.adjusted_action_cost(
        config.energy.costs.eat_cost,
        creature.cached_complexity,
        creature.age,
    );
    flows.action_charges.eat += creature.observe_energy(before, DeathCause::ActionEat);
    food > 0.0
}

/// Apply a Move action: move one step in `dir` if the target is valid.
///
/// Energy cost is always deducted even if the move is rejected (blocked cell).
/// Returns `true` if the creature actually moved, `false` if the target was invalid.
#[must_use]
pub fn apply_move(
    id: CreatureId,
    creature: &mut CreatureState,
    world: &mut WorldState,
    dir: Direction,
    config: &SimulationConfig,
    flows: &mut EnergyFlows,
) -> bool {
    let target = world
        .resolve_neighbor(creature.position, dir)
        .filter(|&p| world.is_valid_target_cell(p));

    let succeeded = if let Some(target_pos) = target {
        world.remove_creature(creature.position);
        world.place_creature(target_pos, id);
        creature.position = target_pos;
        true
    } else {
        false
    };

    let before = creature.energy;
    creature.energy -= config.energy.adjusted_action_cost(
        config.energy.costs.move_cost,
        creature.cached_complexity,
        creature.age,
    );
    flows.action_charges.r#move += creature.observe_energy(before, DeathCause::ActionMove);
    succeeded
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SimulationConfig;
    use crate::contracts::Position;
    use crate::creature::founder::v3alpha1_founder_genome;
    use crate::creature::identity::CreatureIdentityState;
    use crate::kernel::WorldState;
    use crate::simulation::seeding::seed_simulation;
    use crate::simulation::simulation::Simulation;
    use rand::SeedableRng;
    use slotmap::SlotMap;

    fn small_config() -> SimulationConfig {
        let mut cfg = SimulationConfig::default();
        cfg.world.width = 10;
        cfg.world.height = 10;
        cfg.population.initial_creatures = 1;
        cfg
    }

    /// Create a minimal Simulation with one creature at the given position.
    fn make_sim_one_creature(pos: Position, energy: f32) -> (Simulation, CreatureId) {
        use rand::SeedableRng;
        let cfg = small_config();
        let mut world = WorldState::new(cfg.world.width, cfg.world.height, cfg.world.edge_mode);
        world.reconfigure_food(cfg.world.food.clone());
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

    /// Cumulative `applied` across every per-operator mutation funnel.
    fn applied_funnel_total(sim: &Simulation) -> u64 {
        sim.stats
            .mutation_operator_funnel_total_by_operator
            .values()
            .map(|funnel| funnel.applied)
            .sum()
    }

    /// Cumulative `skipped` across every per-operator mutation funnel.
    fn skipped_funnel_total(sim: &Simulation) -> u64 {
        sim.stats
            .mutation_operator_funnel_total_by_operator
            .values()
            .map(|funnel| funnel.skipped)
            .sum()
    }

    #[test]
    fn energy_only_reproduction_needs_no_reserve_preparation() {
        let (mut sim, id) = make_sim_one_creature(Position::new(5, 5), 80.0);
        sim.creatures[id].age = sim.config.energy.lifecycle.min_reproduce_age;
        let mut rng = rand::rngs::SmallRng::seed_from_u64(1);
        assert_eq!(
            apply_reproduce(id, &mut sim, Direction::N, 20.0, &mut rng),
            ReproductionActionResult::Spawned
        );
    }

    // ── typed Eat ─────────────────────────────────────────────────────────────

    #[test]
    fn apply_typed_eat_increases_energy_and_clears_food() {
        let pos = Position::new(3, 3);
        let (mut sim, id) = make_sim_one_creature(pos, 10.0);
        sim.world.set_food(pos, 0.5);
        let energy_before = sim.creatures[id].energy;
        let creature = sim.creatures.get_mut(id).unwrap();
        let _ = apply_typed_eat(
            creature,
            &mut sim.world,
            &sim.config,
            OrdinaryFoodTypeId::default(),
            &mut sim.stats.energy_flows,
        );
        assert!(
            sim.creatures[id].energy > energy_before,
            "eat should increase energy"
        );
        assert!(
            (sim.world.food_at(pos) - 0.0).abs() < 1e-6,
            "food should be consumed"
        );
    }

    #[test]
    fn apply_typed_eat_caps_energy_at_max() {
        let pos = Position::new(3, 3);
        let (mut sim, id) = make_sim_one_creature(pos, 195.0);
        sim.config.energy.costs.eat_reward_per_food = 20.0;
        sim.config.energy.costs.eat_cost = 2.0;
        // Place max food to ensure energy would exceed max without cap.
        sim.world.set_food(pos, 1.0);
        let max = sim.config.energy.lifecycle.max_energy;
        let creature = sim.creatures.get_mut(id).unwrap();
        let _ = apply_typed_eat(
            creature,
            &mut sim.world,
            &sim.config,
            OrdinaryFoodTypeId::default(),
            &mut sim.stats.energy_flows,
        );
        assert_eq!(
            sim.creatures[id].energy,
            max - 2.0,
            "cap must precede the Eat cost"
        );
    }

    #[test]
    fn typed_eat_applies_shared_reward_from_actual_consumption() {
        let pos = Position::new(3, 3);
        let (mut sim, id) = make_sim_one_creature(pos, 10.0);
        sim.world
            .set_food_type(pos, OrdinaryFoodTypeId::new(0), 0.25);
        let before = sim.creatures[id].energy;
        let creature = sim.creatures.get_mut(id).expect("fixture creature");

        let succeeded = apply_typed_eat(
            creature,
            &mut sim.world,
            &sim.config,
            OrdinaryFoodTypeId::new(0),
            &mut sim.stats.energy_flows,
        );

        assert!(succeeded);
        assert!((sim.creatures[id].energy - (before + 0.25 * 5.0)).abs() < 1e-5);
    }

    // ── apply_move ─────────────────────────────────────────────────────────────

    #[test]
    fn apply_move_updates_position_and_occupancy() {
        let start = Position::new(5, 5);
        let (mut sim, id) = make_sim_one_creature(start, 50.0);
        let adjusted_move_cost = sim.config.energy.adjusted_action_cost(
            sim.config.energy.costs.move_cost,
            sim.creatures[id].cached_complexity,
            sim.creatures[id].age,
        );
        let energy_before = sim.creatures[id].energy;
        {
            let creature = sim.creatures.get_mut(id).unwrap();
            let _ = apply_move(
                id,
                creature,
                &mut sim.world,
                Direction::N,
                &sim.config,
                &mut sim.stats.energy_flows,
            );
        }
        // In wrap mode, N of (5,5) on a 10×10 world is (5,4).
        let expected = Position::new(5, 4);
        assert_eq!(sim.creatures[id].position, expected);
        assert!(sim.world.creature_at(expected).is_some());
        assert!(sim.world.creature_at(start).is_none());
        assert!((sim.creatures[id].energy - (energy_before - adjusted_move_cost)).abs() < 1e-6);
    }

    #[test]
    fn apply_move_into_barrier_no_position_change_cost_deducted() {
        let start = Position::new(5, 5);
        let (mut sim, id) = make_sim_one_creature(start, 50.0);
        // Place a barrier to the north.
        sim.world.set_barrier(Position::new(5, 4), true);
        let energy_before = sim.creatures[id].energy;
        let adjusted_move_cost = sim.config.energy.adjusted_action_cost(
            sim.config.energy.costs.move_cost,
            sim.creatures[id].cached_complexity,
            sim.creatures[id].age,
        );
        {
            let creature = sim.creatures.get_mut(id).unwrap();
            let _ = apply_move(
                id,
                creature,
                &mut sim.world,
                Direction::N,
                &sim.config,
                &mut sim.stats.energy_flows,
            );
        }
        assert_eq!(
            sim.creatures[id].position, start,
            "should not move into barrier"
        );
        assert!(
            (sim.creatures[id].energy - (energy_before - adjusted_move_cost)).abs() < 1e-6,
            "cost still deducted"
        );
    }

    #[test]
    fn apply_move_into_occupied_cell_no_position_change() {
        let pos1 = Position::new(5, 5);
        let pos2 = Position::new(5, 4); // north of pos1
        let cfg = small_config();
        let mut world = WorldState::new(cfg.world.width, cfg.world.height, cfg.world.edge_mode);
        let mut creatures: SlotMap<CreatureId, CreatureState> = SlotMap::with_key();
        let id1 = creatures.insert_with_key(|id| {
            CreatureState::new(
                id,
                v3alpha1_founder_genome(),
                pos1,
                50.0,
                0,
                [0, 0, 92, 92, 138, 138],
                0,
                [true; 6],
                CreatureIdentityState::default(),
                [0.0; 16],
            )
        });
        let id2 = creatures.insert_with_key(|id| {
            CreatureState::new(
                id,
                v3alpha1_founder_genome(),
                pos2,
                50.0,
                0,
                [0, 0, 92, 92, 138, 138],
                0,
                [true; 6],
                CreatureIdentityState::default(),
                [0.0; 16],
            )
        });
        world.place_creature(pos1, id1);
        world.place_creature(pos2, id2);
        let mut sim = Simulation {
            world,
            creatures,
            action_logs: slotmap::SecondaryMap::new(),
            tick: 0,
            config: cfg,
            stats: crate::simulation::stats::SimStats::default(),
            rng: rand::rngs::SmallRng::seed_from_u64(0),
        };
        {
            let creature = sim.creatures.get_mut(id1).unwrap();
            let _ = apply_move(
                id1,
                creature,
                &mut sim.world,
                Direction::N,
                &sim.config,
                &mut sim.stats.energy_flows,
            );
        }
        assert_eq!(
            sim.creatures[id1].position, pos1,
            "cannot move into occupied cell"
        );
    }

    // ── apply_reproduce ────────────────────────────────────────────────────────

    #[test]
    fn apply_reproduce_creates_child_with_inherited_genome() {
        // Give parent plenty of energy.
        let pos = Position::new(5, 5);
        let (mut sim, parent_id) = make_sim_one_creature(pos, 80.0);
        sim.creatures[parent_id].age = sim.config.energy.lifecycle.min_reproduce_age;
        let mut rng = rand::rngs::SmallRng::seed_from_u64(1);
        let result = apply_reproduce(parent_id, &mut sim, Direction::N, 20.0, &mut rng);
        assert_eq!(result, ReproductionActionResult::Spawned);
        assert_eq!(sim.creatures.len(), 2);
        let child = sim.creatures.values().find(|c| c.id != parent_id).unwrap();
        assert_eq!(child.generation, 1);
    }

    #[test]
    fn apply_reproduce_default_cap_allows_twenty_energy_transfer() {
        let pos = Position::new(5, 5);
        let (mut sim, parent_id) = make_sim_one_creature(pos, 80.0);
        sim.creatures[parent_id].age = sim.config.energy.lifecycle.min_reproduce_age;
        let mut rng = rand::rngs::SmallRng::seed_from_u64(11);

        let result = apply_reproduce(parent_id, &mut sim, Direction::N, 20.0, &mut rng);

        assert_eq!(result, ReproductionActionResult::Spawned);
        let child = sim
            .creatures
            .values()
            .find(|creature| creature.generation == 1)
            .expect("child not found");
        assert!(
            (child.energy - 20.0).abs() < 1e-6,
            "default offspring cap should not clamp a 20.0 transfer request"
        );
    }

    #[test]
    fn apply_reproduce_fails_when_target_occupied() {
        let pos = Position::new(5, 5);
        let north = Position::new(5, 4);
        let cfg = small_config();
        let mut world = WorldState::new(cfg.world.width, cfg.world.height, cfg.world.edge_mode);
        let mut creatures: SlotMap<CreatureId, CreatureState> = SlotMap::with_key();
        let parent = creatures.insert_with_key(|id| {
            CreatureState::new(
                id,
                v3alpha1_founder_genome(),
                pos,
                80.0,
                0,
                [0, 0, 92, 92, 138, 138],
                0,
                [true; 6],
                CreatureIdentityState::default(),
                [0.0; 16],
            )
        });
        let blocker = creatures.insert_with_key(|id| {
            CreatureState::new(
                id,
                v3alpha1_founder_genome(),
                north,
                80.0,
                0,
                [0, 0, 92, 92, 138, 138],
                0,
                [true; 6],
                CreatureIdentityState::default(),
                [0.0; 16],
            )
        });
        world.place_creature(pos, parent);
        world.place_creature(north, blocker);
        let mut sim = Simulation {
            world,
            creatures,
            action_logs: slotmap::SecondaryMap::new(),
            tick: 0,
            config: cfg,
            stats: crate::simulation::stats::SimStats::default(),
            rng: rand::rngs::SmallRng::seed_from_u64(0),
        };
        let mut rng = rand::rngs::SmallRng::seed_from_u64(2);
        let result = apply_reproduce(parent, &mut sim, Direction::N, 20.0, &mut rng);
        assert_eq!(result, ReproductionActionResult::RejectedInvalidTarget);
        assert_eq!(sim.creatures.len(), 2, "no new creature spawned");
    }

    #[test]
    fn apply_reproduce_fails_when_energy_insufficient() {
        // Set energy just enough to cover reproduce_cost but NOT enough for transfer.
        let pos = Position::new(5, 5);
        let cost = SimulationConfig::default().energy.costs.reproduce_cost;
        // Give just barely enough to cover cost but less than min_reproduce_energy after deduction.
        let low_energy = cost
            + SimulationConfig::default()
                .energy
                .lifecycle
                .min_reproduce_energy
            - 0.1;
        let (mut sim, parent_id) = make_sim_one_creature(pos, low_energy);
        sim.creatures[parent_id].age = sim.config.energy.lifecycle.min_reproduce_age;
        let mut rng = rand::rngs::SmallRng::seed_from_u64(3);
        let result = apply_reproduce(parent_id, &mut sim, Direction::N, 20.0, &mut rng);
        assert_eq!(result, ReproductionActionResult::RejectedEnergyConstraints);
    }

    #[test]
    fn apply_reproduce_fails_when_parent_below_min_reproduce_age() {
        let pos = Position::new(5, 5);
        let (mut sim, parent_id) = make_sim_one_creature(pos, 80.0);
        sim.config.energy.lifecycle.min_reproduce_age = 20;
        sim.creatures[parent_id].age = 0;
        let mut rng = rand::rngs::SmallRng::seed_from_u64(31);
        let result = apply_reproduce(parent_id, &mut sim, Direction::N, 20.0, &mut rng);
        assert_eq!(result, ReproductionActionResult::RejectedAgeConstraints);
        assert_eq!(sim.creatures.len(), 1, "no child should be spawned");
    }

    #[test]
    fn apply_reproduce_age_rejection_does_not_charge_reproduce_cost() {
        let pos = Position::new(5, 5);
        let (mut sim, parent_id) = make_sim_one_creature(pos, 80.0);
        sim.config.energy.lifecycle.min_reproduce_age = 20;
        sim.creatures[parent_id].age = 0;
        let energy_before = sim.creatures[parent_id].energy;
        let mut rng = rand::rngs::SmallRng::seed_from_u64(32);
        let result = apply_reproduce(parent_id, &mut sim, Direction::N, 20.0, &mut rng);
        assert_eq!(result, ReproductionActionResult::RejectedAgeConstraints);
        assert!(
            (sim.creatures[parent_id].energy - energy_before).abs() < f32::EPSILON,
            "age rejection should happen before reproduce_cost is charged"
        );
    }

    /// The reproduce charge in `apply_reproduce`, as the same product
    /// expression the engine evaluates (T16.F01 invariant 3).
    fn reproduce_charge(sim: &Simulation, parent_id: CreatureId) -> f32 {
        let parent = &sim.creatures[parent_id];
        sim.config.energy.adjusted_action_cost(
            sim.config.energy.costs.reproduce_cost,
            parent.cached_complexity,
            parent.age,
        ) * reproduction::genome_replication_cost_multiplier(
            sim.config.energy.lifecycle.genome_replication_cost_per_unit,
            parent.cached_genome_size,
        )
    }

    /// T16.F01 invariant 2: a `RejectedEnergyConstraints` outcome leaves the
    /// parent and the reproduce/transfer flows untouched while the rejection
    /// counters still move. Energy is compared bit-exactly on purpose: the
    /// claim is "unchanged", not "close".
    fn assert_energy_rejection_is_free(energy: f32, transfer_request: f32, seed: u64) {
        let pos = Position::new(5, 5);
        let (mut sim, parent_id) = make_sim_one_creature(pos, energy);
        sim.creatures[parent_id].age = sim.config.energy.lifecycle.min_reproduce_age;
        let energy_before = sim.creatures[parent_id].energy;
        let death_cause_before = sim.creatures[parent_id].pending_death_cause;
        let charges_before = sim.stats.energy_flows.action_charges.reproduce;
        let transfer_before = sim.stats.energy_flows.parental_transfer_debit;
        let rejected_before = sim.stats.reproduction_actions_rejected_total;
        let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);

        let result = apply_reproduce(
            parent_id,
            &mut sim,
            Direction::N,
            transfer_request,
            &mut rng,
        );

        assert_eq!(result, ReproductionActionResult::RejectedEnergyConstraints);
        assert_eq!(sim.creatures.len(), 1, "no child should be spawned");
        assert_eq!(
            sim.creatures[parent_id].energy.to_bits(),
            energy_before.to_bits(),
            "energy-gate rejection must not charge the parent"
        );
        assert_eq!(
            sim.creatures[parent_id].pending_death_cause,
            death_cause_before
        );
        // The flows are compared exactly, not within a tolerance: the claim is
        // "unchanged, bit for bit", and nothing was added to them.
        assert_eq!(
            sim.stats.energy_flows.action_charges.reproduce, charges_before,
            "rejection must not land in action_charges.reproduce"
        );
        assert_eq!(
            sim.stats.energy_flows.parental_transfer_debit, transfer_before,
            "rejection must not land in parental_transfer_debit"
        );
        assert_eq!(
            sim.stats.reproduction_actions_rejected_total,
            rejected_before + 1
        );
        assert_eq!(
            sim.stats
                .reproduction_actions_rejected_by_reason
                .get(&ReproductionActionResult::RejectedEnergyConstraints),
            Some(&1)
        );
    }

    #[test]
    fn apply_reproduce_min_energy_rejection_does_not_charge_reproduce_cost() {
        // energy - cost lands below min_reproduce_energy (30.0).
        assert_energy_rejection_is_free(30.0, 20.0, 34);
    }

    #[test]
    fn apply_reproduce_infeasible_transfer_rejection_does_not_charge_reproduce_cost() {
        // energy - cost clears min_reproduce_energy but cannot cover a
        // transfer of default_offspring_energy (100.0).
        assert_energy_rejection_is_free(50.0, 100.0, 35);
    }

    #[test]
    fn apply_reproduce_non_positive_transfer_rejection_does_not_charge_reproduce_cost() {
        assert_energy_rejection_is_free(80.0, 0.0, 36);
    }

    #[test]
    fn apply_reproduce_accepts_post_charge_energy_exactly_at_min_reproduce_energy() {
        // The energy gate is `after_cost < min_reproduce_energy`: landing
        // exactly on the floor is accepted, as it was before T16.F01. The
        // floor is pinned to the engine's own f32 `energy - cost` so the
        // boundary is exact, not approximate.
        let pos = Position::new(5, 5);
        let (mut sim, parent_id) = make_sim_one_creature(pos, 80.0);
        sim.creatures[parent_id].age = sim.config.energy.lifecycle.min_reproduce_age;
        let energy_before = sim.creatures[parent_id].energy;
        let cost = reproduce_charge(&sim, parent_id);
        let after_cost = energy_before - cost;
        sim.config.energy.lifecycle.min_reproduce_energy = after_cost;
        let transfer = 20.0_f32;
        assert!(
            transfer <= after_cost,
            "fixture must leave only the min-energy gate in play"
        );
        let mut rng = rand::rngs::SmallRng::seed_from_u64(38);

        let result = apply_reproduce(parent_id, &mut sim, Direction::N, transfer, &mut rng);

        assert_eq!(result, ReproductionActionResult::Spawned);
        assert_eq!(sim.creatures.len(), 2, "one child should be spawned");
        assert_eq!(
            sim.creatures[parent_id].energy.to_bits(),
            (after_cost - transfer).to_bits()
        );
    }

    #[test]
    fn apply_reproduce_birth_pays_cost_then_transfer_as_two_subtractions() {
        // T16.F01 invariant 3: the accepted path pays `cost` then `transfer`
        // as two successive f32 subtractions, so the parent's post-birth
        // energy is bit-identical to the pre-feature engine's. Bit-exact
        // comparison is the point of this test.
        let pos = Position::new(5, 5);
        let (mut sim, parent_id) = make_sim_one_creature(pos, 80.0);
        sim.creatures[parent_id].age = sim.config.energy.lifecycle.min_reproduce_age;
        let energy_before = sim.creatures[parent_id].energy;
        let cost = reproduce_charge(&sim, parent_id);
        let transfer = 20.0_f32;
        let after_cost = energy_before - cost;
        let expected_energy = after_cost - transfer;
        let mut rng = rand::rngs::SmallRng::seed_from_u64(37);

        let result = apply_reproduce(parent_id, &mut sim, Direction::N, transfer, &mut rng);

        assert_eq!(result, ReproductionActionResult::Spawned);
        assert_eq!(
            sim.creatures[parent_id].energy.to_bits(),
            expected_energy.to_bits(),
            "parent must pay cost then transfer as two f32 subtractions"
        );
        assert_eq!(
            sim.stats.energy_flows.action_charges.reproduce,
            applied_debit(energy_before, after_cost),
            "action_charges.reproduce must carry exactly the reproduce charge"
        );
        assert_eq!(
            sim.stats.energy_flows.parental_transfer_debit,
            applied_debit(after_cost, expected_energy),
            "parental_transfer_debit must carry exactly the transfer"
        );
        let child = sim
            .creatures
            .iter()
            .find(|(id, _)| *id != parent_id)
            .map(|(_, c)| c.energy)
            .expect("one child should be spawned");
        assert_eq!(child.to_bits(), transfer.to_bits());
    }

    #[test]
    fn apply_reproduce_succeeds_when_parent_meets_min_reproduce_age() {
        let pos = Position::new(5, 5);
        let (mut sim, parent_id) = make_sim_one_creature(pos, 80.0);
        sim.config.energy.lifecycle.min_reproduce_age = 20;
        sim.creatures[parent_id].age = 20;
        let mut rng = rand::rngs::SmallRng::seed_from_u64(33);
        let result = apply_reproduce(parent_id, &mut sim, Direction::N, 20.0, &mut rng);
        assert_eq!(result, ReproductionActionResult::Spawned);
        assert_eq!(sim.creatures.len(), 2, "one child should be spawned");
    }

    #[test]
    fn apply_reproduce_deducts_cost_from_parent() {
        let pos = Position::new(5, 5);
        let (mut sim, parent_id) = make_sim_one_creature(pos, 80.0);
        sim.creatures[parent_id].age = sim.config.energy.lifecycle.min_reproduce_age;
        let cost = sim.config.energy.costs.reproduce_cost;
        let energy_before = sim.creatures[parent_id].energy;
        let mut rng = rand::rngs::SmallRng::seed_from_u64(4);
        let result = apply_reproduce(parent_id, &mut sim, Direction::N, 20.0, &mut rng);
        assert_eq!(result, ReproductionActionResult::Spawned);
        // Parent should have lost at least reproduce_cost.
        assert!(
            sim.creatures[parent_id].energy < energy_before - cost + f32::EPSILON,
            "parent energy not reduced by reproduce_cost"
        );
    }

    #[test]
    fn apply_reproduce_at_pop_cap_returns_rejected() {
        let pos = Position::new(5, 5);
        let (mut sim, parent_id) = make_sim_one_creature(pos, 80.0);
        sim.config.population.max_creatures = 1; // cap at current count
        let mut rng = rand::rngs::SmallRng::seed_from_u64(5);
        let result = apply_reproduce(parent_id, &mut sim, Direction::N, 20.0, &mut rng);
        assert_eq!(result, ReproductionActionResult::RejectedPopulationCap);
    }

    #[test]
    fn apply_reproduce_child_has_correct_generation() {
        let pos = Position::new(5, 5);
        let (mut sim, parent_id) = make_sim_one_creature(pos, 80.0);
        sim.creatures[parent_id].age = sim.config.energy.lifecycle.min_reproduce_age;
        let parent_gen = sim.creatures[parent_id].generation;
        let mut rng = rand::rngs::SmallRng::seed_from_u64(6);
        let result = apply_reproduce(parent_id, &mut sim, Direction::N, 20.0, &mut rng);
        assert_eq!(result, ReproductionActionResult::Spawned);
        let child = sim
            .creatures
            .values()
            .find(|c| c.generation == parent_gen + 1)
            .expect("child not found");
        assert_eq!(child.generation, parent_gen + 1);
    }

    #[test]
    fn apply_reproduce_child_inherits_shared_memory() {
        let pos = Position::new(5, 5);
        let (mut sim, parent_id) = make_sim_one_creature(pos, 80.0);
        sim.creatures[parent_id].age = sim.config.energy.lifecycle.min_reproduce_age;
        // Write distinctive values to parent shared_memory.
        sim.creatures[parent_id].shared_memory[3] = 0.42;
        sim.creatures[parent_id].shared_memory[7] = 0.99;
        let parent_gen = sim.creatures[parent_id].generation;
        let mut rng = rand::rngs::SmallRng::seed_from_u64(7);
        let result = apply_reproduce(parent_id, &mut sim, Direction::N, 20.0, &mut rng);
        assert_eq!(result, ReproductionActionResult::Spawned);
        let child = sim
            .creatures
            .values()
            .find(|c| c.generation == parent_gen + 1)
            .expect("child not found");
        assert!((child.shared_memory[3] - 0.42).abs() < f32::EPSILON);
        assert!((child.shared_memory[7] - 0.99).abs() < f32::EPSILON);
        // Child's prev_shared_memory must be zeroed (no "previous tick" for newborn).
        assert_eq!(child.prev_shared_memory, [0.0; 16]);
    }

    #[test]
    fn applied_event_triggers_phenotype_mutation() {
        let pos = Position::new(5, 5);
        let mut observed = false;

        for seed in 0u64..10_000 {
            let (mut sim, parent_id) = make_sim_one_creature(pos, 80.0);
            sim.creatures[parent_id].age = sim.config.energy.lifecycle.min_reproduce_age;
            sim.config.mutation.per_unit_supply_enabled = false;
            sim.config.mutation.mutation_probability = 1.0;
            sim.config.mutation.per_birth_mutation_events_min = 1;
            sim.config.mutation.per_birth_mutation_events_max = 1;
            sim.config.mutation.phenotype.channel_change_chance = 0.0;
            sim.config.mutation.phenotype.polarity_flip_chance = 0.0;
            sim.config.mutation.phenotype.channel_step = 1;
            sim.config.mutation.genome_size_pressure_enabled = true;
            sim.config.mutation.genome_size_cap = 1;

            let parent_channels = sim.creatures[parent_id].phenotype_channels;
            let parent_generation = sim.creatures[parent_id].generation;
            let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
            let result = apply_reproduce(parent_id, &mut sim, Direction::N, 20.0, &mut rng);

            if result == ReproductionActionResult::Spawned
                && sim.stats.mutation_events_applied_total > 0
            {
                let child = sim
                    .creatures
                    .values()
                    .find(|c| c.generation == parent_generation + 1)
                    .expect("child must exist when reproduction spawned");
                assert_ne!(
                    child.phenotype_channels, parent_channels,
                    "applied event must still trigger phenotype mutation"
                );
                observed = true;
                break;
            }
        }

        assert!(
            observed,
            "expected at least one spawned child with applied mutation"
        );
    }

    #[test]
    fn skipped_only_mutation_event_does_not_trigger_phenotype_mutation() {
        use crate::contracts::NodeId;
        use crate::creature::genome::{BackendDef, NodeGenome, VmBackendDef, VmInstruction};

        let pos = Position::new(5, 5);
        let mut observed = false;

        for seed in 0u64..10_000 {
            let (mut sim, parent_id) = make_sim_one_creature(pos, 80.0);
            sim.creatures[parent_id].age = sim.config.energy.lifecycle.min_reproduce_age;
            sim.config.mutation.per_unit_supply_enabled = false;
            sim.config.mutation.mutation_probability = 1.0;
            sim.config.mutation.per_birth_mutation_events_min = 1;
            sim.config.mutation.per_birth_mutation_events_max = 1;
            sim.config.mutation.phenotype.channel_change_chance = 0.0;
            sim.config.mutation.phenotype.polarity_flip_chance = 0.0;
            sim.config.mutation.phenotype.channel_step = 1;

            // Bias toward skipped mutations: no Graph nodes, no input refs, no route targets.
            sim.creatures[parent_id].genome.nodes = vec![NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![],
                    program: vec![VmInstruction::Halt],
                }),
                targets: vec![],
            }];

            let parent_channels = sim.creatures[parent_id].phenotype_channels;
            let parent_generation = sim.creatures[parent_id].generation;
            let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
            let result = apply_reproduce(parent_id, &mut sim, Direction::N, 20.0, &mut rng);
            if result == ReproductionActionResult::Spawned
                && sim.stats.mutation_events_applied_total == 0
                && sim.stats.mutation_events_skipped_total > 0
            {
                let child = sim
                    .creatures
                    .values()
                    .find(|c| c.generation == parent_generation + 1)
                    .expect("child must exist when reproduction spawned");
                assert_eq!(
                    child.phenotype_channels, parent_channels,
                    "skipped-only mutation event must not trigger phenotype mutation"
                );
                observed = true;
                break;
            }
        }

        assert!(
            observed,
            "expected at least one spawned child with skipped-only mutation event"
        );
    }

    /// T11.F17: each birth adds its executed-target events to the cumulative
    /// simulation total, so the counter grows across successive births.
    #[test]
    fn reproduce_accumulates_executed_target_totals_across_births() {
        let pos = Position::new(5, 5);
        let (mut sim, parent_id) = make_sim_one_creature(pos, 400.0);
        sim.creatures[parent_id].age = sim.config.energy.lifecycle.min_reproduce_age;
        sim.config.mutation.per_unit_supply_enabled = false;
        sim.config.mutation.mutation_probability = 1.0;
        sim.config.mutation.per_birth_mutation_events_min = 6;
        sim.config.mutation.per_birth_mutation_events_max = 6;
        // Stand in for lived ticks: the parent dispatched both mesh nodes.
        for index in 0..sim.creatures[parent_id].genome.nodes.len() {
            sim.creatures[parent_id]
                .graph_runtime
                .dispatch_record
                .record_dispatch(index);
        }

        let mut rng = rand::rngs::SmallRng::seed_from_u64(100);
        assert_eq!(
            apply_reproduce(parent_id, &mut sim, Direction::N, 20.0, &mut rng),
            ReproductionActionResult::Spawned
        );
        let after_first = sim.stats.mutation_executed_target_total;
        assert!(
            after_first > 0,
            "a parent with a live dispatch record contributes executed targets"
        );

        assert_eq!(
            apply_reproduce(parent_id, &mut sim, Direction::S, 20.0, &mut rng),
            ReproductionActionResult::Spawned
        );
        assert!(
            sim.stats.mutation_executed_target_total > after_first,
            "the second birth adds to the running total ({} then {})",
            after_first,
            sim.stats.mutation_executed_target_total
        );
        assert!(
            sim.stats.mutation_executed_target_total <= sim.stats.mutation_reachable_target_total,
            "executed targets are a subset of reachable targets on this genome"
        );
    }

    /// Every birth adds its per-operator funnel to the cumulative one, so the
    /// `applied` stage carries the running sum and decomposes the
    /// simulation-wide applied-event total.
    #[test]
    fn reproduce_accumulates_applied_operator_funnel_across_births() {
        // Arrange
        let (mut sim, parent_id) = make_sim_one_creature(Position::new(5, 5), 400.0);
        sim.creatures[parent_id].age = sim.config.energy.lifecycle.min_reproduce_age;
        sim.config.mutation.per_unit_supply_enabled = false;
        sim.config.mutation.mutation_probability = 1.0;
        sim.config.mutation.per_birth_mutation_events_min = 6;
        sim.config.mutation.per_birth_mutation_events_max = 6;
        let mut rng = rand::rngs::SmallRng::seed_from_u64(100);

        // Act
        assert_eq!(
            apply_reproduce(parent_id, &mut sim, Direction::N, 20.0, &mut rng),
            ReproductionActionResult::Spawned
        );

        // Assert: the first birth's applied events are in the funnel.
        let applied = applied_funnel_total(&sim);
        assert!(applied > 0, "this birth applies at least one event");
        assert_eq!(applied, sim.stats.mutation_events_applied_total);

        // Act: a second birth from the same parent.
        assert_eq!(
            apply_reproduce(parent_id, &mut sim, Direction::S, 20.0, &mut rng),
            ReproductionActionResult::Spawned
        );

        // Assert: the stage carries the sum over both births.
        let applied_after = applied_funnel_total(&sim);
        assert!(
            applied_after > applied,
            "the second birth adds its applied events ({applied} then {applied_after})"
        );
        assert_eq!(applied_after, sim.stats.mutation_events_applied_total);
    }

    /// An operator that is selected but whose result the parseability gate
    /// rejects lands in the funnel's `skipped` stage, and that stage is
    /// cumulative across births like every other.
    #[test]
    fn reproduce_accumulates_skipped_operator_funnel_across_births() {
        // Arrange: a duplicated node id leaves every mutated genome
        // unparseable, so each selected operator is rolled back and skipped.
        let (mut sim, parent_id) = make_sim_one_creature(Position::new(5, 5), 400.0);
        sim.creatures[parent_id].age = sim.config.energy.lifecycle.min_reproduce_age;
        sim.config.mutation.per_unit_supply_enabled = false;
        sim.config.mutation.mutation_probability = 1.0;
        sim.config.mutation.per_birth_mutation_events_min = 6;
        sim.config.mutation.per_birth_mutation_events_max = 6;
        let duplicate = sim.creatures[parent_id].genome.nodes[0].clone();
        sim.creatures[parent_id].genome.nodes.push(duplicate);
        let mut rng = rand::rngs::SmallRng::seed_from_u64(100);

        // Act
        assert_eq!(
            apply_reproduce(parent_id, &mut sim, Direction::N, 20.0, &mut rng),
            ReproductionActionResult::Spawned
        );

        // Assert: the rejected events are counted, and nothing applied.
        let skipped = skipped_funnel_total(&sim);
        assert!(skipped > 0, "a rejected operator is a skipped funnel event");
        assert_eq!(applied_funnel_total(&sim), 0);
        assert!(skipped <= sim.stats.mutation_events_skipped_total);

        // Act: a second birth from the same parent.
        assert_eq!(
            apply_reproduce(parent_id, &mut sim, Direction::S, 20.0, &mut rng),
            ReproductionActionResult::Spawned
        );

        // Assert: the stage carries the sum over both births.
        let skipped_after = skipped_funnel_total(&sim);
        assert!(
            skipped_after > skipped,
            "the second birth adds its skipped events ({skipped} then {skipped_after})"
        );
        assert!(skipped_after <= sim.stats.mutation_events_skipped_total);
    }

    #[test]
    fn reproduce_updates_domain_and_operator_mutation_stats() {
        let pos = Position::new(5, 5);
        let (mut sim, parent_id) = make_sim_one_creature(pos, 80.0);
        sim.creatures[parent_id].age = sim.config.energy.lifecycle.min_reproduce_age;
        sim.config.mutation.per_unit_supply_enabled = false;
        sim.config.mutation.mutation_probability = 1.0;
        sim.config.mutation.per_birth_mutation_events_min = 3;
        sim.config.mutation.per_birth_mutation_events_max = 3;

        let mut rng = rand::rngs::SmallRng::seed_from_u64(100);
        let result = apply_reproduce(parent_id, &mut sim, Direction::N, 20.0, &mut rng);
        assert_eq!(result, ReproductionActionResult::Spawned);

        let attempted_by_domain: u64 = sim
            .stats
            .mutation_events_attempted_total_by_domain
            .values()
            .sum();
        let applied_by_domain: u64 = sim
            .stats
            .mutation_events_applied_total_by_domain
            .values()
            .sum();
        let attempted_by_operator: u64 = sim
            .stats
            .mutation_events_attempted_total_by_operator
            .values()
            .sum();
        let applied_by_operator: u64 = sim
            .stats
            .mutation_events_applied_total_by_operator
            .values()
            .sum();

        assert_eq!(
            attempted_by_domain,
            sim.stats.mutation_events_attempted_total
        );
        assert_eq!(applied_by_domain, sim.stats.mutation_events_applied_total);
        assert_eq!(
            attempted_by_operator,
            sim.stats.mutation_events_attempted_total
        );
        assert_eq!(applied_by_operator, sim.stats.mutation_events_applied_total);
        assert!(
            !sim.stats
                .mutation_events_attempted_total_by_domain
                .is_empty(),
            "expected at least one attempted domain counter entry"
        );
        assert!(
            !sim.stats
                .mutation_events_attempted_total_by_operator
                .is_empty(),
            "expected at least one attempted operator counter entry"
        );
    }

    // ── apply_move return value ─────────────────────────────────────────────

    #[test]
    fn apply_move_returns_true_on_success() {
        let start = Position::new(5, 5);
        let (mut sim, id) = make_sim_one_creature(start, 50.0);
        let creature = sim.creatures.get_mut(id).unwrap();
        let result = apply_move(
            id,
            creature,
            &mut sim.world,
            Direction::N,
            &sim.config,
            &mut sim.stats.energy_flows,
        );
        assert!(result, "successful move should return true");
    }

    #[test]
    fn apply_move_returns_false_on_barrier() {
        let start = Position::new(5, 5);
        let (mut sim, id) = make_sim_one_creature(start, 50.0);
        sim.world.set_barrier(Position::new(5, 4), true);
        let creature = sim.creatures.get_mut(id).unwrap();
        let result = apply_move(
            id,
            creature,
            &mut sim.world,
            Direction::N,
            &sim.config,
            &mut sim.stats.energy_flows,
        );
        assert!(!result, "blocked move should return false");
    }

    #[test]
    fn apply_move_returns_false_on_occupied() {
        let pos1 = Position::new(5, 5);
        let pos2 = Position::new(5, 4);
        let cfg = small_config();
        let mut world = WorldState::new(cfg.world.width, cfg.world.height, cfg.world.edge_mode);
        let mut creatures: SlotMap<CreatureId, CreatureState> = SlotMap::with_key();
        let id1 = creatures.insert_with_key(|id| {
            CreatureState::new(
                id,
                v3alpha1_founder_genome(),
                pos1,
                50.0,
                0,
                [0, 0, 92, 92, 138, 138],
                0,
                [true; 6],
                CreatureIdentityState::default(),
                [0.0; 16],
            )
        });
        let _id2 = creatures.insert_with_key(|id| {
            CreatureState::new(
                id,
                v3alpha1_founder_genome(),
                pos2,
                50.0,
                0,
                [0, 0, 92, 92, 138, 138],
                0,
                [true; 6],
                CreatureIdentityState::default(),
                [0.0; 16],
            )
        });
        world.place_creature(pos1, id1);
        world.place_creature(pos2, _id2);
        let mut sim = Simulation {
            world,
            creatures,
            action_logs: slotmap::SecondaryMap::new(),
            tick: 0,
            config: cfg,
            stats: crate::simulation::stats::SimStats::default(),
            rng: rand::rngs::SmallRng::seed_from_u64(0),
        };
        let creature = sim.creatures.get_mut(id1).unwrap();
        let result = apply_move(
            id1,
            creature,
            &mut sim.world,
            Direction::N,
            &sim.config,
            &mut sim.stats.energy_flows,
        );
        assert!(!result, "move into occupied cell should return false");
    }

    // ── typed Eat return value ──────────────────────────────────────────────

    #[test]
    fn apply_typed_eat_returns_true_with_food() {
        let pos = Position::new(3, 3);
        let (mut sim, id) = make_sim_one_creature(pos, 10.0);
        sim.world.set_food(pos, 0.5);
        let creature = sim.creatures.get_mut(id).unwrap();
        let result = apply_typed_eat(
            creature,
            &mut sim.world,
            &sim.config,
            OrdinaryFoodTypeId::default(),
            &mut sim.stats.energy_flows,
        );
        assert!(result, "eating food should return true");
    }

    #[test]
    fn apply_typed_eat_returns_false_without_food() {
        let pos = Position::new(3, 3);
        let (mut sim, id) = make_sim_one_creature(pos, 10.0);
        // No food seeded — cell has 0 food
        let creature = sim.creatures.get_mut(id).unwrap();
        let result = apply_typed_eat(
            creature,
            &mut sim.world,
            &sim.config,
            OrdinaryFoodTypeId::default(),
            &mut sim.stats.energy_flows,
        );
        assert!(!result, "eating empty cell should return false");
    }

    #[test]
    fn apply_typed_eat_rejects_an_unconfigured_food_type_without_consumption() {
        let pos = Position::new(3, 3);
        let (mut sim, id) = make_sim_one_creature(pos, 10.0);
        sim.world
            .set_food_type(pos, OrdinaryFoodTypeId::new(0), 0.5);
        let food_before = sim.world.food_at_type(pos, OrdinaryFoodTypeId::new(0));
        let creature = sim.creatures.get_mut(id).unwrap();

        let result = apply_typed_eat(
            creature,
            &mut sim.world,
            &sim.config,
            OrdinaryFoodTypeId::new(99),
            &mut sim.stats.energy_flows,
        );

        assert!(!result);
        assert_eq!(
            sim.world.food_at_type(pos, OrdinaryFoodTypeId::new(0)),
            food_before
        );
    }

    // ── complexity energy cost integration tests ────────────────────────────

    /// Build a genome with a specific minimum complexity by adding VM nodes
    /// with enough instructions.
    fn genome_with_complexity(min_complexity: u32) -> crate::creature::genome::CreatureGenome {
        use crate::contracts::NodeId;
        use crate::creature::genome::{
            BackendDef, CreatureGenome, NodeGenome, VmBackendDef, VmInstruction,
        };

        // Each PushAction is an output instruction → counts as live for functional complexity.
        // Node itself = 1, each PushAction = 1 live instruction.
        let instruction_count = (min_complexity.saturating_sub(1)) as usize;
        let program = vec![VmInstruction::PushAction { action_type: 0 }; instruction_count];
        CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![],
                    program,
                }),
                targets: vec![],
            }],
        }
    }

    /// Create a sim with one creature that has the given genome.
    fn make_sim_with_genome(
        pos: Position,
        energy: f32,
        genome: crate::creature::genome::CreatureGenome,
    ) -> (Simulation, CreatureId) {
        let mut cfg = small_config();
        cfg.energy.complexity_cost.enabled = true;
        let mut world = WorldState::new(cfg.world.width, cfg.world.height, cfg.world.edge_mode);
        let mut creatures: SlotMap<CreatureId, CreatureState> = SlotMap::with_key();
        let id = creatures.insert_with_key(|id| {
            CreatureState::new(
                id,
                genome,
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
    fn complexity_noop_cost_scales_with_genome_complexity() {
        // Low-complexity creature (below threshold=50): pays base noop_cost
        let low_genome = genome_with_complexity(10);
        assert!(low_genome.complexity() <= 50);
        let (mut sim_low, id_low) = make_sim_with_genome(Position::new(5, 5), 100.0, low_genome);
        let energy_before_low = sim_low.creatures[id_low].energy;
        apply_noop(
            sim_low.creatures.get_mut(id_low).unwrap(),
            &sim_low.config,
            &mut sim_low.stats.energy_flows,
        );
        let cost_low = energy_before_low - sim_low.creatures[id_low].energy;

        // High-complexity creature (well above threshold): pays more
        let high_genome = genome_with_complexity(300);
        assert!(high_genome.complexity() > 50);
        let (mut sim_high, id_high) = make_sim_with_genome(Position::new(5, 5), 100.0, high_genome);
        let energy_before_high = sim_high.creatures[id_high].energy;
        apply_noop(
            sim_high.creatures.get_mut(id_high).unwrap(),
            &sim_high.config,
            &mut sim_high.stats.energy_flows,
        );
        let cost_high = energy_before_high - sim_high.creatures[id_high].energy;

        assert!(
            cost_high > cost_low,
            "high-complexity creature should pay more for noop: low={cost_low}, high={cost_high}"
        );
    }

    #[test]
    fn complexity_eat_cost_scales_with_genome_complexity() {
        let low_genome = genome_with_complexity(10);
        let high_genome = genome_with_complexity(300);

        // Low-complexity
        let (mut sim_low, id_low) = make_sim_with_genome(Position::new(3, 3), 100.0, low_genome);
        sim_low.config.energy.costs.eat_cost = 2.0; // non-zero eat cost
        let energy_before_low = sim_low.creatures[id_low].energy;
        {
            let creature = sim_low.creatures.get_mut(id_low).unwrap();
            let _ = apply_typed_eat(
                creature,
                &mut sim_low.world,
                &sim_low.config,
                OrdinaryFoodTypeId::default(),
                &mut sim_low.stats.energy_flows,
            );
        }
        let cost_low = energy_before_low - sim_low.creatures[id_low].energy;

        // High-complexity
        let (mut sim_high, id_high) = make_sim_with_genome(Position::new(3, 3), 100.0, high_genome);
        sim_high.config.energy.costs.eat_cost = 2.0;
        let energy_before_high = sim_high.creatures[id_high].energy;
        {
            let creature = sim_high.creatures.get_mut(id_high).unwrap();
            let _ = apply_typed_eat(
                creature,
                &mut sim_high.world,
                &sim_high.config,
                OrdinaryFoodTypeId::default(),
                &mut sim_high.stats.energy_flows,
            );
        }
        let cost_high = energy_before_high - sim_high.creatures[id_high].energy;

        // Both ate from empty cells (no food), so only eat_cost applies.
        // High complexity should pay more.
        assert!(
            cost_high > cost_low,
            "high-complexity creature should pay more eat_cost: low={cost_low}, high={cost_high}"
        );
    }

    #[test]
    fn complexity_move_cost_scales_with_genome_complexity() {
        let low_genome = genome_with_complexity(10);
        let high_genome = genome_with_complexity(300);

        // Low-complexity
        let (mut sim_low, id_low) = make_sim_with_genome(Position::new(5, 5), 100.0, low_genome);
        let energy_before_low = sim_low.creatures[id_low].energy;
        {
            let creature = sim_low.creatures.get_mut(id_low).unwrap();
            let _ = apply_move(
                id_low,
                creature,
                &mut sim_low.world,
                Direction::N,
                &sim_low.config,
                &mut sim_low.stats.energy_flows,
            );
        }
        let cost_low = energy_before_low - sim_low.creatures[id_low].energy;

        // High-complexity
        let (mut sim_high, id_high) = make_sim_with_genome(Position::new(5, 5), 100.0, high_genome);
        let energy_before_high = sim_high.creatures[id_high].energy;
        {
            let creature = sim_high.creatures.get_mut(id_high).unwrap();
            let _ = apply_move(
                id_high,
                creature,
                &mut sim_high.world,
                Direction::N,
                &sim_high.config,
                &mut sim_high.stats.energy_flows,
            );
        }
        let cost_high = energy_before_high - sim_high.creatures[id_high].energy;

        assert!(
            cost_high > cost_low,
            "high-complexity creature should pay more move_cost: low={cost_low}, high={cost_high}"
        );
    }

    #[test]
    fn complexity_reproduce_cost_scales_with_genome_complexity() {
        let low_genome = genome_with_complexity(10);
        let high_genome = genome_with_complexity(300);

        // Low-complexity parent — record energy after reproduce
        let (mut sim_low, parent_low) = make_sim_with_genome(Position::new(5, 5), 80.0, low_genome);
        sim_low.creatures[parent_low].age = sim_low.config.energy.lifecycle.min_reproduce_age;
        let energy_before_low = sim_low.creatures[parent_low].energy;
        let mut rng = rand::rngs::SmallRng::seed_from_u64(100);
        let _ = apply_reproduce(parent_low, &mut sim_low, Direction::N, 20.0, &mut rng);
        let cost_low = energy_before_low - sim_low.creatures[parent_low].energy;

        // High-complexity parent
        let (mut sim_high, parent_high) =
            make_sim_with_genome(Position::new(5, 5), 80.0, high_genome);
        sim_high.creatures[parent_high].age = sim_high.config.energy.lifecycle.min_reproduce_age;
        let energy_before_high = sim_high.creatures[parent_high].energy;
        let mut rng = rand::rngs::SmallRng::seed_from_u64(100);
        let _ = apply_reproduce(parent_high, &mut sim_high, Direction::N, 20.0, &mut rng);
        let cost_high = energy_before_high - sim_high.creatures[parent_high].energy;

        // The reproduce_cost portion should be higher for complex creatures.
        // Both deduct transfer + cost, but the cost portion should scale.
        assert!(
            cost_high > cost_low,
            "high-complexity creature should pay more reproduce_cost: low={cost_low}, high={cost_high}"
        );
    }

    #[test]
    fn complexity_steal_cost_scales_with_genome_complexity() {
        let low_genome = genome_with_complexity(10);
        let high_genome = genome_with_complexity(300);

        // Set up low-complexity attacker with victim
        let (mut sim_low, attacker_low) =
            make_sim_with_genome(Position::new(5, 5), 100.0, low_genome);
        // Add a victim to the north
        let victim_pos = Position::new(5, 4);
        let victim_id_low = sim_low.creatures.insert_with_key(|id| {
            CreatureState::new(
                id,
                v3alpha1_founder_genome(),
                victim_pos,
                50.0,
                0,
                [0, 0, 92, 92, 138, 138],
                0,
                [true; 6],
                CreatureIdentityState::default(),
                [0.0; 16],
            )
        });
        sim_low.world.place_creature(victim_pos, victim_id_low);
        let energy_before_low = sim_low.creatures[attacker_low].energy;
        let _ = apply_steal_energy(attacker_low, &mut sim_low, Direction::N, 10.0);
        let cost_low = energy_before_low - sim_low.creatures[attacker_low].energy;

        // Set up high-complexity attacker with victim
        let (mut sim_high, attacker_high) =
            make_sim_with_genome(Position::new(5, 5), 100.0, high_genome);
        let victim_id_high = sim_high.creatures.insert_with_key(|id| {
            CreatureState::new(
                id,
                v3alpha1_founder_genome(),
                victim_pos,
                50.0,
                0,
                [0, 0, 92, 92, 138, 138],
                0,
                [true; 6],
                CreatureIdentityState::default(),
                [0.0; 16],
            )
        });
        sim_high.world.place_creature(victim_pos, victim_id_high);
        let energy_before_high = sim_high.creatures[attacker_high].energy;
        let _ = apply_steal_energy(attacker_high, &mut sim_high, Direction::N, 10.0);
        let cost_high = energy_before_high - sim_high.creatures[attacker_high].energy;

        // Both steal 10.0 from victim and get 10.0 back, but the cost (steal_cost_rate * amount)
        // should be higher for complex creatures. Net cost = steal_cost - stolen_energy.
        // Since stolen amounts are equal, the difference is in the steal_cost.
        assert!(
            cost_high > cost_low,
            "high-complexity attacker should pay more steal cost: low={cost_low}, high={cost_high}"
        );
    }

    #[test]
    fn seeded_founders_start_with_correct_energy() {
        let cfg = {
            let mut c = small_config();
            c.population.initial_creatures = 3;
            c
        };
        let expected = cfg.energy.lifecycle.initial_energy;
        let sim = seed_simulation(cfg, 42);
        for (_, c) in &sim.creatures {
            assert!((c.energy - expected).abs() < f32::EPSILON);
        }
    }

    // ── Age-adjusted cost integration tests ───────────────────────────────

    #[test]
    fn age_noop_cost_increases_with_age() {
        let (mut sim_young, id_young) = make_sim_one_creature(Position::new(5, 5), 100.0);
        let energy_before_young = sim_young.creatures[id_young].energy;
        apply_noop(
            sim_young.creatures.get_mut(id_young).unwrap(),
            &sim_young.config,
            &mut sim_young.stats.energy_flows,
        );
        let cost_young = energy_before_young - sim_young.creatures[id_young].energy;

        let (mut sim_old, id_old) = make_sim_one_creature(Position::new(5, 5), 100.0);
        sim_old.creatures[id_old].age = 400;
        let energy_before_old = sim_old.creatures[id_old].energy;
        apply_noop(
            sim_old.creatures.get_mut(id_old).unwrap(),
            &sim_old.config,
            &mut sim_old.stats.energy_flows,
        );
        let cost_old = energy_before_old - sim_old.creatures[id_old].energy;

        assert!(
            cost_old > cost_young,
            "old creature (age=400) should pay more for noop: young={cost_young}, old={cost_old}"
        );
    }

    #[test]
    fn age_eat_cost_increases_with_age() {
        let (mut sim_young, id_young) = make_sim_one_creature(Position::new(3, 3), 100.0);
        sim_young.config.energy.costs.eat_cost = 2.0;
        let energy_before_young = sim_young.creatures[id_young].energy;
        {
            let creature = sim_young.creatures.get_mut(id_young).unwrap();
            let _ = apply_typed_eat(
                creature,
                &mut sim_young.world,
                &sim_young.config,
                OrdinaryFoodTypeId::default(),
                &mut sim_young.stats.energy_flows,
            );
        }
        let cost_young = energy_before_young - sim_young.creatures[id_young].energy;

        let (mut sim_old, id_old) = make_sim_one_creature(Position::new(3, 3), 100.0);
        sim_old.config.energy.costs.eat_cost = 2.0;
        sim_old.creatures[id_old].age = 400;
        let energy_before_old = sim_old.creatures[id_old].energy;
        {
            let creature = sim_old.creatures.get_mut(id_old).unwrap();
            let _ = apply_typed_eat(
                creature,
                &mut sim_old.world,
                &sim_old.config,
                OrdinaryFoodTypeId::default(),
                &mut sim_old.stats.energy_flows,
            );
        }
        let cost_old = energy_before_old - sim_old.creatures[id_old].energy;

        assert!(
            cost_old > cost_young,
            "old creature should pay more eat_cost: young={cost_young}, old={cost_old}"
        );
    }

    #[test]
    fn age_move_cost_increases_with_age() {
        let (mut sim_young, id_young) = make_sim_one_creature(Position::new(5, 5), 100.0);
        let energy_before_young = sim_young.creatures[id_young].energy;
        {
            let creature = sim_young.creatures.get_mut(id_young).unwrap();
            let _ = apply_move(
                id_young,
                creature,
                &mut sim_young.world,
                Direction::N,
                &sim_young.config,
                &mut sim_young.stats.energy_flows,
            );
        }
        let cost_young = energy_before_young - sim_young.creatures[id_young].energy;

        let (mut sim_old, id_old) = make_sim_one_creature(Position::new(5, 5), 100.0);
        sim_old.creatures[id_old].age = 400;
        let energy_before_old = sim_old.creatures[id_old].energy;
        {
            let creature = sim_old.creatures.get_mut(id_old).unwrap();
            let _ = apply_move(
                id_old,
                creature,
                &mut sim_old.world,
                Direction::N,
                &sim_old.config,
                &mut sim_old.stats.energy_flows,
            );
        }
        let cost_old = energy_before_old - sim_old.creatures[id_old].energy;

        assert!(
            cost_old > cost_young,
            "old creature should pay more move_cost: young={cost_young}, old={cost_old}"
        );
    }

    #[test]
    fn age_reproduce_cost_increases_with_age() {
        let (mut sim_young, parent_young) = make_sim_one_creature(Position::new(5, 5), 80.0);
        let energy_before_young = sim_young.creatures[parent_young].energy;
        let mut rng = rand::rngs::SmallRng::seed_from_u64(100);
        let _ = apply_reproduce(parent_young, &mut sim_young, Direction::N, 20.0, &mut rng);
        let cost_young = energy_before_young - sim_young.creatures[parent_young].energy;

        let (mut sim_old, parent_old) = make_sim_one_creature(Position::new(5, 5), 80.0);
        sim_old.creatures[parent_old].age = 400;
        let energy_before_old = sim_old.creatures[parent_old].energy;
        let mut rng = rand::rngs::SmallRng::seed_from_u64(100);
        let _ = apply_reproduce(parent_old, &mut sim_old, Direction::N, 20.0, &mut rng);
        let cost_old = energy_before_old - sim_old.creatures[parent_old].energy;

        assert!(
            cost_old > cost_young,
            "old creature should pay more reproduce_cost: young={cost_young}, old={cost_old}"
        );
    }

    #[test]
    fn age_steal_cost_increases_with_age() {
        let victim_pos = Position::new(5, 4);

        // Young attacker
        let (mut sim_young, attacker_young) = make_sim_one_creature(Position::new(5, 5), 100.0);
        let victim_young = sim_young.creatures.insert_with_key(|id| {
            CreatureState::new(
                id,
                v3alpha1_founder_genome(),
                victim_pos,
                50.0,
                0,
                [0, 0, 92, 92, 138, 138],
                0,
                [true; 6],
                CreatureIdentityState::default(),
                [0.0; 16],
            )
        });
        sim_young.world.place_creature(victim_pos, victim_young);
        let energy_before_young = sim_young.creatures[attacker_young].energy;
        let _ = apply_steal_energy(attacker_young, &mut sim_young, Direction::N, 10.0);
        let cost_young = energy_before_young - sim_young.creatures[attacker_young].energy;

        // Old attacker
        let (mut sim_old, attacker_old) = make_sim_one_creature(Position::new(5, 5), 100.0);
        sim_old.creatures[attacker_old].age = 400;
        let victim_old = sim_old.creatures.insert_with_key(|id| {
            CreatureState::new(
                id,
                v3alpha1_founder_genome(),
                victim_pos,
                50.0,
                0,
                [0, 0, 92, 92, 138, 138],
                0,
                [true; 6],
                CreatureIdentityState::default(),
                [0.0; 16],
            )
        });
        sim_old.world.place_creature(victim_pos, victim_old);
        let energy_before_old = sim_old.creatures[attacker_old].energy;
        let _ = apply_steal_energy(attacker_old, &mut sim_old, Direction::N, 10.0);
        let cost_old = energy_before_old - sim_old.creatures[attacker_old].energy;

        // Both steal from victim, but old attacker's cost portion is higher.
        // Net energy change = -cost + stolen. Cost is higher for old, so net is more negative.
        assert!(
            cost_old > cost_young,
            "old attacker should have higher net cost: young={cost_young}, old={cost_old}"
        );
    }
    proptest::proptest! {
        #[test]
        fn typed_eat_shared_reward_is_independent_of_type(density in 0.001f32..1.0, reward in 0.0f32..20.0) {
            let pos = Position::new(3, 3);
            let (mut sim, id) = make_sim_one_creature(pos, 10.0);
            sim.config.world.food.types.push(crate::config::FoodTypeConfig::default());
            sim.config.energy.costs.eat_reward_per_food = reward;
            sim.world.reconfigure_food(sim.config.world.food.clone());
            for selected in 0..2 {
                sim.creatures[id].energy = 10.0;
                for index in 0..2 { sim.world.set_food_type(pos, OrdinaryFoodTypeId::new(index), density); }
                let applied = apply_typed_eat(&mut sim.creatures[id], &mut sim.world, &sim.config, OrdinaryFoodTypeId::new(selected), &mut sim.stats.energy_flows);
                proptest::prop_assert!(applied);
                proptest::prop_assert!((sim.creatures[id].energy - (10.0 + density * reward)).abs() < 1e-5);
                proptest::prop_assert_eq!(sim.world.food_at_type(pos, OrdinaryFoodTypeId::new(selected)), 0.0);
                proptest::prop_assert_eq!(sim.world.food_at_type(pos, OrdinaryFoodTypeId::new(1-selected)), density);
            }
        }
    }

    #[test]
    fn reproduction_rejection_charging_order_and_transfer_accounting() {
        // The three early gates must reject before charging, even when energy is also insufficient.
        for gate in 0..3 {
            let (mut sim, id) = make_sim_one_creature(Position::new(5, 5), 1.0);
            sim.creatures[id].age = 20;
            let expected = match gate {
                0 => {
                    sim.world.set_barrier(Position::new(5, 4), true);
                    ReproductionActionResult::RejectedInvalidTarget
                }
                1 => {
                    sim.config.population.max_creatures = 1;
                    ReproductionActionResult::RejectedPopulationCap
                }
                _ => {
                    sim.creatures[id].age = 19;
                    ReproductionActionResult::RejectedAgeConstraints
                }
            };
            assert_eq!(
                apply_reproduce(
                    id,
                    &mut sim,
                    Direction::N,
                    20.0,
                    &mut rand::rngs::SmallRng::seed_from_u64(1)
                ),
                expected
            );
            assert_eq!(sim.creatures[id].energy, 1.0);
            assert_eq!(sim.stats.energy_flows.action_charges.reproduce, 0.0);
            assert_eq!(sim.stats.energy_flows.parental_transfer_debit, 0.0);
            assert_eq!(sim.stats.energy_flows.offspring_energy_credit, 0.0);
        }
        for (energy, request) in [
            (20.0, 10.0),
            (80.0, 0.0),
            (80.0, -1.0),
            (80.0, f32::NAN),
            (80.0, f32::INFINITY),
            (40.0, 50.0),
        ] {
            let (mut sim, id) = make_sim_one_creature(Position::new(5, 5), energy);
            sim.creatures[id].age = 20;
            assert_eq!(
                apply_reproduce(
                    id,
                    &mut sim,
                    Direction::N,
                    request,
                    &mut rand::rngs::SmallRng::seed_from_u64(1)
                ),
                ReproductionActionResult::RejectedEnergyConstraints
            );
            // T16.F01: the energy gates reject before the charge, so a
            // rejected attempt is free (bit-exact "unchanged" comparison).
            assert_eq!(sim.creatures[id].energy.to_bits(), energy.to_bits());
            assert_eq!(sim.stats.energy_flows.action_charges.reproduce, 0.0);
            assert_eq!(sim.stats.energy_flows.parental_transfer_debit, 0.0);
            assert_eq!(sim.stats.energy_flows.offspring_energy_credit, 0.0);
            assert_eq!(sim.creatures.len(), 1);
        }
        let (mut sim, id) = make_sim_one_creature(Position::new(5, 5), 180.0);
        sim.creatures[id].age = 20;
        let cost = sim.config.energy.adjusted_action_cost(
            sim.config.energy.costs.reproduce_cost,
            sim.creatures[id].cached_complexity,
            20,
        );
        assert_eq!(
            apply_reproduce(
                id,
                &mut sim,
                Direction::N,
                150.0,
                &mut rand::rngs::SmallRng::seed_from_u64(1)
            ),
            ReproductionActionResult::Spawned
        );
        assert!((sim.creatures[id].energy - (180.0 - cost - 100.0)).abs() < 1e-5);
        assert_eq!(
            sim.creatures
                .values()
                .find(|child| child.id != id)
                .unwrap()
                .energy,
            100.0
        );
    }
    #[test]
    fn empty_eat_does_not_clamp_energy_after_live_maximum_is_lowered() {
        let (mut sim, id) = make_sim_one_creature(Position::new(3, 3), 50.0);
        sim.config.energy.lifecycle.max_energy = 20.0;
        sim.config.energy.costs.eat_cost = 2.0;
        assert!(!apply_typed_eat(
            &mut sim.creatures[id],
            &mut sim.world,
            &sim.config,
            OrdinaryFoodTypeId::default(),
            &mut sim.stats.energy_flows
        ));
        assert_eq!(sim.creatures[id].energy, 48.0);
    }

    proptest::proptest! {
        #[test]
        fn reproduction_result_keys_match_public_contract((result, key) in proptest::sample::select(vec![
            (ReproductionActionResult::Spawned, "Spawned"),
            (ReproductionActionResult::RejectedInvalidTarget, "RejectedInvalidTarget"),
            (ReproductionActionResult::RejectedAgeConstraints, "RejectedAgeConstraints"),
            (ReproductionActionResult::RejectedEnergyConstraints, "RejectedEnergyConstraints"),
            (ReproductionActionResult::RejectedPopulationCap, "RejectedPopulationCap"),
        ])) {
            proptest::prop_assert_eq!(result.as_key(), key);
        }
    }
}
