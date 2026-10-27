import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import type { BrainBackend } from "../../types/generated/BrainBackend.ts";
import { NodeBackendDetail } from "./NodeBackendDetail.tsx";

describe("NodeBackendDetail", () => {
	it("renders VM instructions with junk marking", () => {
		const backendDef: BrainBackend = {
			Vm: {
				register_count: 2,
				constants: [1, 2],
				program: [
					{ ReadInput: { dst: 0, ref_idx: 0, sub_idx: 0 } },
					{ WriteRouteGate: { slot: 0, src: 0 } },
					"Halt",
				],
			},
		};

		render(
			<NodeBackendDetail
				backendDef={backendDef}
				inputRefs={[{ family: "world", key: "FoodHere" }]}
				liveInstructionIndices={[0, 2]}
				liveInternalNodeIndices={[]}
			/>,
		);

		const instr0 = screen.getByTestId("vm-instruction-0");
		const instr1 = screen.getByTestId("vm-instruction-1");
		const instr2 = screen.getByTestId("vm-instruction-2");

		expect(instr0).toHaveAttribute("data-junk", "false");
		expect(instr1).toHaveAttribute("data-junk", "true");
		expect(instr2).toHaveAttribute("data-junk", "false");
	});

	it("returns null for Graph backends (handled by GraphInternalsViz)", () => {
		const backendDef: BrainBackend = {
			Graph: {
				bundles: [],
				compute_nodes: [
					{
						kind: "Add",
						edges: [
							{ source: { InputLeaf: { ref_idx: 0, sub_idx: 0 } }, own: 1, genomic: 1, applied: 1 },
						],
					},
				],
				output_sinks: [],
			},
		};

		const { container } = render(
			<NodeBackendDetail
				backendDef={backendDef}
				inputRefs={[]}
				liveInstructionIndices={[]}
				liveInternalNodeIndices={[]}
			/>,
		);

		expect(container.innerHTML).toBe("");
	});
});
