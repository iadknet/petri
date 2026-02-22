use crate::simulation::simulation::Simulation;

/// Run Phase 0 of a tick: food growth, creature aging, energy decay, dead-creature removal.
///
/// Sub-step canonical order (v3-tick-orchestration-spec.md Section 3):
/// 1. Food growth
/// 2. Creature aging (+1 per creature)
/// 3. Energy decay (subtract `energy_decay_per_tick`)
/// 4. Death removal (remove creatures where energy <= 0 from slotmap + world occupancy)
pub fn run_phase_0(sim: &mut Simulation) {
    // Step 1: Food growth
    sim.world.grow_food(&mut sim.rng, &sim.config);

    // Steps 2 & 3: Age and energy decay
    for (_, creature) in sim.creatures.iter_mut() {
        creature.age += 1;
        creature.energy -= sim.config.energy.lifecycle.energy_decay_per_tick;
    }

    // Step 4: Collect dead IDs first to avoid borrow conflict during removal.
    let dead_ids: Vec<_> = sim
        .creatures
        .iter()
        .filter(|(_, c)| c.energy <= 0.0)
        .map(|(id, _)| id)
        .collect();

    for id in dead_ids {
        if let Some(creature) = sim.creatures.remove(id) {
            sim.world.remove_creature(creature.position);
        }
    }
}

/// Run one full simulation tick per v3-tick-orchestration-spec.md.
///
/// 1. Phase 0 world updates
/// 2. Build turn queue: sort all creature IDs then shuffle
/// 3. For each creature in queue (skip if removed mid-tick):
///    a. Assemble static sensor snapshot
///    b. Execute creature mesh → WorldAction
///    c. Apply action
/// 4. Increment sim.tick
pub fn run_tick(sim: &mut Simulation) {
    use rand::seq::SliceRandom;
    use rand::RngCore;
    use rand::SeedableRng;

    use crate::contracts::WorldAction;
    use crate::runtime::mesh::execute_creature_mesh;
    use crate::sensors::static_inputs::assemble_static_inputs;
    use crate::simulation::actions::{apply_eat, apply_move, apply_noop, apply_reproduce};

    run_phase_0(sim);

    // Build turn queue: stable sort for reproducibility, then shuffle.
    let mut queue: Vec<_> = sim.creatures.keys().collect();
    queue.sort();
    queue.shuffle(&mut sim.rng);

    // Derive a separate RNG for reproduction to avoid double-borrowing sim.rng.
    let mut reproduce_rng = rand::rngs::SmallRng::seed_from_u64(sim.rng.next_u64());

    // Clone config once per tick so action-dispatch can access it alongside
    // a mutable creature borrow (different SlotMap entry).
    let config = sim.config.clone();

    for id in queue {
        // Skip creatures removed mid-tick (killed by a previous action this tick).
        if !sim.creatures.contains_key(id) {
            continue;
        }

        let static_inputs = assemble_static_inputs(&sim.world, &sim.creatures[id]);

        // Execute the creature's mesh chain.  The borrow of `sim.creatures` ends
        // when this block closes.
        let action = {
            let creature = sim.creatures.get_mut(id).unwrap();
            execute_creature_mesh(
                &creature.genome,
                &static_inputs,
                &mut creature.energy,
                &mut creature.memory,
                &mut creature.graph_state,
                &config.runtime,
            )
        };

        // Apply the chosen action.  Each branch re-borrows only what it needs.
        match action {
            WorldAction::NoOp => {
                if let Some(creature) = sim.creatures.get_mut(id) {
                    apply_noop(creature, &config);
                }
            }
            WorldAction::Eat => {
                if let Some(creature) = sim.creatures.get_mut(id) {
                    apply_eat(creature, &mut sim.world, &config);
                }
            }
            WorldAction::Move(dir) => {
                if let Some(creature) = sim.creatures.get_mut(id) {
                    apply_move(id, creature, &mut sim.world, dir, &config);
                }
            }
            WorldAction::Reproduce {
                direction,
                energy_transfer,
            } => {
                apply_reproduce(id, sim, direction, energy_transfer, &mut reproduce_rng);
            }
        }
    }

    sim.tick += 1;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SimulationConfig;
    use crate::contracts::{CreatureId, Position};
    use crate::creature::founder::v3alpha1_founder_genome;
    use crate::creature::state::CreatureState;
    use crate::kernel::WorldState;
    use crate::simulation::seeding::seed_simulation;
    use rand::SeedableRng;
    use slotmap::SlotMap;

    fn small_config() -> SimulationConfig {
        let mut cfg = SimulationConfig::default();
        cfg.world.width = 20;
        cfg.world.height = 20;
        cfg.population.initial_creatures = 5;
        cfg
    }

    /// Build a minimal simulation with one creature at (5,5) and no food/food-growth.
    fn make_sim_with_one_creature(energy: f32) -> (Simulation, CreatureId) {
        let mut cfg = small_config();
        cfg.world.food.growth_rate = 0.0;
        cfg.world.food.initial_coverage = 0.0;

        let mut world = WorldState::new(cfg.world.width, cfg.world.height, cfg.world.edge_mode);
        let mut creatures: SlotMap<CreatureId, CreatureState> = SlotMap::with_key();
        let pos = Position::new(5, 5);
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

    #[test]
    fn phase_0_increments_creature_age() {
        let (mut sim, id) = make_sim_with_one_creature(50.0);
        assert_eq!(sim.creatures[id].age, 0);
        run_phase_0(&mut sim);
        assert_eq!(sim.creatures[id].age, 1);
    }

    #[test]
    fn phase_0_decays_energy() {
        let initial = 50.0f32;
        let (mut sim, id) = make_sim_with_one_creature(initial);
        let decay = sim.config.energy.lifecycle.energy_decay_per_tick;
        run_phase_0(&mut sim);
        let expected = initial - decay;
        assert!(
            (sim.creatures[id].energy - expected).abs() < f32::EPSILON,
            "energy {} vs expected {}",
            sim.creatures[id].energy,
            expected
        );
    }

    #[test]
    fn phase_0_removes_dead_creatures() {
        let decay = SimulationConfig::default()
            .energy
            .lifecycle
            .energy_decay_per_tick;
        let (mut sim, id) = make_sim_with_one_creature(decay * 0.5);
        assert!(sim.creatures.contains_key(id));
        run_phase_0(&mut sim);
        assert!(!sim.creatures.contains_key(id));
    }

    #[test]
    fn phase_0_grows_food() {
        let mut cfg = small_config();
        cfg.world.food.growth_rate = 1.0;
        cfg.world.food.initial_coverage = 0.0;
        let mut sim = seed_simulation(cfg, 42);
        let before = sim.world.total_food();
        run_phase_0(&mut sim);
        assert!(sim.world.total_food() > before);
    }

    #[test]
    fn dead_creature_removed_from_occupancy() {
        let decay = SimulationConfig::default()
            .energy
            .lifecycle
            .energy_decay_per_tick;
        let (mut sim, _) = make_sim_with_one_creature(decay * 0.5);
        let pos = Position::new(5, 5);
        assert!(sim.world.creature_at(pos).is_some());
        run_phase_0(&mut sim);
        assert!(sim.world.creature_at(pos).is_none());
    }

    #[test]
    fn run_tick_increments_tick_counter() {
        let mut sim = seed_simulation(small_config(), 42);
        assert_eq!(sim.tick_number(), 0);
        run_tick(&mut sim);
        assert_eq!(sim.tick_number(), 1);
        run_tick(&mut sim);
        assert_eq!(sim.tick_number(), 2);
    }
}
