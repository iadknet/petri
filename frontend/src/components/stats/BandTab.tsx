import { useMemo } from "react";
import type { StatsSample } from "../../stores/stats.ts";
import { TrendCanvas } from "./TrendCanvas.tsx";
import { BAND_COLOR } from "./bandColors.ts";
import {
	type BandId,
	type Reading,
	type Tone,
	buildReadings,
	changeOverWindow,
	formatDelta,
} from "./readings.ts";

const TONE_CLASS: Record<Tone, string> = {
	good: "text-emerald-400",
	bad: "text-rose-400",
	neutral: "text-slate-300",
	flat: "text-slate-400",
};

const TONE_WORD: Record<Tone, string> = {
	good: "better",
	bad: "worse",
	neutral: "",
	flat: "steady",
};

/** One band's readings as side-by-side charts that fill the panel: small multiples on one time axis. */
export function BandTab({
	band,
	samples,
	foodNames,
	index,
	scrubbing,
	onScrub,
}: {
	band: BandId;
	samples: StatsSample[];
	foodNames: string[];
	/** Sample shown in the values: the latest, or the scrubbed one. */
	index: number;
	scrubbing: boolean;
	onScrub: (index: number | null) => void;
}) {
	const readings = useMemo(
		() => buildReadings(samples, foodNames).filter((r) => r.band === band),
		[samples, foodNames, band],
	);

	return (
		<div className="@container h-full">
			<div className="grid h-full grid-cols-2 grid-rows-2 @3xl:grid-cols-4 @3xl:grid-rows-1">
				{readings.map((reading) => (
					<ReadingChart
						key={reading.id}
						reading={reading}
						index={index}
						scrub={scrubbing ? index : null}
						onScrub={onScrub}
					/>
				))}
			</div>
		</div>
	);
}

function extent(values: (number | null)[]): { min: number; max: number } | null {
	let min = Number.POSITIVE_INFINITY;
	let max = Number.NEGATIVE_INFINITY;
	for (const v of values) {
		if (v == null) continue;
		if (v < min) min = v;
		if (v > max) max = v;
	}
	return min <= max ? { min, max } : null;
}

function ReadingChart({
	reading,
	index,
	scrub,
	onScrub,
}: {
	reading: Reading;
	index: number;
	scrub: number | null;
	onScrub: (index: number | null) => void;
}) {
	const value = reading.values[index];
	const change = changeOverWindow(reading.values, index, reading.better);
	const detail = reading.detail?.(index) ?? null;
	const range = useMemo(() => extent(reading.values), [reading.values]);
	const word = change.delta == null ? "" : TONE_WORD[change.tone];

	return (
		<section
			// biome-ignore lint/a11y/useSemanticElements: a named group of one reading's value, change and chart
			role="group"
			aria-label={reading.label}
			className="flex min-h-0 min-w-0 flex-col border-petri-border px-3 pt-2 pb-2 odd:border-r [&:nth-child(-n+2)]:border-b @3xl:border-r @3xl:last:border-r-0 @3xl:[&:nth-child(-n+2)]:border-b-0"
		>
			<h3
				className="w-fit max-w-full cursor-help truncate text-xs text-slate-300 decoration-slate-500 decoration-dotted underline-offset-4 hover:underline"
				title={reading.hint}
			>
				{reading.label}
			</h3>
			<div className="mt-0.5 flex min-w-0 items-baseline gap-2">
				<span
					data-testid="reading-value"
					className="font-mono text-[22px] font-medium leading-7 text-slate-50 tabular-nums"
				>
					{value == null ? <span className="text-slate-400">–</span> : reading.format(value)}
				</span>
				{change.delta != null && (
					<span className={`truncate text-xs ${TONE_CLASS[change.tone]}`}>
						<span className="font-mono tabular-nums">{formatDelta(reading, change.delta)}</span>
						{word && <span className="ml-1">{word}</span>}
					</span>
				)}
			</div>
			<p className="flex h-4 min-w-0 items-center gap-1 text-[11px] text-slate-400">
				{reading.secondary && detail && (
					<span aria-hidden="true" className="inline-block h-px w-2.5 shrink-0 bg-slate-400" />
				)}
				<span className="truncate">{detail}</span>
			</p>
			<div className="relative mt-1.5 min-h-6 flex-1">
				<TrendCanvas
					values={reading.values}
					secondary={reading.secondary}
					color={BAND_COLOR[reading.band]}
					scrubIndex={scrub}
					onScrub={onScrub}
				/>
				{range && range.max !== range.min && (
					<>
						<span
							aria-hidden="true"
							className="pointer-events-none absolute top-0 left-0 rounded-sm bg-petri-panel/80 px-0.5 font-mono text-[11px] leading-none text-slate-400 tabular-nums"
						>
							{reading.format(range.max)}
						</span>
						<span
							aria-hidden="true"
							className="pointer-events-none absolute bottom-1 left-0 rounded-sm bg-petri-panel/80 px-0.5 font-mono text-[11px] leading-none text-slate-400 tabular-nums"
						>
							{reading.format(range.min)}
						</span>
					</>
				)}
			</div>
		</section>
	);
}
