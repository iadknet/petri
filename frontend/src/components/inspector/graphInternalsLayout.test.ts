import { describe, expect, it } from "vitest";
import type { ComputeNode } from "../../types/genome.ts";
import { type NodeCategory, layoutGraphInternals } from "./graphInternalsLayout.ts";

// Simple 3-node chain: constant → processing → processing
const threeNodeChain: ComputeNode[] = [
	{ kind: { Constant: 1.0 }, inputs: [] },
	{ kind: { Threshold: 0.5 }, inputs: [{ source: { ComputeNode: 0 }, weight: 1 }] },
	{ kind: "Add", inputs: [{ source: { ComputeNode: 1 }, weight: 1 }] },
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

	it("places constant nodes left of processing nodes with layer constraints", async () => {
		const categories: NodeCategory[] = ["constant", "processing", "processing"];
		const result = await layoutGraphInternals(threeNodeChain, undefined, categories);
		const nodeByIndex = new Map(result.nodes.map((n) => [n.index, n]));

		const constantX = nodeByIndex.get(0)!.x;
		const proc1X = nodeByIndex.get(1)!.x;
		const proc2X = nodeByIndex.get(2)!.x;

		expect(constantX).toBeLessThan(proc1X);
		expect(proc1X).toBeLessThan(proc2X);
	});

	it("aligns multiple constants in the same leftmost layer", async () => {
		// Two constants feeding into one processing node
		const nodes: ComputeNode[] = [
			{ kind: { Constant: 1.0 }, inputs: [] },
			{ kind: { Constant: 2.0 }, inputs: [] },
			{
				kind: "Add",
				inputs: [
					{ source: { ComputeNode: 0 }, weight: 1 },
					{ source: { ComputeNode: 1 }, weight: 1 },
				],
			},
		];
		const categories: NodeCategory[] = ["constant", "constant", "processing"];
		const result = await layoutGraphInternals(nodes, undefined, categories);
		const nodeByIndex = new Map(result.nodes.map((n) => [n.index, n]));

		const const0X = nodeByIndex.get(0)!.x;
		const const1X = nodeByIndex.get(1)!.x;
		const procX = nodeByIndex.get(2)!.x;

		// Both constants should be in the same layer (same x)
		expect(const0X).toBe(const1X);
		// Constants should be left of processing
		expect(const0X).toBeLessThan(procX);
	});

	it("ignores InputLeaf and SharedMemory sources in layout edges", async () => {
		// A processing node with one InputLeaf source and one ComputeNode source
		const nodes: ComputeNode[] = [
			{ kind: { Constant: 1.0 }, inputs: [] },
			{
				kind: "Add",
				inputs: [
					{ source: { InputLeaf: { ref_idx: 0, sub_idx: 0 } }, weight: 0.5 },
					{ source: { ComputeNode: 0 }, weight: 1.0 },
					{ source: { SharedMemory: { slot: 0, previous: false } }, weight: 0.3 },
				],
			},
		];
		const result = await layoutGraphInternals(nodes);
		// Only the ComputeNode edge should appear in layout
		expect(result.edges).toHaveLength(1);
		expect(result.edges[0]?.fromIndex).toBe(0);
		expect(result.edges[0]?.toIndex).toBe(1);
		expect(result.edges[0]?.weight).toBe(1.0);
	});

	it("builds correct edge data with weights", async () => {
		const nodes: ComputeNode[] = [
			{ kind: { Constant: 1.0 }, inputs: [] },
			{ kind: "Add", inputs: [{ source: { ComputeNode: 0 }, weight: 0.75 }] },
		];
		const result = await layoutGraphInternals(nodes);
		expect(result.edges).toHaveLength(1);
		expect(result.edges[0]?.fromIndex).toBe(0);
		expect(result.edges[0]?.toIndex).toBe(1);
		expect(result.edges[0]?.weight).toBe(0.75);
		expect(result.edges[0]?.isBackward).toBe(false);
	});

	it("marks forward edge as not backward", async () => {
		const nodes: ComputeNode[] = [
			{ kind: { Constant: 1.0 }, inputs: [] },
			{ kind: { Threshold: 0.5 }, inputs: [{ source: { ComputeNode: 0 }, weight: 1.0 }] },
		];
		const result = await layoutGraphInternals(nodes);
		expect(result.edges).toHaveLength(1);
		expect(result.edges[0]?.isBackward).toBe(false);
	});

	it("marks backward edge as backward", async () => {
		const nodes: ComputeNode[] = [
			{ kind: { Constant: 1.0 }, inputs: [] },
			{ kind: { Threshold: 0.5 }, inputs: [{ source: { ComputeNode: 2 }, weight: 0.5 }] },
			{ kind: "Add", inputs: [{ source: { ComputeNode: 1 }, weight: 1.0 }] },
		];
		const result = await layoutGraphInternals(nodes);
		const backwardEdge = result.edges.find((e) => e.fromIndex === 2 && e.toIndex === 1);
		expect(backwardEdge).toBeDefined();
		expect(backwardEdge?.isBackward).toBe(true);
	});

	it("marks self-loop edge as backward", async () => {
		const nodes: ComputeNode[] = [
			{ kind: { Threshold: 0.5 }, inputs: [{ source: { ComputeNode: 0 }, weight: 0.8 }] },
		];
		const result = await layoutGraphInternals(nodes);
		expect(result.edges).toHaveLength(1);
		expect(result.edges[0]?.fromIndex).toBe(0);
		expect(result.edges[0]?.toIndex).toBe(0);
		expect(result.edges[0]?.isBackward).toBe(true);
	});
});
