import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import type { MeshPassTrace, TickTrace } from "../../types/trace.ts";
import { VOTE_SINK_COUNT } from "../../types/trace.ts";
import { VoteSurfaceBlock } from "./VoteSurfaceBlock.tsx";

function makeTick(passes: MeshPassTrace[], commitCounts: number[]): TickTrace {
	return {
		tick_number: 3,
		energy_before: 10,
		energy_after: 9,
		static_inputs: {
			food_here: 0,
			neighbor_food: [0, 0, 0, 0, 0, 0, 0, 0],
			neighbor_barrier: [0, 0, 0, 0, 0, 0, 0, 0],
			neighbor_occupied: [0, 0, 0, 0, 0, 0, 0, 0],
			age_ticks: 0,
		},
		debug_perception: null,
		hops: [],
		passes,
		final_actions: ["NoOp"],
		termination_reason: "NoDecision",
		priority_bid: 0,
		commit_counts: commitCounts,
	};
}

function zeroVotes(): number[] {
	return Array.from({ length: VOTE_SINK_COUNT }, () => 0);
}

describe("VoteSurfaceBlock", () => {
	it("shows each pass's reason, committed action, and non-zero votes", () => {
		const votes = zeroVotes();
		votes[0] = 1.5;
		votes[3] = -2.25;
		votes[26] = 0.5;
		render(
			<VoteSurfaceBlock
				tick={makeTick(
					[
						{
							pass_index: 0,
							end_reason: "Decided",
							votes,
							effective_votes: [1.5, 0, 0, 0],
							committed: { Eat: { type_idx: 0 } },
							hops: 2,
						},
						{
							pass_index: 1,
							end_reason: "NoTargets",
							votes: zeroVotes(),
							effective_votes: [-1, 0, 0, 0],
							committed: null,
							hops: 1,
						},
					],
					[1, 0, 0, 0],
				)}
			/>,
		);

		expect(screen.getByText("Action Selection · NoDecision")).toBeInTheDocument();
		expect(screen.getByText("commit Eat(food0)")).toBeInTheDocument();
		expect(screen.getByText("no commit")).toBeInTheDocument();
		expect(screen.getByText("Eat: 1.500")).toBeInTheDocument();
		expect(screen.getByText("Move[2]: -2.250")).toBeInTheDocument();
		expect(screen.getByText("Decide: 0.500")).toBeInTheDocument();
		expect(screen.getByText("no votes this pass")).toBeInTheDocument();
		expect(screen.queryByText(/Terminate:/)).not.toBeInTheDocument();
	});

	it("shows each pass's effective vote per kind against the bar so the commits line up", () => {
		const votes = zeroVotes();
		votes[3] = 0.52; // Move[2]
		votes[11] = 0.026; // Reproduce[2]
		const pass = (
			pass_index: number,
			effective_votes: number[],
			committed: MeshPassTrace["committed"],
		): MeshPassTrace => ({
			pass_index,
			end_reason: "NoTargets",
			votes,
			effective_votes,
			committed,
			hops: 3,
		});
		render(
			<VoteSurfaceBlock
				tick={makeTick(
					[
						pass(0, [0, 0.52, 0.026, 0], { Move: "E" }),
						pass(1, [0, -0.48, 0.026, 0], {
							Reproduce: { direction: "E", energy_transfer_fraction: 0.5 },
						}),
						pass(2, [0, -0.48, -0.974, 0], null),
					],
					[0, 1, 1, 0],
				)}
			/>,
		);

		expect(screen.getByTestId("effective-0-Move")).toHaveTextContent("Move 0.520 − 0 = 0.520");
		expect(screen.getByTestId("effective-0-Move")).toHaveClass("text-orange-300");
		expect(screen.getByTestId("effective-1-Move")).toHaveTextContent("Move 0.520 − 1 = -0.480");
		expect(screen.getByTestId("effective-1-Reproduce")).toHaveTextContent(
			"Reproduce 0.026 − 0 = 0.026",
		);
		expect(screen.getByTestId("effective-1-Reproduce")).toHaveClass("text-orange-300");
		expect(screen.getByTestId("effective-2-Reproduce")).toHaveTextContent(
			"Reproduce 0.026 − 1 = -0.974",
		);
		expect(screen.getByTestId("effective-2-Reproduce")).not.toHaveClass("text-orange-300");
		expect(screen.queryByTestId("effective-0-Eat")).not.toBeInTheDocument();
	});

	it("says so when the tick ran no passes, and always shows the four bars", () => {
		render(<VoteSurfaceBlock tick={makeTick([], [0, 1, 2, 3])} />);

		expect(screen.getByText("no passes this tick")).toBeInTheDocument();
		expect(screen.getByText("Eat: 0")).toBeInTheDocument();
		expect(screen.getByText("Move: 1")).toBeInTheDocument();
		expect(screen.getByText("Reproduce: 2")).toBeInTheDocument();
		expect(screen.getByText("StealEnergy: 3")).toBeInTheDocument();
	});
});
