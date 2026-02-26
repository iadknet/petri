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
/// Two-phase model:
/// 1. Phase 0: world updates (food growth, aging, decay, death removal)
/// 2. Build turn queue: sort all creature IDs then shuffle
/// 3. Phase 1 — Batch cognition: all creatures see the frozen post-Phase-0 world snapshot
/// 4. Phase 2 — Sequential action execution: apply decisions in queue order
/// 5. Increment sim.tick
pub fn run_tick(sim: &mut Simulation) {
    use rand::seq::SliceRandom;
    use rand::RngCore;
    use rand::SeedableRng;

    use std::collections::HashMap;

    use rayon::prelude::*;

    use crate::contracts::{CreatureId, WorldAction};
    use crate::runtime::mesh::execute_creature_mesh;
    use crate::runtime::types::ComputeCostReport;
    use crate::sensors::static_inputs::assemble_static_inputs;
    use crate::simulation::actions::{apply_eat, apply_move, apply_noop, apply_reproduce};

    // Reset per-tick counters at the start of each tick.
    sim.stats.last_tick_move = 0;
    sim.stats.last_tick_eat = 0;
    sim.stats.last_tick_noop = 0;
    sim.stats.last_tick_reproduce = 0;
    sim.stats.last_tick_compute_total_mean = 0.0;
    sim.stats.last_tick_compute_total_min = 0.0;
    sim.stats.last_tick_compute_total_max = 0.0;
    sim.stats.last_tick_compute_vm_mean = 0.0;
    sim.stats.last_tick_compute_graph_mean = 0.0;

    run_phase_0(sim);

    // Build turn queue: stable sort for reproducibility, then shuffle.
    let mut queue: Vec<_> = sim.creatures.keys().collect();
    queue.sort();
    queue.shuffle(&mut sim.rng);

    // Derive a separate RNG for reproduction to avoid double-borrowing sim.rng.
    let mut reproduce_rng = rand::rngs::SmallRng::seed_from_u64(sim.rng.next_u64());

    // Clone RuntimeConfig for cognition phase (small struct, ~7 scalars).
    let runtime_config = sim.config.runtime.clone();

    // ── Phase 1: Batch cognition (parallel) ──────────────────────────────────
    // All creatures see the frozen post-Phase-0 world snapshot. Cognition only
    // mutates each creature's private state (energy, memory, graph_state).

    // 1a: Assemble sensor inputs sequentially (needs &sim.world + &sim.creatures).
    let inputs: Vec<_> = queue
        .iter()
        .filter(|&&id| sim.creatures.contains_key(id))
        .map(|&id| (id, assemble_static_inputs(&sim.world, &sim.creatures[id])))
        .collect();

    // 1b: Parallel cognition — each creature's mesh executes independently.
    // Extract disjoint &mut CreatureState refs via HashMap::remove, then run
    // par_iter_mut so each thread gets its own exclusive creature reference.
    let decisions: Vec<(CreatureId, WorldAction, ComputeCostReport)> = {
        let mut creature_refs: HashMap<_, _> = sim.creatures.iter_mut().collect();

        let mut work: Vec<_> = inputs
            .into_iter()
            .filter_map(|(id, si)| creature_refs.remove(&id).map(|c| (id, si, c)))
            .collect();

        work.par_iter_mut()
            .map(|(id, si, creature)| {
                let (action, cost) = execute_creature_mesh(
                    &creature.genome,
                    si,
                    &mut creature.energy,
                    &mut creature.memory,
                    &mut creature.graph_state,
                    &runtime_config,
                );
                (*id, action, cost)
            })
            .collect()
    };

    // ── Phase 2: Sequential action execution ────────────────────────────────
    // Apply decisions in queue order. Compute cost stats are accumulated here.
    let mut compute_total_sum = 0.0f32;
    let mut compute_total_min = f32::MAX;
    let mut compute_total_max = 0.0f32;
    let mut compute_vm_sum = 0.0f32;
    let mut compute_vm_count = 0u32;
    let mut compute_graph_sum = 0.0f32;
    let mut compute_graph_count = 0u32;
    let mut compute_creature_count = 0u32;

    for (id, action, compute_cost) in decisions {
        // Accumulate compute cost for this creature.
        let total_cost = compute_cost.vm_cost + compute_cost.graph_cost;
        compute_total_sum += total_cost;
        if total_cost < compute_total_min {
            compute_total_min = total_cost;
        }
        if total_cost > compute_total_max {
            compute_total_max = total_cost;
        }
        if compute_cost.vm_cost > 0.0 {
            compute_vm_sum += compute_cost.vm_cost;
            compute_vm_count += 1;
        }
        if compute_cost.graph_cost > 0.0 {
            compute_graph_sum += compute_cost.graph_cost;
            compute_graph_count += 1;
        }
        compute_creature_count += 1;

        // Apply the chosen action using the factored apply_* functions.
        match action {
            WorldAction::NoOp => {
                if let Some(creature) = sim.creatures.get_mut(id) {
                    apply_noop(creature, &sim.config);
                    sim.stats.last_tick_noop += 1;
                }
            }
            WorldAction::Eat => {
                if let Some(creature) = sim.creatures.get_mut(id) {
                    apply_eat(creature, &mut sim.world, &sim.config);
                    sim.stats.last_tick_eat += 1;
                }
            }
            WorldAction::Move(dir) => {
                if let Some(creature) = sim.creatures.get_mut(id) {
                    apply_move(id, creature, &mut sim.world, dir, &sim.config);
                    sim.stats.last_tick_move += 1;
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

    // Write per-tick compute stats after the creature queue is fully processed.
    if compute_creature_count > 0 {
        sim.stats.last_tick_compute_total_mean = compute_total_sum / compute_creature_count as f32;
        sim.stats.last_tick_compute_total_min = compute_total_min;
        sim.stats.last_tick_compute_total_max = compute_total_max;
        sim.stats.last_tick_compute_vm_mean = if compute_vm_count > 0 {
            compute_vm_sum / compute_vm_count as f32
        } else {
            0.0
        };
        sim.stats.last_tick_compute_graph_mean = if compute_graph_count > 0 {
            compute_graph_sum / compute_graph_count as f32
        } else {
            0.0
        };
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
                0,
                [true; 3],
            )
        });
        world.place_creature(pos, id);

        let sim = Simulation {
            world,
            creatures,
            tick: 0,
            config: cfg,
            stats: crate::simulation::stats::SimStats::default(),
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
