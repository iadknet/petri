interface InspectorHeaderProps {
	id: number;
	rgb: [number, number, number];
	generation: number;
	age: number;
	complexity: number;
	onClose: () => void;
}

export function InspectorHeader({
	id,
	rgb,
	generation,
	age,
	complexity,
	onClose,
}: InspectorHeaderProps) {
	return (
		<div className="flex items-start gap-3 px-4 py-3">
			<div
				className="w-10 h-10 rounded-md shrink-0 border border-white/10"
				style={{ backgroundColor: `rgb(${rgb[0]},${rgb[1]},${rgb[2]})` }}
			/>
			<div className="flex-1 min-w-0">
				<div className="flex items-baseline gap-2">
					<span className="text-sm font-mono text-slate-100 font-semibold">#{id}</span>
					<span className="text-xs text-slate-500">Gen {generation}</span>
				</div>
				<div className="text-xs text-slate-400 mt-0.5">
					Age {age.toLocaleString()} · Complexity {complexity}
				</div>
			</div>
			<button
				type="button"
				onClick={onClose}
				aria-label="Close inspector"
				className="text-slate-500 hover:text-slate-300 transition-colors p-1 -mr-1 -mt-0.5"
			>
				<svg width="14" height="14" viewBox="0 0 14 14" fill="none" aria-hidden="true">
					<path
						d="M3 3l8 8M11 3l-8 8"
						stroke="currentColor"
						strokeWidth="1.5"
						strokeLinecap="round"
					/>
				</svg>
			</button>
		</div>
	);
}
