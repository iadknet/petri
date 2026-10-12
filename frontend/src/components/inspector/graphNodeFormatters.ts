import {
	ACTION_PARAM_FIELDS,
	type ActionParamField,
	type ComputeNodeKind,
	type GraphSource,
	type InputReference,
	type OutputSinkKind,
	type VoteSink,
} from "../../types/genome.ts";
import { directionWord } from "./directions.ts";
import { describeInputRef } from "./inputRefUtils.ts";
import { formatVoteSink } from "./voteSinks.ts";

export {
	driveSinkIndex,
	formatVoteSink,
	voteSinkFromIndex,
	voteSinkIndex,
	voteSinkLabel,
} from "./voteSinks.ts";

export function getKindName(kind: ComputeNodeKind): string {
	if (typeof kind === "string") return kind;
	const [name] = Object.entries(kind)[0] ?? ["?"];
	return name;
}

/** A graph element's display name and the plain-words tooltip behind it. */
export interface Described {
	label: string;
	description: string;
}

/** Stateful kinds advance once per run of their mesh node and keep their
 * state across passes and ticks. */
const STATEFUL_NOTE = "Its state advances each time this mesh node runs and carries across ticks.";

const STRING_KINDS: Record<string, Described> = {
	Add: { label: "Add", description: "Sum of the weighted inputs (each input × its edge weight)." },
	WeightedSum: {
		label: "Σ Sum",
		description: "Sum of the weighted inputs (each input × its edge weight). Same as Add.",
	},
	Multiply: {
		label: "Multiply",
		description: "Product of the weighted inputs (1 with no inputs).",
	},
	Negate: { label: "Negate", description: "Minus the weighted sum." },
	Abs: { label: "Abs", description: "Absolute value of the weighted sum." },
	Min: { label: "Min", description: "Smallest weighted input (0 with no inputs)." },
	Max: { label: "Max", description: "Largest weighted input (0 with no inputs)." },
	Sigmoid: { label: "Sigmoid", description: "Squashes the weighted sum into 0 to 1." },
	Tanh: { label: "Tanh", description: "Squashes the weighted sum into −1 to 1." },
	Relu: { label: "Relu", description: "The weighted sum, or 0 when it is negative." },
	Clamp01: { label: "Clamp", description: "The weighted sum, clipped to 0 to 1." },
	GreaterThan: {
		label: "A > B",
		description: "1 when weighted input 0 is greater than weighted input 1, otherwise 0.",
	},
	Select: {
		label: "Select",
		description: "Switch: outputs input 1 when input 0 is at least 0.5, otherwise input 2.",
	},
	AdaptiveGain: {
		label: "AdaptiveGain",
		description: `The weighted sum times a gain (0.1 to 2) that rises with positive input and falls with negative. ${STATEFUL_NOTE}`,
	},
};

/** Describe a compute node kind, naming its parameter in the tooltip. */
export function describeComputeKind(kind: ComputeNodeKind): Described {
	if (typeof kind === "string") return STRING_KINDS[kind] ?? { label: kind, description: "" };
	const [name, raw] = Object.entries(kind)[0] ?? ["?", 0];
	const value = typeof raw === "number" ? raw : Number.NaN;
	const v = value.toFixed(2);
	switch (name) {
		case "Constant":
			return { label: `Const(${v})`, description: `Always outputs ${v}; ignores inputs.` };
		case "Threshold":
			return {
				label: `Thresh(${v})`,
				description: `1 when the weighted sum is above ${v}, otherwise 0.`,
			};
		case "DecayIntegrator":
			return {
				label: `Decay(${v})`,
				description: `Running average: each run keeps ${(1 - value).toFixed(2)} of the old value and adds ${v} × the weighted sum. ${STATEFUL_NOTE}`,
			};
		case "Momentum":
			return {
				label: `Momentum(${v})`,
				description: `Running average: each run keeps ${v} of the old value and adds ${(1 - value).toFixed(2)} × the weighted sum. ${STATEFUL_NOTE}`,
			};
		case "Oscillator":
			return {
				label: `Osc(${v})`,
				description: `Sine wave advancing ${v} cycles per run; only the fractional part matters, so 1.00 is flat. Ignores inputs. ${STATEFUL_NOTE}`,
			};
		default:
			return {
				label: Number.isNaN(value)
					? name
					: `${name}(${Number.isInteger(value) ? value : value.toFixed(2)})`,
				description: "",
			};
	}
}

/** Produce a human-readable label for a compute node kind. */
export function getKindLabel(kind: ComputeNodeKind, _inputRefs: InputReference[]): string {
	return describeComputeKind(kind).label;
}

/** Describe the source of a graph edge: an input, a memory read or a node. */
export function describeGraphSource(source: GraphSource, inputRefs: InputReference[]): Described {
	if ("ComputeNode" in source) {
		return { label: `CN${source.ComputeNode}`, description: `Compute node ${source.ComputeNode}.` };
	}
	if ("SharedMemory" in source) {
		const { slot, previous } = source.SharedMemory;
		return previous
			? {
					label: `PrevMem[${slot}]`,
					description: `Shared memory slot ${slot} as it stood at the start of this tick (last tick's final value).`,
				}
			: {
					label: `Mem[${slot}]`,
					description: `Shared memory slot ${slot} now, including writes made earlier this tick.`,
				};
	}
	if ("InputLeaf" in source) {
		const { ref_idx, sub_idx } = source.InputLeaf;
		const ref = ref_idx === 0xffff ? undefined : inputRefs[ref_idx];
		if (!ref) {
			return {
				label: "No input",
				description: `Points at input ${ref_idx}, which this node does not have; it reads 0.`,
			};
		}
		return describeInputRef(ref, sub_idx);
	}
	return { label: "?", description: "" };
}

/** Format a GraphSource as a human-readable label. */
export function formatGraphSource(source: GraphSource, inputRefs: InputReference[]): string {
	return describeGraphSource(source, inputRefs).label;
}

/** What a vote for each sink does once the pass ends. */
export function describeVoteSink(sink: VoteSink): string {
	if (sink === "Eat") {
		return "Vote to eat. At pass end the action whose vote most exceeds its bar commits.";
	}
	if (sink === "Terminate") {
		return "Ends the tick without another action when it is at least the winning vote. Needs an action already committed this tick.";
	}
	if (sink === "Decide") {
		return "When positive and some action is winning, ends this pass at once, skipping routing.";
	}
	if ("Drive" in sink) {
		return `Added to ${sink.Drive}'s best direction vote: whether to ${sink.Drive} at all, not where. With flat direction votes a positive drive keeps the current heading; a negative one suppresses ${sink.Drive}.`;
	}
	const [kind, dir] = Object.entries(sink)[0] ?? ["?", 0];
	return `Vote to ${kind} toward the ${directionWord(Number(dir))}. The highest of ${kind}'s 8 direction votes sets its direction.`;
}

const ACTION_PARAM_DESCRIPTIONS: Readonly<Record<ActionParamField, string>> = {
	EatFoodType: "Which food type Eat targets (rounded). A type that does not exist eats nothing.",
	ReproduceTransferFraction: "Share of the parent's energy given to the child (0 to 1).",
	StealEnergyAmount: "Energy to steal; the effort cost grows with the amount asked.",
};

/** Describe an output sink: its label, subtitle and tooltip. */
export function describeOutputSink(kind: OutputSinkKind): Described & { subtitle?: string } {
	if ("RouterGate" in kind) {
		return {
			label: `Route[${kind.RouterGate}]`,
			subtitle: "route score",
			description: `Score for route slot ${kind.RouterGate}. The next mesh node is the target with the highest bias + score. Ignored when the pass decides.`,
		};
	}
	if ("CustomOutput" in kind) {
		return {
			label: `Bus[${kind.CustomOutput}]`,
			subtitle: "to next node",
			description: `Writes bus slot ${kind.CustomOutput}; the next mesh node to run reads it as Bus[${kind.CustomOutput}].`,
		};
	}
	if ("WriteSlot" in kind) {
		return {
			label: `Write Mem[${kind.WriteSlot}]`,
			subtitle: "shared memory",
			description: `Overwrites shared memory slot ${kind.WriteSlot} at once. Read back as Mem[${kind.WriteSlot}] this tick and PrevMem[${kind.WriteSlot}] next tick.`,
		};
	}
	if ("ClearSlot" in kind) {
		return {
			label: `Clear Mem[${kind.ClearSlot}]`,
			subtitle: "shared memory",
			description: `Sets shared memory slot ${kind.ClearSlot} to 0 whenever this node runs, whatever its input.`,
		};
	}
	if ("ActionVote" in kind) {
		return {
			label: `Vote ${formatVoteSink(kind.ActionVote)}`,
			subtitle: "action vote",
			description: describeVoteSink(kind.ActionVote),
		};
	}
	if ("ActionParam" in kind) {
		return {
			label: `Param ${ACTION_PARAM_FIELD_LABELS[kind.ActionParam]}`,
			subtitle: "action parameter",
			description: ACTION_PARAM_DESCRIPTIONS[kind.ActionParam],
		};
	}
	return { label: "?", description: "" };
}

/** Short label of each action-parameter field. */
const ACTION_PARAM_FIELD_LABELS: Readonly<Record<ActionParamField, string>> = {
	EatFoodType: "Eat.food",
	EatLeaveShare: "Eat.leave",
	ReproduceTransferFraction: "Reproduce.frac",
	StealEnergyAmount: "StealEnergy.amt",
};

/** Target of a VM `WriteActionParam` at `field_idx`: `param <field>`, or
 * `param[i]` for an index outside the catalog, which the VM ignores. */
export function actionParamTargetLabel(fieldIdx: number): string {
	const field = ACTION_PARAM_FIELDS[fieldIdx];
	return field ? `param ${ACTION_PARAM_FIELD_LABELS[field]}` : `param[${fieldIdx}]`;
}

/** Tooltip for a VM `WriteActionParam` at `field_idx`. */
export function actionParamDescription(fieldIdx: number): string {
	const field = ACTION_PARAM_FIELDS[fieldIdx];
	return field
		? ACTION_PARAM_DESCRIPTIONS[field]
		: "Not a parameter field; the VM ignores this write.";
}

/** Format an OutputSinkKind as a human-readable label. */
export function formatOutputSinkKind(kind: OutputSinkKind): string {
	return describeOutputSink(kind).label;
}

/** Human-readable subtitle for an OutputSinkKind. */
export function outputSinkSubtitle(kind: OutputSinkKind): string | undefined {
	return describeOutputSink(kind).subtitle;
}
