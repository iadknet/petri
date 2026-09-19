import { act, render } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { api } from "../api/rest.ts";
import { useSamplePlaybackStore } from "../stores/samplePlayback.ts";
import { useSampleSessionStore } from "../stores/sampleSession.ts";
import type { ExecutionSample, SampleResponse } from "../types/trace.ts";
import { useSampleSessionResource } from "./useSampleSessionResource.ts";

vi.mock("../api/rest.ts", () => ({
	api: {
		startSample: vi.fn(),
		getSample: vi.fn(),
	},
}));

let sessionResource: ReturnType<typeof useSampleSessionResource> | null = null;

function Harness() {
	sessionResource = useSampleSessionResource();
	return null;
}

function buildSample(creatureId: number): ExecutionSample {
	return {
		creature_id: creatureId,
		ticks: [
			{
				tick_number: 1,
				energy_before: 10,
				energy_after: 9,
				static_inputs: {
					food_here: 0,
					neighbor_food: [0, 0, 0, 0],
					neighbor_barrier: [0, 0, 0, 0],
					neighbor_occupied: [0, 0, 0, 0],
					age_ticks: 5,
				},
				hops: [],
				final_actions: ["NoOp"],
				termination_reason: "NoTargets",
				debug_perception: null,
				priority_bid: 0,
			},
		],
	};
}

function deferred<T>() {
	let resolve!: (value: T) => void;
	let reject!: (reason?: unknown) => void;
	const promise = new Promise<T>((resolvePromise, rejectPromise) => {
		resolve = resolvePromise;
		reject = rejectPromise;
	});

	return { promise, resolve, reject };
}

describe("useSampleSessionResource", () => {
	beforeEach(() => {
		sessionResource = null;
		useSamplePlaybackStore.getState().clear();
		useSampleSessionStore.getState().clear();
		vi.clearAllMocks();
		vi.useFakeTimers();
	});

	afterEach(() => {
		vi.useRealTimers();
	});

	it("ignores late sample completion after clearing the active session", async () => {
		const getSampleRequest = deferred<SampleResponse>();

		vi.mocked(api.startSample).mockResolvedValue({
			protocol_version: "v3alpha2",
			status: "recording",
			ticks_requested: 5,
			include_perception_debug: false,
		});
		vi.mocked(api.getSample).mockImplementation(() => getSampleRequest.promise);

		render(<Harness />);

		if (!sessionResource) {
			throw new Error("expected sample session resource to be available");
		}

		await act(async () => {
			await sessionResource?.startSampling(7);
		});

		expect(useSampleSessionStore.getState().status).toBe("recording");
		expect(useSamplePlaybackStore.getState().sample).toBeNull();

		await act(async () => {
			vi.advanceTimersByTime(200);
			await Promise.resolve();
		});

		expect(api.getSample).toHaveBeenCalledWith(7, expect.any(AbortSignal));

		act(() => {
			sessionResource?.clearSampling();
		});

		expect(useSampleSessionStore.getState().status).toBe("idle");

		await act(async () => {
			getSampleRequest.resolve({
				protocol_version: "v3alpha2",
				status: "complete",
				sample: buildSample(7),
			});
			await Promise.resolve();
		});

		expect(useSamplePlaybackStore.getState().sample).toBeNull();
		expect(useSampleSessionStore.getState().status).toBe("idle");
	});
});
