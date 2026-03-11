//! HTTP sampler response DTOs owned by the transport layer.

use serde::Serialize;

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
    pub final_actions: Vec<WorldActionPayload>,
    pub termination_reason: TerminationReasonPayload,
    pub priority_bid: f32,
}

#[derive(Debug, Clone, Serialize)]
pub enum TerminationReasonPayload {
    ActionEmitted,
    EnergyExhausted,
    MaxHopsReached,
    NoTargets,
    MissingNode,
}

#[derive(Debug, Clone, Serialize)]
pub struct StaticInputsSnapshotPayload {
    pub food_here: f32,
    pub neighbor_food: [f32; 8],
    pub neighbor_barrier: [f32; 8],
    pub neighbor_occupied: [f32; 8],
    pub generation: f32,
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
    pub node_id: u64,
    pub input_refs: Vec<InputReferencePayload>,
    pub upstream_slots: [f32; 12],
    pub energy_before: f32,
    pub energy_after: f32,
    pub output_slots: [f32; 12],
    pub route: RouteDecisionPayload,
    pub backend_trace: BackendTracePayload,
}

#[derive(Debug, Clone, Serialize)]
pub struct RouteDecisionPayload {
    pub kind: RouteKindPayload,
    pub raw_value: f32,
    pub resolved_target_index: usize,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteKindPayload {
    VmWrap,
    CgpNormalized,
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
    pub final_payload: [f32; 12],
    pub final_meta: [f32; 8],
    pub final_route_value: f32,
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
    pub action_slots: Vec<GraphActionSlotTracePayload>,
    pub execute_gate: GraphExecuteGateTracePayload,
}

#[derive(Debug, Clone, Serialize)]
pub struct GraphOutputSinkTracePayload {
    pub wired: bool,
    pub weighted_sum: f32,
    pub applied: bool,
    pub applied_value: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct GraphActionSlotTracePayload {
    pub wired: bool,
    pub gate_weighted_sum: f32,
    pub fired: bool,
    pub param_values: [f32; 2],
    pub queue_len_before: usize,
    pub queue_len_after: usize,
    pub emitted_action: Option<WorldActionPayload>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GraphExecuteGateTracePayload {
    pub wired: bool,
    pub weighted_sum: f32,
    pub queue_non_empty: bool,
    pub fired: bool,
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
