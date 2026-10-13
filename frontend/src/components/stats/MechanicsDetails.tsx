import { useStatsHistoryStore } from "../../stores/stats.ts";
import { ActionBar } from "./shared/ActionBar.tsx";
import { deathCauseLabel, keyWords, mutationOperatorLabel, rejectionReason } from "./statLabels.ts";

const TOP = 6;

/** Whole counts, one format down every column. */
const count = (n: number) => n.toLocaleString("en-US");

interface Entry {
	key: string;
	label: string;
	hint?: string;
	count: number;
}

function entries(
	counts: Record<string, number>,
	label: (key: string) => { label: string; hint?: string },
): Entry[] {
	const sorted = Object.entries(counts)
		.filter(([, n]) => n > 0)
		.sort((a, b) => b[1] - a[1])
		.map(([key, count]) => ({ key, count, ...label(key) }));
	if (sorted.length <= TOP) return sorted;
	const rest = sorted.slice(TOP - 1);
	return [
		...sorted.slice(0, TOP - 1),
		{
			key: "other",
			label: `${rest.length} more`,
			count: rest.reduce((sum, e) => sum + e.count, 0),
		},
	];
}

/** A ranked count list: label, a proportion bar on a shared track, the count. */
function Breakdown({
	title,
	total,
	totalLabel,
	items,
	empty,
}: {
	title: string;
	total: number;
	totalLabel: string;
	items: Entry[];
	empty: string;
}) {
	const max = items.reduce((m, e) => Math.max(m, e.count), 0);
	return (
		<section aria-label={title} className="min-w-0">
			<h4 className="flex items-baseline justify-between gap-2 border-b border-petri-border pb-1 text-xs">
				<span className="font-semibold text-slate-200">{title}</span>
				<span className="font-mono text-[11px] text-slate-400 tabular-nums">
					{count(total)} {totalLabel}
				</span>
			</h4>
			{items.length === 0 ? (
				<p className="pt-1.5 text-[11px] text-slate-400">{empty}</p>
			) : (
				<ul className="pt-1">
					{items.map((item) => (
						<li
							key={item.key}
							className="grid grid-cols-[minmax(0,1fr)_3.5rem_3.25rem] items-center gap-2 py-[3px] text-[11px]"
							title={item.hint || undefined}
						>
							<span className={`truncate text-slate-300 ${item.hint ? "cursor-help" : ""}`}>
								{item.label}
							</span>
							<span aria-hidden="true" className="h-1.5 rounded-sm bg-slate-800">
								<span
									className="block h-full rounded-sm bg-slate-400"
									style={{ width: `${max > 0 ? (item.count / max) * 100 : 0}%` }}
								/>
							</span>
							<span className="text-right font-mono text-slate-200 tabular-nums">
								{count(item.count)}
							</span>
						</li>
					))}
				</ul>
			)}
		</section>
	);
}

/** Why things fail: the old Actions, Evolution and Predation detail, folded under the sheet. */
export function MechanicsDetails() {
	const lastActions = useStatsHistoryStore((s) => s.lastActions);
	const deathsByCause = useStatsHistoryStore((s) => s.deathsByCause);
	const reproRejected = useStatsHistoryStore((s) => s.reproRejected);
	const reproRejectedByReason = useStatsHistoryStore((s) => s.reproRejectedByReason);
	const moveBlocked = useStatsHistoryStore((s) => s.moveActionsBlockedByCause);
	const mutationSkipped = useStatsHistoryStore((s) => s.mutationSkipped);
	const mutationApplied = useStatsHistoryStore((s) => s.mutationApplied);
	const skippedByOperator = useStatsHistoryStore((s) => s.mutationSkippedByOperator);
	const reach = useStatsHistoryStore((s) => s.mutationTargetReachabilityTotal);
	const predationAttempted = useStatsHistoryStore((s) => s.predationAttempted);
	const predationByResult = useStatsHistoryStore((s) => s.predationByResult);
	const predationKills = useStatsHistoryStore((s) => s.predationKills);

	const deaths = Object.values(deathsByCause).reduce((a, b) => a + b, 0);
	const moveBlockedTotal = Object.values(moveBlocked).reduce((a, b) => a + b, 0);
	const reachTotal = reach.reachable + reach.unreachable + reach.notApplicable;
	const words = (key: string) => ({ label: keyWords(key) });

	return (
		<details className="group border-t border-petri-border">
			<summary className="flex cursor-pointer list-none items-baseline gap-2.5 px-3 py-2 text-[13px] hover:bg-slate-800/30 focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-sky-400 [&::-webkit-details-marker]:hidden">
				<svg
					aria-hidden="true"
					viewBox="0 0 8 8"
					className="h-2 w-2 shrink-0 text-slate-400 transition-transform duration-150 group-open:rotate-90"
				>
					<path d="M2 1l4 3-4 3" fill="none" stroke="currentColor" strokeWidth="1.5" />
				</svg>
				<span className="font-semibold text-slate-100">Mechanics</span>
				<span className="truncate text-slate-400">
					Actions, deaths by cause, failed births, mutation skips, predation
				</span>
			</summary>
			<div className="grid grid-cols-[repeat(auto-fill,minmax(15rem,1fr))] gap-x-6 gap-y-4 px-3 pt-1 pb-4">
				<section aria-label="Actions last tick" className="col-span-full min-w-0">
					<h4 className="border-b border-petri-border pb-1 text-xs font-semibold text-slate-200">
						Actions last tick
					</h4>
					<div className="pt-1.5">
						<ActionBar
							move={lastActions?.move ?? 0}
							eat={lastActions?.eat ?? 0}
							reproduce={lastActions?.reproduce ?? 0}
							steal={lastActions?.steal ?? 0}
							noop={lastActions?.noop ?? 0}
						/>
					</div>
				</section>
				<Breakdown
					title="Deaths by cause"
					total={deaths}
					totalLabel="died"
					items={entries(deathsByCause, (k) => ({ label: deathCauseLabel(k) }))}
					empty="No deaths yet."
				/>
				<Breakdown
					title="Failed births"
					total={reproRejected}
					totalLabel="rejected"
					items={entries(reproRejectedByReason, rejectionReason)}
					empty="No rejected births yet."
				/>
				<Breakdown
					title="Blocked moves"
					total={moveBlockedTotal}
					totalLabel="blocked"
					items={entries(moveBlocked, words)}
					empty="No blocked moves yet."
				/>
				<Breakdown
					title="Mutation skips by operator"
					total={mutationSkipped}
					totalLabel={`skipped, ${count(mutationApplied)} applied`}
					items={entries(skippedByOperator, (k) => ({ label: mutationOperatorLabel(k) }))}
					empty="No operator-level skips recorded yet."
				/>
				<Breakdown
					title="Mutation targets"
					total={reachTotal}
					totalLabel="targets"
					items={entries(
						{
							reachable: reach.reachable,
							unreachable: reach.unreachable,
							not_applicable: reach.notApplicable,
						},
						(k) => ({
							label:
								k === "not_applicable"
									? "Not applicable"
									: k === "reachable"
										? "Reachable"
										: "Unreachable",
							hint:
								k === "unreachable"
									? "The mutated node cannot affect any action: the mutation changes nothing."
									: undefined,
						}),
					)}
					empty="No mutation targets recorded yet."
				/>
				<Breakdown
					title="Predation"
					total={predationAttempted}
					totalLabel={`attempts, ${count(predationKills)} kills`}
					items={entries(predationByResult, words)}
					empty="No steals attempted yet."
				/>
			</div>
		</details>
	);
}
