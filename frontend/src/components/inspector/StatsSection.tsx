interface StatsSectionProps {
	energy: number;
	maxEnergy: number;
	position: { x: number; y: number };
	age: number;
	generation: number;
}

export function StatsSection({ energy, maxEnergy, position, age, generation }: StatsSectionProps) {
	const ratio = maxEnergy > 0 ? Math.min(1, energy / maxEnergy) : 0;
	const barColor = ratio > 0.3 ? "#34d399" : "#ef4444";

	return (
		<div className="px-4 py-3 space-y-3">
			{/* Energy bar */}
			<div>
				<div className="flex items-baseline justify-between mb-1">
					<span className="text-xs text-slate-400">Energy</span>
					<span className="text-xs font-mono text-slate-300">
						{energy.toFixed(1)} / {maxEnergy.toFixed(0)}
					</span>
				</div>
				<div
					className="h-2 rounded-full bg-slate-800 overflow-hidden"
					role="meter"
					aria-valuenow={energy}
					aria-valuemin={0}
					aria-valuemax={maxEnergy}
					aria-label="Creature energy"
				>
					<div
						className="h-full rounded-full transition-[width] duration-200"
						style={{
							width: `${ratio * 100}%`,
							backgroundColor: barColor,
						}}
					/>
				</div>
			</div>

			{/* Stat rows */}
			<div className="grid grid-cols-2 gap-x-4 gap-y-1.5 text-xs">
				<StatRow label="Position" value={`(${position.x}, ${position.y})`} />
				<StatRow label="Age" value={`${age.toLocaleString()} ticks`} />
				<StatRow label="Generation" value={String(generation)} />
			</div>
		</div>
	);
}

function StatRow({ label, value }: { label: string; value: string }) {
	return (
		<>
			<span className="text-slate-500">{label}</span>
			<span className="text-slate-300 font-mono text-right">{value}</span>
		</>
	);
}
