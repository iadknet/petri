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
		max: 1,
		step: 0.01,
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
	{
		path: "world.food.spread_threshold_ratio",
		label: "Spread Threshold Ratio",
		min: 0,
		max: 1,
		step: 0.01,
		testId: "startup-field-food-spread-threshold-ratio",
	},
	{
		path: "world.food.recovery_spawn_rate",
		label: "Recovery Spawn Rate",
		min: 0,
		max: 1,
		step: 0.01,
		testId: "startup-field-food-recovery-spawn-rate",
	},
	{
		path: "world.food.recovery_floor_ratio",
		label: "Recovery Floor Ratio",
		min: 0,
		max: 1,
		step: 0.01,
		testId: "startup-field-food-recovery-floor-ratio",
	},
	{
		path: "world.food.max_density",
		label: "Food Max Density",
		min: 0.1,
		max: 1,
		step: 0.01,
		testId: "startup-field-food-max-density",
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
