import type { ParallelLinesParams } from "../../types/api.ts";
import { ParamField } from "./ParamField.tsx";

interface LinesParamsProps {
	params: ParallelLinesParams;
	onChange: (params: ParallelLinesParams) => void;
}

export function LinesParams({ params, onChange }: LinesParamsProps) {
	return (
		<div className="flex flex-col gap-2" data-testid="pattern-params-lines">
			<ParamField
				label="Spacing"
				value={params.spacing}
				min={2}
				max={32}
				step={1}
				onChange={(v) => onChange({ ...params, spacing: v })}
				testId="param-spacing"
			/>
			<ParamField
				label="Thickness"
				value={params.thickness}
				min={1}
				max={8}
				step={1}
				onChange={(v) => onChange({ ...params, thickness: v })}
				testId="param-thickness"
			/>
			<ParamField
				label="Jaggedness"
				value={params.jaggedness}
				min={0}
				max={1}
				step={0.01}
				onChange={(v) => onChange({ ...params, jaggedness: v })}
				testId="param-jaggedness"
			/>
			<ParamField
				label="Angle"
				value={params.angle_degrees}
				min={0}
				max={360}
				step={1}
				onChange={(v) => onChange({ ...params, angle_degrees: v })}
				testId="param-angle"
			/>
		</div>
	);
}
