import type { DecisionInputsPayload } from "../types/generated/DecisionInputsPayload.ts";
import type { StaticInputsSnapshotPayload } from "../types/generated/StaticInputsSnapshotPayload.ts";
import { VOTE_KINDS, VOTE_SINK_COUNT, type VoteKind } from "../types/vote-sinks.ts";

/** A vote vector with no contributions. */
export function zeroVotes(): number[] {
	return Array.from({ length: VOTE_SINK_COUNT }, () => 0);
}

/** A per-kind vector (commit counts, bars), one entry per `VOTE_KINDS` kind. */
type KindCounts = DecisionInputsPayload["commit_counts"];

/** A per-kind vector with every kind at zero. */
export function zeroKindCounts(): KindCounts {
	return [0, 0, 0, 0, 0];
}

/** A per-kind vector in `VOTE_KINDS` order from values by kind (absent: 0). */
export function kindCounts(byKind: Partial<Record<VoteKind, number>>): KindCounts {
	const counts = zeroKindCounts();
	VOTE_KINDS.forEach((kind, index) => {
		counts[index] = byKind[kind] ?? 0;
	});
	return counts;
}

/** A neighbor ring with every cell at zero. */
export function zeroRing(): StaticInputsSnapshotPayload["neighbor_food"] {
	return [0, 0, 0, 0, 0, 0, 0, 0];
}

/** Decision-state inputs of a dispatch before any vote, commit, or hop. */
export function zeroDecisionInputs(): DecisionInputsPayload {
	return {
		action_votes: zeroVotes(),
		previous_pass_votes: zeroVotes(),
		commit_counts: zeroKindCounts(),
		hops_this_tick: 0,
	};
}
