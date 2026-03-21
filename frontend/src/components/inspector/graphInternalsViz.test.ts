import { describe, expect, it } from "vitest";
import type { InputReference } from "../../types/genome.ts";
import { resolveSelectedTarget } from "./GraphInternalsViz.tsx";
import { categorizeComputeNode } from "./graphNodeCategories.ts";
import { getKindLabel, getKindName } from "./graphNodeFormatters.ts";

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

describe("categorizeComputeNode", () => {
	it("classifies Constant as constant", () => {
		expect(categorizeComputeNode({ Constant: 1 })).toBe("constant");
		expect(categorizeComputeNode({ Constant: 0 })).toBe("constant");
		expect(categorizeComputeNode({ Constant: -3.14 })).toBe("constant");
	});

	it("classifies arithmetic kinds", () => {
		expect(categorizeComputeNode("Add")).toBe("arithmetic");
		expect(categorizeComputeNode("Multiply")).toBe("arithmetic");
		expect(categorizeComputeNode("Negate")).toBe("arithmetic");
		expect(categorizeComputeNode("Abs")).toBe("arithmetic");
		expect(categorizeComputeNode("Min")).toBe("arithmetic");
		expect(categorizeComputeNode("Max")).toBe("arithmetic");
		expect(categorizeComputeNode("WeightedSum")).toBe("arithmetic");
	});

	it("classifies activation kinds", () => {
		expect(categorizeComputeNode("Sigmoid")).toBe("activation");
		expect(categorizeComputeNode("Tanh")).toBe("activation");
		expect(categorizeComputeNode("Relu")).toBe("activation");
		expect(categorizeComputeNode("Clamp01")).toBe("activation");
		expect(categorizeComputeNode({ Threshold: 0.5 })).toBe("activation");
	});

	it("classifies logic kinds", () => {
		expect(categorizeComputeNode("GreaterThan")).toBe("logic");
		expect(categorizeComputeNode("Select")).toBe("logic");
	});

	it("classifies stateful kinds", () => {
		expect(categorizeComputeNode({ DecayIntegrator: 0.9 })).toBe("stateful");
		expect(categorizeComputeNode({ Momentum: 0.8 })).toBe("stateful");
		expect(categorizeComputeNode({ Oscillator: 0.5 })).toBe("stateful");
		expect(categorizeComputeNode("AdaptiveGain")).toBe("stateful");
	});
});

describe("resolveSelectedTarget", () => {
	const targets = [
		{ target_id: 3, slot: 0, gate_bias: 0.0 },
		{ target_id: 5, slot: 1, gate_bias: 0.0 },
		{ target_id: 7, slot: 2, gate_bias: 0.0 },
	];

	it("selects target_id at the given index", () => {
		expect(resolveSelectedTarget(0, targets)).toBe(3);
		expect(resolveSelectedTarget(1, targets)).toBe(5);
		expect(resolveSelectedTarget(2, targets)).toBe(7);
	});

	it("returns null for out-of-bounds index", () => {
		expect(resolveSelectedTarget(3, targets)).toBeNull();
		expect(resolveSelectedTarget(-1, targets)).toBeNull();
	});

	it("returns null for empty targets", () => {
		expect(resolveSelectedTarget(0, [])).toBeNull();
	});

	it("works with single target", () => {
		const single = [{ target_id: 42, slot: 0, gate_bias: 0.0 }];
		expect(resolveSelectedTarget(0, single)).toBe(42);
		expect(resolveSelectedTarget(1, single)).toBeNull();
	});
});
