import type { PatternParams } from "../../types/pattern.ts";
import { LinesParams } from "./LinesParams.tsx";
import { MazeParams } from "./MazeParams.tsx";
import { NoiseParams } from "./NoiseParams.tsx";
import { SpiralParams } from "./SpiralParams.tsx";
import { StarParams } from "./StarParams.tsx";
export function PatternParamsPanel({
	params,
	onChange,
}: {
	params: PatternParams;
	onChange: (params: PatternParams) => void;
}) {
	switch (params.pattern_type) {
		case "Maze":
			return <MazeParams params={params} onChange={(p) => onChange(p)} />;
		case "Spiral":
			return <SpiralParams params={params} onChange={(p) => onChange(p)} />;
		case "Noise":
			return <NoiseParams params={params} onChange={(p) => onChange(p)} />;
		case "ParallelLines":
			return <LinesParams params={params} onChange={(p) => onChange(p)} />;
		case "Star":
			return <StarParams params={params} onChange={(p) => onChange(p)} />;
	}
}
