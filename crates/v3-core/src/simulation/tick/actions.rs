//! Phase 2, the commit step (T30.F06): each queued action resolves as one
//! transaction in `simulation::actions`; this step reads the context before
//! it, delivers a span-0 conception, derives every record from the applied
//! result (the creature's history and action log, the reward count, the run
//! totals, the barrier telemetry) and settles the removals the action caused
//! through `Simulation::remove_creature`.

use std::collections::HashSet;

use rand::rngs::SmallRng;
use rand::Rng;

use super::helpers::remove_creature_if_dead;
#[cfg(feature = "telemetry-seams")]
use super::window_seam::FlowMarks;
use super::TickComputeStats;
use crate::contracts::{ActionOutcome, CreatureId, Direction, Position, WorldAction};
use crate::creature::action_log::{ActionLogEntry, NO_DIRECTION};
use crate::creature::state::CreatureState;
use crate::kernel::WorldState;
use crate::runtime::types::MeshOutput;
use crate::simulation::actions::{
    deliver_pending_birth, resolve, spawn_child, Applied, BarrierReaderState, BirthDelivery,
    GiveActionResult, MoveBlockedCause, PredationActionResult, PredationEventRecord,
    ReproductionActionResult, ReproductionInvalidTargetCause,
};
use crate::simulation::outcomes::OutcomeAccumulator;
use crate::simulation::simulation::Simulation;

/// Phase 2: sequential action execution.
///
/// Applies decisions in queue order, accumulating compute cost stats.
pub(super) fn run_phase_2(
    sim: &mut Simulation,
    decisions: Vec<(CreatureId, MeshOutput)>,
    outcome_acc: &mut OutcomeAccumulator,
    reproduce_rng: &mut SmallRng,
) -> TickComputeStats {
    let mut compute = TickComputeStats::default();
    let mut successful_spawn_targets: HashSet<Position> = HashSet::new();

    for (id, output) in decisions {
        compute.record(&output);
        sim.stats
            .energy_flows
            .record_cognition(output.energy_observation);

        // Skip all actions for creatures killed by earlier predation this tick.
        if !sim.creatures.contains_key(id) {
            continue;
        }

        // Gestation hold (T03.F12): a parent carrying a litter commits
        // nothing; on the span's last tick its child is born here first.
        if let Some(pending) = sim.creatures[id].pending_birth {
            // Phase 1 charges can exhaust a creature without removing it.
            if remove_creature_if_dead(sim, id) {
                continue;
            }
            if sim.tick >= pending.deliver_tick {
                // WORKAROUND(G4): the birth transaction of T29.F01 takes world context, not the simulation
                if let BirthDelivery::Delivered(cell) =
                    deliver_pending_birth(id, sim, reproduce_rng)
                {
                    successful_spawn_targets.insert(cell);
                    outcome_acc.record_offspring(id);
                }
            }
            sim.stats.gestation_creature_ticks_total += 1;
            continue;
        }

        for action in &output.actions {
            // The creature may have been removed by an earlier action.
            let Some(actor) = sim.creatures.get(id) else {
                break;
            };
            let ctx = ActionContext::read(
                &sim.world,
                sim.tick,
                output.priority_bid,
                id,
                actor,
                *action,
            );
            if let Some(barrier) = ctx.barrier {
                record_barrier_attempt(sim, action, barrier);
            }

            #[cfg(feature = "telemetry-seams")]
            let marks = (sim.stats.observed_creature == Some(id))
                .then(|| FlowMarks::of(&sim.stats.energy_flows));
            let applied = resolve(&mut sim.action_inputs(), id, action);
            #[cfg_attr(not(feature = "telemetry-seams"), allow(unused_variables))]
            let entry = commit(
                sim,
                &ctx,
                &applied,
                outcome_acc,
                &mut successful_spawn_targets,
                reproduce_rng,
            );

            #[cfg(feature = "telemetry-seams")]
            if let Some(marks) = marks {
                let applied = marks.applied(entry, &sim.stats.energy_flows);
                sim.stats.observed_actions.push(applied);
            }

            settle_killed_victim(sim, &applied);
            if applied.actor_removed {
                sim.remove_creature(id);
                break;
            }
            // A conception ends the queue: the rest is discarded with no
            // charge and no log entry (T03.F12). Held creatures never reach
            // this loop, so a pending birth here was conceived just now.
            if sim.creatures[id].pending_birth.is_some() {
                break;
            }
        }
    }

    compute
}

/// Per-action bookkeeping read before the action resolves.
#[derive(Clone, Copy)]
struct ActionContext {
    id: CreatureId,
    action: WorldAction,
    tick: u64,
    priority_bid: f32,
    /// Creature energy captured before this action was applied.
    energy_before: f32,
    /// The actor's cell before the action.
    position: Position,
    /// The cell's food of the eaten type before an eat; `0.0` otherwise.
    food_before: f32,
    /// The barrier context before a move or reproduce.
    barrier: Option<BarrierContext>,
}

impl ActionContext {
    fn read(
        world: &WorldState,
        tick: u64,
        priority_bid: f32,
        id: CreatureId,
        actor: &CreatureState,
        action: WorldAction,
    ) -> Self {
        let position = actor.position;
        let (food_before, barrier) = match action {
            WorldAction::Eat { type_idx, .. } => (world.food_at_type(position, type_idx), None),
            WorldAction::Move(_) | WorldAction::Reproduce { .. } => {
                (0.0, Some(BarrierContext::for_creature(world, actor)))
            }
            _ => (0.0, None),
        };
        Self {
            id,
            action,
            tick,
            priority_bid,
            energy_before: actor.energy,
            position,
            food_before,
            barrier,
        }
    }
}

/// Commit one resolved action: its records, a span-0 birth, its reward,
/// barrier telemetry and log entry. Returns the entry.
fn commit(
    sim: &mut Simulation,
    ctx: &ActionContext,
    applied: &Applied,
    outcome_acc: &mut OutcomeAccumulator,
    successful_spawn_targets: &mut HashSet<Position>,
    rng: &mut impl Rng,
) -> ActionLogEntry {
    let (id, action) = (ctx.id, &ctx.action);
    record_totals(sim, ctx, applied);
    if let (Some(conception), WorldAction::Reproduce { direction, .. }) =
        (applied.conceived, *action)
    {
        // WORKAROUND(G4): the birth transaction of T29.F01 takes world context, not the simulation
        spawn_child(
            id,
            sim,
            conception.target,
            direction,
            conception.litter,
            rng,
        );
        successful_spawn_targets.insert(conception.target);
        outcome_acc.record_offspring(id);
    }
    let result = applied.result;
    outcome_acc.record_action_result(id, result.outcome.succeeded());
    // A surviving victim takes the damage; a killed one is removed by the
    // caller.
    if let (WorldAction::StealEnergy { .. }, ActionOutcome::Success, Some(victim)) =
        (action, result.outcome, applied.counterpart)
    {
        outcome_acc.record_damage(victim, result.amount);
    }
    if let Some(barrier) = ctx.barrier {
        record_barrier_rejection(
            sim,
            action,
            result.outcome,
            barrier,
            successful_spawn_targets,
        );
    }
    push_action_log(sim, ctx, applied)
}

/// Remove the counterpart a `TransferredAndKilled` steal drained.
fn settle_killed_victim(sim: &mut Simulation, applied: &Applied) {
    if applied.result.outcome == ActionOutcome::TransferredAndKilled {
        if let Some(victim) = applied.counterpart {
            sim.remove_creature(victim);
        }
    }
}

/// The creature history and run totals an applied action derives, with the
/// keys the per-action result enums give them.
fn record_totals(sim: &mut Simulation, ctx: &ActionContext, applied: &Applied) {
    let (id, action) = (ctx.id, &ctx.action);
    let outcome = applied.result.outcome;
    let amount = applied.result.amount;
    let stats = &mut sim.stats;
    let creature = &mut sim.creatures[id];
    creature.record_action_attempt(action.action_type());
    match *action {
        WorldAction::NoOp => stats.last_tick_noop += 1,
        WorldAction::Eat { type_idx, .. } => {
            stats.last_tick_eat += 1;
            if outcome == ActionOutcome::Success {
                creature.record_applied_eat(type_idx);
                *stats
                    .eat_actions_applied_total_by_type
                    .entry(type_idx)
                    .or_insert(0) += 1;
                if amount < ctx.food_before {
                    *stats
                        .eat_actions_partial_total_by_type
                        .entry(type_idx)
                        .or_insert(0) += 1;
                }
            } else {
                *stats
                    .eat_actions_failed_total_by_type
                    .entry(type_idx)
                    .or_insert(0) += 1;
            }
        }
        WorldAction::Move(_) => {
            stats.last_tick_move += 1;
            stats.move_actions_attempted_total += 1;
            if outcome == ActionOutcome::Blocked {
                creature.lifetime_blocked_move_count += 1;
            }
        }
        WorldAction::Reproduce { .. } => {
            stats.reproduction_actions_attempted_total += 1;
            stats.last_tick_reproduce += 1;
            if outcome.succeeded() {
                stats.gestations_started_total += 1;
            } else {
                stats.reproduction_actions_rejected_total += 1;
                if outcome == ActionOutcome::LitterTooSmall {
                    stats.reproduction_actions_rejected_litter_total += 1;
                }
                if outcome == ActionOutcome::InvalidTarget {
                    creature.lifetime_invalid_reproduce_count += 1;
                }
                let reason = ReproductionActionResult::from_outcome(outcome)
                    .expect("a reproduce outcome has a reproduction key");
                *stats
                    .reproduction_actions_rejected_by_reason
                    .entry(reason)
                    .or_insert(0) += 1;
            }
        }
        WorldAction::StealEnergy { .. } => record_steal(sim, ctx, applied),
        WorldAction::GiveEnergy { .. } => {
            stats.give_actions_attempted_total += 1;
            stats.last_tick_give += 1;
            if outcome.succeeded() {
                stats.give_actions_transferred_total += 1;
            } else {
                stats.give_actions_rejected_total += 1;
            }
            let key =
                GiveActionResult::from_outcome(outcome).expect("a give outcome has a give key");
            *stats.give_actions_by_result.entry(key).or_insert(0) += 1;
        }
    }
}

/// A steal's run totals, kill and hit counts, and its predation event from
/// both creatures' cells, read before the victim's removal.
fn record_steal(sim: &mut Simulation, ctx: &ActionContext, applied: &Applied) {
    let id = ctx.id;
    let outcome = applied.result.outcome;
    let stats = &mut sim.stats;
    stats.predation_actions_attempted_total += 1;
    stats.last_tick_steal += 1;
    let key = PredationActionResult::from_outcome(outcome).expect("a steal outcome has a key");
    *stats.predation_actions_by_result.entry(key).or_insert(0) += 1;
    if !key.transferred() {
        stats.predation_actions_rejected_total += 1;
        return;
    }
    let victim = applied
        .counterpart
        .expect("a transferring steal has a victim");
    let amount = applied.result.amount;
    stats.predation_actions_transferred_total += 1;
    #[cfg(feature = "telemetry-seams")]
    if stats.observed_creature == Some(victim) {
        stats.observed_damage += amount;
    }
    let killed = outcome == ActionOutcome::TransferredAndKilled;
    if killed {
        sim.creatures[id].lifetime_predation_kills_count += 1;
        stats.predation_kills_total += 1;
        stats.last_tick_predation_kills += 1;
    } else {
        sim.creatures[victim].lifetime_predation_hits_taken_count += 1;
    }
    let victim_pos = sim.creatures[victim].position;
    stats.last_tick_predation_events.push(PredationEventRecord {
        attacker_x: ctx.position.x,
        attacker_y: ctx.position.y,
        victim_x: victim_pos.x,
        victim_y: victim_pos.y,
        energy_stolen: amount,
        killed,
    });
}

/// Append the action's entry to its creature's action log.
fn push_action_log(sim: &mut Simulation, ctx: &ActionContext, applied: &Applied) -> ActionLogEntry {
    let (id, action) = (ctx.id, &ctx.action);
    let entry = ActionLogEntry {
        tick: ctx.tick,
        action_type: action.action_type(),
        outcome: applied.result.outcome,
        direction: action
            .direction()
            .map_or(NO_DIRECTION, |direction| direction.to_index() as u8),
        energy_before: ctx.energy_before,
        energy_after: applied.energy_after,
        amount: applied.result.amount,
        food_type: match *action {
            WorldAction::Eat { type_idx, .. } => Some(type_idx),
            _ => None,
        },
        priority_bid: ctx.priority_bid,
    };
    #[cfg(all(test, feature = "telemetry-seams"))]
    super::window_seam::capture_logged(id, entry);
    if let Some(log) = sim.action_logs.get_mut(id) {
        log.push(entry);
    }
    entry
}

/// Resolve and commit `action` for `id` outside a tick, with a killed victim
/// removed and the actor kept whatever its energy, for the mechanics tests.
#[cfg(test)]
pub(crate) fn act_now(
    sim: &mut Simulation,
    id: CreatureId,
    action: WorldAction,
    rng: &mut impl Rng,
) -> Applied {
    let ctx = ActionContext::read(&sim.world, sim.tick, 0.0, id, &sim.creatures[id], action);
    let applied = resolve(&mut sim.action_inputs(), id, &action);
    commit(
        sim,
        &ctx,
        &applied,
        &mut OutcomeAccumulator::default(),
        &mut HashSet::new(),
        rng,
    );
    settle_killed_victim(sim, &applied);
    applied
}

/// A reproduce committed by [`act_now`], with its result's key.
#[cfg(test)]
pub(crate) fn reproduce_now(
    parent: CreatureId,
    sim: &mut Simulation,
    direction: Direction,
    energy_transfer_fraction: f32,
    rng: &mut impl Rng,
) -> ReproductionActionResult {
    let action = WorldAction::Reproduce {
        direction,
        energy_transfer_fraction,
    };
    let applied = act_now(sim, parent, action, rng);
    ReproductionActionResult::from_outcome(applied.result.outcome)
        .expect("a reproduce outcome has a reproduction key")
}

/// A steal committed by [`act_now`].
#[cfg(test)]
pub(crate) fn steal_now(
    attacker: CreatureId,
    sim: &mut Simulation,
    direction: Direction,
    effort: f32,
) -> Applied {
    let action = WorldAction::StealEnergy { direction, effort };
    act_now(
        sim,
        attacker,
        action,
        &mut <SmallRng as rand::SeedableRng>::seed_from_u64(0),
    )
}

/// A give committed by [`act_now`].
#[cfg(test)]
pub(crate) fn give_now(
    giver: CreatureId,
    sim: &mut Simulation,
    direction: Direction,
    effort: f32,
) -> Applied {
    let action = WorldAction::GiveEnergy { direction, effort };
    act_now(
        sim,
        giver,
        action,
        &mut <SmallRng as rand::SeedableRng>::seed_from_u64(0),
    )
}

/// Barrier-awareness telemetry captured before a move or reproduce attempt.
#[derive(Clone, Copy)]
struct BarrierContext {
    position: Position,
    has_barrier_neighbor: bool,
    reader_state: BarrierReaderState,
    has_alternative_target: bool,
}

impl BarrierContext {
    fn for_creature(world: &WorldState, creature: &CreatureState) -> Self {
        let position = creature.position;
        Self {
            position,
            has_barrier_neighbor: has_neighbor_barrier(world, position),
            reader_state: if creature.has_barrier_reader() {
                BarrierReaderState::HasBarrierReader
            } else {
                BarrierReaderState::NoBarrierReader
            },
            has_alternative_target: has_any_valid_adjacent_target(world, position),
        }
    }
}

/// Count a move or reproduce attempt made beside a barrier.
fn record_barrier_attempt(sim: &mut Simulation, action: &WorldAction, barrier: BarrierContext) {
    if !barrier.has_barrier_neighbor {
        return;
    }
    let by_reader_state = match action {
        WorldAction::Move(_) => {
            &mut sim
                .stats
                .move_attempts_with_barrier_neighbor_total_by_reader_state
        }
        _ => {
            &mut sim
                .stats
                .reproduction_attempts_with_barrier_neighbor_total_by_reader_state
        }
    };
    *by_reader_state.entry(barrier.reader_state).or_insert(0) += 1;
}

/// Classify a blocked move or an invalid-target reproduce and count it by
/// cause, as avoidable, and beside a barrier.
fn record_barrier_rejection(
    sim: &mut Simulation,
    action: &WorldAction,
    outcome: ActionOutcome,
    barrier: BarrierContext,
    successful_spawn_targets: &HashSet<Position>,
) {
    let stats = &mut sim.stats;
    match (*action, outcome) {
        (WorldAction::Move(dir), ActionOutcome::Blocked) => {
            let cause = classify_move_blocked_cause(&sim.world, barrier.position, dir);
            *stats
                .move_actions_blocked_total_by_cause
                .entry(cause)
                .or_insert(0) += 1;
            if barrier.has_alternative_target {
                *stats
                    .move_actions_blocked_avoidable_total_by_reader_state
                    .entry(barrier.reader_state)
                    .or_insert(0) += 1;
            }
            if barrier.has_barrier_neighbor && cause == MoveBlockedCause::Barrier {
                *stats
                    .move_blocked_barrier_with_barrier_neighbor_total_by_reader_state
                    .entry(barrier.reader_state)
                    .or_insert(0) += 1;
            }
        }
        (WorldAction::Reproduce { direction, .. }, ActionOutcome::InvalidTarget) => {
            let target = sim.world.resolve_neighbor(barrier.position, direction);
            let cause = classify_reproduction_invalid_target_cause(
                &sim.world,
                target,
                successful_spawn_targets,
            );
            *stats
                .reproduction_actions_rejected_invalid_target_total_by_cause
                .entry(cause)
                .or_insert(0) += 1;
            if barrier.has_alternative_target {
                *stats
                    .reproduction_actions_rejected_invalid_target_avoidable_total_by_reader_state
                    .entry(barrier.reader_state)
                    .or_insert(0) += 1;
            }
            if barrier.has_barrier_neighbor && cause == ReproductionInvalidTargetCause::Barrier {
                *stats
                    .reproduction_invalid_target_barrier_with_barrier_neighbor_total_by_reader_state
                    .entry(barrier.reader_state)
                    .or_insert(0) += 1;
            }
        }
        _ => {}
    }
}

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

pub(super) fn classify_reproduction_invalid_target_cause(
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
