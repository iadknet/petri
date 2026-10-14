/**
 * Food tiles flow together like liquid. Each cell is a full square; a corner
 * is rounded only where none of the three cells meeting it holds food, and
 * every concave corner of an empty cell between food is filled with a fillet.
 * Adjacent food reads as one continuous blob, a lone cell is a circle, an
 * enclosed empty cell is a bubble, and diagonal neighbours join by a neck.
 */

/** Grid of occupied cells, row-major, 1 where a cell holds food. */
export interface Occupancy {
	cells: Uint8Array;
	width: number;
	height: number;
}

export function createOccupancy(width: number, height: number): Occupancy {
	return { cells: new Uint8Array(Math.max(0, width * height)), width, height };
}

function occupied(grid: Occupancy, x: number, y: number): boolean {
	return (
		x >= 0 && y >= 0 && x < grid.width && y < grid.height && grid.cells[y * grid.width + x] === 1
	);
}

/** Corner radii in canvas order: top-left, top-right, bottom-right, bottom-left. */
export function tileCornerRadii(
	grid: Occupancy,
	x: number,
	y: number,
	radius: number,
): [number, number, number, number] {
	const free = (dx: number, dy: number) =>
		!occupied(grid, x + dx, y) && !occupied(grid, x, y + dy) && !occupied(grid, x + dx, y + dy);
	return [
		free(-1, -1) ? radius : 0,
		free(1, -1) ? radius : 0,
		free(1, 1) ? radius : 0,
		free(-1, 1) ? radius : 0,
	];
}

/**
 * A fillet to draw for an occupied cell: `x`, `y` is the corner point in cell
 * units and `dx`, `dy` point from it into the empty cell being filled.
 */
export interface Fillet {
	x: number;
	y: number;
	dx: 1 | -1;
	dy: 1 | -1;
}

/**
 * Fillets owned by the occupied cell at `x`, `y`: concave corners of the empty
 * cells to its left and right. Ownership by horizontal neighbour means every
 * concave corner is drawn exactly once.
 */
export function tileFillets(grid: Occupancy, x: number, y: number): Fillet[] {
	const fillets: Fillet[] = [];
	if (!occupied(grid, x + 1, y)) {
		if (occupied(grid, x + 1, y - 1)) fillets.push({ x: x + 1, y, dx: 1, dy: 1 });
		if (occupied(grid, x + 1, y + 1)) fillets.push({ x: x + 1, y: y + 1, dx: 1, dy: -1 });
	}
	if (!occupied(grid, x - 1, y)) {
		if (occupied(grid, x - 1, y - 1)) fillets.push({ x, y, dx: -1, dy: 1 });
		if (occupied(grid, x - 1, y + 1)) fillets.push({ x, y: y + 1, dx: -1, dy: -1 });
	}
	return fillets;
}
