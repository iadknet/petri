import type { InputReference, VmInstruction, VoteKind } from "./genome.ts";

// ── Execution Sampler types ──────────────────────────────────────────────────

export interface StaticInputsSnapshot {
	food_here: number;
	neighbor_food: number[];
	neighbor_barrier: number[];
	neighbor_occupied: number[];
	age_ticks: number;
}

export interface PerceptionDebugSnapshot {
	area_food: number[];
	area_barrier: number[];
	area_occupancy: number[];
	nearby_core: number[];
	nearby_vitals: number[];
	nearby_identity: number[];
}

export interface VmStepTrace {
	pc: number;
	instruction: VmInstruction;
	energy_cost: number;
	energy_after: number;
	register_changes: [number, number][];
}

export interface SlotWriteTrace {
	slot_idx: number;
	old_value: number;
	new_value: number;
}

export interface VmTrace {
	register_count: number;
	constants: number[];
	steps: VmStepTrace[];
	final_registers: number[];
	final_payload: number[];
	final_meta: number[];
	slot_writes: SlotWriteTrace[];
}

export interface GraphNodeEvalTrace {
	node_index: number;
	kind: string;
	weighted_inputs: number[];
	weighted_sum: number;
	state_before: number;
	state_after: number;
	output: number;
}

export interface GraphPassTrace {
	pass_index: number;
	energy_cost: number;
	energy_after: number;
	node_evaluations: GraphNodeEvalTrace[];
	max_delta: number;
}

export interface GraphOutputSinkTrace {
	wired: boolean;
	weighted_sum: number;
	applied: boolean;
	applied_value: number;
}

export interface GraphActionSlotTrace {
	wired: boolean;
	gate_weighted_sum: number;
	fired: boolean;
	param_values: [number, number];
	queue_len_before: number;
	queue_len_after: number;
	emitted_action: WorldAction | null;
}

export interface GraphExecuteGateTrace {
	wired: boolean;
	weighted_sum: number;
	queue_non_empty: boolean;
	fired: boolean;
}

export interface GraphTrace {
	temporal_committed: boolean;
	passes: GraphPassTrace[];
	converged: boolean;
	stable_passes_count: number;
	final_outputs: number[];
	output_sinks: GraphOutputSinkTrace[];
	action_slots: GraphActionSlotTrace[];
	execute_gate: GraphExecuteGateTrace;
}

export type BackendTrace = { Vm: VmTrace } | { Graph: GraphTrace };

export interface TraceGateScore {
	slot: number;
	target_id: number;
	gate_bias: number;
	runtime_score: number;
	effective_score: number;
}

export interface TraceRouteDecision {
	gate_scores: TraceGateScore[];
	selected_target_idx: number;
	selected_target_id: number;
}

export interface MeshHopTrace {
	hop_index: number;
	node_id: number;
	input_refs: InputReference[];
	upstream_slots: number[];
	energy_before: number;
	energy_after: number;
	output_slots: number[];
	route: TraceRouteDecision | null;
	/** The vote contribution this hop committed (T19.F03); zeros when it did not. */
	vote_contribution: readonly number[];
	backend_trace: BackendTrace;
}

export type TerminationReason =
	| "ActionEmitted"
	| "EnergyExhausted"
	| "MaxHopsReached"
	| "NoTargets"
	| "MissingNode";

export type WorldAction =
	| "NoOp"
	| "Eat"
	| { Eat: { type_idx: number } }
	| { Move: string }
	| { Reproduce: { direction: string; energy_transfer_fraction: number } }
	| { StealEnergy: { direction: string; amount: number } };

/** Vote sinks in the catalog: Eat, 8 each of Move/Reproduce/StealEnergy, Terminate, Decide. */
export const VOTE_SINK_COUNT = 27;
/** Action kinds carrying a vote commit counter, in `commit_counts` order. */
export const VOTE_KINDS: readonly VoteKind[] = Object.freeze([
	"Eat",
	"Move",
	"Reproduce",
	"StealEnergy",
]);
/** A vote vector with no contributions. */
export const ZERO_VOTES: readonly number[] = Object.freeze(
	Array.from({ length: VOTE_SINK_COUNT }, () => 0),
);

export interface TickTrace {
	tick_number: number;
	energy_before: number;
	energy_after: number;
	static_inputs: StaticInputsSnapshot;
	debug_perception: PerceptionDebugSnapshot | null;
	hops: MeshHopTrace[];
	final_actions: WorldAction[];
	termination_reason: TerminationReason;
	priority_bid: number;
	/** Inert vote surface (T19.F03), in catalog index order. Nothing reads it. */
	votes: readonly number[];
	/** Per-kind vote commit counters (T19.F03). */
	commit_counts: readonly number[];
}

export interface ExecutionSample {
	creature_id: number;
	ticks: TickTrace[];
}

export type SampleResponse =
	| { protocol_version: string; status: "idle" }
	| {
			protocol_version: string;
			status: "recording";
			ticks_completed: number;
			ticks_remaining: number;
	  }
	| { protocol_version: string; status: "complete"; sample: ExecutionSample };

export interface StartSampleResponse {
	protocol_version: string;
	status: "recording";
	ticks_requested: number;
	include_perception_debug: boolean;
}
