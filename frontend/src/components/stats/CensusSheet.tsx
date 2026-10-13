import { useEffect, useMemo, useState } from "react";
import type { StatsSample } from "../../stores/stats.ts";
import { TrendCanvas } from "./TrendCanvas.tsx";
import { BAND_COLOR } from "./bandColors.ts";
import {
	BANDS,
	type Reading,
	type Tone,
	bandVerdict,
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

/** Readings as ruled rows in three bands: now, change over the window, and trend. */
export function CensusSheet({
	samples,
	foodNames,
	onScrubbingChange,
}: {
	samples: StatsSample[];
	foodNames: string[];
	/** Tells the dock when the rows show the past, so its live strip can say "now". */
	onScrubbingChange?: (scrubbing: boolean) => void;
}) {
	const [scrub, setScrub] = useState<number | null>(null);
	const readings = useMemo(() => buildReadings(samples, foodNames), [samples, foodNames]);
	const last = samples.length - 1;
	const scrubbing = scrub != null && scrub < last;
	useEffect(() => onScrubbingChange?.(scrubbing), [scrubbing, onScrubbingChange]);
	// Collapsing the dock unmounts the sheet: leave the strip reading "now".
	useEffect(() => () => onScrubbingChange?.(false), [onScrubbingChange]);
	const index = scrubbing ? scrub : last;
	const span = last > 0 ? samples[last]!.tick - samples[0]!.tick : 0;
	const at = samples[index]!;
	// Band verdicts read the same span as the rows: up to the scrubbed tick.
	const upTo = useMemo(() => samples.slice(0, index + 1), [samples, index]);

	return (
		<table
			aria-label="Census of live readings"
			className="w-full table-fixed border-collapse text-[13px]"
			onPointerLeave={() => setScrub(null)}
		>
			<colgroup>
				<col className="w-48" />
				<col className="w-32" />
				<col className="w-24" />
				<col />
			</colgroup>
			<thead className="sticky top-0 z-10 bg-petri-panel text-[11px] text-slate-400">
				<tr className="border-b border-petri-border">
					<th scope="col" className="py-1.5 pl-3 text-left font-normal">
						Reading
					</th>
					<th
						scope="col"
						className={`py-1.5 text-right font-normal tabular-nums ${scrubbing ? "text-sky-300" : ""}`}
					>
						{scrubbing ? `At tick ${at.tick.toLocaleString("en-US")}` : "Now"}
					</th>
					<th scope="col" className="py-1.5 text-right font-normal">
						Change
					</th>
					<th
						scope="col"
						title="Hover a trend, or focus here and use the arrow keys, to read every row at that tick"
						className="relative py-1.5 pr-3 pl-4 text-left font-normal has-[input:focus-visible]:outline-2 has-[input:focus-visible]:-outline-offset-2 has-[input:focus-visible]:outline-sky-400"
					>
						{span > 0 ? `Trend over the last ${span.toLocaleString("en-US")} ticks` : "Trend"}
						<input
							type="range"
							aria-label="Scrub through time"
							aria-valuetext={`tick ${at.tick}`}
							min={0}
							max={Math.max(last, 0)}
							value={index}
							onChange={(event) => {
								const next = Number(event.target.value);
								setScrub(next >= last ? null : next);
							}}
							onBlur={() => setScrub(null)}
							onKeyDown={(event) => {
								if (event.key !== "Escape" || !scrubbing) return;
								// Back to now without also clearing the map selection.
								event.preventDefault();
								setScrub(null);
							}}
							className="absolute inset-0 h-full w-full cursor-default opacity-0"
						/>
					</th>
				</tr>
			</thead>
			{BANDS.map((band) => (
				<tbody key={band.id}>
					<tr className="border-b border-petri-border bg-slate-950/60">
						<th scope="rowgroup" colSpan={4} className="px-3 py-1.5 text-left font-normal">
							<span className="flex min-w-0 items-baseline gap-2.5">
								<span
									aria-hidden="true"
									className="h-2 w-2 shrink-0 translate-y-[-1px] rounded-full"
									style={{ backgroundColor: BAND_COLOR[band.id] }}
								/>
								<span className="font-semibold text-slate-100">{band.label}</span>
								<span className="truncate text-slate-300">{bandVerdict(band.id, upTo).full}</span>
							</span>
						</th>
					</tr>
					{readings
						.filter((r) => r.band === band.id)
						.map((reading) => (
							<ReadingRow
								key={reading.id}
								reading={reading}
								index={index}
								scrub={scrubbing ? index : null}
								onScrub={setScrub}
							/>
						))}
				</tbody>
			))}
		</table>
	);
}

function ReadingRow({
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
	const changeText = change.delta == null ? "–" : formatDelta(reading, change.delta);

	return (
		<tr className="border-b border-petri-border/70 transition-colors duration-150 hover:bg-slate-800/30">
			<th scope="row" className="py-1.5 pr-2 pl-3 text-left font-normal">
				<span
					className="block cursor-help truncate text-slate-200 decoration-slate-500 decoration-dotted underline-offset-4 hover:underline"
					title={reading.hint}
				>
					{reading.label}
				</span>
			</th>
			<td className="py-1.5 text-right leading-tight">
				<span className="block font-mono text-lg font-medium leading-6 text-slate-50 tabular-nums">
					{value == null ? <span className="text-slate-400">–</span> : reading.format(value)}
				</span>
				{detail && (
					<span className="flex items-center justify-end gap-1 text-[11px] text-slate-400">
						{reading.secondary && (
							<span aria-hidden="true" className="inline-block h-px w-2.5 shrink-0 bg-slate-400" />
						)}
						<span className="truncate">{detail}</span>
					</span>
				)}
			</td>
			<td className={`py-1.5 text-right leading-tight ${TONE_CLASS[change.tone]}`}>
				<span className="block font-mono text-xs tabular-nums">{changeText}</span>
				{change.delta != null && TONE_WORD[change.tone] && (
					<span className="block text-[11px]">{TONE_WORD[change.tone]}</span>
				)}
			</td>
			<td className="py-1 pr-3 pl-4">
				<TrendCanvas
					values={reading.values}
					secondary={reading.secondary}
					color={BAND_COLOR[reading.band]}
					scrubIndex={scrub}
					onScrub={onScrub}
				/>
			</td>
		</tr>
	);
}
