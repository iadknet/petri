use std::collections::VecDeque;
use std::mem::size_of;

/// Action type discriminant for log entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[repr(u8)]
pub enum ActionType {
    NoOp = 0,
    Eat = 1,
    Move = 2,
    Reproduce = 3,
    StealEnergy = 4,
}

/// Outcome of an action for log entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[repr(u8)]
pub enum ActionResult {
    Success = 0,
    NoFood = 1,
    Blocked = 2,
    InvalidTarget = 3,
    EnergyConstraints = 4,
    PopulationCap = 5,
    TransferredAndKilled = 6,
    NoVictim = 7,
}

/// Single action record in a creature's action log.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
pub struct ActionLogEntry {
    /// Simulation tick when this action was executed.
    pub tick: u64,
    /// Action type.
    pub action_type: ActionType,
    /// Action result.
    pub result: ActionResult,
    /// Direction parameter (0-7 for cardinal+diagonal, 255=N/A).
    pub direction: u8,
    /// Creature energy BEFORE this action was applied.
    pub energy_before: f32,
    /// Creature energy AFTER this action was applied (includes costs and penalties).
    pub energy_after: f32,
    /// Action-specific amount (food consumed, energy transferred/stolen, 0.0 for Move/NoOp).
    pub amount: f32,
    /// Priority bid value for this tick.
    pub priority_bid: f32,
}

const _: () = assert!(size_of::<ActionLogEntry>() == 32);

/// Ring-buffer action log for a single creature.
#[derive(Debug)]
pub struct ActionLog {
    entries: VecDeque<ActionLogEntry>,
    capacity: usize,
}

impl ActionLog {
    /// Create a new action log with the given capacity, pre-allocated.
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    /// Push an entry, evicting the oldest if at capacity.
    pub fn push(&mut self, entry: ActionLogEntry) {
        if self.entries.len() == self.capacity {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
    }

    /// Access entries as a slice-pair (VecDeque may be non-contiguous).
    pub fn entries(&self) -> &VecDeque<ActionLogEntry> {
        &self.entries
    }

    /// Number of entries currently stored.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the log is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entry(tick: u64, action_type: ActionType) -> ActionLogEntry {
        ActionLogEntry {
            tick,
            action_type,
            result: ActionResult::Success,
            direction: 255,
            energy_before: 100.0,
            energy_after: 95.0,
            amount: 0.0,
            priority_bid: 0.5,
        }
    }

    #[test]
    fn new_log_is_empty() {
        let log = ActionLog::new(10);
        assert!(log.is_empty());
        assert_eq!(log.len(), 0);
    }

    #[test]
    fn push_adds_entries() {
        let mut log = ActionLog::new(10);
        log.push(sample_entry(1, ActionType::Eat));
        log.push(sample_entry(2, ActionType::Move));
        assert_eq!(log.len(), 2);
        assert_eq!(log.entries()[0].tick, 1);
        assert_eq!(log.entries()[1].tick, 2);
    }

    #[test]
    fn evicts_oldest_at_capacity() {
        let mut log = ActionLog::new(3);
        log.push(sample_entry(1, ActionType::NoOp));
        log.push(sample_entry(2, ActionType::Eat));
        log.push(sample_entry(3, ActionType::Move));
        assert_eq!(log.len(), 3);

        // Push a 4th — should evict tick 1
        log.push(sample_entry(4, ActionType::Reproduce));
        assert_eq!(log.len(), 3);
        assert_eq!(log.entries()[0].tick, 2);
        assert_eq!(log.entries()[2].tick, 4);
    }

    #[test]
    fn pre_allocates_with_capacity() {
        let log = ActionLog::new(500);
        assert!(log.entries.capacity() >= 500);
    }

    #[test]
    fn entry_fields_preserved() {
        let entry = ActionLogEntry {
            tick: 42,
            action_type: ActionType::StealEnergy,
            result: ActionResult::NoVictim,
            direction: 3,
            energy_before: 80.0,
            energy_after: 78.0,
            amount: 5.0,
            priority_bid: 0.8,
        };
        let mut log = ActionLog::new(10);
        log.push(entry);

        let stored = &log.entries()[0];
        assert_eq!(stored.tick, 42);
        assert_eq!(stored.action_type, ActionType::StealEnergy);
        assert_eq!(stored.result, ActionResult::NoVictim);
        assert_eq!(stored.direction, 3);
        assert!((stored.energy_before - 80.0).abs() < f32::EPSILON);
        assert!((stored.energy_after - 78.0).abs() < f32::EPSILON);
        assert!((stored.amount - 5.0).abs() < f32::EPSILON);
        assert!((stored.priority_bid - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn size_of_entry_is_32_bytes() {
        // Runtime check mirrors compile-time assertion
        assert_eq!(size_of::<ActionLogEntry>(), 32);
    }
}
