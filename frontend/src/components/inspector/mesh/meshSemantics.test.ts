import { describe, expect, it } from "vitest";
import type { CreatureMeshAnnotation } from "../../../types/creature-detail.ts";
import type { CreatureGenome } from "../../../types/genome.ts";
import { deriveMeshSemantics } from "./meshSemantics.ts";

function makeGenome(): CreatureGenome {
	return {
		entry_node_id: 1,
		nodes: [
			{
				node_id: 1,
				input_refs: [{ World: "FoodHere" }],
				targets: [{ target_id: 2, slot: 0, gate_bias: 0.0 }],
				backend_def: {
					Vm: {
						register_count: 2,
						constants: [1],
						program: [
							{ ReadInput: { dst: 0, input_idx: 0 } },
							{ WriteRouteGate: { slot: 0, src: 0 } },
							"Halt",
						],
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
								kind: { DecayIntegrator: 0.25 },
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
								kind: { RouterGate: 0 },
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
				targets: [],
				backend_def: {
					Vm: {
						register_count: 1,
						constants: [],
						program: [{ LoadSlotImm: { dst: 0, slot_idx: 4 } }, "Halt"],
					},
				},
			},
		],
	};
}

const annotations: CreatureMeshAnnotation[] = [
	{
		node_id: 1,
		reachable: true,
		read_classes: ["food"],
		write_classes: ["route"],
		has_stateful_behavior: false,
		live_instruction_indices: [0, 1],
	},
	{
		node_id: 2,
		reachable: true,
		read_classes: ["upstream"],
		write_classes: ["route"],
		has_stateful_behavior: true,
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

describe("meshSemantics", () => {
	it("derives deterministic labels and rationale from factual annotations", () => {
		const semantics = deriveMeshSemantics(makeGenome(), annotations);
		const node = semantics.nodesById.get(1);

		expect(node?.role).toBe("route_selector");
		expect(node?.sourceClass).toBe("food");
		expect(node?.confidence).toBe("high");
		expect(node?.label).toBe("Route Selector · Food");
		expect(node?.badges).toEqual(expect.arrayContaining(["input", "route"]));
		expect(node?.rationale.role).toContain("Writes route");
		expect(node?.rationale.confidence).toContain("server-provided");
	});

	it("falls back to heuristics when annotations are unavailable", () => {
		const semantics = deriveMeshSemantics(makeGenome(), null);
		const node = semantics.nodesById.get(3);

		expect(node?.role).toBe("memory_reader");
		expect(node?.confidence).toBe("medium");
		expect(node?.badges).toContain("slot");
		expect(node?.label).toBe("Slot Reader · Introspection");
		expect(node?.rationale.confidence).toContain("Inferred");
	});

	it("handles typed world food references without crashing", () => {
		const genome = makeGenome();
		genome.nodes[0]!.input_refs = [
			{
				World: {
					FoodHere: {
						type_idx: 1,
					},
				},
			} as unknown as CreatureGenome["nodes"][number]["input_refs"][number],
		];

		expect(() => deriveMeshSemantics(genome, null)).not.toThrow();
		const semantics = deriveMeshSemantics(genome, null);
		expect(semantics.nodesById.get(1)?.sourceClass).toBe("food");
	});
});
