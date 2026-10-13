import { afterEach, describe, expect, it, vi } from "vitest";
import type { CreatureGenome } from "../../../types/genome.ts";
import { analyzeMesh } from "./meshAnalysis.ts";
import { createElk, layoutMesh } from "./meshLayout.ts";

function makeCyclicGenome(): CreatureGenome {
	return {
		entry_node_id: 1,
		nodes: [
			{
				node_id: 1,
				input_refs: [],
				targets: [{ target_id: 2, slot: 0, gate_bias: 0.0 }],
				backend_def: {
					Vm: {
						register_count: 1,
						constants: [],
						program: ["Halt"],
					},
				},
			},
			{
				node_id: 2,
				input_refs: [],
				targets: [{ target_id: 1, slot: 0, gate_bias: 0.0 }],
				backend_def: {
					Graph: {
						compute_nodes: [{ kind: { Constant: 1 }, inputs: [] }],
						output_sinks: [],
					},
				},
			},
			{
				node_id: 9,
				input_refs: [],
				targets: [{ target_id: 42, slot: 0, gate_bias: 0.0 }],
				backend_def: {
					Vm: {
						register_count: 1,
						constants: [],
						program: [{ WriteRouteGate: { slot: 0, src: 0 } }, "Halt"],
					},
				},
			},
		],
	};
}

describe("layoutMesh", () => {
	it("lays out cyclic reachable graphs and trailing unreachable clusters", async () => {
		const layout = await layoutMesh(analyzeMesh(makeCyclicGenome()));
		const node1 = layout.nodesById.get(1);
		const node2 = layout.nodesById.get(2);
		const node9 = layout.nodesById.get(9);

		expect(node1).toBeDefined();
		expect(node2).toBeDefined();
		expect(node9).toBeDefined();
		expect(node9?.x ?? 0).toBeGreaterThan(Math.max(node1?.x ?? 0, node2?.x ?? 0));
		expect(layout.width).toBeGreaterThan(0);
		expect(layout.height).toBeGreaterThan(0);
	});

	it("drops routed geometry for missing targets without failing layout", async () => {
		const layout = await layoutMesh(analyzeMesh(makeCyclicGenome()));

		expect(layout.edges.some((edge) => edge.toId === 42)).toBe(false);
		expect(layout.edges.some((edge) => edge.fromId === 1 && edge.toId === 2)).toBe(true);
	});
});

describe("createElk", () => {
	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it("runs ELK in a Web Worker when the browser provides one", async () => {
		const workerUrls: string[] = [];
		class FakeWorker {
			onmessage: ((event: MessageEvent) => void) | null = null;
			constructor(url: string | URL) {
				workerUrls.push(String(url));
			}
			postMessage() {}
			terminate() {}
		}
		vi.stubGlobal("Worker", FakeWorker);

		await createElk();

		expect(workerUrls).toHaveLength(1);
		expect(workerUrls[0]).toContain("elk-worker");
	});

	it("falls back to main-thread ELK when Worker is unavailable", async () => {
		vi.stubGlobal("Worker", undefined);

		const elk = await createElk();
		const result = await elk.layout({
			id: "root",
			children: [{ id: "a", width: 10, height: 10 }],
			edges: [],
		});

		expect(result.children?.[0]?.x).toBeDefined();
	});
});
