import { FieldGroup } from "../shared/FieldGroup.tsx";
import { FieldRow } from "../shared/FieldRow.tsx";
import { ToggleRow } from "../shared/ToggleRow.tsx";
import { getByPath } from "../shared/pathUtils.ts";
import type { BooleanFieldDef, FieldDef, StartupSectionProps } from "../shared/types.ts";

const TOGGLE: BooleanFieldDef = {
	path: "startup.ramps.failed_action_penalty.enabled",
	label: "Enable Failed Action Penalty Ramp",
	testId: "startup-field-ramp-failed-action-penalty-enabled",
	defaultValue: true,
	tooltip:
		"When enabled, failed action penalty linearly interpolates from Start to End until Target Tick.",
};

const FIELDS: FieldDef[] = [
	{
		path: "startup.ramps.failed_action_penalty.start",
		label: "Start Penalty",
		min: 0,
		max: 100,
		step: 0.1,
		testId: "startup-field-ramp-failed-action-penalty-start",
		defaultValue: 0.0,
		tooltip: "Failed action penalty applied at tick 0 while ramp is enabled.",
	},
	{
		path: "startup.ramps.failed_action_penalty.end",
		label: "End Penalty",
		min: 0,
		max: 100,
		step: 0.1,
		testId: "startup-field-ramp-failed-action-penalty-end",
		defaultValue: 1.0,
		tooltip:
			"Target failed action penalty reached at Target Tick and used as runtime config value.",
	},
	{
		path: "startup.ramps.failed_action_penalty.target_tick",
		label: "Target Tick",
		min: 1,
		max: 1_000_000,
		step: 1,
		testId: "startup-field-ramp-failed-action-penalty-target-tick",
		defaultValue: 62680,
		tooltip: "Ramp completion tick. Interpolation runs while current tick is less than this value.",
	},
];

export function StartupRampsSection({ startupPreset, updateStartupPreset }: StartupSectionProps) {
	return (
		<FieldGroup title="Startup Ramps">
			<ToggleRow
				field={TOGGLE}
				id={`startup-${TOGGLE.path.replaceAll(".", "-")}`}
				value={getByPath(startupPreset, TOGGLE.path) as boolean}
				disabled={false}
				onChange={updateStartupPreset}
				testId={TOGGLE.testId}
			/>
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
