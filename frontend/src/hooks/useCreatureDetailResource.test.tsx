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
