import type { BrainBundle } from "../../types/generated/BrainBundle.ts";
import type { BrainEdge } from "../../types/generated/BrainEdge.ts";
import type { LearningEdge } from "../../types/generated/LearningEdge.ts";
import type { NodeLearning } from "../../types/generated/NodeLearning.ts";
import type { WeightSurface } from "../../types/generated/WeightSurface.ts";

/**
 * The runtime state of one edge: `brain`'s edge joined to its `learning`
 * entry by `(surface, container, edge)` (T30.F01 invariant 5). A listed edge
 * takes the poll's values; any other keeps the brain's. Nothing is
 * recomputed.
 */
export interface EdgeState {
	/** The weight in force. */
	applied: number;
	/** A learner's learned own quantity, when initialized. */
	learned?: number;
	/** A modulated learner's eligibility trace, when initialized. */
	trace?: number;
	/** A learner's initialization; absent for a non-learner. */
	initialized?: boolean;
}

/** Joins a node's `learning` entry to its brain edges. */
export interface LearningJoin {
	/** The runtime state of `edge` at `(surface, container, index)`. */
	edge(surface: WeightSurface, container: number, index: number, edge: BrainEdge): EdgeState;
	/** The learned scale of bundle `index`, when its scale slot is initialized. */
	learnedScale(index: number, bundle: BrainBundle | undefined): number | undefined;
}

/** An edge's state from its weight in force and its learner facts, if any. */
function edgeState(applied: number, learner: Omit<EdgeState, "applied"> | undefined): EdgeState {
	return {
		applied,
		...(learner?.learned === undefined ? {} : { learned: learner.learned }),
		...(learner?.trace === undefined ? {} : { trace: learner.trace }),
		...(learner?.initialized === undefined ? {} : { initialized: learner.initialized }),
	};
}

export function joinLearning(learning: NodeLearning | null): LearningJoin {
	const entries = new Map<string, LearningEdge>(
		(learning?.edges ?? []).map((entry) => [
			`${entry.surface}:${entry.container}:${entry.edge}`,
			entry,
		]),
	);
	const scales = new Map((learning?.scales ?? []).map((scale) => [scale.bundle, scale]));
	return {
		edge(surface, container, index, edge) {
			const entry = entries.get(`${surface}:${container}:${index}`);
			return entry === undefined
				? edgeState(edge.applied, edge.learner)
				: edgeState(entry.applied, entry);
		},
		learnedScale(index, bundle) {
			const scale = scales.get(index);
			return scale === undefined ? bundle?.learned_scale : scale.learned;
		},
	};
}
