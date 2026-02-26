import { FieldGroup } from "../shared/FieldGroup.tsx";
import { FieldRow } from "../shared/FieldRow.tsx";
import { getByPath } from "../shared/pathUtils.ts";
import type { FieldDef, StartupSectionProps } from "../shared/types.ts";

const FIELDS: FieldDef[] = [
	{
		path: "world.food.initial_density",
		label: "Food Initial Density",
		min: 0,
		max: 1,
		step: 0.01,
		testId: "startup-field-food-initial-density",
		defaultValue: 1.0,
	},
	{
		path: "world.food.initial_coverage",
		label: "Food Coverage",
		min: 0,
		max: 1,
		step: 0.01,
		testId: "startup-field-food-initial-coverage",
		defaultValue: 0.15,
	},
];

export function FoodParametersSection({ startupPreset, updateStartupPreset }: StartupSectionProps) {
	return (
		<FieldGroup title="Food Parameters">
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
		</FieldGroup>
	);
}
