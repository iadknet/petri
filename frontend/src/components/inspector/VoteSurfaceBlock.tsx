import { memo } from "react";
import type { TickTrace } from "../../types/trace.ts";
import { voteSinkLabel } from "./vmInstructionFormat.ts";

const COMMIT_KINDS = ["Eat", "Move", "Reproduce", "StealEnergy"] as const;

interface VoteSurfaceBlockProps {
	tick: TickTrace;
}

/**
 * Read-only view of the sampled tick's vote surface (T19.F03): the non-zero
 * vote entries and the four per-kind commit counters. Nothing in the
 * simulation reads these values, so this block reports and never steers.
 */
export const VoteSurfaceBlock = memo(function VoteSurfaceBlock({ tick }: VoteSurfaceBlockProps) {
	const nonZero = tick.votes
		.map((value, index) => ({ value, index }))
		.filter((entry) => entry.value !== 0);

	return (
		<div className="px-3 py-2 border-t border-white/[0.06]">
			<div className="text-xs text-slate-500 uppercase tracking-wider font-medium mb-1.5">
				Vote Surface (not yet read)
			</div>
			{nonZero.length === 0 ? (
				<div className="text-[10px] font-mono text-slate-500">no votes this tick</div>
			) : (
				<div className="flex flex-wrap gap-1 mb-1.5">
					{nonZero.map((entry) => (
						<span
							key={entry.index}
							className="text-[10px] font-mono px-1.5 py-0.5 rounded bg-slate-800/50 text-slate-300"
						>
							{voteSinkLabel(entry.index)}: {entry.value.toFixed(3)}
						</span>
					))}
				</div>
			)}
			<div className="flex flex-wrap gap-1">
				{COMMIT_KINDS.map((kind, index) => (
					<span key={kind} className="text-[10px] font-mono text-slate-500">
						{kind}: {tick.commit_counts[index] ?? 0}
					</span>
				))}
			</div>
		</div>
	);
});
