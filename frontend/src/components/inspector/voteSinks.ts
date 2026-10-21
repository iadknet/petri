import type { ActionParamField, DirectedVoteKind, VoteKind, VoteSink } from "../../types/genome.ts";
import { directionName } from "./directions.ts";
import catalog from "./voteSinks.json";

/** The action-vote catalog, as v3-core declares it (`creature/genome/vote.rs`
 * pins `voteSinks.json` to the declarations; T27.F03). */
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

/** The action-code legend `ReadActionQueueType` reads: `0 NoOp, 1 Eat, …`. */
export function actionCodeLegend(): string {
	return ACTION_TYPES.map(({ name, code }) => `${code} ${name}`).join(", ");
}

/** Label the vote sink at a catalog index; out of range reads as invalid. */
export function voteSinkLabel(index: number): string {
	const sink = voteSinkFromIndex(index);
	return sink === null ? `invalid(${index})` : formatVoteSink(sink);
}

/** Format one vote sink of the catalog. */
export function formatVoteSink(sink: VoteSink): string {
	if (typeof sink === "string") return sink;
	if ("Directed" in sink) return `${sink.Directed[0]}[${directionName(sink.Directed[1])}]`;
	if ("Drive" in sink) return `Drive ${sink.Drive}`;
	return "?";
}
