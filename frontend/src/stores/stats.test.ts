import { beforeEach, describe, expect, it } from "vitest";
import { MOCK_HEALTH_CENSUS } from "../test/fixtures.ts";
import type { HealthPayload } from "../types/generated/HealthPayload.ts";
import { SAMPLE_RING_SIZE, sampleFromHealth, useStatsHistoryStore } from "./stats.ts";

function health(overrides: Partial<HealthPayload> = {}): HealthPayload {
	return {
		...MOCK_HEALTH_CENSUS,
		population: 50,
		mean_energy: 37.4,
		mutation_events_attempted_total: 0,
		mutation_events_applied_total: 0,
		mutation_events_skipped_total: 0,
		mutation_events_attempted_total_by_domain: {},
		mutation_events_applied_total_by_domain: {},
		mutation_events_attempted_total_by_operator: {},
		mutation_events_applied_total_by_operator: {},
		reproduction_actions_attempted_total: 0,
		reproduction_actions_spawned_total: 0,
		reproduction_actions_rejected_total: 0,
		reproduction_actions_rejected_total_by_reason: {},
		mutation_events_skipped_total_by_reason: {},
		predation_actions_attempted_total: 0,
		predation_actions_transferred_total: 0,
		predation_actions_rejected_total: 0,
		predation_kills_total: 0,
		predation_actions_by_result: {},
		give_actions_attempted_total: 0,
		give_actions_transferred_total: 0,
		give_actions_rejected_total: 0,
		give_actions_by_result: {},
		genome_complexity_mean: 0,
		genome_complexity_min: 0,
		genome_complexity_max: 0,
		...overrides,
	};
}

describe("sampleFromHealth", () => {
	it("copies the census and sums food over types", () => {
		const sample = sampleFromHealth(
			12,
			health({
				reproduction_actions_spawned_total: 9,
				deaths_total: 4,
				food_total_density_by_type: [3.5, 1.5],
				generation_mean: 2.5,
				generation_max: 6,
				lineages_alive: 3,
				lineages_effective: 2.4,
				mutation_outcome_summary: {
					...MOCK_HEALTH_CENSUS.mutation_outcome_summary,
					carriers_observed_total: 10,
					helpful_total: 2,
					neutral_total: 5,
					detrimental_total: 3,
				},
				plasticity_changes_total: 40,
				shared_memory_writes_changed_total: 8,
				creature_ticks_total: 400,
				world_inputs_read_mean: 4.5,
			}),
			0.75,
		);

		expect(sample).toMatchObject({
			tick: 12,
			population: 50,
			birthsTotal: 9,
			deathsTotal: 4,
			foodTotal: 5,
			generationMean: 2.5,
			generationMax: 6,
			lineagesAlive: 3,
			lineagesEffective: 2.4,
			mutantsHelpful: 2,
			mutantsNeutral: 5,
			mutantsDetrimental: 3,
			weightChangesTotal: 40,
			memoryWritesTotal: 8,
			creatureTicksTotal: 400,
			worldInputsReadMean: 4.5,
			thinkingCostMean: 0.75,
		});
	});
});

describe("sampleFromHealth before the first tick", () => {
	it("reads no food total until the server has summed food", () => {
		expect(sampleFromHealth(0, health({ food_total_density_by_type: [] }), 0).foodTotal).toBeNull();
	});
});

describe("StatsHistoryStore", () => {
	beforeEach(() => {
		useStatsHistoryStore.getState().reset();
	});

	it("starts with no samples and no last-tick actions", () => {
		const state = useStatsHistoryStore.getState();
		expect(state.samples).toHaveLength(0);
		expect(state.lastActions).toBeNull();
	});

	it("records a sample per health message, stamped with the latest status compute cost", () => {
		const store = useStatsHistoryStore.getState();
		store.recordStatus(
			{ move: 1, eat: 2, reproduce: 0, noop: 0, steal: 0, give: 0, predation_kills: 0 },
			0.5,
		);
		store.recordHealth(1, health());
		store.recordHealth(2, health({ population: 51 }));

		const { samples, lastActions } = useStatsHistoryStore.getState();
		expect(samples.map((s) => s.tick)).toEqual([1, 2]);
		expect(samples[1]!.population).toBe(51);
		expect(samples[0]!.thinkingCostMean).toBe(0.5);
		expect(lastActions?.eat).toBe(2);
	});

	it("keeps cumulative detail but skips the sample when history is off", () => {
		useStatsHistoryStore.getState().recordHealth(
			3,
			health({
				reproduction_actions_attempted_total: 4,
				deaths_total_by_cause: { action_move: 2 },
			}),
			{ recordHistory: false },
		);
		const state = useStatsHistoryStore.getState();
		expect(state.samples).toHaveLength(0);
		expect(state.reproAttempted).toBe(4);
		expect(state.deathsByCause.action_move).toBe(2);
	});

	it("caps the ring and keeps earlier snapshots intact", () => {
		const store = useStatsHistoryStore.getState();
		for (let i = 0; i < SAMPLE_RING_SIZE; i++) store.recordHealth(i, health());
		const full = useStatsHistoryStore.getState().samples;
		store.recordHealth(SAMPLE_RING_SIZE, health());
		const next = useStatsHistoryStore.getState().samples;

		expect(next).not.toBe(full);
		expect(next).toHaveLength(SAMPLE_RING_SIZE);
		expect(next[0]!.tick).toBe(1);
		expect(full[0]!.tick).toBe(0);
	});

	it("tracks mutation detail from health", () => {
		useStatsHistoryStore.getState().recordHealth(
			1,
			health({
				mutation_events_attempted_total: 500,
				mutation_events_applied_total: 300,
				mutation_events_skipped_total: 200,
				mutation_events_skipped_total_by_operator: { insert_node: 120 },
				mutation_target_reachability_total: {
					reachable: 260,
					unreachable: 30,
					executed: 0,
					not_applicable: 10,
				},
				move_actions_blocked_total_by_cause: { barrier: 5 },
			}),
		);
		const state = useStatsHistoryStore.getState();
		expect(state.mutationApplied).toBe(300);
		expect(state.mutationSkippedByOperator.insert_node).toBe(120);
		expect(state.mutationTargetReachabilityTotal).toEqual({
			reachable: 260,
			unreachable: 30,
			notApplicable: 10,
		});
		expect(state.moveActionsBlockedByCause.barrier).toBe(5);
	});

	it("resets all state", () => {
		const store = useStatsHistoryStore.getState();
		store.recordStatus(
			{ move: 1, eat: 0, reproduce: 0, noop: 0, steal: 0, give: 0, predation_kills: 0 },
			1,
		);
		store.recordHealth(1, health({ mutation_events_attempted_total: 100 }));
		store.reset();
		const state = useStatsHistoryStore.getState();
		expect(state.samples).toHaveLength(0);
		expect(state.lastActions).toBeNull();
		expect(state.mutationAttempted).toBe(0);
		expect(Object.keys(state.deathsByCause)).toHaveLength(0);
	});
});
