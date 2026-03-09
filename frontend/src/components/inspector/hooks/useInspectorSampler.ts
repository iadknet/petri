import { useCallback, useEffect, useRef } from "react";
import { useExecutionSampler } from "../../../hooks/useExecutionSampler.ts";
import {
	type PlaybackState,
	type SamplerPosition,
	getDetailCount,
	useSamplePlaybackStore,
} from "../../../stores/samplePlayback.ts";
import { useSampleSessionStore } from "../../../stores/sampleSession.ts";
import type { ExecutionSample } from "../../../types/trace.ts";

export interface UseInspectorSamplerResult {
	sample: ExecutionSample | null;
	position: SamplerPosition;
	playbackState: PlaybackState | "sampling";
	playbackSpeed: number;
	totalTicks: number;
	totalHops: number;
	totalDetails: number;
	samplingError: string | null;
	handleSample: () => void;
	handleResample: () => void;
	handleTickSelect: (index: number) => void;
	handleHopSelect: (index: number) => void;
	clearSample: () => void;
	setPlaying: () => void;
	setPaused: () => void;
	stepForward: () => void;
	stepBackward: () => void;
	setPlaybackSpeed: (ms: number) => void;
}

export function useInspectorSampler(creatureId: number | null): UseInspectorSamplerResult {
	const { clearSampling, startSampling } = useExecutionSampler();

	// Playback store subscriptions
	const sample = useSamplePlaybackStore((s) => s.sample);
	const playbackMode = useSamplePlaybackStore((s) => s.playbackState);
	const position = useSamplePlaybackStore((s) => s.position);
	const playbackSpeed = useSamplePlaybackStore((s) => s.playbackSpeed);

	// Session store subscriptions
	const samplingStatus = useSampleSessionStore((s) => s.status);
	const samplingError = useSampleSessionStore((s) => s.error);

	// Derive playbackState: "sampling" when recording, else from playback store
	const playbackState: PlaybackState | "sampling" =
		samplingStatus === "recording" ? "sampling" : playbackMode;

	// Auto-clear sample when creatureId changes
	const prevCreatureIdRef = useRef(creatureId);
	useEffect(() => {
		if (prevCreatureIdRef.current !== null && prevCreatureIdRef.current !== creatureId) {
			clearSampling();
		}
		prevCreatureIdRef.current = creatureId;
	}, [clearSampling, creatureId]);

	// Sampling callbacks
	const handleSample = useCallback(() => {
		if (creatureId !== null) {
			startSampling(creatureId);
		}
	}, [creatureId, startSampling]);

	const handleResample = useCallback(() => {
		clearSampling();
		if (creatureId !== null) {
			startSampling(creatureId);
		}
	}, [clearSampling, creatureId, startSampling]);

	// Derived values
	const totalTicks = sample ? sample.ticks.length : 0;

	const currentTick = sample?.ticks[position.tickIndex];
	const totalHops = currentTick ? currentTick.hops.length : 0;

	const currentHop = currentTick?.hops[position.hopIndex];
	const totalDetails = currentHop ? getDetailCount(currentHop.backend_trace) : 0;

	// Navigation callbacks
	const handleTickSelect = useCallback((index: number) => {
		useSamplePlaybackStore
			.getState()
			.jumpToPosition({ tickIndex: index, hopIndex: 0, detailIndex: 0 });
	}, []);

	const handleHopSelect = useCallback((index: number) => {
		const currentPos = useSamplePlaybackStore.getState().position;
		useSamplePlaybackStore
			.getState()
			.jumpToPosition({ ...currentPos, hopIndex: index, detailIndex: 0 });
	}, []);

	// Stable action refs via selector
	const clearSample = useSamplePlaybackStore((s) => s.clear);
	const setPlaying = useSamplePlaybackStore((s) => s.setPlaying);
	const setPaused = useSamplePlaybackStore((s) => s.setPaused);
	const stepForward = useSamplePlaybackStore((s) => s.stepForward);
	const stepBackward = useSamplePlaybackStore((s) => s.stepBackward);
	const setPlaybackSpeed = useSamplePlaybackStore((s) => s.setPlaybackSpeed);

	return {
		sample,
		position,
		playbackState,
		playbackSpeed,
		totalTicks,
		totalHops,
		totalDetails,
		samplingError,
		handleSample,
		handleResample,
		handleTickSelect,
		handleHopSelect,
		clearSample,
		setPlaying,
		setPaused,
		stepForward,
		stepBackward,
		setPlaybackSpeed,
	};
}
