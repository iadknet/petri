import { act, render } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { ApiRequestError, api } from "../api/rest.ts";
import { useConfigStore } from "../stores/config.ts";
import {
	creatureInspectorSelectors,
	useCreatureInspectorStore,
} from "../stores/creatureInspector.ts";
import { useSimulationStore } from "../stores/simulation.ts";
import { MOCK_CONFIG, MOCK_STATUS } from "../test/fixtures.ts";
import type { CreatureDetailResponse } from "../types/generated/CreatureDetailResponse.ts";
import type { NodeLearning } from "../types/generated/NodeLearning.ts";
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

function buildCreatureDetail(): CreatureDetailResponse {
	return {
		protocol_version: "v3alpha2",
		id: 7,
		position: { x: 10, y: 12 },
		energy: 42,
		max_energy: 100,
		steal_share: 0.2,
		give_share: 0.2,
		age: 8,
		generation: 2,
		heading: "N",
		complexity: 5,
		genome_size: 2,
		phenotype: {
			channels: [10, 20, 30, 40, 50, 60],
			active_channel: 0,
			polarity: [true, false, true, false, true, false],
			rgb: [1, 2, 3],
		},
		brain: {
			entry_node_id: 2,
			nodes: [],
		},
		brain_context: { food_type_count: 1, action_queue_slots: 10 },
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
				barrier_decision_writer_reachable_node_count: 0,
				barrier_reader_without_decision_writer_reachable_node_count: 1,
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
		useConfigStore.getState().reset();
		useCreatureInspectorStore.getState().clearSelection();
	});

	afterEach(() => {
		vi.useRealTimers();
	});

	it("updates store with new data when tick changes via setStatus", async () => {
		const detail1 = buildCreatureDetail();
		const detail2: CreatureDetailResponse = {
			...buildCreatureDetail(),
			energy: 35,
			age: 9,
			position: { x: 11, y: 13 },
			latest_tick: 2733,
			action_log: [
				{
					tick: 2733,
					action_type: "Move" as const,
					outcome: "Success" as const,
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
				...MOCK_STATUS,
				state: "running",
				population: 50,
				mean_energy: 40,
				last_tick_actions: {
					move: 10,
					eat: 5,
					reproduce: 2,
					noop: 30,
					steal: 0,
					give: 0,
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
				give_actions_attempted_total: 0,
				give_actions_transferred_total: 0,
				give_actions_rejected_total: 0,
				give_actions_by_result: {},
				mutation_events_attempted_total: 0,
				mutation_events_applied_total: 0,
				mutation_events_skipped_total: 0,
				mutation_events_attempted_total_by_domain: {},
				mutation_events_applied_total_by_domain: {},
				mutation_events_attempted_total_by_operator: {},
				mutation_events_applied_total_by_operator: {},
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

	it("replaces learning on every poll, which excludes the brain only, and clears it on a new selection", async () => {
		const learning: NodeLearning[] = [
			{
				node_id: 2,
				edges: [{ surface: "sink", container: 65, edge: 0, applied: 0.5, initialized: false }],
				scales: [],
			},
		];
		// The weights in force move between polls.
		const polledLearning: NodeLearning[] = [
			{
				node_id: 2,
				edges: [
					{
						surface: "sink",
						container: 65,
						edge: 0,
						applied: 0.75,
						learned: 0.75,
						initialized: true,
					},
				],
				scales: [],
			},
		];
		const { brain: _brain, ...polled } = {
			...buildCreatureDetail(),
			latest_tick: 2733,
			learning: polledLearning,
		};
		vi.mocked(api.getCreature)
			.mockResolvedValueOnce({ ...buildCreatureDetail(), learning })
			.mockResolvedValueOnce(polled)
			.mockResolvedValueOnce({ ...buildCreatureDetail(), id: 8 });

		useCreatureInspectorStore.getState().selectCreature(7);
		await act(async () => {
			render(<Harness />);
			await Promise.resolve();
		});
		const stored = () =>
			creatureInspectorSelectors.creatureLearning(useCreatureInspectorStore.getState());
		expect(stored()).toEqual(learning);

		act(() => {
			vi.advanceTimersByTime(200);
			useSimulationStore.getState().setTick(2733);
		});
		await act(async () => {
			await Promise.resolve();
		});
		expect(api.getCreature).toHaveBeenCalledTimes(2);
		expect(vi.mocked(api.getCreature).mock.calls[1]?.[2]).toEqual({
			since_tick: 2732,
			exclude: "brain",
		});
		expect(stored()).toEqual(polledLearning);

		act(() => {
			useCreatureInspectorStore.getState().selectCreature(8);
		});
		expect(stored()).toBeNull();
		await act(async () => {
			await Promise.resolve();
		});
		expect(api.getCreature).toHaveBeenCalledTimes(3);
		expect(stored()).toBeNull();
	});

	it("refetches the brain when a poll's brain context differs from the held brain's", async () => {
		const wider = { food_type_count: 1, action_queue_slots: 20 };
		const { brain: _brain, ...polled } = {
			...buildCreatureDetail(),
			latest_tick: 2733,
			brain_context: wider,
		};
		const refetchedBrain = { entry_node_id: 9, nodes: [] };
		vi.mocked(api.getCreature)
			.mockResolvedValueOnce(buildCreatureDetail())
			.mockResolvedValueOnce(polled)
			.mockResolvedValueOnce({
				...buildCreatureDetail(),
				latest_tick: 2733,
				brain: refetchedBrain,
				brain_context: wider,
			});

		useCreatureInspectorStore.getState().selectCreature(7);
		await act(async () => {
			render(<Harness />);
			await Promise.resolve();
		});
		act(() => {
			vi.advanceTimersByTime(200);
			useSimulationStore.getState().setTick(2733);
		});
		await act(async () => {
			await Promise.resolve();
		});
		await act(async () => {
			await Promise.resolve();
		});

		expect(api.getCreature).toHaveBeenCalledTimes(3);
		expect(vi.mocked(api.getCreature).mock.calls[2]?.[2]).toEqual({ since_tick: 2733 });
		const { brain, brainContext } = useCreatureInspectorStore.getState().staticDetail;
		expect(brain).toEqual(refetchedBrain);
		expect(brainContext).toEqual(wider);
	});

	it("refetches the brain when a paused config commit changes the queue, without a tick", async () => {
		const wider = { food_type_count: 1, action_queue_slots: 60 };
		const refetchedBrain = { entry_node_id: 9, nodes: [] };
		vi.mocked(api.getCreature)
			.mockResolvedValueOnce(buildCreatureDetail())
			.mockResolvedValueOnce({
				...buildCreatureDetail(),
				brain: refetchedBrain,
				brain_context: wider,
			});
		useConfigStore.getState().setServerConfig(MOCK_CONFIG, "paused");

		useCreatureInspectorStore.getState().selectCreature(7);
		await act(async () => {
			render(<Harness />);
			await Promise.resolve();
		});
		await act(async () => {
			useConfigStore
				.getState()
				.commitServerConfig(
					{ ...MOCK_CONFIG, runtime: { ...MOCK_CONFIG.runtime, max_actions_per_turn: 20 } },
					"paused",
				);
			await Promise.resolve();
		});

		expect(api.getCreature).toHaveBeenCalledTimes(2);
		expect(vi.mocked(api.getCreature).mock.calls[1]?.[2]).toEqual({ since_tick: 2732 });
		const { brain, brainContext } = useCreatureInspectorStore.getState().staticDetail;
		expect(brain).toEqual(refetchedBrain);
		expect(brainContext).toEqual(wider);
	});

	it("lands a brain refresh while ticks keep arriving faster than every response", async () => {
		const wider = { food_type_count: 1, action_queue_slots: 20 };
		const refetchedBrain = { entry_node_id: 9, nodes: [] };
		// Every poll after the first resolves 150 ms later under the wider
		// context, and honours its abort signal.
		vi.mocked(api.getCreature)
			.mockResolvedValueOnce(buildCreatureDetail())
			.mockImplementation((_id, signal, query) => {
				const { brain: _brain, ...withoutBrain } = {
					...buildCreatureDetail(),
					latest_tick: 2733,
					brain_context: wider,
				};
				const response: CreatureDetailResponse =
					query?.exclude === "brain" ? withoutBrain : { ...withoutBrain, brain: refetchedBrain };
				return new Promise((resolve, reject) => {
					const timer = setTimeout(() => resolve(response), 150);
					signal?.addEventListener("abort", () => {
						clearTimeout(timer);
						reject(new DOMException("aborted", "AbortError"));
					});
				});
			});

		useCreatureInspectorStore.getState().selectCreature(7);
		await act(async () => {
			render(<Harness />);
			await Promise.resolve();
		});
		for (let tick = 2733; tick < 2743; tick++) {
			await act(async () => {
				await vi.advanceTimersByTimeAsync(100);
				useSimulationStore.getState().setTick(tick);
			});
		}

		const { brain, brainContext } = useCreatureInspectorStore.getState().staticDetail;
		expect(brain).toEqual(refetchedBrain);
		expect(brainContext).toEqual(wider);
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
