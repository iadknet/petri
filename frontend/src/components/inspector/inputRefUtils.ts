import type { InputReference } from "../../types/genome.ts";
import type { WorldAction } from "../../types/trace.ts";

export function formatAction(action: WorldAction | null | undefined): string {
	if (!action) return "?";
	if (typeof action === "string") return action;
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

const DIRECTION_NAMES = ["N", "NE", "E", "SE", "S", "SW", "W", "NW"] as const;

/** Map a ring sensor sub_idx to a direction name (N=0, NE=1, ..., NW=7). */
export function directionName(subIdx: number): string {
	return DIRECTION_NAMES[subIdx % 8] ?? `?${subIdx}`;
}

/** True if the given World key is a ring sensor (compound with 8 directional sub-values). */
export function isRingSensor(worldKey: string): boolean {
	return worldKey in RING_LABELS;
}

export function formatInputRef(ref: InputReference): string {
	if (typeof ref === "string") return ref;
	if ("World" in ref) return RING_LABELS[ref.World] ?? ref.World;
	if ("StaticIntrospection" in ref) return ref.StaticIntrospection;
	if ("DynamicIntrospection" in ref) return ref.DynamicIntrospection;
	if ("UpstreamSlot" in ref) return `slot[${ref.UpstreamSlot}]`;
	return "?";
}

export function inputRefColor(ref: InputReference): string {
	if (typeof ref === "string") return ref === "ActionQueue" ? "#f59e0b" : "#94a3b8";
	if ("World" in ref) return "#34d399"; // green
	if ("StaticIntrospection" in ref || "DynamicIntrospection" in ref) return "#60a5fa"; // blue
	return "#94a3b8"; // gray for upstream
}
