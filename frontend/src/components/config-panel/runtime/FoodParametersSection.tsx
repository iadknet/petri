import { CollapsibleGroup } from "../shared/CollapsibleGroup.tsx";
import { FieldRow } from "../shared/FieldRow.tsx";
import { getByPath } from "../shared/pathUtils.ts";
import type { FieldDef, RuntimePanelProps } from "../shared/types.ts";

export const FOOD_PARAMETERS_FIELDS: FieldDef[] = [
	{
		path: "world.food.growth_rate",
		label: "Food Growth Rate",
		min: 0,
		max: 1,
		step: 0.001,
	},
	{
		path: "world.food.initial_density",
		label: "Food Initial Density",
		min: 0,
		max: 255,
		step: 1,
	},
	{
		path: "world.food.initial_coverage",
		label: "Food Coverage",
		min: 0,
		max: 1,
		step: 0.01,
	},
];

export function FoodParametersSection({
	localDraft,
	serverConfig,
	simState,
	updateDraft,
}: RuntimePanelProps) {
	return (
		<CollapsibleGroup title="Food Parameters">
			{FOOD_PARAMETERS_FIELDS.map((field) => (
				<FieldRow
					key={`runtime-${field.path}`}
					field={field}
					id={`runtime-${field.path.replaceAll(".", "-")}`}
					value={getByPath(localDraft, field.path) as number}
					serverValue={getByPath(serverConfig, field.path) as number}
					disabled={simState === "running"}
					onChange={updateDraft}
					testId={field.testId}
				/>
			))}
		</CollapsibleGroup>
	);
}
