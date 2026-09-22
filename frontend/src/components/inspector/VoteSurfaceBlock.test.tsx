import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import type { TickTrace } from "../../types/trace.ts";
import { VOTE_SINK_COUNT } from "../../types/trace.ts";
import { VoteSurfaceBlock } from "./VoteSurfaceBlock.tsx";

function makeTick(votes: number[], commitCounts: number[]): TickTrace {
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
		final_actions: ["NoOp"],
		termination_reason: "NoTargets",
		priority_bid: 0,
		votes,
		commit_counts: commitCounts,
	};
}

describe("VoteSurfaceBlock", () => {
	it("lists only the non-zero vote entries with their catalog labels", () => {
		const votes = Array.from({ length: VOTE_SINK_COUNT }, () => 0);
		votes[0] = 1.5;
		votes[3] = -2.25;
		votes[26] = 0.5;
		render(<VoteSurfaceBlock tick={makeTick(votes, [0, 0, 0, 0])} />);

		expect(screen.getByText("Eat: 1.500")).toBeInTheDocument();
		expect(screen.getByText("Move[2]: -2.250")).toBeInTheDocument();
		expect(screen.getByText("Decide: 0.500")).toBeInTheDocument();
		expect(screen.queryByText(/Terminate:/)).not.toBeInTheDocument();
	});

	it("says so when the tick recorded no votes, and always shows the four counters", () => {
		const votes = Array.from({ length: VOTE_SINK_COUNT }, () => 0);
		render(<VoteSurfaceBlock tick={makeTick(votes, [0, 1, 2, 3])} />);

		expect(screen.getByText("no votes this tick")).toBeInTheDocument();
		expect(screen.getByText("Eat: 0")).toBeInTheDocument();
		expect(screen.getByText("Move: 1")).toBeInTheDocument();
		expect(screen.getByText("Reproduce: 2")).toBeInTheDocument();
		expect(screen.getByText("StealEnergy: 3")).toBeInTheDocument();
	});
});
