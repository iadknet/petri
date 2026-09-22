import { memo } from "react";
import type { MeshPassTrace, TickTrace } from "../../types/trace.ts";
import { VOTE_KINDS } from "../../types/trace.ts";
import { voteSinkLabel } from "./graphNodeFormatters.ts";
import { formatAction } from "./inputRefUtils.ts";

interface VoteSurfaceBlockProps {
	tick: TickTrace;
}

/** One pass: its end reason, the action it committed, and its non-zero votes. */
function PassRow({ pass }: { pass: MeshPassTrace }) {
	const nonZero = pass.votes
		.map((value, index) => ({ value, index }))
		.filter((entry) => entry.value !== 0);
	return (
		<div className="mb-1.5">
			<div className="text-[10px] font-mono text-slate-400">
				Pass {pass.pass_index} · {pass.end_reason} · {pass.hops} hops ·{" "}
				<span className={pass.committed ? "text-orange-300" : "text-slate-500"}>
					{pass.committed ? `commit ${formatAction(pass.committed)}` : "no commit"}
				</span>
			</div>
			{nonZero.length === 0 ? (
				<div className="text-[10px] font-mono text-slate-500">no votes this pass</div>
			) : (
				<div className="flex flex-wrap gap-1">
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
		</div>
	);
}

/**
 * The sampled tick's action selection: one row per pass with the votes it
 * summed, why it ended, and the action it committed, then the tick's end
 * reason and the final per-kind bars (commits of each kind this tick).
 */
export const VoteSurfaceBlock = memo(function VoteSurfaceBlock({ tick }: VoteSurfaceBlockProps) {
	return (
		<div className="px-3 py-2 border-t border-white/[0.06]">
			<div className="text-xs text-slate-500 uppercase tracking-wider font-medium mb-1.5">
				Action Selection · {tick.termination_reason}
			</div>
			{tick.passes.length === 0 ? (
				<div className="text-[10px] font-mono text-slate-500 mb-1.5">no passes this tick</div>
			) : (
				tick.passes.map((pass) => <PassRow key={pass.pass_index} pass={pass} />)
			)}
			<div className="flex flex-wrap gap-1">
				{VOTE_KINDS.map((kind, index) => (
					<span key={kind} className="text-[10px] font-mono text-slate-500">
						{kind}: {tick.commit_counts[index] ?? 0}
					</span>
				))}
			</div>
		</div>
	);
});
