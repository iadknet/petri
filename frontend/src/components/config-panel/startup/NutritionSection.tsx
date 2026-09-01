import { FieldGroup } from "../shared/FieldGroup.tsx";
import { FieldRow } from "../shared/FieldRow.tsx";
import { getByPath } from "../shared/pathUtils.ts";
import type { FieldDef, StartupSectionProps } from "../shared/types.ts";

const FIELDS: FieldDef[] = [
	{
		path: "nutrition.reproductive_reserve_capacity",
		label: "Reserve Capacity",
		min: 0.1,
		max: 100,
		step: 0.1,
		defaultValue: 8,
		testId: "startup-field-nutrition-reserve-capacity",
		tooltip: "Maximum reproductive reserve held by a creature",
	},
	{
		path: "nutrition.reproductive_reserve_cost",
		label: "Reserve Cost",
		min: 0.1,
		max: 100,
		step: 0.1,
		defaultValue: 4,
		testId: "startup-field-nutrition-reserve-cost",
		tooltip: "Reserve spent by the parent when reproduction succeeds",
	},
];

export function NutritionSection({ startupPreset, updateStartupPreset }: StartupSectionProps) {
	return (
		<FieldGroup title="Nutrition">
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
