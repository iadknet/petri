import type {
	ActionSlotBehavior,
	ComputeNodeKind,
	GraphSource,
	InputReference,
	OutputSinkKind,
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

/** Format an OutputSinkKind as a human-readable label. */
export function formatOutputSinkKind(kind: OutputSinkKind): string {
	if ("RouterGate" in kind) return `Route[${kind.RouterGate}]`;
	if ("CustomOutput" in kind) return `Payload[${kind.CustomOutput}]`;
	if ("WriteSlot" in kind) return `Write Mem[${kind.WriteSlot}]`;
	if ("ClearSlot" in kind) return `Clear Mem[${kind.ClearSlot}]`;
	return "?";
}

/** Human-readable subtitle for an OutputSinkKind. */
export function outputSinkSubtitle(kind: OutputSinkKind): string | undefined {
	if ("CustomOutput" in kind) return "downstream slot value";
	if ("RouterGate" in kind) return "route score";
	if ("WriteSlot" in kind) return "shared memory";
	if ("ClearSlot" in kind) return "shared memory";
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
				return "gate · direction · energy";
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
