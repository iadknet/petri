/** Live readings for the stats dock, derived from the sample ring. */

import type { StatsSample } from "../../stores/stats.ts";

export type BandId = "world" | "evolution" | "brains";
export type Better = "up" | "down" | "none";
export type Tone = "good" | "bad" | "flat" | "neutral";

export interface Reading {
	id: string;
	band: BandId;
	label: string;
	/** Tooltip: what the number counts, in plain words. */
	hint: string;
	better: Better;
	/** One value per sample; null where it cannot be read yet. */
	values: (number | null)[];
	/** A second trace on the same scale (deaths beside births). */
	secondary?: (number | null)[];
	format: (value: number) => string;
	/** Unit after a change, when it differs from the value's own (percentage points). */
	deltaSuffix?: string;
	/** Small print under the value at sample `index`. */
	detail?: (index: number) => string | null;
}

export const BANDS: { id: BandId; label: string }[] = [
	{ id: "world", label: "World" },
	{ id: "evolution", label: "Evolution" },
	{ id: "brains", label: "Brains" },
];

/** Samples behind a rate: short enough to follow change, long enough to smooth a tick. */
const RATE_WINDOW = 12;
/** Mutants are judged only at death, so their share needs a longer window. */
const MUTANT_WINDOW = 120;
const MUTANT_MIN_JUDGED = 25;
/** A change smaller than this share of the reading reads as no change. */
export const NOISE_FLOOR = 0.03;

export function formatCompact(value: number): string {
	const abs = Math.abs(value);
	if (abs >= 1e6) return `${trim(value / 1e6, 1)}M`;
	if (abs >= 1e4) return `${trim(value / 1e3, 1)}k`;
	if (abs >= 100 || Number.isInteger(value)) return Math.round(value).toLocaleString("en-US");
	if (abs >= 1) return trim(value, 2);
	if (abs === 0) return "0";
	return value.toLocaleString("en-US", { maximumSignificantDigits: 2, maximumFractionDigits: 20 });
}

function trim(value: number, digits: number): string {
	return Number(value.toFixed(digits)).toString();
}

const percent = (value: number) => `${trim(value, value >= 10 ? 0 : 1)}%`;

/** Fixed decimals, so a row's figures line up down the column; big values compact. */
export function fixed(digits: number): (value: number) => string {
	return (value) =>
		Math.abs(value) >= 1e4
			? formatCompact(value)
			: value.toLocaleString("en-US", {
					minimumFractionDigits: digits,
					maximumFractionDigits: digits,
				});
}

/** Two significant figures, for per-creature rates that span orders of magnitude. */
const sig2 = (value: number) =>
	value === 0
		? "0"
		: value.toLocaleString("en-US", { maximumSignificantDigits: 2, maximumFractionDigits: 20 });

/**
 * Growth of `num` over growth of `den` across the trailing `window` samples.
 * The window never reaches back across a restart, where counters fall, and
 * with `full` a rate is read only once a whole window has passed.
 */
export function ratioSeries(
	samples: StatsSample[],
	num: (s: StatsSample) => number,
	den: (s: StatsSample) => number,
	window: number,
	{ minDen = 0, full = true }: { minDen?: number; full?: boolean } = {},
): (number | null)[] {
	const out: (number | null)[] = [];
	let segmentStart = 0;
	for (let i = 0; i < samples.length; i++) {
		const s = samples[i]!;
		if (i > 0) {
			const prev = samples[i - 1]!;
			if (s.tick < prev.tick || num(s) < num(prev) || den(s) < den(prev)) segmentStart = i;
		}
		// A partial window would start every rate near zero and read as change.
		if (full && i - segmentStart < window) {
			out.push(null);
			continue;
		}
		const j = Math.max(segmentStart, i - window);
		const first = samples[j]!;
		const dDen = den(s) - den(first);
		out.push(j < i && dDen > minDen ? (num(s) - num(first)) / dDen : null);
	}
	return out;
}

const judged = (s: StatsSample) => s.mutantsHelpful + s.mutantsNeutral + s.mutantsDetrimental;

export function buildReadings(samples: StatsSample[], foodNames: string[] = []): Reading[] {
	const tick = (s: StatsSample) => s.tick;
	const births = ratioSeries(samples, (s) => s.birthsTotal, tick, RATE_WINDOW);
	const deaths = ratioSeries(samples, (s) => s.deathsTotal, tick, RATE_WINDOW);
	// Judged mutants gate their own share, so a partial window is fine here.
	const judgedWindow = { minDen: MUTANT_MIN_JUDGED - 1, full: false };
	const better = ratioSeries(samples, (s) => s.mutantsHelpful, judged, MUTANT_WINDOW, judgedWindow);
	const worse = ratioSeries(
		samples,
		(s) => s.mutantsDetrimental,
		judged,
		MUTANT_WINDOW,
		judgedWindow,
	);
	const creatureTicks = (s: StatsSample) => s.creatureTicksTotal;
	const at = (i: number) => samples[i];

	return [
		{
			id: "population",
			band: "world",
			label: "Creatures alive",
			hint: "Living creatures at the tick.",
			better: "up",
			values: samples.map((s) => s.population),
			format: fixed(0),
		},
		{
			id: "births",
			band: "world",
			label: "Births / tick",
			hint: "Children delivered per tick (line) against deaths per tick (grey line), over the last few samples.",
			better: "up",
			values: births,
			secondary: deaths,
			format: fixed(2),
			detail: (i) => {
				const d = deaths[i];
				return d == null ? null : `${fixed(2)(d)} deaths / tick`;
			},
		},
		{
			id: "energy",
			band: "world",
			label: "Mean energy",
			hint: "Average energy per living creature.",
			better: "up",
			values: samples.map((s) => s.meanEnergy),
			format: fixed(1),
		},
		{
			id: "food",
			band: "world",
			label: "Food on the map",
			hint: "Food density summed over every cell, all food types.",
			better: "none",
			values: samples.map((s) => s.foodTotal),
			format: formatCompact,
			detail: (i) => foodSplit(at(i), foodNames),
		},
		{
			id: "generation",
			band: "evolution",
			label: "Generation",
			hint: "Mean generation of living creatures; founders are generation 0.",
			better: "none",
			values: samples.map((s) => s.generationMean),
			format: fixed(1),
			detail: (i) => {
				const s = at(i);
				return s ? `deepest ${fixed(0)(s.generationMax)}` : null;
			},
		},
		{
			id: "lineages",
			band: "evolution",
			label: "Founder lineages",
			hint: "Effective number of founder lineages: how many equally common lineages would give the same diversity. Small print: lineages with any living descendant.",
			better: "up",
			values: samples.map((s) => s.lineagesEffective),
			format: fixed(1),
			detail: (i) => {
				const s = at(i);
				return s ? `${fixed(0)(s.lineagesAlive)} alive` : null;
			},
		},
		{
			id: "genome",
			band: "evolution",
			label: "Genome size",
			hint: "Mean genome size of living creatures, in units. Watch for bloat: large genomes slow births.",
			better: "none",
			values: samples.map((s) => s.genomeSizeMean),
			format: fixed(0),
		},
		{
			id: "mutants",
			band: "evolution",
			label: "Mutants beating average",
			hint: `Of mutants judged at death in recent samples, the share whose viability scored clearly above the baseline. Needs ${MUTANT_MIN_JUDGED} judged.`,
			better: "up",
			values: better.map((v) => (v == null ? null : v * 100)),
			format: percent,
			deltaSuffix: " pts",
			detail: (i) => {
				const w = worse[i];
				return w == null ? "too few judged yet" : `${percent(w * 100)} fell below`;
			},
		},
		{
			id: "weights",
			band: "brains",
			label: "Weight changes",
			hint: "Plasticity updates that changed a stored edge weight, per creature per tick.",
			better: "none",
			values: ratioSeries(samples, (s) => s.weightChangesTotal, creatureTicks, RATE_WINDOW),
			format: sig2,
			detail: () => "per creature-tick",
		},
		{
			id: "memory",
			band: "brains",
			label: "Memory writes",
			hint: "Writes that changed a shared-memory slot, per creature per tick.",
			better: "none",
			values: ratioSeries(samples, (s) => s.memoryWritesTotal, creatureTicks, RATE_WINDOW),
			format: sig2,
			detail: () => "per creature-tick",
		},
		{
			id: "inputs",
			band: "brains",
			label: "World inputs read",
			hint: "Distinct world inputs a creature's last mesh run read, averaged over living creatures.",
			better: "none",
			values: samples.map((s) => s.worldInputsReadMean),
			format: fixed(2),
			detail: () => "per creature",
		},
		{
			id: "thinking",
			band: "brains",
			label: "Thinking cost",
			hint: "Energy spent on compute per creature in the last tick.",
			better: "none",
			values: samples.map((s) => s.thinkingCostMean),
			format: sig2,
			detail: () => "energy per creature-tick",
		},
	];
}

function foodSplit(s: StatsSample | undefined, names: string[]): string | null {
	const total = s?.foodTotal;
	if (!s || total == null || total <= 0 || s.foodByType.length < 2) return null;
	return s.foodByType
		.map((v, i) => `${names[i] ?? `type ${i}`} ${percent((v / total) * 100)}`)
		.join(" · ");
}

/** A signed change in the reading's own format, with a true minus sign. */
export function formatDelta(reading: Reading, delta: number): string {
	const sign = delta > 0 ? "+" : delta < 0 ? "\u2212" : "\u00b1";
	const body = reading.deltaSuffix ? trim(Math.abs(delta), 1) : reading.format(Math.abs(delta));
	return `${sign}${body}${reading.deltaSuffix ?? ""}`;
}

export function readingById(readings: Reading[], id: string): Reading {
	const reading = readings.find((r) => r.id === id);
	if (!reading) throw new Error(`unknown reading ${id}`);
	return reading;
}

/** Change from the first value in the window to the value at `index`. */
export function changeOverWindow(
	values: (number | null)[],
	index: number,
	better: Better,
): { delta: number | null; tone: Tone } {
	const first = values.find((v) => v != null);
	const current = values[index];
	if (first == null || current == null) return { delta: null, tone: "flat" };
	const delta = current - first;
	const scale = Math.max(Math.abs(first), Math.abs(current));
	if (scale === 0 || Math.abs(delta) <= scale * NOISE_FLOOR) return { delta, tone: "flat" };
	if (better === "none") return { delta, tone: "neutral" };
	return { delta, tone: delta > 0 === (better === "up") ? "good" : "bad" };
}

function lastValue(values: (number | null)[] | undefined): number | null {
	if (!values) return null;
	for (let i = values.length - 1; i >= 0; i--) {
		const v = values[i];
		if (v != null) return v;
	}
	return null;
}

export interface Verdict {
	/** One word for a tab label. */
	word: string;
	/** Answer first: a trend word and one figure, short enough for the collapsed strip. */
	short: string;
	/** The short form plus the figures behind it. */
	full: string;
}

const signed = (value: number, format: (v: number) => string) =>
	`${value > 0 ? "+" : value < 0 ? "\u2212" : "\u00b1"}${format(Math.abs(value))}`;

function trendWord(first: number, last: number, words: [string, string, string]): string {
	const change = first === 0 ? (last > 0 ? 1 : 0) : (last - first) / Math.abs(first);
	return change > NOISE_FLOOR ? words[0] : change < -NOISE_FLOOR ? words[1] : words[2];
}

/** Plain words per band over the samples given: the window, or the window up to a scrubbed tick. */
export function bandVerdict(band: BandId, samples: StatsSample[]): Verdict {
	const first = samples[0];
	const last = samples.at(-1);
	if (!first || !last) {
		const waiting = "Waiting for the first tick";
		return { word: "Waiting", short: waiting, full: waiting };
	}
	const span = last.tick - first.tick;
	const over = span > 0 ? ` in ${span.toLocaleString("en-US")} ticks` : "";
	// Rates over the whole window, the same span the trend word reads.
	const whole = (num: (s: StatsSample) => number, den: (s: StatsSample) => number) =>
		lastValue(ratioSeries(samples, num, den, samples.length, { full: false }));

	if (last.population === 0) {
		const extinct = band === "world" ? "Extinct · no creatures alive" : "No creatures alive";
		return { word: "Extinct", short: extinct, full: extinct };
	}
	if (band === "world") {
		const trend = trendWord(first.population, last.population, [
			"Growing",
			"Shrinking",
			"Holding steady",
		]);
		const short = `${trend} · ${signed(last.population - first.population, fixed(0))} creatures`;
		const word = trend === "Holding steady" ? "Steady" : trend;
		const born = whole(
			(s) => s.birthsTotal,
			(s) => s.tick,
		);
		const died = whole(
			(s) => s.deathsTotal,
			(s) => s.tick,
		);
		if (born == null || died == null) return { word, short, full: `${short}${over}` };
		return {
			word,
			short,
			full: `${short}${over} · ${fixed(2)(born)} born vs ${fixed(2)(died)} died per tick`,
		};
	}
	if (band === "evolution") {
		const trend = trendWord(first.lineagesEffective, last.lineagesEffective, [
			"Lineages widening",
			"Lineages narrowing",
			"Lineages steady",
		]);
		const short = `${trend} · ${signed(last.generationMean - first.generationMean, fixed(1))} generations`;
		return {
			word: trend.replace("Lineages ", "").replace(/^./, (c) => c.toUpperCase()),
			short,
			full: `${short}${over} · ${fixed(1)(last.lineagesEffective)} effective of ${fixed(0)(last.lineagesAlive)} alive`,
		};
	}
	const creatureTicks = (s: StatsSample) => s.creatureTicksTotal;
	const weights = whole((s) => s.weightChangesTotal, creatureTicks);
	const memory = whole((s) => s.memoryWritesTotal, creatureTicks);
	if (weights == null || memory == null) {
		const waiting = "Waiting for a few ticks of mesh work";
		return { word: "Waiting", short: waiting, full: waiting };
	}
	const word = weights > 0 ? "Weights changing" : "Weights static";
	const short = `${word} · ${memory > 0 ? "memory in use" : "memory unused"}`;
	return {
		word,
		short,
		full: `${short} · ${sig2(weights)} weight changes and ${sig2(memory)} memory writes per creature-tick`,
	};
}
