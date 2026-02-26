export function Gauge({
	label,
	value,
	delta,
	format,
}: { label: string; value: number; delta?: number; format?: (v: number) => string }) {
	const fmt = format ?? ((v: number) => v.toLocaleString());
	return (
		<div className="flex flex-col">
			<span className="text-[11px] font-sans text-slate-400">{label}</span>
			<div className="flex items-baseline gap-1.5">
				<span className="text-xl font-mono text-slate-100 tabular-nums">{fmt(value)}</span>
				{delta !== undefined && delta !== 0 && (
					<span
						className={`text-[11px] font-mono tabular-nums ${
							delta > 0 ? "text-emerald-400" : "text-red-400"
						}`}
					>
						{delta > 0 ? "+" : ""}
						{fmt(delta)}
					</span>
				)}
			</div>
		</div>
	);
}
