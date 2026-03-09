import type { StarParams as StarParamsType } from "../../types/api.ts";
import { ParamField } from "./ParamField.tsx";

interface StarParamsProps {
	params: StarParamsType;
	onChange: (params: StarParamsType) => void;
}

export function StarParams({ params, onChange }: StarParamsProps) {
	return (
		<div className="flex flex-col gap-2" data-testid="pattern-params-star">
			<ParamField
				label="Centers"
				value={params.point_count}
				min={1}
				max={10}
				step={1}
				onChange={(v) => onChange({ ...params, point_count: v })}
				testId="param-point-count"
			/>
			<ParamField
				label="Rays"
				value={params.ray_count}
				min={2}
				max={32}
				step={1}
				onChange={(v) => onChange({ ...params, ray_count: v })}
				testId="param-ray-count"
			/>
			<ParamField
				label="Ray length"
				value={params.ray_length}
				min={1}
				max={100}
				step={1}
				onChange={(v) => onChange({ ...params, ray_length: v })}
				testId="param-ray-length"
			/>
			<ParamField
				label="Ray thickness"
				value={params.ray_thickness}
				min={1}
				max={8}
				step={1}
				onChange={(v) => onChange({ ...params, ray_thickness: v })}
				testId="param-ray-thickness"
			/>
		</div>
	);
}
