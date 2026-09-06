import { act, renderHook } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useSamplePlaybackStore } from "../../../stores/samplePlayback.ts";
import { useSampleSessionStore } from "../../../stores/sampleSession.ts";
import type { ExecutionSample } from "../../../types/trace.ts";
import { useInspectorSampler } from "./useInspectorSampler.ts";

const mockClearSampling = vi.fn();
const mockStartSampling = vi.fn();

vi.mock("../../../hooks/useExecutionSampler.ts", () => ({
	useExecutionSampler: () => ({
		clearSampling: mockClearSampling,
		startSampling: mockStartSampling,
	}),
}));

function buildSample(creatureId: number, tickCount = 2): ExecutionSample {
	const ticks = Array.from({ length: tickCount }, (_, i) => ({
		tick_number: i,
		energy_before: 10,
		energy_after: 9,
		static_inputs: {
			food_here: 0,
			neighbor_food: [0, 0, 0, 0],
			neighbor_barrier: [0, 0, 0, 0],
			neighbor_occupied: [0, 0, 0, 0],
			generation: 1,
			age_ticks: 5,
		},
		hops: [
			{
				hop_index: 0,
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
							{
								pc: 1,
								instruction: { Add: { dst: 0, a: 1, b: 2 } },
								energy_cost: 0.1,
								energy_after: 9.8,
								register_changes: [],
							},
						],
						final_registers: [0, 0, 0, 0],
						final_payload: [],
						final_meta: [],
						slot_writes: [],
					},
				},
			},
			{
				hop_index: 1,
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
						converged: false,
						temporal_committed: true,
						stable_passes_count: 0,
						final_outputs: [],
						output_sinks: [],
						action_slots: [],
						execute_gate: {
							wired: false,
							weighted_sum: 0,
							queue_non_empty: false,
							fired: false,
						},
					},
				},
			},
		],
		final_actions: ["NoOp" as const],
		termination_reason: "NoTargets" as const,
		debug_perception: null,
		priority_bid: 0,
	}));

	return { creature_id: creatureId, ticks };
}

describe("useInspectorSampler", () => {
	beforeEach(() => {
		useSamplePlaybackStore.getState().clear();
		useSampleSessionStore.getState().clear();
		mockClearSampling.mockClear();
		mockStartSampling.mockClear();
	});

	it("returns idle playback state when no sample exists", () => {
		const { result } = renderHook(() => useInspectorSampler(1));

		expect(result.current.sample).toBeNull();
		expect(result.current.playbackState).toBe("idle");
		expect(result.current.totalTicks).toBe(0);
		expect(result.current.totalHops).toBe(0);
		expect(result.current.totalDetails).toBe(0);
		expect(result.current.samplingError).toBeNull();
	});

	it("returns 'sampling' playbackState when session is recording", () => {
		act(() => {
			useSampleSessionStore.getState().setRecording(1);
		});

		const { result } = renderHook(() => useInspectorSampler(1));

		expect(result.current.playbackState).toBe("sampling");
	});

	it("derives totalTicks from sample", () => {
		const sample = buildSample(1, 3);

		act(() => {
			useSamplePlaybackStore.getState().setSample(sample);
		});

		const { result } = renderHook(() => useInspectorSampler(1));

		expect(result.current.totalTicks).toBe(3);
		expect(result.current.sample).toBe(sample);
	});

	it("handleSample calls startSampling with creatureId", () => {
		const { result } = renderHook(() => useInspectorSampler(42));

		act(() => {
			result.current.handleSample();
		});

		expect(mockStartSampling).toHaveBeenCalledWith(42);
	});

	it("handleResample clears then starts sampling", () => {
		const { result } = renderHook(() => useInspectorSampler(42));

		act(() => {
			result.current.handleResample();
		});

		expect(mockClearSampling).toHaveBeenCalled();
		expect(mockStartSampling).toHaveBeenCalledWith(42);

		// clearSampling should be called before startSampling
		const clearOrder = mockClearSampling.mock.invocationCallOrder[0] ?? 0;
		const startOrder = mockStartSampling.mock.invocationCallOrder[0] ?? 0;
		expect(clearOrder).toBeLessThan(startOrder);
	});

	it("auto-clears sample when creatureId changes", () => {
		const { rerender } = renderHook(
			({ creatureId }: { creatureId: number | null }) => useInspectorSampler(creatureId),
			{ initialProps: { creatureId: 1 } },
		);

		expect(mockClearSampling).not.toHaveBeenCalled();

		rerender({ creatureId: 2 });

		expect(mockClearSampling).toHaveBeenCalledTimes(1);
	});

	it("handleTickSelect jumps to correct position", () => {
		const sample = buildSample(1, 3);
		act(() => {
			useSamplePlaybackStore.getState().setSample(sample);
		});

		const { result } = renderHook(() => useInspectorSampler(1));

		act(() => {
			result.current.handleTickSelect(2);
		});

		const position = useSamplePlaybackStore.getState().position;
		expect(position).toEqual({ tickIndex: 2, hopIndex: 0, detailIndex: 0 });
	});

	it("returns stable action refs from getState", () => {
		const { result } = renderHook(() => useInspectorSampler(1));

		expect(typeof result.current.clearSample).toBe("function");
		expect(typeof result.current.setPlaying).toBe("function");
		expect(typeof result.current.setPaused).toBe("function");
		expect(typeof result.current.stepForward).toBe("function");
		expect(typeof result.current.stepBackward).toBe("function");
		expect(typeof result.current.setPlaybackSpeed).toBe("function");
	});
});
