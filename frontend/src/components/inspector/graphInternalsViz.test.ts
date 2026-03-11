import { describe, expect, it } from "vitest";
import type { InputReference } from "../../types/genome.ts";
import {
	categorize,
	getKindLabel,
	getKindName,
	resolveSelectedTarget,
} from "./GraphInternalsViz.tsx";

describe("getKindName", () => {
	it("returns string kinds directly", () => {
		expect(getKindName("Add")).toBe("Add");
		expect(getKindName("Multiply")).toBe("Multiply");
		expect(getKindName("Negate")).toBe("Negate");
		expect(getKindName("Abs")).toBe("Abs");
		expect(getKindName("Min")).toBe("Min");
		expect(getKindName("Max")).toBe("Max");
		expect(getKindName("WeightedSum")).toBe("WeightedSum");
		expect(getKindName("Sigmoid")).toBe("Sigmoid");
		expect(getKindName("Tanh")).toBe("Tanh");
		expect(getKindName("Relu")).toBe("Relu");
		expect(getKindName("Clamp01")).toBe("Clamp01");
		expect(getKindName("GreaterThan")).toBe("GreaterThan");
		expect(getKindName("Select")).toBe("Select");
		expect(getKindName("AdaptiveGain")).toBe("AdaptiveGain");
	});

	it("extracts name from object kinds", () => {
		expect(getKindName({ Constant: 1.5 })).toBe("Constant");
		expect(getKindName({ Threshold: 0.5 })).toBe("Threshold");
		expect(getKindName({ DecayIntegrator: 0.9 })).toBe("DecayIntegrator");
		expect(getKindName({ Momentum: 0.8 })).toBe("Momentum");
		expect(getKindName({ Oscillator: 0.5 })).toBe("Oscillator");
	});
});

describe("getKindLabel", () => {
	const inputRefs: InputReference[] = [
		{ World: "NeighborFoodRing" },
		{ World: "FoodHere" },
		{ UpstreamSlot: 0 },
	];

	it("returns friendly names for string kinds", () => {
		expect(getKindLabel("WeightedSum", inputRefs)).toBe("\u03A3 Weight");
		expect(getKindLabel("Clamp01", inputRefs)).toBe("Clamp");
		expect(getKindLabel("GreaterThan", inputRefs)).toBe("GT");
	});

	it("passes through string kinds without special labels", () => {
		expect(getKindLabel("Add", inputRefs)).toBe("Add");
		expect(getKindLabel("Sigmoid", inputRefs)).toBe("Sigmoid");
		expect(getKindLabel("Multiply", inputRefs)).toBe("Multiply");
		expect(getKindLabel("Negate", inputRefs)).toBe("Negate");
		expect(getKindLabel("Abs", inputRefs)).toBe("Abs");
		expect(getKindLabel("Min", inputRefs)).toBe("Min");
		expect(getKindLabel("Max", inputRefs)).toBe("Max");
		expect(getKindLabel("Tanh", inputRefs)).toBe("Tanh");
		expect(getKindLabel("Relu", inputRefs)).toBe("Relu");
		expect(getKindLabel("Select", inputRefs)).toBe("Select");
		expect(getKindLabel("AdaptiveGain", inputRefs)).toBe("AdaptiveGain");
	});

	it("formats object kinds with numeric values", () => {
		expect(getKindLabel({ Constant: 1.5 }, inputRefs)).toBe("Const(1.50)");
		expect(getKindLabel({ Threshold: 0.25 }, inputRefs)).toBe("Thresh(0.25)");
		expect(getKindLabel({ DecayIntegrator: 0.9 }, inputRefs)).toBe("Decay(0.90)");
		expect(getKindLabel({ Momentum: 0.8 }, inputRefs)).toBe("Momentum(0.80)");
		expect(getKindLabel({ Oscillator: 0.5 }, inputRefs)).toBe("Osc(0.50)");
	});
});

describe("categorize", () => {
	it("classifies Constant as constant", () => {
		expect(categorize({ Constant: 1 })).toBe("constant");
		expect(categorize({ Constant: 0 })).toBe("constant");
		expect(categorize({ Constant: -3.14 })).toBe("constant");
	});

	it("classifies all string kinds as processing", () => {
		expect(categorize("Add")).toBe("processing");
		expect(categorize("Multiply")).toBe("processing");
		expect(categorize("Negate")).toBe("processing");
		expect(categorize("Abs")).toBe("processing");
		expect(categorize("Min")).toBe("processing");
		expect(categorize("Max")).toBe("processing");
		expect(categorize("WeightedSum")).toBe("processing");
		expect(categorize("Sigmoid")).toBe("processing");
		expect(categorize("Tanh")).toBe("processing");
		expect(categorize("Relu")).toBe("processing");
		expect(categorize("Clamp01")).toBe("processing");
		expect(categorize("GreaterThan")).toBe("processing");
		expect(categorize("Select")).toBe("processing");
		expect(categorize("AdaptiveGain")).toBe("processing");
	});

	it("classifies parameterized non-Constant kinds as processing", () => {
		expect(categorize({ Threshold: 0.5 })).toBe("processing");
		expect(categorize({ DecayIntegrator: 0.9 })).toBe("processing");
		expect(categorize({ Momentum: 0.8 })).toBe("processing");
		expect(categorize({ Oscillator: 0.5 })).toBe("processing");
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
		// NaN -> idx = -1, rem_euclid(3) = 2
		expect(resolveSelectedTarget(Number.NaN, targets)).toBe(7);
	});

	it("works with single target", () => {
		expect(resolveSelectedTarget(0.0, [42])).toBe(42);
		expect(resolveSelectedTarget(5.0, [42])).toBe(42);
		expect(resolveSelectedTarget(-3.0, [42])).toBe(42);
	});
});
