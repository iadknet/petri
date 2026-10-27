//! The creature-window recording the scheduler composes (T21.F04): each
//! recorded tick is runtime's brain-execution trace plus what the world
//! applied and left behind, kept under the recording's budget.

#[cfg(feature = "telemetry-seams")]
use std::mem::size_of;

#[cfg(feature = "telemetry-seams")]
use crate::config::SimulationConfig;
use crate::contracts::CreatureId;
#[cfg(feature = "telemetry-seams")]
use crate::contracts::OUTCOME_CHANNEL_COUNT;
#[cfg(feature = "telemetry-seams")]
use crate::contracts::{Position, WorldAction, SHARED_MEMORY_SLOTS};
#[cfg(feature = "telemetry-seams")]
use crate::creature::action_log::ActionLogEntry;
#[cfg(feature = "telemetry-seams")]
use crate::ledger::DeathCause;
#[cfg(feature = "telemetry-seams")]
use crate::runtime::trace::domain::MeshHopTrace;
use crate::runtime::trace::domain::{ExecutionSample, TickTrace};
#[cfg(feature = "telemetry-seams")]
use crate::runtime::trace::size::buffer;
#[cfg(feature = "telemetry-seams")]
use crate::runtime::traced_mesh::TruncationReason;
#[cfg(feature = "telemetry-seams")]
use crate::sensors::typed_food::TypedFoodLocalSnapshot;
#[cfg(feature = "telemetry-seams")]
use crate::simulation::stats::TickPhaseTimings;

/// One recorded tick: the brain's execution trace and, for a window
/// recording, what the tick applied and left behind.
#[derive(Debug, Clone)]
pub struct RecordedTick {
    pub trace: TickTrace,
    /// Built by cognition and filled once the tick has run (T21.F04).
    #[cfg(feature = "telemetry-seams")]
    pub outcome: Option<TickOutcome>,
}

/// One action the world applied to the traced creature, as `push_action_log`
/// built its entry, with the realized accounting from the `EnergyFlows`
/// deltas around it (T21.F04).
#[cfg(feature = "telemetry-seams")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AppliedAction {
    pub entry: ActionLogEntry,
    /// `action_charges` delta.
    pub charge: f64,
    /// `food_intake_by_type`, `predation_attacker_credit` and
    /// `predation_kill_bonus_credit` deltas.
    pub reward: f64,
}

/// The traced creature's state after the tick.
#[cfg(feature = "telemetry-seams")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StateAfter {
    pub position: Position,
    pub energy: f32,
    pub shared_memory: [f32; SHARED_MEMORY_SLOTS],
    pub previous_outcome: [f32; OUTCOME_CHANNEL_COUNT],
}

/// What a recorded tick applied and left behind (T21.F04).
#[cfg(feature = "telemetry-seams")]
#[derive(Debug, Clone)]
pub struct TickOutcome {
    /// Position at cognition.
    pub position: Position,
    /// Age at cognition.
    pub age: u64,
    /// The typed local food bank cognition read (empty when not read).
    pub typed_local_food: TypedFoodLocalSnapshot,
    /// The typed area food bank cognition read (empty when not read).
    pub typed_area_food: Vec<[f32; 7]>,
    /// Whether sensor assembly built the typed local food bank for the genome.
    pub uses_typed_local_food: bool,
    /// Whether sensor assembly built the extended perception banks.
    pub uses_extended_perception: bool,
    /// Applied actions in order.
    pub applied: Vec<AppliedAction>,
    /// Energy stolen from the creature this tick, lethal or not.
    pub damage_received: f32,
    pub offspring_spawned: u32,
    /// `None` when the creature was removed.
    pub after: Option<StateAfter>,
    /// The removal cause when the creature was removed this tick.
    pub died: Option<DeathCause>,
    /// The tick's phase timings.
    pub phases: Option<TickPhaseTimings>,
}

/// Hard caps on what one recording retains (T21.F04). An event is one hop,
/// pass or applied-action record; bytes are records' in-memory sizes
/// (`crate::runtime::trace::size` and the outcome sizes below).
#[cfg(feature = "telemetry-seams")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TraceBudget {
    pub max_events: u32,
    pub max_bytes: u64,
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
    pub ticks: Vec<RecordedTick>,
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
    pub removed: Option<DeathCause>,
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

    /// Convert the completed recording into a serializable sample of its
    /// ticks' execution traces.
    #[must_use]
    pub fn into_sample(self, creature_ffi_id: u64) -> ExecutionSample {
        ExecutionSample {
            creature_id: creature_ffi_id,
            ticks: self.ticks.into_iter().map(|tick| tick.trace).collect(),
        }
    }
}

#[cfg(feature = "telemetry-seams")]
impl TickOutcome {
    /// Bytes the outcome owns beyond its own `size_of`.
    #[must_use]
    pub fn heap_bytes(&self) -> u64 {
        buffer(&self.typed_local_food.food_here_by_type)
            + buffer(&self.typed_local_food.neighbor_food_by_type)
            + buffer(&self.typed_area_food)
            + buffer(&self.applied)
    }
}

#[cfg(feature = "telemetry-seams")]
impl RecordedTick {
    /// The recorded tick's bytes: its fixed part, the trace's records and
    /// its outcome.
    #[must_use]
    pub fn retained_bytes(&self) -> u64 {
        size_of::<Self>() as u64 + self.heap_bytes()
    }

    /// Bytes the recorded tick owns beyond its own `size_of`, which lives in
    /// the recording's tick buffer: the trace's and the outcome's.
    #[must_use]
    pub fn heap_bytes(&self) -> u64 {
        self.trace.heap_bytes() + self.outcome.as_ref().map_or(0, TickOutcome::heap_bytes)
    }
}

/// What a recorded tick reserves against the caps before its first hop.
/// The tick record's fixed part is not in it: that is a slot of the
/// recording's tick buffer, charged when the buffer grows.
#[cfg(feature = "telemetry-seams")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TickReserve {
    /// `max_actions_per_turn` applied-action events.
    pub events: u32,
    /// `max_actions_per_turn` selected and applied action records, the
    /// outcome's food banks at the configured type count, and the
    /// recorder's hop preallocation (`max_mesh_hops`).
    pub bytes: u64,
}

#[cfg(feature = "telemetry-seams")]
impl TickReserve {
    #[must_use]
    pub fn for_config(config: &SimulationConfig) -> Self {
        let actions = config.runtime.max_actions_per_turn;
        let types = config.world.food.types.len();
        let hops = config.runtime.max_mesh_hops.max(1) as usize;
        let bytes = actions * (size_of::<WorldAction>() + size_of::<AppliedAction>())
            + types * (size_of::<f32>() + size_of::<[f32; 8]>() + size_of::<[f32; 7]>())
            + hops * size_of::<MeshHopTrace>();
        Self {
            events: u32::try_from(actions).unwrap_or(u32::MAX),
            bytes: bytes as u64,
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
