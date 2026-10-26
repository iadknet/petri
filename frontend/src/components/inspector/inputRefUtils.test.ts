import { describe, expect, it } from "vitest";
import type { InputKey } from "../../types/generated/InputKey.ts";
import type { InputRef } from "../../types/generated/InputRef.ts";
import type { WorldAction } from "../../types/generated/WorldAction.ts";
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
} from "./inputRefUtils.ts";
import nearbyFieldsFixture from "./nearbyFields.json";

describe("formatAction", () => {
	it("formats current server world action variants", () => {
		const move: WorldAction = { Move: "E" };
		const reproduce: WorldAction = { Reproduce: { direction: "N", energy_transfer_fraction: 0.5 } };
		const steal: WorldAction = { StealEnergy: { direction: "W", effort: 2 } };
		const give: WorldAction = { GiveEnergy: { direction: "S", effort: 1 } };

		expect(formatAction(move)).toBe("Move(E)");
		expect(formatAction(reproduce)).toBe("Reproduce(N)");
		expect(formatAction(steal)).toBe("Steal(W)");
		expect(formatAction(give)).toBe("Give(S)");
	});

	it("summarizes action arrays without crashing on empty entries", () => {
		expect(formatActionList(["NoOp"])).toBe("NoOp");
		expect(formatActionList([{ Eat: { type_idx: 1, leave: 0 } }, { Move: "S" }])).toBe(
			"Eat(food1) +1",
		);
		expect(formatActionList([])).toBe("NoOp");
		expect(formatAction(undefined)).toBe("?");
	});
});

describe("formatInputRef", () => {
	it("formats World string variant", () => {
		const ref: InputRef = { family: "world", key: "FoodHere" };
		expect(formatInputRef(ref)).toBe("FoodHere");
	});

	it("formats ring sensor variants", () => {
		expect(formatInputRef({ family: "world", key: "NeighborFoodRing" })).toBe("FoodRing");
		expect(formatInputRef({ family: "world", key: "NeighborBarrierRing" })).toBe("BarrierRing");
		expect(formatInputRef({ family: "world", key: "NeighborOccupiedRing" })).toBe("OccRing");
	});

	it("formats typed world food variants", () => {
		const ref: InputRef = { family: "world", key: "FoodHere", food_type: 2 };
		expect(formatInputRef(ref)).toBe("FoodHere[t2]");
	});

	it("formats StaticIntrospection variant", () => {
		const ref: InputRef = { family: "static", key: "AgeTicks" };
		expect(formatInputRef(ref)).toBe("Age");
	});

	it("labels and explains the three own-colour inputs", () => {
		for (const [key, channel] of [
			["OwnRed", "red"],
			["OwnGreen", "green"],
			["OwnBlue", "blue"],
		] as const) {
			const described = describeInputRef({ family: "static", key }, 0);
			expect(described.label).toBe(key);
			expect(described.description).toBe(
				`Own ${channel}: this creature's own ${channel} channel (0 to 1).`,
			);
		}
	});

	it("formats UpstreamSlot variant", () => {
		const ref: InputRef = { family: "upstream", key: "UpstreamSlot", slot: 3 };
		expect(formatInputRef(ref)).toBe("Bus[3]");
	});

	it("labels the decision-state references by short names", () => {
		expect(formatInputRef({ family: "decision", key: "ActionVotes" })).toBe("Votes");
		expect(formatInputRef({ family: "decision", key: "PreviousOutcome" })).toBe("PrevOutcome");
		expect(formatInputRef({ family: "dynamic", key: "HopsThisTick" })).toBe("Hops");
	});

	it("handles ActionQueue string variant without crashing", () => {
		const ref: InputRef = { family: "action_queue", key: "ActionQueue" };
		expect(formatInputRef(ref)).toBe("Queue");
	});
});

describe("inputRefColor", () => {
	it("returns green for World refs", () => {
		const ref: InputRef = { family: "world", key: "FoodHere" };
		expect(inputRefColor(ref)).toBe("#34d399");
	});

	it("returns blue for introspection refs", () => {
		const ref: InputRef = { family: "static", key: "AgeTicks" };
		expect(inputRefColor(ref)).toBe("#60a5fa");
	});

	it("returns gray for upstream refs", () => {
		const ref: InputRef = { family: "upstream", key: "UpstreamSlot", slot: 0 };
		expect(inputRefColor(ref)).toBe("#94a3b8");
	});

	it("returns amber for ActionQueue string variant", () => {
		const ref: InputRef = { family: "action_queue", key: "ActionQueue" };
		expect(inputRefColor(ref)).toBe("#f59e0b");
	});

	it("returns violet for the in-tick decision-state references", () => {
		for (const key of ["ActionVotes", "PreviousPassVotes", "CommitCounts"] as const) {
			expect(inputRefColor({ family: "decision", key })).toBe("#a78bfa");
		}
	});

	it("returns blue for the frozen previous outcome", () => {
		expect(inputRefColor({ family: "decision", key: "PreviousOutcome" })).toBe("#60a5fa");
	});
});

describe("formatInputRefWithSubIndex", () => {
	it("uses direction labels for ring sensors, with the food type", () => {
		expect(formatInputRefWithSubIndex({ family: "world", key: "NeighborBarrierRing" }, 0)).toBe(
			"BarrierRing[N]",
		);
		expect(
			formatInputRefWithSubIndex({ family: "world", key: "NeighborFoodRing", food_type: 1 }, 2),
		).toBe("FoodRing[t1][E]");
	});

	it("names every area-summary sub-field, index 0 included", () => {
		const barrier: InputRef = { family: "world", key: "AreaBarrierSummary" };
		expect(formatInputRefWithSubIndex(barrier, 0)).toBe("AreaBarrier.density");
		expect(formatInputRefWithSubIndex(barrier, 1)).toBe("AreaBarrier.adjacent");
		expect(formatInputRefWithSubIndex(barrier, 6)).toBe("AreaBarrier.nearestDist");
		expect(
			formatInputRefWithSubIndex({ family: "world", key: "AreaFoodSummary", food_type: 0 }, 0),
		).toBe("AreaFood[t0].amount");
		expect(formatInputRefWithSubIndex({ family: "world", key: "AreaOccupancySummary" }, 6)).toBe(
			"AreaOcc.crowding",
		);
	});

	it("names nearby-creature fields by rank, nearest first", () => {
		expect(formatInputRefWithSubIndex({ family: "world", key: "NearbyCreature" }, 0)).toBe(
			"Nearest1.present",
		);
		expect(formatInputRefWithSubIndex({ family: "world", key: "NearbyCreature" }, 12)).toBe(
			"Nearest2.dy",
		);
		expect(formatInputRefWithSubIndex({ family: "world", key: "NearbyCreature" }, 5)).toBe(
			"Nearest1.ready",
		);
		expect(formatInputRefWithSubIndex({ family: "world", key: "NearbyCreature" }, 14)).toBe(
			"Nearest2.energy",
		);
		expect(formatInputRefWithSubIndex({ family: "world", key: "NearbyCreature" }, 16)).toBe(
			"Nearest2.kin",
		);
		expect(formatInputRefWithSubIndex({ family: "world", key: "NearbyCreature" }, 17)).toBe(
			"Nearest2.red",
		);
		expect(formatInputRefWithSubIndex({ family: "world", key: "NearbyCreature" }, 7)).toBe(
			"Nearest1.red",
		);
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
				const described = describeInputRef({ family: "world", key: "NearbyCreature" }, channel);
				expect(described.label).toBe(`Nearest${slot + 1}.${field.name}`);
				expect(described.description).toContain(field.description.slice(1));
			});
		}
		const width = slots * fields.length;
		expect(width).toBe(40);
		expect(formatInputRefWithSubIndex({ family: "world", key: "NearbyCreature" }, 39)).toBe(
			"Nearest4.blue",
		);
		expect(formatInputRefWithSubIndex({ family: "world", key: "NearbyCreature" }, 40)).toBe(
			"Nearby[40]",
		);
	});

	it("names decision-state and queue sub-values", () => {
		expect(formatInputRefWithSubIndex({ family: "decision", key: "ActionVotes" }, 0)).toBe(
			"Votes.Eat",
		);
		expect(formatInputRefWithSubIndex({ family: "decision", key: "ActionVotes" }, 22)).toBe(
			"Votes.StealEnergy[SW]",
		);
		expect(formatInputRefWithSubIndex({ family: "decision", key: "PreviousPassVotes" }, 33)).toBe(
			"PrevVotes.Drive Move",
		);
		expect(formatInputRefWithSubIndex({ family: "decision", key: "CommitCounts" }, 2)).toBe(
			"Commits.Reproduce",
		);
		expect(formatInputRefWithSubIndex({ family: "decision", key: "PreviousOutcome" }, 0)).toBe(
			"PrevOutcome.energy",
		);
		expect(formatInputRefWithSubIndex({ family: "action_queue", key: "ActionQueue" }, 0)).toBe(
			"Queue1.type",
		);
		expect(formatInputRefWithSubIndex({ family: "action_queue", key: "ActionQueue" }, 4)).toBe(
			"Queue2.param0",
		);
	});

	it("keeps scalars bare and shows an out-of-range sub-index raw", () => {
		expect(formatInputRefWithSubIndex({ family: "dynamic", key: "EnergyCurrent" }, 0)).toBe(
			"Energy",
		);
		expect(formatInputRefWithSubIndex({ family: "world", key: "AreaBarrierSummary" }, 9)).toBe(
			"AreaBarrier[9]",
		);
	});
});

describe("describeInputRef", () => {
	it("explains a sub-field in plain words", () => {
		const d = describeInputRef({ family: "world", key: "AreaBarrierSummary" }, 0);
		expect(d.label).toBe("AreaBarrier.density");
		expect(d.description).toMatch(/barrier/i);
	});

	it("explains every ring direction and nearby rank", () => {
		expect(
			describeInputRef({ family: "world", key: "NeighborOccupiedRing" }, 4).description,
		).toMatch(/south/i);
		expect(describeInputRef({ family: "world", key: "NearbyCreature" }, 10).description).toMatch(
			/2nd-nearest/,
		);
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

describe("directionPairs", () => {
	it("reads every family's pairs from the fixture v3-core is pinned to", () => {
		for (const [key, pairs] of Object.entries(directionPairsFixture)) {
			expect(directionPairs({ family: "world", key: key as InputKey })).toEqual(pairs);
		}
		expect(directionPairs({ family: "world", key: "NearbyCreature" })[1]).toEqual({
			x: 11,
			y: 12,
			facts: [10, 13, 14, 15, 16, 17, 18, 19],
		});
		expect(directionPairs({ family: "world", key: "NeighborBarrierRing" })).toEqual([]);
		expect(directionPairs({ family: "action_queue", key: "ActionQueue" })).toEqual([]);
	});
});
