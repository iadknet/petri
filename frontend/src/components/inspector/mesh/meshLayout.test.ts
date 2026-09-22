import { describe, expect, it } from "vitest";
import type { CreatureGenome } from "../../../types/genome.ts";
import { analyzeMesh } from "./meshAnalysis.ts";
import { layoutMesh } from "./meshLayout.ts";

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
