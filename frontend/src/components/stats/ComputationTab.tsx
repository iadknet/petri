import { useStatsHistoryStore } from "../../stores/stats.ts";
import { Gauge } from "./shared/Gauge.tsx";
import { MiniChart } from "./shared/MiniChart.tsx";

const COMPUTE_HINT =
	"Energy a creature's brain cost to run last tick: VM nodes + graph nodes + the hop ramp.";

export function ComputationTab() {
	const computeHistory = useStatsHistoryStore((s) => s.computeHistory);
	const latest = computeHistory[computeHistory.length - 1];

	const totalMean = latest?.totalMean ?? 0;
	const totalMin = latest?.totalMin ?? 0;
	const totalMax = latest?.totalMax ?? 0;
	const vmMean = latest?.vmMean ?? 0;
	const graphMean = latest?.graphMean ?? 0;

	const fmt = (v: number) => {
		if (v === 0) return "0";
		const abs = Math.abs(v);
		if (abs >= 0.01) return v.toFixed(4);
		if (abs >= 0.0001) return v.toFixed(6);
		return v.toExponential(2);
	};

	return (
		<div className="flex flex-col gap-3 p-3">
			<div className="flex flex-col gap-1">
				<span className="text-xs font-medium text-slate-300">
					Brain energy per creature, last tick
				</span>
				<div className="flex gap-4">
					<Gauge label="Mean" value={totalMean} format={fmt} hint={COMPUTE_HINT} />
					<Gauge label="Min" value={totalMin} format={fmt} hint={COMPUTE_HINT} />
					<Gauge label="Max" value={totalMax} format={fmt} hint={COMPUTE_HINT} />
				</div>
			</div>
			<div className="flex flex-col gap-1">
				<span className="text-xs font-medium text-slate-300">By node type</span>
				<div className="flex gap-4">
					<Gauge
						label="VM nodes"
						value={vmMean}
						format={fmt}
						hint="Energy spent running VM (instruction-list) nodes last tick, averaged over creatures that ran one."
					/>
					<Gauge
						label="Graph nodes"
						value={graphMean}
						format={fmt}
						hint="Energy spent running graph (wired-network) nodes last tick, averaged over creatures that ran one."
					/>
				</div>
			</div>
			<div>
				<span className="text-[11px] text-slate-400">Mean brain energy over time</span>
				<MiniChart
					data={computeHistory.map((c) => ({ tick: c.tick, value: c.totalMean }))}
					color="#a78bfa"
				/>
			</div>
		</div>
	);
}
