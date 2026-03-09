import type { SpiralParams as SpiralParamsType } from "../../types/api.ts";
import { ParamField } from "./ParamField.tsx";

interface SpiralParamsProps {
	params: SpiralParamsType;
	onChange: (params: SpiralParamsType) => void;
}

export function SpiralParams({ params, onChange }: SpiralParamsProps) {
	const toggleId = "pattern-param-clockwise";
	return (
		<div className="flex flex-col gap-2" data-testid="pattern-params-spiral">
			<ParamField
				label="Arms"
				value={params.arm_count}
				min={1}
				max={12}
				step={1}
				onChange={(v) => onChange({ ...params, arm_count: v })}
				testId="param-arm-count"
			/>
			<ParamField
				label="Arm thickness"
				value={params.arm_thickness}
				min={1}
				max={8}
				step={1}
				onChange={(v) => onChange({ ...params, arm_thickness: v })}
				testId="param-arm-thickness"
			/>
			<ParamField
				label="Gap width"
				value={params.gap_width}
				min={1}
				max={16}
				step={1}
				onChange={(v) => onChange({ ...params, gap_width: v })}
				testId="param-gap-width"
			/>
			<ParamField
				label="Open center"
				value={params.open_center_radius}
				min={0}
				max={20}
				step={1}
				onChange={(v) => onChange({ ...params, open_center_radius: v })}
				testId="param-open-center"
			/>
			<div className="flex items-center justify-between">
				<label htmlFor={toggleId} className="text-[10px] text-slate-400">
					Clockwise
				</label>
				<input
					id={toggleId}
					data-testid="param-clockwise"
					type="checkbox"
					checked={params.clockwise}
					onChange={(e) => onChange({ ...params, clockwise: e.target.checked })}
					className="accent-emerald-500"
				/>
			</div>
		</div>
	);
}
