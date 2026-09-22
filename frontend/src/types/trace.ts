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

export interface GraphTrace {
	temporal_committed: boolean;
	passes: GraphPassTrace[];
	converged: boolean;
	stable_passes_count: number;
	final_outputs: number[];
	output_sinks: GraphOutputSinkTrace[];
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
	/** Tick-wide hop index. */
	hop_index: number;
	/** The pass this hop belongs to. */
	pass_index: number;
	node_id: number;
	input_refs: InputReference[];
	upstream_slots: number[];
	energy_before: number;
	energy_after: number;
	output_slots: number[];
	route: TraceRouteDecision | null;
	/** The vote contribution this hop committed; zeros when it did not. */
	vote_contribution: readonly number[];
	backend_trace: BackendTrace;
}

/** Why the tick's pass loop ended. */
export type TerminationReason =
	| "NoDecision"
	| "TerminateVoted"
	| "ActionCapReached"
	| "EnergyExhausted";

/** Why one pass ended. */
export type PassEndReason =
	| "Decided"
	| "PassCapReached"
	| "NoTargets"
	| "MissingNode"
	| "EnergyExhausted";

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

/** One pass of a tick: its vote vector, the effective vote per kind against
 * the bars it started with, and the action it committed, if any. */
export interface MeshPassTrace {
	pass_index: number;
	end_reason: PassEndReason;
	votes: readonly number[];
	effective_votes: readonly number[];
	committed: WorldAction | null;
	hops: number;
}

export interface TickTrace {
	tick_number: number;
	energy_before: number;
	energy_after: number;
	static_inputs: StaticInputsSnapshot;
	debug_perception: PerceptionDebugSnapshot | null;
	hops: MeshHopTrace[];
	/** One record per pass, in order. */
	passes: MeshPassTrace[];
	final_actions: WorldAction[];
	termination_reason: TerminationReason;
	priority_bid: number;
	/** Final per-kind bars: commits of each kind this tick, in `VOTE_KINDS` order. */
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
