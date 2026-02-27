import { memo } from "react";
import type { TickTrace } from "../../types/api.ts";
import { formatAction } from "./inputRefUtils.ts";

interface TickTimelineProps {
	ticks: TickTrace[];
	activeTickIndex: number;
	onTickSelect: (index: number) => void;
}

export const TickTimeline = memo(function TickTimeline({
	ticks,
	activeTickIndex,
	onTickSelect,
}: TickTimelineProps) {
	return (
		<div className="px-3 py-2">
			<div className="text-xs text-slate-500 uppercase tracking-wider font-medium mb-1.5">
				Tick Timeline
			</div>
			<div className="flex gap-1 overflow-x-auto pb-1">
				{ticks.map((tick, i) => {
					const delta = tick.energy_after - tick.energy_before;
					const isActive = i === activeTickIndex;

					return (
						<button
							type="button"
							key={tick.tick_number}
							onClick={() => onTickSelect(i)}
							className={`flex-shrink-0 px-2 py-1 rounded text-[10px] font-mono transition-colors ${
								isActive
									? "bg-white/[0.04] ring-1 ring-white/10 text-slate-200"
									: "bg-slate-800/50 hover:bg-slate-700/50 text-slate-400"
							}`}
						>
							<div className="font-medium">T{tick.tick_number}</div>
							<div className="text-[9px]">{formatAction(tick.final_action)}</div>
							<div className={`text-[9px] ${delta >= 0 ? "text-emerald-400" : "text-red-400"}`}>
								{delta >= 0 ? "+" : ""}
								{delta.toFixed(1)}e
							</div>
						</button>
					);
				})}
			</div>
		</div>
	);
});
