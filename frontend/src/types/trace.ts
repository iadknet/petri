import type { DecisionInputsPayload } from "./generated/DecisionInputsPayload.ts";
import type { OutcomeChannel } from "./generated/OutcomeChannel.ts";
import { VOTE_KINDS, VOTE_SINK_COUNT } from "./vote-sinks.ts";

export { VOTE_KINDS, VOTE_SINK_COUNT } from "./vote-sinks.ts";

/** `PreviousOutcome` channels, in `previous_outcome` order. */
export const OUTCOME_CHANNELS: readonly OutcomeChannel[] = Object.freeze([
	"EnergyDelta",
	"ActionSuccess",
	"DamageDelta",
	"OffspringSuccess",
]);
/** A vote vector with no contributions. */
export const ZERO_VOTES: readonly number[] = Object.freeze(
	Array.from({ length: VOTE_SINK_COUNT }, () => 0),
);
/** A per-kind vector (commit counts, bars) with every kind at zero, in
 * `VOTE_KINDS` order. */
export const ZERO_KIND_COUNTS: readonly number[] = Object.freeze(VOTE_KINDS.map(() => 0));

/** Decision-state inputs of a dispatch before any vote, commit, or hop. */
const zeroDecisionInputs: DecisionInputsPayload = {
	action_votes: [...ZERO_VOTES],
	previous_pass_votes: [...ZERO_VOTES],
	commit_counts: [0, 0, 0, 0, 0],
	hops_this_tick: 0,
};
export const ZERO_DECISION_INPUTS: Readonly<DecisionInputsPayload> =
	Object.freeze(zeroDecisionInputs);
