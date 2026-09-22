import { describe, expect, it } from "vitest";
import type { CreatureMeshAnnotation } from "../../../types/creature-detail.ts";
import type { CreatureGenome } from "../../../types/genome.ts";
import { searchMeshSemantics } from "./meshSearch.ts";
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
					program: [{ WriteRouteGate: { slot: 0, src: 0 } }, "Halt"],
				},
			},
		},
		{
			node_id: 2,
			input_refs: [{ UpstreamSlot: 0 }],
			targets: [{ target_id: 3, slot: 0, gate_bias: 0.0 }],
			backend_def: {
				Graph: {
					compute_nodes: [
						{
							kind: "Add",
							inputs: [{ source: { InputLeaf: { ref_idx: 0, sub_idx: 0 } }, weight: 1 }],
						},
					],
					output_sinks: [
						{ kind: { RouterGate: 0 }, inputs: [{ source: { ComputeNode: 0 }, weight: 1 }] },
					],
				},
			},
		},
		{
			node_id: 3,
			input_refs: [{ DynamicIntrospection: "AgeTicks" }],
			targets: [],
			backend_def: {
				Vm: {
					register_count: 1,
					constants: [],
					program: [{ LoadSlotImm: { dst: 0, slot_idx: 3 } }, "Halt"],
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
	{
		node_id: 3,
		reachable: true,
		read_classes: ["introspection"],
		write_classes: [],
		has_stateful_behavior: false,
		live_instruction_indices: [0],
	},
];

describe("meshSearch", () => {
	it("ranks exact label and id matches ahead of partial semantic matches", () => {
		const semantics = deriveMeshSemantics(genome, annotations);

		expect(searchMeshSemantics(semantics, "Route Selector")[0]?.nodeId).toBe(1);
		expect(searchMeshSemantics(semantics, "#3")[0]?.nodeId).toBe(3);
		expect(searchMeshSemantics(semantics, "introspection")[0]?.nodeId).toBe(3);
	});
});
