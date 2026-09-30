//! The creature-window seam (T21.F04): the per-tick budget reserve, the
//! applied-action capture and the outcome join `run_tick` performs for a
//! traced creature. Plain data; nothing here changes what the tick does.

use crate::config::SimulationConfig;
use crate::contracts::CreatureId;
use crate::creature::action_log::ActionLogEntry;
use crate::runtime::trace::domain::{AppliedAction, StateAfter, TickOutcome};
use crate::runtime::trace::recording::{ActiveTrace, Truncation, TruncationReason};
use crate::runtime::trace::size::TickReserve;
use crate::runtime::traced_mesh::RecordLimit;
use crate::simulation::energy_accounting::EnergyFlows;
use crate::simulation::outcomes::OutcomeAccumulator;
use crate::simulation::simulation::Simulation;

/// The creature `trace` records this tick, when it records one.
pub(super) fn observed(trace: &Option<ActiveTrace>) -> Option<CreatureId> {
    trace
        .as_ref()
        .filter(|active| !active.is_complete())
        .map(|active| active.creature_id)
}

/// Charges the tick's reserve for a budgeted, still-recording trace: a tick
/// whose reserve does not fit either cap is not recorded and the recording
/// ends before it, truncated.
pub(super) fn reserve_tick(trace: &mut Option<ActiveTrace>, config: &SimulationConfig, tick: u64) {
    let Some(active) = trace.as_mut().filter(|active| !active.is_complete()) else {
        return;
    };
    let Some(budget) = active.budget else {
        return;
    };
    let reserve = TickReserve::for_config(config);
    let reason =
        if u64::from(active.events) + u64::from(reserve.events) > u64::from(budget.max_events) {
            Some(TruncationReason::Events)
        } else if active.bytes.saturating_add(reserve.bytes) > budget.max_bytes {
            Some(TruncationReason::Bytes)
        } else {
            None
        };
    if let Some(reason) = reason {
        active.truncated = Some(Truncation { tick, reason });
        active.ticks_remaining = 0;
    }
}

/// What the tick's mesh records may take once its reserve is charged;
/// `None` records everything.
pub(super) fn record_limit(active: &ActiveTrace, config: &SimulationConfig) -> Option<RecordLimit> {
    let budget = active.budget?;
    let reserve = TickReserve::for_config(config);
    Some(RecordLimit {
        events: budget
            .max_events
            .saturating_sub(active.events)
            .saturating_sub(reserve.events),
        bytes: budget
            .max_bytes
            .saturating_sub(active.bytes)
            .saturating_sub(reserve.bytes),
    })
}

/// The outcome of the tick `trace` recorded as `tick`, if it recorded it.
pub(super) fn recorded_outcome(
    trace: &mut Option<ActiveTrace>,
    tick: u64,
) -> Option<&mut TickOutcome> {
    trace
        .as_mut()?
        .ticks
        .last_mut()
        .filter(|record| record.tick_number == tick)?
        .outcome
        .as_mut()
}

/// The `EnergyFlows` sums an applied action's accounting is read from.
#[derive(Clone, Copy)]
pub(super) struct FlowMarks {
    charge: f64,
    penalty: f64,
    reward: f64,
}

impl FlowMarks {
    pub(super) fn of(flows: &EnergyFlows) -> Self {
        let charges = &flows.action_charges;
        Self {
            charge: charges.noop
                + charges.eat
                + charges.r#move
                + charges.reproduce
                + charges.steal_energy,
            penalty: flows.failed_action_penalty,
            reward: flows.food_intake_by_type.iter().sum::<f64>()
                + flows.predation_attacker_credit
                + flows.predation_kill_bonus_credit,
        }
    }

    /// `entry` with the flows' deltas since `self`.
    pub(super) fn applied(self, entry: ActionLogEntry, flows: &EnergyFlows) -> AppliedAction {
        let now = Self::of(flows);
        AppliedAction {
            entry,
            charge: now.charge - self.charge,
            penalty: now.penalty - self.penalty,
            reward: now.reward - self.reward,
        }
    }
}

/// Fills the recorded tick's outcome once the tick has run, and counts what
/// the record retains against the budget.
pub(super) fn finish_tick(
    sim: &Simulation,
    trace: &mut Option<ActiveTrace>,
    outcome_acc: &OutcomeAccumulator,
    tick: u64,
) {
    let Some(id) = observed_record(trace, tick) else {
        return;
    };
    let Some(outcome) = recorded_outcome(trace, tick) else {
        return;
    };
    outcome
        .applied
        .extend_from_slice(&sim.stats.observed_actions);
    outcome.damage_received = sim.stats.observed_damage;
    outcome.offspring_spawned = outcome_acc.offspring_spawned(id);
    outcome.died = sim.stats.observed_removal;
    outcome.after = sim.creatures.get(id).map(|creature| StateAfter {
        position: creature.position,
        energy: creature.energy,
        shared_memory: creature.shared_memory,
        previous_outcome: creature.previous_outcome,
    });
    outcome.phases = sim.stats.last_tick_phases;
    let Some(active) = trace.as_mut().filter(|active| active.budget.is_some()) else {
        return;
    };
    let record = active.ticks.last().expect("the recorded tick");
    let events = record.hops.len()
        + record.passes.len()
        + record.outcome.as_ref().map_or(0, |o| o.applied.len());
    active.events = active
        .events
        .saturating_add(u32::try_from(events).unwrap_or(u32::MAX));
    active.bytes = active.bytes.saturating_add(record.retained_bytes());
}

/// The traced creature, when `trace` recorded `tick`.
fn observed_record(trace: &Option<ActiveTrace>, tick: u64) -> Option<CreatureId> {
    let active = trace.as_ref()?;
    active
        .ticks
        .last()
        .filter(|record| record.tick_number == tick)
        .map(|_| active.creature_id)
}
