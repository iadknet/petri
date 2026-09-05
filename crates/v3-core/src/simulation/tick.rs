use std::collections::{HashMap, HashSet};
use std::time::Instant;

use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::{RngCore, SeedableRng};
use rayon::prelude::*;

use crate::config::{EnergyConfig, OrdinaryFoodTypeId};
use crate::contracts::{CreatureId, Direction, Position, WorldAction};
use crate::creature::action_log::{ActionLogEntry, ActionResult, ActionType, NO_DIRECTION};
use crate::creature::genome::BackendDef;
use crate::creature::state::CreatureState;
use crate::kernel::WorldState;
use crate::runtime::mesh::execute_creature_mesh_with_reserve;
use crate::runtime::plasticity::reward::apply_reward_modulated_updates;
use crate::runtime::plasticity::traces::has_any_reward_modulated;
use crate::runtime::trace::domain::{PerceptionDebugSnapshot, StaticInputsSnapshot, TickTrace};
use crate::runtime::trace::recording::ActiveTrace;
use crate::runtime::traced_mesh::execute_creature_mesh_traced_with_reserve;
use crate::runtime::types::MeshOutput;
use crate::sensors::perception::{
    genome_uses_extended_perception, PerceptionConfig, PerceptionSnapshot, SensorSnapshot,
};
use crate::sensors::reducers::assemble_perception;
use crate::sensors::static_inputs::assemble_static_inputs;
use crate::sensors::typed_food::{
    assemble_typed_food_local_snapshot, genome_uses_typed_local_food, TypedFoodLocalSnapshot,
};
use crate::sensors::visibility::{
    compute_visible_cells_into, get_visibility_table, VisibilityScratch,
};
use crate::simulation::actions::{
    apply_move, apply_noop, apply_reproduce, apply_steal_energy, apply_typed_eat,
    BarrierReaderState, MoveBlockedCause, PredationActionResult, ReproductionActionResult,
    ReproductionInvalidTargetCause,
};
use crate::simulation::outcomes::OutcomeAccumulator;
use crate::simulation::simulation::Simulation;
use crate::simulation::stats::SimStats;

#[path = "tick/helpers.rs"]
mod helpers;

pub(crate) use self::helpers::sort_by_priority_bid;
use self::helpers::{remove_creature_from_sim, remove_creature_if_dead};

#[cfg(test)]
#[path = "tick/tests/mod.rs"]
mod tests;

fn classify_move_blocked_cause(
    world: &WorldState,
    from: Position,
    dir: Direction,
) -> MoveBlockedCause {
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
    world: &WorldState,
    target: Option<Position>,
    successful_spawn_targets: &HashSet<Position>,
) -> ReproductionInvalidTargetCause {
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

fn has_neighbor_barrier(world: &WorldState, position: Position) -> bool {
    Direction::ALL.iter().any(|dir| {
        world
            .resolve_neighbor(position, *dir)
            .is_some_and(|neighbor| world.is_barrier(neighbor))
    })
}

fn has_any_valid_adjacent_target(world: &WorldState, position: Position) -> bool {
    Direction::ALL.iter().any(|dir| {
        world
            .resolve_neighbor(position, *dir)
            .is_some_and(|target| !world.is_barrier(target) && world.creature_at(target).is_none())
    })
}

fn barrier_reader_state_for_creature(creature: &CreatureState) -> BarrierReaderState {
    if creature.cached_has_barrier_reader {
        BarrierReaderState::HasBarrierReader
    } else {
        BarrierReaderState::NoBarrierReader
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
    let food_growth = sim.world.grow_food(sim.tick, &mut sim.rng);
    sim.stats.record_food_growth_summary(food_growth);

    // Steps 2 & 3: Age, energy decay, and shared memory snapshot + decay
    let decay_rate = sim.config.shared_memory.decay_rate;
    for (_, creature) in sim.creatures.iter_mut() {
        creature.age += 1;
        creature.energy -= sim.config.energy.lifecycle.energy_decay_per_tick;
        creature.lifetime_energy_sum += f64::from(creature.energy.max(0.0));
        creature.lifetime_energy_sample_count += 1;

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

/// Phase 1a: assemble sensor inputs sequentially over the post-Phase-0 world.
///
/// Conditional perception: only assemble full visibility + area reduction for
/// genomes that reference extended perception keys. Others use `zero()` fallback.
fn assemble_sensor_inputs(
    sim: &Simulation,
    queue: &[CreatureId],
) -> Vec<(CreatureId, SensorSnapshot)> {
    let perception_config = PerceptionConfig::from_sim_config(&sim.config);
    let vis_table = get_visibility_table(perception_config.vision_radius);
    let mut visible_scratch = VisibilityScratch::default();
    queue
        .iter()
        .filter(|&&id| sim.creatures.contains_key(id))
        .map(|&id| {
            let creature = &sim.creatures[id];
            let local = assemble_static_inputs(&sim.world, creature);
            let typed_local_food = if genome_uses_typed_local_food(&creature.genome) {
                assemble_typed_food_local_snapshot(&sim.world, creature.position)
            } else {
                TypedFoodLocalSnapshot::zero()
            };
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
            let ss = SensorSnapshot {
                local,
                typed_local_food,
                perception,
            };
            (id, ss)
        })
        .collect()
}

/// The complete action queue selected for one final-state observation.
///
/// The three queues are evaluated from the same frozen sensor snapshot. Each
/// evaluation owns fresh copies of the mutable cognition state, so this is
/// observational and never changes the simulation.
#[derive(Debug, Clone, PartialEq)]
pub struct FinalActionObservation {
    pub creature_id: CreatureId,
    pub intact: Vec<WorldAction>,
    pub zeroed: Vec<WorldAction>,
    pub scrambled: Vec<WorldAction>,
}

/// Observe every living creature's selected action queue at the simulation's
/// final state without advancing or mutating the simulation.
///
/// Creature IDs are sorted before sensor assembly to make the returned order
/// stable. The scramble is a fixed left rotation of the 16 shared-memory
/// slots; it preserves the memory multiset and does not consume the simulation
/// RNG. The normal mesh executor itself has no RNG input.
pub fn observe_final_actions(sim: &Simulation) -> Vec<FinalActionObservation> {
    let mut creature_ids: Vec<_> = sim.creatures.keys().collect();
    creature_ids.sort();
    let inputs = assemble_sensor_inputs(sim, &creature_ids);

    inputs
        .into_iter()
        .map(|(creature_id, sensors)| {
            let creature = &sim.creatures[creature_id];
            let mut scrambled_memory = creature.shared_memory;
            scrambled_memory.rotate_left(1);
            FinalActionObservation {
                creature_id,
                intact: observe_action_queue(
                    creature,
                    &sensors,
                    creature.shared_memory,
                    &sim.config.runtime,
                ),
                zeroed: observe_action_queue(creature, &sensors, [0.0; 16], &sim.config.runtime),
                scrambled: observe_action_queue(
                    creature,
                    &sensors,
                    scrambled_memory,
                    &sim.config.runtime,
                ),
            }
        })
        .collect()
}

fn observe_action_queue(
    creature: &CreatureState,
    sensors: &SensorSnapshot,
    mut shared_memory: [f32; 16],
    runtime_config: &crate::config::RuntimeConfig,
) -> Vec<WorldAction> {
    let mut energy = creature.energy;
    let mut graph_runtime = creature.graph_runtime.clone();
    execute_creature_mesh_with_reserve(
        &creature.genome,
        sensors,
        &mut energy,
        creature.reproductive_reserve,
        &mut shared_memory,
        &creature.prev_shared_memory,
        &mut graph_runtime,
        runtime_config,
    )
    .actions
}

/// Phase 1b: cognition — parallel for all creatures, sequential for the traced one.
///
/// Cognition only mutates each creature's private state (energy, memory,
/// `graph_runtime`); the world snapshot stays frozen for the whole phase.
fn run_cognition(
    sim: &mut Simulation,
    inputs: &[(CreatureId, SensorSnapshot)],
    trace: &mut Option<ActiveTrace>,
    trace_target: Option<CreatureId>,
) -> Vec<(CreatureId, MeshOutput)> {
    // Clone RuntimeConfig for cognition phase (small struct, ~7 scalars).
    let runtime_config = sim.config.runtime.clone();
    let tick_number = sim.tick;
    let mut creature_refs: HashMap<_, _> = sim.creatures.iter_mut().collect();

    // Extract traced creature (if any) before building parallel work vec.
    let traced_creature = trace_target.and_then(|tid| creature_refs.remove(&tid).map(|c| (tid, c)));

    let mut work: Vec<_> = inputs
        .iter()
        .filter_map(|(id, ss)| creature_refs.remove(id).map(|c| (*id, ss, c)))
        .collect();

    // Run all non-traced creatures in parallel.
    let mut parallel_decisions: Vec<_> = work
        .par_iter_mut()
        .map(|(id, ss, creature)| {
            let output = execute_creature_mesh_with_reserve(
                &creature.genome,
                ss,
                &mut creature.energy,
                creature.reproductive_reserve,
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
            let si_snapshot = StaticInputsSnapshot::from(&ss.local);

            let (output, hops, termination_reason) = execute_creature_mesh_traced_with_reserve(
                &creature.genome,
                ss,
                &mut creature.energy,
                creature.reproductive_reserve,
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
}

/// Per-tick compute-cost and deterministic work counters accumulated during
/// Phase 2, committed to [`SimStats`] once the creature queue is fully processed.
struct TickComputeStats {
    total_sum: f32,
    total_min: f32,
    total_max: f32,
    vm_sum: f32,
    vm_count: u32,
    graph_sum: f32,
    graph_count: u32,
    creature_count: u32,
    priority_bid_sum: f32,
    priority_bid_count: u32,
    mesh_hops: u64,
    vm_steps: u64,
    graph_relax_iters: u64,
    plasticity_updates: u64,
}

impl Default for TickComputeStats {
    fn default() -> Self {
        Self {
            total_sum: 0.0,
            total_min: f32::MAX,
            total_max: 0.0,
            vm_sum: 0.0,
            vm_count: 0,
            graph_sum: 0.0,
            graph_count: 0,
            creature_count: 0,
            priority_bid_sum: 0.0,
            priority_bid_count: 0,
            mesh_hops: 0,
            vm_steps: 0,
            graph_relax_iters: 0,
            plasticity_updates: 0,
        }
    }
}

impl TickComputeStats {
    /// Accumulate one creature's mesh output in queue order.
    ///
    /// Integer work counters commute exactly, so summing in queue order is
    /// deterministic regardless of the parallel mesh phase's thread scheduling.
    fn record(&mut self, output: &MeshOutput) {
        let compute_cost = &output.cost_report;
        let total_cost = compute_cost.vm_cost + compute_cost.graph_cost;
        self.total_sum += total_cost;
        if total_cost < self.total_min {
            self.total_min = total_cost;
        }
        if total_cost > self.total_max {
            self.total_max = total_cost;
        }
        if compute_cost.vm_cost > 0.0 {
            self.vm_sum += compute_cost.vm_cost;
            self.vm_count += 1;
        }
        if compute_cost.graph_cost > 0.0 {
            self.graph_sum += compute_cost.graph_cost;
            self.graph_count += 1;
        }
        self.creature_count += 1;

        let work = output.work_counters;
        self.mesh_hops += u64::from(work.mesh_hops);
        self.vm_steps += u64::from(work.vm_steps);
        self.graph_relax_iters += u64::from(work.graph_relax_iters);
        self.plasticity_updates += u64::from(work.plasticity_updates);

        self.priority_bid_sum += output.priority_bid;
        if output.priority_bid > 0.0 {
            self.priority_bid_count += 1;
        }
    }

    /// Write per-tick compute stats and commit deterministic work counters.
    ///
    /// `plasticity_updates_total` also receives the reward-modulated
    /// contribution from Phase 2.5, which runs after `decisions` is consumed
    /// and cannot flow through `MeshOutput`.
    fn commit(&self, stats: &mut SimStats) {
        if self.creature_count > 0 {
            stats.last_tick_compute_total_mean = self.total_sum / self.creature_count as f32;
            stats.last_tick_compute_total_min = self.total_min;
            stats.last_tick_compute_total_max = self.total_max;
            stats.last_tick_compute_vm_mean = if self.vm_count > 0 {
                self.vm_sum / self.vm_count as f32
            } else {
                0.0
            };
            stats.last_tick_compute_graph_mean = if self.graph_count > 0 {
                self.graph_sum / self.graph_count as f32
            } else {
                0.0
            };
            stats.last_tick_priority_bid_mean = self.priority_bid_sum / self.creature_count as f32;
            stats.last_tick_priority_bidders_count = self.priority_bid_count;
        }

        stats.mesh_hops_total += self.mesh_hops;
        stats.vm_steps_total += self.vm_steps;
        stats.graph_relax_iters_total += self.graph_relax_iters;
        stats.plasticity_updates_total += self.plasticity_updates;
        stats.creature_ticks_total += u64::from(self.creature_count);
        stats.actions_applied_total += u64::from(
            stats.last_tick_move
                + stats.last_tick_eat
                + stats.last_tick_noop
                + stats.last_tick_reproduce
                + stats.last_tick_steal,
        );
    }
}

/// Per-action bookkeeping shared by every Phase 2 action executor.
#[derive(Clone, Copy)]
struct ActionContext {
    id: CreatureId,
    tick: u64,
    priority_bid: f32,
    /// Creature energy captured before this action was applied.
    energy_before: f32,
    failed_action_penalty: f32,
}

/// Append one entry to a creature's action log, reading energy after the action.
fn push_action_log(
    sim: &mut Simulation,
    ctx: &ActionContext,
    action_type: ActionType,
    result: ActionResult,
    direction: u8,
    amount: f32,
    food_type: Option<OrdinaryFoodTypeId>,
) {
    let energy_after = sim.creatures.get(ctx.id).map_or(0.0, |c| c.energy);
    if let Some(log) = sim.action_logs.get_mut(ctx.id) {
        log.push(ActionLogEntry {
            tick: ctx.tick,
            action_type,
            result,
            direction,
            energy_before: ctx.energy_before,
            energy_after,
            amount,
            food_type,
            priority_bid: ctx.priority_bid,
        });
    }
}

/// Charge the complexity- and age-adjusted penalty for a failed action.
fn debit_failed_action(creature: &mut CreatureState, energy: &EnergyConfig, penalty: f32) {
    creature.energy -=
        energy.adjusted_action_cost(penalty, creature.cached_complexity, creature.age);
}

/// Barrier-awareness telemetry captured before a move or reproduce attempt.
///
/// `position` is `None` when the creature is already gone, in which case the
/// remaining fields carry the neutral "no barrier awareness" values.
#[derive(Clone, Copy)]
struct BarrierContext {
    position: Option<Position>,
    has_barrier_neighbor: bool,
    reader_state: BarrierReaderState,
    has_alternative_target: bool,
}

impl BarrierContext {
    fn for_creature(sim: &Simulation, id: CreatureId) -> Self {
        let Some(creature) = sim.creatures.get(id) else {
            return Self {
                position: None,
                has_barrier_neighbor: false,
                reader_state: BarrierReaderState::NoBarrierReader,
                has_alternative_target: false,
            };
        };
        let position = creature.position;
        Self {
            position: Some(position),
            has_barrier_neighbor: has_neighbor_barrier(&sim.world, position),
            reader_state: barrier_reader_state_for_creature(creature),
            has_alternative_target: has_any_valid_adjacent_target(&sim.world, position),
        }
    }
}

/// NoOp cannot fail; no `failed_action_penalty` is possible.
fn execute_noop(sim: &mut Simulation, ctx: &ActionContext, outcome_acc: &mut OutcomeAccumulator) {
    if let Some(creature) = sim.creatures.get_mut(ctx.id) {
        creature.lifetime_action_attempted_count += 1;
        apply_noop(creature, &sim.config);
        sim.stats.last_tick_noop += 1;
        outcome_acc.record_action_result(ctx.id, true);
    }
    push_action_log(
        sim,
        ctx,
        ActionType::NoOp,
        ActionResult::Success,
        NO_DIRECTION,
        0.0,
        None,
    );
}

fn execute_eat(
    sim: &mut Simulation,
    ctx: &ActionContext,
    type_idx: OrdinaryFoodTypeId,
    outcome_acc: &mut OutcomeAccumulator,
) {
    let mut action_result = ActionResult::Success;
    let mut amount = 0.0;
    if let Some(creature) = sim.creatures.get_mut(ctx.id) {
        creature.lifetime_action_attempted_count += 1;
        let food_before = sim.world.food_at_type(creature.position, type_idx);
        let succeeded = apply_typed_eat(creature, &mut sim.world, &sim.config, type_idx);
        sim.stats.last_tick_eat += 1;
        outcome_acc.record_action_result(ctx.id, succeeded);
        if succeeded {
            amount = food_before;
        } else {
            action_result = ActionResult::NoFood;
            debit_failed_action(creature, &sim.config.energy, ctx.failed_action_penalty);
        }
    }
    push_action_log(
        sim,
        ctx,
        ActionType::Eat,
        action_result,
        NO_DIRECTION,
        amount,
        Some(type_idx),
    );
}

fn execute_move(
    sim: &mut Simulation,
    ctx: &ActionContext,
    dir: Direction,
    outcome_acc: &mut OutcomeAccumulator,
) {
    let mut action_result = ActionResult::Success;
    let mut blocked_cause = None;
    let barrier = BarrierContext::for_creature(sim, ctx.id);
    if barrier.has_barrier_neighbor {
        *sim.stats
            .move_attempts_with_barrier_neighbor_total_by_reader_state
            .entry(barrier.reader_state)
            .or_insert(0) += 1;
    }
    if let Some(creature) = sim.creatures.get_mut(ctx.id) {
        creature.lifetime_action_attempted_count += 1;
        let from = creature.position;
        let succeeded = apply_move(ctx.id, creature, &mut sim.world, dir, &sim.config);
        sim.stats.last_tick_move += 1;
        outcome_acc.record_action_result(ctx.id, succeeded);
        if !succeeded {
            creature.lifetime_blocked_move_count += 1;
            blocked_cause = Some(classify_move_blocked_cause(&sim.world, from, dir));
            action_result = ActionResult::Blocked;
            debit_failed_action(creature, &sim.config.energy, ctx.failed_action_penalty);
        }
    }
    if let Some(cause) = blocked_cause {
        *sim.stats
            .move_actions_blocked_total_by_cause
            .entry(cause)
            .or_insert(0) += 1;
        if barrier.has_alternative_target {
            *sim.stats
                .move_actions_blocked_avoidable_total_by_reader_state
                .entry(barrier.reader_state)
                .or_insert(0) += 1;
        }
        if barrier.has_barrier_neighbor && matches!(cause, MoveBlockedCause::Barrier) {
            *sim.stats
                .move_blocked_barrier_with_barrier_neighbor_total_by_reader_state
                .entry(barrier.reader_state)
                .or_insert(0) += 1;
        }
    }
    push_action_log(
        sim,
        ctx,
        ActionType::Move,
        action_result,
        dir.to_index() as u8,
        0.0,
        None,
    );
}

fn execute_reproduce(
    sim: &mut Simulation,
    ctx: &ActionContext,
    direction: Direction,
    energy_transfer: f32,
    outcome_acc: &mut OutcomeAccumulator,
    reproduce_rng: &mut SmallRng,
    successful_spawn_targets: &mut HashSet<Position>,
) {
    let barrier = BarrierContext::for_creature(sim, ctx.id);
    let reproduction_target = barrier
        .position
        .and_then(|pos| sim.world.resolve_neighbor(pos, direction));
    if barrier.has_barrier_neighbor {
        *sim.stats
            .reproduction_attempts_with_barrier_neighbor_total_by_reader_state
            .entry(barrier.reader_state)
            .or_insert(0) += 1;
    }
    let result = apply_reproduce(ctx.id, sim, direction, energy_transfer, reproduce_rng);
    if let Some(creature) = sim.creatures.get_mut(ctx.id) {
        creature.lifetime_action_attempted_count += 1;
    }
    let succeeded = result == ReproductionActionResult::Spawned;
    outcome_acc.record_action_result(ctx.id, succeeded);
    if succeeded {
        if let Some(target) = reproduction_target {
            successful_spawn_targets.insert(target);
        }
        outcome_acc.record_offspring(ctx.id);
    }
    if result == ReproductionActionResult::RejectedInvalidTarget {
        if let Some(creature) = sim.creatures.get_mut(ctx.id) {
            creature.lifetime_invalid_reproduce_count += 1;
        }
        let cause = classify_reproduction_invalid_target_cause(
            &sim.world,
            reproduction_target,
            successful_spawn_targets,
        );
        *sim.stats
            .reproduction_actions_rejected_invalid_target_total_by_cause
            .entry(cause)
            .or_insert(0) += 1;
        if barrier.has_alternative_target {
            *sim.stats
                .reproduction_actions_rejected_invalid_target_avoidable_total_by_reader_state
                .entry(barrier.reader_state)
                .or_insert(0) += 1;
        }
        if barrier.has_barrier_neighbor && matches!(cause, ReproductionInvalidTargetCause::Barrier)
        {
            *sim.stats
                .reproduction_invalid_target_barrier_with_barrier_neighbor_total_by_reader_state
                .entry(barrier.reader_state)
                .or_insert(0) += 1;
        }
    }
    let action_result = match result {
        ReproductionActionResult::Spawned => ActionResult::Success,
        ReproductionActionResult::RejectedInvalidTarget => ActionResult::InvalidTarget,
        ReproductionActionResult::RejectedAgeConstraints => ActionResult::AgeConstraints,
        ReproductionActionResult::RejectedEnergyConstraints => ActionResult::EnergyConstraints,
        ReproductionActionResult::RejectedNutritionConstraints => {
            ActionResult::NutritionConstraints
        }
        ReproductionActionResult::RejectedPopulationCap => ActionResult::PopulationCap,
    };
    if !succeeded {
        if let Some(creature) = sim.creatures.get_mut(ctx.id) {
            debit_failed_action(creature, &sim.config.energy, ctx.failed_action_penalty);
        }
    }
    push_action_log(
        sim,
        ctx,
        ActionType::Reproduce,
        action_result,
        direction.to_index() as u8,
        energy_transfer,
        None,
    );
}

fn execute_steal_energy(
    sim: &mut Simulation,
    ctx: &ActionContext,
    direction: Direction,
    amount: f32,
    outcome_acc: &mut OutcomeAccumulator,
) {
    // Snapshot predation events length to extract damage info.
    let pred_events_before = sim.stats.last_tick_predation_events.len();
    if let Some(creature) = sim.creatures.get_mut(ctx.id) {
        creature.lifetime_action_attempted_count += 1;
    }
    let result = apply_steal_energy(ctx.id, sim, direction, amount);
    let succeeded = result != PredationActionResult::RejectedNoVictim;
    outcome_acc.record_action_result(ctx.id, succeeded);
    // Record damage to victim from predation event (if any).
    if succeeded {
        if let Some(event) = sim.stats.last_tick_predation_events.get(pred_events_before) {
            let victim_pos = Position::new(event.victim_x, event.victim_y);
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
        PredationActionResult::TransferredAndKilled => ActionResult::TransferredAndKilled,
        PredationActionResult::RejectedNoVictim => {
            if let Some(creature) = sim.creatures.get_mut(ctx.id) {
                debit_failed_action(creature, &sim.config.energy, ctx.failed_action_penalty);
            }
            ActionResult::NoVictim
        }
    };
    push_action_log(
        sim,
        ctx,
        ActionType::StealEnergy,
        action_result,
        direction.to_index() as u8,
        actual_stolen,
        None,
    );
}

/// Phase 2: sequential action execution.
///
/// Applies decisions in queue order, accumulating compute cost stats.
fn run_phase_2(
    sim: &mut Simulation,
    decisions: Vec<(CreatureId, MeshOutput)>,
    outcome_acc: &mut OutcomeAccumulator,
    reproduce_rng: &mut SmallRng,
) -> TickComputeStats {
    let mut compute = TickComputeStats::default();
    let mut successful_spawn_targets: HashSet<Position> = HashSet::new();
    let failed_action_penalty = sim.config.failed_action_penalty_for_tick(sim.tick);

    for (id, output) in decisions {
        compute.record(&output);

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
            let ctx = ActionContext {
                id,
                tick: current_tick,
                priority_bid,
                energy_before,
                failed_action_penalty,
            };

            match *action {
                WorldAction::NoOp => execute_noop(sim, &ctx, outcome_acc),
                WorldAction::Eat { type_idx } => execute_eat(sim, &ctx, type_idx, outcome_acc),
                WorldAction::Move(dir) => execute_move(sim, &ctx, dir, outcome_acc),
                WorldAction::Reproduce {
                    direction,
                    energy_transfer,
                } => execute_reproduce(
                    sim,
                    &ctx,
                    direction,
                    energy_transfer,
                    outcome_acc,
                    reproduce_rng,
                    &mut successful_spawn_targets,
                ),
                WorldAction::StealEnergy { direction, amount } => {
                    execute_steal_energy(sim, &ctx, direction, amount, outcome_acc);
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

    compute
}

/// Phase 2.5: reward-modulated learning pass.
///
/// For each creature with reward-modulated graph nodes, compute outcome signals
/// and apply three-factor weight updates using eligibility traces.
fn run_reward_learning(sim: &mut Simulation, outcome_acc: &OutcomeAccumulator) {
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
                    let (update_cost, update_count) = apply_reward_modulated_updates(
                        def,
                        node_idx,
                        &mut creature.graph_runtime.plasticity_weights,
                        &creature.graph_runtime.eligibility_traces,
                        &signals,
                        reward_cost,
                    );
                    sim.stats.plasticity_updates_total += u64::from(update_count);
                    creature.energy -= update_cost;
                }
            }
        }
        // Floor energy at 0.0 — same invariant as Phase 2 action costs.
        creature.energy = creature.energy.max(0.0);
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
/// sequentially with `execute_creature_mesh_traced`, recording detailed trace data.
/// When `None`, behavior is identical to the untraced path.
pub fn run_tick(sim: &mut Simulation, trace: &mut Option<ActiveTrace>) {
    // Reset per-tick counters at the start of each tick.
    sim.stats.reset_tick_counters();

    let phase_started = Instant::now();
    run_phase_0(sim);
    sim.stats.phase_wall_clock.world_update += phase_started.elapsed();

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
    let mut reproduce_rng = SmallRng::seed_from_u64(sim.rng.next_u64());

    // Determine if we need to trace a specific creature this tick.
    let trace_target: Option<CreatureId> = trace
        .as_ref()
        .filter(|t| !t.is_complete())
        .map(|t| t.creature_id);

    // ── Phase 1: Batch cognition (parallel, with optional trace extraction) ──
    let phase_started = Instant::now();
    let inputs = assemble_sensor_inputs(sim, &queue);
    sim.stats.phase_wall_clock.sensor_assembly += phase_started.elapsed();

    let phase_started = Instant::now();
    let mut decisions = run_cognition(sim, &inputs, trace, trace_target);
    sim.stats.phase_wall_clock.cognition += phase_started.elapsed();

    // Sort decisions by priority bid descending. Stable sort preserves the
    // pre-existing random shuffle order among creatures with equal bids.
    sort_by_priority_bid(&mut decisions);

    // ── Phase 2: Sequential action execution ────────────────────────────────
    let phase_started = Instant::now();
    let compute = run_phase_2(sim, decisions, &mut outcome_acc, &mut reproduce_rng);
    compute.commit(&mut sim.stats);
    sim.stats.phase_wall_clock.actions += phase_started.elapsed();

    // ── Phase 2.5: Reward-modulated learning pass ────────────────────────
    let phase_started = Instant::now();
    run_reward_learning(sim, &outcome_acc);
    sim.stats.phase_wall_clock.reward_learning += phase_started.elapsed();

    sim.tick += 1;
}

#[cfg(test)]
mod final_action_observation_tests {
    use super::*;
    use crate::config::SimulationConfig;
    use crate::contracts::{Direction, NodeId, OrdinaryFoodTypeId, WorldAction};
    use crate::creature::genome::cgp::{
        ActionSlot, ActionSlotBehavior, CgpGraphBackendDef, ComputeNode, ComputeNodeKind,
        ExecuteGate, GraphEdge, GraphSource, WorldActionKind,
    };
    use crate::creature::genome::{
        BackendDef, CreatureGenome, NodeGenome, VmBackendDef, VmInstruction,
    };
    use crate::simulation::seed_simulation;
    use proptest::prelude::*;
    use rand::RngCore;

    fn memory_direction_genome() -> CreatureGenome {
        CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 3,
                    constants: vec![0.5],
                    program: vec![
                        VmInstruction::LoadSlotImm {
                            dst: 0,
                            slot_idx: 0,
                        },
                        VmInstruction::LoadConst {
                            dst: 1,
                            const_idx: 0,
                        },
                        VmInstruction::CmpGt { dst: 2, a: 0, b: 1 },
                        VmInstruction::WriteWorldActionMeta {
                            slot_idx: 0,
                            src: 2,
                        },
                        VmInstruction::WriteWorldActionMeta {
                            slot_idx: 1,
                            src: 0,
                        },
                        VmInstruction::PushAction { action_type: 3 },
                        VmInstruction::ExecuteActionQueue,
                    ],
                }),
                targets: vec![],
            }],
        }
    }

    fn runtime_state_action_genome() -> CreatureGenome {
        let runtime_output = GraphEdge {
            source: GraphSource::ComputeNode(0),
            weight: 1.0,
        };
        CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![],
                backend_def: BackendDef::Graph(CgpGraphBackendDef {
                    compute_nodes: vec![ComputeNode {
                        kind: ComputeNodeKind::DecayIntegrator(0.0),
                        inputs: vec![],
                        plasticity: None,
                    }],
                    output_sinks: vec![],
                    action_bank: vec![ActionSlot {
                        behavior: ActionSlotBehavior::Emit(WorldActionKind::Eat),
                        gate_inputs: vec![runtime_output],
                        param_inputs: vec![],
                    }],
                    execute_gate: ExecuteGate {
                        inputs: vec![runtime_output],
                    },
                }),
                targets: vec![],
            }],
        }
    }

    fn expected_memory_action(value: f32) -> Vec<WorldAction> {
        vec![WorldAction::Reproduce {
            direction: if value > 0.5 {
                Direction::NE
            } else {
                Direction::N
            },
            energy_transfer: value,
        }]
    }

    #[test]
    fn final_action_observation_uses_full_actions_and_leaves_simulation_unchanged() {
        let mut config = SimulationConfig::default();
        config.world.width = 16;
        config.world.height = 16;
        config.population.initial_creatures = 1;
        let mut sim = seed_simulation(config, 11);
        let id = sim.creatures.keys().next().expect("one founder");
        let creature = &mut sim.creatures[id];
        creature.genome = memory_direction_genome();
        creature.shared_memory[0] = 1.0;
        creature.shared_memory[1] = 0.25;
        creature.prev_shared_memory[0] = 0.75;
        creature.graph_runtime.node_state = vec![vec![0.5]];
        creature.graph_runtime.plasticity_weights = vec![vec![Box::new([0.25])]];
        creature.graph_runtime.eligibility_traces = vec![vec![Box::new([0.125])]];
        creature.graph_runtime.scratch_prev = vec![1.0];
        creature.graph_runtime.scratch_curr = vec![2.0];
        creature.graph_runtime.scratch_backup = vec![3.0];
        creature.graph_runtime.scratch_w_inputs = vec![4.0];

        let memory_before = creature.shared_memory;
        let previous_memory_before = creature.prev_shared_memory;
        let energy_before = creature.energy;
        let graph_state_before = creature.graph_runtime.clone();
        let mut rng_before = sim.rng.clone();

        let observations = observe_final_actions(&sim);

        assert_eq!(observations.len(), 1);
        let observation = &observations[0];
        assert_eq!(observation.creature_id, id);
        assert_eq!(
            observation.intact,
            vec![WorldAction::Reproduce {
                direction: Direction::NE,
                energy_transfer: 1.0,
            }]
        );
        assert_ne!(observation.intact, observation.zeroed);
        assert_ne!(observation.intact, observation.scrambled);
        assert_eq!(sim.creatures[id].shared_memory, memory_before);
        assert_eq!(sim.creatures[id].prev_shared_memory, previous_memory_before);
        assert_eq!(sim.creatures[id].energy, energy_before);
        assert_eq!(
            sim.creatures[id].graph_runtime.node_state,
            graph_state_before.node_state
        );
        assert_eq!(
            sim.creatures[id].graph_runtime.plasticity_weights,
            graph_state_before.plasticity_weights
        );
        assert_eq!(
            sim.creatures[id].graph_runtime.eligibility_traces,
            graph_state_before.eligibility_traces
        );
        assert_eq!(
            sim.creatures[id].graph_runtime.scratch_prev,
            graph_state_before.scratch_prev
        );
        assert_eq!(
            sim.creatures[id].graph_runtime.scratch_curr,
            graph_state_before.scratch_curr
        );
        assert_eq!(
            sim.creatures[id].graph_runtime.scratch_backup,
            graph_state_before.scratch_backup
        );
        assert_eq!(
            sim.creatures[id].graph_runtime.scratch_w_inputs,
            graph_state_before.scratch_w_inputs
        );
        assert_eq!(sim.rng.next_u64(), rng_before.next_u64());
    }

    #[test]
    fn final_action_observation_preserves_learned_runtime_for_memory_insensitive_controller() {
        let mut config = SimulationConfig::default();
        config.world.width = 16;
        config.world.height = 16;
        config.population.initial_creatures = 1;
        let mut sim = seed_simulation(config, 11);
        let id = sim.creatures.keys().next().expect("one founder");
        let creature = &mut sim.creatures[id];
        creature.genome = runtime_state_action_genome();
        creature.shared_memory[0] = 1.0;
        creature.shared_memory[1] = 0.25;
        creature.graph_runtime.node_state = vec![vec![1.0]];

        let observations = observe_final_actions(&sim);

        let expected = vec![WorldAction::Eat {
            type_idx: OrdinaryFoodTypeId::new(0),
        }];
        let observation = observations.first().expect("one observation");
        assert_eq!(observation.intact, expected);
        assert_eq!(observation.zeroed, expected);
        assert_eq!(observation.scrambled, expected);
        assert_eq!(sim.creatures[id].graph_runtime.node_state, vec![vec![1.0]]);
    }

    proptest! {
        #[test]
        fn observation_scramble_reads_slot_one_while_intact_reads_slot_zero(
            slot_zero in 0.0f32..1.0,
            slot_one in 0.0f32..1.0,
        ) {
            let mut config = SimulationConfig::default();
            config.world.width = 16;
            config.world.height = 16;
            config.population.initial_creatures = 1;
            let mut sim = seed_simulation(config, 11);
            let id = sim.creatures.keys().next().expect("one founder");
            let creature = &mut sim.creatures[id];
            creature.genome = memory_direction_genome();
            creature.shared_memory[0] = slot_zero;
            creature.shared_memory[1] = slot_one;

            let observation = observe_final_actions(&sim).pop().expect("one observation");

            prop_assert_eq!(observation.intact, expected_memory_action(slot_zero));
            prop_assert_eq!(observation.zeroed, expected_memory_action(0.0));
            prop_assert_eq!(observation.scrambled, expected_memory_action(slot_one));
            prop_assert_eq!(sim.creatures[id].shared_memory[0], slot_zero);
            prop_assert_eq!(sim.creatures[id].shared_memory[1], slot_one);
        }
    }
}
