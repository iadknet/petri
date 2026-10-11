import { FieldGroup } from "../shared/FieldGroup.tsx";
import { FieldRow } from "../shared/FieldRow.tsx";
import { getByPath } from "../shared/pathUtils.ts";
import type { FieldDef, StartupSectionProps } from "../shared/types.ts";

export const ENERGY_STARTUP_FIELDS: FieldDef[] = [
	{
		path: "energy.initial_energy",
		label: "Initial Energy",
		min: 0,
		max: 500,
		step: 0.5,
		testId: "startup-field-energy-initial-energy",
		defaultValue: 20.0,
		tooltip:
			"Starting energy for each founder. While the world runs it is also the smallest litter a parent may conceive",
	},
];

export function EnergySection({ startupPreset, updateStartupPreset }: StartupSectionProps) {
	return (
		<FieldGroup title="Founder energy">
			{ENERGY_STARTUP_FIELDS.map((field) => (
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
