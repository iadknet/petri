import type {
	ActionSlotBehavior,
	ComputeNodeKind,
	GraphSource,
	InputReference,
	OutputSinkKind,
	VoteSink,
} from "../../types/genome.ts";
import { formatInputRefWithSubIndex } from "./inputRefUtils.ts";

export function getKindName(kind: ComputeNodeKind): string {
	if (typeof kind === "string") return kind;
	const [name] = Object.entries(kind)[0] ?? ["?"];
	return name;
}

/** Produce a human-readable label for a compute node kind. */
export function getKindLabel(kind: ComputeNodeKind, _inputRefs: InputReference[]): string {
	if (typeof kind === "string") {
		switch (kind) {
			case "WeightedSum":
				return "\u03A3 Weight";
			case "Clamp01":
				return "Clamp";
			case "GreaterThan":
				return "GT";
			default:
				return kind;
		}
	}
	const [name, value] = Object.entries(kind)[0] ?? ["?", ""];
	switch (name) {
		case "Constant":
			return `Const(${(value as number).toFixed(2)})`;
		case "Threshold":
			return `Thresh(${(value as number).toFixed(2)})`;
		case "DecayIntegrator":
			return `Decay(${(value as number).toFixed(2)})`;
		case "Momentum":
			return `Momentum(${(value as number).toFixed(2)})`;
		case "Oscillator":
			return `Osc(${(value as number).toFixed(2)})`;
		default:
			if (typeof value === "number") {
				return `${name}(${Number.isInteger(value) ? value : (value as number).toFixed(2)})`;
			}
			return name;
	}
}

/** Format a GraphSource as a human-readable label. */
export function formatGraphSource(source: GraphSource, inputRefs: InputReference[]): string {
	if ("ComputeNode" in source) return `CN${source.ComputeNode}`;
	if ("SharedMemory" in source) {
		const { slot, previous } = source.SharedMemory;
		return previous ? `Prev s[${slot}]` : `Read s[${slot}]`;
	}
	if ("InputLeaf" in source) {
		const { ref_idx, sub_idx } = source.InputLeaf;
		if (ref_idx === 0xffff) return sub_idx > 0 ? `Dead[${sub_idx}]` : "Dead";
		const ref = inputRefs[ref_idx];
		return ref ? formatInputRefWithSubIndex(ref, sub_idx) : `In(${ref_idx})`;
	}
	return "?";
}

/** Directions each directed vote sink covers — the `Direction::ALL` index range. */
const VOTE_DIRECTION_COUNT = 8;

/** The vote sink at a catalog index (T19.F03); null at or above the catalog count. */
function voteSinkFromIndex(index: number): VoteSink | null {
	if (index === 0) return "Eat";
	if (index < 1 + VOTE_DIRECTION_COUNT) return { Move: index - 1 };
	if (index < 1 + 2 * VOTE_DIRECTION_COUNT) {
		return { Reproduce: index - 1 - VOTE_DIRECTION_COUNT };
	}
	if (index < 1 + 3 * VOTE_DIRECTION_COUNT) {
		return { StealEnergy: index - 1 - 2 * VOTE_DIRECTION_COUNT };
	}
	if (index === 1 + 3 * VOTE_DIRECTION_COUNT) return "Terminate";
	if (index === 2 + 3 * VOTE_DIRECTION_COUNT) return "Decide";
	return null;
}

/** Label the vote sink at a catalog index (T19.F03); out of range reads as invalid. */
export function voteSinkLabel(index: number): string {
	const sink = voteSinkFromIndex(index);
	return sink === null ? `invalid(${index})` : formatVoteSink(sink);
}

/** Format one vote sink of the T19.F03 catalog. */
export function formatVoteSink(sink: VoteSink): string {
	if (typeof sink === "string") return sink;
	if ("Move" in sink) return `Move[${sink.Move}]`;
	if ("Reproduce" in sink) return `Reproduce[${sink.Reproduce}]`;
	if ("StealEnergy" in sink) return `StealEnergy[${sink.StealEnergy}]`;
	return "?";
}

/** Format an OutputSinkKind as a human-readable label. */
export function formatOutputSinkKind(kind: OutputSinkKind): string {
	if ("RouterGate" in kind) return `Route[${kind.RouterGate}]`;
	if ("CustomOutput" in kind) return `Payload[${kind.CustomOutput}]`;
	if ("WriteSlot" in kind) return `Write Mem[${kind.WriteSlot}]`;
	if ("ClearSlot" in kind) return `Clear Mem[${kind.ClearSlot}]`;
	if ("ActionVote" in kind) return `Vote ${formatVoteSink(kind.ActionVote)}`;
	if ("ActionParam" in kind) {
		const [voteKind, slot] = kind.ActionParam;
		return `Param ${voteKind}[${slot}]`;
	}
	return "?";
}

/** Human-readable subtitle for an OutputSinkKind. */
export function outputSinkSubtitle(kind: OutputSinkKind): string | undefined {
	if ("CustomOutput" in kind) return "downstream slot value";
	if ("RouterGate" in kind) return "route score";
	if ("WriteSlot" in kind) return "shared memory";
	if ("ClearSlot" in kind) return "shared memory";
	if ("ActionVote" in kind) return "action vote (not yet read)";
	if ("ActionParam" in kind) return "action parameter (not yet read)";
	return undefined;
}

/** Format an ActionSlotBehavior as a human-readable label. */
export function formatActionSlotBehavior(behavior: ActionSlotBehavior): string {
	if (typeof behavior === "string") return behavior;
	if ("Emit" in behavior) return behavior.Emit;
	return "?";
}

/** Human-readable subtitle explaining what an action slot's inputs control. */
export function actionSlotSubtitle(behavior: ActionSlotBehavior): string {
	if (typeof behavior === "string") {
		return behavior === "Pop" ? "removes last queued action" : behavior;
	}
	if ("Emit" in behavior) {
		switch (behavior.Emit) {
			case "Move":
				return "gate · direction";
			case "Eat":
				return "gate · food type";
			case "Reproduce":
				return "gate · direction · fraction";
			case "StealEnergy":
				return "gate · direction · amount";
			case "NoOp":
				return "gate only";
			default:
				return "gate";
		}
	}
	return "";
}
