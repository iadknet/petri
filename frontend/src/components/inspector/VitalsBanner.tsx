import { memo, useMemo, useState } from "react";
import { ActionResult } from "../../types/action-log.ts";
import type { ActionLogEntry } from "../../types/action-log.ts";
import type { CreaturePhenotype } from "../../types/genome.ts";
import { PhenotypeDetail } from "./PhenotypeDetail.tsx";

const ACTION_COLORS: Record<string, string> = {
	NoOp: "#475569",
	Eat: "#34d399",
	Move: "#60a5fa",
	Reproduce: "#fbbf24",
	StealEnergy: "#f87171",
};

const MAX_ACTION_DOTS = 50;
const COL_W = 7; // 6px dot + 1px gap

const DIRECTION_LABELS: Record<number, string> = {
	0: "N",
	1: "NE",
	2: "E",
	3: "SE",
	4: "S",
	5: "SW",
	6: "W",
	7: "NW",
};
const BRACKET_H = 8;

interface TickBracket {
	startIdx: number;
	count: number;
}

function findTickBrackets(actions: ActionLogEntry[]): TickBracket[] {
	const brackets: TickBracket[] = [];
	let i = 0;
	while (i < actions.length) {
		let j = i + 1;
		while (j < actions.length && actions[j]!.tick === actions[i]!.tick) j++;
		if (j - i >= 2) brackets.push({ startIdx: i, count: j - i });
		i = j;
	}
	return brackets;
}

interface VitalsBannerProps {
	id: number;
	rgb: [number, number, number];
	generation: number;
	age: number;
	energy: number;
	maxEnergy: number;
	reproductiveReserve: number;
	reproductiveReserveCapacity: number;
	position: { x: number; y: number };
	actionLog: ActionLogEntry[] | null;
	phenotype?: CreaturePhenotype;
	isDead: boolean;
	onClose: () => void;
}

export const VitalsBanner = memo(function VitalsBanner({
	id,
	rgb,
	generation,
	age,
	energy,
	maxEnergy,
	reproductiveReserve,
	reproductiveReserveCapacity,
	position,
	actionLog,
	phenotype,
	isDead,
	onClose,
}: VitalsBannerProps) {
	const [drawerOpen, setDrawerOpen] = useState(false);
	const [hoveredIdx, setHoveredIdx] = useState<number | null>(null);

	const energyRatio = maxEnergy > 0 ? energy / maxEnergy : 0;
	const energyBarColor = energyRatio > 0.3 ? "bg-emerald-500" : "bg-red-500";
	const energyWidthPercent = Math.min(energyRatio * 100, 100);
	const reserveRatio =
		reproductiveReserveCapacity > 0 ? reproductiveReserve / reproductiveReserveCapacity : 0;
	const reserveWidthPercent = Math.min(Math.max(reserveRatio, 0) * 100, 100);

	const visibleActions = actionLog ? actionLog.slice(-MAX_ACTION_DOTS) : [];

	const tickBrackets = useMemo(() => findTickBrackets(visibleActions), [visibleActions]);

	const timelineWidth = visibleActions.length * COL_W;

	return (
		<div className="bg-slate-950 border-b border-slate-800">
			{/* Dead banner */}
			{isDead && (
				<div className="bg-red-900/60 border-b border-red-700/50 px-3 py-1 text-xs text-red-300 font-medium">
					DEAD
				</div>
			)}

			{/* Row 1: Vitals */}
			<div className="flex items-center gap-2 px-3 py-1.5 text-xs">
				{/* Color swatch */}
				<div
					data-testid="color-swatch"
					className="w-2 h-2 rounded-full shrink-0"
					style={{ backgroundColor: `rgb(${rgb[0]}, ${rgb[1]}, ${rgb[2]})` }}
				/>

				{/* ID */}
				<span className="font-mono text-slate-50 font-medium">#{id}</span>

				{/* Gen / Age */}
				<span className="text-slate-400">Gen {generation}</span>
				<span className="text-slate-400">Age {age}</span>

				{/* Energy bar */}
				<div className="flex items-center gap-1.5 min-w-0">
					<div className="w-16 h-1.5 rounded-full bg-slate-800 overflow-hidden">
						<div
							data-testid="energy-bar-fill"
							className={`h-full rounded-full ${energyBarColor}`}
							style={{ width: `${energyWidthPercent}%` }}
						/>
					</div>
					<span className="font-mono text-slate-400 whitespace-nowrap">
						{Math.round(energy)}/{maxEnergy}
					</span>
				</div>

				{/* Applied reproductive reserve */}
				<div className="flex items-center gap-1.5 min-w-0" title="Current reproductive reserve">
					<div className="w-16 h-1.5 rounded-full bg-slate-800 overflow-hidden">
						<div
							data-testid="reserve-bar-fill"
							className="h-full rounded-full bg-amber-400"
							style={{ width: `${reserveWidthPercent}%` }}
						/>
					</div>
					<span className="font-mono text-slate-400 whitespace-nowrap">
						Reserve {reproductiveReserve.toFixed(1)}/{reproductiveReserveCapacity.toFixed(1)}
					</span>
				</div>

				{/* Position */}
				<span className="font-mono text-slate-400">
					({position.x}, {position.y})
				</span>

				{/* Close button */}
				<button
					type="button"
					onClick={onClose}
					aria-label="Close"
					className="ml-auto shrink-0 text-slate-500 hover:text-slate-300 transition-colors px-1"
				>
					[X]
				</button>
			</div>

			{/* Row 2: Action timeline with brackets, dots, and sparkline */}
			{visibleActions.length > 0 && (
				<div className="px-3 pb-1.5" style={{ width: timelineWidth + 24 }}>
					{/* Tick brackets for multi-action ticks */}
					{tickBrackets.length > 0 && (
						<svg width={timelineWidth} height={BRACKET_H} className="block">
							<title>Grouped action ticks</title>
							{tickBrackets.map((b) => {
								const x1 = b.startIdx * COL_W + 1;
								const x2 = (b.startIdx + b.count - 1) * COL_W + 5;
								return (
									<path
										key={b.startIdx}
										d={`M ${x1} ${BRACKET_H} L ${x1} 2 L ${x2} 2 L ${x2} ${BRACKET_H}`}
										fill="none"
										stroke="rgba(251,191,36,0.5)"
										strokeWidth={1}
									/>
								);
							})}
						</svg>
					)}

					{/* Action dots */}
					<div className="relative flex items-center gap-px">
						{visibleActions.map((entry, i) => {
							const failed = entry.result !== ActionResult.Success;
							return (
								<div
									key={`${entry.tick}-${i}`}
									data-testid="action-dot"
									className="rounded-full shrink-0"
									style={{
										width: 6,
										height: 6,
										backgroundColor: ACTION_COLORS[entry.action_type] ?? ACTION_COLORS.NoOp,
										boxShadow: failed ? "0 0 0 1.5px #ef4444" : undefined,
									}}
									onMouseEnter={() => setHoveredIdx(i)}
									onMouseLeave={() => setHoveredIdx(null)}
								/>
							);
						})}

						{/* Hover tooltip */}
						{hoveredIdx !== null &&
							visibleActions[hoveredIdx] &&
							(() => {
								const entry = visibleActions[hoveredIdx]!;
								const failed = entry.result !== ActionResult.Success;
								const energyDelta = entry.energy_after - entry.energy_before;
								const dir = DIRECTION_LABELS[entry.direction];
								// Position tooltip above the dot, clamped to not overflow left
								const left = Math.max(0, hoveredIdx * COL_W - 40);
								return (
									<div
										className="absolute top-full mt-1.5 z-50 pointer-events-none bg-slate-900 border border-slate-700 rounded px-2 py-1.5 text-[10px] font-mono leading-relaxed text-slate-300 whitespace-nowrap shadow-lg"
										style={{ left }}
									>
										<div className="text-slate-100 font-medium mb-0.5">
											T{entry.tick} — {entry.action_type}
										</div>
										<div>
											Result:{" "}
											<span className={failed ? "text-red-400" : "text-emerald-400"}>
												{entry.result}
											</span>
										</div>
										{dir && <div>Direction: {dir}</div>}
										<div>
											Energy: {Math.round(entry.energy_before)} → {Math.round(entry.energy_after)}{" "}
											<span className={energyDelta >= 0 ? "text-emerald-400" : "text-red-400"}>
												({energyDelta >= 0 ? "+" : ""}
												{energyDelta.toFixed(1)})
											</span>
										</div>
										{entry.amount > 0 && <div>Amount: {entry.amount.toFixed(1)}</div>}
										<div className="text-slate-500">Bid: {entry.priority_bid.toFixed(2)}</div>
									</div>
								);
							})()}
					</div>
				</div>
			)}

			{/* Phenotype drawer */}
			{phenotype && (
				<div className="border-t border-slate-800">
					<button
						type="button"
						onClick={() => setDrawerOpen((prev) => !prev)}
						aria-label="Phenotype"
						aria-expanded={drawerOpen}
						className="w-full flex items-center gap-1 px-3 py-1 text-xs text-slate-400 hover:text-slate-300 transition-colors"
					>
						<span className="text-[10px]">{drawerOpen ? "\u25BC" : "\u25B6"}</span>
						<span>Phenotype</span>
					</button>
					{drawerOpen && <PhenotypeDetail phenotype={phenotype} />}
				</div>
			)}
		</div>
	);
});
