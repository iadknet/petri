import { usePaintStore } from "../stores/paint.ts";
import { usePatternStore } from "../stores/pattern.ts";
import type {
	MazeParams as MazeParamsType,
	NoiseParams as NoiseParamsType,
	ParallelLinesParams,
	PatternParams,
	PatternType,
	SpiralParams as SpiralParamsType,
	StarParams as StarParamsType,
} from "../types/api.ts";
import { LinesParams } from "./pattern-params/LinesParams.tsx";
import { MazeParams } from "./pattern-params/MazeParams.tsx";
import { NoiseParams } from "./pattern-params/NoiseParams.tsx";
import { SpiralParams } from "./pattern-params/SpiralParams.tsx";
import { StarParams } from "./pattern-params/StarParams.tsx";

const PATTERNS: { id: PatternType; label: string }[] = [
	{ id: "Maze", label: "Maze" },
	{ id: "Spiral", label: "Spiral" },
	{ id: "Noise", label: "Noise" },
	{ id: "ParallelLines", label: "Lines" },
	{ id: "Star", label: "Star" },
];

function PatternParamsPanel({
	params,
	onChange,
}: {
	params: PatternParams;
	onChange: (params: PatternParams) => void;
}) {
	switch (params.pattern_type) {
		case "Maze":
			return <MazeParams params={params} onChange={onChange as (p: MazeParamsType) => void} />;
		case "Spiral":
			return (
				<SpiralParams params={params} onChange={onChange as (p: SpiralParamsType) => void} />
			);
		case "Noise":
			return (
				<NoiseParams params={params} onChange={onChange as (p: NoiseParamsType) => void} />
			);
		case "ParallelLines":
			return (
				<LinesParams params={params} onChange={onChange as (p: ParallelLinesParams) => void} />
			);
		case "Star":
			return <StarParams params={params} onChange={onChange as (p: StarParamsType) => void} />;
	}
}

export function PatternToolbar() {
	const setMode = usePaintStore((s) => s.setMode);
	const selectedPattern = usePatternStore((s) => s.selectedPattern);
	const patternParams = usePatternStore((s) => s.patternParams);
	const areaBounds = usePatternStore((s) => s.areaBounds);
	const patternSeed = usePatternStore((s) => s.patternSeed);
	const selectPattern = usePatternStore((s) => s.selectPattern);
	const setPatternParams = usePatternStore((s) => s.setPatternParams);
	const randomizeSeed = usePatternStore((s) => s.randomizeSeed);
	const clearPattern = usePatternStore((s) => s.clearPattern);

	const hasArea = areaBounds !== null;

	return (
		<div
			data-testid="pattern-toolbar"
			className="absolute top-3 left-3 flex flex-col gap-2 z-10 bg-slate-800/60 backdrop-blur-sm rounded-lg p-2 w-48"
		>
			{/* Mode switcher */}
			<div className="flex gap-0.5">
				<button
					type="button"
					data-testid="mode-brush"
					onClick={() => setMode("brush")}
					className="flex-1 px-2 py-1 text-[10px] uppercase tracking-wider rounded transition-colors bg-slate-700 text-slate-200 hover:bg-slate-600"
				>
					Brush
				</button>
				<button
					type="button"
					data-testid="mode-pattern"
					aria-pressed={true}
					className="flex-1 px-2 py-1 text-[10px] uppercase tracking-wider rounded transition-colors bg-emerald-600 text-white"
				>
					Pattern
				</button>
			</div>

			{/* Pattern type selection */}
			<div className="text-[10px] uppercase tracking-wider text-slate-400 px-1">Pattern</div>
			<div className="flex flex-col gap-0.5">
				{PATTERNS.map((p) => (
					<button
						key={p.id}
						type="button"
						data-testid={`pattern-type-${p.id}`}
						onClick={() => selectPattern(p.id)}
						aria-pressed={selectedPattern === p.id}
						aria-label={`${p.label} pattern`}
						className={`px-2 py-1 text-xs rounded transition-colors ${
							selectedPattern === p.id
								? "bg-emerald-600 text-white"
								: "bg-slate-700 text-slate-200 hover:bg-slate-600"
						}`}
					>
						{p.label}
					</button>
				))}
			</div>

			{/* Pattern parameters */}
			<div className="text-[10px] uppercase tracking-wider text-slate-400 px-1 mt-1">
				Parameters
			</div>
			<PatternParamsPanel params={patternParams} onChange={setPatternParams} />

			{/* Seed */}
			<div className="text-[10px] uppercase tracking-wider text-slate-400 px-1 mt-1">Seed</div>
			<div className="flex items-center gap-1">
				<span
					data-testid="pattern-seed"
					className="flex-1 px-1.5 py-0.5 text-[10px] font-mono bg-slate-800 border border-slate-700 rounded text-slate-300 truncate"
				>
					{patternSeed}
				</span>
				<button
					type="button"
					data-testid="pattern-randomize-seed"
					onClick={randomizeSeed}
					title="Randomize seed"
					aria-label="Randomize seed"
					className="px-1.5 py-0.5 text-xs bg-slate-700 text-slate-200 rounded hover:bg-slate-600 transition-colors"
				>
					&#x21bb;
				</button>
			</div>

			{/* Actions */}
			<div className="flex gap-1 mt-1">
				<button
					type="button"
					data-testid="pattern-apply"
					disabled={!hasArea}
					className="flex-1 px-2 py-1.5 text-xs font-medium rounded transition-colors bg-emerald-600 text-white hover:bg-emerald-500 disabled:opacity-40 disabled:cursor-not-allowed"
				>
					Apply
				</button>
				<button
					type="button"
					data-testid="pattern-cancel"
					onClick={clearPattern}
					disabled={!hasArea}
					className="flex-1 px-2 py-1.5 text-xs rounded transition-colors bg-slate-700 text-slate-200 hover:bg-slate-600 disabled:opacity-40 disabled:cursor-not-allowed"
				>
					Cancel
				</button>
			</div>

			{!hasArea && (
				<div className="text-[10px] text-slate-500 px-1 text-center">
					Click and drag on the canvas to select an area
				</div>
			)}
		</div>
	);
}
