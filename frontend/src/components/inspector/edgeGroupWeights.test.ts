import { describe, expect, it } from "vitest";
import type { EdgeGroup, InputReference } from "../../types/genome.ts";
import { compassX, compassY, compose, memberPattern, sinkDirection } from "./edgeGroupWeights.ts";
import { directionPairs } from "./inputRefUtils.ts";

const D = Math.SQRT1_2;

describe("compass", () => {
	it("is the normalized screen delta of each direction, y growing south", () => {
		// N, NE, E, SE, S, SW, W, NW
		expect([0, 1, 2, 3, 4, 5, 6, 7].map(compassX)).toEqual([0, D, 1, D, 0, -D, -1, -D]);
		expect([0, 1, 2, 3, 4, 5, 6, 7].map(compassY)).toEqual([-1, -D, 0, D, 1, D, 0, -D]);
	});
});

describe("compose", () => {
	it("is scale times pattern plus the member's own correction", () => {
		expect(compose(0.25, 0.5, -1)).toBe(-0.25);
	});
});

describe("sinkDirection", () => {
	it("reads a directed vote's direction and nothing else", () => {
		expect(sinkDirection({ ActionVote: { Move: 3 } })).toBe(3);
		expect(sinkDirection({ ActionVote: { Reproduce: 5 } })).toBe(5);
		expect(sinkDirection({ ActionVote: "Eat" })).toBeUndefined();
		expect(sinkDirection(undefined)).toBeUndefined();
	});
});

describe("memberPattern", () => {
	const ring: EdgeGroup = {
		shape: { Ring: { ring_ref: 1 } },
		kind: "Move",
		offset: 2,
		scale: 0,
		inherit_scale: false,
	};

	it("is 1 for a ring group whose ring resolves, 0 when it does not", () => {
		const refs: InputReference[] = [{ World: "FoodHere" }, { World: "NeighborBarrierRing" }];
		expect(memberPattern(ring, refs, 4, 6)).toBe(1);
		expect(memberPattern(ring, [{ World: "FoodHere" }, { World: "FoodHere" }], 4, 6)).toBe(0);
		expect(memberPattern(ring, [], 4, 6)).toBe(0);
		// A ring ignores the coordinate: a member reading no field, no direction.
		expect(memberPattern(ring, refs, 0xffff, undefined)).toBe(1);
	});

	it("is the compass at the sink direction less the offset for a pair member", () => {
		const refs: InputReference[] = [{ World: "AreaFoodSummary" }];
		const pair = directionPairs(refs[0])[0];
		if (pair === undefined) throw new Error("AreaFoodSummary declares a direction pair");
		const group: EdgeGroup = {
			shape: { DirectionPair: { ref_idx: 0, pair: 0 } },
			kind: "Move",
			offset: 1,
			scale: 0,
			inherit_scale: false,
		};
		// Sink E (2) at offset 1 is relative NE (1).
		expect(memberPattern(group, refs, pair.x, 2)).toBeCloseTo(D);
		expect(memberPattern(group, refs, pair.y, 2)).toBeCloseTo(-D);
		// Sink N (0) at offset 1 is relative NW (7).
		expect(memberPattern(group, refs, pair.x, 0)).toBeCloseTo(-D);
		// A field outside the pair, a sink without a direction, an unresolved pair.
		expect(memberPattern(group, refs, 99, 2)).toBe(0);
		expect(memberPattern(group, refs, pair.x, undefined)).toBe(0);
		expect(memberPattern(group, [], pair.x, 2)).toBe(0);
	});
});
