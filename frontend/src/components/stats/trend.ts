/** Geometry for the dock's row trends: one x per sample, shared by every row. */

const INSET = 2;

export interface Point {
	x: number;
	y: number;
}

export interface TrendGeometry {
	/** Unbroken stretches of the main series; a missing value ends a run. */
	runs: Point[][];
	secondaryRuns: Point[][];
}

export function sampleX(index: number, count: number, width: number): number {
	return count < 2 ? width : (index / (count - 1)) * width;
}

export function xToIndex(x: number, width: number, count: number): number {
	if (count < 2 || width <= 0) return Math.max(count - 1, 0);
	const index = Math.round((x / width) * (count - 1));
	return Math.min(Math.max(index, 0), count - 1);
}

export function trendGeometry(
	values: (number | null)[],
	secondary: (number | null)[] | undefined,
	width: number,
	height: number,
	/** Smallest y-range as a share of the mean, so noise is not stretched to full height. */
	minRangeShare = 0,
): TrendGeometry {
	let min = Number.POSITIVE_INFINITY;
	let max = Number.NEGATIVE_INFINITY;
	for (const series of [values, secondary ?? []]) {
		for (const v of series) {
			if (v == null) continue;
			if (v < min) min = v;
			if (v > max) max = v;
		}
	}
	const floor = minRangeShare * Math.abs((min + max) / 2);
	if (max - min < floor) {
		const mid = (min + max) / 2;
		min = mid - floor / 2;
		max = mid + floor / 2;
	}
	const range = max - min;
	const y = (v: number) =>
		range === 0 ? height / 2 : height - INSET - ((v - min) / range) * (height - 2 * INSET);
	const runsOf = (series: (number | null)[]) => {
		const runs: Point[][] = [];
		let run: Point[] = [];
		series.forEach((v, i) => {
			if (v == null) {
				if (run.length) runs.push(run);
				run = [];
				return;
			}
			run.push({ x: sampleX(i, series.length, width), y: y(v) });
		});
		if (run.length) runs.push(run);
		return runs;
	};
	return { runs: runsOf(values), secondaryRuns: secondary ? runsOf(secondary) : [] };
}
