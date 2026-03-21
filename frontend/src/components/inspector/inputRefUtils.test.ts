import { describe, expect, it } from "vitest";
import type { InputReference } from "../../types/genome.ts";
import type { WorldAction } from "../../types/trace.ts";
import {
	directionName,
	formatAction,
	formatActionList,
	formatInputRef,
	formatInputRefWithSubIndex,
	inputRefColor,
	isRingSensor,
	parseWorldInputRef,
} from "./inputRefUtils.ts";

describe("formatAction", () => {
	it("formats current server world action variants", () => {
		const move: WorldAction = { Move: "E" };
		const reproduce: WorldAction = { Reproduce: { direction: "N", energy_transfer: 12 } };
		const steal: WorldAction = { StealEnergy: { direction: "W", amount: 3 } };

		expect(formatAction(move)).toBe("Move(E)");
		expect(formatAction(reproduce)).toBe("Reproduce(N)");
		expect(formatAction(steal)).toBe("Steal(W)");
	});

	it("summarizes action arrays without crashing on empty entries", () => {
		expect(formatActionList(["NoOp"])).toBe("NoOp");
		expect(formatActionList(["Eat", { Move: "S" }])).toBe("Eat +1");
		expect(formatActionList([])).toBe("NoOp");
		expect(formatAction(undefined)).toBe("?");
	});
});

describe("formatInputRef", () => {
	it("formats World string variant", () => {
		const ref: InputReference = { World: "FoodHere" };
		expect(formatInputRef(ref)).toBe("FoodHere");
	});

	it("formats ring sensor variants", () => {
		expect(formatInputRef({ World: "NeighborFoodRing" })).toBe("FoodRing");
		expect(formatInputRef({ World: "NeighborBarrierRing" })).toBe("BarrierRing");
		expect(formatInputRef({ World: "NeighborOccupiedRing" })).toBe("OccRing");
	});

	it("formats typed world food variants", () => {
		const ref = {
			World: {
				FoodHere: {
					type_idx: 2,
				},
			},
		} as unknown as InputReference;
		expect(formatInputRef(ref)).toBe("FoodHere[2]");
	});

	it("formats StaticIntrospection variant", () => {
		const ref: InputReference = { StaticIntrospection: "Generation" };
		expect(formatInputRef(ref)).toBe("Generation");
	});

	it("formats UpstreamSlot variant", () => {
		const ref: InputReference = { UpstreamSlot: 3 };
		expect(formatInputRef(ref)).toBe("slot[3]");
	});

	it("handles ActionQueue string variant without crashing", () => {
		const ref: InputReference = "ActionQueue";
		expect(formatInputRef(ref)).toBe("ActionQueue");
	});

	it("handles unknown string variant gracefully", () => {
		const ref = "SomeFutureVariant" as unknown as InputReference;
		expect(formatInputRef(ref)).toBe("SomeFutureVariant");
	});
});

describe("inputRefColor", () => {
	it("returns green for World refs", () => {
		const ref: InputReference = { World: "FoodHere" };
		expect(inputRefColor(ref)).toBe("#34d399");
	});

	it("returns blue for introspection refs", () => {
		const ref: InputReference = { StaticIntrospection: "Age" };
		expect(inputRefColor(ref)).toBe("#60a5fa");
	});

	it("returns gray for upstream refs", () => {
		const ref: InputReference = { UpstreamSlot: 0 };
		expect(inputRefColor(ref)).toBe("#94a3b8");
	});

	it("returns amber for ActionQueue string variant", () => {
		const ref: InputReference = "ActionQueue";
		expect(inputRefColor(ref)).toBe("#f59e0b");
	});

	it("returns gray for unknown string variant", () => {
		const ref = "UnknownFuture" as unknown as InputReference;
		expect(inputRefColor(ref)).toBe("#94a3b8");
	});
});

describe("formatInputRefWithSubIndex", () => {
	it("uses direction labels for ring sensors", () => {
		expect(formatInputRefWithSubIndex({ World: "NeighborFoodRing" }, 2)).toBe("FoodRing[E]");
	});

	it("uses numeric sub-index for non-ring world refs", () => {
		expect(formatInputRefWithSubIndex({ World: "FoodHere" }, 3)).toBe("FoodHere[3]");
	});

	it("passes through base label for zero sub-index", () => {
		expect(formatInputRefWithSubIndex({ DynamicIntrospection: "AgeTicks" }, 0)).toBe("AgeTicks");
	});
});

describe("directionName", () => {
	it("maps sub_idx 0..7 to direction names", () => {
		expect(directionName(0)).toBe("N");
		expect(directionName(1)).toBe("NE");
		expect(directionName(2)).toBe("E");
		expect(directionName(3)).toBe("SE");
		expect(directionName(4)).toBe("S");
		expect(directionName(5)).toBe("SW");
		expect(directionName(6)).toBe("W");
		expect(directionName(7)).toBe("NW");
	});

	it("wraps values >= 8", () => {
		expect(directionName(8)).toBe("N");
		expect(directionName(10)).toBe("E");
	});
});

describe("isRingSensor", () => {
	it("returns true for ring sensor keys", () => {
		expect(isRingSensor("NeighborFoodRing")).toBe(true);
		expect(isRingSensor("NeighborBarrierRing")).toBe(true);
		expect(isRingSensor("NeighborOccupiedRing")).toBe(true);
	});

	it("returns false for non-ring keys", () => {
		expect(isRingSensor("FoodHere")).toBe(false);
		expect(isRingSensor("AreaFoodSummary")).toBe(false);
	});
});

describe("parseWorldInputRef", () => {
	it("extracts key and type index from typed world payloads", () => {
		expect(
			parseWorldInputRef({
				NeighborFoodRing: {
					type_idx: 3,
				},
			}),
		).toEqual({
			key: "NeighborFoodRing",
			typeIdx: 3,
		});
	});

	it("passes through legacy world string payloads", () => {
		expect(parseWorldInputRef("FoodHere")).toEqual({
			key: "FoodHere",
			typeIdx: null,
		});
	});
});
