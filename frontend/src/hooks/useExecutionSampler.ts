import { useCallback, useEffect, useRef } from "react";
import { api } from "../api/rest.ts";
import { useExecutionSamplerStore } from "../stores/executionSampler.ts";

const POLL_INTERVAL_MS = 200;

export function useExecutionSampler() {
	const abortRef = useRef<AbortController | null>(null);
	const pollTimerRef = useRef<ReturnType<typeof setInterval> | null>(null);
	const playTimerRef = useRef<ReturnType<typeof setInterval> | null>(null);

	const store = useExecutionSamplerStore;

	// Clean up polling timer.
	const clearPollTimer = useCallback(() => {
		if (pollTimerRef.current !== null) {
			clearInterval(pollTimerRef.current);
			pollTimerRef.current = null;
		}
	}, []);

	// Clean up play timer.
	const clearPlayTimer = useCallback(() => {
		if (playTimerRef.current !== null) {
			clearInterval(playTimerRef.current);
			playTimerRef.current = null;
		}
	}, []);

	// Abort in-flight requests.
	const abortRequests = useCallback(() => {
		if (abortRef.current) {
			abortRef.current.abort();
			abortRef.current = null;
		}
	}, []);

	// Start sampling for a creature.
	const startSampling = useCallback(
		async (creatureId: number, ticks = 5) => {
			abortRequests();
			clearPollTimer();
			clearPlayTimer();

			const controller = new AbortController();
			abortRef.current = controller;

			store.getState().setSampling();

			try {
				await api.startSample(creatureId, ticks, controller.signal);
			} catch (err: unknown) {
				if (err instanceof Error && err.name === "AbortError") return;
				store.getState().setError(err instanceof Error ? err.message : "failed to start sample");
				return;
			}

			// Poll for completion.
			pollTimerRef.current = setInterval(async () => {
				const pollController = new AbortController();
				abortRef.current = pollController;

				try {
					const resp = await api.getSample(creatureId, pollController.signal);
					if (resp.status === "complete") {
						clearPollTimer();
						store.getState().setSample(resp.sample);
					}
					// "recording" — keep polling.
					// "idle" — shouldn't happen, but just keep polling.
				} catch (err: unknown) {
					if (err instanceof Error && err.name === "AbortError") return;
					clearPollTimer();
					store.getState().setError(err instanceof Error ? err.message : "polling failed");
				}
			}, POLL_INTERVAL_MS);
		},
		[abortRequests, clearPollTimer, clearPlayTimer, store],
	);

	// Auto-advance interval for play mode.
	useEffect(() => {
		const unsubscribe = store.subscribe((state, prevState) => {
			if (state.playbackState === "playing" && prevState.playbackState !== "playing") {
				clearPlayTimer();
				playTimerRef.current = setInterval(() => {
					store.getState().stepForward();
				}, state.playbackSpeed);
			} else if (state.playbackState !== "playing" && prevState.playbackState === "playing") {
				clearPlayTimer();
			}

			// Update interval when speed changes during playback.
			if (
				state.playbackState === "playing" &&
				prevState.playbackState === "playing" &&
				state.playbackSpeed !== prevState.playbackSpeed
			) {
				clearPlayTimer();
				playTimerRef.current = setInterval(() => {
					store.getState().stepForward();
				}, state.playbackSpeed);
			}
		});

		return () => {
			unsubscribe();
			clearPlayTimer();
		};
	}, [store, clearPlayTimer]);

	// Cleanup on unmount.
	useEffect(() => {
		return () => {
			abortRequests();
			clearPollTimer();
			clearPlayTimer();
		};
	}, [abortRequests, clearPollTimer, clearPlayTimer]);

	return { startSampling };
}
