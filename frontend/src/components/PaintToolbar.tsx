import { usePaintStore } from "../stores/paint.ts";
import type { PaintTool } from "../types/api.ts";

const TOOLS: { id: PaintTool; label: string; title: string }[] = [
	{ id: "barrier", label: "Barrier", title: "Draw barriers" },
	{ id: "food", label: "Food", title: "Place food at max density" },
	{ id: "erase_barrier", label: "- Barrier", title: "Erase barriers" },
	{ id: "erase_food", label: "- Food", title: "Erase food" },
];

const BRUSH_SIZES: { extent: 0 | 1 | 2; label: string }[] = [
	{ extent: 0, label: "1x1" },
	{ extent: 1, label: "3x3" },
	{ extent: 2, label: "5x5" },
];

export function PaintToolbar() {
	const tool = usePaintStore((s) => s.tool);
	const brushHalfExtent = usePaintStore((s) => s.brushHalfExtent);
	const setTool = usePaintStore((s) => s.setTool);
	const setBrushHalfExtent = usePaintStore((s) => s.setBrushHalfExtent);

	return (
		<div className="absolute top-3 left-3 flex flex-col gap-2 z-10 bg-slate-800/60 backdrop-blur-sm rounded-lg p-2">
			<div className="text-[10px] uppercase tracking-wider text-slate-400 px-1">Tool</div>
			<div className="flex flex-col gap-0.5">
				{TOOLS.map((t) => (
					<button
						key={t.id}
						type="button"
						onClick={() => setTool(t.id)}
						title={t.title}
						aria-pressed={tool === t.id}
						aria-label={t.title}
						className={`px-2 py-1 text-xs rounded transition-colors ${
							tool === t.id
								? "bg-emerald-600 text-white"
								: "bg-slate-700 text-slate-200 hover:bg-slate-600"
						}`}
					>
						{t.label}
					</button>
				))}
			</div>
			<div className="text-[10px] uppercase tracking-wider text-slate-400 px-1 mt-1">Brush</div>
			<div className="flex gap-0.5">
				{BRUSH_SIZES.map((b) => (
					<button
						key={b.extent}
						type="button"
						onClick={() => setBrushHalfExtent(b.extent)}
						aria-pressed={brushHalfExtent === b.extent}
						aria-label={`Brush size ${b.label}`}
						className={`px-2 py-1 text-xs rounded transition-colors ${
							brushHalfExtent === b.extent
								? "bg-emerald-600 text-white"
								: "bg-slate-700 text-slate-200 hover:bg-slate-600"
						}`}
					>
						{b.label}
					</button>
				))}
			</div>
		</div>
	);
}
