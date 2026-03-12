import { beforeEach, describe, expect, it } from "vitest";
import { useStatsHistoryStore } from "./stats.ts";

describe("StatsHistoryStore", () => {
	beforeEach(() => {
		useStatsHistoryStore.getState().reset();
	});

	it("starts with empty history", () => {
		const state = useStatsHistoryStore.getState();
		expect(state.statsHistory).toHaveLength(0);
		expect(state.actionsHistory).toHaveLength(0);
	});

	it("accumulates stats points", () => {
		const store = useStatsHistoryStore.getState();
		store.pushStats(1, 50, 37.4);
		store.pushStats(2, 51, 37.8);
		store.pushStats(3, 49, 36.9);

		const history = useStatsHistoryStore.getState().statsHistory;
		expect(history).toHaveLength(3);
		expect(history[0]!.population).toBe(50);
		expect(history[2]!.tick).toBe(3);
	});

	it("ring buffer caps at 500 entries", () => {
		const store = useStatsHistoryStore.getState();
		for (let i = 0; i < 510; i++) {
			store.pushStats(i, i, i * 0.5);
		}
		const history = useStatsHistoryStore.getState().statsHistory;
		expect(history).toHaveLength(500);
		// Oldest entries trimmed, newest retained
		expect(history[0]!.tick).toBe(10);
		expect(history[499]!.tick).toBe(509);
	});

	it("tracks reproduction stats", () => {
		useStatsHistoryStore.getState().setReproStats(100, 20, 80, {
			RejectedInvalidTarget: 60,
			RejectedEnergyConstraints: 20,
		});
		const state = useStatsHistoryStore.getState();
		expect(state.reproAttempted).toBe(100);
		expect(state.reproSpawned).toBe(20);
		expect(state.reproRejectedByReason.RejectedInvalidTarget).toBe(60);
	});

	it("tracks mutation stats", () => {
		useStatsHistoryStore
			.getState()
			.setMutationStats(
				500,
				300,
				200,
				{ insert_node: 120, rewired_input: 80 },
				{ reachable: 260, unreachable: 30, notApplicable: 10 },
				{ barrier: 5, occupied: 2 },
				{ barrier: 9, contention: 3 },
			);
		const state = useStatsHistoryStore.getState();
		expect(state.mutationAttempted).toBe(500);
		expect(state.mutationApplied).toBe(300);
		expect(state.mutationSkipped).toBe(200);
		expect(state.mutationSkippedByOperator.insert_node).toBe(120);
		expect(state.mutationTargetReachabilityTotal.reachable).toBe(260);
		expect(state.moveActionsBlockedByCause.barrier).toBe(5);
		expect(state.reproductionInvalidTargetRejectedByCause.contention).toBe(3);
	});

	it("resets all state", () => {
		useStatsHistoryStore.getState().pushStats(1, 50, 37.4);
		useStatsHistoryStore.getState().setMutationStats(100, 50, 50);
		useStatsHistoryStore.getState().reset();
		const state = useStatsHistoryStore.getState();
		expect(state.statsHistory).toHaveLength(0);
		expect(state.mutationAttempted).toBe(0);
		expect(state.mutationTargetReachabilityTotal.reachable).toBe(0);
		expect(Object.keys(state.mutationSkippedByOperator)).toHaveLength(0);
	});
});
