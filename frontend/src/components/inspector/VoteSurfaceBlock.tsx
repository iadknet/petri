import { memo } from "react";
import type { VoteKind } from "../../types/genome.ts";
import type { MeshPassTrace, TickTrace, WorldAction } from "../../types/trace.ts";
import { VOTE_KINDS, VOTE_SINK_COUNT } from "../../types/trace.ts";
import { voteSinkFromIndex, voteSinkLabel } from "./graphNodeFormatters.ts";
import { formatAction } from "./inputRefUtils.ts";

interface VoteSurfaceBlockProps {
	tick: TickTrace;
}

/** The vote catalog, by index. */
const SINKS = Array.from({ length: VOTE_SINK_COUNT }, (_, index) => voteSinkFromIndex(index));
/** Catalog indices of each kind's sinks, in `VOTE_KINDS` order. */
const KIND_SINKS: readonly (readonly number[])[] = VOTE_KINDS.map((kind) =>
	SINKS.flatMap((sink, index) =>
		sink === kind || (sink !== null && typeof sink === "object" && kind in sink) ? [index] : [],
	),
);
const TERMINATE_SINK = SINKS.indexOf("Terminate");
const DECIDE_SINK = SINKS.indexOf("Decide");

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

/** The largest of recorded values; zero when there are none. */
function maxRecorded(values: readonly number[]): number {
	return values.length === 0 ? 0 : Math.max(...values);
}

/** A kind's best sink vote: the maximum of its recorded sink votes. */
function bestSinkVote(votes: readonly number[], sinks: readonly number[]): number {
	return maxRecorded(sinks.map((sink) => votes[sink] ?? 0));
}

/**
 * Per kind: the best sink vote, the bar the pass started with, and the
 * recorded effective vote the commit rule compared; the committed kind is
 * highlighted. Kinds with no vote and no bar are omitted.
 */
function KindRows({ pass, bars }: { pass: MeshPassTrace; bars: readonly number[] }) {
	const committedKind = actionKind(pass.committed);
	const shown = VOTE_KINDS.map((kind, index) => ({
		kind,
		bar: bars[index] ?? 0,
		best: bestSinkVote(pass.votes, KIND_SINKS[index] ?? []),
		effective: pass.effective_votes[index] ?? 0,
	})).filter((entry) => entry.best !== 0 || entry.effective !== 0);
	if (shown.length === 0) return null;
	return (
		<div className="flex flex-wrap gap-1 mt-0.5">
			<span className="text-[10px] font-mono text-slate-500">effective</span>
			{shown.map(({ kind, bar, best, effective }) => (
				<span
					key={kind}
					data-testid={`effective-${pass.pass_index}-${kind}`}
					className={`text-[10px] font-mono ${kind === committedKind ? "text-orange-300" : "text-slate-400"}`}
				>
					{kind} {best.toFixed(3)} − {bar} = {effective.toFixed(3)}
				</span>
			))}
		</div>
	);
}

interface PassRowProps {
	pass: MeshPassTrace;
	bars: readonly number[];
	endsTerminateVotedTick: boolean;
}

/**
 * One pass: its end reason and commit, the non-zero kind sink votes, each
 * kind's effective vote against its bar, and `Terminate` and `Decide` as
 * their own fields. The pass that ends a `TerminateVoted` tick shows
 * `Terminate` against the winning effective vote; a `Decided` pass names
 * the `Decide` vote that ended it.
 */
function PassRow({ pass, bars, endsTerminateVotedTick }: PassRowProps) {
	const kindVotes = pass.votes
		.map((value, index) => ({ value, index }))
		.filter((entry) => entry.index < TERMINATE_SINK && entry.value !== 0);
	const terminate = pass.votes[TERMINATE_SINK] ?? 0;
	const decide = pass.votes[DECIDE_SINK] ?? 0;
	return (
		<div className="mb-1.5" data-testid={`pass-detail-${pass.pass_index}`}>
			<div className="text-[10px] font-mono text-slate-400">
				Pass {pass.pass_index} · {pass.end_reason} · {pass.hops} hops ·{" "}
				<span className={pass.committed ? "text-orange-300" : "text-slate-500"}>
					{pass.committed ? `commit ${formatAction(pass.committed)}` : "no commit"}
				</span>
			</div>
			{kindVotes.length === 0 ? (
				<div className="text-[10px] font-mono text-slate-500">no kind votes this pass</div>
			) : (
				<div className="flex flex-wrap gap-1">
					{kindVotes.map((entry) => (
						<span
							key={entry.index}
							className="text-[10px] font-mono px-1.5 py-0.5 rounded bg-slate-800/50 text-slate-300"
						>
							{voteSinkLabel(entry.index)}: {entry.value.toFixed(3)}
						</span>
					))}
				</div>
			)}
			<KindRows pass={pass} bars={bars} />
			<div className="flex flex-wrap gap-2 mt-0.5 text-[10px] font-mono text-slate-400">
				<span data-testid={`terminate-${pass.pass_index}`}>Terminate {terminate.toFixed(3)}</span>
				<span data-testid={`decide-${pass.pass_index}`}>Decide {decide.toFixed(3)}</span>
				{endsTerminateVotedTick ? (
					<span data-testid="terminate-verdict" className="text-orange-300">
						Terminate {terminate.toFixed(3)} ≥ winning effective{" "}
						{maxRecorded(pass.effective_votes).toFixed(3)}
					</span>
				) : null}
				{pass.end_reason === "Decided" ? (
					<span data-testid={`decided-${pass.pass_index}`} className="text-orange-300">
						ended by Decide {decide.toFixed(3)}
					</span>
				) : null}
			</div>
		</div>
	);
}

/**
 * The sampled tick's action selection: one row per recorded pass with the
 * votes it summed, each kind's recorded effective vote against its bar,
 * `Terminate` and `Decide`, why it ended, and the action it committed, then
 * the tick's end reason and the final per-kind bars. Every number shown is a
 * recorded value; the view only counts, groups, and selects.
 */
export const VoteSurfaceBlock = memo(function VoteSurfaceBlock({ tick }: VoteSurfaceBlockProps) {
	const bars = barsPerPass(tick.passes);
	const lastPass = tick.passes.length - 1;
	return (
		<div className="px-3 py-2 border-t border-white/[0.06]">
			<div className="text-xs text-slate-500 uppercase tracking-wider font-medium mb-1.5">
				Action Selection · {tick.termination_reason}
			</div>
			{tick.passes.length === 0 ? (
				<div className="text-[10px] font-mono text-slate-500 mb-1.5">no passes this tick</div>
			) : (
				tick.passes.map((pass, index) => (
					<PassRow
						key={pass.pass_index}
						pass={pass}
						bars={bars[index] ?? []}
						endsTerminateVotedTick={
							index === lastPass && tick.termination_reason === "TerminateVoted"
						}
					/>
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
