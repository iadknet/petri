//! Active sampler recording state (non-serialized).

use crate::contracts::CreatureId;
use crate::runtime::trace::domain::{ExecutionSample, TickTrace};

/// In-progress trace recording state.
#[derive(Debug)]
pub struct ActiveTrace {
    pub creature_id: CreatureId,
    pub ticks_remaining: u32,
    pub ticks: Vec<TickTrace>,
    /// Whether to capture extended perception debug snapshots per tick.
    pub include_perception_debug: bool,
}

impl ActiveTrace {
    /// Create a new trace recording request.
    pub fn new(creature_id: CreatureId, num_ticks: u32) -> Self {
        Self {
            creature_id,
            ticks_remaining: num_ticks,
            ticks: Vec::with_capacity(num_ticks as usize),
            include_perception_debug: false,
        }
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
