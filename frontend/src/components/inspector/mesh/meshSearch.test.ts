import { describe, expect, it } from "vitest";
import { annotation, withAnnotations } from "../../../test/brainFixtures.ts";
import type { BrainView } from "../../../types/generated/BrainView.ts";
import type { NodeAnnotation } from "../../../types/generated/NodeAnnotation.ts";
import { searchMeshSemantics } from "./meshSearch.ts";
import { deriveMeshSemantics } from "./meshSemantics.ts";

const genome: BrainView = {
	entry_node_id: 1,
	nodes: [
		{
			node_id: 1,
			annotation: annotation(),
			input_refs: [{ family: "world", key: "FoodHere" }],
			targets: [{ target_id: 2, slot: 0, gate_bias: 0.0 }],
			backend: {
				Vm: {
					register_count: 1,
					constants: [],
					program: [{ WriteRouteGate: { slot: 0, src: 0 } }, "Halt"],
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
			targets: [],
			backend: {
				Vm: {
					register_count: 1,
					constants: [],
					program: [{ LoadSlotImm: { dst: 0, slot_idx: 3 } }, "Halt"],
				},
			},
		},
	],
};

const annotations: Record<number, Partial<NodeAnnotation>> = {
	1: {
		reachable: true,
		read_classes: ["food"],
		write_classes: ["route"],
		has_stateful_behavior: false,
		live_instruction_indices: [0],
	},
	2: {
		reachable: true,
		read_classes: ["upstream"],
		write_classes: ["route"],
		has_stateful_behavior: false,
		live_internal_node_indices: [0],
	},
	3: {
		reachable: true,
		read_classes: ["introspection"],
		write_classes: [],
		has_stateful_behavior: false,
		live_instruction_indices: [0],
	},
};

describe("meshSearch", () => {
	it("ranks exact label and id matches ahead of partial semantic matches", () => {
		const semantics = deriveMeshSemantics(withAnnotations(genome, annotations));

		expect(searchMeshSemantics(semantics, "Route Selector")[0]?.nodeId).toBe(1);
		expect(searchMeshSemantics(semantics, "#3")[0]?.nodeId).toBe(3);
		expect(searchMeshSemantics(semantics, "introspection")[0]?.nodeId).toBe(3);
	});
});
