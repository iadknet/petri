import { describe, expect, it } from "vitest";
import type { ComputeNode, GraphBackendDef, GraphEdge, OutputSink } from "../../types/genome.ts";
import { buildFullGraphModel } from "./graphFullModel.ts";

function makeGraphDef(overrides: Partial<GraphBackendDef> = {}): GraphBackendDef {
	return {
		compute_nodes: [],
		output_sinks: [],
		...overrides,
	};
}

function inputEdge(ref_idx: number, sub_idx: number, weight = 1): GraphEdge {
	return { source: { InputLeaf: { ref_idx, sub_idx } }, weight };
}

function computeEdge(nodeIdx: number, weight = 1): GraphEdge {
	return { source: { ComputeNode: nodeIdx }, weight };
}

describe("buildFullGraphModel", () => {
	it("returns empty model for empty graph", () => {
		const model = buildFullGraphModel(makeGraphDef(), []);
		expect(model.nodes).toHaveLength(0);
		expect(model.edges).toHaveLength(0);
	});

	it("deduplicates input sources across compute nodes", () => {
		const cn0: ComputeNode = {
			kind: "Add",
			inputs: [inputEdge(0, 0), inputEdge(1, 0)],
		};
		const cn1: ComputeNode = {
			kind: "Add",
			inputs: [inputEdge(0, 0), inputEdge(2, 0)],
		};
		const model = buildFullGraphModel(makeGraphDef({ compute_nodes: [cn0, cn1] }), [
			{ World: "FoodHere" },
			{ World: "EnergyLevel" },
			{ StaticIntrospection: "Age" },
		]);
		const inputNodes = model.nodes.filter((n) => n.nodeType === "input");
		// ref 0 appears in both cn0 and cn1 but should only produce one node
		expect(inputNodes).toHaveLength(3);
		const ids = inputNodes.map((n) => n.id);
		expect(ids).toContain("in:leaf:0:0");
		expect(ids).toContain("in:leaf:1:0");
		expect(ids).toContain("in:leaf:2:0");
	});

	it("creates compute nodes with correct IDs and categories", () => {
		const model = buildFullGraphModel(
			makeGraphDef({
				compute_nodes: [
					{ kind: "Add", inputs: [] },
					{ kind: "Sigmoid", inputs: [] },
					{ kind: { DecayIntegrator: 0.9 }, inputs: [] },
				],
			}),
			[],
		);
		const computeNodes = model.nodes.filter((n) => n.nodeType === "compute");
		expect(computeNodes).toHaveLength(3);
		expect(computeNodes[0]!.id).toBe("cn:0");
		expect(computeNodes[0]!.category).toBe("arithmetic");
		expect(computeNodes[1]!.id).toBe("cn:1");
		expect(computeNodes[1]!.category).toBe("activation");
		expect(computeNodes[2]!.id).toBe("cn:2");
		expect(computeNodes[2]!.category).toBe("stateful");
	});

	it("creates output sink nodes only for wired sinks", () => {
		const wired: OutputSink = {
			kind: { CustomOutput: 0 },
			inputs: [computeEdge(0)],
		};
		const unwired: OutputSink = { kind: { CustomOutput: 1 }, inputs: [] };
		const model = buildFullGraphModel(
			makeGraphDef({
				compute_nodes: [{ kind: "Add", inputs: [] }],
				output_sinks: [wired, unwired],
			}),
			[],
		);
		const sinkNodes = model.nodes.filter((n) => n.nodeType === "output_sink");
		expect(sinkNodes).toHaveLength(1);
		expect(sinkNodes[0]!.id).toBe("sink:0");
	});

	it("draws wired vote and parameter sinks as action outputs and skips unwired ones", () => {
		const vote: OutputSink = { kind: { ActionVote: { Move: 2 } }, inputs: [computeEdge(0)] };
		const param: OutputSink = { kind: { ActionParam: ["Reproduce", 1] }, inputs: [computeEdge(0)] };
		const unwired: OutputSink = { kind: { ActionVote: "Eat" }, inputs: [] };
		const route: OutputSink = { kind: { RouterGate: 0 }, inputs: [computeEdge(0)] };
		const model = buildFullGraphModel(
			makeGraphDef({
				compute_nodes: [{ kind: "Add", inputs: [] }],
				output_sinks: [vote, param, unwired, route],
			}),
			[],
		);
		const sinks = model.nodes.filter((n) => n.nodeType === "output_sink");
		expect(sinks.map((n) => [n.id, n.category, n.label])).toEqual([
			["sink:0", "output_action", "Vote Move[2]"],
			["sink:1", "output_action", "Param Reproduce[1]"],
			["sink:3", "output_value", "Route[0]"],
		]);
	});

	it("builds correct edges between layers", () => {
		const model = buildFullGraphModel(
			makeGraphDef({
				compute_nodes: [
					{ kind: "Add", inputs: [inputEdge(0, 0)] },
					{ kind: "Add", inputs: [computeEdge(0)] },
				],
				output_sinks: [{ kind: { CustomOutput: 0 }, inputs: [computeEdge(1)] }],
			}),
			[{ World: "FoodHere" }],
		);
		expect(model.edges).toHaveLength(3);

		const inputToCompute = model.edges.filter((e) => e.edgeType === "input_to_compute");
		expect(inputToCompute).toHaveLength(1);
		expect(inputToCompute[0]!.sourceId).toBe("in:leaf:0:0");
		expect(inputToCompute[0]!.targetId).toBe("cn:0");

		const c2c = model.edges.filter((e) => e.edgeType === "compute_to_compute");
		expect(c2c).toHaveLength(1);
		expect(c2c[0]!.sourceId).toBe("cn:0");
		expect(c2c[0]!.targetId).toBe("cn:1");
		expect(c2c[0]!.isBackward).toBe(false);

		const c2o = model.edges.filter((e) => e.edgeType === "compute_to_output");
		expect(c2o).toHaveLength(1);
		expect(c2o[0]!.sourceId).toBe("cn:1");
		expect(c2o[0]!.targetId).toBe("sink:0");
	});

	it("marks backward edges correctly", () => {
		const model = buildFullGraphModel(
			makeGraphDef({
				compute_nodes: [
					{ kind: "Add", inputs: [computeEdge(1)] }, // cn:0 reads cn:1 → backward
					{ kind: "Add", inputs: [computeEdge(0)] }, // cn:1 reads cn:0 → forward
				],
			}),
			[],
		);
		const c2c = model.edges.filter((e) => e.edgeType === "compute_to_compute");
		expect(c2c).toHaveLength(2);
		const backEdge = c2c.find((e) => e.targetId === "cn:0")!;
		const fwdEdge = c2c.find((e) => e.targetId === "cn:1")!;
		expect(backEdge.isBackward).toBe(true);
		expect(fwdEdge.isBackward).toBe(false);
	});

	it("drops edges with out-of-range compute node source", () => {
		const model = buildFullGraphModel(
			makeGraphDef({
				compute_nodes: [
					{ kind: "Add", inputs: [computeEdge(5)] }, // index 5 doesn't exist
				],
			}),
			[],
		);
		// The dangling edge should be silently dropped
		expect(model.edges).toHaveLength(0);
	});

	it("drops edges with negative compute node source", () => {
		const model = buildFullGraphModel(
			makeGraphDef({
				compute_nodes: [{ kind: "Add", inputs: [computeEdge(-1)] }],
			}),
			[],
		);
		expect(model.edges).toHaveLength(0);
	});

	it("preserves SharedMemory input source deduplication", () => {
		const memEdge = (slot: number, prev: boolean): GraphEdge => ({
			source: { SharedMemory: { slot, previous: prev } },
			weight: 1,
		});
		const model = buildFullGraphModel(
			makeGraphDef({
				compute_nodes: [
					{ kind: "Add", inputs: [memEdge(0, false), memEdge(0, false)] },
					{ kind: "Add", inputs: [memEdge(0, false)] },
				],
			}),
			[],
		);
		const memNodes = model.nodes.filter(
			(n) => n.nodeType === "input" && n.id.startsWith("in:mem:"),
		);
		// Same slot+previous should deduplicate to one node
		expect(memNodes).toHaveLength(1);
		expect(memNodes[0]!.id).toBe("in:mem:0:false");
	});
});
