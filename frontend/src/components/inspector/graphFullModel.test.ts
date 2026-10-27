import { describe, expect, it } from "vitest";
import { graphBrain, ringBundle } from "../../test/brainFixtures.ts";
import type { BrainComputeNode } from "../../types/generated/BrainComputeNode.ts";
import type { BrainEdge } from "../../types/generated/BrainEdge.ts";
import type { BrainSink } from "../../types/generated/BrainSink.ts";
import type { GraphBrain } from "../../types/generated/GraphBrain.ts";
import { buildFullGraphModel } from "./graphFullModel.ts";

function makeGraphDef(overrides: Partial<GraphBrain> = {}): GraphBrain {
	return graphBrain(overrides);
}

function inputEdge(ref_idx: number, sub_idx: number, weight = 1): BrainEdge {
	return {
		source: { InputLeaf: { ref_idx, sub_idx } },
		own: weight,
		genomic: weight,
		applied: weight,
	};
}

function computeEdge(nodeIdx: number, weight = 1): BrainEdge {
	return { source: { ComputeNode: nodeIdx }, own: weight, genomic: weight, applied: weight };
}

describe("buildFullGraphModel edge-bundle members", () => {
	/** A Move bundle member on sink 0 (scale 0.5, correction 0, composed
	 * 0.5) beside an independent edge. */
	function grouped(): GraphBrain {
		return makeGraphDef({
			output_sinks: [
				{
					kind: { ActionVote: { Directed: ["Move", 0] } },
					edges: [
						inputEdge(1, 3, 0.25),
						{
							...inputEdge(0, 0, 0),
							genomic: 0.5,
							applied: 0.5,
							member: {
								bundle: 0,
								scope: "Edge",
								site: { Direction: 0 },
								field: 0,
								pattern: 1,
								scale: 0.5,
							},
						},
					],
				},
			],
			bundles: [ringBundle(0, { scale: 0.5 })],
		});
	}

	it("draws a listed edge at learning's applied weight: a passive member follows the learned scale", () => {
		// Preservation case (T30.F01 invariant 5): a static member of a
		// learning bundle, genomic 0.5, applied 1.5 through the learned scale.
		const model = buildFullGraphModel(
			grouped(),
			[{ family: "world", key: "NeighborBarrierRing" }],
			{
				node_id: 0,
				scales: [{ bundle: 0, learned: 3, initialized: true }],
				edges: [{ surface: "sink", container: 0, edge: 1, applied: 1.5 }],
			},
		);
		const member = model.edges.find((edge) => edge.id === "sink:0:1");
		expect(member?.weight).toBe(1.5);
		expect(member?.genomic).toBe(0.5);
		expect(member?.member?.applied).toBe(1.5);
		expect(member?.isMember).toBe(true);
		const independent = model.edges.find((edge) => edge.id === "sink:0:0");
		expect(independent?.weight).toBe(0.25);
		expect(independent?.isMember).toBe(false);
	});

	it("draws an unlisted edge at the brain's applied weight", () => {
		const model = buildFullGraphModel(grouped(), [{ family: "world", key: "NeighborBarrierRing" }]);
		const member = model.edges.find((edge) => edge.id === "sink:0:1");
		expect(member?.weight).toBe(0.5);
		expect(member?.member?.applied).toBe(0.5);
	});

	it("carries the view's composition for its tooltip", () => {
		const model = buildFullGraphModel(grouped(), [{ family: "world", key: "NeighborBarrierRing" }]);
		const member = model.edges.find((edge) => edge.id === "sink:0:1");
		expect(member?.own).toBe(0);
		expect(member?.genomic).toBe(0.5);
		expect(member?.member).toEqual({
			bundle: 0,
			scope: "Edge",
			scale: 0.5,
			pattern: 1,
			applied: 0.5,
		});
		expect(model.edges.find((edge) => edge.id === "sink:0:0")?.member).toBeUndefined();
	});
});

describe("buildFullGraphModel", () => {
	it("returns empty model for empty graph", () => {
		const model = buildFullGraphModel(makeGraphDef(), []);
		expect(model.nodes).toHaveLength(0);
		expect(model.edges).toHaveLength(0);
	});

	it("deduplicates input sources across compute nodes", () => {
		const cn0: BrainComputeNode = {
			kind: "Add",
			edges: [inputEdge(0, 0), inputEdge(1, 0)],
		};
		const cn1: BrainComputeNode = {
			kind: "Add",
			edges: [inputEdge(0, 0), inputEdge(2, 0)],
		};
		const model = buildFullGraphModel(makeGraphDef({ compute_nodes: [cn0, cn1] }), [
			{ family: "world", key: "FoodHere" },
			{ family: "world", key: "EnergyCurrent" },
			{ family: "static", key: "AgeTicks" },
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
					{ kind: "Add", edges: [] },
					{ kind: "Sigmoid", edges: [] },
					{ kind: { DecayIntegrator: 0.9 }, edges: [] },
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
		const wired: BrainSink = {
			kind: { CustomOutput: 0 },
			edges: [computeEdge(0)],
		};
		const unwired: BrainSink = { kind: { CustomOutput: 1 }, edges: [] };
		const model = buildFullGraphModel(
			makeGraphDef({
				compute_nodes: [{ kind: "Add", edges: [] }],
				output_sinks: [wired, unwired],
			}),
			[],
		);
		const sinkNodes = model.nodes.filter((n) => n.nodeType === "output_sink");
		expect(sinkNodes).toHaveLength(1);
		expect(sinkNodes[0]!.id).toBe("sink:0");
	});

	it("draws wired vote and parameter sinks as action outputs and skips unwired ones", () => {
		const vote: BrainSink = {
			kind: { ActionVote: { Directed: ["Move", 2] } },
			edges: [computeEdge(0)],
		};
		const param: BrainSink = {
			kind: { ActionParam: "ReproduceTransferFraction" },
			edges: [computeEdge(0)],
		};
		const unwired: BrainSink = { kind: { ActionVote: "Eat" }, edges: [] };
		const route: BrainSink = { kind: { RouterGate: 0 }, edges: [computeEdge(0)] };
		const model = buildFullGraphModel(
			makeGraphDef({
				compute_nodes: [{ kind: "Add", edges: [] }],
				output_sinks: [vote, param, unwired, route],
			}),
			[],
		);
		const sinks = model.nodes.filter((n) => n.nodeType === "output_sink");
		expect(sinks.map((n) => [n.id, n.category, n.label])).toEqual([
			["sink:0", "output_action", "Vote Move[E]"],
			["sink:1", "output_action", "Param Reproduce.frac"],
			["sink:3", "output_value", "Route[0]"],
		]);
	});

	it("builds correct edges between layers", () => {
		const model = buildFullGraphModel(
			makeGraphDef({
				compute_nodes: [
					{ kind: "Add", edges: [inputEdge(0, 0)] },
					{ kind: "Add", edges: [computeEdge(0)] },
				],
				output_sinks: [{ kind: { CustomOutput: 0 }, edges: [computeEdge(1)] }],
			}),
			[{ family: "world", key: "FoodHere" }],
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
					{ kind: "Add", edges: [computeEdge(1)] }, // cn:0 reads cn:1 → backward
					{ kind: "Add", edges: [computeEdge(0)] }, // cn:1 reads cn:0 → forward
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
					{ kind: "Add", edges: [computeEdge(5)] }, // index 5 doesn't exist
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
				compute_nodes: [{ kind: "Add", edges: [computeEdge(-1)] }],
			}),
			[],
		);
		expect(model.edges).toHaveLength(0);
	});

	it("preserves SharedMemory input source deduplication", () => {
		const memEdge = (slot: number, prev: boolean): BrainEdge => ({
			source: { SharedMemory: { slot, previous: prev } },
			own: 1,
			genomic: 1,
			applied: 1,
		});
		const model = buildFullGraphModel(
			makeGraphDef({
				compute_nodes: [
					{ kind: "Add", edges: [memEdge(0, false), memEdge(0, false)] },
					{ kind: "Add", edges: [memEdge(0, false)] },
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
