//! HTTP sampler response DTOs owned by the transport layer.

use serde::Serialize;
use v3_core::contracts::Direction;
use v3_core::creature::genome::vote::{VOTE_KIND_COUNT, VOTE_SINK_COUNT};
use v3_core::creature::genome::OUTCOME_CHANNEL_COUNT;
use v3_core::runtime::OUTPUT_SLOT_COUNT;

use crate::transport::genome_wire::{InputRef, VmInstruction, VoteSink, WorldAction};

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct ExecutionSamplePayload {
    pub creature_id: u64,
    pub ticks: Vec<TickTracePayload>,
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct TickTracePayload {
    pub tick_number: u64,
    /// The heading the tick started with (T11.F30).
    pub heading: Direction,
    pub energy_before: f32,
    pub energy_after: f32,
    pub static_inputs: StaticInputsSnapshotPayload,
    pub debug_perception: Option<PerceptionDebugSnapshotPayload>,
    pub hops: Vec<MeshHopTracePayload>,
    /// One record per pass of the tick (T19.F04).
    pub passes: Vec<MeshPassTracePayload>,
    pub final_actions: Vec<WorldAction>,
    pub termination_reason: TerminationReasonPayload,
    pub priority_bid: f32,
    /// Final per-kind bars: commits of each kind this tick (T19.F04).
    pub commit_counts: [u32; VOTE_KIND_COUNT],
}

/// Why the tick's pass loop ended (T19.F04).
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub enum TerminationReasonPayload {
    NoDecision,
    TerminateVoted,
    ActionCapReached,
    EnergyExhausted,
}

/// Why one pass ended (T19.F04).
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub enum PassEndReasonPayload {
    Decided,
    PassCapReached,
    NoTargets,
    MissingNode,
    EnergyExhausted,
}

/// One pass of a tick (T19.F04).
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct MeshPassTracePayload {
    pub pass_index: u32,
    pub end_reason: PassEndReasonPayload,
    /// The pass's final vote vector, in `VoteSink` index order.
    #[serde(serialize_with = "v3_core::runtime::trace::domain::serialize_array")]
    #[cfg_attr(test, ts(as = "Vec<f32>"))]
    pub votes: [f32; VOTE_SINK_COUNT],
    /// Each kind's best sink at the pass end, in `VoteKind` index order: the
    /// direction the heading tie chose for a directed kind (T11.F30).
    pub best: Vec<VoteSink>,
    /// The bars the pass started with, in `VoteKind` index order.
    pub bars: [u32; VOTE_KIND_COUNT],
    /// Effective vote per kind against the bars the pass started with.
    pub effective_votes: [f32; VOTE_KIND_COUNT],
    pub committed: Option<WorldAction>,
    pub hops: u32,
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct StaticInputsSnapshotPayload {
    pub food_here: f32,
    pub neighbor_food: [f32; 8],
    pub neighbor_barrier: [f32; 8],
    pub neighbor_occupied: [f32; 8],
    pub age_ticks: f32,
    /// The scaled previous-outcome channels the genome reads (T19.F06).
    pub previous_outcome: [f32; OUTCOME_CHANNEL_COUNT],
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct PerceptionDebugSnapshotPayload {
    pub area_food: [f32; 7],
    pub area_barrier: [f32; 7],
    pub area_occupancy: [f32; 7],
    /// The `NearbyCreature` family, 4 slots × 10 fields (T27.F01).
    #[serde(serialize_with = "v3_core::runtime::trace::domain::serialize_array")]
    #[cfg_attr(test, ts(as = "Vec<f32>"))]
    pub nearby: [f32; v3_core::contracts::nearby_idx::WIDTH],
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct MeshHopTracePayload {
    pub hop_index: usize,
    /// The pass this hop belongs to (T19.F04).
    pub pass_index: u32,
    pub node_id: u64,
    pub input_refs: Vec<InputRef>,
    pub upstream_slots: [f32; OUTPUT_SLOT_COUNT],
    pub energy_before: f32,
    pub energy_after: f32,
    pub output_slots: [f32; OUTPUT_SLOT_COUNT],
    pub route: Option<RouteDecisionPayload>,
    /// The vote contribution this hop committed (T19.F03).
    #[serde(serialize_with = "v3_core::runtime::trace::domain::serialize_array")]
    #[cfg_attr(test, ts(as = "Vec<f32>"))]
    pub vote_contribution: [f32; VOTE_SINK_COUNT],
    /// The decision-state values this dispatch's inputs resolved against (T19.F06).
    pub decision_inputs: DecisionInputsPayload,
    pub backend_trace: BackendTracePayload,
}

/// The decision-state input values available to one dispatch (T19.F06).
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct DecisionInputsPayload {
    #[serde(serialize_with = "v3_core::runtime::trace::domain::serialize_array")]
    #[cfg_attr(test, ts(as = "Vec<f32>"))]
    pub action_votes: [f32; VOTE_SINK_COUNT],
    #[serde(serialize_with = "v3_core::runtime::trace::domain::serialize_array")]
    #[cfg_attr(test, ts(as = "Vec<f32>"))]
    pub previous_pass_votes: [f32; VOTE_SINK_COUNT],
    pub commit_counts: [f32; VOTE_KIND_COUNT],
    pub hops_this_tick: f32,
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct RouteDecisionPayload {
    pub gate_scores: Vec<GateScorePayload>,
    pub selected_target_idx: usize,
    pub selected_target_id: u32,
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct GateScorePayload {
    pub slot: u8,
    pub target_id: u32,
    pub gate_bias: f32,
    pub runtime_score: f32,
    pub effective_score: f32,
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub enum BackendTracePayload {
    Vm(VmTracePayload),
    Graph(GraphTracePayload),
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct VmTracePayload {
    pub register_count: u8,
    pub constants: Vec<f32>,
    pub steps: Vec<VmStepTracePayload>,
    pub final_registers: Vec<f32>,
    pub final_payload: [f32; OUTPUT_SLOT_COUNT],
    pub slot_writes: Vec<SlotWritePayload>,
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct VmStepTracePayload {
    pub pc: usize,
    pub instruction: VmInstruction,
    pub energy_cost: f32,
    pub energy_after: f32,
    pub register_changes: Vec<(u8, f32)>,
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct SlotWritePayload {
    pub slot_idx: u8,
    pub old_value: f32,
    pub new_value: f32,
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct GraphTracePayload {
    pub passes: Vec<GraphPassTracePayload>,
    pub final_outputs: Vec<f32>,
    pub output_sinks: Vec<GraphOutputSinkTracePayload>,
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct GraphOutputSinkTracePayload {
    pub wired: bool,
    /// Each edge's value times its applied weight, in sink order (T25.F06);
    /// empty for an unwired sink.
    pub weighted_inputs: Vec<f32>,
    pub weighted_sum: f32,
    pub applied: bool,
    pub applied_value: f32,
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct GraphPassTracePayload {
    pub pass_index: u32,
    pub energy_cost: f32,
    pub energy_after: f32,
    pub node_evaluations: Vec<GraphNodeEvalTracePayload>,
    pub max_delta: f32,
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct GraphNodeEvalTracePayload {
    pub node_index: usize,
    pub kind: &'static str,
    pub weighted_inputs: Vec<f32>,
    pub weighted_sum: f32,
    pub state_before: f32,
    pub state_after: f32,
    pub output: f32,
}
