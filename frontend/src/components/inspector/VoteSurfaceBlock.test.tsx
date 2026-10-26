import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { kindCounts, zeroVotes } from "../../test/traceFixtures.ts";
import type { DirectedVoteKind } from "../../types/generated/DirectedVoteKind.ts";
import type { Direction } from "../../types/generated/Direction.ts";
import type { MeshPassTracePayload } from "../../types/generated/MeshPassTracePayload.ts";
import type { TickTracePayload } from "../../types/generated/TickTracePayload.ts";
import type { VoteSink } from "../../types/generated/VoteSink.ts";
import type { VoteKind } from "../../types/vote-sinks.ts";
import { VOTE_KINDS, driveSinkIndex, voteSinkIndex } from "../../types/vote-sinks.ts";
import { VoteSurfaceBlock } from "./VoteSurfaceBlock.tsx";

function makeTick(
	passes: MeshPassTracePayload[],
	commitCounts: ReturnType<typeof kindCounts>,
	heading: Direction = "N",
): TickTracePayload {
	return {
		tick_number: 3,
		heading,
		energy_before: 10,
		energy_after: 9,
		static_inputs: {
			food_here: 0,
			neighbor_food: [0, 0, 0, 0, 0, 0, 0, 0],
			neighbor_barrier: [0, 0, 0, 0, 0, 0, 0, 0],
			neighbor_occupied: [0, 0, 0, 0, 0, 0, 0, 0],
			age_ticks: 0,
			previous_outcome: [0, 0, 0, 0],
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

/** A vote vector, zero except at the given sinks. */
function votesAt(entries: readonly [number, number][] = []): number[] {
	const votes = zeroVotes();
	for (const [index, value] of entries) votes[index] = value;
	return votes;
}

/** A per-kind row in `VOTE_KINDS` order, zero for every kind not given. */
function perKind(values: Partial<Record<VoteKind, number>> = {}) {
	return kindCounts(values);
}

/** Each kind's best sink: Eat, and each directed kind at the given
 * direction, else 0. */
function bestAt(directions: Partial<Record<DirectedVoteKind, number>> = {}): VoteSink[] {
	return VOTE_KINDS.map((kind) =>
		kind === "Eat" ? "Eat" : { Directed: [kind, directions[kind] ?? 0] },
	);
}

const moveAt = (d: number) => voteSinkIndex({ Directed: ["Move", d] });
const reproduceAt = (d: number) => voteSinkIndex({ Directed: ["Reproduce", d] });

function makePass(
	fields: Partial<MeshPassTracePayload> & Pick<MeshPassTracePayload, "pass_index">,
): MeshPassTracePayload {
	return {
		end_reason: "NoTargets",
		votes: votesAt(),
		best: bestAt(),
		bars: perKind(),
		effective_votes: perKind(),
		committed: null,
		hops: 1,
		...fields,
	};
}

describe("VoteSurfaceBlock", () => {
	it("shows each pass's reason, commit, kind votes, and Terminate and Decide as their own fields", () => {
		const votes = votesAt([
			[voteSinkIndex("Eat"), 1.5],
			[moveAt(2), -2.25],
			[voteSinkIndex("Decide"), 0.5],
		]);
		render(
			<VoteSurfaceBlock
				tick={makeTick(
					[
						makePass({
							pass_index: 0,
							end_reason: "Decided",
							votes,
							effective_votes: perKind({ Eat: 1.5 }),
							committed: { Eat: { type_idx: 0, leave: 0 } },
							hops: 2,
						}),
						makePass({
							pass_index: 1,
							effective_votes: perKind({ Eat: -1 }),
							bars: perKind({ Eat: 1 }),
						}),
					],
					perKind({ Eat: 1 }),
				)}
			/>,
		);

		expect(screen.getByText("Action Selection · NoDecision")).toBeInTheDocument();
		expect(screen.getByText("commit Eat(food0)")).toBeInTheDocument();
		expect(screen.getByText("no commit")).toBeInTheDocument();
		expect(screen.getByText("Eat: 1.500")).toBeInTheDocument();
		expect(screen.getByText("Move[E]: -2.250")).toBeInTheDocument();
		expect(screen.getByText("no kind votes this pass")).toBeInTheDocument();
		// Decide and Terminate are fields of the pass, never kind votes.
		expect(screen.queryByText("Decide: 0.500")).not.toBeInTheDocument();
		expect(screen.getByTestId("decide-0")).toHaveTextContent("Decide 0.500");
		expect(screen.getByTestId("terminate-0")).toHaveTextContent("Terminate 0.000");
		expect(screen.getByTestId("decided-0")).toHaveTextContent("ended by Decide 0.500");
		expect(screen.queryByTestId("decided-1")).not.toBeInTheDocument();
		expect(screen.queryByTestId("terminate-verdict")).not.toBeInTheDocument();
	});

	it("shows the recorded best sink, drive, bar and effective vote, not a recomputation", () => {
		const votes = votesAt([
			[moveAt(0), 0.9], // tied with Move[3]
			[moveAt(3), 0.9], // the sink the heading tie recorded
			[driveSinkIndex("Move"), 0.4],
		]);
		render(
			<VoteSurfaceBlock
				tick={makeTick(
					[
						makePass({
							pass_index: 0,
							votes,
							best: bestAt({ Move: 3 }),
							bars: perKind({ Move: 2 }),
							// Deliberately not drive + best − bar: the view must show what was recorded.
							effective_votes: perKind({ Move: 0.125 }),
						}),
					],
					perKind({ Move: 2 }),
					"SE",
				)}
			/>,
		);

		expect(screen.getByTestId("effective-0-Move")).toHaveTextContent(
			"Move drive 0.400 + Move[SE] 0.900 − 2 = 0.125",
		);
		expect(screen.getByText("Drive Move: 0.400")).toBeInTheDocument();
		expect(screen.getByTestId("tick-heading")).toHaveTextContent("heading at tick start SE");
	});

	it("shows a kind whose only vote is its drive", () => {
		const votes = votesAt([[driveSinkIndex("Reproduce"), 0.7]]);
		render(
			<VoteSurfaceBlock
				tick={makeTick(
					[
						makePass({
							pass_index: 0,
							votes,
							best: bestAt({ Reproduce: 1 }),
							effective_votes: perKind({ Reproduce: 0.7 }),
							committed: { Reproduce: { direction: "NE", energy_transfer_fraction: 0 } },
						}),
					],
					perKind({ Reproduce: 1 }),
					"NE",
				)}
			/>,
		);

		expect(screen.getByTestId("effective-0-Reproduce")).toHaveTextContent(
			"Reproduce drive 0.700 + Reproduce[NE] 0.000 − 0 = 0.700",
		);
		expect(screen.getByTestId("effective-0-Reproduce")).toHaveClass("text-orange-300");
		expect(screen.queryByTestId("effective-0-Move")).not.toBeInTheDocument();
	});

	it("shows Terminate against the winning effective vote on the pass that ends a TerminateVoted tick", () => {
		const votes = votesAt([
			[voteSinkIndex("Eat"), 0.4],
			[voteSinkIndex("Terminate"), 0.75],
		]);
		const tick = makeTick(
			[
				makePass({
					pass_index: 0,
					votes,
					bars: perKind({ Move: 1 }),
					effective_votes: perKind({ Eat: 0.4, Move: -1, Reproduce: 0.2 }),
					hops: 3,
				}),
			],
			perKind({ Move: 1 }),
		);
		tick.termination_reason = "TerminateVoted";
		render(<VoteSurfaceBlock tick={tick} />);

		expect(screen.getByTestId("terminate-verdict")).toHaveTextContent(
			"Terminate 0.750 ≥ winning effective 0.400",
		);
	});

	it("shows each pass's effective vote per kind against its recorded bar so the commits line up", () => {
		const votes = votesAt([
			[moveAt(2), 0.52],
			[reproduceAt(2), 0.026],
		]);
		const best = bestAt({ Move: 2, Reproduce: 2 });
		const pass = (
			pass_index: number,
			bars: ReturnType<typeof kindCounts>,
			effective_votes: ReturnType<typeof kindCounts>,
			committed: MeshPassTracePayload["committed"],
		): MeshPassTracePayload =>
			makePass({ pass_index, votes, best, bars, effective_votes, committed, hops: 3 });
		render(
			<VoteSurfaceBlock
				tick={makeTick(
					[
						pass(0, perKind(), perKind({ Move: 0.52, Reproduce: 0.026 }), { Move: "E" }),
						pass(1, perKind({ Move: 1 }), perKind({ Move: -0.48, Reproduce: 0.026 }), {
							Reproduce: { direction: "E", energy_transfer_fraction: 0.5 },
						}),
						pass(
							2,
							perKind({ Move: 1, Reproduce: 1 }),
							perKind({ Move: -0.48, Reproduce: -0.974 }),
							null,
						),
					],
					perKind({ Move: 1, Reproduce: 1 }),
				)}
			/>,
		);

		expect(screen.getByTestId("effective-0-Move")).toHaveTextContent(
			"Move drive 0.000 + Move[E] 0.520 − 0 = 0.520",
		);
		expect(screen.getByTestId("effective-0-Move")).toHaveClass("text-orange-300");
		expect(screen.getByTestId("effective-1-Move")).toHaveTextContent(
			"Move drive 0.000 + Move[E] 0.520 − 1 = -0.480",
		);
		expect(screen.getByTestId("effective-1-Reproduce")).toHaveTextContent(
			"Reproduce drive 0.000 + Reproduce[E] 0.026 − 0 = 0.026",
		);
		expect(screen.getByTestId("effective-1-Reproduce")).toHaveClass("text-orange-300");
		expect(screen.getByTestId("effective-2-Reproduce")).toHaveTextContent(
			"Reproduce drive 0.000 + Reproduce[E] 0.026 − 1 = -0.974",
		);
		expect(screen.getByTestId("effective-2-Reproduce")).not.toHaveClass("text-orange-300");
		expect(screen.queryByTestId("effective-0-Eat")).not.toBeInTheDocument();
	});

	it("says so when the tick ran no passes, and always shows the four bars", () => {
		render(
			<VoteSurfaceBlock tick={makeTick([], perKind({ Move: 1, Reproduce: 2, StealEnergy: 3 }))} />,
		);

		expect(screen.getByText("no passes this tick")).toBeInTheDocument();
		expect(screen.getByText("Eat: 0")).toBeInTheDocument();
		expect(screen.getByText("Move: 1")).toBeInTheDocument();
		expect(screen.getByText("Reproduce: 2")).toBeInTheDocument();
		expect(screen.getByText("StealEnergy: 3")).toBeInTheDocument();
	});
});
