import { act, render } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { ApiRequestError, api } from "../api/rest.ts";
import { useCreatureInspectorStore } from "../stores/creatureInspector.ts";
import { useSimulationStore } from "../stores/simulation.ts";
import { useCreatureDetailResource } from "./useCreatureDetailResource.ts";

vi.mock("../api/rest.ts", () => ({
	api: {
		getCreature: vi.fn(),
	},
	ApiRequestError: class ApiRequestError extends Error {
		status: number;
		body: { error: { code: string; message: string } };

		constructor(status: number, body: { error: { code: string; message: string } }) {
			super(body.error.message);
			this.name = "ApiRequestError";
			this.status = status;
			this.body = body;
		}
	},
}));

function Harness() {
	useCreatureDetailResource();
	return null;
}

function buildCreatureDetail() {
	return {
		protocol_version: "v3alpha2",
		id: 7,
		position: { x: 10, y: 12 },
		energy: 42,
		max_energy: 100,
		reproductive_reserve: 3,
		reproductive_reserve_capacity: 8,
		age: 8,
		generation: 2,
		complexity: 5,
		genome_size: 2,
		phenotype: {
			channels: [10, 20, 30, 40, 50, 60] as [number, number, number, number, number, number],
			active_channel: 0,
			polarity: [true, false, true, false, true, false] as [
				boolean,
				boolean,
				boolean,
				boolean,
				boolean,
				boolean,
			],
			rgb: [1, 2, 3] as [number, number, number],
		},
		genome: {
			entry_node_id: 2,
			nodes: [],
		},
		mesh_annotations: [],
		shared_memory: [0, 1],
		action_log: [],
		diagnostics: {
			current_inputs: {
				food_here: 0.1,
				neighbor_food: [0, 0, 0, 0, 0, 0, 0, 0],
				neighbor_barrier: [0, 1, 0, 0, 0, 0, 0, 0],
				neighbor_occupied: [0, 0, 0, 1, 0, 0, 0, 0],
			},
			live_circuit: {
				reachable_node_count: 2,
				stateful_reachable_node_count: 1,
				barrier_reader_reachable_node_count: 1,
				distinct_upstream_slots_read: [0],
				distinct_payload_slots_written: [0],
				distinct_custom_output_slots_written: [],
				reachable_read_class_counts: { barrier: 1 },
				reachable_write_class_counts: { action: 1 },
			},
			recent_actions: {
				sampled_entries: 3,
				blocked_move_count: 1,
				invalid_target_reproduce_count: 0,
				by_action_result: { "Move:Blocked": 1 },
			},
		},
		latest_tick: 2732,
	};
}

describe("useCreatureDetailResource", () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.useFakeTimers();
		vi.setSystemTime(new Date("2026-03-07T10:00:00Z"));
		useSimulationStore.getState().reset();
		useCreatureInspectorStore.getState().clearSelection();
	});

	afterEach(() => {
		vi.useRealTimers();
	});

	it("updates store with new data when tick changes via setStatus", async () => {
		const detail1 = buildCreatureDetail();
		const detail2 = {
			...buildCreatureDetail(),
			energy: 35,
			age: 9,
			position: { x: 11, y: 13 },
			latest_tick: 2733,
			action_log: [
				{
					tick: 2733,
					action_type: "Move" as const,
					result: "Success" as const,
					direction: 2,
					energy_before: 42,
					energy_after: 35,
					amount: 0,
					food_type: null,
					priority_bid: 0.5,
				},
			],
		};
		vi.mocked(api.getCreature).mockResolvedValueOnce(detail1).mockResolvedValueOnce(detail2);

		useCreatureInspectorStore.getState().selectCreature(7);
		await act(async () => {
			render(<Harness />);
			await Promise.resolve();
		});

		expect(api.getCreature).toHaveBeenCalledTimes(1);
		expect(useCreatureInspectorStore.getState().liveDetail.stats?.energy).toBe(42);
		expect(useCreatureInspectorStore.getState().liveDetail.stats?.age).toBe(8);
		expect(
			useCreatureInspectorStore.getState().liveDetail.diagnostics?.live_circuit
				.reachable_node_count,
		).toBe(2);

		// Simulate tick change via setStatus (production path)
		act(() => {
			vi.advanceTimersByTime(200);
			useSimulationStore.getState().setStatus(1, 2733, {
				state: "running",
				population: 50,
				mean_energy: 40,
				last_tick_actions: {
					move: 10,
					eat: 5,
					reproduce: 2,
					noop: 30,
					steal: 0,
					predation_kills: 0,
				},
				reproduction_actions_attempted_total: 0,
				reproduction_actions_spawned_total: 0,
				reproduction_actions_rejected_total: 0,
				predation_actions_attempted_total: 0,
				predation_actions_transferred_total: 0,
				predation_actions_rejected_total: 0,
				predation_kills_total: 0,
				predation_actions_by_result: {},
				mutation_events_attempted_total: 0,
				mutation_events_applied_total: 0,
				mutation_events_skipped_total: 0,
				mutation_events_attempted_total_by_domain: {},
				mutation_events_applied_total_by_domain: {},
				mutation_events_attempted_total_by_operator: {},
				mutation_events_applied_total_by_operator: {},
				mutation_events_applied_total_semantic_noop: 0,
				mutation_events_applied_total_semantic_change: 0,
				last_tick_compute_energy_total_mean: 0,
				last_tick_compute_energy_total_min: 0,
				last_tick_compute_energy_total_max: 0,
				last_tick_compute_energy_vm_mean: 0,
				last_tick_compute_energy_graph_mean: 0,
				perf: {
					projection_publish_ms: 0,
					ws_frame_publish_ms: 0,
					subscriber_count: 1,
				},
			});
		});

		await act(async () => {
			await Promise.resolve();
		});

		expect(api.getCreature).toHaveBeenCalledTimes(2);
		expect(useCreatureInspectorStore.getState().liveDetail.stats?.energy).toBe(35);
		expect(useCreatureInspectorStore.getState().liveDetail.stats?.age).toBe(9);
	});

	it("stops incremental polling after the selected creature is marked dead", async () => {
		vi.mocked(api.getCreature)
			.mockResolvedValueOnce(buildCreatureDetail())
			.mockRejectedValueOnce(
				new ApiRequestError(404, {
					protocol_version: "v3alpha2",
					error: { code: "invalid_request", message: "creature not found" },
				}),
			);

		useCreatureInspectorStore.getState().selectCreature(7);
		await act(async () => {
			render(<Harness />);
			await Promise.resolve();
		});

		expect(api.getCreature).toHaveBeenCalledTimes(1);

		act(() => {
			vi.advanceTimersByTime(200);
			useSimulationStore.getState().setTick(2733);
		});

		await act(async () => {
			await Promise.resolve();
		});

		expect(api.getCreature).toHaveBeenCalledTimes(2);
		expect(useCreatureInspectorStore.getState().resourceMeta.isDead).toBe(true);

		act(() => {
			vi.advanceTimersByTime(200);
			useSimulationStore.getState().setTick(2734);
		});

		await act(async () => {
			await Promise.resolve();
		});

		expect(api.getCreature).toHaveBeenCalledTimes(2);
	});
});
