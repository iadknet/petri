import { useStatsHistoryStore } from "../../stores/stats.ts";
import { Gauge } from "./shared/Gauge.tsx";
import { MiniChart } from "./shared/MiniChart.tsx";

export function ActionsTab() {
	const actionsHistory = useStatsHistoryStore((s) => s.actionsHistory);
	const reproAttempted = useStatsHistoryStore((s) => s.reproAttempted);
	const reproSpawned = useStatsHistoryStore((s) => s.reproSpawned);

	const successRate =
		reproAttempted > 0 ? ((reproSpawned / reproAttempted) * 100).toFixed(1) : "0.0";

	return (
		<div className="flex flex-col gap-3 p-3">
			<div>
				<span className="text-[11px] text-slate-400">Moves per tick</span>
				<MiniChart
					data={actionsHistory.map((a) => ({ tick: a.tick, value: a.actions.move }))}
					color="#3b82f6"
				/>
			</div>
			<div>
				<span className="text-[11px] text-slate-400">Eats per tick</span>
				<MiniChart
					data={actionsHistory.map((a) => ({ tick: a.tick, value: a.actions.eat }))}
					color="#10b981"
				/>
			</div>
			<div className="flex gap-4">
				<Gauge
					label="Reproduce tries"
					value={reproAttempted}
					hint="Reproduce actions attempted. Total since the run started."
				/>
				<Gauge
					label="Born"
					value={reproSpawned}
					hint="Children delivered (after gestation). Total since the run started."
				/>
				<Gauge
					label="Born per try"
					value={Number(successRate)}
					format={(v) => `${v}%`}
					hint="Born ÷ reproduce tries. Children still gestating count as not born yet."
				/>
			</div>
		</div>
	);
}
