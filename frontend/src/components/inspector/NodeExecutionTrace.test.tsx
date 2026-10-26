import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { zeroDecisionInputs, zeroVotes } from "../../test/traceFixtures.ts";
import type { MeshHopTracePayload } from "../../types/generated/MeshHopTracePayload.ts";
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
		const hop: MeshHopTracePayload = {
			hop_index: 0,
			pass_index: 0,
			vote_contribution: zeroVotes(),
			decision_inputs: zeroDecisionInputs(),
			node_id: 1,
			input_refs: [{ family: "world", key: "FoodHere" }],
			upstream_slots: [0.5],
			energy_before: 100,
			energy_after: 95,
			output_slots: [1.0],
			route: null,
			backend_trace: {
				Vm: {
					register_count: 2,
					constants: [],
					steps: [],
					final_registers: [0, 0],
					final_payload: [],
					slot_writes: [],
				},
			},
		};

		render(<NodeExecutionTrace executionHop={{ hop, detailIndex: 0 }} />);

		expect(screen.getByTestId("vm-execution-view")).toBeInTheDocument();
	});

	it("renders GraphExecutionView when Graph trace provided", () => {
		const hop: MeshHopTracePayload = {
			hop_index: 0,
			pass_index: 0,
			vote_contribution: zeroVotes(),
			decision_inputs: zeroDecisionInputs(),
			node_id: 2,
			input_refs: [{ family: "upstream", key: "UpstreamSlot", slot: 0 }],
			upstream_slots: [0.5],
			energy_before: 95,
			energy_after: 90,
			output_slots: [0.8],
			route: { gate_scores: [], selected_target_idx: 1, selected_target_id: 2 },
			backend_trace: {
				Graph: {
					passes: [],
					temporal_committed: true,
					final_outputs: [0.8],
					output_sinks: [],
				},
			},
		};

		render(<NodeExecutionTrace executionHop={{ hop, detailIndex: 0 }} />);

		expect(screen.getByTestId("graph-execution-view")).toBeInTheDocument();
	});
});
