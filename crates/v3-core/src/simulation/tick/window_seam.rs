//! The creature-window seam (T21.F04): the per-tick budget reserve, the
//! applied-action capture and the outcome join `run_tick` performs for a
//! traced creature. Plain data; nothing here changes what the tick does.

use std::mem::size_of;

use crate::config::SimulationConfig;
use crate::contracts::CreatureId;
use crate::creature::action_log::ActionLogEntry;
use crate::runtime::trace::domain::{AppliedAction, StateAfter, TickTrace};
use crate::runtime::trace::recording::{ActiveTrace, Truncation, TruncationReason};
use crate::runtime::trace::size::TickReserve;
use crate::runtime::traced_mesh::RecordLimit;
use crate::simulation::energy_accounting::EnergyFlows;
use crate::simulation::outcomes::OutcomeAccumulator;
use crate::simulation::simulation::Simulation;

/// One creature's entry in a tick's sorted decision queue, as the action
/// phase receives it: the traced/untraced equivalence probe.
#[cfg(test)]
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Decided {
    pub(super) id: CreatureId,
    pub(super) actions: Vec<crate::contracts::WorldAction>,
    pub(super) priority_bid: f32,
    pub(super) termination_reason: crate::runtime::trace::domain::TerminationReason,
    pub(super) commit_counts: [u32; crate::creature::genome::VOTE_KIND_COUNT],
}

#[cfg(test)]
thread_local! {
    /// The last tick's decision queue on this thread.
    static DECIDED: std::cell::RefCell<Vec<Decided>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Stores the tick's sorted decision queue for [`take_decided`].
#[cfg(test)]
pub(super) fn capture_decided(decisions: &[(CreatureId, crate::runtime::types::MeshOutput)]) {
    let queue = decisions
        .iter()
        .map(|(id, output)| Decided {
            id: *id,
            actions: output.actions.clone(),
            priority_bid: output.priority_bid,
            termination_reason: output.termination_reason,
            commit_counts: output.commit_counts,
        })
        .collect();
    DECIDED.set(queue);
}

/// The decision queue of the last tick run on this thread.
#[cfg(test)]
pub(super) fn take_decided() -> Vec<Decided> {
    DECIDED.take()
}

/// The creature `trace` records this tick, when it records one.
pub(super) fn observed(trace: &Option<ActiveTrace>) -> Option<CreatureId> {
    trace
        .as_ref()
        .filter(|active| !active.is_complete())
        .map(|active| active.creature_id)
}

/// Charges the tick's reserve for a budgeted, still-recording trace, with
/// the tick buffer's new slot when the buffer is full: a tick whose reserve
/// does not fit either cap is not recorded and the recording ends before
/// it, truncated; otherwise the buffer grows by the charged slot.
pub(super) fn reserve_tick(trace: &mut Option<ActiveTrace>, config: &SimulationConfig, tick: u64) {
    let Some(active) = trace.as_mut().filter(|active| !active.is_complete()) else {
        return;
    };
    let Some(budget) = active.budget else {
        return;
    };
    debug_assert!(
        active.ticks.capacity() <= active.ticks.len() + 1,
        "a budgeted recording starts from `ActiveTrace::budgeted`"
    );
    let slot = size_of::<TickTrace>() as u64;
    let growth = if active.ticks.len() == active.ticks.capacity() {
        slot
    } else {
        0
    };
    let reserve = TickReserve::for_config(config);
    let reason =
        if u64::from(active.events) + u64::from(reserve.events) > u64::from(budget.max_events) {
            Some(TruncationReason::Events)
        } else if active.bytes.saturating_add(growth + reserve.bytes) > budget.max_bytes {
            Some(TruncationReason::Bytes)
        } else {
            None
        };
    if let Some(reason) = reason {
        active.truncated = Some(Truncation { tick, reason });
        active.ticks_remaining = 0;
    } else if growth > 0 {
        let before = active.ticks.capacity();
        active.ticks.reserve_exact(1);
        active.bytes += (active.ticks.capacity() - before) as u64 * slot;
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
    let Some(active) = trace.as_mut() else {
        return;
    };
    let id = active.creature_id;
    let Some(record) = active
        .ticks
        .last_mut()
        .filter(|record| record.tick_number == tick)
    else {
        return;
    };
    let Some(outcome) = record.outcome.as_mut() else {
        return;
    };
    outcome
        .applied
        .extend_from_slice(&sim.stats.observed_actions);
    outcome.damage_received = sim.stats.observed_damage;
    outcome.offspring_spawned = outcome_acc.offspring_spawned(id);
    outcome.died = sim.stats.removal_of(id);
    outcome.after = sim.creatures.get(id).map(|creature| StateAfter {
        position: creature.position,
        energy: creature.energy,
        shared_memory: creature.shared_memory,
        previous_outcome: creature.previous_outcome,
    });
    outcome.phases = sim.stats.last_tick_phases;
    if active.budget.is_none() {
        return;
    }
    let events = record.hops.len() + record.passes.len() + outcome.applied.len();
    let bytes = record.heap_bytes();
    active.events = active
        .events
        .saturating_add(u32::try_from(events).unwrap_or(u32::MAX));
    active.bytes = active.bytes.saturating_add(bytes);
}
