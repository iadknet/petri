interface NodeMemorySlotsProps {
	sharedMemory: number[] | null;
	hasMemoryWrite: boolean;
}

export function NodeMemorySlots({ sharedMemory, hasMemoryWrite }: NodeMemorySlotsProps) {
	if (!hasMemoryWrite || !sharedMemory) {
		return null;
	}

	return (
		<div className="px-3 py-2 space-y-1">
			<div className="flex items-baseline justify-between">
				<span className="text-[10px] text-slate-500 uppercase tracking-wider font-medium">
					Shared Memory
				</span>
				<span className="text-[10px] text-slate-600 font-mono">
					{sharedMemory.filter((v) => v !== 0).length}/{sharedMemory.length} active
				</span>
			</div>

			<div className="grid grid-cols-4 gap-1" aria-label="Shared memory slots">
				{sharedMemory.map((value, i) => (
					<div
						// biome-ignore lint/suspicious/noArrayIndexKey: memory slots are indexed by position
						key={i}
						className={`flex items-center gap-1.5 px-1.5 py-0.5 rounded text-[10px] font-mono ${
							value !== 0 ? "bg-slate-800/80 text-slate-200" : "bg-slate-900/40 text-slate-600"
						}`}
					>
						<span className="text-slate-500 w-3 text-right shrink-0">{i}</span>
						<span className="truncate">{value.toFixed(3)}</span>
					</div>
				))}
			</div>
		</div>
	);
}
