import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { SamplerPosition } from "../../stores/samplePlayback.ts";
import type { ExecutionSample, TickTrace } from "../../types/trace.ts";
import { ZERO_DECISION_INPUTS, ZERO_VOTES } from "../../types/trace.ts";
import { SamplerBar } from "./SamplerBar.tsx";

vi.mock("./TickTimeline.tsx", () => ({
	TickTimeline: () => <div data-testid="tick-timeline">TickTimeline</div>,
}));
vi.mock("./MeshHopTimeline.tsx", () => ({
	MeshHopTimeline: () => <div data-testid="mesh-hop-timeline">MeshHopTimeline</div>,
}));

function buildSample(tickCount = 2): ExecutionSample {
	const ticks: TickTrace[] = Array.from({ length: tickCount }, (_, i) => ({
		tick_number: i,
		energy_before: 10,
		energy_after: 9,
		static_inputs: {
			food_here: 0,
			neighbor_food: [0, 0, 0, 0],
			neighbor_barrier: [0, 0, 0, 0],
			neighbor_occupied: [0, 0, 0, 0],
			age_ticks: 5,
			previous_outcome: [0, 0, 0, 0],
		},
		hops: [
			{
				hop_index: 0,
				pass_index: 0,
				vote_contribution: ZERO_VOTES,
				decision_inputs: ZERO_DECISION_INPUTS,
				node_id: 1,
				input_refs: [],
				upstream_slots: [],
				energy_before: 10,
				energy_after: 9.5,
				output_slots: [],
				route: null,
				backend_trace: {
					Vm: {
						register_count: 4,
						constants: [],
						steps: [
							{
								pc: 0,
								instruction: { Add: { dst: 0, a: 1, b: 2 } },
								energy_cost: 0.1,
								energy_after: 9.9,
								register_changes: [],
							},
						],
						final_registers: [0, 0, 0, 0],
						final_payload: [],
						slot_writes: [],
					},
				},
			},
			{
				hop_index: 1,
				pass_index: 0,
				vote_contribution: ZERO_VOTES,
				decision_inputs: ZERO_DECISION_INPUTS,
				node_id: 2,
				input_refs: [],
				upstream_slots: [],
				energy_before: 9.5,
				energy_after: 9,
				output_slots: [],
				route: null,
				backend_trace: {
					Graph: {
						passes: [
							{
								pass_index: 0,
								energy_cost: 0.1,
								energy_after: 9.4,
								node_evaluations: [],
								max_delta: 0,
							},
						],
						temporal_committed: true,
						final_outputs: [],
						output_sinks: [],
					},
				},
			},
		],
		passes: [],
		final_actions: ["NoOp" as const],
		termination_reason: "NoDecision" as const,
		debug_perception: null,
		priority_bid: 0,
		commit_counts: [0, 0, 0, 0],
	}));

	return { creature_id: 1, ticks };
}

const defaultPosition: SamplerPosition = {
	tickIndex: 0,
	hopIndex: 0,
	detailIndex: 0,
};

const defaultProps = {
	sample: null as ExecutionSample | null,
	position: defaultPosition,
	playbackState: "idle" as const,
	playbackSpeed: 500,
	totalTicks: 0,
	totalHops: 0,
	samplingError: null as string | null,
	isDead: false,
	meshSemantics: null,
	onSample: vi.fn(),
	onResample: vi.fn(),
	onClear: vi.fn(),
	onPlay: vi.fn(),
	onPause: vi.fn(),
	onStepForward: vi.fn(),
	onStepBackward: vi.fn(),
	onSpeedChange: vi.fn(),
	onTickSelect: vi.fn(),
	onHopSelect: vi.fn(),
};

describe("SamplerBar", () => {
	it("renders Sample button in idle state", () => {
		render(<SamplerBar {...defaultProps} />);
		const button = screen.getByRole("button", { name: /sample/i });
		expect(button).toBeDefined();
		expect(button).not.toBeDisabled();
	});

	it("disables Sample button when dead with no sample", () => {
		render(<SamplerBar {...defaultProps} isDead={true} />);
		const button = screen.getByRole("button", { name: /sample/i });
		expect(button).toBeDisabled();
		expect(screen.getByText(/has died/i)).toBeDefined();
	});

	it("renders sampling spinner in sampling state", () => {
		render(<SamplerBar {...defaultProps} playbackState="sampling" />);
		expect(screen.getByText(/sampling/i)).toBeDefined();
	});

	it("renders error message with retry button", () => {
		render(<SamplerBar {...defaultProps} samplingError="Network error" />);
		expect(screen.getByText(/network error/i)).toBeDefined();
		expect(screen.getByRole("button", { name: /retry/i })).toBeDefined();
	});

	it("renders transport controls in active state", () => {
		const sample = buildSample(3);
		render(
			<SamplerBar
				{...defaultProps}
				sample={sample}
				playbackState="loaded"
				totalTicks={3}
				totalHops={2}
			/>,
		);
		// Transport buttons should be present
		expect(screen.getByRole("button", { name: /step backward/i })).toBeDefined();
		expect(screen.getByRole("button", { name: /play/i })).toBeDefined();
		expect(screen.getByRole("button", { name: /step forward/i })).toBeDefined();
		expect(screen.getByRole("button", { name: /resample/i })).toBeDefined();
		expect(screen.getByRole("button", { name: /clear/i })).toBeDefined();
	});

	it("calls onSample when Sample clicked", () => {
		const onSample = vi.fn();
		render(<SamplerBar {...defaultProps} onSample={onSample} />);
		fireEvent.click(screen.getByRole("button", { name: /sample/i }));
		expect(onSample).toHaveBeenCalledOnce();
	});

	it("calls onPlay when play button clicked", () => {
		const onPlay = vi.fn();
		const sample = buildSample();
		render(
			<SamplerBar
				{...defaultProps}
				sample={sample}
				playbackState="loaded"
				totalTicks={2}
				totalHops={2}
				onPlay={onPlay}
			/>,
		);
		fireEvent.click(screen.getByRole("button", { name: /play/i }));
		expect(onPlay).toHaveBeenCalledOnce();
	});

	it("calls onPause when pause button clicked", () => {
		const onPause = vi.fn();
		const sample = buildSample();
		render(
			<SamplerBar
				{...defaultProps}
				sample={sample}
				playbackState="playing"
				totalTicks={2}
				totalHops={2}
				onPause={onPause}
			/>,
		);
		fireEvent.click(screen.getByRole("button", { name: /pause/i }));
		expect(onPause).toHaveBeenCalledOnce();
	});

	it("renders position indicator with tick/hop counts", () => {
		const sample = buildSample(5);
		render(
			<SamplerBar
				{...defaultProps}
				sample={sample}
				playbackState="loaded"
				position={{ tickIndex: 1, hopIndex: 2, detailIndex: 0 }}
				totalTicks={5}
				totalHops={3}
			/>,
		);
		expect(screen.getByText(/T:2\/5/)).toBeDefined();
		expect(screen.getByText(/H:3\/3/)).toBeDefined();
	});

	it("renders timeline components when sample active", () => {
		const sample = buildSample();
		render(
			<SamplerBar
				{...defaultProps}
				sample={sample}
				playbackState="loaded"
				totalTicks={2}
				totalHops={2}
			/>,
		);
		expect(screen.getByTestId("tick-timeline")).toBeDefined();
		expect(screen.getByTestId("mesh-hop-timeline")).toBeDefined();
	});

	it("keeps a ten-pass tick's passes and details reachable in a bounded vertical scroll body", () => {
		const sample = buildSample(1);
		const tick = sample.ticks[0];
		if (!tick) throw new Error("buildSample(1) must produce one tick");
		tick.passes = Array.from({ length: 10 }, (_, pass_index) => ({
			pass_index,
			end_reason: "NoTargets" as const,
			votes: ZERO_VOTES,
			effective_votes: [0, 0, 0, 0],
			committed: null,
			hops: 1,
		}));
		render(
			<SamplerBar
				{...defaultProps}
				sample={sample}
				playbackState="loaded"
				totalTicks={1}
				totalHops={2}
			/>,
		);
		const body = screen.getByTestId("sampler-body");
		expect(body.className).toMatch(/(^|\s)max-h-\S+/);
		expect(body).toHaveClass("overflow-y-auto");
		expect(body).toContainElement(screen.getByTestId("mesh-hop-timeline"));
		expect(body).toContainElement(screen.getByTestId("pass-detail-9"));
		expect(body).not.toContainElement(screen.getByRole("button", { name: /resample/i }));
	});
});
