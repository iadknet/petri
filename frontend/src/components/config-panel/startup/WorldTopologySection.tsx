import { FieldGroup } from "../shared/FieldGroup.tsx";
import { FieldRow } from "../shared/FieldRow.tsx";
import { getByPath } from "../shared/pathUtils.ts";
import type { FieldDef, StartupSectionProps } from "../shared/types.ts";

const FIELDS: FieldDef[] = [
	{
		path: "world.width",
		label: "Width",
		min: 10,
		max: 2000,
		step: 10,
		testId: "startup-field-world-width",
		defaultValue: 400,
		tooltip: "World grid width in cells",
	},
	{
		path: "world.height",
		label: "Height",
		min: 10,
		max: 2000,
		step: 10,
		testId: "startup-field-world-height",
		defaultValue: 400,
		tooltip: "World grid height in cells",
	},
];

export function WorldTopologySection({ startupPreset, updateStartupPreset }: StartupSectionProps) {
	return (
		<FieldGroup title="World Topology">
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
