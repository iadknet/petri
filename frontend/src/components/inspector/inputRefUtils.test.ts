import { describe, expect, it } from "vitest";
import type { InputReference } from "../../types/genome.ts";
import type { WorldAction } from "../../types/trace.ts";
import directionPairsFixture from "./directionPairs.json";
import {
	describeInputRef,
	directionName,
	directionPairs,
	formatAction,
	formatActionList,
	formatInputRef,
	formatInputRefWithSubIndex,
	inputRefColor,
	isRingSensor,
	parseWorldInputRef,
} from "./inputRefUtils.ts";
import nearbyFieldsFixture from "./nearbyFields.json";

describe("formatAction", () => {
	it("formats current server world action variants", () => {
		const move: WorldAction = { Move: "E" };
		const reproduce: WorldAction = { Reproduce: { direction: "N", energy_transfer_fraction: 0.5 } };
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
		expect(formatInputRef(ref)).toBe("FoodHere[t2]");
	});

	it("formats StaticIntrospection variant", () => {
		const ref: InputReference = { StaticIntrospection: "AgeTicks" };
		expect(formatInputRef(ref)).toBe("Age");
	});

	it("formats UpstreamSlot variant", () => {
		const ref: InputReference = { UpstreamSlot: 3 };
		expect(formatInputRef(ref)).toBe("Bus[3]");
	});

	it("labels the decision-state references by short names", () => {
		expect(formatInputRef("ActionVotes")).toBe("Votes");
		expect(formatInputRef("PreviousOutcome")).toBe("PrevOutcome");
		expect(formatInputRef({ DynamicIntrospection: "HopsThisTick" })).toBe("Hops");
	});

	it("handles ActionQueue string variant without crashing", () => {
		const ref: InputReference = "ActionQueue";
		expect(formatInputRef(ref)).toBe("Queue");
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

	it("returns violet for the in-tick decision-state references", () => {
		for (const ref of ["ActionVotes", "PreviousPassVotes", "CommitCounts"] as const) {
			expect(inputRefColor(ref)).toBe("#a78bfa");
		}
	});

	it("returns blue for the frozen previous outcome", () => {
		expect(inputRefColor("PreviousOutcome")).toBe("#60a5fa");
	});

	it("returns gray for unknown string variant", () => {
		const ref = "UnknownFuture" as unknown as InputReference;
		expect(inputRefColor(ref)).toBe("#94a3b8");
	});
});

describe("formatInputRefWithSubIndex", () => {
	it("uses direction labels for ring sensors, with the food type", () => {
		expect(formatInputRefWithSubIndex({ World: "NeighborBarrierRing" }, 0)).toBe("BarrierRing[N]");
		expect(formatInputRefWithSubIndex({ World: { NeighborFoodRing: { type_idx: 1 } } }, 2)).toBe(
			"FoodRing[t1][E]",
		);
	});

	it("names every area-summary sub-field, index 0 included", () => {
		const barrier: InputReference = { World: "AreaBarrierSummary" };
		expect(formatInputRefWithSubIndex(barrier, 0)).toBe("AreaBarrier.density");
		expect(formatInputRefWithSubIndex(barrier, 1)).toBe("AreaBarrier.adjacent");
		expect(formatInputRefWithSubIndex(barrier, 6)).toBe("AreaBarrier.nearestDist");
		expect(formatInputRefWithSubIndex({ World: { AreaFoodSummary: { type_idx: 0 } } }, 0)).toBe(
			"AreaFood[t0].amount",
		);
		expect(formatInputRefWithSubIndex({ World: "AreaOccupancySummary" }, 6)).toBe(
			"AreaOcc.crowding",
		);
	});

	it("names nearby-creature fields by rank, nearest first", () => {
		expect(formatInputRefWithSubIndex({ World: "NearbyCreature" }, 0)).toBe("Nearest1.present");
		expect(formatInputRefWithSubIndex({ World: "NearbyCreature" }, 12)).toBe("Nearest2.dy");
		expect(formatInputRefWithSubIndex({ World: "NearbyCreature" }, 5)).toBe("Nearest1.ready");
		expect(formatInputRefWithSubIndex({ World: "NearbyCreature" }, 14)).toBe("Nearest2.energy");
		expect(formatInputRefWithSubIndex({ World: "NearbyCreature" }, 16)).toBe("Nearest2.kin");
		expect(formatInputRefWithSubIndex({ World: "NearbyCreature" }, 17)).toBe("Nearest2.red");
		expect(formatInputRefWithSubIndex({ World: "NearbyCreature" }, 7)).toBe("Nearest1.red");
	});

	it("names every field of the nearbyFields fixture at every rank, in layout order", () => {
		const { slots, fields } = nearbyFieldsFixture;
		expect(fields.map((field) => field.name)).toEqual([
			"present",
			"dx",
			"dy",
			"dist",
			"energy",
			"ready",
			"kin",
			"red",
			"green",
			"blue",
		]);
		for (let slot = 0; slot < slots; slot++) {
			fields.forEach((field, index) => {
				const channel = slot * fields.length + index;
				const described = describeInputRef({ World: "NearbyCreature" }, channel);
				expect(described.label).toBe(`Nearest${slot + 1}.${field.name}`);
				expect(described.description).toContain(field.description.slice(1));
			});
		}
		const width = slots * fields.length;
		expect(width).toBe(40);
		expect(formatInputRefWithSubIndex({ World: "NearbyCreature" }, 39)).toBe("Nearest4.blue");
		expect(formatInputRefWithSubIndex({ World: "NearbyCreature" }, 40)).toBe("Nearby[40]");
	});

	it("names decision-state and queue sub-values", () => {
		expect(formatInputRefWithSubIndex("ActionVotes", 0)).toBe("Votes.Eat");
		expect(formatInputRefWithSubIndex("ActionVotes", 22)).toBe("Votes.StealEnergy[SW]");
		expect(formatInputRefWithSubIndex("PreviousPassVotes", 25)).toBe("PrevVotes.Drive Move");
		expect(formatInputRefWithSubIndex("CommitCounts", 2)).toBe("Commits.Reproduce");
		expect(formatInputRefWithSubIndex("PreviousOutcome", 0)).toBe("PrevOutcome.energy");
		expect(formatInputRefWithSubIndex("ActionQueue", 0)).toBe("Queue1.type");
		expect(formatInputRefWithSubIndex("ActionQueue", 4)).toBe("Queue2.param0");
	});

	it("keeps scalars bare and shows an out-of-range sub-index raw", () => {
		expect(formatInputRefWithSubIndex({ DynamicIntrospection: "EnergyCurrent" }, 0)).toBe("Energy");
		expect(formatInputRefWithSubIndex({ World: "AreaBarrierSummary" }, 9)).toBe("AreaBarrier[9]");
	});
});

describe("describeInputRef", () => {
	it("explains a sub-field in plain words", () => {
		const d = describeInputRef({ World: "AreaBarrierSummary" }, 0);
		expect(d.label).toBe("AreaBarrier.density");
		expect(d.description).toMatch(/barrier/i);
	});

	it("explains every ring direction and nearby rank", () => {
		expect(describeInputRef({ World: "NeighborOccupiedRing" }, 4).description).toMatch(/south/i);
		expect(describeInputRef({ World: "NearbyCreature" }, 10).description).toMatch(/2nd-nearest/);
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

describe("directionPairs", () => {
	it("reads every family's pairs from the fixture v3-core is pinned to", () => {
		for (const [key, pairs] of Object.entries(directionPairsFixture)) {
			expect(directionPairs({ World: key } as InputReference)).toEqual(pairs);
		}
		expect(directionPairs({ World: "NearbyCreature" })[1]).toEqual({
			x: 11,
			y: 12,
			facts: [10, 13, 14, 15, 16, 17, 18, 19],
		});
		expect(directionPairs({ World: "NeighborBarrierRing" })).toEqual([]);
		expect(directionPairs("ActionQueue")).toEqual([]);
	});
});
