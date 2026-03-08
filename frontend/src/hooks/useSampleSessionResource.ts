import { useCallback, useEffect, useRef } from "react";
import { api } from "../api/rest.ts";
import { useSamplePlaybackStore } from "../stores/samplePlayback.ts";
import { useSampleSessionStore } from "../stores/sampleSession.ts";

const POLL_DELAY_MS = 200;

export function useSampleSessionResource() {
	const abortRef = useRef<AbortController | null>(null);
	const pollTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
	const playTimerRef = useRef<ReturnType<typeof setInterval> | null>(null);
	const sessionVersionRef = useRef(0);

	const clearPollTimer = useCallback(() => {
		if (pollTimerRef.current !== null) {
			clearTimeout(pollTimerRef.current);
			pollTimerRef.current = null;
		}
	}, []);

	const clearPlayTimer = useCallback(() => {
		if (playTimerRef.current !== null) {
			clearInterval(playTimerRef.current);
			playTimerRef.current = null;
		}
	}, []);

	const abortRequests = useCallback(() => {
		if (abortRef.current) {
			abortRef.current.abort();
			abortRef.current = null;
		}
	}, []);

	const clearSampling = useCallback(() => {
		sessionVersionRef.current += 1;
		abortRequests();
		clearPollTimer();
		clearPlayTimer();
		useSamplePlaybackStore.getState().clear();
		useSampleSessionStore.getState().clear();
	}, [abortRequests, clearPlayTimer, clearPollTimer]);

	const startSampling = useCallback(
		async (creatureId: number, ticks = 5) => {
			sessionVersionRef.current += 1;
			const sessionVersion = sessionVersionRef.current;
			abortRequests();
			clearPollTimer();
			clearPlayTimer();

			const controller = new AbortController();
			abortRef.current = controller;

			useSamplePlaybackStore.getState().clear();
			useSampleSessionStore.getState().setRecording(creatureId);

			try {
				await api.startSample(creatureId, ticks, controller.signal);
				if (sessionVersion !== sessionVersionRef.current) return;
			} catch (err: unknown) {
				if (err instanceof Error && err.name === "AbortError") return;
				if (sessionVersion !== sessionVersionRef.current) return;
				useSampleSessionStore
					.getState()
					.setError(err instanceof Error ? err.message : "failed to start sample");
				return;
			}

			const poll = async () => {
				const pollController = new AbortController();
				abortRef.current = pollController;

				try {
					const resp = await api.getSample(creatureId, pollController.signal);
					if (sessionVersion !== sessionVersionRef.current) return;
					if (resp.status === "complete") {
						useSamplePlaybackStore.getState().setSample(resp.sample);
						useSampleSessionStore.getState().clear();
						return;
					}
					pollTimerRef.current = setTimeout(poll, POLL_DELAY_MS);
				} catch (err: unknown) {
					if (err instanceof Error && err.name === "AbortError") return;
					if (sessionVersion !== sessionVersionRef.current) return;
					useSamplePlaybackStore.getState().clear();
					useSampleSessionStore
						.getState()
						.setError(err instanceof Error ? err.message : "polling failed");
				}
			};

			pollTimerRef.current = setTimeout(poll, POLL_DELAY_MS);
		},
		[abortRequests, clearPlayTimer, clearPollTimer],
	);

	useEffect(() => {
		const unsubscribe = useSamplePlaybackStore.subscribe((state, prevState) => {
			if (state.playbackState === "playing" && prevState.playbackState !== "playing") {
				clearPlayTimer();
				playTimerRef.current = setInterval(() => {
					useSamplePlaybackStore.getState().stepForward();
				}, state.playbackSpeed);
				return;
			}

			if (state.playbackState !== "playing" && prevState.playbackState === "playing") {
				clearPlayTimer();
				return;
			}

			if (
				state.playbackState === "playing" &&
				prevState.playbackState === "playing" &&
				state.playbackSpeed !== prevState.playbackSpeed
			) {
				clearPlayTimer();
				playTimerRef.current = setInterval(() => {
					useSamplePlaybackStore.getState().stepForward();
				}, state.playbackSpeed);
			}
		});

		return () => {
			unsubscribe();
			clearPlayTimer();
		};
	}, [clearPlayTimer]);

	useEffect(
		() => () => {
			abortRequests();
			clearPollTimer();
			clearPlayTimer();
		},
		[abortRequests, clearPlayTimer, clearPollTimer],
	);

	return { clearSampling, startSampling };
}
