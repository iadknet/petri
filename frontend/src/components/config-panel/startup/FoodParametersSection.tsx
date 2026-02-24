import { CollapsibleGroup } from "../shared/CollapsibleGroup.tsx";
import { FieldRow } from "../shared/FieldRow.tsx";
import { getByPath } from "../shared/pathUtils.ts";
import type { FieldDef, StartupSectionProps } from "../shared/types.ts";

const FIELDS: FieldDef[] = [
	{
		path: "world.food.growth_rate",
		label: "Food Growth Rate",
		min: 0,
		max: 1,
		step: 0.001,
		testId: "startup-field-food-growth-rate",
	},
	{
		path: "world.food.initial_density",
		label: "Food Initial Density",
		min: 0,
		max: 255,
		step: 1,
		testId: "startup-field-food-initial-density",
	},
	{
		path: "world.food.initial_coverage",
		label: "Food Coverage",
		min: 0,
		max: 1,
		step: 0.01,
		testId: "startup-field-food-initial-coverage",
	},
];

export function FoodParametersSection({ startupPreset, updateStartupPreset }: StartupSectionProps) {
	return (
		<CollapsibleGroup title="Food Parameters">
			{FIELDS.map((field) => (
				<FieldRow
					key={`startup-${field.path}`}
					field={field}
					id={`startup-${field.path.replaceAll(".", "-")}`}
					value={getByPath(startupPreset, field.path) as number}
					disabled={false}
					onChange={updateStartupPreset}
					testId={field.testId}
				/>
			))}
		</CollapsibleGroup>
	);
}
