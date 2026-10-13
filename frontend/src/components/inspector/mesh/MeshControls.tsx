import type { MeshFocusMode } from "../../../stores/inspectorWorkspace.ts";

interface MeshControlsProps {
	focusMode: MeshFocusMode;
	dimUnreachable: boolean;
	onFocusModeChange: (mode: MeshFocusMode) => void;
	onDimUnreachableChange: (dim: boolean) => void;
}

const FOCUS_MODES: Array<{ id: MeshFocusMode; arrow?: string; label: string }> = [
	{ id: "none", label: "None" },
	{ id: "upstream", arrow: "↑", label: "Up" },
	{ id: "downstream", arrow: "↓", label: "Down" },
];

export function MeshControls({
	focusMode,
	dimUnreachable,
	onFocusModeChange,
	onDimUnreachableChange,
}: MeshControlsProps) {
	return (
		<div className="flex items-center gap-2 border-b border-white/6 px-3 py-1">
			<span className="text-[9px] font-mono uppercase tracking-[0.16em] text-slate-400">Focus</span>
			<div className="flex items-center gap-px rounded-md bg-white/[0.03] p-0.5">
				{FOCUS_MODES.map((mode) => (
					<button
						key={mode.id}
						type="button"
						aria-pressed={focusMode === mode.id}
						onClick={() => onFocusModeChange(mode.id)}
						className={`rounded px-2 py-0.5 text-[10px] font-mono transition-colors ${
							focusMode === mode.id
								? "bg-cyan-400/14 text-cyan-200"
								: "text-slate-400 hover:text-slate-300"
						}`}
					>
						{mode.arrow ? <span aria-hidden="true">{mode.arrow} </span> : null}
						{mode.label}
					</button>
				))}
			</div>

			<button
				type="button"
				aria-pressed={dimUnreachable}
				onClick={() => onDimUnreachableChange(!dimUnreachable)}
				className={`rounded px-2 py-0.5 text-[10px] font-mono transition-colors ${
					dimUnreachable ? "bg-white/[0.04] text-slate-300" : "text-slate-400 hover:text-slate-300"
				}`}
			>
				Dim junk
			</button>
		</div>
	);
}
