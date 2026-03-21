import { describe, expect, it } from "vitest";
import type { NodeGenome } from "../../../types/genome.ts";
import {
	collectNodeBadges,
	describeComputeNodeKind,
	describeVmInstruction,
	summarizeBackendDef,
} from "./meshPresentation.ts";

const vmNode: NodeGenome = {
	node_id: 5,
	input_refs: [{ World: "FoodHere" }],
	targets: [{ target_id: 6, slot: 0, gate_bias: 0.0 }],
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
			compute_nodes: [
				{
					kind: { DecayIntegrator: 0.5 },
					inputs: [{ source: { InputLeaf: { ref_idx: 0, sub_idx: 0 } }, weight: 1 }],
				},
			],
			output_sinks: [
				{
					kind: { CustomOutput: 1 },
					inputs: [{ source: { ComputeNode: 0 }, weight: 1 }],
				},
				{
					kind: "RouterOutput",
					inputs: [{ source: { ComputeNode: 0 }, weight: 0.5 }],
				},
			],
			action_bank: [],
			execute_gate: { inputs: [] },
		},
	},
};

describe("meshPresentation", () => {
	it("classifies VM instructions and node badges", () => {
		expect(describeVmInstruction({ ReadInput: { dst: 0, input_idx: 0 } }).badges).toContain(
			"input",
		);
		expect(describeVmInstruction({ StoreSlotImm: { slot_idx: 4, src: 0 } }).badges).toContain(
			"slot",
		);
		expect(describeVmInstruction({ PushAction: { action_type: 2 } }).badges).toContain("action");
		expect(describeVmInstruction({ WriteRouteTarget: { src: 0 } }).badges).toContain("route");
		expect(collectNodeBadges(vmNode)).toEqual(
			expect.arrayContaining(["action", "input", "slot", "route"]),
		);
	});

	it("classifies graph compute node roles and backend summaries", () => {
		expect(describeComputeNodeKind({ DecayIntegrator: 0.5 }).badges).toContain("stateful");
		expect(collectNodeBadges(graphNode)).toEqual(
			expect.arrayContaining(["output", "route", "stateful"]),
		);
		expect(summarizeBackendDef(graphNode.backend_def).detail).toContain("1 compute");
		expect(summarizeBackendDef(graphNode.backend_def).detail).toContain("2 sinks");
	});
});
