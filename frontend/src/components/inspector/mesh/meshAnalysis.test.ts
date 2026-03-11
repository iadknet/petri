import { describe, expect, it } from "vitest";
import type { CreatureGenome } from "../../../types/genome.ts";
import { analyzeMesh, collectMeshClosure, matchesBackendFilter } from "./meshAnalysis.ts";

function makeGenome(): CreatureGenome {
	return {
		entry_node_id: 1,
		nodes: [
			{
				node_id: 1,
				input_refs: [{ World: "FoodHere" }],
				targets: [2],
				backend_def: {
					Vm: {
						register_count: 2,
						constants: [1],
						program: [{ ReadInput: { dst: 0, input_idx: 0 } }, "Halt"],
					},
				},
			},
			{
				node_id: 2,
				input_refs: [{ UpstreamSlot: 0 }],
				targets: [3],
				backend_def: {
					Graph: {
						compute_nodes: [
							{
								kind: "Add",
								inputs: [
									{
										source: { InputLeaf: { ref_idx: 0, sub_idx: 0 } },
										weight: 1,
									},
								],
							},
						],
						output_sinks: [
							{
								kind: "RouterOutput",
								inputs: [{ source: { ComputeNode: 0 }, weight: 1 }],
							},
						],
						action_bank: [],
						execute_gate: { inputs: [] },
					},
				},
			},
			{
				node_id: 3,
				input_refs: [{ DynamicIntrospection: "AgeTicks" }],
				targets: [99],
				backend_def: {
					Vm: {
						register_count: 1,
						constants: [],
						program: [{ WriteRouteTarget: { src: 0 } }, "Halt"],
					},
				},
			},
			{
				node_id: 8,
				input_refs: [{ StaticIntrospection: "Generation" }],
				targets: [],
				backend_def: {
					Graph: {
						compute_nodes: [{ kind: { Constant: 1 }, inputs: [] }],
						output_sinks: [],
						action_bank: [],
						execute_gate: { inputs: [] },
					},
				},
			},
		],
	};
}

describe("analyzeMesh", () => {
	it("separates reachable and unreachable nodes and preserves dangling edges", () => {
		const analysis = analyzeMesh(makeGenome());

		expect(analysis.entryNodeId).toBe(1);
		expect(analysis.reachableNodeIds).toEqual(new Set([1, 2, 3]));
		expect(analysis.unreachableNodeIds).toEqual(new Set([8]));
		expect(analysis.danglingTargetIds).toEqual(new Set([99]));
		expect(analysis.edges.find((edge) => edge.fromId === 3 && edge.toId === 99)?.isDangling).toBe(
			true,
		);
	});

	it("computes upstream and downstream closures from the selected node", () => {
		const analysis = analyzeMesh(makeGenome());

		expect(collectMeshClosure(analysis, 3, "upstream")).toEqual(new Set([1, 2, 3]));
		expect(collectMeshClosure(analysis, 1, "downstream")).toEqual(new Set([1, 2, 3]));
	});

	it("matches backend filters without mutating topology", () => {
		const analysis = analyzeMesh(makeGenome());
		const vmNode = analysis.nodes.find((node) => node.id === 1);
		const graphNode = analysis.nodes.find((node) => node.id === 2);

		expect(vmNode).toBeDefined();
		expect(graphNode).toBeDefined();
		expect(matchesBackendFilter(vmNode as NonNullable<typeof vmNode>, "all")).toBe(true);
		expect(matchesBackendFilter(vmNode as NonNullable<typeof vmNode>, "vm")).toBe(true);
		expect(matchesBackendFilter(vmNode as NonNullable<typeof vmNode>, "graph")).toBe(false);
		expect(matchesBackendFilter(graphNode as NonNullable<typeof graphNode>, "graph")).toBe(true);
	});

	it("assigns stable unique edge ids when a node targets the same destination more than once", () => {
		const genome = makeGenome();
		const duplicatedNode = genome.nodes[1];
		if (!duplicatedNode) {
			throw new Error("missing duplicate-edge fixture node");
		}

		genome.nodes[1] = {
			...duplicatedNode,
			targets: [3, 3],
		};

		const analysis = analyzeMesh(genome);
		const duplicateEdges = analysis.edges.filter((edge) => edge.fromId === 2 && edge.toId === 3);

		expect(duplicateEdges).toHaveLength(2);
		expect(new Set(duplicateEdges.map((edge) => edge.id))).toHaveLength(2);
	});
});
