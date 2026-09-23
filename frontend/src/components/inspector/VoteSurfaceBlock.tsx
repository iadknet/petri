import { memo } from "react";
import type { VoteKind } from "../../types/genome.ts";
import type { MeshPassTrace, TickTrace, WorldAction } from "../../types/trace.ts";
import { VOTE_KINDS } from "../../types/trace.ts";
import { voteSinkLabel } from "./graphNodeFormatters.ts";
import { formatAction } from "./inputRefUtils.ts";

interface VoteSurfaceBlockProps {
	tick: TickTrace;
}

/** The vote kind of a committed action; `NoOp` has none. */
function actionKind(action: WorldAction | null): VoteKind | null {
	if (action === null || action === "NoOp") return null;
	if (action === "Eat") return "Eat";
	return Object.keys(action)[0] as VoteKind;
}

/** Each kind's bar at the start of every pass: its commits in earlier passes. */
function barsPerPass(passes: readonly MeshPassTrace[]): number[][] {
	let bars = VOTE_KINDS.map(() => 0);
	return passes.map((pass) => {
		const start = bars;
		const kind = actionKind(pass.committed);
		bars = bars.map((bar, index) => (VOTE_KINDS[index] === kind ? bar + 1 : bar));
		return start;
	});
}

/**
 * The pass-end commit reading: each kind's best vote minus its bar, the
 * effective vote the commit rule compares. Kinds reading zero are omitted;
 * the committed kind is highlighted.
 */
function EffectiveRow({ pass, bars }: { pass: MeshPassTrace; bars: readonly number[] }) {
	const committedKind = actionKind(pass.committed);
	const shown = VOTE_KINDS.map((kind, index) => ({
		kind,
		bar: bars[index] ?? 0,
		effective: pass.effective_votes[index] ?? 0,
	})).filter((entry) => entry.effective !== 0);
	if (shown.length === 0) return null;
	return (
		<div className="flex flex-wrap gap-1 mt-0.5">
			<span className="text-[10px] font-mono text-slate-500">effective</span>
			{shown.map(({ kind, bar, effective }) => (
				<span
					key={kind}
					data-testid={`effective-${pass.pass_index}-${kind}`}
					className={`text-[10px] font-mono ${kind === committedKind ? "text-orange-300" : "text-slate-400"}`}
				>
					{kind} {(effective + bar).toFixed(3)} − {bar} = {effective.toFixed(3)}
				</span>
			))}
		</div>
	);
}

/**
 * One pass: its end reason, the action it committed, its non-zero votes, and
 * each kind's effective vote against the bar, which is what the commit reads.
 */
function PassRow({ pass, bars }: { pass: MeshPassTrace; bars: readonly number[] }) {
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
			<EffectiveRow pass={pass} bars={bars} />
		</div>
	);
}

/**
 * The sampled tick's action selection: one row per pass with the votes it
 * summed, each kind's effective vote against its bar, why it ended, and the
 * action it committed, then the tick's end
 * reason and the final per-kind bars (commits of each kind this tick).
 */
export const VoteSurfaceBlock = memo(function VoteSurfaceBlock({ tick }: VoteSurfaceBlockProps) {
	const bars = barsPerPass(tick.passes);
	return (
		<div className="px-3 py-2 border-t border-white/[0.06]">
			<div className="text-xs text-slate-500 uppercase tracking-wider font-medium mb-1.5">
				Action Selection · {tick.termination_reason}
			</div>
			{tick.passes.length === 0 ? (
				<div className="text-[10px] font-mono text-slate-500 mb-1.5">no passes this tick</div>
			) : (
				tick.passes.map((pass, index) => (
					<PassRow key={pass.pass_index} pass={pass} bars={bars[index] ?? []} />
				))
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
