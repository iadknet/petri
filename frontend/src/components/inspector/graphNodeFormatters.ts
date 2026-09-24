import {
	ACTION_PARAM_FIELDS,
	type ActionParamField,
	type ComputeNodeKind,
	type GraphSource,
	type InputReference,
	type OutputSinkKind,
	type VoteSink,
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

/** The vote sink at a catalog index; null at or above the catalog count. */
export function voteSinkFromIndex(index: number): VoteSink | null {
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

/** Label the vote sink at a catalog index; out of range reads as invalid. */
export function voteSinkLabel(index: number): string {
	const sink = voteSinkFromIndex(index);
	return sink === null ? `invalid(${index})` : formatVoteSink(sink);
}

/** Format one vote sink of the catalog. */
export function formatVoteSink(sink: VoteSink): string {
	if (typeof sink === "string") return sink;
	if ("Move" in sink) return `Move[${sink.Move}]`;
	if ("Reproduce" in sink) return `Reproduce[${sink.Reproduce}]`;
	if ("StealEnergy" in sink) return `StealEnergy[${sink.StealEnergy}]`;
	return "?";
}

/** Short label of each action-parameter field. */
const ACTION_PARAM_FIELD_LABELS: Readonly<Record<ActionParamField, string>> = {
	EatFoodType: "Eat.food",
	ReproduceTransferFraction: "Reproduce.frac",
	StealEnergyAmount: "StealEnergy.amt",
};

/** Target of a VM `WriteActionParam` at `field_idx`: `param <field>`, or
 * `param[i]` for an index outside the catalog, which the VM ignores. */
export function actionParamTargetLabel(fieldIdx: number): string {
	const field = ACTION_PARAM_FIELDS[fieldIdx];
	return field ? `param ${ACTION_PARAM_FIELD_LABELS[field]}` : `param[${fieldIdx}]`;
}

/** Format an OutputSinkKind as a human-readable label. */
export function formatOutputSinkKind(kind: OutputSinkKind): string {
	if ("RouterGate" in kind) return `Route[${kind.RouterGate}]`;
	if ("CustomOutput" in kind) return `Payload[${kind.CustomOutput}]`;
	if ("WriteSlot" in kind) return `Write Mem[${kind.WriteSlot}]`;
	if ("ClearSlot" in kind) return `Clear Mem[${kind.ClearSlot}]`;
	if ("ActionVote" in kind) return `Vote ${formatVoteSink(kind.ActionVote)}`;
	if ("ActionParam" in kind) return `Param ${ACTION_PARAM_FIELD_LABELS[kind.ActionParam]}`;
	return "?";
}

/** Human-readable subtitle for an OutputSinkKind. */
export function outputSinkSubtitle(kind: OutputSinkKind): string | undefined {
	if ("CustomOutput" in kind) return "downstream slot value";
	if ("RouterGate" in kind) return "route score";
	if ("WriteSlot" in kind) return "shared memory";
	if ("ClearSlot" in kind) return "shared memory";
	if ("ActionVote" in kind) return "action vote";
	if ("ActionParam" in kind) return "action parameter";
	return undefined;
}
