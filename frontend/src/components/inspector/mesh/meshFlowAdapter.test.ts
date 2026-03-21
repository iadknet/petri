import { describe, expect, it } from "vitest";
import type { CreatureMeshAnnotation } from "../../../types/creature-detail.ts";
import type { CreatureGenome } from "../../../types/genome.ts";
import { analyzeMesh } from "./meshAnalysis.ts";
import {
	MESH_FLOW_SOURCE_HANDLE_ID,
	MESH_FLOW_TARGET_HANDLE_ID,
	buildMeshFlowScene,
} from "./meshFlowAdapter.ts";
import type { MeshLayout } from "./meshLayout.ts";
import { deriveMeshSemantics } from "./meshSemantics.ts";

const genome: CreatureGenome = {
	entry_node_id: 1,
	nodes: [
		{
			node_id: 1,
			input_refs: [{ World: "FoodHere" }],
			targets: [{ target_id: 2, slot: 0, gate_bias: 0.0 }],
			backend_def: {
				Vm: {
					register_count: 1,
					constants: [],
					program: [{ WriteRouteTarget: { src: 0 } }, "Halt"],
				},
			},
		},
		{
			node_id: 2,
			input_refs: [{ UpstreamSlot: 0 }],
			targets: [],
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
	],
};

const annotations: CreatureMeshAnnotation[] = [
	{
		node_id: 1,
		reachable: true,
		read_classes: ["food"],
		write_classes: ["route"],
		has_stateful_behavior: false,
		live_instruction_indices: [0],
	},
	{
		node_id: 2,
		reachable: true,
		read_classes: ["upstream"],
		write_classes: ["route"],
		has_stateful_behavior: false,
		live_internal_node_indices: [0],
	},
];

const layout: MeshLayout = {
	topologyKey: "1:2|2:",
	nodes: [
		{ id: 1, x: 24, y: 32, width: 156, height: 64, region: "reachable" },
		{ id: 2, x: 284, y: 32, width: 156, height: 64, region: "reachable" },
	],
	nodesById: new Map(),
	edges: [
		{
			id: "1->2",
			fromId: 1,
			toId: 2,
			points: [
				{ x: 196, y: 80 },
				{ x: 240, y: 80 },
				{ x: 284, y: 80 },
			],
		},
	],
	width: 480,
	height: 180,
};

layout.nodesById = new Map(layout.nodes.map((node) => [node.id, node]));

describe("meshFlowAdapter", () => {
	it("maps mesh layout and semantics into React Flow scene data without owning layout", () => {
		const analysis = analyzeMesh(genome);
		const semantics = deriveMeshSemantics(genome, annotations);

		const scene = buildMeshFlowScene({
			analysis,
			layout,
			semanticsById: semantics.nodesById,
			selectedNodeId: 1,
			activeNodeId: 2,
			activeEdgeId: "1->2",
			dimmedNodeIds: new Set<number>([2]),
		});

		expect(scene.nodes).toHaveLength(2);
		expect(scene.nodes[0]?.type).toBe("meshNode");
		expect(scene.nodes[0]?.position).toEqual({ x: 24, y: 32 });
		expect(scene.nodes[0]?.data.label).toBe("Route Selector · Food");
		expect(scene.nodes[1]?.data.dimmed).toBe(true);
		expect(scene.edges[0]?.type).toBe("meshEdge");
		expect(scene.edges[0]?.sourceHandle).toBe(MESH_FLOW_SOURCE_HANDLE_ID);
		expect(scene.edges[0]?.targetHandle).toBe(MESH_FLOW_TARGET_HANDLE_ID);
		expect(scene.edges[0]?.data?.active).toBe(true);
		expect(scene.edges[0]?.data?.points).toHaveLength(3);
	});

	it("derives hasSharedMemory from semantics writeClasses", () => {
		const analysis = analyzeMesh(genome);
		const memoryAnnotations: CreatureMeshAnnotation[] = [
			{
				node_id: 1,
				reachable: true,
				read_classes: ["food"],
				write_classes: ["memory"],
				has_stateful_behavior: true,
				live_instruction_indices: [0],
			},
			{
				node_id: 2,
				reachable: true,
				read_classes: ["upstream"],
				write_classes: ["route"],
				has_stateful_behavior: false,
				live_internal_node_indices: [0],
			},
		];
		const semantics = deriveMeshSemantics(genome, memoryAnnotations);

		const scene = buildMeshFlowScene({
			analysis,
			layout,
			semanticsById: semantics.nodesById,
			selectedNodeId: null,
			activeNodeId: null,
			activeEdgeId: null,
			dimmedNodeIds: new Set<number>(),
		});

		expect(scene.nodes[0]?.data.hasSharedMemory).toBe(true);
		expect(scene.nodes[1]?.data.hasSharedMemory).toBe(false);
	});
});
