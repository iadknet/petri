import { create } from "zustand";
import type { ExecutionSample } from "../types/trace.ts";

export type PlaybackState = "idle" | "loaded" | "playing" | "stepping";

export interface SamplerPosition {
	tickIndex: number;
	hopIndex: number;
	detailIndex: number;
}

interface SamplePlaybackState {
	sample: ExecutionSample | null;
	playbackState: PlaybackState;
	position: SamplerPosition;
	playbackSpeed: number;

	setSample: (sample: ExecutionSample) => void;
	clear: () => void;
	setPlaying: () => void;
	setPaused: () => void;
	stepForward: () => void;
	stepBackward: () => void;
	jumpToPosition: (pos: SamplerPosition) => void;
	setPlaybackSpeed: (ms: number) => void;
}

const INITIAL_POSITION: SamplerPosition = { tickIndex: 0, hopIndex: 0, detailIndex: 0 };

function makeInitialState(): Pick<
	SamplePlaybackState,
	"sample" | "playbackState" | "position" | "playbackSpeed"
> {
	return {
		sample: null,
		playbackState: "idle",
		position: { ...INITIAL_POSITION },
		playbackSpeed: 500,
	};
}

export const useSamplePlaybackStore = create<SamplePlaybackState>()((set, get) => ({
	...makeInitialState(),

	setSample: (sample) =>
		set({
			sample,
			playbackState: "loaded",
			position: { ...INITIAL_POSITION },
		}),

	clear: () =>
		set({
			...makeInitialState(),
		}),

	setPlaying: () => set({ playbackState: "playing" }),

	setPaused: () => {
		if (get().playbackState === "playing") {
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
		const nextState = playbackState === "playing" ? "playing" : "stepping";

		if (detailIndex < totalDetails - 1) {
			set({
				position: { tickIndex, hopIndex, detailIndex: detailIndex + 1 },
				playbackState: nextState,
			});
			return;
		}

		if (hopIndex < tick.hops.length - 1) {
			set({
				position: { tickIndex, hopIndex: hopIndex + 1, detailIndex: 0 },
				playbackState: nextState,
			});
			return;
		}

		for (let nextTick = tickIndex + 1; nextTick < sample.ticks.length; nextTick++) {
			const next = sample.ticks[nextTick];
			if (next && next.hops.length > 0) {
				set({
					position: { tickIndex: nextTick, hopIndex: 0, detailIndex: 0 },
					playbackState: nextState,
				});
				return;
			}
		}

		set({ playbackState: "loaded" });
	},

	stepBackward: () => {
		const { sample, position, playbackState } = get();
		if (!sample) return;

		const { tickIndex, hopIndex, detailIndex } = position;
		const nextState = playbackState === "playing" ? "playing" : "stepping";

		if (detailIndex > 0) {
			set({
				position: { tickIndex, hopIndex, detailIndex: detailIndex - 1 },
				playbackState: nextState,
			});
			return;
		}

		if (hopIndex > 0) {
			const currentTick = sample.ticks[tickIndex];
			const prevHop = currentTick?.hops[hopIndex - 1];
			const prevDetails = prevHop ? getDetailCount(prevHop.backend_trace) : 0;
			set({
				position: {
					tickIndex,
					hopIndex: hopIndex - 1,
					detailIndex: Math.max(0, prevDetails - 1),
				},
				playbackState: nextState,
			});
			return;
		}

		for (let prevTick = tickIndex - 1; prevTick >= 0; prevTick--) {
			const tick = sample.ticks[prevTick];
			if (tick && tick.hops.length > 0) {
				const lastHopIndex = tick.hops.length - 1;
				const lastHop = tick.hops[lastHopIndex];
				const lastDetails = lastHop ? getDetailCount(lastHop.backend_trace) : 0;
				set({
					position: {
						tickIndex: prevTick,
						hopIndex: lastHopIndex,
						detailIndex: Math.max(0, lastDetails - 1),
					},
					playbackState: nextState,
				});
				return;
			}
		}
	},

	jumpToPosition: (pos) => set({ position: pos }),

	setPlaybackSpeed: (ms) => set({ playbackSpeed: ms }),
}));

export function getDetailCount(
	trace: ExecutionSample["ticks"][0]["hops"][0]["backend_trace"],
): number {
	if (typeof trace === "string") return 0;
	if ("Vm" in trace) return trace.Vm.steps.length;
	if ("Graph" in trace) return trace.Graph.passes.length;
	return 0;
}
