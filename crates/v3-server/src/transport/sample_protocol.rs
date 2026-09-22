//! HTTP sampler response DTOs owned by the transport layer.

use serde::Serialize;
use v3_core::creature::genome::vote::{VOTE_KIND_COUNT, VOTE_SINK_COUNT};
use v3_core::runtime::OUTPUT_SLOT_COUNT;

/// Serialized input-reference shape owned by transport wire DTOs.
pub type InputReferencePayload = serde_json::Value;
/// Serialized world-action shape owned by transport wire DTOs.
pub type WorldActionPayload = serde_json::Value;
/// Serialized VM-instruction shape owned by transport wire DTOs.
pub type VmInstructionPayload = serde_json::Value;

#[derive(Debug, Clone, Serialize)]
pub struct ExecutionSamplePayload {
    pub creature_id: u64,
    pub ticks: Vec<TickTracePayload>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TickTracePayload {
    pub tick_number: u64,
    pub energy_before: f32,
    pub energy_after: f32,
    pub static_inputs: StaticInputsSnapshotPayload,
    pub debug_perception: Option<PerceptionDebugSnapshotPayload>,
    pub hops: Vec<MeshHopTracePayload>,
    /// One record per pass of the tick (T19.F04).
    pub passes: Vec<MeshPassTracePayload>,
    pub final_actions: Vec<WorldActionPayload>,
    pub termination_reason: TerminationReasonPayload,
    pub priority_bid: f32,
    /// Final per-kind bars: commits of each kind this tick (T19.F04).
    pub commit_counts: [u32; VOTE_KIND_COUNT],
}

/// Why the tick's pass loop ended (T19.F04).
#[derive(Debug, Clone, Serialize)]
pub enum TerminationReasonPayload {
    NoDecision,
    TerminateVoted,
    ActionCapReached,
    EnergyExhausted,
}

/// Why one pass ended (T19.F04).
#[derive(Debug, Clone, Serialize)]
pub enum PassEndReasonPayload {
    Decided,
    PassCapReached,
    NoTargets,
    MissingNode,
    EnergyExhausted,
}

/// One pass of a tick (T19.F04).
#[derive(Debug, Clone, Serialize)]
pub struct MeshPassTracePayload {
    pub pass_index: u32,
    pub end_reason: PassEndReasonPayload,
    /// The pass's final vote vector, in `VoteSink` index order.
    pub votes: [f32; VOTE_SINK_COUNT],
    /// Effective vote per kind against the bars the pass started with.
    pub effective_votes: [f32; VOTE_KIND_COUNT],
    pub committed: Option<WorldActionPayload>,
    pub hops: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct StaticInputsSnapshotPayload {
    pub food_here: f32,
    pub neighbor_food: [f32; 8],
    pub neighbor_barrier: [f32; 8],
    pub neighbor_occupied: [f32; 8],
    pub age_ticks: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct PerceptionDebugSnapshotPayload {
    pub area_food: [f32; 7],
    pub area_barrier: [f32; 7],
    pub area_occupancy: [f32; 7],
    pub nearby_core: [f32; 16],
    pub nearby_vitals: [f32; 8],
    pub nearby_identity: [f32; 12],
}

#[derive(Debug, Clone, Serialize)]
pub struct MeshHopTracePayload {
    pub hop_index: usize,
    /// The pass this hop belongs to (T19.F04).
    pub pass_index: u32,
    pub node_id: u64,
    pub input_refs: Vec<InputReferencePayload>,
    pub upstream_slots: [f32; OUTPUT_SLOT_COUNT],
    pub energy_before: f32,
    pub energy_after: f32,
    pub output_slots: [f32; OUTPUT_SLOT_COUNT],
    pub route: Option<RouteDecisionPayload>,
    /// The vote contribution this hop committed (T19.F03).
    pub vote_contribution: [f32; VOTE_SINK_COUNT],
    pub backend_trace: BackendTracePayload,
}

#[derive(Debug, Clone, Serialize)]
pub struct RouteDecisionPayload {
    pub gate_scores: Vec<GateScorePayload>,
    pub selected_target_idx: usize,
    pub selected_target_id: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct GateScorePayload {
    pub slot: u8,
    pub target_id: u32,
    pub gate_bias: f32,
    pub runtime_score: f32,
    pub effective_score: f32,
}

#[derive(Debug, Clone, Serialize)]
pub enum BackendTracePayload {
    Vm(VmTracePayload),
    Graph(GraphTracePayload),
}

#[derive(Debug, Clone, Serialize)]
pub struct VmTracePayload {
    pub register_count: u8,
    pub constants: Vec<f32>,
    pub steps: Vec<VmStepTracePayload>,
    pub final_registers: Vec<f32>,
    pub final_payload: [f32; OUTPUT_SLOT_COUNT],
    pub slot_writes: Vec<SlotWritePayload>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VmStepTracePayload {
    pub pc: usize,
    pub instruction: VmInstructionPayload,
    pub energy_cost: f32,
    pub energy_after: f32,
    pub register_changes: Vec<(u8, f32)>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SlotWritePayload {
    pub slot_idx: u8,
    pub old_value: f32,
    pub new_value: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct GraphTracePayload {
    pub passes: Vec<GraphPassTracePayload>,
    pub converged: bool,
    pub stable_passes_count: u32,
    pub final_outputs: Vec<f32>,
    pub output_sinks: Vec<GraphOutputSinkTracePayload>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GraphOutputSinkTracePayload {
    pub wired: bool,
    pub weighted_sum: f32,
    pub applied: bool,
    pub applied_value: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct GraphPassTracePayload {
    pub pass_index: u32,
    pub energy_cost: f32,
    pub energy_after: f32,
    pub node_evaluations: Vec<GraphNodeEvalTracePayload>,
    pub max_delta: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct GraphNodeEvalTracePayload {
    pub node_index: usize,
    pub kind: &'static str,
    pub weighted_inputs: Vec<f32>,
    pub weighted_sum: f32,
    pub state_before: f32,
    pub state_after: f32,
    pub output: f32,
}
