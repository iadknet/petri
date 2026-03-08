import { describe, expect, it } from "vitest";
import type { NodeGenome } from "../../../types/genome.ts";
import {
	collectNodeBadges,
	describeGraphInternalNode,
	describeVmInstruction,
	summarizeBackendDef,
} from "./meshPresentation.ts";

const vmNode: NodeGenome = {
	node_id: 5,
	input_refs: [{ World: "FoodHere" }],
	targets: [6],
	backend_def: {
		Vm: {
			register_count: 2,
			constants: [1, 2],
			program: [
				{ ReadInput: { dst: 0, input_idx: 0 } },
				{ StoreSlotImm: { slot_idx: 4, src: 0 } },
				{ WriteWorldActionMeta: { slot_idx: 0, src: 0 } },
				{ PushAction: { action_type: 2 } },
				{ WriteRouteTarget: { src: 0 } },
				"Halt",
			],
		},
	},
};

const graphNode: NodeGenome = {
	node_id: 7,
	input_refs: [{ UpstreamSlot: 0 }],
	targets: [],
	backend_def: {
		Graph: {
			internal_nodes: [
				{ kind: { InputRef: { ref_idx: 0, sub_idx: 0 } }, inputs: [] },
				{
					kind: { DecayIntegrator: 0.5 },
					inputs: [{ source_idx: 0, weight: 1 }],
				},
				{ kind: { CustomOutput: 1 }, inputs: [{ source_idx: 1, weight: 1 }] },
				{ kind: "RouterOutput", inputs: [{ source_idx: 1, weight: 0.5 }] },
			],
		},
	},
};

describe("meshPresentation", () => {
	it("classifies VM instructions and node badges", () => {
		expect(
			describeVmInstruction({ ReadInput: { dst: 0, input_idx: 0 } }).badges,
		).toContain("input");
		expect(
			describeVmInstruction({ StoreSlotImm: { slot_idx: 4, src: 0 } }).badges,
		).toContain("slot");
		expect(
			describeVmInstruction({ PushAction: { action_type: 2 } }).badges,
		).toContain("action");
		expect(
			describeVmInstruction({ WriteRouteTarget: { src: 0 } }).badges,
		).toContain("route");
		expect(collectNodeBadges(vmNode)).toEqual(
			expect.arrayContaining(["action", "input", "slot", "route"]),
		);
	});

	it("classifies graph node roles and backend summaries", () => {
		expect(
			describeGraphInternalNode({ DecayIntegrator: 0.5 }).badges,
		).toContain("stateful");
		expect(describeGraphInternalNode({ CustomOutput: 1 }).badges).toContain(
			"output",
		);
		expect(describeGraphInternalNode("RouterOutput").badges).toContain("route");
		expect(collectNodeBadges(graphNode)).toEqual(
			expect.arrayContaining(["output", "route", "stateful"]),
		);
		expect(summarizeBackendDef(graphNode.backend_def).detail).toContain(
			"4 nodes",
		);
	});
});
