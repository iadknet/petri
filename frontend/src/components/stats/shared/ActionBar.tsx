export function ActionBar({
	move,
	eat,
	reproduce,
	noop,
	steal,
	give,
}: { move: number; eat: number; reproduce: number; noop: number; steal: number; give: number }) {
	const total = move + eat + reproduce + steal + give + noop || 1;
	const pct = (n: number) => `${((n / total) * 100).toFixed(1)}%`;

	return (
		<div className="flex flex-col gap-1">
			<div className="flex h-3 rounded overflow-hidden">
				<div className="bg-blue-500" style={{ width: pct(move) }} title={`Move: ${move}`} />
				<div className="bg-emerald-500" style={{ width: pct(eat) }} title={`Eat: ${eat}`} />
				<div
					className="bg-amber-500"
					style={{ width: pct(reproduce) }}
					title={`Reproduce: ${reproduce}`}
				/>
				<div className="bg-red-500" style={{ width: pct(steal) }} title={`Steal: ${steal}`} />
				<div className="bg-purple-400" style={{ width: pct(give) }} title={`Give: ${give}`} />
				<div className="bg-slate-600" style={{ width: pct(noop) }} title={`Noop: ${noop}`} />
			</div>
			<div className="flex gap-3 text-[10px] text-slate-400">
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
					Reproduce {reproduce}
				</span>
				<span className="flex items-center gap-1">
					<span className="w-2 h-2 bg-red-500 rounded-sm" />
					Steal {steal}
				</span>
				<span className="flex items-center gap-1">
					<span className="w-2 h-2 bg-purple-400 rounded-sm" />
					Give {give}
				</span>
				<span className="flex items-center gap-1">
					<span className="w-2 h-2 bg-slate-600 rounded-sm" />
					Noop {noop}
				</span>
			</div>
		</div>
	);
}
