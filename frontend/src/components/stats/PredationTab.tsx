import { useStatsHistoryStore } from "../../stores/stats.ts";
import { Gauge } from "./shared/Gauge.tsx";
import { MiniChart } from "./shared/MiniChart.tsx";

export function PredationTab() {
	const actionsHistory = useStatsHistoryStore((s) => s.actionsHistory);
	const predationAttempted = useStatsHistoryStore((s) => s.predationAttempted);
	const predationTransferred = useStatsHistoryStore((s) => s.predationTransferred);
	const predationKills = useStatsHistoryStore((s) => s.predationKills);

	const successRate =
		predationAttempted > 0 ? ((predationTransferred / predationAttempted) * 100).toFixed(1) : "0.0";

	return (
		<div className="flex flex-col gap-3 p-3">
			<div>
				<span className="text-[11px] text-slate-400">Steals per tick</span>
				<MiniChart
					data={actionsHistory.map((a) => ({ tick: a.tick, value: a.actions.steal }))}
					color="#ef4444"
				/>
			</div>
			<div>
				<span className="text-[11px] text-slate-400">Kills per tick</span>
				<MiniChart
					data={actionsHistory.map((a) => ({
						tick: a.tick,
						value: a.actions.predation_kills,
					}))}
					color="#f97316"
				/>
			</div>
			<div className="flex gap-4">
				<Gauge
					label="Steal tries"
					value={predationAttempted}
					hint="StealEnergy actions attempted. Total since the run started."
				/>
				<Gauge
					label="Landed"
					value={predationTransferred}
					hint="Steals that took energy from a victim, kills included. Total since the run started."
				/>
				<Gauge
					label="Kills"
					value={predationKills}
					hint="Steals that left the victim with no energy. Total since the run started."
				/>
				<Gauge
					label="Landed per try"
					value={Number(successRate)}
					format={(v) => `${v}%`}
					hint="Landed ÷ steal tries."
				/>
			</div>
		</div>
	);
}
