import { describe, expect, it } from "vitest";
import { createOccupancy, tileCornerRadii, tileFillets } from "./foodTiles.ts";

function grid(rows: string[]) {
	const occupancy = createOccupancy(rows[0]!.length, rows.length);
	rows.forEach((row, y) => {
		for (let x = 0; x < row.length; x++) {
			if (row[x] === "#") occupancy.cells[y * occupancy.width + x] = 1;
		}
	});
	return occupancy;
}

describe("tileCornerRadii", () => {
	it("rounds every corner of a lone cell into a circle", () => {
		expect(tileCornerRadii(grid(["...", ".#.", "..."]), 1, 1, 5)).toEqual([5, 5, 5, 5]);
	});

	it("keeps edges shared with a neighbour square so adjacent cells join", () => {
		const row = grid(["###"]);
		expect(tileCornerRadii(row, 0, 0, 5)).toEqual([5, 0, 0, 5]);
		expect(tileCornerRadii(row, 1, 0, 5)).toEqual([0, 0, 0, 0]);
		expect(tileCornerRadii(row, 2, 0, 5)).toEqual([0, 5, 5, 0]);
	});

	it("keeps the corner square where a diagonal neighbour joins", () => {
		const diagonal = grid(["#.", ".#"]);
		expect(tileCornerRadii(diagonal, 0, 0, 5)).toEqual([5, 5, 0, 5]);
		expect(tileCornerRadii(diagonal, 1, 1, 5)).toEqual([0, 5, 5, 5]);
	});

	it("treats cells beyond the grid edge as empty", () => {
		expect(tileCornerRadii(grid(["#"]), 0, 0, 2)).toEqual([2, 2, 2, 2]);
	});
});

describe("tileFillets", () => {
	it("fills the concave corner of an L-shaped patch once", () => {
		const shape = grid(["##", "#."]);
		const all = [
			...tileFillets(shape, 0, 0),
			...tileFillets(shape, 1, 0),
			...tileFillets(shape, 0, 1),
		];
		expect(all).toEqual([{ x: 1, y: 1, dx: 1, dy: 1 }]);
	});

	it("bridges diagonal neighbours with a fillet on each side of the neck", () => {
		const diagonal = grid(["#.", ".#"]);
		const all = [...tileFillets(diagonal, 0, 0), ...tileFillets(diagonal, 1, 1)];
		expect(all).toEqual([
			{ x: 1, y: 1, dx: 1, dy: -1 },
			{ x: 1, y: 1, dx: -1, dy: 1 },
		]);
	});

	it("rounds all four corners of an enclosed empty cell into a bubble", () => {
		const ring = grid(["###", "#.#", "###"]);
		const all = [];
		for (let y = 0; y < 3; y++) {
			for (let x = 0; x < 3; x++) {
				if (x === 1 && y === 1) continue;
				all.push(...tileFillets(ring, x, y));
			}
		}
		expect(all).toHaveLength(4);
		expect(new Set(all.map((f) => `${f.x},${f.y}`))).toEqual(new Set(["1,1", "2,1", "1,2", "2,2"]));
	});

	it("draws nothing for a straight run", () => {
		const row = grid(["###"]);
		expect([0, 1, 2].flatMap((x) => tileFillets(row, x, 0))).toEqual([]);
	});
});
