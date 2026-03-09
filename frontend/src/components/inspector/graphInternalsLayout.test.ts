import { describe, expect, it } from "vitest";
import type { GraphInternalNode } from "../../types/genome.ts";
import { type NodeCategory, layoutGraphInternals } from "./graphInternalsLayout.ts";

// Simple 3-node chain: input → processing → output
const threeNodeChain: GraphInternalNode[] = [
	{ kind: { InputRef: { ref_idx: 0, sub_idx: 0 } }, inputs: [] },
	{ kind: { Threshold: 0.5 }, inputs: [{ source_idx: 0, weight: 1 }] },
	{ kind: "RouterOutput", inputs: [{ source_idx: 1, weight: 1 }] },
];

describe("layoutGraphInternals", () => {
	it("returns empty result for empty input", async () => {
		const result = await layoutGraphInternals([]);
		expect(result.nodes).toHaveLength(0);
		expect(result.edges).toHaveLength(0);
		expect(result.width).toBe(0);
		expect(result.height).toBe(0);
	});

	it("lays out nodes and edges for a simple chain", async () => {
		const result = await layoutGraphInternals(threeNodeChain);
		expect(result.nodes).toHaveLength(3);
		expect(result.edges).toHaveLength(2);
		expect(result.width).toBeGreaterThan(0);
		expect(result.height).toBeGreaterThan(0);
	});

	it("uses custom node widths when provided", async () => {
		const widths = [100, 80, 120];
		const result = await layoutGraphInternals(threeNodeChain, widths);
		const nodeByIndex = new Map(result.nodes.map((n) => [n.index, n]));
		expect(nodeByIndex.get(0)?.width).toBe(100);
		expect(nodeByIndex.get(1)?.width).toBe(80);
		expect(nodeByIndex.get(2)?.width).toBe(120);
	});

	it("places input nodes left of output nodes with layer constraints", async () => {
		const categories: NodeCategory[] = ["input", "processing", "output"];
		const result = await layoutGraphInternals(threeNodeChain, undefined, categories);
		const nodeByIndex = new Map(result.nodes.map((n) => [n.index, n]));

		const inputX = nodeByIndex.get(0)!.x;
		const processingX = nodeByIndex.get(1)!.x;
		const outputX = nodeByIndex.get(2)!.x;

		expect(inputX).toBeLessThan(processingX);
		expect(processingX).toBeLessThan(outputX);
	});

	it("aligns multiple inputs in the same leftmost layer", async () => {
		// Two inputs feeding into one output
		const nodes: GraphInternalNode[] = [
			{ kind: { InputRef: { ref_idx: 0, sub_idx: 0 } }, inputs: [] },
			{ kind: { InputRef: { ref_idx: 1, sub_idx: 0 } }, inputs: [] },
			{
				kind: "RouterOutput",
				inputs: [
					{ source_idx: 0, weight: 1 },
					{ source_idx: 1, weight: 1 },
				],
			},
		];
		const categories: NodeCategory[] = ["input", "input", "output"];
		const result = await layoutGraphInternals(nodes, undefined, categories);
		const nodeByIndex = new Map(result.nodes.map((n) => [n.index, n]));

		const input0X = nodeByIndex.get(0)!.x;
		const input1X = nodeByIndex.get(1)!.x;
		const outputX = nodeByIndex.get(2)!.x;

		// Both inputs should be in the same layer (same x)
		expect(input0X).toBe(input1X);
		// Inputs should be left of output
		expect(input0X).toBeLessThan(outputX);
	});

	it("aligns multiple outputs in the same rightmost layer", async () => {
		// One input feeding two outputs
		const nodes: GraphInternalNode[] = [
			{ kind: { InputRef: { ref_idx: 0, sub_idx: 0 } }, inputs: [] },
			{ kind: "RouterOutput", inputs: [{ source_idx: 0, weight: 1 }] },
			{ kind: { CustomOutput: 0 }, inputs: [{ source_idx: 0, weight: 1 }] },
		];
		const categories: NodeCategory[] = ["input", "output", "output"];
		const result = await layoutGraphInternals(nodes, undefined, categories);
		const nodeByIndex = new Map(result.nodes.map((n) => [n.index, n]));

		const inputX = nodeByIndex.get(0)!.x;
		const out0X = nodeByIndex.get(1)!.x;
		const out1X = nodeByIndex.get(2)!.x;

		// Both outputs should be in the same layer (same x)
		expect(out0X).toBe(out1X);
		// Outputs should be right of input
		expect(inputX).toBeLessThan(out0X);
	});

	it("places disconnected output nodes in the rightmost layer", async () => {
		// Input → Output chain, plus a disconnected output (no edges)
		const nodes: GraphInternalNode[] = [
			{ kind: { InputRef: { ref_idx: 0, sub_idx: 0 } }, inputs: [] },
			{ kind: { CustomOutput: 0 }, inputs: [{ source_idx: 0, weight: 1 }] },
			{ kind: "RouterOutput", inputs: [] }, // disconnected
		];
		const categories: NodeCategory[] = ["input", "output", "output"];
		const result = await layoutGraphInternals(nodes, undefined, categories);
		const nodeByIndex = new Map(result.nodes.map((n) => [n.index, n]));

		const inputX = nodeByIndex.get(0)!.x;
		const connectedOutX = nodeByIndex.get(1)!.x;
		const disconnectedOutX = nodeByIndex.get(2)!.x;

		// Both outputs should be in the same rightmost layer
		expect(connectedOutX).toBe(disconnectedOutX);
		// And right of input
		expect(inputX).toBeLessThan(disconnectedOutX);
	});

	it("builds correct edge data with weights", async () => {
		const nodes: GraphInternalNode[] = [
			{ kind: { InputRef: { ref_idx: 0, sub_idx: 0 } }, inputs: [] },
			{ kind: "RouterOutput", inputs: [{ source_idx: 0, weight: 0.75 }] },
		];
		const result = await layoutGraphInternals(nodes);
		expect(result.edges).toHaveLength(1);
		expect(result.edges[0]?.fromIndex).toBe(0);
		expect(result.edges[0]?.toIndex).toBe(1);
		expect(result.edges[0]?.weight).toBe(0.75);
		expect(result.edges[0]?.isBackward).toBe(false);
	});

	it("marks forward edge as not backward", async () => {
		const nodes: GraphInternalNode[] = [
			{ kind: { InputRef: { ref_idx: 0, sub_idx: 0 } }, inputs: [] },
			{ kind: { Threshold: 0.5 }, inputs: [{ source_idx: 0, weight: 1.0 }] },
		];
		const result = await layoutGraphInternals(nodes);
		expect(result.edges).toHaveLength(1);
		expect(result.edges[0]?.isBackward).toBe(false);
	});

	it("marks backward edge as backward", async () => {
		const nodes: GraphInternalNode[] = [
			{ kind: { InputRef: { ref_idx: 0, sub_idx: 0 } }, inputs: [] },
			{ kind: { Threshold: 0.5 }, inputs: [{ source_idx: 2, weight: 0.5 }] },
			{ kind: "RouterOutput", inputs: [{ source_idx: 1, weight: 1.0 }] },
		];
		const result = await layoutGraphInternals(nodes);
		const backwardEdge = result.edges.find((e) => e.fromIndex === 2 && e.toIndex === 1);
		expect(backwardEdge).toBeDefined();
		expect(backwardEdge?.isBackward).toBe(true);
	});

	it("marks self-loop edge as backward", async () => {
		const nodes: GraphInternalNode[] = [
			{ kind: { Threshold: 0.5 }, inputs: [{ source_idx: 0, weight: 0.8 }] },
		];
		const result = await layoutGraphInternals(nodes);
		expect(result.edges).toHaveLength(1);
		expect(result.edges[0]?.fromIndex).toBe(0);
		expect(result.edges[0]?.toIndex).toBe(0);
		expect(result.edges[0]?.isBackward).toBe(true);
	});
});
