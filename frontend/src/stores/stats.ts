import { create } from "zustand";
import type { HealthPayload } from "../types/generated/HealthPayload.ts";
import type { LastTickActions } from "../types/generated/LastTickActions.ts";

export const SAMPLE_RING_SIZE = 600;

/** One health message's readings: gauges at the tick plus cumulative counters. */
export interface StatsSample {
	tick: number;
	population: number;
	meanEnergy: number;
	birthsTotal: number;
	deathsTotal: number;
	/** Null until the server has summed food once (before the first tick). */
	foodTotal: number | null;
	foodByType: number[];
	generationMean: number;
	generationMax: number;
	genomeSizeMean: number;
	lineagesAlive: number;
	lineagesEffective: number;
	mutantsHelpful: number;
	mutantsNeutral: number;
	mutantsDetrimental: number;
	weightChangesTotal: number;
	memoryWritesTotal: number;
	creatureTicksTotal: number;
	worldInputsReadMean: number;
	/** Mean compute energy per creature on the latest status before this sample. */
	thinkingCostMean: number;
}

export interface MutationTargetReachabilityTotal {
	reachable: number;
	unreachable: number;
	notApplicable: number;
}

export function sampleFromHealth(
	tick: number,
	health: HealthPayload,
	thinkingCostMean: number,
): StatsSample {
	const outcomes = health.mutation_outcome_summary;
	return {
		tick,
		population: health.population,
		meanEnergy: health.mean_energy,
		birthsTotal: health.reproduction_actions_spawned_total,
		deathsTotal: health.deaths_total,
		foodTotal:
			health.food_total_density_by_type.length === 0
				? null
				: health.food_total_density_by_type.reduce((sum, v) => sum + v, 0),
		foodByType: health.food_total_density_by_type,
		generationMean: health.generation_mean,
		generationMax: health.generation_max,
		genomeSizeMean: health.genome_size_mean,
		lineagesAlive: health.lineages_alive,
		lineagesEffective: health.lineages_effective,
		mutantsHelpful: outcomes.helpful_total,
		mutantsNeutral: outcomes.neutral_total,
		mutantsDetrimental: outcomes.detrimental_total,
		weightChangesTotal: health.plasticity_changes_total,
		memoryWritesTotal: health.shared_memory_writes_changed_total,
		creatureTicksTotal: health.creature_ticks_total,
		worldInputsReadMean: health.world_inputs_read_mean,
		thinkingCostMean,
	};
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

const EMPTY_REACHABILITY: MutationTargetReachabilityTotal = {
	reachable: 0,
	unreachable: 0,
	notApplicable: 0,
};

interface CumulativeDetail {
	reproAttempted: number;
	reproSpawned: number;
	reproRejected: number;
	reproRejectedByReason: Record<string, number>;
	mutationAttempted: number;
	mutationApplied: number;
	mutationSkipped: number;
	mutationSkippedByOperator: Record<string, number>;
	mutationTargetReachabilityTotal: MutationTargetReachabilityTotal;
	moveActionsBlockedByCause: Record<string, number>;
	reproductionInvalidTargetRejectedByCause: Record<string, number>;
	predationAttempted: number;
	predationTransferred: number;
	predationRejected: number;
	predationKills: number;
	predationByResult: Record<string, number>;
	deathsByCause: Record<string, number>;
}

function detailFromHealth(health: HealthPayload): CumulativeDetail {
	const reach = health.mutation_target_reachability_total;
	return {
		reproAttempted: health.reproduction_actions_attempted_total,
		reproSpawned: health.reproduction_actions_spawned_total,
		reproRejected: health.reproduction_actions_rejected_total,
		reproRejectedByReason: health.reproduction_actions_rejected_total_by_reason,
		mutationAttempted: health.mutation_events_attempted_total,
		mutationApplied: health.mutation_events_applied_total,
		mutationSkipped: health.mutation_events_skipped_total,
		mutationSkippedByOperator: health.mutation_events_skipped_total_by_operator ?? {},
		mutationTargetReachabilityTotal: reach
			? {
					reachable: reach.reachable,
					unreachable: reach.unreachable,
					notApplicable: reach.not_applicable,
				}
			: EMPTY_REACHABILITY,
		moveActionsBlockedByCause: health.move_actions_blocked_total_by_cause ?? {},
		reproductionInvalidTargetRejectedByCause:
			health.reproduction_actions_rejected_invalid_target_total_by_cause ?? {},
		predationAttempted: health.predation_actions_attempted_total,
		predationTransferred: health.predation_actions_transferred_total,
		predationRejected: health.predation_actions_rejected_total,
		predationKills: health.predation_kills_total,
		predationByResult: health.predation_actions_by_result,
		deathsByCause: health.deaths_total_by_cause,
	};
}

const EMPTY_DETAIL: CumulativeDetail = {
	reproAttempted: 0,
	reproSpawned: 0,
	reproRejected: 0,
	reproRejectedByReason: {},
	mutationAttempted: 0,
	mutationApplied: 0,
	mutationSkipped: 0,
	mutationSkippedByOperator: {},
	mutationTargetReachabilityTotal: EMPTY_REACHABILITY,
	moveActionsBlockedByCause: {},
	reproductionInvalidTargetRejectedByCause: {},
	predationAttempted: 0,
	predationTransferred: 0,
	predationRejected: 0,
	predationKills: 0,
	predationByResult: {},
	deathsByCause: {},
};

export interface StatsHistoryState extends CumulativeDetail {
	samples: StatsSample[];
	lastActions: LastTickActions | null;
	lastThinkingCostMean: number;

	/** Status arrives before health for a tick: keep what only status carries. */
	recordStatus: (actions: LastTickActions, thinkingCostMean: number) => void;
	recordHealth: (
		tick: number,
		health: HealthPayload,
		options?: { recordHistory?: boolean },
	) => void;
	reset: () => void;
}

export const useStatsHistoryStore = create<StatsHistoryState>()((set) => ({
	...EMPTY_DETAIL,
	samples: [],
	lastActions: null,
	lastThinkingCostMean: 0,

	recordStatus: (actions, thinkingCostMean) =>
		set({ lastActions: actions, lastThinkingCostMean: thinkingCostMean }),

	recordHealth: (tick, health, options) =>
		set((s) => {
			const detail = detailFromHealth(health);
			if (options?.recordHistory === false) return detail;
			return {
				...detail,
				samples: pushRing(
					s.samples,
					sampleFromHealth(tick, health, s.lastThinkingCostMean),
					SAMPLE_RING_SIZE,
				),
			};
		}),

	reset: () => set({ ...EMPTY_DETAIL, samples: [], lastActions: null, lastThinkingCostMean: 0 }),
}));
