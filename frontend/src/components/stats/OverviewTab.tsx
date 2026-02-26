import { useSimulationStore } from "../../stores/simulation.ts";
import { useStatsHistoryStore } from "../../stores/stats.ts";
import { ActionBar } from "./shared/ActionBar.tsx";
import { Gauge } from "./shared/Gauge.tsx";
import { MiniChart } from "./shared/MiniChart.tsx";

export function OverviewTab() {
	const status = useSimulationStore((s) => s.status);
	const statsHistory = useStatsHistoryStore((s) => s.statsHistory);

	const pop = status?.population ?? 0;
	const energy = status?.mean_energy ?? 0;
	const actions = status?.last_tick_actions ?? { move: 0, eat: 0, reproduce: 0, noop: 0 };

	// Delta from 10 ticks ago
	const delta =
		statsHistory.length >= 10
			? pop - (statsHistory[statsHistory.length - 10]?.population ?? pop)
			: undefined;

	return (
		<div className="flex flex-col gap-3 p-3">
			<div className="flex gap-6">
				<Gauge label="Population" value={pop} delta={delta} />
				<Gauge label="Mean Energy" value={energy} format={(v) => v.toFixed(1)} />
			</div>
			<ActionBar {...actions} />
			<div>
				<span className="text-[11px] text-slate-400">Population Trend</span>
				<MiniChart
					data={statsHistory.map((s) => ({ tick: s.tick, value: s.population }))}
					color="#10b981"
				/>
			</div>
			<div>
				<span className="text-[11px] text-slate-400">Energy Trend</span>
				<MiniChart
					data={statsHistory.map((s) => ({ tick: s.tick, value: s.meanEnergy }))}
					color="#3b82f6"
				/>
			</div>
		</div>
	);
}
