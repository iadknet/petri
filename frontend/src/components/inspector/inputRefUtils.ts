import type { InputReference, WorldInputReference } from "../../types/genome.ts";
import type { WorldAction } from "../../types/trace.ts";

export function formatAction(action: WorldAction | null | undefined): string {
	if (!action) return "?";
	if (typeof action === "string") return action;
	if ("Eat" in action) {
		const payload = action.Eat;
		if (typeof payload === "object" && payload && "type_idx" in payload) {
			return `Eat(food${(payload as { type_idx: number }).type_idx})`;
		}
		return "Eat";
	}
	if ("Move" in action) return `Move(${action.Move})`;
	if ("Reproduce" in action) return `Reproduce(${action.Reproduce.direction})`;
	if ("StealEnergy" in action) return `Steal(${action.StealEnergy.direction})`;
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
	NearbyCreatureCore: "NearbyCore",
	NearbyCreatureVitals: "NearbyVitals",
	NearbyCreatureIdentity: "NearbyIdent",
};

const DIRECTION_NAMES = ["N", "NE", "E", "SE", "S", "SW", "W", "NW"] as const;

/** Map a ring sensor sub_idx to a direction name (N=0, NE=1, ..., NW=7). */
export function directionName(subIdx: number): string {
	return DIRECTION_NAMES[subIdx % 8] ?? `?${subIdx}`;
}

/** True if the given World key is a ring sensor (compound with 8 directional sub-values). */
export function isRingSensor(worldKey: string): boolean {
	return worldKey in RING_LABELS;
}

export interface ParsedWorldInputReference {
	key: string;
	typeIdx: number | null;
}

export function parseWorldInputRef(world: WorldInputReference): ParsedWorldInputReference {
	if (typeof world === "string") {
		return {
			key: world,
			typeIdx: null,
		};
	}

	const [key, payload] = Object.entries(world)[0] ?? ["?", null];
	const keyString = String(key);
	if (
		payload &&
		typeof payload === "object" &&
		"type_idx" in payload &&
		typeof (payload as { type_idx?: unknown }).type_idx === "number"
	) {
		return {
			key: keyString,
			typeIdx: (payload as { type_idx: number }).type_idx,
		};
	}

	return {
		key: keyString,
		typeIdx: null,
	};
}

export function formatInputRef(ref: InputReference): string {
	if (typeof ref === "string") return ref;
	if ("World" in ref) {
		const parsed = parseWorldInputRef(ref.World);
		const base = SENSOR_LABELS[parsed.key] ?? parsed.key;
		if (parsed.typeIdx !== null) {
			return `${base}[${parsed.typeIdx}]`;
		}
		return base;
	}
	if ("StaticIntrospection" in ref) return String(ref.StaticIntrospection);
	if ("DynamicIntrospection" in ref) return String(ref.DynamicIntrospection);
	if ("UpstreamSlot" in ref) return `slot[${ref.UpstreamSlot}]`;
	return "?";
}

export function formatInputRefWithSubIndex(ref: InputReference, subIdx: number): string {
	const label = formatInputRef(ref);
	if (typeof ref !== "string" && "World" in ref && isRingSensor(parseWorldInputRef(ref.World).key)) {
		return `${label}[${directionName(subIdx)}]`;
	}
	return subIdx > 0 ? `${label}[${subIdx}]` : label;
}

export function inputRefColor(ref: InputReference): string {
	if (typeof ref === "string") return ref === "ActionQueue" ? "#f59e0b" : "#94a3b8";
	if ("World" in ref) return "#34d399"; // green
	if ("StaticIntrospection" in ref || "DynamicIntrospection" in ref) return "#60a5fa"; // blue
	return "#94a3b8"; // gray for upstream
}
