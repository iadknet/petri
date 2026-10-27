import { describe, expect, it } from "vitest";
import { annotation, withAnnotations } from "../../../test/brainFixtures.ts";
import type { BrainView } from "../../../types/generated/BrainView.ts";
import {
	type MeshAnalysis,
	analyzeMesh,
	collectMeshClosure,
	matchesBackendFilter,
} from "./meshAnalysis.ts";

function reachableIds(analysis: MeshAnalysis): number[] {
	return analysis.nodes.filter((node) => node.reachable).map((node) => node.id);
}

function makeGenome(): BrainView {
	return {
		entry_node_id: 1,
		nodes: [
			{
				node_id: 1,
				annotation: annotation(),
				input_refs: [{ family: "world", key: "FoodHere" }],
				targets: [{ target_id: 2, slot: 0, gate_bias: 0.0 }],
				backend: {
					Vm: {
						register_count: 2,
						constants: [1],
						program: [{ ReadInput: { dst: 0, ref_idx: 0, sub_idx: 0 } }, "Halt"],
					},
				},
			},
			{
				node_id: 2,
				annotation: annotation(),
				input_refs: [{ family: "upstream", key: "UpstreamSlot", slot: 0 }],
				targets: [{ target_id: 3, slot: 0, gate_bias: 0.0 }],
				backend: {
					Graph: {
						bundles: [],
						compute_nodes: [
							{
								kind: "Add",
								edges: [
									{
										source: { InputLeaf: { ref_idx: 0, sub_idx: 0 } },
										own: 1,
										genomic: 1,
										applied: 1,
									},
								],
							},
						],
						output_sinks: [
							{
								kind: { RouterGate: 0 },
								edges: [{ source: { ComputeNode: 0 }, own: 1, genomic: 1, applied: 1 }],
							},
						],
					},
				},
			},
			{
				node_id: 3,
				annotation: annotation(),
				input_refs: [{ family: "dynamic", key: "AgeTicks" }],
				targets: [{ target_id: 99, slot: 0, gate_bias: 0.0 }],
				backend: {
					Vm: {
						register_count: 1,
						constants: [],
						program: [{ WriteRouteGate: { slot: 0, src: 0 } }, "Halt"],
					},
				},
			},
			{
				node_id: 8,
				annotation: annotation({ reachable: false }),
				input_refs: [{ family: "static", key: "AgeTicks" }],
				targets: [],
				backend: {
					Graph: {
						bundles: [],
						compute_nodes: [{ kind: { Constant: 1 }, edges: [] }],
						output_sinks: [],
					},
				},
			},
		],
	};
}

describe("analyzeMesh", () => {
	it("classifies nodes by their annotation and preserves dangling edges", () => {
		const analysis = analyzeMesh(makeGenome());

		expect(analysis.entryNodeId).toBe(1);
		expect(reachableIds(analysis)).toEqual([1, 2, 3]);
		expect(analysis.danglingTargetIds).toEqual(new Set([99]));
		expect(analysis.edges.find((edge) => edge.fromId === 3 && edge.toId === 99)?.isDangling).toBe(
			true,
		);
	});

	it("reads reachability from the brain view's annotation, not from the routes", () => {
		const analysis = analyzeMesh(
			withAnnotations(makeGenome(), { 3: { reachable: false }, 8: { reachable: true } }),
		);

		expect(analysis.nodesById.get(3)?.reachable).toBe(false);
		expect(reachableIds(analysis)).toEqual([1, 2, 8]);
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
			targets: [
				{ target_id: 3, slot: 0, gate_bias: 0.0 },
				{ target_id: 3, slot: 1, gate_bias: 0.0 },
			],
		};

		const analysis = analyzeMesh(genome);
		const duplicateEdges = analysis.edges.filter((edge) => edge.fromId === 2 && edge.toId === 3);

		expect(duplicateEdges).toHaveLength(2);
		expect(new Set(duplicateEdges.map((edge) => edge.id))).toHaveLength(2);
	});
});
