import type { FbmThresholdParams as Params } from "../../types/pattern.ts";
import { ParamField } from "./ParamField.tsx";

const fields = [
	["octaves", "Octaves", 1, 32, 1],
	["frequency", "Frequency", 0.000001, 1, 0.001],
	["lacunarity", "Lacunarity", 1, 4, 0.1],
	["persistence", "Persistence", 0, 1, 0.01],
	["threshold", "Barrier threshold", -1, 1, 0.01],
] as const;

export function FbmThresholdParams({
	params,
	onChange,
}: { params: Params; onChange: (next: Params) => void }) {
	return (
		<div className="flex flex-col gap-2" data-testid="pattern-params-fbm-threshold">
			<p className="text-xs text-slate-400">Values above the threshold become barriers.</p>
			{fields.map(([key, label, min, max, step]) => (
				<ParamField
					key={key}
					label={label}
					value={params[key]}
					min={min}
					max={max}
					step={step}
					onChange={(value) => onChange({ ...params, [key]: value })}
					testId={`param-fbm-${key}`}
				/>
			))}
		</div>
	);
}
