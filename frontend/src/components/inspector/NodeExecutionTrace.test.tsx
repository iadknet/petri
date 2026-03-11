import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { MeshHopTrace } from "../../types/trace.ts";
import { NodeExecutionTrace } from "./NodeExecutionTrace.tsx";

vi.mock("./VmExecutionView.tsx", () => ({
	VmExecutionView: () => <div data-testid="vm-execution-view">VM</div>,
}));
vi.mock("./GraphExecutionView.tsx", () => ({
	GraphExecutionView: () => <div data-testid="graph-execution-view">Graph</div>,
}));

describe("NodeExecutionTrace", () => {
	it("renders nothing when no execution hop", () => {
		const { container } = render(<NodeExecutionTrace executionHop={null} />);

		expect(container.innerHTML).toBe("");
	});

	it("renders VmExecutionView when VM trace provided", () => {
		const hop: MeshHopTrace = {
			hop_index: 0,
			node_id: 1,
			input_refs: [{ World: "FoodHere" }],
			upstream_slots: [0.5],
			energy_before: 100,
			energy_after: 95,
			output_slots: [1.0],
			route: { kind: "vm_wrap", raw_value: 0, resolved_target_index: 0 },
			backend_trace: {
				Vm: {
					register_count: 2,
					constants: [],
					steps: [],
					final_registers: [0, 0],
					final_payload: [],
					final_meta: [],
					final_route_value: 0,
					slot_writes: [],
				},
			},
		};

		render(<NodeExecutionTrace executionHop={{ hop, detailIndex: 0 }} />);

		expect(screen.getByTestId("vm-execution-view")).toBeInTheDocument();
	});

	it("renders GraphExecutionView when Graph trace provided", () => {
		const hop: MeshHopTrace = {
			hop_index: 0,
			node_id: 2,
			input_refs: [{ UpstreamSlot: 0 }],
			upstream_slots: [0.5],
			energy_before: 95,
			energy_after: 90,
			output_slots: [0.8],
			route: { kind: "cgp_normalized", raw_value: 0.8, resolved_target_index: 1 },
			backend_trace: {
				Graph: {
					passes: [],
					converged: true,
					stable_passes_count: 1,
					final_outputs: [0.8],
				},
			},
		};

		render(<NodeExecutionTrace executionHop={{ hop, detailIndex: 0 }} />);

		expect(screen.getByTestId("graph-execution-view")).toBeInTheDocument();
	});
});
