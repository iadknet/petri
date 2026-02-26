import { create } from "zustand";
import type { LastTickActions } from "../types/api.ts";

const RING_SIZE = 500;

export interface StatsPoint {
	tick: number;
	population: number;
	meanEnergy: number;
}

export interface ActionPoint {
	tick: number;
	actions: LastTickActions;
}

export interface ComputePoint {
	tick: number;
	totalMean: number;
	totalMin: number;
	totalMax: number;
	vmMean: number;
	graphMean: number;
}

export interface ComplexityPoint {
	tick: number;
	mean: number;
	min: number;
	max: number;
}

/** Fixed-size ring buffer that overwrites oldest entries */
function pushRing<T>(buf: T[], item: T, maxSize: number): T[] {
	if (buf.length >= maxSize) {
		const next = buf.slice(1);
		next.push(item);
		return next;
	}
	return [...buf, item];
}

export interface StatsHistoryState {
	statsHistory: StatsPoint[];
	actionsHistory: ActionPoint[];
	computeHistory: ComputePoint[];
	complexityHistory: ComplexityPoint[];

	// Cumulative reproduction stats (from health events)
	reproAttempted: number;
	reproSpawned: number;
	reproRejected: number;
	reproRejectedByReason: Record<string, number>;

	// Cumulative mutation stats (from health events)
	mutationAttempted: number;
	mutationApplied: number;
	mutationSkipped: number;

	// Actions
	pushStats: (tick: number, population: number, meanEnergy: number) => void;
	pushActions: (tick: number, actions: LastTickActions) => void;
	pushCompute: (
		tick: number,
		totalMean: number,
		totalMin: number,
		totalMax: number,
		vmMean: number,
		graphMean: number,
	) => void;
	pushComplexity: (tick: number, mean: number, min: number, max: number) => void;
	setReproStats: (
		attempted: number,
		spawned: number,
		rejected: number,
		byReason: Record<string, number>,
	) => void;
	setMutationStats: (attempted: number, applied: number, skipped: number) => void;
	reset: () => void;
}

export const useStatsHistoryStore = create<StatsHistoryState>()((set) => ({
	statsHistory: [],
	actionsHistory: [],
	computeHistory: [],
	complexityHistory: [],
	reproAttempted: 0,
	reproSpawned: 0,
	reproRejected: 0,
	reproRejectedByReason: {},
	mutationAttempted: 0,
	mutationApplied: 0,
	mutationSkipped: 0,

	pushStats: (tick, population, meanEnergy) =>
		set((s) => ({
			statsHistory: pushRing(s.statsHistory, { tick, population, meanEnergy }, RING_SIZE),
		})),

	pushActions: (tick, actions) =>
		set((s) => ({
			actionsHistory: pushRing(s.actionsHistory, { tick, actions }, RING_SIZE),
		})),

	pushCompute: (tick, totalMean, totalMin, totalMax, vmMean, graphMean) =>
		set((s) => ({
			computeHistory: pushRing(
				s.computeHistory,
				{ tick, totalMean, totalMin, totalMax, vmMean, graphMean },
				RING_SIZE,
			),
		})),

	pushComplexity: (tick, mean, min, max) =>
		set((s) => ({
			complexityHistory: pushRing(s.complexityHistory, { tick, mean, min, max }, RING_SIZE),
		})),

	setReproStats: (attempted, spawned, rejected, byReason) =>
		set({
			reproAttempted: attempted,
			reproSpawned: spawned,
			reproRejected: rejected,
			reproRejectedByReason: byReason,
		}),

	setMutationStats: (attempted, applied, skipped) =>
		set({
			mutationAttempted: attempted,
			mutationApplied: applied,
			mutationSkipped: skipped,
		}),

	reset: () =>
		set({
			statsHistory: [],
			actionsHistory: [],
			computeHistory: [],
			complexityHistory: [],
			reproAttempted: 0,
			reproSpawned: 0,
			reproRejected: 0,
			reproRejectedByReason: {},
			mutationAttempted: 0,
			mutationApplied: 0,
			mutationSkipped: 0,
		}),
}));
