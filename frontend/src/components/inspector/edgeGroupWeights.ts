import type { EdgeGroup, InputReference, OutputSinkKind, Scope } from "../../types/genome.ts";
import { directionPairs, isRingSensor, parseWorldInputRef } from "./inputRefUtils.ts";

/** `Direction::ALL` screen deltas: x grows east, y grows south. */
const DELTAS: readonly (readonly [number, number])[] = [
	[0, -1],
	[1, -1],
	[1, 0],
	[1, 1],
	[0, 1],
	[-1, 1],
	[-1, 0],
	[-1, -1],
];

/** The unit compass's `x` at direction `d`, as v3-core's `compass_x`. */
export function compassX(d: number): number {
	const [dx, dy] = DELTAS[((d % 8) + 8) % 8] ?? [0, 0];
	if (dx === 0) return 0;
	return dy === 0 ? dx : dx * Math.SQRT1_2;
}

/** The unit compass's `y` at direction `d`: `compassX((d + 6) mod 8)`. */
export function compassY(d: number): number {
	return compassX((d + 6) % 8);
}

/** A member's genomic weight, as v3-core's `compose` (`cgp/group.rs`). */
export function compose(own: number, scale: number, pattern: number): number {
	return scale * pattern + own;
}

/** The direction of a directed vote sink. */
export function sinkDirection(kind: OutputSinkKind | undefined): number | undefined {
	if (kind === undefined || !("ActionVote" in kind)) return undefined;
	const vote = kind.ActionVote;
	if (typeof vote !== "object") return undefined;
	if ("Move" in vote) return vote.Move;
	if ("Reproduce" in vote) return vote.Reproduce;
	if ("StealEnergy" in vote) return vote.StealEnergy;
	return undefined;
}

/**
 * A member's pattern, as v3-core's `Group::pattern`: 1 for a ring group whose
 * ring resolves; for a pair group, the compass at the sink's direction less
 * the offset, `x` at the pair's `x` field and `y` at its `y`; else 0.
 */
export function memberPattern(
	group: EdgeGroup,
	inputRefs: InputReference[],
	subIdx: number,
	direction: number | undefined,
): number {
	if ("Ring" in group.shape) {
		const ref = inputRefs[group.shape.Ring.ring_ref];
		const isRing =
			ref !== undefined &&
			typeof ref === "object" &&
			"World" in ref &&
			isRingSensor(parseWorldInputRef(ref.World).key);
		return isRing ? 1 : 0;
	}
	const { ref_idx, pair } = group.shape.DirectionPair;
	const fields = directionPairs(inputRefs[ref_idx])[pair];
	if (fields === undefined || direction === undefined || direction >= 8) return 0;
	const relative = (direction - group.offset + 8) % 8;
	if (subIdx === fields.x) return compassX(relative);
	if (subIdx === fields.y) return compassY(relative);
	return 0;
}

/** A member's composition in numbers: `scale × pattern + correction = genomic`. */
export function composeTerms(own: number, scale: number, pattern: number): string {
	return `${scale.toFixed(3)} × ${pattern.toFixed(2)} + ${own.toFixed(3)} = ${compose(
		own,
		scale,
		pattern,
	).toFixed(3)}`;
}

/** A member's scope as the inspector badges print it (T25.F07). */
export function scopeLabel(scope: Scope): string {
	return `${scope.toLowerCase()} scope`;
}
