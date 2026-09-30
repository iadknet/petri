//! Active sampler recording state (non-serialized).

use crate::contracts::CreatureId;
use crate::runtime::trace::domain::{ExecutionSample, TickTrace};

/// Hard caps on what one recording retains (T21.F04). An event is one hop,
/// pass or applied-action record; bytes are records' in-memory sizes
/// (`crate::runtime::trace::size`).
#[cfg(feature = "telemetry-seams")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TraceBudget {
    pub max_events: u32,
    pub max_bytes: u64,
}

/// Which cap ended a recording.
#[cfg(feature = "telemetry-seams")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TruncationReason {
    Events,
    Bytes,
}

/// Where a budgeted recording was cut: the tick (`TickTrace::tick_number`)
/// whose record did not fit, and the cap it met.
#[cfg(feature = "telemetry-seams")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Truncation {
    pub tick: u64,
    pub reason: TruncationReason,
}

/// In-progress trace recording state.
#[derive(Debug)]
pub struct ActiveTrace {
    pub creature_id: CreatureId,
    pub ticks_remaining: u32,
    pub ticks: Vec<TickTrace>,
    /// Whether to capture extended perception debug snapshots per tick.
    pub include_perception_debug: bool,
    /// The caps this recording keeps under; `None` records everything. A
    /// budgeted recording starts from [`Self::budgeted`], whose tick buffer
    /// holds no uncharged slot.
    #[cfg(feature = "telemetry-seams")]
    pub budget: Option<TraceBudget>,
    /// Events retained so far.
    #[cfg(feature = "telemetry-seams")]
    pub events: u32,
    /// Bytes retained so far: the tick buffer at capacity and every
    /// record's own buffers.
    #[cfg(feature = "telemetry-seams")]
    pub bytes: u64,
    /// Set once, when a record did not fit the budget; the recording then
    /// ends after that tick.
    #[cfg(feature = "telemetry-seams")]
    pub truncated: Option<Truncation>,
    /// The removal cause of a creature gone at a tick's start, recorded in
    /// that tick: a phase-0 death, a removal between ticks, or one in an
    /// earlier tick of a recording still running.
    #[cfg(feature = "telemetry-seams")]
    pub removed: Option<crate::simulation::energy_accounting::DeathCause>,
}

impl ActiveTrace {
    /// Create a new trace recording request.
    pub fn new(creature_id: CreatureId, num_ticks: u32) -> Self {
        Self {
            creature_id,
            ticks_remaining: num_ticks,
            ticks: Vec::with_capacity(num_ticks as usize),
            include_perception_debug: false,
            #[cfg(feature = "telemetry-seams")]
            budget: None,
            #[cfg(feature = "telemetry-seams")]
            events: 0,
            #[cfg(feature = "telemetry-seams")]
            bytes: 0,
            #[cfg(feature = "telemetry-seams")]
            truncated: None,
            #[cfg(feature = "telemetry-seams")]
            removed: None,
        }
    }

    /// A recording of `num_ticks` ticks kept under `budget`. Its tick buffer
    /// starts empty and grows one slot per recorded tick, each charged to
    /// `bytes` before it is allocated, so the counter covers every buffer
    /// the recording holds.
    #[cfg(feature = "telemetry-seams")]
    #[must_use]
    pub fn budgeted(creature_id: CreatureId, num_ticks: u32, budget: TraceBudget) -> Self {
        let mut trace = Self::new(creature_id, 0);
        trace.ticks_remaining = num_ticks;
        trace.budget = Some(budget);
        trace
    }

    /// Whether recording is complete (all requested ticks captured).
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.ticks_remaining == 0
    }

    /// Convert the completed recording into a serializable sample.
    #[must_use]
    pub fn into_sample(self, creature_ffi_id: u64) -> ExecutionSample {
        ExecutionSample {
            creature_id: creature_ffi_id,
            ticks: self.ticks,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use slotmap::SlotMap;

    #[test]
    fn active_trace_new_preallocates() {
        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        let id = sm.insert(());
        let trace = ActiveTrace::new(id, 5);
        assert_eq!(trace.ticks.capacity(), 5);
        assert_eq!(trace.ticks_remaining, 5);
        assert!(!trace.is_complete());
    }

    #[test]
    fn active_trace_is_complete_when_zero_remaining() {
        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        let id = sm.insert(());
        let mut trace = ActiveTrace::new(id, 1);
        assert!(!trace.is_complete());
        trace.ticks_remaining = 0;
        assert!(trace.is_complete());
    }

    #[test]
    fn into_sample_sets_creature_id() {
        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        let id = sm.insert(());
        let trace = ActiveTrace::new(id, 0);
        let sample = trace.into_sample(42);
        assert_eq!(sample.creature_id, 42);
        assert!(sample.ticks.is_empty());
    }

    #[test]
    fn active_trace_defaults_perception_debug_off() {
        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        let id = sm.insert(());
        let trace = ActiveTrace::new(id, 3);
        assert!(!trace.include_perception_debug);
    }
}
