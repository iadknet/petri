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
	if (typeof kind === "string") return "Router";
	if ("CustomOutput" in kind) return `Out(${kind.CustomOutput})`;
	if ("WriteSlot" in kind) return `Write[${kind.WriteSlot}]`;
	if ("ClearSlot" in kind) return `Clear[${kind.ClearSlot}]`;
	return "?";
}

/** Format an ActionSlotBehavior as a human-readable label. */
export function formatActionSlotBehavior(behavior: ActionSlotBehavior): string {
	if (typeof behavior === "string") return behavior;
	if ("Emit" in behavior) return behavior.Emit;
	return "?";
}
