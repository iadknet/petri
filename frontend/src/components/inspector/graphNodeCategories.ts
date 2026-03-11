import type { ComputeNodeKind } from "../../types/genome.ts";
import { getKindName } from "./graphNodeFormatters.ts";

export type NodeCategory =
	| "input"
	| "arithmetic"
	| "activation"
	| "logic"
	| "stateful"
	| "constant"
	| "output_value"
	| "output_action"
	| "output_gate";

const STATEFUL = new Set(["DecayIntegrator", "Momentum", "Oscillator", "AdaptiveGain"]);
const ACTIVATION = new Set(["Sigmoid", "Tanh", "Relu", "Clamp01", "Threshold"]);
const LOGIC = new Set(["GreaterThan", "Select"]);

export function categorizeComputeNode(kind: ComputeNodeKind): NodeCategory {
	const name = getKindName(kind);
	if (name === "Constant") return "constant";
	if (STATEFUL.has(name)) return "stateful";
	if (ACTIVATION.has(name)) return "activation";
	if (LOGIC.has(name)) return "logic";
	return "arithmetic";
}
