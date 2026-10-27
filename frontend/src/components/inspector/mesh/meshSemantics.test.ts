import { describe, expect, it } from "vitest";
import { annotation } from "../../../test/brainFixtures.ts";
import type { BrainView } from "../../../types/generated/BrainView.ts";
import type { NodeAnnotation } from "../../../types/generated/NodeAnnotation.ts";
import { deriveMeshSemantics } from "./meshSemantics.ts";

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
						program: [
							{ ReadInput: { dst: 0, ref_idx: 0, sub_idx: 0 } },
							{ WriteRouteGate: { slot: 0, src: 0 } },
							"Halt",
						],
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
								kind: { DecayIntegrator: 0.25 },
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
						program: [{ LoadSlotImm: { dst: 0, slot_idx: 4 } }, "Halt"],
					},
				},
			},
		],
	};
}

/** The server's annotations of `makeGenome`'s nodes, by node id. */
const annotations: Record<number, NodeAnnotation> = {
	1: annotation({
		read_classes: ["food"],
		write_classes: ["route"],
		live_instruction_indices: [0, 1],
	}),
	2: annotation({
		read_classes: ["upstream"],
		write_classes: ["route"],
		has_stateful_behavior: true,
		live_internal_node_indices: [0],
	}),
	3: annotation({
		read_classes: ["introspection"],
		live_instruction_indices: [0],
	}),
};

/** `brain` with each node's annotation from `byId`. */
function annotate(brain: BrainView, byId: Record<number, NodeAnnotation>): BrainView {
	return {
		...brain,
		nodes: brain.nodes.map((node) => ({
			...node,
			annotation: byId[node.node_id] ?? node.annotation,
		})),
	};
}

describe("meshSemantics", () => {
	it("derives deterministic labels and rationale from factual annotations", () => {
		const semantics = deriveMeshSemantics(annotate(makeGenome(), annotations));
		const node = semantics.nodesById.get(1);

		expect(node?.role).toBe("route_selector");
		expect(node?.sourceClass).toBe("food");
		expect(node?.confidence).toBe("high");
		expect(node?.label).toBe("Route Selector · Food");
		expect(node?.badges).toEqual(expect.arrayContaining(["input", "route"]));
		expect(node?.rationale.role).toContain("Scores route slots");
		expect(node?.rationale.confidence).toContain("server-provided");
	});

	it("reads classes only from the brain view's annotations, never from the structure", () => {
		const genome = makeGenome();
		genome.nodes[0]!.input_refs = [{ family: "world", key: "FoodHere", food_type: 1 }];

		expect(() => deriveMeshSemantics(genome)).not.toThrow();
		const semantics = deriveMeshSemantics(genome);
		for (const node of semantics.nodes) {
			expect(node.readClasses).toEqual([]);
			expect(node.writeClasses).toEqual([]);
			expect(node.hasStatefulBehavior).toBe(false);
		}
		const annotated = deriveMeshSemantics(annotate(genome, annotations));
		expect(annotated.nodesById.get(1)?.sourceClass).toBe("food");
	});
});
