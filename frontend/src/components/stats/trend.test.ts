import { describe, expect, it } from "vitest";
import { trendGeometry, xToIndex } from "./trend.ts";

describe("trendGeometry", () => {
	it("puts both series on one scale with the minimum at the bottom inset", () => {
		const geo = trendGeometry([0, 10], [5, 5], 100, 24);
		expect(geo.runs[0]).toEqual([
			{ x: 0, y: 22 },
			{ x: 100, y: 2 },
		]);
		expect(geo.secondaryRuns[0]![0]!.y).toBe(12);
	});

	it("breaks the line where a value is missing", () => {
		const geo = trendGeometry([1, 2, null, 4, 5], undefined, 40, 24);
		expect(geo.runs).toHaveLength(2);
		expect(geo.runs[0]!.map((p) => p.x)).toEqual([0, 10]);
		expect(geo.runs[1]!.map((p) => p.x)).toEqual([30, 40]);
	});

	it("runs a constant series through the middle so it is not read as zero", () => {
		const geo = trendGeometry([3, 3, 3], undefined, 20, 24);
		expect(geo.runs[0]!.every((p) => p.y === 12)).toBe(true);
	});

	it("draws a move inside the noise floor nearly flat instead of full height", () => {
		const geo = trendGeometry([704_400, 704_738], undefined, 20, 24, 0.03);
		const [a, b] = geo.runs[0]!;
		expect(Math.abs(a!.y - b!.y)).toBeLessThan(1);
		expect(Math.abs((a!.y + b!.y) / 2 - 12)).toBeLessThan(0.5);
	});

	it("has nothing to draw without values", () => {
		expect(trendGeometry([null, null], undefined, 20, 24).runs).toEqual([]);
	});
});

describe("xToIndex", () => {
	it("snaps a pointer x to the nearest sample", () => {
		expect(xToIndex(0, 100, 5)).toBe(0);
		expect(xToIndex(60, 100, 5)).toBe(2);
		expect(xToIndex(140, 100, 5)).toBe(4);
		expect(xToIndex(-5, 100, 5)).toBe(0);
	});
});
