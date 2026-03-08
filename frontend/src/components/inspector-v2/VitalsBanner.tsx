import { memo, useState } from "react";
import type { ActionLogEntry } from "../../types/action-log.ts";
import type { CreaturePhenotype } from "../../types/genome.ts";
import { PhenotypeDetail } from "../inspector/PhenotypeDetail.tsx";

const ACTION_COLORS: Record<string, string> = {
	NoOp: "#475569",
	Eat: "#34d399",
	Move: "#60a5fa",
	Reproduce: "#fbbf24",
	StealEnergy: "#f87171",
};

const MAX_ACTION_DOTS = 50;

interface VitalsBannerProps {
	id: number;
	rgb: [number, number, number];
	generation: number;
	age: number;
	energy: number;
	maxEnergy: number;
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
	position,
	actionLog,
	phenotype,
	isDead,
	onClose,
}: VitalsBannerProps) {
	const [drawerOpen, setDrawerOpen] = useState(false);

	const energyRatio = maxEnergy > 0 ? energy / maxEnergy : 0;
	const energyBarColor = energyRatio > 0.3 ? "bg-emerald-500" : "bg-red-500";
	const energyWidthPercent = Math.min(energyRatio * 100, 100);

	const visibleActions = actionLog ? actionLog.slice(-MAX_ACTION_DOTS) : [];

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

			{/* Row 2: Action timeline dots */}
			{visibleActions.length > 0 && (
				<div className="flex items-center gap-px px-3 pb-1.5">
					{visibleActions.map((entry) => (
						<div
							key={entry.tick}
							data-testid="action-dot"
							className="rounded-full shrink-0"
							style={{
								width: 6,
								height: 6,
								backgroundColor: ACTION_COLORS[entry.action_type] ?? ACTION_COLORS.NoOp,
							}}
							title={`Tick ${entry.tick}: ${entry.action_type}`}
						/>
					))}
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
