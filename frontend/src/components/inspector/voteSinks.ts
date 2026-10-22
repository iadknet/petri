import type { VoteSink } from "../../types/genome.ts";
import { ACTION_TYPES, voteSinkFromIndex } from "../../types/vote-sinks.ts";
import { directionName } from "./directions.ts";

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
