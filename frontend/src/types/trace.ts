import type { InputReference, VmInstruction, VoteSink } from "./genome.ts";
import { VOTE_KINDS, VOTE_SINK_COUNT } from "./vote-sinks.ts";

// ── Execution Sampler types ──────────────────────────────────────────────────

export interface StaticInputsSnapshot {
	food_here: number;
	neighbor_food: number[];
	neighbor_barrier: number[];
	neighbor_occupied: number[];
	age_ticks: number;
	/** The scaled previous-outcome channels the genome reads, in `OUTCOME_CHANNELS` order. */
	previous_outcome: readonly number[];
}

export interface PerceptionDebugSnapshot {
	area_food: number[];
	area_barrier: number[];
	area_occupancy: number[];
	/** `NearbyCreature`: 4 slots × 10 fields. */
	nearby: number[];
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
	/** Each edge's value times its applied weight, in sink order (T25.F06). */
	weighted_inputs: number[];
	weighted_sum: number;
	applied: boolean;
	applied_value: number;
}

export interface GraphTrace {
	temporal_committed: boolean;
	passes: GraphPassTrace[];
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
	/** The decision-state values this dispatch's inputs resolved against. */
	decision_inputs: DecisionInputs;
	backend_trace: BackendTrace;
}

/** The decision-state input values available to one dispatch, as recorded. */
export interface DecisionInputs {
	/** `ActionVotes`: the pass's committed votes, in vote sink order. */
	action_votes: readonly number[];
	/** `PreviousPassVotes`: the vote vector at the previous pass's end. */
	previous_pass_votes: readonly number[];
	/** `CommitCounts`: the per-kind bars, in `VOTE_KINDS` order. */
	commit_counts: readonly number[];
	/** `HopsThisTick`: dispatches this tick, this one included. */
	hops_this_tick: number;
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
	| { Eat: { type_idx: number; leave: number } }
	| { Move: string }
	| { Reproduce: { direction: string; energy_transfer_fraction: number } }
	| { StealEnergy: { direction: string; amount: number } };

export { VOTE_KINDS, VOTE_SINK_COUNT } from "./vote-sinks.ts";
/** `PreviousOutcome` channels, in `previous_outcome` order. */
export const OUTCOME_CHANNELS: readonly string[] = Object.freeze([
	"EnergyDelta",
	"ActionSuccess",
	"DamageDelta",
	"OffspringSuccess",
]);
/** A vote vector with no contributions. */
export const ZERO_VOTES: readonly number[] = Object.freeze(
	Array.from({ length: VOTE_SINK_COUNT }, () => 0),
);
/** A per-kind vector (commit counts, bars) with every kind at zero, in
 * `VOTE_KINDS` order. */
export const ZERO_KIND_COUNTS: readonly number[] = Object.freeze(VOTE_KINDS.map(() => 0));

/** Decision-state inputs of a dispatch before any vote, commit, or hop. */
export const ZERO_DECISION_INPUTS: DecisionInputs = Object.freeze({
	action_votes: ZERO_VOTES,
	previous_pass_votes: ZERO_VOTES,
	commit_counts: ZERO_KIND_COUNTS,
	hops_this_tick: 0,
});

/** One pass of a tick: its vote vector, the effective vote per kind against
 * the bars it started with, and the action it committed, if any. */
export interface MeshPassTrace {
	pass_index: number;
	end_reason: PassEndReason;
	votes: readonly number[];
	/** Each kind's best sink at the pass end, in `VOTE_KINDS` order: the
	 * direction the heading tie chose for a directed kind. */
	best: readonly VoteSink[];
	/** The bars the pass started with, in `VOTE_KINDS` order. */
	bars: readonly number[];
	effective_votes: readonly number[];
	committed: WorldAction | null;
	hops: number;
}

export interface TickTrace {
	tick_number: number;
	/** The heading the tick started with (a `Direction`, e.g. `"NE"`). */
	heading: string;
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
