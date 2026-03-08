import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import type { BackendDef } from "../../types/genome.ts";
import { NodeBackendDetail } from "./NodeBackendDetail.tsx";

describe("NodeBackendDetail", () => {
	it("renders VM instructions with junk marking", () => {
		const backendDef: BackendDef = {
			Vm: {
				register_count: 2,
				constants: [1, 2],
				program: [
					{ ReadInput: { dst: 0, input_idx: 0 } },
					{ WriteRouteTarget: { src: 0 } },
					"Halt",
				],
			},
		};

		render(
			<NodeBackendDetail
				backendDef={backendDef}
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

	it("renders Graph internal nodes", () => {
		const backendDef: BackendDef = {
			Graph: {
				internal_nodes: [
					{ kind: { InputRef: { ref_idx: 0, sub_idx: 0 } }, inputs: [] },
					{ kind: { DecayIntegrator: 0.2 }, inputs: [{ source_idx: 0, weight: 1 }] },
					{ kind: "RouterOutput", inputs: [{ source_idx: 1, weight: 1 }] },
				],
			},
		};

		render(
			<NodeBackendDetail
				backendDef={backendDef}
				liveInstructionIndices={[]}
				liveInternalNodeIndices={[0, 1, 2]}
			/>,
		);

		expect(screen.getByTestId("graph-internal-0")).toBeInTheDocument();
		expect(screen.getByTestId("graph-internal-1")).toBeInTheDocument();
		expect(screen.getByTestId("graph-internal-2")).toBeInTheDocument();
	});

	it("marks non-live instructions as junk", () => {
		const backendDef: BackendDef = {
			Graph: {
				internal_nodes: [
					{ kind: { InputRef: { ref_idx: 0, sub_idx: 0 } }, inputs: [] },
					{ kind: { DecayIntegrator: 0.2 }, inputs: [{ source_idx: 0, weight: 1 }] },
					{ kind: "RouterOutput", inputs: [{ source_idx: 1, weight: 1 }] },
				],
			},
		};

		render(
			<NodeBackendDetail
				backendDef={backendDef}
				liveInstructionIndices={[]}
				liveInternalNodeIndices={[0, 2]}
			/>,
		);

		expect(screen.getByTestId("graph-internal-0")).toHaveAttribute("data-junk", "false");
		expect(screen.getByTestId("graph-internal-1")).toHaveAttribute("data-junk", "true");
		expect(screen.getByTestId("graph-internal-2")).toHaveAttribute("data-junk", "false");
	});
});
