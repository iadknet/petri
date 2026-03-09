import type { MazeParams as MazeParamsType } from "../../types/api.ts";
import { ParamField } from "./ParamField.tsx";

interface MazeParamsProps {
	params: MazeParamsType;
	onChange: (params: MazeParamsType) => void;
}

export function MazeParams({ params, onChange }: MazeParamsProps) {
	return (
		<div className="flex flex-col gap-2" data-testid="pattern-params-maze">
			<ParamField
				label="Corridor width"
				value={params.corridor_width}
				min={1}
				max={10}
				step={1}
				onChange={(v) => onChange({ ...params, corridor_width: v })}
				testId="param-corridor-width"
			/>
			<ParamField
				label="Wall thickness"
				value={params.wall_thickness}
				min={1}
				max={5}
				step={1}
				onChange={(v) => onChange({ ...params, wall_thickness: v })}
				testId="param-wall-thickness"
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
		</div>
	);
}
