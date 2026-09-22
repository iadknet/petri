import { beforeEach, describe, expect, it } from "vitest";
import type { ExecutionSample } from "../types/trace.ts";
import { ZERO_VOTES } from "../types/trace.ts";
import { useSamplePlaybackStore } from "./samplePlayback.ts";

function makeSample(): ExecutionSample {
	return {
		creature_id: 7,
		ticks: [
			{
				tick_number: 1,
				energy_before: 10,
				energy_after: 9,
				static_inputs: {
					food_here: 0,
					neighbor_food: [],
					neighbor_barrier: [],
					neighbor_occupied: [],
					age_ticks: 0,
				},
				hops: [
					{
						hop_index: 0,
						vote_contribution: ZERO_VOTES,
						node_id: 1,
						input_refs: [],
						upstream_slots: [],
						energy_before: 10,
						energy_after: 9.8,
						output_slots: [],
						route: null,
						backend_trace: {
							Vm: {
								register_count: 2,
								constants: [],
								steps: [
									{
										pc: 0,
										instruction: "Noop",
										energy_cost: 0.1,
										energy_after: 9.9,
										register_changes: [],
									},
									{
										pc: 1,
										instruction: "Halt",
										energy_cost: 0.1,
										energy_after: 9.8,
										register_changes: [],
									},
								],
								final_registers: [],
								final_payload: [],
								final_meta: [],
								slot_writes: [],
							},
						},
					},
					{
						hop_index: 1,
						vote_contribution: ZERO_VOTES,
						node_id: 2,
						input_refs: [],
						upstream_slots: [],
						energy_before: 9.8,
						energy_after: 9.5,
						output_slots: [],
						route: null,
						backend_trace: {
							Graph: {
								passes: [
									{
										pass_index: 0,
										energy_cost: 0.1,
										energy_after: 9.7,
										node_evaluations: [],
										max_delta: 0.1,
									},
									{
										pass_index: 1,
										energy_cost: 0.2,
										energy_after: 9.5,
										node_evaluations: [],
										max_delta: 0.01,
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
				final_actions: ["NoOp"],
				termination_reason: "NoTargets",
				debug_perception: null,
				priority_bid: 0,
				votes: ZERO_VOTES,
				commit_counts: [0, 0, 0, 0],
			},
			{
				tick_number: 2,
				energy_before: 9.5,
				energy_after: 9,
				static_inputs: {
					food_here: 0,
					neighbor_food: [],
					neighbor_barrier: [],
					neighbor_occupied: [],
					age_ticks: 1,
				},
				hops: [
					{
						hop_index: 0,
						vote_contribution: ZERO_VOTES,
						node_id: 3,
						input_refs: [],
						upstream_slots: [],
						energy_before: 9.5,
						energy_after: 9,
						output_slots: [],
						route: null,
						backend_trace: {
							Vm: {
								register_count: 1,
								constants: [],
								steps: [
									{
										pc: 0,
										instruction: "Halt",
										energy_cost: 0.5,
										energy_after: 9,
										register_changes: [],
									},
								],
								final_registers: [],
								final_payload: [],
								final_meta: [],
								slot_writes: [],
							},
						},
					},
				],
				final_actions: ["NoOp"],
				termination_reason: "NoTargets",
				debug_perception: null,
				priority_bid: 0,
				votes: ZERO_VOTES,
				commit_counts: [0, 0, 0, 0],
			},
		],
	};
}

describe("samplePlaybackStore", () => {
	beforeEach(() => {
		useSamplePlaybackStore.getState().clear();
	});

	it("loads a sample and resets playback position", () => {
		const store = useSamplePlaybackStore.getState();
		store.setSample(makeSample());
		const state = useSamplePlaybackStore.getState();

		expect(state.sample?.creature_id).toBe(7);
		expect(state.playbackState).toBe("loaded");
		expect(state.position).toEqual({
			tickIndex: 0,
			hopIndex: 0,
			detailIndex: 0,
		});
	});

	it("steps forward across details, hops, and ticks", () => {
		const store = useSamplePlaybackStore.getState();
		store.setSample(makeSample());

		store.stepForward();
		expect(useSamplePlaybackStore.getState().position).toEqual({
			tickIndex: 0,
			hopIndex: 0,
			detailIndex: 1,
		});

		store.stepForward();
		expect(useSamplePlaybackStore.getState().position).toEqual({
			tickIndex: 0,
			hopIndex: 1,
			detailIndex: 0,
		});

		store.stepForward();
		expect(useSamplePlaybackStore.getState().position).toEqual({
			tickIndex: 0,
			hopIndex: 1,
			detailIndex: 1,
		});

		store.stepForward();
		expect(useSamplePlaybackStore.getState().position).toEqual({
			tickIndex: 1,
			hopIndex: 0,
			detailIndex: 0,
		});
	});

	it("steps backward across hops and ticks", () => {
		const store = useSamplePlaybackStore.getState();
		store.setSample(makeSample());
		store.jumpToPosition({ tickIndex: 1, hopIndex: 0, detailIndex: 0 });

		store.stepBackward();
		expect(useSamplePlaybackStore.getState().position).toEqual({
			tickIndex: 0,
			hopIndex: 1,
			detailIndex: 1,
		});

		store.stepBackward();
		expect(useSamplePlaybackStore.getState().position).toEqual({
			tickIndex: 0,
			hopIndex: 1,
			detailIndex: 0,
		});
	});
});
