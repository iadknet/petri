import type { ActionParamField } from "./generated/ActionParamField.ts";
import type { ActionType } from "./generated/ActionType.ts";
import type { DirectedVoteKind } from "./generated/DirectedVoteKind.ts";
import type { VoteSink } from "./generated/VoteSink.ts";
import catalog from "./vote-sinks.json";

/** An action kind carrying votes and a bar: every action but `NoOp`. */
export type VoteKind = Exclude<ActionType, "NoOp">;

/** The action-vote catalog, as v3-core declares it (`creature/genome/vote.rs`
 * pins `vote-sinks.json` to the declarations; T27.F03). */
const VOTE_SINKS = Object.freeze(catalog.voteSinks as VoteSink[]);

/** Vote sinks in the catalog. */
export const VOTE_SINK_COUNT = VOTE_SINKS.length;
/** Action kinds carrying a vote commit counter, in `commit_counts` order. */
export const VOTE_KINDS: readonly VoteKind[] = Object.freeze(catalog.voteKinds as VoteKind[]);
/** The directed kinds in catalog order. */
export const DIRECTED_VOTE_KINDS: readonly DirectedVoteKind[] = Object.freeze(
	catalog.directedKinds as DirectedVoteKind[],
);
/** Catalog order: the Graph sink order and the VM `field_idx` addressing. */
export const ACTION_PARAM_FIELDS: readonly ActionParamField[] = Object.freeze(
	catalog.actionParamFields.map(({ field }) => field as ActionParamField),
);
/** Action types in discriminant order: `name` at its queue `code`. */
export const ACTION_TYPES: readonly { name: string; code: number }[] = Object.freeze(
	catalog.actionTypes,
);

const INDEX_BY_KEY = new Map(VOTE_SINKS.map((sink, index) => [JSON.stringify(sink), index]));

/** The vote sink at a catalog index; null at or above the catalog count. */
export function voteSinkFromIndex(index: number): VoteSink | null {
	return VOTE_SINKS[index] ?? null;
}

/** The catalog index of a vote sink, the inverse of `voteSinkFromIndex`; -1
 * for a sink outside the catalog. */
export function voteSinkIndex(sink: VoteSink): number {
	return INDEX_BY_KEY.get(JSON.stringify(sink)) ?? -1;
}

/** The catalog index of a directed kind's drive. */
export function driveSinkIndex(kind: DirectedVoteKind): number {
	return voteSinkIndex({ Drive: kind });
}
