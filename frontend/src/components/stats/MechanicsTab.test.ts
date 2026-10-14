import { describe, expect, it } from "vitest";
import { fold, rowsThatFit } from "./MechanicsTab.tsx";

describe("rowsThatFit", () => {
	it("shows as many rows as the measured height holds, at least one", () => {
		expect(rowsThatFit(95, 6)).toBe(5);
		expect(rowsThatFit(20, 6)).toBe(1);
		expect(rowsThatFit(1000, 6)).toBe(6);
	});

	it("shows every row before the box is measured", () => {
		expect(rowsThatFit(0, 6)).toBe(6);
	});
});

describe("fold", () => {
	const items = ["a", "b", "c", "d"].map((key, i) => ({ key, label: key, count: 10 - i }));

	it("keeps the top rows and folds the rest into one row", () => {
		expect(fold(items, 3).map((e) => [e.label, e.count])).toEqual([
			["a", 10],
			["b", 9],
			["2 more", 15],
		]);
	});

	it("shows the top entry, not a fold of everything, when one row fits", () => {
		expect(fold(items, 1).map((e) => e.label)).toEqual(["a"]);
	});
});
