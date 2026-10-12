import type { DirectedVoteKind, VoteSink } from "../../types/genome.ts";
import { directionName } from "./directions.ts";

/** Directions each directed vote sink covers — the `Direction::ALL` index range. */
const VOTE_DIRECTION_COUNT = 8;
/** The directed kinds in catalog order: their drives follow the direction sinks. */
const DIRECTED_VOTE_KINDS: readonly DirectedVoteKind[] = ["Move", "Reproduce", "StealEnergy"];
/** Catalog index of the first drive sink. */
const FIRST_DRIVE_SINK = 1 + 3 * VOTE_DIRECTION_COUNT;
/** Catalog index of `Terminate`; `Decide` follows it. */
const TERMINATE_SINK = FIRST_DRIVE_SINK + DIRECTED_VOTE_KINDS.length;

/** The vote sink at a catalog index; null at or above the catalog count. */
export function voteSinkFromIndex(index: number): VoteSink | null {
	if (index === 0) return "Eat";
	if (index < 1 + VOTE_DIRECTION_COUNT) return { Move: index - 1 };
	if (index < 1 + 2 * VOTE_DIRECTION_COUNT) {
		return { Reproduce: index - 1 - VOTE_DIRECTION_COUNT };
	}
	if (index < FIRST_DRIVE_SINK) {
		return { StealEnergy: index - 1 - 2 * VOTE_DIRECTION_COUNT };
	}
	const drive = DIRECTED_VOTE_KINDS[index - FIRST_DRIVE_SINK];
	if (drive !== undefined) return { Drive: drive };
	if (index === TERMINATE_SINK) return "Terminate";
	if (index === TERMINATE_SINK + 1) return "Decide";
	return null;
}

/** The catalog index of a vote sink, the inverse of `voteSinkFromIndex`. */
export function voteSinkIndex(sink: VoteSink): number {
	if (sink === "Eat") return 0;
	if (sink === "Terminate") return TERMINATE_SINK;
	if (sink === "Decide") return TERMINATE_SINK + 1;
	if ("Move" in sink) return 1 + sink.Move;
	if ("Reproduce" in sink) return 1 + VOTE_DIRECTION_COUNT + sink.Reproduce;
	if ("StealEnergy" in sink) return 1 + 2 * VOTE_DIRECTION_COUNT + sink.StealEnergy;
	return driveSinkIndex(sink.Drive);
}

/** The catalog index of a directed kind's drive. */
export function driveSinkIndex(kind: DirectedVoteKind): number {
	return FIRST_DRIVE_SINK + DIRECTED_VOTE_KINDS.indexOf(kind);
}

/** Label the vote sink at a catalog index; out of range reads as invalid. */
export function voteSinkLabel(index: number): string {
	const sink = voteSinkFromIndex(index);
	return sink === null ? `invalid(${index})` : formatVoteSink(sink);
}

/** Format one vote sink of the catalog. */
export function formatVoteSink(sink: VoteSink): string {
	if (typeof sink === "string") return sink;
	if ("Move" in sink) return `Move[${directionName(sink.Move)}]`;
	if ("Reproduce" in sink) return `Reproduce[${directionName(sink.Reproduce)}]`;
	if ("StealEnergy" in sink) return `StealEnergy[${directionName(sink.StealEnergy)}]`;
	if ("Drive" in sink) return `Drive ${sink.Drive}`;
	return "?";
}
