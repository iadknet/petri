import { describe, expect, it } from "vitest";
import type { CreatureSnapshot } from "../types/generated/CreatureSnapshot.ts";
import {
	birthPop,
	glideProgress,
	nextGlideDuration,
	oozeRect,
	planCreatureMotion,
	sampleMotion,
} from "./motion.ts";

function creature(id: number, x: number, y: number): CreatureSnapshot {
	return { id, x, y, energy: 50, generation: 0, phenotype_rgb: [10, 20, 30] };
}

const rect = { x: 0, y: 0, width: 20, height: 20 };

describe("planCreatureMotion", () => {
	it("glides a creature from where it was drawn to its new cell", () => {
		const motions = planCreatureMotion(new Map([[1, { x: 5, y: 5 }]]), [creature(1, 6, 5)], rect);

		expect(motions.get(1)).toEqual({ fromX: 5, fromY: 5, toX: 6, toY: 5, born: false });
	});

	it("snaps instead of sliding across a wrap or teleport", () => {
		const motions = planCreatureMotion(new Map([[1, { x: 0, y: 5 }]]), [creature(1, 19, 5)], rect);

		expect(motions.get(1)).toMatchObject({ fromX: 19, fromY: 5, toX: 19, toY: 5 });
	});

	it("marks creatures appearing inside the drawn area as born", () => {
		const motions = planCreatureMotion(new Map(), [creature(2, 8, 8)], rect);

		expect(motions.get(2)?.born).toBe(true);
	});

	it("does not mark creatures entering at the edge of the view as born", () => {
		const motions = planCreatureMotion(new Map(), [creature(2, 0, 8), creature(3, 19, 8)], rect);

		expect(motions.get(2)?.born).toBe(false);
		expect(motions.get(3)?.born).toBe(false);
	});

	it("never marks births on the first frame", () => {
		const motions = planCreatureMotion(null, [creature(2, 8, 8)], rect);

		expect(motions.get(2)?.born).toBe(false);
	});
});

describe("glide timing", () => {
	it("eases out and clamps to the glide window", () => {
		expect(glideProgress(0, 200)).toBe(0);
		expect(glideProgress(100, 200)).toBeGreaterThan(0.5);
		expect(glideProgress(400, 200)).toBe(1);
		expect(glideProgress(10, 0)).toBe(1);
	});

	it("samples positions along the glide", () => {
		const motion = { fromX: 2, fromY: 4, toX: 3, toY: 2, born: false };

		expect(sampleMotion(motion, 0)).toEqual({ x: 2, y: 4 });
		expect(sampleMotion(motion, 0.5)).toEqual({ x: 2.5, y: 3 });
		expect(sampleMotion(motion, 1)).toEqual({ x: 3, y: 2 });
	});

	it("springs newborns in from nothing, overshoots, and settles", () => {
		expect(birthPop(0).scaleX).toBeCloseTo(0);
		const peak = Math.max(...[0.2, 0.25, 0.3, 0.35, 0.4].map((t) => birthPop(t).scaleX));
		expect(peak).toBeGreaterThan(1.1);
		expect(peak).toBeLessThan(1.45);
		expect(birthPop(1)).toEqual({ scaleX: 1, scaleY: 1, rippleRadius: 0, rippleAlpha: 0 });
	});

	it("squashes and stretches while popping, then rests square", () => {
		const mid = birthPop(0.15);
		expect(mid.scaleX).not.toBeCloseTo(mid.scaleY, 2);
		const late = birthPop(0.98);
		expect(Math.abs(late.scaleX - late.scaleY)).toBeLessThan(0.02);
	});

	it("ripples outward and fades", () => {
		expect(birthPop(0.1).rippleRadius).toBeLessThan(birthPop(0.4).rippleRadius);
		expect(birthPop(0.1).rippleAlpha).toBeGreaterThan(birthPop(0.4).rippleAlpha);
		expect(birthPop(0.7).rippleAlpha).toBe(0);
	});

	it("tracks the tick interval within bounds", () => {
		expect(nextGlideDuration(null, 160)).toBeCloseTo(144);
		expect(nextGlideDuration(null, 5)).toBe(60);
		expect(nextGlideDuration(null, 5000)).toBe(420);
		const smoothed = nextGlideDuration(144, 400);
		expect(smoothed).toBeGreaterThan(144);
		expect(smoothed).toBeLessThan(360);
	});
});

describe("oozeRect", () => {
	const right = { fromX: 2, fromY: 4, toX: 3, toY: 4, born: false };

	it("rests as a centred square at the start and end of a glide", () => {
		const start = oozeRect(right, 0, 0.8);
		expect(start.x).toBeCloseTo(2.1);
		expect(start.y).toBeCloseTo(4.1);
		expect(start.width).toBeCloseTo(0.8);
		expect(start.height).toBeCloseTo(0.8);
		const end = oozeRect(right, 1, 0.8);
		expect(end.x).toBeCloseTo(3.1);
		expect(end.width).toBeCloseTo(0.8);
		expect(end.height).toBeCloseTo(0.8);
	});

	it("stretches along the move and thins across it mid-glide", () => {
		const mid = oozeRect(right, 0.4, 0.8);
		expect(mid.width).toBeGreaterThan(0.8);
		expect(mid.height).toBeLessThan(0.8);
		// Never thinner than 65% of the resting size.
		expect(mid.height).toBeGreaterThanOrEqual(0.52 - 1e-9);
		// Stays centred across the direction of travel.
		expect(mid.y + mid.height / 2).toBeCloseTo(4.5);
	});

	it("leads with the edge facing the new cell", () => {
		const left = { fromX: 3, fromY: 4, toX: 2, toY: 4, born: false };
		const mid = oozeRect(left, 0.3, 0.8);
		// The left (leading) edge has moved further than the right (trailing) edge.
		const leadMoved = 3.1 - mid.x;
		const trailMoved = 3.9 - (mid.x + mid.width);
		expect(leadMoved).toBeGreaterThan(trailMoved);
	});

	it("does not change a creature that stays put", () => {
		const still = { fromX: 5, fromY: 5, toX: 5, toY: 5, born: false };
		const rect = oozeRect(still, 0.5, 0.8);
		expect(rect.x).toBeCloseTo(5.1);
		expect(rect.y).toBeCloseTo(5.1);
		expect(rect.width).toBeCloseTo(0.8);
		expect(rect.height).toBeCloseTo(0.8);
	});

	it("keeps the area of a gently stretched body", () => {
		const early = oozeRect(right, 0.06, 0.8);
		expect(early.width).toBeGreaterThan(0.8);
		expect(early.width * early.height).toBeCloseTo(0.64, 2);
	});
});
