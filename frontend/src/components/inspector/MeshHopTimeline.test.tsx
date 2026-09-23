import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { MeshHopTrace, MeshPassTrace } from "../../types/trace.ts";
import { ZERO_DECISION_INPUTS, ZERO_VOTES } from "../../types/trace.ts";
import { MeshHopTimeline } from "./MeshHopTimeline.tsx";

function hop(
	hop_index: number,
	pass_index: number,
	node_id: number,
	route: MeshHopTrace["route"],
	graph = false,
): MeshHopTrace {
	return {
		hop_index,
		pass_index,
		node_id,
		input_refs: [],
		upstream_slots: [],
		energy_before: 10,
		energy_after: 9,
		output_slots: [],
		route,
		vote_contribution: ZERO_VOTES,
		decision_inputs: ZERO_DECISION_INPUTS,
		backend_trace: graph
			? {
					Graph: {
						temporal_committed: true,
						passes: [],
						final_outputs: [],
						output_sinks: [],
					},
				}
			: {
					Vm: {
						register_count: 1,
						constants: [],
						steps: [],
						final_registers: [0],
						final_payload: [],
						slot_writes: [],
					},
				},
	};
}

function pass(
	pass_index: number,
	end_reason: MeshPassTrace["end_reason"],
	hops: number,
	committed: MeshPassTrace["committed"],
): MeshPassTrace {
	return {
		pass_index,
		end_reason,
		votes: ZERO_VOTES,
		effective_votes: [0, 0, 0, 0],
		committed,
		hops,
	};
}

const toNode = (id: number): MeshHopTrace["route"] => ({
	gate_scores: [],
	selected_target_idx: 0,
	selected_target_id: id,
});

describe("MeshHopTimeline", () => {
	it("groups recorded hops under their pass, keeps a pass with no recorded hop, and ends with the tick", () => {
		const onHopSelect = vi.fn();
		render(
			<MeshHopTimeline
				hops={[hop(0, 0, 1, toNode(2)), hop(1, 0, 2, null), hop(2, 2, 1, toNode(2))]}
				passes={[
					pass(0, "Decided", 2, { Eat: { type_idx: 0 } }),
					pass(1, "EnergyExhausted", 1, null),
					pass(2, "PassCapReached", 1, null),
				]}
				meshSemantics={null}
				activeHopIndex={0}
				terminationReason="EnergyExhausted"
				finalActions={[{ Eat: { type_idx: 0 } }, { Move: "N" }]}
				onHopSelect={onHopSelect}
			/>,
		);

		const first = screen.getByTestId("pass-group-0");
		expect(first).toHaveTextContent("Pass 0 · Decided · 2 hops · commit Eat(food0)");
		expect(within(first).getByTestId("hop-0")).toBeInTheDocument();
		expect(within(first).getByTestId("hop-1")).toBeInTheDocument();
		const empty = screen.getByTestId("pass-group-1");
		expect(empty).toHaveTextContent("Pass 1 · EnergyExhausted · 1 hops · no commit");
		expect(empty).toHaveTextContent("no recorded hop");
		expect(within(screen.getByTestId("pass-group-2")).getByTestId("hop-2")).toBeInTheDocument();
		expect(screen.getByTestId("tick-end")).toHaveTextContent(
			"EnergyExhausted · queue Eat(food0), Move(N)",
		);

		fireEvent.click(screen.getByTestId("hop-2"));
		expect(onHopSelect).toHaveBeenCalledWith(2);
	});

	it("shows a route only on hops that applied one and no convergence label on graph hops", () => {
		render(
			<MeshHopTimeline
				hops={[hop(0, 0, 1, toNode(7), true), hop(1, 0, 7, null, true)]}
				passes={[pass(0, "Decided", 2, null)]}
				meshSemantics={null}
				activeHopIndex={0}
				terminationReason="NoDecision"
				finalActions={["NoOp"]}
				onHopSelect={vi.fn()}
			/>,
		);

		expect(screen.getByTestId("hop-0")).toHaveTextContent("→ #7");
		expect(screen.getByTestId("hop-1")).not.toHaveTextContent("→");
		expect(screen.queryByText(/converged/i)).not.toBeInTheDocument();
	});
});
