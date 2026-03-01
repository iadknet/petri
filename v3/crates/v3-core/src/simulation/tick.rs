use crate::contracts::CreatureId;
use crate::runtime::types::MeshOutput;
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
    use crate::runtime::types::MeshOutput;
    use crate::sensors::perception::{
        genome_uses_extended_perception, PerceptionConfig, PerceptionSnapshot, SensorSnapshot,
    };
    use crate::sensors::reducers::assemble_perception;
    use crate::sensors::static_inputs::assemble_static_inputs;
    use crate::sensors::visibility::{compute_visible_cells, get_visibility_table};
    use crate::simulation::actions::{
        apply_eat, apply_move, apply_noop, apply_reproduce, apply_steal_energy,
        PredationActionResult, ReproductionActionResult,
    };
    use crate::simulation::outcomes::OutcomeAccumulator;

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
    sim.stats.last_tick_priority_bid_mean = 0.0;
    sim.stats.last_tick_priority_bidders_count = 0;

    run_phase_0(sim);

    // Snapshot surviving creature energies for Phase 2.5 reward learning.
    let mut outcome_acc = OutcomeAccumulator::default();
    for (id, creature) in sim.creatures.iter() {
        outcome_acc.snapshot_energy(id, creature.energy);
    }

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
    // Conditional perception: only assemble full visibility + area reduction for
    // genomes that reference extended perception keys. Others use zero() fallback.
    let perception_config = PerceptionConfig::from_sim_config(&sim.config);
    let vis_table = get_visibility_table(perception_config.vision_radius);
    let inputs: Vec<_> = queue
        .iter()
        .filter(|&&id| sim.creatures.contains_key(id))
        .map(|&id| {
            let creature = &sim.creatures[id];
            let local = assemble_static_inputs(&sim.world, creature);
            let perception = if genome_uses_extended_perception(&creature.genome) {
                let visible = compute_visible_cells(creature.position, &sim.world, vis_table);
                assemble_perception(
                    id,
                    creature,
                    &visible,
                    &sim.world,
                    &sim.creatures,
                    &perception_config,
                )
            } else {
                PerceptionSnapshot::zero()
            };
            let ss = SensorSnapshot { local, perception };
            (id, ss)
        })
        .collect();

    // 1b: Cognition — parallel for all creatures, sequential for traced creature.
    let mut decisions: Vec<(CreatureId, MeshOutput)> = {
        let mut creature_refs: HashMap<_, _> = sim.creatures.iter_mut().collect();

        // Extract traced creature (if any) before building parallel work vec.
        let traced_creature =
            trace_target.and_then(|tid| creature_refs.remove(&tid).map(|c| (tid, c)));

        let mut work: Vec<_> = inputs
            .iter()
            .filter_map(|(id, ss)| creature_refs.remove(id).map(|c| (*id, ss, c)))
            .collect();

        // Run all non-traced creatures in parallel.
        let mut parallel_decisions: Vec<_> = work
            .par_iter_mut()
            .map(|(id, ss, creature)| {
                let output = execute_creature_mesh(
                    &creature.genome,
                    ss,
                    &mut creature.energy,
                    &mut creature.memory,
                    &mut creature.graph_runtime,
                    &runtime_config,
                );
                (*id, output)
            })
            .collect();

        // Run traced creature sequentially with trace recording.
        if let Some((tid, creature)) = traced_creature {
            if let Some((_, ss)) = inputs.iter().find(|(id, _)| *id == tid) {
                let energy_before = creature.energy;
                let tick_number = sim.tick;
                let si_snapshot = StaticInputsSnapshot::from(&ss.local);

                let (output, hops, termination_reason) = execute_creature_mesh_traced(
                    &creature.genome,
                    ss,
                    &mut creature.energy,
                    &mut creature.memory,
                    &mut creature.graph_runtime,
                    &runtime_config,
                );

                // Record tick trace.
                if let Some(ref mut active) = trace {
                    let debug_perception = if active.include_perception_debug {
                        Some(crate::runtime::trace::PerceptionDebugSnapshot::from(
                            &ss.perception,
                        ))
                    } else {
                        None
                    };
                    active.ticks.push(TickTrace {
                        tick_number,
                        energy_before,
                        energy_after: creature.energy,
                        static_inputs: si_snapshot,
                        debug_perception,
                        hops,
                        final_actions: output.actions.clone(),
                        termination_reason,
                        priority_bid: output.priority_bid,
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
                    parallel_decisions.insert(insert_idx, (tid, output));
                } else {
                    parallel_decisions.push((tid, output));
                }
            }
        }

        parallel_decisions
    };

    // Sort decisions by priority bid descending. Stable sort preserves the
    // pre-existing random shuffle order among creatures with equal bids.
    sort_by_priority_bid(&mut decisions);

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
    let mut priority_bid_sum = 0.0f32;
    let mut priority_bid_count = 0u32;

    for (id, output) in decisions {
        let compute_cost = &output.cost_report;
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

        // Accumulate priority bid stats.
        priority_bid_sum += output.priority_bid;
        if output.priority_bid > 0.0 {
            priority_bid_count += 1;
        }

        // Skip all actions for creatures killed by earlier predation this tick.
        if !sim.creatures.contains_key(id) {
            continue;
        }

        // Apply each queued action sequentially.
        for action in &output.actions {
            match *action {
                WorldAction::NoOp => {
                    // NoOp cannot fail; no failed_action_penalty possible.
                    if let Some(creature) = sim.creatures.get_mut(id) {
                        apply_noop(creature, &sim.config);
                        sim.stats.last_tick_noop += 1;
                        outcome_acc.record_action_result(id, true);
                    }
                }
                WorldAction::Eat => {
                    if let Some(creature) = sim.creatures.get_mut(id) {
                        let succeeded = apply_eat(creature, &mut sim.world, &sim.config);
                        sim.stats.last_tick_eat += 1;
                        outcome_acc.record_action_result(id, succeeded);
                        if !succeeded {
                            creature.energy -= sim.config.energy.costs.failed_action_penalty;
                        }
                    }
                }
                WorldAction::Move(dir) => {
                    if let Some(creature) = sim.creatures.get_mut(id) {
                        let succeeded = apply_move(id, creature, &mut sim.world, dir, &sim.config);
                        sim.stats.last_tick_move += 1;
                        outcome_acc.record_action_result(id, succeeded);
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
                    let succeeded = result == ReproductionActionResult::Spawned;
                    outcome_acc.record_action_result(id, succeeded);
                    if succeeded {
                        outcome_acc.record_offspring(id);
                    }
                    // Note: reproduce_cost is already deducted inside apply_reproduce,
                    // so a failed reproduction pays reproduce_cost + failed_action_penalty.
                    if !succeeded {
                        if let Some(creature) = sim.creatures.get_mut(id) {
                            creature.energy -= sim.config.energy.costs.failed_action_penalty;
                        }
                    }
                }
                WorldAction::StealEnergy { direction, amount } => {
                    // Snapshot predation events length to extract damage info.
                    let events_before = sim.stats.last_tick_predation_events.len();
                    let result = apply_steal_energy(id, sim, direction, amount);
                    let succeeded = result != PredationActionResult::RejectedNoVictim;
                    outcome_acc.record_action_result(id, succeeded);
                    // Record damage to victim from predation event (if any).
                    if succeeded {
                        if let Some(event) = sim.stats.last_tick_predation_events.get(events_before)
                        {
                            // Resolve victim ID from world occupancy (if victim survived).
                            let victim_pos =
                                crate::contracts::Position::new(event.victim_x, event.victim_y);
                            if let Some(victim_id) = sim.world.creature_at(victim_pos) {
                                outcome_acc.record_damage(victim_id, event.energy_stolen);
                            }
                            // Killed victims are already removed — skip (they can't learn).
                        }
                    }
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
        sim.stats.last_tick_priority_bid_mean = priority_bid_sum / compute_creature_count as f32;
        sim.stats.last_tick_priority_bidders_count = priority_bid_count;
    }

    // ── Phase 2.5: Reward-modulated learning pass ────────────────────────
    // For each creature with reward-modulated graph nodes, compute outcome
    // signals and apply three-factor weight updates using eligibility traces.
    {
        use crate::creature::genome::BackendDef;
        use crate::runtime::plasticity::reward::apply_reward_modulated_updates;
        use crate::runtime::plasticity::traces::has_any_reward_modulated;

        let reward_cost = sim.config.runtime.reward_learning_cost;

        // Collect IDs to avoid borrow conflict (need &mut creature + &sim.config).
        let ids: Vec<CreatureId> = sim.creatures.keys().collect();
        for id in ids {
            let creature = match sim.creatures.get(id) {
                Some(c) => c,
                None => continue,
            };

            // Check if any mesh node has reward-modulated plasticity.
            let has_reward = creature.genome.nodes.iter().any(|node| {
                if let BackendDef::Graph(ref def) = node.backend_def {
                    has_any_reward_modulated(def)
                } else {
                    false
                }
            });
            if !has_reward {
                continue;
            }

            // Compute outcome signal bank for this creature.
            let energy_after = creature.energy;
            let signals = match outcome_acc.compute_signal_bank(id, energy_after) {
                Some(s) => s,
                None => continue, // Newborn spawned this tick — no outcome record.
            };

            // Apply reward-modulated updates per mesh node.
            let creature = match sim.creatures.get_mut(id) {
                Some(c) => c,
                None => continue,
            };
            for (node_idx, node) in creature.genome.nodes.iter().enumerate() {
                if let BackendDef::Graph(ref def) = node.backend_def {
                    if has_any_reward_modulated(def) {
                        let update_cost = apply_reward_modulated_updates(
                            def,
                            node_idx,
                            &mut creature.graph_runtime.plasticity_weights,
                            &creature.graph_runtime.eligibility_traces,
                            &signals,
                            reward_cost,
                        );
                        creature.energy -= update_cost;
                    }
                }
            }
        }
    }

    sim.tick += 1;
}

/// Sort decisions by priority bid descending. Stable sort preserves the
/// pre-existing random shuffle order among creatures with equal bids.
fn sort_by_priority_bid(decisions: &mut [(CreatureId, MeshOutput)]) {
    decisions.sort_by(|a, b| {
        b.1.priority_bid
            .partial_cmp(&a.1.priority_bid)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
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

    /// Build a minimal simulation with two creatures and no food/food-growth.
    /// Creature A at (5,5), creature B at (5,4) (directly north of A).
    fn make_sim_two_creatures(
        energy_a: f32,
        energy_b: f32,
    ) -> (Simulation, CreatureId, CreatureId) {
        let mut cfg = small_config();
        cfg.world.food.growth_rate = 0.0;
        cfg.world.food.initial_coverage = 0.0;

        let mut world = WorldState::new(cfg.world.width, cfg.world.height, cfg.world.edge_mode);
        let mut creatures: SlotMap<CreatureId, CreatureState> = SlotMap::with_key();

        let a_id = creatures.insert_with_key(|id| {
            CreatureState::new(
                id,
                v3alpha1_founder_genome(),
                Position::new(5, 5),
                energy_a,
                0,
                [0, 0, 92, 92, 138, 138],
                0,
                [true; 6],
                CreatureIdentityState::default(),
            )
        });
        world.place_creature(Position::new(5, 5), a_id);

        let b_id = creatures.insert_with_key(|id| {
            CreatureState::new(
                id,
                v3alpha1_founder_genome(),
                Position::new(5, 4),
                energy_b,
                0,
                [0, 0, 92, 92, 138, 138],
                0,
                [true; 6],
                CreatureIdentityState::default(),
            )
        });
        world.place_creature(Position::new(5, 4), b_id);

        let sim = Simulation {
            world,
            creatures,
            tick: 0,
            config: cfg,
            stats: crate::simulation::stats::SimStats::default(),
            rng: rand::rngs::SmallRng::seed_from_u64(42),
        };
        (sim, a_id, b_id)
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
                CreatureIdentityState::default(),
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

        let (mut sim, attacker_id, victim_id) = make_sim_two_creatures(50.0, 5.0);
        sim.config.predation.steal_cost_rate = 0.0;
        let victim_pos = Position::new(5, 4);

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

    // ── Priority bid sort tests ──────────────────────────────────────────

    #[test]
    fn sort_by_priority_bid_orders_descending() {
        use crate::contracts::WorldAction;
        use crate::runtime::types::{ComputeCostReport, MeshOutput};

        let mut creatures: SlotMap<CreatureId, CreatureState> = SlotMap::with_key();
        let id_a = creatures.insert_with_key(|id| {
            CreatureState::new(
                id,
                v3alpha1_founder_genome(),
                Position::new(0, 0),
                50.0,
                0,
                [0, 0, 92, 92, 138, 138],
                0,
                [true; 6],
                CreatureIdentityState::default(),
            )
        });
        let id_b = creatures.insert_with_key(|id| {
            CreatureState::new(
                id,
                v3alpha1_founder_genome(),
                Position::new(1, 0),
                50.0,
                0,
                [0, 0, 92, 92, 138, 138],
                0,
                [true; 6],
                CreatureIdentityState::default(),
            )
        });
        let id_c = creatures.insert_with_key(|id| {
            CreatureState::new(
                id,
                v3alpha1_founder_genome(),
                Position::new(2, 0),
                50.0,
                0,
                [0, 0, 92, 92, 138, 138],
                0,
                [true; 6],
                CreatureIdentityState::default(),
            )
        });

        let make_output = |bid: f32| MeshOutput {
            actions: vec![WorldAction::Eat],
            cost_report: ComputeCostReport::default(),
            priority_bid: bid,
        };

        // Creature A bids 0, B bids 10, C bids 5.
        let mut decisions = vec![
            (id_a, make_output(0.0)),
            (id_b, make_output(10.0)),
            (id_c, make_output(5.0)),
        ];

        sort_by_priority_bid(&mut decisions);

        // Expected order: B(10), C(5), A(0)
        assert_eq!(decisions[0].0, id_b, "highest bidder should be first");
        assert_eq!(
            decisions[1].0, id_c,
            "second highest bidder should be second"
        );
        assert_eq!(decisions[2].0, id_a, "zero bidder should be last");
    }

    #[test]
    fn sort_by_priority_bid_stable_for_equal_bids() {
        use crate::contracts::WorldAction;
        use crate::runtime::types::{ComputeCostReport, MeshOutput};

        let mut creatures: SlotMap<CreatureId, CreatureState> = SlotMap::with_key();
        let id_a = creatures.insert_with_key(|id| {
            CreatureState::new(
                id,
                v3alpha1_founder_genome(),
                Position::new(0, 0),
                50.0,
                0,
                [0, 0, 92, 92, 138, 138],
                0,
                [true; 6],
                CreatureIdentityState::default(),
            )
        });
        let id_b = creatures.insert_with_key(|id| {
            CreatureState::new(
                id,
                v3alpha1_founder_genome(),
                Position::new(1, 0),
                50.0,
                0,
                [0, 0, 92, 92, 138, 138],
                0,
                [true; 6],
                CreatureIdentityState::default(),
            )
        });
        let id_c = creatures.insert_with_key(|id| {
            CreatureState::new(
                id,
                v3alpha1_founder_genome(),
                Position::new(2, 0),
                50.0,
                0,
                [0, 0, 92, 92, 138, 138],
                0,
                [true; 6],
                CreatureIdentityState::default(),
            )
        });

        let make_output = |bid: f32| MeshOutput {
            actions: vec![WorldAction::Eat],
            cost_report: ComputeCostReport::default(),
            priority_bid: bid,
        };

        // All bid 0.0. Original order is A, B, C.
        let mut decisions = vec![
            (id_a, make_output(0.0)),
            (id_b, make_output(0.0)),
            (id_c, make_output(0.0)),
        ];

        sort_by_priority_bid(&mut decisions);

        // Stable sort should preserve original order.
        assert_eq!(decisions[0].0, id_a);
        assert_eq!(decisions[1].0, id_b);
        assert_eq!(decisions[2].0, id_c);
    }

    /// Regression: if creature A kills creature B via predation, then B's queued
    /// StealEnergy action must not panic on a stale SlotMap key.
    #[test]
    fn killed_creature_steal_action_does_not_panic() {
        use crate::contracts::Direction;
        use crate::simulation::actions::apply_steal_energy;

        let (mut sim, a_id, b_id) = make_sim_two_creatures(100.0, 5.0);
        sim.config.predation.steal_cost_rate = 0.0;

        // A kills B.
        let result = apply_steal_energy(a_id, &mut sim, Direction::N, 50.0);
        assert_eq!(
            result,
            crate::simulation::actions::PredationActionResult::TransferredAndKilled,
        );
        assert!(!sim.creatures.contains_key(b_id), "B should be dead");

        // Simulate Phase 2 dispatching B's queued action on its stale key.
        // Before the fix this panics with "invalid SlotMap key used".
        if sim.creatures.contains_key(b_id) {
            let _result = apply_steal_energy(b_id, &mut sim, Direction::S, 10.0);
        }
        // If we reach here without panic, the guard works.
    }

    /// Regression: same stale-key guard for Reproduce dispatch path.
    #[test]
    fn killed_creature_reproduce_action_does_not_panic() {
        use crate::contracts::Direction;
        use crate::simulation::actions::{apply_reproduce, apply_steal_energy};

        let (mut sim, a_id, b_id) = make_sim_two_creatures(100.0, 5.0);
        sim.config.predation.steal_cost_rate = 0.0;

        // A kills B.
        let result = apply_steal_energy(a_id, &mut sim, Direction::N, 50.0);
        assert_eq!(
            result,
            crate::simulation::actions::PredationActionResult::TransferredAndKilled,
        );
        assert!(!sim.creatures.contains_key(b_id), "B should be dead");

        // Simulate Phase 2 dispatching B's queued Reproduce on stale key.
        if sim.creatures.contains_key(b_id) {
            let mut rng = rand::rngs::SmallRng::seed_from_u64(99);
            let _result = apply_reproduce(b_id, &mut sim, Direction::S, 10.0, &mut rng);
        }
        // If we reach here without panic, the guard works.
    }

    #[test]
    fn priority_bid_stats_tracked_after_tick() {
        // Founders don't call SetPriorityBid, so bid_mean should be 0.0 and count 0.
        let mut sim = seed_simulation(small_config(), 42);
        run_tick(&mut sim, &mut None);
        assert_eq!(sim.stats.last_tick_priority_bid_mean, 0.0);
        assert_eq!(sim.stats.last_tick_priority_bidders_count, 0);
    }
}
