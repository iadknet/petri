import type { NoiseParams as NoiseParamsType } from "../../types/api.ts";
import { ParamField } from "./ParamField.tsx";

interface NoiseParamsProps {
	params: NoiseParamsType;
	onChange: (params: NoiseParamsType) => void;
}

export function NoiseParams({ params, onChange }: NoiseParamsProps) {
	return (
		<div className="flex flex-col gap-2" data-testid="pattern-params-noise">
			<ParamField
				label="Density"
				value={params.density}
				min={0}
				max={1}
				step={0.01}
				onChange={(v) => onChange({ ...params, density: v })}
				testId="param-density"
			/>
			<ParamField
				label="Cluster size"
				value={params.cluster_size}
				min={1}
				max={10}
				step={1}
				onChange={(v) => onChange({ ...params, cluster_size: v })}
				testId="param-cluster-size"
			/>
		</div>
	);
}
