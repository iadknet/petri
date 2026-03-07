import { memo, useMemo } from "react";

interface SharedMemoryViewProps {
	slots: number[];
}

function slotsEqual(prev: SharedMemoryViewProps, next: SharedMemoryViewProps): boolean {
	const a = prev.slots;
	const b = next.slots;
	if (a === b) return true;
	if (a.length !== b.length) return false;
	for (let i = 0; i < a.length; i++) {
		if (a[i] !== b[i]) return false;
	}
	return true;
}

export const SharedMemoryView = memo(function SharedMemoryView({ slots }: SharedMemoryViewProps) {
	const nonZeroCount = useMemo(() => slots.filter((v) => v !== 0).length, [slots]);

	return (
		<div className="px-4 py-3 space-y-2">
			<div className="flex items-baseline justify-between">
				<span className="text-xs text-slate-500 uppercase tracking-wider font-medium">
					Shared Memory
				</span>
				<span className="text-[10px] text-slate-600 font-mono">
					{nonZeroCount}/{slots.length} active
				</span>
			</div>

			<div className="grid grid-cols-4 gap-1" aria-label="Shared memory slots">
				{slots.map((value, i) => (
					<div
						key={i}
						className={`flex items-center gap-1.5 px-1.5 py-0.5 rounded text-[10px] font-mono ${
							value !== 0
								? "bg-slate-800/80 text-slate-200"
								: "bg-slate-900/40 text-slate-600"
						}`}
					>
						<span className="text-slate-500 w-3 text-right shrink-0">{i}</span>
						<span className="truncate">{value.toFixed(3)}</span>
					</div>
				))}
			</div>
		</div>
	);
}, slotsEqual);
