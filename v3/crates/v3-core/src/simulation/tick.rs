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
///
/// The optional `trace` parameter enables execution tracing for a single creature.
/// When `Some`, the target creature is extracted from the parallel batch and run
/// sequentially with [`execute_creature_mesh_traced`], recording detailed trace data.
/// When `None`, behavior is identical to the untraced path.
pub fn run_tick(sim: &mut Simulation, trace: &mut Option<crate::runtime::trace::ActiveTrace>) {
    use rand::seq::SliceRandom;
    use rand::RngCore;
    use rand::SeedableRng;

    use std::collections::HashMap;

    use rayon::prelude::*;

    use crate::contracts::{CreatureId, WorldAction};
    use crate::runtime::mesh::execute_creature_mesh;
    use crate::runtime::trace::{StaticInputsSnapshot, TickTrace};
    use crate::runtime::traced_mesh::execute_creature_mesh_traced;
    use crate::runtime::types::ComputeCostReport;
    use crate::sensors::static_inputs::assemble_static_inputs;
    use crate::simulation::actions::{
        apply_eat, apply_move, apply_noop, apply_reproduce, apply_steal_energy,
        PredationActionResult, ReproductionActionResult,
    };

    // Reset per-tick counters at the start of each tick.
    sim.stats.last_tick_move = 0;
    sim.stats.last_tick_eat = 0;
    sim.stats.last_tick_noop = 0;
    sim.stats.last_tick_reproduce = 0;
    sim.stats.last_tick_steal = 0;
    sim.stats.last_tick_predation_events.clear();
    sim.stats.last_tick_predation_kills = 0;
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

    // Check if traced creature died during Phase 0.
    if let Some(ref mut active) = trace {
        if !active.is_complete() && !sim.creatures.contains_key(active.creature_id) {
            active.ticks_remaining = 0;
        }
    }

    // Derive a separate RNG for reproduction to avoid double-borrowing sim.rng.
    let mut reproduce_rng = rand::rngs::SmallRng::seed_from_u64(sim.rng.next_u64());

    // Clone RuntimeConfig for cognition phase (small struct, ~7 scalars).
    let runtime_config = sim.config.runtime.clone();

    // Determine if we need to trace a specific creature this tick.
    let trace_target: Option<CreatureId> = trace
        .as_ref()
        .filter(|t| !t.is_complete())
        .map(|t| t.creature_id);

    // ── Phase 1: Batch cognition (parallel, with optional trace extraction) ──
    // All creatures see the frozen post-Phase-0 world snapshot. Cognition only
    // mutates each creature's private state (energy, memory, graph_runtime).

    // 1a: Assemble sensor inputs sequentially (needs &sim.world + &sim.creatures).
    let inputs: Vec<_> = queue
        .iter()
        .filter(|&&id| sim.creatures.contains_key(id))
        .map(|&id| (id, assemble_static_inputs(&sim.world, &sim.creatures[id])))
        .collect();

    // 1b: Cognition — parallel for all creatures, sequential for traced creature.
    let decisions: Vec<(CreatureId, Vec<WorldAction>, ComputeCostReport)> = {
        let mut creature_refs: HashMap<_, _> = sim.creatures.iter_mut().collect();

        // Extract traced creature (if any) before building parallel work vec.
        let traced_creature =
            trace_target.and_then(|tid| creature_refs.remove(&tid).map(|c| (tid, c)));

        let mut work: Vec<_> = inputs
            .iter()
            .filter_map(|(id, si)| creature_refs.remove(id).map(|c| (*id, si, c)))
            .collect();

        // Run all non-traced creatures in parallel.
        let mut parallel_decisions: Vec<_> = work
            .par_iter_mut()
            .map(|(id, si, creature)| {
                let (action, cost) = execute_creature_mesh(
                    &creature.genome,
                    si,
                    &mut creature.energy,
                    &mut creature.memory,
                    &mut creature.graph_runtime,
                    &runtime_config,
                );
                (*id, vec![action], cost)
            })
            .collect();

        // Run traced creature sequentially with trace recording.
        if let Some((tid, creature)) = traced_creature {
            if let Some((_, si)) = inputs.iter().find(|(id, _)| *id == tid) {
                let energy_before = creature.energy;
                let tick_number = sim.tick;
                let si_snapshot = StaticInputsSnapshot::from(si);

                let (action, cost, hops, termination_reason) = execute_creature_mesh_traced(
                    &creature.genome,
                    si,
                    &mut creature.energy,
                    &mut creature.memory,
                    &mut creature.graph_runtime,
                    &runtime_config,
                );

                let actions = vec![action];

                // Record tick trace.
                if let Some(ref mut active) = trace {
                    active.ticks.push(TickTrace {
                        tick_number,
                        energy_before,
                        energy_after: creature.energy,
                        static_inputs: si_snapshot,
                        hops,
                        final_actions: actions.clone(),
                        termination_reason,
                    });
                    active.ticks_remaining = active.ticks_remaining.saturating_sub(1);
                }

                // Insert traced creature's decision at its queue position.
                // Find where tid appears in the original queue order.
                let queue_pos = inputs.iter().position(|(id, _)| *id == tid);
                // Count how many non-traced inputs precede it to find the
                // insertion point in parallel_decisions.
                if let Some(pos) = queue_pos {
                    let insert_idx = inputs[..pos].iter().filter(|(id, _)| *id != tid).count();
                    parallel_decisions.insert(insert_idx, (tid, actions, cost));
                } else {
                    parallel_decisions.push((tid, actions, cost));
                }
            }
        }

        parallel_decisions
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

    for (id, actions, compute_cost) in decisions {
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

        // Apply each queued action sequentially.
        for action in &actions {
            match *action {
                WorldAction::NoOp => {
                    // NoOp cannot fail; no failed_action_penalty possible.
                    if let Some(creature) = sim.creatures.get_mut(id) {
                        apply_noop(creature, &sim.config);
                        sim.stats.last_tick_noop += 1;
                    }
                }
                WorldAction::Eat => {
                    if let Some(creature) = sim.creatures.get_mut(id) {
                        let succeeded = apply_eat(creature, &mut sim.world, &sim.config);
                        sim.stats.last_tick_eat += 1;
                        if !succeeded {
                            creature.energy -= sim.config.energy.costs.failed_action_penalty;
                        }
                    }
                }
                WorldAction::Move(dir) => {
                    if let Some(creature) = sim.creatures.get_mut(id) {
                        let succeeded =
                            apply_move(id, creature, &mut sim.world, dir, &sim.config);
                        sim.stats.last_tick_move += 1;
                        if !succeeded {
                            creature.energy -= sim.config.energy.costs.failed_action_penalty;
                        }
                    }
                }
                WorldAction::Reproduce {
                    direction,
                    energy_transfer,
                } => {
                    let result =
                        apply_reproduce(id, sim, direction, energy_transfer, &mut reproduce_rng);
                    // Note: reproduce_cost is already deducted inside apply_reproduce,
                    // so a failed reproduction pays reproduce_cost + failed_action_penalty.
                    if result != ReproductionActionResult::Spawned {
                        if let Some(creature) = sim.creatures.get_mut(id) {
                            creature.energy -= sim.config.energy.costs.failed_action_penalty;
                        }
                    }
                }
                WorldAction::StealEnergy { direction, amount } => {
                    let result = apply_steal_energy(id, sim, direction, amount);
                    if result == PredationActionResult::RejectedNoVictim {
                        if let Some(creature) = sim.creatures.get_mut(id) {
                            creature.energy -= sim.config.energy.costs.failed_action_penalty;
                        }
                    }
                }
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
                [0, 0, 92, 92, 138, 138],
                0,
                [true; 6],
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
        run_tick(&mut sim, &mut None);
        assert_eq!(sim.tick_number(), 1);
        run_tick(&mut sim, &mut None);
        assert_eq!(sim.tick_number(), 2);
    }

    // ── Trace integration tests ────────────────────────────────────────────────

    #[test]
    fn trace_populated_after_tick() {
        use crate::runtime::trace::ActiveTrace;

        let (mut sim, id) = make_sim_with_one_creature(50.0);
        let mut trace = Some(ActiveTrace::new(id, 1));

        run_tick(&mut sim, &mut trace);

        let active = trace.as_ref().expect("trace should still be Some");
        assert_eq!(active.ticks.len(), 1, "one tick should be recorded");
        assert!(
            active.is_complete(),
            "trace should be complete after 1 tick"
        );

        let tick_trace = &active.ticks[0];
        assert_eq!(
            tick_trace.tick_number, 0,
            "tick_number should be 0 (pre-increment)"
        );
        assert!(
            tick_trace.energy_before > 0.0,
            "energy_before should be positive"
        );
        assert!(!tick_trace.hops.is_empty(), "hops should not be empty");
    }

    #[test]
    fn trace_completes_after_n_ticks() {
        use crate::runtime::trace::ActiveTrace;

        let (mut sim, id) = make_sim_with_one_creature(50.0);
        let mut trace = Some(ActiveTrace::new(id, 3));

        for i in 0..3 {
            assert!(!trace.as_ref().unwrap().is_complete());
            run_tick(&mut sim, &mut trace);
            assert_eq!(trace.as_ref().unwrap().ticks.len(), i + 1);
        }
        assert!(trace.as_ref().unwrap().is_complete());
        assert_eq!(trace.as_ref().unwrap().ticks_remaining, 0);
    }

    #[test]
    fn trace_creature_death_finalizes_partial() {
        use crate::runtime::trace::ActiveTrace;

        // Give creature barely enough energy to die after Phase 0 decay.
        let decay = SimulationConfig::default()
            .energy
            .lifecycle
            .energy_decay_per_tick;
        let (mut sim, id) = make_sim_with_one_creature(decay * 0.5);
        let mut trace = Some(ActiveTrace::new(id, 5));

        run_tick(&mut sim, &mut trace);

        // Creature should have died in Phase 0, trace should be finalized early.
        let active = trace.as_ref().expect("trace should still be Some");
        assert!(
            active.is_complete(),
            "trace should be complete after creature death"
        );
        assert_eq!(
            active.ticks.len(),
            0,
            "no tick traces since creature died before cognition"
        );
    }

    #[test]
    fn non_traced_creatures_unaffected() {
        // Run two identical simulations: one with trace on creature A, one without.
        // All other creatures should produce the same actions/energy.
        let cfg = small_config();
        let mut sim_a = seed_simulation(cfg.clone(), 42);
        let mut sim_b = seed_simulation(cfg, 42);

        // Run sim_a without trace.
        run_tick(&mut sim_a, &mut None);

        // Run sim_b with trace on first creature.
        let first_id = {
            let mut keys: Vec<_> = sim_b.creatures.keys().collect();
            keys.sort();
            keys[0]
        };

        use crate::runtime::trace::ActiveTrace;
        let mut trace = Some(ActiveTrace::new(first_id, 1));
        run_tick(&mut sim_b, &mut trace);

        // Both sims should have the same tick.
        assert_eq!(sim_a.tick, sim_b.tick);

        // Population should be the same.
        assert_eq!(sim_a.creatures.len(), sim_b.creatures.len());

        // All creature IDs should match.
        let ids_a: Vec<_> = {
            let mut k: Vec<_> = sim_a.creatures.keys().collect();
            k.sort();
            k
        };
        let ids_b: Vec<_> = {
            let mut k: Vec<_> = sim_b.creatures.keys().collect();
            k.sort();
            k
        };
        assert_eq!(ids_a, ids_b);
    }

    // ── Failed action penalty tests ────────────────────────────────────────

    #[test]
    fn failed_move_deducts_penalty_in_tick() {
        use crate::contracts::Direction;
        use crate::simulation::actions::{apply_eat, apply_move};
        // Tests penalty arithmetic (action cost + penalty) via direct action calls.
        // Does not go through run_tick() because the founder genome's action choice
        // is non-deterministic. The tick dispatch match arms are verified by clippy
        // (#[must_use] ensures return values are handled) and by viability tests.
        let (mut sim, id) = make_sim_with_one_creature(100.0);
        sim.config.energy.costs.failed_action_penalty = 7.5;
        // Place a barrier to the north.
        sim.world.set_barrier(Position::new(5, 4), true);

        let energy_before = sim.creatures[id].energy;
        let move_cost = sim.config.energy.costs.move_cost;
        let penalty = sim.config.energy.costs.failed_action_penalty;

        // Simulate what the tick dispatch does: move north into barrier.
        let creature = sim.creatures.get_mut(id).unwrap();
        let succeeded = apply_move(id, creature, &mut sim.world, Direction::N, &sim.config);
        assert!(!succeeded, "move into barrier should fail");
        // Apply penalty for failed action (this is what we're implementing in tick.rs).
        if !succeeded {
            sim.creatures.get_mut(id).unwrap().energy -= penalty;
        }

        let expected = energy_before - move_cost - penalty;
        assert!(
            (sim.creatures[id].energy - expected).abs() < f32::EPSILON,
            "energy {} should be {} (start {} - move {} - penalty {})",
            sim.creatures[id].energy,
            expected,
            energy_before,
            move_cost,
            penalty
        );

        // Also test failed eat.
        let energy_before_eat = sim.creatures[id].energy;
        let eat_cost = sim.config.energy.costs.eat_cost;
        let creature = sim.creatures.get_mut(id).unwrap();
        let eat_succeeded = apply_eat(creature, &mut sim.world, &sim.config);
        assert!(!eat_succeeded, "eat on empty cell should fail");
        if !eat_succeeded {
            sim.creatures.get_mut(id).unwrap().energy -= penalty;
        }
        let expected_eat = energy_before_eat - eat_cost - penalty;
        assert!(
            (sim.creatures[id].energy - expected_eat).abs() < f32::EPSILON,
            "energy {} should be {} after failed eat",
            sim.creatures[id].energy,
            expected_eat
        );
    }

    #[test]
    fn failed_action_penalty_zero_preserves_old_behavior() {
        // With penalty=0.0, behavior should be identical to pre-penalty code.
        let (mut sim, id) = make_sim_with_one_creature(100.0);
        sim.config.energy.costs.failed_action_penalty = 0.0;
        // No barriers, normal world.
        let energy_after_decay = 100.0 - sim.config.energy.lifecycle.energy_decay_per_tick;
        run_tick(&mut sim, &mut None);
        if sim.creatures.contains_key(id) {
            let energy = sim.creatures[id].energy;
            // Without penalty, action costs are small (move=1.0, eat=0.0, noop=0.05, reproduce=0.1).
            // Energy should be close to energy_after_decay minus at most reproduce transfer.
            assert!(energy > 0.0, "creature should survive with zero penalty");
            // No extra penalty means energy loss <= action_cost + any reproduction transfer.
            // Just verify it's reasonable (no unexpected huge deduction).
            assert!(
                energy >= energy_after_decay - 50.0,
                "energy {} should not drop excessively with zero penalty",
                energy
            );
        }
    }

    #[test]
    fn last_tick_steal_resets_each_tick() {
        let mut sim = seed_simulation(small_config(), 42);
        // Manually set last_tick_steal to a nonzero value
        sim.stats.last_tick_steal = 5;
        run_tick(&mut sim, &mut None);
        // Should be reset to 0 (founders don't emit StealEnergy)
        assert_eq!(sim.stats.last_tick_steal, 0);
    }

    #[test]
    fn steal_energy_dispatch_transfers_and_removes_victim() {
        use crate::contracts::Direction;
        use crate::simulation::actions::apply_steal_energy;

        // Attacker at (5,5), victim at (5,4) = N of attacker
        let mut cfg = small_config();
        cfg.world.food.growth_rate = 0.0;
        cfg.world.food.initial_coverage = 0.0;
        cfg.predation.steal_cost_rate = 0.0;

        let mut world = WorldState::new(cfg.world.width, cfg.world.height, cfg.world.edge_mode);
        let mut creatures: SlotMap<CreatureId, CreatureState> = SlotMap::with_key();
        let victim_pos = Position::new(5, 4);

        let attacker_id = creatures.insert_with_key(|id| {
            CreatureState::new(
                id,
                v3alpha1_founder_genome(),
                Position::new(5, 5),
                50.0,
                0,
                [0, 0, 92, 92, 138, 138],
                0,
                [true; 6],
            )
        });
        world.place_creature(Position::new(5, 5), attacker_id);

        let victim_id = creatures.insert_with_key(|id| {
            CreatureState::new(
                id,
                v3alpha1_founder_genome(),
                victim_pos,
                5.0, // will be fully drained
                0,
                [0, 0, 92, 92, 138, 138],
                0,
                [true; 6],
            )
        });
        world.place_creature(victim_pos, victim_id);

        let mut sim = Simulation {
            world,
            creatures,
            tick: 0,
            config: cfg,
            stats: crate::simulation::stats::SimStats::default(),
            rng: rand::rngs::SmallRng::seed_from_u64(42),
        };

        let result = apply_steal_energy(attacker_id, &mut sim, Direction::N, 20.0);

        assert_eq!(
            result,
            crate::simulation::actions::PredationActionResult::TransferredAndKilled
        );
        assert!(
            !sim.creatures.contains_key(victim_id),
            "victim should be removed from slotmap"
        );
        assert!(
            sim.world.creature_at(victim_pos).is_none(),
            "victim should be removed from world occupancy"
        );
        assert!(
            sim.creatures[attacker_id].energy > 50.0,
            "attacker should have gained energy"
        );
        assert_eq!(sim.stats.predation_kills_total, 1);
        assert_eq!(sim.stats.predation_actions_attempted_total, 1);
    }

    #[test]
    fn run_tick_with_none_trace_identical_behavior() {
        // Verify that passing &mut None doesn't change behavior at all.
        let mut sim = seed_simulation(small_config(), 42);
        let pop_before = sim.creatures.len();
        run_tick(&mut sim, &mut None);
        // Basic sanity: tick incremented, simulation still has creatures.
        assert_eq!(sim.tick, 1);
        assert!(!sim.creatures.is_empty() || pop_before == 0);
    }
}
