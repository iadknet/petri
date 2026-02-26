export function ActionBar({
	move,
	eat,
	reproduce,
	noop,
}: { move: number; eat: number; reproduce: number; noop: number }) {
	const total = move + eat + reproduce + noop || 1;
	const pct = (n: number) => `${((n / total) * 100).toFixed(1)}%`;

	return (
		<div className="flex flex-col gap-1">
			<span className="text-[11px] text-slate-400">Actions</span>
			<div className="flex h-3 rounded overflow-hidden">
				<div className="bg-blue-500" style={{ width: pct(move) }} title={`Move: ${move}`} />
				<div className="bg-emerald-500" style={{ width: pct(eat) }} title={`Eat: ${eat}`} />
				<div
					className="bg-amber-500"
					style={{ width: pct(reproduce) }}
					title={`Reproduce: ${reproduce}`}
				/>
				<div className="bg-slate-600" style={{ width: pct(noop) }} title={`Noop: ${noop}`} />
			</div>
			<div className="flex gap-3 text-[10px] text-slate-500">
				<span className="flex items-center gap-1">
					<span className="w-2 h-2 bg-blue-500 rounded-sm" />
					Move {move}
				</span>
				<span className="flex items-center gap-1">
					<span className="w-2 h-2 bg-emerald-500 rounded-sm" />
					Eat {eat}
				</span>
				<span className="flex items-center gap-1">
					<span className="w-2 h-2 bg-amber-500 rounded-sm" />
					Repro {reproduce}
				</span>
				<span className="flex items-center gap-1">
					<span className="w-2 h-2 bg-slate-600 rounded-sm" />
					Noop {noop}
				</span>
			</div>
		</div>
	);
}
