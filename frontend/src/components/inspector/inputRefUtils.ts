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

export function formatInputRef(ref: InputReference): string {
	if (typeof ref === "string") return ref;
	if ("World" in ref) {
		const w = ref.World;
		if (typeof w === "string") return w;
		if ("NeighborCellFood" in w) return `Food.${w.NeighborCellFood}`;
		if ("NeighborCellBarrier" in w) return `Barrier.${w.NeighborCellBarrier}`;
		if ("NeighborCellOccupied" in w) return `Occ.${w.NeighborCellOccupied}`;
		return "World:?";
	}
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
