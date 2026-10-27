//! Phase 2, the commit step (T30.F06): each queued action resolves as one
//! transaction in `simulation::actions`; this step reads the context before
//! it, delivers a span-0 conception, writes the records the result derives
//! (the action log, the reward count, the barrier telemetry) and settles the
//! removals the action caused through `Simulation::remove_creature`.

use std::collections::HashSet;

use rand::rngs::SmallRng;

use super::helpers::remove_creature_if_dead;
#[cfg(feature = "telemetry-seams")]
use super::window_seam::FlowMarks;
use super::TickComputeStats;
use crate::contracts::{CreatureId, Direction, Position, WorldAction};
use crate::creature::action_log::{ActionLogEntry, ActionResult, ActionType};
use crate::creature::state::CreatureState;
use crate::kernel::WorldState;
use crate::runtime::types::MeshOutput;
use crate::simulation::actions::{
    deliver_pending_birth, resolve, spawn_child, Applied, BarrierReaderState, BirthDelivery,
    MoveBlockedCause, ReproductionInvalidTargetCause,
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
            let ctx = ActionContext {
                id,
                tick: sim.tick,
                priority_bid: output.priority_bid,
                energy_before: actor.energy,
            };
            let barrier = match *action {
                WorldAction::Move(_) | WorldAction::Reproduce { .. } => {
                    Some(BarrierContext::for_creature(&sim.world, actor))
                }
                _ => None,
            };
            if let Some(barrier) = barrier {
                record_barrier_attempt(sim, action, barrier);
            }

            #[cfg(feature = "telemetry-seams")]
            let marks = (sim.stats.observed_creature == Some(id))
                .then(|| FlowMarks::of(&sim.stats.energy_flows));
            let applied = resolve(&mut sim.action_inputs(), id, action);

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
                    reproduce_rng,
                );
                successful_spawn_targets.insert(conception.target);
                outcome_acc.record_offspring(id);
            }
            outcome_acc.record_action_result(id, applied.result.succeeded());
            // A surviving victim takes the damage; a killed one is removed below.
            if let (ActionType::StealEnergy, ActionResult::Success, Some(victim)) = (
                applied.result.action_type,
                applied.result.result,
                applied.counterpart,
            ) {
                outcome_acc.record_damage(victim, applied.result.amount);
            }
            if let Some(barrier) = barrier {
                record_barrier_rejection(sim, action, &applied, barrier, &successful_spawn_targets);
            }
            #[cfg_attr(not(feature = "telemetry-seams"), allow(unused_variables))]
            let entry = push_action_log(sim, &ctx, &applied);

            #[cfg(feature = "telemetry-seams")]
            if let Some(marks) = marks {
                let applied = marks.applied(entry, &sim.stats.energy_flows);
                sim.stats.observed_actions.push(applied);
            }

            if applied.result.result == ActionResult::TransferredAndKilled {
                if let Some(victim) = applied.counterpart {
                    sim.remove_creature(victim);
                }
            }
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
    tick: u64,
    priority_bid: f32,
    /// Creature energy captured before this action was applied.
    energy_before: f32,
}

/// Append the action's entry to its creature's action log.
fn push_action_log(sim: &mut Simulation, ctx: &ActionContext, applied: &Applied) -> ActionLogEntry {
    let logged = applied.result;
    let entry = ActionLogEntry {
        tick: ctx.tick,
        action_type: logged.action_type,
        result: logged.result,
        direction: logged.direction,
        energy_before: ctx.energy_before,
        energy_after: applied.energy_after,
        amount: logged.amount,
        food_type: logged.food_type,
        priority_bid: ctx.priority_bid,
    };
    #[cfg(all(test, feature = "telemetry-seams"))]
    super::window_seam::capture_logged(ctx.id, entry);
    if let Some(log) = sim.action_logs.get_mut(ctx.id) {
        log.push(entry);
    }
    entry
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
    applied: &Applied,
    barrier: BarrierContext,
    successful_spawn_targets: &HashSet<Position>,
) {
    let stats = &mut sim.stats;
    match (*action, applied.result.result) {
        (WorldAction::Move(dir), ActionResult::Blocked) => {
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
        (WorldAction::Reproduce { direction, .. }, ActionResult::InvalidTarget) => {
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
