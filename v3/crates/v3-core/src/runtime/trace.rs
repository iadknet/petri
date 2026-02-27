//! Trace data structures for the Execution Sampler.
//!
//! These types record a creature's brain execution over multiple ticks,
//! capturing instruction-by-instruction detail for VM nodes and
//! pass-by-pass detail for graph nodes.

use crate::contracts::{InputReference, NodeId, WorldAction};
use crate::creature::genome::{GraphNodeKind, VmInstruction};
use crate::sensors::static_inputs::StaticInputs;
use serde::Serialize;

// ─── Top-level sample ────────────────────────────────────────────────────────

/// A completed execution sample containing traces for multiple ticks.
#[derive(Debug, Clone, Serialize)]
pub struct ExecutionSample {
    pub creature_id: u64,
    pub ticks: Vec<TickTrace>,
}

/// Trace data for a single tick's execution.
#[derive(Debug, Clone, Serialize)]
pub struct TickTrace {
    pub tick_number: u64,
    pub energy_before: f32,
    pub energy_after: f32,
    pub static_inputs: StaticInputsSnapshot,
    pub hops: Vec<MeshHopTrace>,
    pub final_action: WorldAction,
    pub termination_reason: TerminationReason,
}

/// Why the mesh chain terminated for this tick.
#[derive(Debug, Clone, Serialize)]
pub enum TerminationReason {
    ActionEmitted,
    EnergyExhausted,
    MaxHopsReached,
    NoTargets,
    MissingNode,
}

// ─── Static inputs snapshot ──────────────────────────────────────────────────

/// Serializable mirror of [`StaticInputs`].
///
/// `StaticInputs` derives `Debug, Clone, PartialEq` but NOT `Serialize`,
/// so we use this snapshot type for trace serialization.
#[derive(Debug, Clone, Serialize)]
pub struct StaticInputsSnapshot {
    pub food_here: f32,
    pub neighbor_food: [f32; 8],
    pub neighbor_barrier: [f32; 8],
    pub neighbor_occupied: [f32; 8],
    pub generation: f32,
    pub age_ticks: f32,
}

impl From<&StaticInputs> for StaticInputsSnapshot {
    fn from(si: &StaticInputs) -> Self {
        Self {
            food_here: si.food_here,
            neighbor_food: si.neighbor_food,
            neighbor_barrier: si.neighbor_barrier,
            neighbor_occupied: si.neighbor_occupied,
            generation: si.generation,
            age_ticks: si.age_ticks,
        }
    }
}

// ─── Mesh hop trace ──────────────────────────────────────────────────────────

/// Trace data for a single hop in the mesh chain.
#[derive(Debug, Clone, Serialize)]
pub struct MeshHopTrace {
    pub hop_index: usize,
    pub node_id: NodeId,
    pub input_refs: Vec<InputReference>,
    pub upstream_slots: [f32; 12],
    pub energy_before: f32,
    pub energy_after: f32,
    pub output_slots: [f32; 12],
    pub route_target_idx: f32,
    pub backend_trace: BackendTrace,
}

/// Discriminated union for backend-specific trace data.
#[derive(Debug, Clone, Serialize)]
pub enum BackendTrace {
    Vm(VmTrace),
    Graph(GraphTrace),
}

// ─── VM trace ────────────────────────────────────────────────────────────────

/// Trace of a VM node's execution.
#[derive(Debug, Clone, Serialize)]
pub struct VmTrace {
    pub register_count: u8,
    pub constants: Vec<f32>,
    pub steps: Vec<VmStepTrace>,
    pub final_registers: Vec<f32>,
    pub final_payload: [f32; 12],
    pub final_meta: [f32; 8],
    pub final_route_target: f32,
    pub memory_writes: Vec<MemoryWrite>,
}

/// Trace of a single VM instruction execution.
#[derive(Debug, Clone, Serialize)]
pub struct VmStepTrace {
    pub pc: usize,
    pub instruction: VmInstruction,
    pub energy_cost: f32,
    pub energy_after: f32,
    pub register_changes: Vec<(u8, f32)>,
}

/// Record of a single memory write operation.
#[derive(Debug, Clone, Serialize)]
pub struct MemoryWrite {
    pub address: u16,
    pub old_value: u8,
    pub new_value: u8,
}

// ─── Graph trace ─────────────────────────────────────────────────────────────

/// Trace of a graph node's relaxation loop.
#[derive(Debug, Clone, Serialize)]
pub struct GraphTrace {
    pub passes: Vec<GraphPassTrace>,
    pub converged: bool,
    pub stable_passes_count: u32,
    pub final_outputs: Vec<f32>,
}

/// Trace of a single relaxation pass.
#[derive(Debug, Clone, Serialize)]
pub struct GraphPassTrace {
    pub pass_index: u32,
    pub energy_cost: f32,
    pub energy_after: f32,
    pub node_evaluations: Vec<GraphNodeEvalTrace>,
    pub max_delta: f32,
}

/// Trace of a single internal graph node evaluation within a pass.
#[derive(Debug, Clone, Serialize)]
pub struct GraphNodeEvalTrace {
    pub node_index: usize,
    pub kind: &'static str,
    pub weighted_inputs: Vec<f32>,
    pub weighted_sum: f32,
    pub state_before: f32,
    pub state_after: f32,
    pub output: f32,
}

/// Map a [`GraphNodeKind`] variant to a static string label.
///
/// Uses `&'static str` to avoid heap allocation per node evaluation
/// in traced passes (`anti-format-hot-path`).
#[inline]
pub fn kind_label(kind: &GraphNodeKind) -> &'static str {
    match kind {
        GraphNodeKind::InputRef(_) => "InputRef",
        GraphNodeKind::Constant(_) => "Constant",
        GraphNodeKind::Add => "Add",
        GraphNodeKind::Multiply => "Multiply",
        GraphNodeKind::Negate => "Negate",
        GraphNodeKind::Abs => "Abs",
        GraphNodeKind::Min => "Min",
        GraphNodeKind::Max => "Max",
        GraphNodeKind::Threshold(_) => "Threshold",
        GraphNodeKind::GreaterThan => "GreaterThan",
        GraphNodeKind::Sigmoid => "Sigmoid",
        GraphNodeKind::Tanh => "Tanh",
        GraphNodeKind::Relu => "Relu",
        GraphNodeKind::Select => "Select",
        GraphNodeKind::Clamp01 => "Clamp01",
        GraphNodeKind::WeightedSum => "WeightedSum",
        GraphNodeKind::DecayIntegrator(_) => "DecayIntegrator",
        GraphNodeKind::Momentum(_) => "Momentum",
        GraphNodeKind::Oscillator(_) => "Oscillator",
        GraphNodeKind::AdaptiveGain => "AdaptiveGain",
        GraphNodeKind::CustomOutput(_) => "CustomOutput",
        GraphNodeKind::RouterOutput => "RouterOutput",
    }
}

// ─── Sample error ────────────────────────────────────────────────────────────

/// Errors that can occur during execution sampling.
///
/// Manual `Display` + `Error` impls used instead of `thiserror` because
/// `thiserror` is not a direct dependency of v3-core, and adding a crate
/// dependency for 2 variants is not warranted.
#[derive(Debug, Clone, Serialize)]
pub enum SampleError {
    CreatureNotFound,
    CreatureDied { ticks_completed: u32 },
}

impl std::fmt::Display for SampleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SampleError::CreatureNotFound => write!(f, "creature not found"),
            SampleError::CreatureDied { ticks_completed } => {
                write!(f, "creature died after {ticks_completed} ticks")
            }
        }
    }
}

impl std::error::Error for SampleError {}

// ─── Active trace (recording state) ─────────────────────────────────────────

use crate::contracts::CreatureId;

/// In-progress trace recording state.
///
/// Lives on `SimHandle` (v3-server), not `Simulation` — it's an
/// instrumentation concern, not a domain concept.
#[derive(Debug)]
pub struct ActiveTrace {
    pub creature_id: CreatureId,
    pub ticks_remaining: u32,
    pub ticks: Vec<TickTrace>,
}

impl ActiveTrace {
    /// Create a new trace recording request.
    pub fn new(creature_id: CreatureId, num_ticks: u32) -> Self {
        Self {
            creature_id,
            ticks_remaining: num_ticks,
            ticks: Vec::with_capacity(num_ticks as usize),
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
    fn static_inputs_snapshot_from_static_inputs() {
        let si = StaticInputs {
            food_here: 0.75,
            neighbor_food: [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8],
            neighbor_barrier: [1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0],
            neighbor_occupied: [0.0; 8],
            generation: 5.0,
            age_ticks: 42.0,
        };
        let snapshot = StaticInputsSnapshot::from(&si);
        assert_eq!(snapshot.food_here, 0.75);
        assert_eq!(snapshot.neighbor_food, si.neighbor_food);
        assert_eq!(snapshot.neighbor_barrier, si.neighbor_barrier);
        assert_eq!(snapshot.generation, 5.0);
        assert_eq!(snapshot.age_ticks, 42.0);
    }

    #[test]
    fn kind_label_covers_all_variants() {
        assert_eq!(kind_label(&GraphNodeKind::InputRef(0)), "InputRef");
        assert_eq!(kind_label(&GraphNodeKind::Constant(1.0)), "Constant");
        assert_eq!(kind_label(&GraphNodeKind::Add), "Add");
        assert_eq!(kind_label(&GraphNodeKind::Multiply), "Multiply");
        assert_eq!(kind_label(&GraphNodeKind::Negate), "Negate");
        assert_eq!(kind_label(&GraphNodeKind::Abs), "Abs");
        assert_eq!(kind_label(&GraphNodeKind::Min), "Min");
        assert_eq!(kind_label(&GraphNodeKind::Max), "Max");
        assert_eq!(kind_label(&GraphNodeKind::Threshold(0.5)), "Threshold");
        assert_eq!(kind_label(&GraphNodeKind::GreaterThan), "GreaterThan");
        assert_eq!(kind_label(&GraphNodeKind::Sigmoid), "Sigmoid");
        assert_eq!(kind_label(&GraphNodeKind::Tanh), "Tanh");
        assert_eq!(kind_label(&GraphNodeKind::Relu), "Relu");
        assert_eq!(kind_label(&GraphNodeKind::Select), "Select");
        assert_eq!(kind_label(&GraphNodeKind::Clamp01), "Clamp01");
        assert_eq!(kind_label(&GraphNodeKind::WeightedSum), "WeightedSum");
        assert_eq!(
            kind_label(&GraphNodeKind::DecayIntegrator(0.5)),
            "DecayIntegrator"
        );
        assert_eq!(kind_label(&GraphNodeKind::Momentum(0.5)), "Momentum");
        assert_eq!(kind_label(&GraphNodeKind::Oscillator(1.0)), "Oscillator");
        assert_eq!(kind_label(&GraphNodeKind::AdaptiveGain), "AdaptiveGain");
        assert_eq!(kind_label(&GraphNodeKind::CustomOutput(0)), "CustomOutput");
        assert_eq!(kind_label(&GraphNodeKind::RouterOutput), "RouterOutput");
    }

    #[test]
    fn sample_error_display() {
        let e1 = SampleError::CreatureNotFound;
        assert_eq!(e1.to_string(), "creature not found");

        let e2 = SampleError::CreatureDied { ticks_completed: 3 };
        assert_eq!(e2.to_string(), "creature died after 3 ticks");
    }
}
