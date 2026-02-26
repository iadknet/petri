import { FieldGroup } from "../shared/FieldGroup.tsx";
import { FieldRow } from "../shared/FieldRow.tsx";
import { getByPath } from "../shared/pathUtils.ts";
import type { FieldDef, StartupSectionProps } from "../shared/types.ts";

const FIELDS: FieldDef[] = [
	{
		path: "population.initial_creatures",
		label: "Initial Creatures",
		min: 1,
		max: 10000,
		step: 1,
		testId: "startup-field-population-initial-creatures",
	},
];

export function PopulationSection({ startupPreset, updateStartupPreset }: StartupSectionProps) {
	return (
		<FieldGroup title="Population">
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
