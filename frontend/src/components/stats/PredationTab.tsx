import { useStatsHistoryStore } from "../../stores/stats.ts";
import { Gauge } from "./shared/Gauge.tsx";
import { MiniChart } from "./shared/MiniChart.tsx";

export function PredationTab() {
	const actionsHistory = useStatsHistoryStore((s) => s.actionsHistory);
	const predationAttempted = useStatsHistoryStore((s) => s.predationAttempted);
	const predationTransferred = useStatsHistoryStore((s) => s.predationTransferred);
	const predationKills = useStatsHistoryStore((s) => s.predationKills);

	const successRate =
		predationAttempted > 0
			? ((predationTransferred / predationAttempted) * 100).toFixed(1)
			: "0.0";

	return (
		<div className="flex flex-col gap-3 p-3">
			<div>
				<span className="text-[11px] text-slate-400">Steal Actions / Tick</span>
				<MiniChart
					data={actionsHistory.map((a) => ({ tick: a.tick, value: a.actions.steal }))}
					color="#ef4444"
				/>
			</div>
			<div>
				<span className="text-[11px] text-slate-400">Kills / Tick</span>
				<MiniChart
					data={actionsHistory.map((a) => ({
						tick: a.tick,
						value: a.actions.predation_kills,
					}))}
					color="#f97316"
				/>
			</div>
			<div className="flex gap-4">
				<Gauge label="Attempted" value={predationAttempted} />
				<Gauge label="Transferred" value={predationTransferred} />
				<Gauge label="Kills" value={predationKills} />
				<Gauge
					label="Success Rate"
					value={Number(successRate)}
					format={(v) => `${v}%`}
				/>
			</div>
		</div>
	);
}
