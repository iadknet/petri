import { useStatsHistoryStore } from "../../stores/stats.ts";
import { Gauge } from "./shared/Gauge.tsx";
import { MiniChart } from "./shared/MiniChart.tsx";
import { mutationOperatorLabel, rejectionReason } from "./statLabels.ts";

export function EvolutionTab() {
	const reproAttempted = useStatsHistoryStore((s) => s.reproAttempted);
	const reproSpawned = useStatsHistoryStore((s) => s.reproSpawned);
	const reproRejected = useStatsHistoryStore((s) => s.reproRejected);
	const reproRejectedByReason = useStatsHistoryStore((s) => s.reproRejectedByReason);
	const mutationAttempted = useStatsHistoryStore((s) => s.mutationAttempted);
	const mutationApplied = useStatsHistoryStore((s) => s.mutationApplied);
	const mutationSkipped = useStatsHistoryStore((s) => s.mutationSkipped);
	const mutationSkippedByOperator = useStatsHistoryStore((s) => s.mutationSkippedByOperator);
	const mutationTargetReachabilityTotal = useStatsHistoryStore(
		(s) => s.mutationTargetReachabilityTotal,
	);
	const complexityHistory = useStatsHistoryStore((s) => s.complexityHistory);

	const latestComplexity = complexityHistory[complexityHistory.length - 1];
	const complexityMean = latestComplexity?.mean ?? 0;
	const complexityMin = latestComplexity?.min ?? 0;
	const complexityMax = latestComplexity?.max ?? 0;
	const skipByOperatorEntries = Object.entries(mutationSkippedByOperator).sort(
		(a, b) => b[1] - a[1],
	);
	const targetReachabilityTotal =
		mutationTargetReachabilityTotal.reachable +
		mutationTargetReachabilityTotal.unreachable +
		mutationTargetReachabilityTotal.notApplicable;
	const targetReachabilityPercent = (count: number) =>
		targetReachabilityTotal > 0 ? (count / targetReachabilityTotal) * 100 : 0;

	return (
		<div className="flex flex-col gap-3 p-3">
			<div className="flex flex-col gap-1">
				<span className="text-xs font-medium text-slate-300">Reproduction</span>
				<div className="flex gap-4">
					<Gauge
						label="Tries"
						value={reproAttempted}
						hint="Reproduce actions attempted. Total since the run started."
					/>
					<Gauge
						label="Born"
						value={reproSpawned}
						hint="Children delivered (after gestation). Total since the run started."
					/>
					<Gauge
						label="Rejected"
						value={reproRejected}
						hint="Tries that failed a reproduction gate; reasons below. Total since the run started."
					/>
				</div>
				{Object.keys(reproRejectedByReason).length > 0 && (
					<div className="mt-1 pl-2 border-l border-slate-700">
						<span className="text-[11px] text-slate-500">Why tries were rejected</span>
						{Object.entries(reproRejectedByReason).map(([reason, count]) => (
							<div key={reason} className="flex justify-between text-[11px]">
								<span className="text-slate-400" title={rejectionReason(reason).hint || undefined}>
									{rejectionReason(reason).label}
								</span>
								<span className="font-mono text-slate-300">{count.toLocaleString()}</span>
							</div>
						))}
					</div>
				)}
			</div>

			<div className="flex flex-col gap-1">
				<span className="text-xs font-medium text-slate-300">Mutations</span>
				<div className="flex gap-4">
					<Gauge
						label="Tried"
						value={mutationAttempted}
						hint="Mutation events drawn at births. Total since the run started."
					/>
					<Gauge
						label="Applied"
						value={mutationApplied}
						hint="Mutation events that changed the child's genome. Total since the run started."
					/>
					<Gauge
						label="Skipped"
						value={mutationSkipped}
						hint="Mutation events that found no valid target or whose change was rejected. Total since the run started."
					/>
				</div>
				<div className="mt-1 pl-2 border-l border-slate-700">
					<span className="text-[11px] text-slate-500">Skipped, by mutation type</span>
					{skipByOperatorEntries.length === 0 && (
						<div className="text-[11px] text-slate-500">No operator-level skips recorded yet.</div>
					)}
					{skipByOperatorEntries.map(([operator, count]) => (
						<div key={operator} className="flex justify-between text-[11px]">
							<span className="text-slate-400" title={operator}>
								{mutationOperatorLabel(operator)}
							</span>
							<span className="font-mono text-slate-300">{count.toLocaleString()}</span>
						</div>
					))}
				</div>
				<div className="mt-1 pl-2 border-l border-slate-700">
					<span
						className="text-[11px] text-slate-500 cursor-help"
						title="Whether each mutation hit a mesh node the brain can reach from its entry node. Unreachable nodes never run, so changes there do nothing until rewired."
					>
						Target Reachability
					</span>
					<div className="flex gap-4">
						<Gauge label="Reachable" value={mutationTargetReachabilityTotal.reachable} />
						<Gauge label="Unreachable" value={mutationTargetReachabilityTotal.unreachable} />
						<Gauge
							label="Not Applicable"
							value={mutationTargetReachabilityTotal.notApplicable}
							hint="Mutations that do not target a single mesh node."
						/>
					</div>
					<div className="flex justify-between text-[11px] text-slate-400">
						<span>
							Reachable{" "}
							{targetReachabilityPercent(mutationTargetReachabilityTotal.reachable).toFixed(1)}%
						</span>
						<span>
							Unreachable{" "}
							{targetReachabilityPercent(mutationTargetReachabilityTotal.unreachable).toFixed(1)}%
						</span>
						<span>
							N/A{" "}
							{targetReachabilityPercent(mutationTargetReachabilityTotal.notApplicable).toFixed(1)}%
						</span>
					</div>
				</div>
			</div>

			<div className="flex flex-col gap-1">
				<span
					className="text-xs font-medium text-slate-300 cursor-help"
					title="Functional complexity: counts only parts that can affect behaviour (reachable nodes, wired components, used inputs). Junk is not counted. The complexity energy cost uses it."
				>
					Genome Complexity
				</span>
				<div className="flex gap-4">
					<Gauge label="Mean" value={complexityMean} format={(v) => v.toFixed(1)} />
					<Gauge label="Min" value={complexityMin} />
					<Gauge label="Max" value={complexityMax} />
				</div>
				<div>
					<span className="text-[11px] text-slate-400">Mean complexity over time</span>
					<MiniChart
						data={complexityHistory.map((c) => ({ tick: c.tick, value: c.mean }))}
						color="#f59e0b"
					/>
				</div>
			</div>
		</div>
	);
}
