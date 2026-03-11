import { describe, expect, it } from "vitest";
import type { LayoutInputEdge, LayoutInputNode } from "./graphInternalsLayout.ts";
import { layoutGraphInternals } from "./graphInternalsLayout.ts";

// Simple 3-node chain: n0 → n1 → n2
const threeNodeChain: LayoutInputNode[] = [
	{ id: "n0", width: 72, height: 24, layerConstraint: "FIRST" },
	{ id: "n1", width: 72, height: 24 },
	{ id: "n2", width: 72, height: 24 },
];

const chainEdges: LayoutInputEdge[] = [
	{ id: "e0", sourceId: "n0", targetId: "n1" },
	{ id: "e1", sourceId: "n1", targetId: "n2" },
];

describe("layoutGraphInternals", () => {
	it("returns empty result for empty input", async () => {
		const result = await layoutGraphInternals([], []);
		expect(result.positions.size).toBe(0);
		expect(result.totalWidth).toBe(0);
		expect(result.totalHeight).toBe(0);
	});

	it("lays out nodes for a simple chain", async () => {
		const result = await layoutGraphInternals(threeNodeChain, chainEdges);
		expect(result.positions.size).toBe(3);
		expect(result.totalWidth).toBeGreaterThan(0);
		expect(result.totalHeight).toBeGreaterThan(0);
	});

	it("uses specified node widths", async () => {
		const nodes: LayoutInputNode[] = [
			{ id: "a", width: 100, height: 24 },
			{ id: "b", width: 80, height: 24 },
			{ id: "c", width: 120, height: 24 },
		];
		const edges: LayoutInputEdge[] = [
			{ id: "e0", sourceId: "a", targetId: "b" },
			{ id: "e1", sourceId: "b", targetId: "c" },
		];
		const result = await layoutGraphInternals(nodes, edges);
		expect(result.positions.get("a")?.width).toBe(100);
		expect(result.positions.get("b")?.width).toBe(80);
		expect(result.positions.get("c")?.width).toBe(120);
	});

	it("places FIRST-constrained nodes left of unconstrained nodes", async () => {
		const result = await layoutGraphInternals(threeNodeChain, chainEdges);
		const n0 = result.positions.get("n0")!;
		const n1 = result.positions.get("n1")!;
		const n2 = result.positions.get("n2")!;

		expect(n0.x).toBeLessThan(n1.x);
		expect(n1.x).toBeLessThan(n2.x);
	});

	it("places LAST-constrained nodes to the right", async () => {
		const nodes: LayoutInputNode[] = [
			{ id: "input", width: 72, height: 24, layerConstraint: "FIRST" },
			{ id: "compute", width: 72, height: 24 },
			{ id: "output", width: 72, height: 24, layerConstraint: "LAST" },
		];
		const edges: LayoutInputEdge[] = [
			{ id: "e0", sourceId: "input", targetId: "compute" },
			{ id: "e1", sourceId: "compute", targetId: "output" },
		];
		const result = await layoutGraphInternals(nodes, edges);
		const inputPos = result.positions.get("input")!;
		const computePos = result.positions.get("compute")!;
		const outputPos = result.positions.get("output")!;

		expect(inputPos.x).toBeLessThan(computePos.x);
		expect(computePos.x).toBeLessThan(outputPos.x);
	});

	it("aligns multiple FIRST nodes in the same leftmost layer", async () => {
		const nodes: LayoutInputNode[] = [
			{ id: "c0", width: 72, height: 24, layerConstraint: "FIRST" },
			{ id: "c1", width: 72, height: 24, layerConstraint: "FIRST" },
			{ id: "proc", width: 72, height: 24 },
		];
		const edges: LayoutInputEdge[] = [
			{ id: "e0", sourceId: "c0", targetId: "proc" },
			{ id: "e1", sourceId: "c1", targetId: "proc" },
		];
		const result = await layoutGraphInternals(nodes, edges);
		const c0X = result.positions.get("c0")!.x;
		const c1X = result.positions.get("c1")!.x;
		const procX = result.positions.get("proc")!.x;

		expect(c0X).toBe(c1X);
		expect(c0X).toBeLessThan(procX);
	});
});
