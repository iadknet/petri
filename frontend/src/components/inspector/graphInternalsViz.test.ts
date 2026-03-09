import { describe, expect, it } from "vitest";
import type { GraphNodeKind, InputReference } from "../../types/genome.ts";
import {
	categorize,
	getKindLabel,
	getKindName,
	resolveSelectedTarget,
} from "./GraphInternalsViz.tsx";

describe("getKindName", () => {
	it("returns string kinds directly", () => {
		expect(getKindName("RouterOutput")).toBe("RouterOutput");
		expect(getKindName("Add")).toBe("Add");
	});

	it("extracts name from object kinds", () => {
		expect(getKindName({ InputRef: { ref_idx: 0, sub_idx: 0 } })).toBe("InputRef");
		expect(getKindName({ Constant: 1.5 })).toBe("Constant");
		expect(getKindName({ Threshold: 0.5 })).toBe("Threshold");
	});
});

describe("getKindLabel", () => {
	const inputRefs: InputReference[] = [
		{ World: "NeighborFoodRing" },
		{ World: "FoodHere" },
		{ UpstreamSlot: 0 },
	];

	it("resolves InputRef ref_idx to sensor name via inputRefs", () => {
		expect(getKindLabel({ InputRef: { ref_idx: 0, sub_idx: 0 } }, inputRefs)).toBe("FoodRing[N]");
		expect(getKindLabel({ InputRef: { ref_idx: 1, sub_idx: 0 } }, inputRefs)).toBe("FoodHere");
		expect(getKindLabel({ InputRef: { ref_idx: 2, sub_idx: 0 } }, inputRefs)).toBe("slot[0]");
	});

	it("shows direction names for ring sensor sub_idx", () => {
		expect(getKindLabel({ InputRef: { ref_idx: 0, sub_idx: 2 } }, inputRefs)).toBe("FoodRing[E]");
		expect(getKindLabel({ InputRef: { ref_idx: 0, sub_idx: 4 } }, inputRefs)).toBe("FoodRing[S]");
	});

	it("appends sub_idx when non-zero for non-ring refs", () => {
		expect(getKindLabel({ InputRef: { ref_idx: 1, sub_idx: 1 } }, inputRefs)).toBe("FoodHere[1]");
	});

	it("falls back for out-of-bounds ref_idx", () => {
		expect(getKindLabel({ InputRef: { ref_idx: 99, sub_idx: 0 } }, inputRefs)).toBe("In(99)");
	});

	it("falls back with sub_idx for out-of-bounds ref_idx", () => {
		expect(getKindLabel({ InputRef: { ref_idx: 99, sub_idx: 3 } }, inputRefs)).toBe("In(99)[3]");
	});

	it("shows 'Dead' for sentinel ref_idx (u16::MAX = 65535)", () => {
		expect(getKindLabel({ InputRef: { ref_idx: 0xffff, sub_idx: 0 } }, inputRefs)).toBe("Dead");
	});

	it("shows 'Dead' with sub_idx for sentinel ref_idx", () => {
		expect(getKindLabel({ InputRef: { ref_idx: 0xffff, sub_idx: 6 } }, inputRefs)).toBe("Dead[6]");
	});

	it("resolves ring sensor variants with direction names", () => {
		const foodRefs: InputReference[] = [{ World: "NeighborFoodRing" }];
		expect(getKindLabel({ InputRef: { ref_idx: 0, sub_idx: 1 } }, foodRefs)).toBe("FoodRing[NE]");

		const barrierRefs: InputReference[] = [{ World: "NeighborBarrierRing" }];
		expect(getKindLabel({ InputRef: { ref_idx: 0, sub_idx: 4 } }, barrierRefs)).toBe(
			"BarrierRing[S]",
		);

		const occRefs: InputReference[] = [{ World: "NeighborOccupiedRing" }];
		expect(getKindLabel({ InputRef: { ref_idx: 0, sub_idx: 2 } }, occRefs)).toBe("OccRing[E]");
	});

	it("resolves StaticIntrospection sensor", () => {
		const refs: InputReference[] = [{ StaticIntrospection: "NodeCount" }];
		expect(getKindLabel({ InputRef: { ref_idx: 0, sub_idx: 0 } }, refs)).toBe("NodeCount");
	});

	it("resolves DynamicIntrospection sensor", () => {
		const refs: InputReference[] = [{ DynamicIntrospection: "EnergyRatio" }];
		expect(getKindLabel({ InputRef: { ref_idx: 0, sub_idx: 0 } }, refs)).toBe("EnergyRatio");
	});

	it("resolves ActionQueue string ref", () => {
		const refs: InputReference[] = ["ActionQueue"];
		expect(getKindLabel({ InputRef: { ref_idx: 0, sub_idx: 0 } }, refs)).toBe("ActionQueue");
	});

	it("returns friendly names for string kinds", () => {
		expect(getKindLabel("RouterOutput", [])).toBe("Route Out");
		expect(getKindLabel("WeightedSum", [])).toBe("Σ Weight");
		expect(getKindLabel("Clamp01", [])).toBe("Clamp");
		expect(getKindLabel("GreaterThan", [])).toBe("GT");
		expect(getKindLabel("ExecuteActionQueue", [])).toBe("ExecQueue");
		expect(getKindLabel("PopAction", [])).toBe("PopAct");
	});

	it("passes through unknown string kinds", () => {
		expect(getKindLabel("Add" as GraphNodeKind, [])).toBe("Add");
		expect(getKindLabel("Sigmoid" as GraphNodeKind, [])).toBe("Sigmoid");
	});

	it("formats object kinds with numeric values", () => {
		expect(getKindLabel({ Constant: 1.5 }, [])).toBe("Const(1.50)");
		expect(getKindLabel({ Threshold: 0.25 }, [])).toBe("Thresh(0.25)");
		expect(getKindLabel({ DecayIntegrator: 0.9 }, [])).toBe("Decay(0.90)");
		expect(getKindLabel({ Momentum: 0.8 }, [])).toBe("Momentum(0.80)");
		expect(getKindLabel({ Oscillator: 0.5 }, [])).toBe("Osc(0.50)");
	});

	it("formats slot and output kinds", () => {
		expect(getKindLabel({ CustomOutput: 2 }, [])).toBe("Out[2]");
		expect(getKindLabel({ WriteSlot: 0 }, [])).toBe("Write s[0]");
		expect(getKindLabel({ ClearSlot: 1 }, [])).toBe("Clear s[1]");
		expect(getKindLabel({ ReadSlot: 0 }, [])).toBe("Read s[0]");
		expect(getKindLabel({ ReadSlotPrev: 1 }, [])).toBe("Prev s[1]");
	});
});

describe("categorize", () => {
	it("classifies input kinds", () => {
		expect(categorize({ InputRef: { ref_idx: 0, sub_idx: 0 } })).toBe("input");
	});

	it("classifies constant kinds", () => {
		expect(categorize({ Constant: 1 })).toBe("constant");
	});

	it("classifies memory kinds", () => {
		expect(categorize({ ReadSlot: 0 })).toBe("memory");
		expect(categorize({ ReadSlotPrev: 0 })).toBe("memory");
	});

	it("classifies output kinds", () => {
		expect(categorize("RouterOutput")).toBe("output");
		expect(categorize({ CustomOutput: 0 })).toBe("output");
		expect(categorize({ WriteSlot: 0 })).toBe("output");
		expect(categorize({ ClearSlot: 0 })).toBe("output");
		expect(categorize({ PushAction: 0 })).toBe("output");
		expect(categorize("ExecuteActionQueue")).toBe("output");
	});

	it("classifies processing kinds", () => {
		expect(categorize("Add")).toBe("processing");
		expect(categorize("Multiply")).toBe("processing");
		expect(categorize("Sigmoid")).toBe("processing");
		expect(categorize({ Threshold: 0.5 })).toBe("processing");
		expect(categorize({ DecayIntegrator: 0.9 })).toBe("processing");
	});
});

describe("resolveSelectedTarget", () => {
	const targets = [3, 5, 7];

	it("selects target at floor(routeTargetIdx) mod targets.length", () => {
		expect(resolveSelectedTarget(0.0, targets)).toBe(3); // floor(0) % 3 = 0
		expect(resolveSelectedTarget(1.0, targets)).toBe(5); // floor(1) % 3 = 1
		expect(resolveSelectedTarget(2.0, targets)).toBe(7); // floor(2) % 3 = 2
	});

	it("wraps around with rem_euclid for values >= targets.length", () => {
		expect(resolveSelectedTarget(3.0, targets)).toBe(3); // floor(3) % 3 = 0
		expect(resolveSelectedTarget(3.7, targets)).toBe(3); // floor(3.7)=3, 3 % 3 = 0
		expect(resolveSelectedTarget(5.9, targets)).toBe(7); // floor(5.9)=5, 5 % 3 = 2
	});

	it("wraps negative values via rem_euclid", () => {
		expect(resolveSelectedTarget(-1.0, targets)).toBe(7); // floor(-1)=-1, rem_euclid(3)=2
		expect(resolveSelectedTarget(-2.0, targets)).toBe(5); // floor(-2)=-2, rem_euclid(3)=1
	});

	it("returns null for empty targets", () => {
		expect(resolveSelectedTarget(0.0, [])).toBeNull();
	});

	it("handles NaN route value", () => {
		// NaN → idx = -1, rem_euclid(3) = 2
		expect(resolveSelectedTarget(Number.NaN, targets)).toBe(7);
	});

	it("works with single target", () => {
		expect(resolveSelectedTarget(0.0, [42])).toBe(42);
		expect(resolveSelectedTarget(5.0, [42])).toBe(42);
		expect(resolveSelectedTarget(-3.0, [42])).toBe(42);
	});
});
