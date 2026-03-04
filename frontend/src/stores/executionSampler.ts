import { create } from "zustand";
import type { ExecutionSample } from "../types/api.ts";

export type PlaybackState = "idle" | "sampling" | "loaded" | "playing" | "stepping";

export interface SamplerPosition {
	tickIndex: number;
	hopIndex: number;
	detailIndex: number;
}

interface ExecutionSamplerState {
	sample: ExecutionSample | null;
	playbackState: PlaybackState;
	position: SamplerPosition;
	playbackSpeed: number;
	error: string | null;

	// Actions
	setSample: (sample: ExecutionSample) => void;
	setSampling: () => void;
	clearSample: () => void;
	setPlaying: () => void;
	setPaused: () => void;
	stepForward: () => void;
	stepBackward: () => void;
	jumpToPosition: (pos: SamplerPosition) => void;
	setPlaybackSpeed: (ms: number) => void;
	setError: (msg: string) => void;
}

const INITIAL_POSITION: SamplerPosition = { tickIndex: 0, hopIndex: 0, detailIndex: 0 };

export const useExecutionSamplerStore = create<ExecutionSamplerState>()((set, get) => ({
	sample: null,
	playbackState: "idle",
	position: { ...INITIAL_POSITION },
	playbackSpeed: 500,
	error: null,

	setSample: (sample) =>
		set({
			sample,
			playbackState: "loaded",
			position: { ...INITIAL_POSITION },
			error: null,
		}),

	setSampling: () =>
		set({
			sample: null,
			playbackState: "sampling",
			position: { ...INITIAL_POSITION },
			error: null,
		}),

	clearSample: () =>
		set({
			sample: null,
			playbackState: "idle",
			position: { ...INITIAL_POSITION },
			error: null,
		}),

	setPlaying: () => set({ playbackState: "playing" }),

	setPaused: () => {
		const { playbackState } = get();
		if (playbackState === "playing") {
			set({ playbackState: "loaded" });
		}
	},

	stepForward: () => {
		const { sample, position, playbackState } = get();
		if (!sample) return;

		const { tickIndex, hopIndex, detailIndex } = position;
		const tick = sample.ticks[tickIndex];
		if (!tick) return;

		const hop = tick.hops[hopIndex];
		const totalDetails = hop ? getDetailCount(hop.backend_trace) : 0;

		// Preserve "playing" state during auto-advance; otherwise transition to "stepping".
		const nextState = playbackState === "playing" ? "playing" : "stepping";

		// Try advancing detail within current hop.
		if (detailIndex < totalDetails - 1) {
			set({
				position: { tickIndex, hopIndex, detailIndex: detailIndex + 1 },
				playbackState: nextState,
			});
			return;
		}

		// Try advancing to next hop within current tick.
		if (hopIndex < tick.hops.length - 1) {
			set({
				position: { tickIndex, hopIndex: hopIndex + 1, detailIndex: 0 },
				playbackState: nextState,
			});
			return;
		}

		// Try advancing to next tick.
		for (let nextTick = tickIndex + 1; nextTick < sample.ticks.length; nextTick++) {
			const nt = sample.ticks[nextTick];
			if (nt && nt.hops.length > 0) {
				set({
					position: { tickIndex: nextTick, hopIndex: 0, detailIndex: 0 },
					playbackState: nextState,
				});
				return;
			}
		}

		// At end — auto-pause playback.
		set({ playbackState: "loaded" });
	},

	stepBackward: () => {
		const { sample, position, playbackState } = get();
		if (!sample) return;

		const { tickIndex, hopIndex, detailIndex } = position;

		// Preserve "playing" state during auto-advance; otherwise transition to "stepping".
		const nextState = playbackState === "playing" ? "playing" : "stepping";

		// Try going back within current hop.
		if (detailIndex > 0) {
			set({
				position: { tickIndex, hopIndex, detailIndex: detailIndex - 1 },
				playbackState: nextState,
			});
			return;
		}

		// Try going back to previous hop.
		if (hopIndex > 0) {
			const currentTickData = sample.ticks[tickIndex];
			const prevHop = currentTickData?.hops[hopIndex - 1];
			const prevDetails = prevHop ? getDetailCount(prevHop.backend_trace) : 0;
			set({
				position: { tickIndex, hopIndex: hopIndex - 1, detailIndex: Math.max(0, prevDetails - 1) },
				playbackState: nextState,
			});
			return;
		}

		// Try going back to previous tick.
		for (let prevTick = tickIndex - 1; prevTick >= 0; prevTick--) {
			const pt = sample.ticks[prevTick];
			if (pt && pt.hops.length > 0) {
				const lastHopIdx = pt.hops.length - 1;
				const lastHop = pt.hops[lastHopIdx];
				const lastDetails = lastHop ? getDetailCount(lastHop.backend_trace) : 0;
				set({
					position: {
						tickIndex: prevTick,
						hopIndex: lastHopIdx,
						detailIndex: Math.max(0, lastDetails - 1),
					},
					playbackState: nextState,
				});
				return;
			}
		}

		// At start — stay at first position.
	},

	jumpToPosition: (pos) => set({ position: pos }),

	setPlaybackSpeed: (ms) => set({ playbackSpeed: ms }),

	setError: (msg) => set({ error: msg, playbackState: "idle", sample: null }),
}));

/** Count detail steps for a backend trace (VM steps or Graph passes). */
export function getDetailCount(
	trace: ExecutionSample["ticks"][0]["hops"][0]["backend_trace"],
): number {
	if (typeof trace === "string") return 0;
	if ("Vm" in trace) return trace.Vm.steps.length;
	if ("Graph" in trace) return trace.Graph.passes.length;
	return 0;
}
