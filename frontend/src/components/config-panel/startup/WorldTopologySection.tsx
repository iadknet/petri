import { EnumRow } from "../shared/EnumRow.tsx";
import { FieldGroup } from "../shared/FieldGroup.tsx";
import { FieldRow } from "../shared/FieldRow.tsx";
import { getByPath } from "../shared/pathUtils.ts";
import type { EnumFieldDef, FieldDef, StartupSectionProps } from "../shared/types.ts";

export const WORLD_TOPOLOGY_FIELDS: FieldDef[] = [
	{
		path: "world.width",
		label: "Width",
		min: 10,
		max: 65535,
		step: 10,
		sliderMax: 4000,
		testId: "startup-field-world-width",
		defaultValue: 1600,
		tooltip: "World grid width in cells",
	},
	{
		path: "world.height",
		label: "Height",
		min: 10,
		max: 65535,
		step: 10,
		sliderMax: 4000,
		testId: "startup-field-world-height",
		defaultValue: 1600,
		tooltip: "World grid height in cells",
	},
];

export const EDGE_MODE_FIELD: EnumFieldDef = {
	path: "world.edge_mode",
	label: "Edge Mode",
	testId: "startup-field-world-edge-mode",
	defaultValue: "Wrap",
	options: [
		{ value: "Wrap", label: "Wrap" },
		{ value: "Bounded", label: "Bounded" },
	],
	tooltip:
		"How creatures interact with world boundaries. Wrap: opposite edges connect. Bounded: edges are walls.",
};

export function WorldTopologySection({ startupPreset, updateStartupPreset }: StartupSectionProps) {
	return (
		<FieldGroup title="Map">
			<EnumRow
				field={EDGE_MODE_FIELD}
				id="startup-world-edge-mode"
				value={startupPreset.world.edge_mode}
				disabled={false}
				onChange={updateStartupPreset}
				testId={EDGE_MODE_FIELD.testId}
			/>
			{WORLD_TOPOLOGY_FIELDS.map((field) => (
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
