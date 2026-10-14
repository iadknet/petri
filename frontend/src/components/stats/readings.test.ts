import { describe, expect, it } from "vitest";
import type { StatsSample } from "../../stores/stats.ts";
import {
	bandVerdict,
	buildReadings,
	changeOverWindow,
	fixed,
	formatCompact,
	ratioSeries,
	readingById,
} from "./readings.ts";

function sample(tick: number, overrides: Partial<StatsSample> = {}): StatsSample {
	return {
		tick,
		population: 100,
		meanEnergy: 20,
		birthsTotal: 0,
		deathsTotal: 0,
		foodTotal: 50,
		foodByType: [50],
		generationMean: 1,
		generationMax: 2,
		genomeSizeMean: 97,
		lineagesAlive: 10,
		lineagesEffective: 8,
		mutantsHelpful: 0,
		mutantsNeutral: 0,
		mutantsDetrimental: 0,
		weightChangesTotal: 0,
		memoryWritesTotal: 0,
		creatureTicksTotal: 0,
		worldInputsReadMean: 3,
		thinkingCostMean: 0.4,
		...overrides,
	};
}

describe("ratioSeries", () => {
	const births = (s: StatsSample) => s.birthsTotal;
	const ticks = (s: StatsSample) => s.tick;

	it("divides counter growth by tick growth once a whole window has passed", () => {
		const samples = [0, 1, 2, 3].map((i) => sample(i * 10, { birthsTotal: i * 30 }));
		expect(ratioSeries(samples, births, ticks, 2)).toEqual([null, null, 3, 3]);
	});

	it("reads partial windows only when asked", () => {
		const samples = [0, 1, 2].map((i) => sample(i, { birthsTotal: i }));
		expect(ratioSeries(samples, births, ticks, 5)).toEqual([null, null, null]);
		expect(ratioSeries(samples, births, ticks, 5, { full: false })).toEqual([null, 1, 1]);
	});

	it("never reaches back across a counter reset", () => {
		const samples = [
			sample(10, { birthsTotal: 500 }),
			sample(20, { birthsTotal: 900 }),
			sample(1, { birthsTotal: 0 }),
			sample(2, { birthsTotal: 4 }),
		];
		const rates = ratioSeries(samples, births, ticks, 1);
		expect(rates[2]).toBeNull();
		expect(rates[3]).toBe(4);
	});

	it("stays null until the denominator has grown", () => {
		expect(ratioSeries([sample(5), sample(5)], births, ticks, 1)).toEqual([null, null]);
	});
});

describe("buildReadings", () => {
	it("reads the share of judged mutants that beat the baseline, once enough were judged", () => {
		const few = [sample(0), sample(1, { mutantsHelpful: 2, mutantsNeutral: 3 })];
		expect(readingById(buildReadings(few), "mutants").values.at(-1)).toBeNull();

		const many = [
			sample(0),
			sample(1, { mutantsHelpful: 20, mutantsNeutral: 60, mutantsDetrimental: 20 }),
		];
		expect(readingById(buildReadings(many), "mutants").values.at(-1)).toBeCloseTo(20);
	});

	it("puts births on the main trace and deaths on the second, from the first whole window", () => {
		const samples = Array.from({ length: 13 }, (_, i) =>
			sample(i * 10, { birthsTotal: i * 50, deathsTotal: i * 20 }),
		);
		const reading = readingById(buildReadings(samples), "births");
		expect(reading.values.slice(0, 12).every((v) => v == null)).toBe(true);
		expect(reading.values.at(-1)).toBe(5);
		expect(reading.secondary?.at(-1)).toBe(2);
	});

	it("rates weight changes per creature-tick", () => {
		const samples = Array.from({ length: 13 }, (_, i) =>
			sample(i, { weightChangesTotal: i * 30, creatureTicksTotal: i * 100 }),
		);
		expect(readingById(buildReadings(samples), "weights").values.at(-1)).toBeCloseTo(0.3);
	});

	it("formats each reading with fixed decimals so its column lines up", () => {
		const energy = readingById(buildReadings([sample(0, { meanEnergy: 25 })]), "energy");
		expect(energy.format(25)).toBe("25.0");
		expect(energy.format(3.14259)).toBe("3.1");
		expect(fixed(2)(12_481)).toBe("12.5k");
	});
});

describe("changeOverWindow", () => {
	it("is good when a higher-is-better reading rises past the noise floor", () => {
		expect(changeOverWindow([100, 120], 1, "up")).toEqual({ delta: 20, tone: "good" });
		expect(changeOverWindow([100, 80], 1, "up")).toEqual({ delta: -20, tone: "bad" });
	});

	it("is flat inside the noise floor and neutral for readings with no better direction", () => {
		expect(changeOverWindow([100, 101], 1, "up").tone).toBe("flat");
		expect(changeOverWindow([100, 150], 1, "none").tone).toBe("neutral");
	});

	it("measures from the first value present, up to the scrubbed index", () => {
		expect(changeOverWindow([null, 10, 15, 40], 2, "up").delta).toBe(5);
		expect(changeOverWindow([null, null], 1, "up").delta).toBeNull();
	});
});

describe("bandVerdict", () => {
	it("waits before the first sample", () => {
		expect(bandVerdict("world", []).short).toBe("Waiting for the first tick");
	});

	it("leads with the trend word and the net change, then the rates behind it", () => {
		const samples = [
			sample(0, { population: 200 }),
			sample(10, { population: 150, birthsTotal: 10, deathsTotal: 60 }),
		];
		expect(bandVerdict("world", samples)).toMatchObject({
			short: "Shrinking · \u221250 creatures",
			full: "Shrinking · \u221250 creatures in 10 ticks · 1.00 born vs 6.00 died per tick",
		});
	});

	it("reads births and deaths over the same window as the trend word", () => {
		const samples = [sample(0, { population: 300 })];
		for (let t = 1; t <= 20; t++) {
			// Deaths outpace births early; the last few ticks recover.
			const late = t > 15;
			samples.push(
				sample(t * 10, {
					population: 300 - t * 8 + (late ? (t - 15) * 3 : 0),
					birthsTotal: t * 10 + (late ? (t - 15) * 50 : 0),
					deathsTotal: t * 90,
				}),
			);
		}
		expect(bandVerdict("world", samples).full).toMatch(/2\.25 born vs 9\.00 died per tick$/);
	});

	it("calls an empty world extinct in every band", () => {
		const samples = [sample(0), sample(1, { population: 0 })];
		expect(bandVerdict("world", samples).short).toBe("Extinct · no creatures alive");
		expect(bandVerdict("evolution", samples).short).toBe("No creatures alive");
		expect(bandVerdict("brains", samples).short).toBe("No creatures alive");
	});

	it("names lineage diversity and generations gained for evolution", () => {
		const samples = [
			sample(0, { lineagesEffective: 10, generationMean: 2 }),
			sample(100, { lineagesEffective: 6, generationMean: 9, lineagesAlive: 8 }),
		];
		expect(bandVerdict("evolution", samples)).toMatchObject({
			short: "Lineages narrowing · +7.0 generations",
			full: "Lineages narrowing · +7.0 generations in 100 ticks · 6.0 effective of 8 alive",
		});
	});

	it("names the brain counters, not abilities", () => {
		const samples = [
			sample(0),
			sample(1, { weightChangesTotal: 25, memoryWritesTotal: 0, creatureTicksTotal: 100 }),
		];
		expect(bandVerdict("brains", samples)).toMatchObject({
			short: "Weights changing · memory unused",
			full: "Weights changing · memory unused · 0.25 weight changes and 0 memory writes per creature-tick",
		});
	});
});

describe("verdict words for tab labels", () => {
	it("gives each band one word", () => {
		const shrinking = [sample(0, { population: 200 }), sample(10, { population: 150 })];
		expect(bandVerdict("world", shrinking).word).toBe("Shrinking");
		const narrowing = [sample(0, { lineagesEffective: 10 }), sample(10, { lineagesEffective: 6 })];
		expect(bandVerdict("evolution", narrowing).word).toBe("Narrowing");
		const learning = [sample(0), sample(1, { weightChangesTotal: 5, creatureTicksTotal: 100 })];
		expect(bandVerdict("brains", learning).word).toBe("Weights changing");
		expect(bandVerdict("brains", [sample(0), sample(1, { population: 0 })]).word).toBe("Extinct");
		expect(bandVerdict("world", []).word).toBe("Waiting");
	});
});

describe("formatCompact", () => {
	it("keeps small numbers readable and compacts large ones", () => {
		expect(formatCompact(0.0042)).toBe("0.0042");
		expect(formatCompact(0.00000079)).toBe("0.00000079");
		expect(formatCompact(3.14259)).toBe("3.14");
		expect(formatCompact(812)).toBe("812");
		expect(formatCompact(12_481)).toBe("12.5k");
		expect(formatCompact(2_400_000)).toBe("2.4M");
	});
});
