use rand::Rng;

use crate::config::SimulationConfig;
use crate::contracts::{CreatureId, Direction};
use crate::creature::state::CreatureState;
use crate::kernel::WorldState;
use crate::mutation::MutationEngine;
use crate::simulation::simulation::Simulation;

/// Result of an attempted reproduction action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReproductionActionResult {
    Spawned,
    RejectedInvalidTarget,
    RejectedEnergyConstraints,
    RejectedPopulationCap,
}

// ─── Action application functions ─────────────────────────────────────────────

/// Apply a NoOp action (deduct noop cost).
pub fn apply_noop(creature: &mut CreatureState, config: &SimulationConfig) {
    creature.energy -= config.energy.costs.noop_cost;
}

/// Apply an Eat action: consume all food on the creature's cell, reward energy, cap at max.
pub fn apply_eat(creature: &mut CreatureState, world: &mut WorldState, config: &SimulationConfig) {
    let food = world.consume_food(creature.position);
    creature.energy += food as f32 * config.energy.costs.eat_reward_per_food;
    creature.energy = creature.energy.min(config.energy.lifecycle.max_energy);
    creature.energy -= config.energy.costs.eat_cost;
}

/// Apply a Move action: move one step in `dir` if the target is valid.
///
/// Energy cost is always deducted even if the move is rejected (blocked cell).
pub fn apply_move(
    id: CreatureId,
    creature: &mut CreatureState,
    world: &mut WorldState,
    dir: Direction,
    config: &SimulationConfig,
) {
    let target = world
        .resolve_neighbor(creature.position, dir)
        .filter(|&p| world.is_valid_target_cell(p));

    if let Some(target_pos) = target {
        world.remove_creature(creature.position);
        world.place_creature(target_pos, id);
        creature.position = target_pos;
    }

    creature.energy -= config.energy.costs.move_cost;
}

/// Apply a Reproduce action per v3-reproduction-spec.md Section 6 unified sequence.
///
/// Returns the outcome indicating whether offspring was spawned or why it was rejected.
pub fn apply_reproduce(
    parent_id: CreatureId,
    sim: &mut Simulation,
    dir: Direction,
    energy_transfer_request: f32,
    rng: &mut impl Rng,
) -> ReproductionActionResult {
    let parent_pos = sim.creatures[parent_id].position;

    // Step 1: Resolve target cell.
    let target = match sim.world.resolve_neighbor(parent_pos, dir) {
        Some(p) => p,
        None => return ReproductionActionResult::RejectedInvalidTarget,
    };

    // Step 2: Validate target cell (no barrier, not occupied).
    if !sim.world.is_valid_target_cell(target) {
        return ReproductionActionResult::RejectedInvalidTarget;
    }

    // Step 3: Check population cap.
    if sim.creatures.len() >= sim.config.population.max_creatures as usize {
        return ReproductionActionResult::RejectedPopulationCap;
    }

    // Step 4: Deduct reproduce_cost from parent.
    sim.creatures[parent_id].energy -= sim.config.energy.costs.reproduce_cost;

    // Step 5: Check parent has sufficient energy after cost deduction.
    if sim.creatures[parent_id].energy < sim.config.energy.lifecycle.min_reproduce_energy {
        return ReproductionActionResult::RejectedEnergyConstraints;
    }

    // Step 6: Compute energy transfer (clamped to [0, default_offspring_energy]).
    let max_transfer = sim.config.energy.lifecycle.default_offspring_energy;
    let transfer = if energy_transfer_request.is_finite() && energy_transfer_request > 0.0 {
        energy_transfer_request.min(max_transfer)
    } else {
        0.0
    };

    if transfer <= 0.0 || sim.creatures[parent_id].energy < transfer {
        return ReproductionActionResult::RejectedEnergyConstraints;
    }

    // Step 7: Deduct transfer from parent.
    sim.creatures[parent_id].energy -= transfer;

    // Step 8: Build offspring draft (clone parent genome + state).
    let child_genome = sim.creatures[parent_id].genome.clone();
    let child_memory = sim.creatures[parent_id].memory;
    let child_generation = sim.creatures[parent_id].generation + 1;
    let child_rgb = sim.creatures[parent_id].phenotype_rgb;
    let child_weights = sim.creatures[parent_id].phenotype_channel_weights;
    let child_polarity = sim.creatures[parent_id].phenotype_channel_polarity;

    // Step 9: Apply mutations (Stage 4 stub — always returns zero summary).
    let mut child_genome = child_genome;
    let _summary = MutationEngine::apply_mutations(&mut child_genome, &sim.config.mutation, rng);

    // Step 10: Phenotype mutation skipped (stub applies 0 events in Stage 4).

    // Step 11–12: Spawn child in slotmap + world.
    let child_id = sim.creatures.insert_with_key(|id| {
        let mut child = CreatureState::new(
            id,
            child_genome,
            target,
            transfer,
            child_generation,
            child_rgb,
            child_weights,
            child_polarity,
        );
        child.memory = child_memory;
        child
    });
    sim.world.place_creature(target, child_id);

    ReproductionActionResult::Spawned
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SimulationConfig;
    use crate::contracts::Position;
    use crate::creature::founder::v3alpha1_founder_genome;
    use crate::kernel::WorldState;
    use crate::simulation::seeding::seed_simulation;
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
        let mut creatures: SlotMap<CreatureId, CreatureState> = SlotMap::with_key();
        let id = creatures.insert_with_key(|id| {
            CreatureState::new(
                id,
                v3alpha1_founder_genome(),
                pos,
                energy,
                0,
                [204, 61, 61],
                [1.0f32; 3],
                [true; 3],
            )
        });
        world.place_creature(pos, id);
        let sim = Simulation {
            world,
            creatures,
            tick: 0,
            config: cfg,
            rng: rand::rngs::SmallRng::seed_from_u64(42),
        };
        (sim, id)
    }

    // ── apply_eat ──────────────────────────────────────────────────────────────

    #[test]
    fn apply_eat_increases_energy_and_clears_food() {
        let pos = Position::new(3, 3);
        let (mut sim, id) = make_sim_one_creature(pos, 10.0);
        // Place food on the creature's cell.
        sim.world
            .seed_food(&mut rand::rngs::SmallRng::seed_from_u64(0), &{
                let mut cfg = small_config();
                cfg.world.food.initial_coverage = 1.0;
                cfg.world.food.initial_density = 100;
                cfg
            });
        let energy_before = sim.creatures[id].energy;
        let creature = sim.creatures.get_mut(id).unwrap();
        apply_eat(creature, &mut sim.world, &sim.config);
        assert!(
            sim.creatures[id].energy > energy_before,
            "eat should increase energy"
        );
        assert_eq!(sim.world.food_at(pos), 0, "food should be consumed");
    }

    #[test]
    fn apply_eat_caps_energy_at_max() {
        let pos = Position::new(3, 3);
        let (mut sim, id) = make_sim_one_creature(pos, 95.0);
        // Place max food to ensure energy would exceed max without cap.
        {
            let mut cfg = small_config();
            cfg.world.food.initial_coverage = 1.0;
            cfg.world.food.initial_density = 255;
            sim.world
                .seed_food(&mut rand::rngs::SmallRng::seed_from_u64(0), &cfg);
        }
        let max = sim.config.energy.lifecycle.max_energy;
        let creature = sim.creatures.get_mut(id).unwrap();
        apply_eat(creature, &mut sim.world, &sim.config);
        assert!(
            sim.creatures[id].energy <= max,
            "energy {} should be <= max {}",
            sim.creatures[id].energy,
            max
        );
    }

    // ── apply_move ─────────────────────────────────────────────────────────────

    #[test]
    fn apply_move_updates_position_and_occupancy() {
        let start = Position::new(5, 5);
        let (mut sim, id) = make_sim_one_creature(start, 50.0);
        let move_cost = sim.config.energy.costs.move_cost;
        let energy_before = sim.creatures[id].energy;
        {
            let creature = sim.creatures.get_mut(id).unwrap();
            apply_move(id, creature, &mut sim.world, Direction::N, &sim.config);
        }
        // In wrap mode, N of (5,5) on a 10×10 world is (5,4).
        let expected = Position::new(5, 4);
        assert_eq!(sim.creatures[id].position, expected);
        assert!(sim.world.creature_at(expected).is_some());
        assert!(sim.world.creature_at(start).is_none());
        assert!((sim.creatures[id].energy - (energy_before - move_cost)).abs() < f32::EPSILON);
    }

    #[test]
    fn apply_move_into_barrier_no_position_change_cost_deducted() {
        let start = Position::new(5, 5);
        let (mut sim, id) = make_sim_one_creature(start, 50.0);
        // Place a barrier to the north.
        sim.world.set_barrier(Position::new(5, 4), true);
        let energy_before = sim.creatures[id].energy;
        let move_cost = sim.config.energy.costs.move_cost;
        {
            let creature = sim.creatures.get_mut(id).unwrap();
            apply_move(id, creature, &mut sim.world, Direction::N, &sim.config);
        }
        assert_eq!(
            sim.creatures[id].position, start,
            "should not move into barrier"
        );
        assert!(
            (sim.creatures[id].energy - (energy_before - move_cost)).abs() < f32::EPSILON,
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
                [204, 61, 61],
                [1.0f32; 3],
                [true; 3],
            )
        });
        let id2 = creatures.insert_with_key(|id| {
            CreatureState::new(
                id,
                v3alpha1_founder_genome(),
                pos2,
                50.0,
                0,
                [204, 61, 61],
                [1.0f32; 3],
                [true; 3],
            )
        });
        world.place_creature(pos1, id1);
        world.place_creature(pos2, id2);
        let mut sim = Simulation {
            world,
            creatures,
            tick: 0,
            config: cfg,
            rng: rand::rngs::SmallRng::seed_from_u64(0),
        };
        {
            let creature = sim.creatures.get_mut(id1).unwrap();
            apply_move(id1, creature, &mut sim.world, Direction::N, &sim.config);
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
        let mut rng = rand::rngs::SmallRng::seed_from_u64(1);
        let result = apply_reproduce(parent_id, &mut sim, Direction::N, 20.0, &mut rng);
        assert_eq!(result, ReproductionActionResult::Spawned);
        assert_eq!(sim.creatures.len(), 2);
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
                [204, 61, 61],
                [1.0f32; 3],
                [true; 3],
            )
        });
        let blocker = creatures.insert_with_key(|id| {
            CreatureState::new(
                id,
                v3alpha1_founder_genome(),
                north,
                80.0,
                0,
                [204, 61, 61],
                [1.0f32; 3],
                [true; 3],
            )
        });
        world.place_creature(pos, parent);
        world.place_creature(north, blocker);
        let mut sim = Simulation {
            world,
            creatures,
            tick: 0,
            config: cfg,
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
        let mut rng = rand::rngs::SmallRng::seed_from_u64(3);
        let result = apply_reproduce(parent_id, &mut sim, Direction::N, 20.0, &mut rng);
        assert_eq!(result, ReproductionActionResult::RejectedEnergyConstraints);
    }

    #[test]
    fn apply_reproduce_deducts_cost_from_parent() {
        let pos = Position::new(5, 5);
        let (mut sim, parent_id) = make_sim_one_creature(pos, 80.0);
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
    fn apply_reproduce_child_inherits_memory() {
        let pos = Position::new(5, 5);
        let (mut sim, parent_id) = make_sim_one_creature(pos, 80.0);
        // Write a distinctive byte pattern to parent memory.
        sim.creatures[parent_id].memory[42] = 0xAB;
        sim.creatures[parent_id].memory[100] = 0xCD;
        let parent_gen = sim.creatures[parent_id].generation;
        let mut rng = rand::rngs::SmallRng::seed_from_u64(7);
        let result = apply_reproduce(parent_id, &mut sim, Direction::N, 20.0, &mut rng);
        assert_eq!(result, ReproductionActionResult::Spawned);
        let child = sim
            .creatures
            .values()
            .find(|c| c.generation == parent_gen + 1)
            .expect("child not found");
        assert_eq!(child.memory[42], 0xAB);
        assert_eq!(child.memory[100], 0xCD);
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
}
