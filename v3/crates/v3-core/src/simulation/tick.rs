use crate::creature::action_log::{ActionLogEntry, ActionResult, ActionType};
use crate::simulation::simulation::Simulation;

#[path = "tick/helpers.rs"]
mod helpers;

pub(crate) use self::helpers::sort_by_priority_bid;
use self::helpers::{remove_creature_from_sim, remove_creature_if_dead};

#[cfg(test)]
#[path = "tick/tests/mod.rs"]
mod tests;

fn classify_move_blocked_cause(
    world: &crate::kernel::WorldState,
    from: crate::contracts::Position,
    dir: crate::contracts::Direction,
) -> crate::simulation::actions::MoveBlockedCause {
    use crate::simulation::actions::MoveBlockedCause;

    let Some(target) = world.resolve_neighbor(from, dir) else {
        return MoveBlockedCause::OutOfBounds;
    };
    if world.is_barrier(target) {
        return MoveBlockedCause::Barrier;
    }
    if world.creature_at(target).is_some() {
        return MoveBlockedCause::Occupied;
    }
    MoveBlockedCause::OutOfBounds
}

fn classify_reproduction_invalid_target_cause(
    world: &crate::kernel::WorldState,
    target: Option<crate::contracts::Position>,
    successful_spawn_targets: &std::collections::HashSet<crate::contracts::Position>,
) -> crate::simulation::actions::ReproductionInvalidTargetCause {
    use crate::simulation::actions::ReproductionInvalidTargetCause;

    let Some(target) = target else {
        return ReproductionInvalidTargetCause::OutOfBounds;
    };
    if successful_spawn_targets.contains(&target) {
        return ReproductionInvalidTargetCause::Contention;
    }
    if world.is_barrier(target) {
        return ReproductionInvalidTargetCause::Barrier;
    }
    if world.creature_at(target).is_some() {
        return ReproductionInvalidTargetCause::Occupied;
    }
    ReproductionInvalidTargetCause::OutOfBounds
}

fn has_neighbor_barrier(
    world: &crate::kernel::WorldState,
    position: crate::contracts::Position,
) -> bool {
    crate::contracts::Direction::ALL.iter().any(|dir| {
        world
            .resolve_neighbor(position, *dir)
            .is_some_and(|neighbor| world.is_barrier(neighbor))
    })
}

fn has_any_valid_adjacent_target(
    world: &crate::kernel::WorldState,
    position: crate::contracts::Position,
) -> bool {
    crate::contracts::Direction::ALL.iter().any(|dir| {
        world
            .resolve_neighbor(position, *dir)
            .is_some_and(|target| !world.is_barrier(target) && world.creature_at(target).is_none())
    })
}

fn barrier_reader_state_for_creature(
    creature: &crate::creature::state::CreatureState,
) -> crate::simulation::actions::BarrierReaderState {
    if creature.cached_has_barrier_reader {
        crate::simulation::actions::BarrierReaderState::HasBarrierReader
    } else {
        crate::simulation::actions::BarrierReaderState::NoBarrierReader
    }
}

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

    // Steps 2 & 3: Age, energy decay, and shared memory snapshot + decay
    let decay_rate = sim.config.shared_memory.decay_rate;
    for (_, creature) in sim.creatures.iter_mut() {
        creature.age += 1;
        creature.energy -= sim.config.energy.lifecycle.energy_decay_per_tick;

        // Snapshot shared_memory → prev_shared_memory, then apply decay.
        creature.prev_shared_memory = creature.shared_memory;
        if decay_rate > 0.0 {
            let factor = 1.0 - decay_rate;
            for slot in &mut creature.shared_memory {
                *slot *= factor;
            }
        }
    }

    // Step 4: Collect dead IDs first to avoid borrow conflict during removal.
    let dead_ids: Vec<_> = sim
        .creatures
        .iter()
        .filter(|(_, c)| c.energy <= 0.0)
        .map(|(id, _)| id)
        .collect();

    for id in dead_ids {
        remove_creature_from_sim(sim, id);
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
pub fn run_tick(
    sim: &mut Simulation,
    trace: &mut Option<crate::runtime::trace::recording::ActiveTrace>,
) {
    use rand::seq::SliceRandom;
    use rand::RngCore;
    use rand::SeedableRng;

    use std::collections::{HashMap, HashSet};

    use rayon::prelude::*;

    use crate::contracts::{CreatureId, WorldAction};
    use crate::runtime::mesh::execute_creature_mesh;
    use crate::runtime::trace::domain::{PerceptionDebugSnapshot, StaticInputsSnapshot, TickTrace};
    use crate::runtime::traced_mesh::execute_creature_mesh_traced;
    use crate::runtime::types::MeshOutput;
    use crate::sensors::perception::{
        genome_uses_extended_perception, PerceptionConfig, PerceptionSnapshot, SensorSnapshot,
    };
    use crate::sensors::reducers::assemble_perception;
    use crate::sensors::static_inputs::assemble_static_inputs;
    use crate::sensors::visibility::{
        compute_visible_cells_into, get_visibility_table, VisibilityScratch,
    };
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
    let mut visible_scratch = VisibilityScratch::default();
    let inputs: Vec<_> = queue
        .iter()
        .filter(|&&id| sim.creatures.contains_key(id))
        .map(|&id| {
            let creature = &sim.creatures[id];
            let local = assemble_static_inputs(&sim.world, creature);
            let perception = if genome_uses_extended_perception(&creature.genome) {
                let visible = compute_visible_cells_into(
                    creature.position,
                    &sim.world,
                    vis_table,
                    &mut visible_scratch,
                );
                assemble_perception(
                    id,
                    creature,
                    visible,
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
                    &mut creature.shared_memory,
                    &creature.prev_shared_memory,
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
                    &mut creature.shared_memory,
                    &creature.prev_shared_memory,
                    &mut creature.graph_runtime,
                    &runtime_config,
                );

                // Record tick trace.
                if let Some(ref mut active) = trace {
                    let debug_perception = if active.include_perception_debug {
                        Some(PerceptionDebugSnapshot::from(&ss.perception))
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
    let mut successful_spawn_targets: HashSet<crate::contracts::Position> = HashSet::new();
    let failed_action_penalty = sim.config.failed_action_penalty_for_tick(sim.tick);

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

        // Apply each queued action sequentially, recording to action log.
        let current_tick = sim.tick;
        let priority_bid = output.priority_bid;
        for action in &output.actions {
            // Capture energy before action (creature may have been killed).
            let energy_before = match sim.creatures.get(id) {
                Some(c) => c.energy,
                None => break,
            };

            match *action {
                WorldAction::NoOp => {
                    // NoOp cannot fail; no failed_action_penalty possible.
                    if let Some(creature) = sim.creatures.get_mut(id) {
                        apply_noop(creature, &sim.config);
                        sim.stats.last_tick_noop += 1;
                        outcome_acc.record_action_result(id, true);
                    }
                    if let Some(log) = sim.action_logs.get_mut(id) {
                        log.push(ActionLogEntry {
                            tick: current_tick,
                            action_type: ActionType::NoOp,
                            result: ActionResult::Success,
                            direction: 255,
                            energy_before,
                            energy_after: sim.creatures.get(id).map_or(0.0, |c| c.energy),
                            amount: 0.0,
                            priority_bid,
                        });
                    }
                }
                WorldAction::Eat => {
                    let mut action_result = ActionResult::Success;
                    let mut amount = 0.0;
                    if let Some(creature) = sim.creatures.get_mut(id) {
                        let food_before = sim.world.food_at(creature.position);
                        let succeeded = apply_eat(creature, &mut sim.world, &sim.config);
                        sim.stats.last_tick_eat += 1;
                        outcome_acc.record_action_result(id, succeeded);
                        if succeeded {
                            amount = food_before;
                        } else {
                            action_result = ActionResult::NoFood;
                            creature.energy -= sim.config.energy.adjusted_action_cost(
                                failed_action_penalty,
                                creature.cached_complexity,
                                creature.age,
                            );
                        }
                    }
                    if let Some(log) = sim.action_logs.get_mut(id) {
                        log.push(ActionLogEntry {
                            tick: current_tick,
                            action_type: ActionType::Eat,
                            result: action_result,
                            direction: 255,
                            energy_before,
                            energy_after: sim.creatures.get(id).map_or(0.0, |c| c.energy),
                            amount,
                            priority_bid,
                        });
                    }
                }
                WorldAction::Move(dir) => {
                    let mut action_result = ActionResult::Success;
                    let mut blocked_cause = None;
                    let (has_barrier_neighbor, barrier_reader_state, has_alternative_target) = sim
                        .creatures
                        .get(id)
                        .map(|creature| {
                            let position = creature.position;
                            (
                                has_neighbor_barrier(&sim.world, position),
                                barrier_reader_state_for_creature(creature),
                                has_any_valid_adjacent_target(&sim.world, position),
                            )
                        })
                        .unwrap_or((
                            false,
                            crate::simulation::actions::BarrierReaderState::NoBarrierReader,
                            false,
                        ));
                    if has_barrier_neighbor {
                        *sim.stats
                            .move_attempts_with_barrier_neighbor_total_by_reader_state
                            .entry(barrier_reader_state)
                            .or_insert(0) += 1;
                    }
                    if let Some(creature) = sim.creatures.get_mut(id) {
                        let from = creature.position;
                        let succeeded = apply_move(id, creature, &mut sim.world, dir, &sim.config);
                        sim.stats.last_tick_move += 1;
                        outcome_acc.record_action_result(id, succeeded);
                        if !succeeded {
                            blocked_cause =
                                Some(classify_move_blocked_cause(&sim.world, from, dir));
                            action_result = ActionResult::Blocked;
                            creature.energy -= sim.config.energy.adjusted_action_cost(
                                failed_action_penalty,
                                creature.cached_complexity,
                                creature.age,
                            );
                        }
                    }
                    if let Some(cause) = blocked_cause {
                        *sim.stats
                            .move_actions_blocked_total_by_cause
                            .entry(cause)
                            .or_insert(0) += 1;
                        if has_alternative_target {
                            *sim.stats
                                .move_actions_blocked_avoidable_total_by_reader_state
                                .entry(barrier_reader_state)
                                .or_insert(0) += 1;
                        }
                        if has_barrier_neighbor
                            && matches!(
                                cause,
                                crate::simulation::actions::MoveBlockedCause::Barrier
                            )
                        {
                            *sim.stats
                                .move_blocked_barrier_with_barrier_neighbor_total_by_reader_state
                                .entry(barrier_reader_state)
                                .or_insert(0) += 1;
                        }
                    }
                    if let Some(log) = sim.action_logs.get_mut(id) {
                        log.push(ActionLogEntry {
                            tick: current_tick,
                            action_type: ActionType::Move,
                            result: action_result,
                            direction: dir.to_index() as u8,
                            energy_before,
                            energy_after: sim.creatures.get(id).map_or(0.0, |c| c.energy),
                            amount: 0.0,
                            priority_bid,
                        });
                    }
                }
                WorldAction::Reproduce {
                    direction,
                    energy_transfer,
                } => {
                    let (
                        has_barrier_neighbor,
                        barrier_reader_state,
                        has_alternative_target,
                        reproduction_target,
                    ) = sim
                        .creatures
                        .get(id)
                        .map(|creature| {
                            let position = creature.position;
                            (
                                has_neighbor_barrier(&sim.world, position),
                                barrier_reader_state_for_creature(creature),
                                has_any_valid_adjacent_target(&sim.world, position),
                                sim.world.resolve_neighbor(position, direction),
                            )
                        })
                        .unwrap_or((
                            false,
                            crate::simulation::actions::BarrierReaderState::NoBarrierReader,
                            false,
                            None,
                        ));
                    if has_barrier_neighbor {
                        *sim.stats
                            .reproduction_attempts_with_barrier_neighbor_total_by_reader_state
                            .entry(barrier_reader_state)
                            .or_insert(0) += 1;
                    }
                    let result =
                        apply_reproduce(id, sim, direction, energy_transfer, &mut reproduce_rng);
                    let succeeded = result == ReproductionActionResult::Spawned;
                    outcome_acc.record_action_result(id, succeeded);
                    if succeeded {
                        if let Some(target) = reproduction_target {
                            successful_spawn_targets.insert(target);
                        }
                        outcome_acc.record_offspring(id);
                    }
                    if result == ReproductionActionResult::RejectedInvalidTarget {
                        let cause = classify_reproduction_invalid_target_cause(
                            &sim.world,
                            reproduction_target,
                            &successful_spawn_targets,
                        );
                        *sim.stats
                            .reproduction_actions_rejected_invalid_target_total_by_cause
                            .entry(cause)
                            .or_insert(0) += 1;
                        if has_alternative_target {
                            *sim.stats
                                .reproduction_actions_rejected_invalid_target_avoidable_total_by_reader_state
                                .entry(barrier_reader_state)
                                .or_insert(0) += 1;
                        }
                        if has_barrier_neighbor
                            && matches!(
                                cause,
                                crate::simulation::actions::ReproductionInvalidTargetCause::Barrier
                            )
                        {
                            *sim.stats
                                .reproduction_invalid_target_barrier_with_barrier_neighbor_total_by_reader_state
                                .entry(barrier_reader_state)
                                .or_insert(0) += 1;
                        }
                    }
                    let action_result = match result {
                        ReproductionActionResult::Spawned => ActionResult::Success,
                        ReproductionActionResult::RejectedInvalidTarget => {
                            ActionResult::InvalidTarget
                        }
                        ReproductionActionResult::RejectedEnergyConstraints => {
                            ActionResult::EnergyConstraints
                        }
                        ReproductionActionResult::RejectedPopulationCap => {
                            ActionResult::PopulationCap
                        }
                    };
                    if !succeeded {
                        if let Some(creature) = sim.creatures.get_mut(id) {
                            creature.energy -= sim.config.energy.adjusted_action_cost(
                                failed_action_penalty,
                                creature.cached_complexity,
                                creature.age,
                            );
                        }
                    }
                    if let Some(log) = sim.action_logs.get_mut(id) {
                        log.push(ActionLogEntry {
                            tick: current_tick,
                            action_type: ActionType::Reproduce,
                            result: action_result,
                            direction: direction.to_index() as u8,
                            energy_before,
                            energy_after: sim.creatures.get(id).map_or(0.0, |c| c.energy),
                            amount: energy_transfer,
                            priority_bid,
                        });
                    }
                }
                WorldAction::StealEnergy { direction, amount } => {
                    // Snapshot predation events length to extract damage info.
                    let pred_events_before = sim.stats.last_tick_predation_events.len();
                    let result = apply_steal_energy(id, sim, direction, amount);
                    let succeeded = result != PredationActionResult::RejectedNoVictim;
                    outcome_acc.record_action_result(id, succeeded);
                    // Record damage to victim from predation event (if any).
                    if succeeded {
                        if let Some(event) =
                            sim.stats.last_tick_predation_events.get(pred_events_before)
                        {
                            let victim_pos =
                                crate::contracts::Position::new(event.victim_x, event.victim_y);
                            if let Some(victim_id) = sim.world.creature_at(victim_pos) {
                                outcome_acc.record_damage(victim_id, event.energy_stolen);
                            }
                        }
                    }
                    // Extract stolen amount from predation event (if any).
                    let actual_stolen = sim
                        .stats
                        .last_tick_predation_events
                        .get(pred_events_before)
                        .map_or(0.0, |e| e.energy_stolen);
                    let action_result = match result {
                        PredationActionResult::Transferred => ActionResult::Success,
                        PredationActionResult::TransferredAndKilled => {
                            ActionResult::TransferredAndKilled
                        }
                        PredationActionResult::RejectedNoVictim => {
                            if let Some(creature) = sim.creatures.get_mut(id) {
                                creature.energy -= sim.config.energy.adjusted_action_cost(
                                    failed_action_penalty,
                                    creature.cached_complexity,
                                    creature.age,
                                );
                            }
                            ActionResult::NoVictim
                        }
                    };
                    if let Some(log) = sim.action_logs.get_mut(id) {
                        log.push(ActionLogEntry {
                            tick: current_tick,
                            action_type: ActionType::StealEnergy,
                            result: action_result,
                            direction: direction.to_index() as u8,
                            energy_before,
                            energy_after: sim.creatures.get(id).map_or(0.0, |c| c.energy),
                            amount: actual_stolen,
                            priority_bid,
                        });
                    }
                }
            }

            // Floor energy at 0.0 — creatures cannot spend more than they have.
            if let Some(creature) = sim.creatures.get_mut(id) {
                creature.energy = creature.energy.max(0.0);
            }

            if remove_creature_if_dead(sim, id) {
                break;
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
            // Floor energy at 0.0 — same invariant as Phase 2 action costs.
            creature.energy = creature.energy.max(0.0);
        }
    }

    sim.tick += 1;
}
