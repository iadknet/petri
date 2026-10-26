import type { InputRef } from "../../types/generated/InputRef.ts";
import type { WorldAction } from "../../types/generated/WorldAction.ts";
import { VOTE_KINDS, voteSinkFromIndex } from "../../types/vote-sinks.ts";
import directionPairsFixture from "./directionPairs.json";
import { directionName, directionWord } from "./directions.ts";
import nearbyFields from "./nearbyFields.json";
import { actionCodeLegend, formatVoteSink } from "./voteSinks.ts";

export { directionName } from "./directions.ts";

export function formatAction(action: WorldAction | null | undefined): string {
	if (!action) return "?";
	if (typeof action === "string") return action;
	if ("Eat" in action) return `Eat(food${action.Eat.type_idx})`;
	if ("Move" in action) return `Move(${action.Move})`;
	if ("Reproduce" in action) return `Reproduce(${action.Reproduce.direction})`;
	if ("StealEnergy" in action) return `Steal(${action.StealEnergy.direction})`;
	if ("GiveEnergy" in action) return `Give(${action.GiveEnergy.direction})`;
	return "?";
}

export function formatActionList(actions: WorldAction[] | null | undefined): string {
	if (!actions || actions.length === 0) return "NoOp";
	if (actions.length === 1) return formatAction(actions[0]);
	return `${formatAction(actions[0])} +${actions.length - 1}`;
}

const RING_LABELS: Record<string, string> = {
	NeighborFoodRing: "FoodRing",
	NeighborBarrierRing: "BarrierRing",
	NeighborOccupiedRing: "OccRing",
};

/** Display-friendly labels for all world sensor keys. */
const SENSOR_LABELS: Record<string, string> = {
	...RING_LABELS,
	FoodHere: "FoodHere",
	AreaFoodSummary: "AreaFood",
	AreaBarrierSummary: "AreaBarrier",
	AreaOccupancySummary: "AreaOcc",
	NearbyCreature: "Nearby",
};

/** What each whole world sensor reads, for tooltips. The view is the
 * (2r+1)² square around the creature (r = vision radius, default 5);
 * barriers block line of sight. */
const SENSOR_DESCRIPTIONS: Record<string, string> = {
	FoodHere: "Food on the creature's own cell (0 to 1).",
	NeighborFoodRing: "Food on each of the 8 neighboring cells (0 to 1).",
	NeighborBarrierRing: "Whether each of the 8 neighboring cells is a barrier (1) or open (0).",
	NeighborOccupiedRing: "Whether another creature stands on each of the 8 neighboring cells.",
	AreaFoodSummary: "Summary of the food the creature can see.",
	AreaBarrierSummary: "Summary of the barriers the creature can see.",
	AreaOccupancySummary: "Summary of the other creatures the creature can see.",
	NearbyCreature: "Position, vitals, kinship and colour of the 4 nearest visible creatures.",
};

interface SubField {
	name: string;
	description: string;
}

const NONE_VISIBLE = " 0 when none visible.";

/** Offset-to-nearest tooltip; offsets are fractions of the vision radius. */
function nearestOffset(axis: "x" | "y", thing: string, none: string): string {
	const sign = axis === "x" ? "+ east, − west" : "+ south, − north";
	return `${axis.toUpperCase()} offset to the nearest ${thing}, as a fraction of vision radius (${sign}).${none}`;
}

/** One `(x, y)` offset pair a world family declares and the thing's facts. */
export interface DirectionPair {
	x: number;
	y: number;
	facts: readonly number[];
}

/** `WorldInputKey::direction_pairs()`, from the fixture v3-core pins. */
const DIRECTION_PAIRS: Record<string, readonly DirectionPair[]> = directionPairsFixture;

/** The pairs `ref` declares; none for a non-world reference. */
export function directionPairs(ref: InputRef | undefined): readonly DirectionPair[] {
	if (ref?.family !== "world") return [];
	return DIRECTION_PAIRS[ref.key] ?? [];
}

/** Area summary sub-fields, in the index order of `perception.rs`. */
const AREA_FIELDS: Record<string, readonly SubField[]> = {
	AreaFoodSummary: [
		{ name: "amount", description: "Total food in view, as a fraction of the view's capacity." },
		{ name: "gradX", description: "Which side holds more food: + east, − west." },
		{ name: "gradY", description: "Which side holds more food: + south, − north." },
		{ name: "nearestDx", description: nearestOffset("x", "food", "") },
		{ name: "nearestDy", description: nearestOffset("y", "food", "") },
		{
			name: "nearestDist",
			description:
				"Distance to the nearest food (0 = on this cell, 1 = edge of view). Also 0 when no food is visible.",
		},
		{ name: "max", description: "Richest single cell in view (0 to 1)." },
	],
	AreaBarrierSummary: [
		{ name: "density", description: "Share of the view that is barrier (0 to 1)." },
		{ name: "adjacent", description: "Share of the 8 neighboring cells that are barrier." },
		{ name: "gradX", description: "Which side holds more barrier: + east, − west." },
		{ name: "gradY", description: "Which side holds more barrier: + south, − north." },
		{ name: "nearestDx", description: nearestOffset("x", "barrier", NONE_VISIBLE) },
		{ name: "nearestDy", description: nearestOffset("y", "barrier", NONE_VISIBLE) },
		{
			name: "nearestDist",
			description: "Distance to the nearest barrier (1 = edge of view). 0 when none visible.",
		},
	],
	AreaOccupancySummary: [
		{ name: "count", description: "Other creatures in view, as a share of the view's cells." },
		{ name: "centerX", description: "Average x of the creatures in view: + east, − west." },
		{ name: "centerY", description: "Average y of the creatures in view: + south, − north." },
		{ name: "nearestDx", description: nearestOffset("x", "creature", NONE_VISIBLE) },
		{ name: "nearestDy", description: nearestOffset("y", "creature", NONE_VISIBLE) },
		{
			name: "nearestDist",
			description: "Distance to the nearest creature (1 = edge of view). 0 when none visible.",
		},
		{ name: "crowding", description: "How crowded the view is, closer creatures counting more." },
	],
};

const OUTCOME_FIELDS: readonly SubField[] = [
	{
		name: "energy",
		description: "Energy change over the last tick, as a fraction of max energy (−1 to 1).",
	},
	{
		name: "success",
		description: "Share of last tick's attempted actions that succeeded (0 if none attempted).",
	},
	{
		name: "damage",
		description:
			"Energy stolen from this creature last tick, as a negative fraction of max energy.",
	},
	{ name: "offspring", description: "Offspring born last tick (a raw count)." },
];

const QUEUE_FIELDS: readonly SubField[] = [
	{ name: "type", description: `Action code: ${actionCodeLegend()}.` },
	{ name: "param0", description: "Direction index (N=0 … NW=7); for Eat, the food type." },
	{ name: "param1", description: "Reproduce: energy share given; StealEnergy: effort modifier." },
];

const SCALAR_LABELS: Record<string, { label: string; description: string }> = {
	AgeTicks: { label: "Age", description: "Age as a fraction of the reference age (caps at 1)." },
	OwnRed: { label: "OwnRed", description: "Own red: this creature's own red channel (0 to 1)." },
	OwnGreen: {
		label: "OwnGreen",
		description: "Own green: this creature's own green channel (0 to 1).",
	},
	OwnBlue: {
		label: "OwnBlue",
		description: "Own blue: this creature's own blue channel (0 to 1).",
	},
	EnergyCurrent: { label: "Energy", description: "Current energy as a fraction of max energy." },
	EnergyConsumedThisTick: {
		label: "EnergySpent",
		description: "Energy spent so far this tick, as a fraction of max energy.",
	},
	HopsThisTick: {
		label: "Hops",
		description: "Mesh nodes run so far this tick, this one included (a raw count).",
	},
};

const STRING_REFS: Record<string, { label: string; description: string }> = {
	ActionQueue: {
		label: "Queue",
		description: "Actions already committed this tick (empty in the first pass).",
	},
	ActionVotes: { label: "Votes", description: "This pass's running vote for each action." },
	PreviousPassVotes: {
		label: "PrevVotes",
		description: "The votes at the end of the previous pass (0 in the first pass).",
	},
	CommitCounts: {
		label: "Commits",
		description: "How many times each action kind has been committed this tick.",
	},
	PreviousOutcome: { label: "PrevOutcome", description: "How the last tick went." },
};

const ORDINALS = ["nearest", "2nd-nearest", "3rd-nearest", "4th-nearest"] as const;
const QUEUE_ORDINALS = ["1st", "2nd", "3rd", "4th", "5th", "6th", "7th", "8th"] as const;

/** True if the given World key is a ring sensor (compound with 8 directional sub-values). */
export function isRingSensor(worldKey: string): boolean {
	return worldKey in RING_LABELS;
}

/** A readable name for an input and a plain-words explanation for its tooltip. */
export interface InputDescription {
	label: string;
	description: string;
}

/** A typed food sensor's description suffix; empty for any other input. */
function foodNote(ref: InputRef): string {
	return ref.food_type === undefined ? "" : ` Food type ${ref.food_type} only.`;
}

/** Describe a whole input reference (no sub-value selected). */
export function describeWholeInputRef(ref: InputRef): InputDescription {
	switch (ref.family) {
		case "world": {
			const base = SENSOR_LABELS[ref.key] ?? ref.key;
			const description = SENSOR_DESCRIPTIONS[ref.key] ?? "";
			if (ref.food_type !== undefined) {
				return {
					label: `${base}[t${ref.food_type}]`,
					description: `${description}${foodNote(ref)}`,
				};
			}
			return { label: base, description };
		}
		case "static":
		case "dynamic":
			return SCALAR_LABELS[ref.key] ?? { label: ref.key, description: "" };
		case "upstream":
			return {
				label: `Bus[${ref.slot}]`,
				description: `Bus slot ${ref.slot}: the value the previous mesh node wrote there (0 at tick start).`,
			};
		case "action_queue":
		case "decision":
			return STRING_REFS[ref.key] ?? { label: ref.key, description: "" };
	}
}

export function formatInputRef(ref: InputRef): string {
	return describeWholeInputRef(ref).label;
}

/** Name one sub-value of an input: a direction, rank or field, never a bare
 * index. Scalars stay bare; an index past the compound's width shows raw. */
export function describeInputRef(ref: InputRef, subIdx: number): InputDescription {
	const whole = describeWholeInputRef(ref);
	const raw = (): InputDescription => ({
		label: subIdx > 0 ? `${whole.label}[${subIdx}]` : whole.label,
		description: whole.description,
	});

	if (ref.family === "decision" || ref.family === "action_queue") {
		switch (ref.key) {
			case "ActionVotes":
			case "PreviousPassVotes": {
				const sink = voteSinkFromIndex(subIdx);
				if (sink === null) return raw();
				const name = formatVoteSink(sink);
				const description =
					ref.key === "ActionVotes"
						? `This pass's running vote for ${name}.`
						: `The vote for ${name} at the end of the previous pass (0 in the first pass).`;
				return { label: `${whole.label}.${name}`, description };
			}
			case "CommitCounts": {
				const kind = VOTE_KINDS[subIdx];
				if (kind === undefined) return raw();
				return {
					label: `${whole.label}.${kind}`,
					description: `${kind} commits so far this tick; each raises the bar the next ${kind} vote must clear.`,
				};
			}
			case "PreviousOutcome":
				return field(whole.label, OUTCOME_FIELDS[subIdx]) ?? raw();
			case "ActionQueue": {
				const slot = Math.floor(subIdx / QUEUE_FIELDS.length);
				const queueField = QUEUE_FIELDS[subIdx % QUEUE_FIELDS.length];
				const ordinal = QUEUE_ORDINALS[slot] ?? `#${slot + 1}`;
				if (!queueField) return raw();
				return {
					label: `Queue${slot + 1}.${queueField.name}`,
					description: `The ${ordinal} action committed this tick. ${queueField.description}`,
				};
			}
			default:
				return raw();
		}
	}

	if (ref.family !== "world") return raw();

	if (isRingSensor(ref.key)) {
		const cell = `the cell to the ${directionWord(subIdx)}`;
		const description =
			ref.key === "NeighborFoodRing"
				? `Food on ${cell} (0 to 1).${foodNote(ref)}`
				: ref.key === "NeighborBarrierRing"
					? `1 when ${cell} is a barrier, 0 when open.`
					: `1 when another creature stands on ${cell}.`;
		return { label: `${whole.label}[${directionName(subIdx)}]`, description };
	}

	const area = AREA_FIELDS[ref.key];
	if (area) {
		const described = field(whole.label, area[subIdx]);
		if (!described) return raw();
		return { ...described, description: `${described.description}${foodNote(ref)}` };
	}

	if (ref.key === "NearbyCreature") {
		const { slots, fields } = nearbyFields;
		const slot = Math.floor(subIdx / fields.length);
		const nearbyField = fields[subIdx % fields.length];
		if (slot >= slots || !nearbyField) return raw();
		return {
			label: `Nearest${slot + 1}.${nearbyField.name}`,
			description: `The ${ORDINALS[slot]} visible creature: ${lowerFirst(nearbyField.description)} All 0 when the slot is empty.`,
		};
	}

	return raw();
}

function field(base: string, subField: SubField | undefined): InputDescription | null {
	if (!subField) return null;
	return { label: `${base}.${subField.name}`, description: subField.description };
}

function lowerFirst(text: string): string {
	return text.charAt(0).toLowerCase() + text.slice(1);
}

export function formatInputRefWithSubIndex(ref: InputRef, subIdx: number): string {
	return describeInputRef(ref, subIdx).label;
}

const STRING_REF_COLORS: Record<string, string> = {
	ActionQueue: "#f59e0b", // amber
	ActionVotes: "#a78bfa", // violet: the in-tick decision state
	PreviousPassVotes: "#a78bfa",
	CommitCounts: "#a78bfa",
	PreviousOutcome: "#60a5fa", // blue: introspection
};

export function inputRefColor(ref: InputRef): string {
	switch (ref.family) {
		case "world":
			return "#34d399"; // green
		case "static":
		case "dynamic":
			return "#60a5fa"; // blue
		case "upstream":
			return "#94a3b8"; // gray
		case "action_queue":
		case "decision":
			return STRING_REF_COLORS[ref.key] ?? "#94a3b8";
	}
}
