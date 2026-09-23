import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import type { MeshHopTrace } from "../../types/trace.ts";
import { ZERO_VOTES } from "../../types/trace.ts";
import { SelectedHopBlock } from "./SelectedHopBlock.tsx";

function votes(entries: Record<number, number>): number[] {
	const vector = [...ZERO_VOTES];
	for (const [index, value] of Object.entries(entries)) vector[Number(index)] = value;
	return vector;
}

const selected: MeshHopTrace = {
	hop_index: 4,
	pass_index: 1,
	node_id: 3,
	input_refs: [],
	upstream_slots: [],
	energy_before: 10,
	energy_after: 9,
	output_slots: [],
	route: null,
	vote_contribution: votes({ 0: 0.25, 26: 1 }),
	decision_inputs: {
		action_votes: votes({ 4: 0.5 }),
		previous_pass_votes: votes({ 0: 1, 25: 0.125 }),
		commit_counts: [1, 0, 2, 0],
		hops_this_tick: 5,
	},
	backend_trace: {
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

describe("SelectedHopBlock", () => {
	it("shows the hop's vote contribution and the five decision-state values by input name", () => {
		render(<SelectedHopBlock hop={selected} previousOutcome={[0.5, 1, -0.25, 0]} />);

		expect(screen.getByText(/Hop 4 · node #3 · pass 1 · no route/)).toBeInTheDocument();
		expect(screen.getByTestId("hop-vote contribution")).toHaveTextContent("Eat 0.250");
		expect(screen.getByTestId("hop-vote contribution")).toHaveTextContent("Decide 1.000");
		expect(screen.getByTestId("hop-ActionVotes")).toHaveTextContent("Move[3] 0.500");
		expect(screen.getByTestId("hop-PreviousPassVotes")).toHaveTextContent("Eat 1.000");
		expect(screen.getByTestId("hop-PreviousPassVotes")).toHaveTextContent("Terminate 0.125");
		for (const text of ["Eat 1", "Move 0", "Reproduce 2", "StealEnergy 0"]) {
			expect(screen.getByTestId("hop-CommitCounts")).toHaveTextContent(text);
		}
		expect(screen.getByTestId("hop-HopsThisTick")).toHaveTextContent("5");
		for (const text of [
			"EnergyDelta 0.500",
			"ActionSuccess 1.000",
			"DamageDelta -0.250",
			"OffspringSuccess 0.000",
		]) {
			expect(screen.getByTestId("hop-PreviousOutcome")).toHaveTextContent(text);
		}
	});

	it("reads all zero for an empty vote vector", () => {
		render(
			<SelectedHopBlock
				hop={{ ...selected, vote_contribution: ZERO_VOTES }}
				previousOutcome={[0, 0, 0, 0]}
			/>,
		);

		expect(screen.getByTestId("hop-vote contribution")).toHaveTextContent("all zero");
	});
});
