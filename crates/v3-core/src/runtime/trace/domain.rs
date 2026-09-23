//! Passive trace-domain data structures for the Execution Sampler.

use crate::contracts::{InputReference, NodeId, WorldAction};
use crate::creature::genome::cgp::ComputeNodeKind;
use crate::creature::genome::vote::{VoteVector, VOTE_KIND_COUNT};
use crate::creature::genome::OUTCOME_CHANNEL_COUNT;
use crate::runtime::OUTPUT_SLOT_COUNT;
use crate::sensors::perception::PerceptionSnapshot;
use crate::sensors::static_inputs::StaticInputs;
use serde::Serialize;

/// Per-target gate score captured in a route decision trace.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TraceGateScore {
    pub slot: u8,
    pub target_id: NodeId,
    pub gate_bias: f32,
    pub runtime_score: f32,
    pub effective_score: f32,
}

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
    /// Optional extended perception debug snapshot.
    /// `null` when the sample request did not set `include_perception_debug = true`.
    pub debug_perception: Option<PerceptionDebugSnapshot>,
    pub hops: Vec<MeshHopTrace>,
    /// One record per pass, in pass order (T19.F04).
    pub passes: Vec<MeshPassTrace>,
    pub final_actions: Vec<WorldAction>,
    pub termination_reason: TerminationReason,
    /// Energy bid for turn-order priority (0.0 if none).
    pub priority_bid: f32,
    /// Final per-kind bars: commits of each kind this tick, in `VoteKind`
    /// index order.
    pub commit_counts: [u32; VOTE_KIND_COUNT],
}

/// Why a tick's pass loop ended (T19.F04).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TerminationReason {
    /// A pass ended with no positive effective vote.
    NoDecision,
    /// `Terminate` met the best effective vote against a non-empty queue.
    TerminateVoted,
    /// A commit filled the queue (`max_actions_per_turn`).
    ActionCapReached,
    /// Energy ran out in a dispatch, the hop ramp, or an all-in bid; the
    /// committed queue is kept.
    EnergyExhausted,
}

/// Why one pass of the mesh ended (T19.F04).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum PassEndReason {
    /// A committed dispatch left `Decide` positive while some kind's
    /// effective vote was positive.
    Decided,
    /// The pass reached `max_mesh_hops` routed hops.
    PassCapReached,
    /// The last node selected no target.
    NoTargets,
    /// The entry node or the routed target is absent from the genome.
    MissingNode,
    /// A dispatch or the hop ramp exhausted the creature.
    EnergyExhausted,
}

/// One pass of a tick's mesh evaluation (T19.F04).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MeshPassTrace {
    pub pass_index: u32,
    pub end_reason: PassEndReason,
    /// The pass's final vote vector, in `VoteSink` index order.
    pub votes: VoteVector,
    /// Effective vote per kind against the bars the pass started with.
    pub effective_votes: [f32; VOTE_KIND_COUNT],
    /// The action the pass end committed, if any.
    pub committed: Option<WorldAction>,
    /// Dispatches this pass.
    pub hops: u32,
}

/// Serializable mirror of [`StaticInputs`].
#[derive(Debug, Clone, Serialize)]
pub struct StaticInputsSnapshot {
    pub food_here: f32,
    pub neighbor_food: [f32; 8],
    pub neighbor_barrier: [f32; 8],
    pub neighbor_occupied: [f32; 8],
    pub age_ticks: f32,
    /// The scaled previous-outcome channels the genome reads
    /// (`PreviousOutcome`, T19.F06).
    pub previous_outcome: [f32; OUTCOME_CHANNEL_COUNT],
}

impl From<&StaticInputs> for StaticInputsSnapshot {
    fn from(si: &StaticInputs) -> Self {
        Self {
            food_here: si.food_here,
            neighbor_food: si.neighbor_food,
            neighbor_barrier: si.neighbor_barrier,
            neighbor_occupied: si.neighbor_occupied,
            age_ticks: si.age_ticks,
            previous_outcome: si.previous_outcome,
        }
    }
}

/// Serializable mirror of [`PerceptionSnapshot`] for trace debug output.
#[derive(Debug, Clone, Serialize)]
pub struct PerceptionDebugSnapshot {
    pub area_food: [f32; 7],
    pub area_barrier: [f32; 7],
    pub area_occupancy: [f32; 7],
    pub nearby_core: [f32; 16],
    pub nearby_vitals: [f32; 8],
    pub nearby_identity: [f32; 12],
}

impl From<&PerceptionSnapshot> for PerceptionDebugSnapshot {
    fn from(p: &PerceptionSnapshot) -> Self {
        Self {
            area_food: p.area_food,
            area_barrier: p.area_barrier,
            area_occupancy: p.area_occupancy,
            nearby_core: p.nearby_core,
            nearby_vitals: p.nearby_vitals,
            nearby_identity: p.nearby_identity,
        }
    }
}

/// Route decision captured per hop, containing per-target gate scores.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TraceRouteDecision {
    pub gate_scores: Vec<TraceGateScore>,
    pub selected_target_idx: usize,
    pub selected_target_id: NodeId,
}

/// Trace data for a single hop in the mesh chain.
#[derive(Debug, Clone, Serialize)]
pub struct MeshHopTrace {
    /// Tick-wide dispatch index.
    pub hop_index: usize,
    /// The pass this hop belongs to (T19.F04).
    pub pass_index: u32,
    pub node_id: NodeId,
    pub input_refs: Vec<InputReference>,
    pub upstream_slots: [f32; OUTPUT_SLOT_COUNT],
    pub energy_before: f32,
    pub energy_after: f32,
    pub output_slots: [f32; OUTPUT_SLOT_COUNT],
    pub route: Option<TraceRouteDecision>,
    /// The vote contribution this hop committed (T19.F03); zeros when the
    /// dispatch ended exhausted and committed nothing.
    pub vote_contribution: VoteVector,
    /// The decision state this dispatch's inputs resolved against (T19.F06).
    pub decision_inputs: DecisionInputs,
    pub backend_trace: BackendTrace,
}

/// The decision-state values a dispatch's resolution context supplied
/// (T19.F06), each exactly as `resolve_input` returns it: what the node
/// could read, not proof that it read them.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DecisionInputs {
    /// `ActionVotes`: the pass's committed votes, in `VoteSink` order.
    pub action_votes: VoteVector,
    /// `PreviousPassVotes`: the vote vector at the previous pass's end.
    pub previous_pass_votes: VoteVector,
    /// `CommitCounts`: the per-kind bars, in `VoteKind` order.
    pub commit_counts: [f32; VOTE_KIND_COUNT],
    /// `HopsThisTick`: dispatches this tick, this one included.
    pub hops_this_tick: f32,
}

/// Discriminated union for backend-specific trace data.
#[derive(Debug, Clone, Serialize)]
pub enum BackendTrace {
    Vm(VmTrace),
    Graph(GraphTrace),
}

/// Trace of a VM node's execution.
#[derive(Debug, Clone, Serialize)]
pub struct VmTrace {
    pub register_count: u8,
    pub constants: Vec<f32>,
    pub steps: Vec<VmStepTrace>,
    pub final_registers: Vec<f32>,
    pub final_payload: [f32; OUTPUT_SLOT_COUNT],
    pub slot_writes: Vec<SlotWrite>,
}

/// Trace of a single VM instruction execution.
#[derive(Debug, Clone, Serialize)]
pub struct VmStepTrace {
    pub pc: usize,
    pub instruction: crate::creature::genome::VmInstruction,
    pub energy_cost: f32,
    pub energy_after: f32,
    pub register_changes: Vec<(u8, f32)>,
}

/// Record of a single shared-memory slot write operation.
#[derive(Debug, Clone, Serialize)]
pub struct SlotWrite {
    pub slot_idx: u8,
    pub old_value: f32,
    pub new_value: f32,
}

/// Trace of one ordered graph visit.
#[derive(Debug, Clone, Serialize)]
pub struct GraphTrace {
    pub passes: Vec<GraphPassTrace>,
    /// Whether candidate operator state and outputs were committed.
    pub temporal_committed: bool,
    pub final_outputs: Vec<f32>,
    pub output_sinks: Vec<GraphOutputSinkTrace>,
}

/// Effect-phase trace for a single output sink (indexed 1:1 with `output_sinks`).
#[derive(Debug, Clone, Serialize)]
pub struct GraphOutputSinkTrace {
    pub wired: bool,
    pub weighted_sum: f32,
    pub applied: bool,
    pub applied_value: f32,
}

/// One entered evaluation; node records are candidates when temporal_committed is false.
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

/// Map a [`ComputeNodeKind`] variant to a static string label.
#[inline]
pub fn kind_label(kind: &ComputeNodeKind) -> &'static str {
    match kind {
        ComputeNodeKind::Constant(_) => "Constant",
        ComputeNodeKind::Add => "Add",
        ComputeNodeKind::Multiply => "Multiply",
        ComputeNodeKind::Negate => "Negate",
        ComputeNodeKind::Abs => "Abs",
        ComputeNodeKind::Min => "Min",
        ComputeNodeKind::Max => "Max",
        ComputeNodeKind::Threshold(_) => "Threshold",
        ComputeNodeKind::GreaterThan => "GreaterThan",
        ComputeNodeKind::Sigmoid => "Sigmoid",
        ComputeNodeKind::Tanh => "Tanh",
        ComputeNodeKind::Relu => "Relu",
        ComputeNodeKind::Select => "Select",
        ComputeNodeKind::Clamp01 => "Clamp01",
        ComputeNodeKind::WeightedSum => "WeightedSum",
        ComputeNodeKind::DecayIntegrator(_) => "DecayIntegrator",
        ComputeNodeKind::Momentum(_) => "Momentum",
        ComputeNodeKind::Oscillator(_) => "Oscillator",
        ComputeNodeKind::AdaptiveGain => "AdaptiveGain",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_inputs_snapshot_from_static_inputs() {
        let si = StaticInputs {
            food_here: 0.75,
            neighbor_food: [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8],
            neighbor_barrier: [1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0],
            neighbor_occupied: [0.0; 8],
            max_energy: 200.0,
            age_ticks: 0.084,
            previous_outcome: [0.25, 0.5, 0.0, 1.0],
        };
        let snapshot = StaticInputsSnapshot::from(&si);
        assert_eq!(snapshot.food_here, 0.75);
        assert_eq!(snapshot.neighbor_food, si.neighbor_food);
        assert_eq!(snapshot.neighbor_barrier, si.neighbor_barrier);
        assert_eq!(snapshot.age_ticks, 0.084);
        assert_eq!(snapshot.previous_outcome, si.previous_outcome);
    }

    #[test]
    fn kind_label_covers_all_variants() {
        use crate::creature::genome::cgp::ComputeNodeKind;

        assert_eq!(kind_label(&ComputeNodeKind::Constant(1.0)), "Constant");
        assert_eq!(kind_label(&ComputeNodeKind::Add), "Add");
        assert_eq!(kind_label(&ComputeNodeKind::Multiply), "Multiply");
        assert_eq!(kind_label(&ComputeNodeKind::Negate), "Negate");
        assert_eq!(kind_label(&ComputeNodeKind::Abs), "Abs");
        assert_eq!(kind_label(&ComputeNodeKind::Min), "Min");
        assert_eq!(kind_label(&ComputeNodeKind::Max), "Max");
        assert_eq!(kind_label(&ComputeNodeKind::Threshold(0.5)), "Threshold");
        assert_eq!(kind_label(&ComputeNodeKind::GreaterThan), "GreaterThan");
        assert_eq!(kind_label(&ComputeNodeKind::Sigmoid), "Sigmoid");
        assert_eq!(kind_label(&ComputeNodeKind::Tanh), "Tanh");
        assert_eq!(kind_label(&ComputeNodeKind::Relu), "Relu");
        assert_eq!(kind_label(&ComputeNodeKind::Select), "Select");
        assert_eq!(kind_label(&ComputeNodeKind::Clamp01), "Clamp01");
        assert_eq!(kind_label(&ComputeNodeKind::WeightedSum), "WeightedSum");
        assert_eq!(
            kind_label(&ComputeNodeKind::DecayIntegrator(0.5)),
            "DecayIntegrator"
        );
        assert_eq!(kind_label(&ComputeNodeKind::Momentum(0.5)), "Momentum");
        assert_eq!(kind_label(&ComputeNodeKind::Oscillator(1.0)), "Oscillator");
        assert_eq!(kind_label(&ComputeNodeKind::AdaptiveGain), "AdaptiveGain");
    }

    #[test]
    fn perception_debug_snapshot_from_perception_snapshot() {
        let mut p = PerceptionSnapshot::zero();
        p.area_food[0] = 0.5;
        p.nearby_core[3] = 0.8;
        p.nearby_identity[11] = 0.9;
        let debug = PerceptionDebugSnapshot::from(&p);
        assert!((debug.area_food[0] - 0.5).abs() < f32::EPSILON);
        assert_eq!(debug.area_barrier, [0.0; 7]);
        assert_eq!(debug.area_occupancy, [0.0; 7]);
        assert!((debug.nearby_core[3] - 0.8).abs() < f32::EPSILON);
        assert_eq!(debug.nearby_vitals, [0.0; 8]);
        assert!((debug.nearby_identity[11] - 0.9).abs() < f32::EPSILON);
    }

    #[test]
    fn trace_route_decision_with_gate_scores() {
        let decision = TraceRouteDecision {
            gate_scores: vec![
                TraceGateScore {
                    slot: 0,
                    target_id: NodeId::new(1),
                    gate_bias: 0.0,
                    runtime_score: -1.0,
                    effective_score: -1.0,
                },
                TraceGateScore {
                    slot: 1,
                    target_id: NodeId::new(2),
                    gate_bias: 0.5,
                    runtime_score: 0.0,
                    effective_score: 0.5,
                },
            ],
            selected_target_idx: 1,
            selected_target_id: NodeId::new(2),
        };
        assert_eq!(decision.gate_scores.len(), 2);
        assert_eq!(decision.selected_target_idx, 1);
        assert_eq!(decision.selected_target_id, NodeId::new(2));
        assert!((decision.gate_scores[0].effective_score - (-1.0)).abs() < 1e-6);
        assert!((decision.gate_scores[1].effective_score - 0.5).abs() < 1e-6);
    }
}
