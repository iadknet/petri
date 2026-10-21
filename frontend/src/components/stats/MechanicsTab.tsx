import { useEffect, useRef, useState } from "react";
import { useStatsHistoryStore } from "../../stores/stats.ts";
import { ActionBar } from "./shared/ActionBar.tsx";
import { deathCauseLabel, keyWords, mutationOperatorLabel, rejectionReason } from "./statLabels.ts";

const ROW_PX = 18;

/** Whole counts, one format down every column. */
const count = (n: number) => n.toLocaleString("en-US");

/** Rows a list box of `height` px holds, at least one; every row until the box is measured. */
export function rowsThatFit(height: number, rows: number): number {
	if (height <= 0) return rows;
	return Math.max(1, Math.min(rows, Math.floor(height / ROW_PX)));
}

interface Entry {
	key: string;
	label: string;
	hint?: string;
	count: number;
}

function ranked(
	counts: Record<string, number>,
	label: (key: string) => { label: string; hint?: string },
): Entry[] {
	return Object.entries(counts)
		.filter(([, n]) => n > 0)
		.sort((a, b) => b[1] - a[1])
		.map(([key, n]) => ({ key, count: n, ...label(key) }));
}

/** Keep the top `limit - 1` and fold the rest into one "N more" row; one row shows the top entry. */
export function fold(items: Entry[], limit: number): Entry[] {
	if (items.length <= limit) return items;
	if (limit <= 1) return items.slice(0, 1);
	const rest = items.slice(limit - 1);
	return [
		...items.slice(0, limit - 1),
		{
			key: "other",
			label: `${rest.length} more`,
			count: rest.reduce((sum, e) => sum + e.count, 0),
		},
	];
}

function useBoxHeight<T extends HTMLElement>() {
	const ref = useRef<T>(null);
	const [height, setHeight] = useState(0);
	useEffect(() => {
		const el = ref.current;
		if (!el || typeof ResizeObserver === "undefined") return;
		const ro = new ResizeObserver((entries) => {
			const entry = entries[0];
			if (entry) setHeight(entry.contentRect.height);
		});
		ro.observe(el);
		return () => ro.disconnect();
	}, []);
	return [ref, height] as const;
}

function Cell({
	title,
	summary,
	children,
}: {
	title: string;
	summary?: string;
	children: React.ReactNode;
}) {
	return (
		<section aria-label={title} className="flex min-h-0 min-w-0 flex-col overflow-hidden">
			<h3 className="flex items-baseline justify-between gap-2 border-b border-petri-border pb-1 text-xs">
				<span className="truncate font-semibold text-slate-200">{title}</span>
				{summary && (
					<span className="shrink-0 font-mono text-[11px] text-slate-400 tabular-nums">
						{summary}
					</span>
				)}
			</h3>
			{children}
		</section>
	);
}

/** A ranked count list that shows as many rows as its cell holds, folding the rest. */
function Breakdown({
	title,
	summary,
	items,
	empty,
}: {
	title: string;
	summary: string;
	items: Entry[];
	empty: string;
}) {
	const [listRef, height] = useBoxHeight<HTMLDivElement>();
	const shown = fold(items, rowsThatFit(height, items.length));
	const max = shown.reduce((m, e) => Math.max(m, e.count), 0);
	return (
		<Cell title={title} summary={summary}>
			<div ref={listRef} className="min-h-0 flex-1 pt-0.5">
				{items.length === 0 ? (
					<p className="text-[11px] text-slate-400">{empty}</p>
				) : (
					<ul>
						{shown.map((item) => (
							<li
								key={item.key}
								className="grid h-[18px] grid-cols-[minmax(0,1fr)_3rem_4rem] items-center gap-2 text-[11px]"
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
			</div>
		</Cell>
	);
}

/** Why things fail: actions, deaths, failed births, blocked moves, mutation skips, predation. */
export function MechanicsTab() {
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
		<div className="grid h-full auto-rows-fr grid-cols-[repeat(auto-fit,minmax(13rem,1fr))] gap-x-6 gap-y-2 overflow-hidden px-3 py-2">
			<Cell title="Actions last tick">
				<div className="pt-1.5">
					<ActionBar
						move={lastActions?.move ?? 0}
						eat={lastActions?.eat ?? 0}
						reproduce={lastActions?.reproduce ?? 0}
						steal={lastActions?.steal ?? 0}
						give={lastActions?.give ?? 0}
						noop={lastActions?.noop ?? 0}
					/>
				</div>
			</Cell>
			<Breakdown
				title="Deaths by cause"
				summary={`${count(deaths)} died`}
				items={ranked(deathsByCause, (k) => ({ label: deathCauseLabel(k) }))}
				empty="No deaths yet."
			/>
			<Breakdown
				title="Failed births"
				summary={`${count(reproRejected)} rejected`}
				items={ranked(reproRejectedByReason, rejectionReason)}
				empty="No rejected births yet."
			/>
			<Breakdown
				title="Blocked moves"
				summary={`${count(moveBlockedTotal)} blocked`}
				items={ranked(moveBlocked, words)}
				empty="No blocked moves yet."
			/>
			<Breakdown
				title="Mutation skips"
				summary={`${count(mutationSkipped)} of ${count(mutationSkipped + mutationApplied)}`}
				items={ranked(skippedByOperator, (k) => ({ label: mutationOperatorLabel(k) }))}
				empty="No operator-level skips recorded yet."
			/>
			<Breakdown
				title="Mutation targets"
				summary={`${count(reachTotal)} targets`}
				items={ranked(
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
				summary={`${count(predationAttempted)} tries, ${count(predationKills)} kills`}
				items={ranked(predationByResult, words)}
				empty="No steals attempted yet."
			/>
		</div>
	);
}
