import { describe, expect, it } from "vitest";
import { annotation } from "../../../test/brainFixtures.ts";
import type { BrainNode } from "../../../types/generated/BrainNode.ts";
import {
	describeComputeNodeKind,
	describeVmInstruction,
	summarizeBackendDef,
} from "./meshPresentation.ts";

const graphNode: BrainNode = {
	node_id: 7,
	annotation: annotation(),
	input_refs: [{ family: "upstream", key: "UpstreamSlot", slot: 0 }],
	targets: [],
	backend: {
		Graph: {
			groups: [],
			compute_nodes: [
				{
					kind: { DecayIntegrator: 0.5 },
					edges: [
						{ source: { InputLeaf: { ref_idx: 0, sub_idx: 0 } }, own: 1, genomic: 1, applied: 1 },
					],
				},
			],
			output_sinks: [
				{
					kind: { CustomOutput: 1 },
					edges: [{ source: { ComputeNode: 0 }, own: 1, genomic: 1, applied: 1 }],
				},
				{
					kind: { RouterGate: 0 },
					edges: [{ source: { ComputeNode: 0 }, own: 0.5, genomic: 0.5, applied: 0.5 }],
				},
			],
		},
	},
};

describe("meshPresentation", () => {
	it("classifies VM instructions", () => {
		expect(
			describeVmInstruction({ ReadInput: { dst: 0, ref_idx: 0, sub_idx: 0 } }).badges,
		).toContain("input");
		expect(describeVmInstruction({ StoreSlotImm: { slot_idx: 4, src: 0 } }).badges).toContain(
			"slot",
		);
		expect(describeVmInstruction({ AddVote: { sink: 3, src: 0 } }).badges).toContain("action");
		expect(describeVmInstruction({ WriteRouteGate: { slot: 0, src: 0 } }).badges).toContain(
			"route",
		);
	});

	it("classifies graph compute node roles and backend summaries", () => {
		expect(describeComputeNodeKind({ DecayIntegrator: 0.5 }).badges).toContain("stateful");
		expect(summarizeBackendDef(graphNode.backend).detail).toContain("1 compute");
		expect(summarizeBackendDef(graphNode.backend).detail).toContain("2 sinks");
	});
});
