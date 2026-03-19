import { FieldGroup } from "../shared/FieldGroup.tsx";
import { FieldRow } from "../shared/FieldRow.tsx";
import { ToggleRow } from "../shared/ToggleRow.tsx";
import { getByPath } from "../shared/pathUtils.ts";
import type { BooleanFieldDef, FieldDef, StartupSectionProps } from "../shared/types.ts";

const FERTILITY_TOGGLE: BooleanFieldDef = {
	path: "world.food.fertility.enabled",
	label: "Enable Fertility Layer",
	testId: "startup-field-fertility-enabled",
	defaultValue: false,
	tooltip: "When enabled, spatial fertility multipliers affect food growth rates across the world",
};

const FERTILITY_FIELDS: FieldDef[] = [
	{
		path: "world.food.fertility.min_fertility",
		label: "Min Fertility",
		min: 0.0,
		max: 5.0,
		step: 0.1,
		testId: "startup-field-fertility-min",
		defaultValue: 0.0,
		tooltip: "Minimum growth multiplier for barren zones",
	},
	{
		path: "world.food.fertility.max_fertility",
		label: "Max Fertility",
		min: 0.0,
		max: 5.0,
		step: 0.1,
		testId: "startup-field-fertility-max",
		defaultValue: 2.0,
		tooltip: "Maximum growth multiplier for fertile zones",
	},
];

const ANNEALING_TOGGLE: BooleanFieldDef = {
	path: "world.food.annealing.enabled",
	label: "Enable Warmup Annealing",
	testId: "startup-field-annealing-enabled",
	defaultValue: false,
	tooltip:
		"When enabled, fertility severity ramps gradually from friendly initial values to target values",
};

const ANNEALING_FIELDS: FieldDef[] = [
	{
		path: "world.food.annealing.ramp_ticks",
		label: "Ramp Ticks",
		min: 100,
		max: 50000,
		step: 100,
		testId: "startup-field-annealing-ramp-ticks",
		defaultValue: 5000,
		tooltip: "Ticks over which fertility severity ramps from initial to target values",
	},
	{
		path: "world.food.annealing.initial_min_fertility",
		label: "Initial Min Fertility",
		min: 0.0,
		max: 5.0,
		step: 0.1,
		testId: "startup-field-annealing-initial-min",
		defaultValue: 0.3,
		tooltip: "Starting minimum fertility (friendlier for founders)",
	},
	{
		path: "world.food.annealing.initial_max_fertility",
		label: "Initial Max Fertility",
		min: 0.0,
		max: 5.0,
		step: 0.1,
		testId: "startup-field-annealing-initial-max",
		defaultValue: 1.5,
		tooltip: "Starting maximum fertility (less extreme early)",
	},
];

export function FertilitySection({ startupPreset, updateStartupPreset }: StartupSectionProps) {
	const fertilityEnabled =
		(getByPath(startupPreset, FERTILITY_TOGGLE.path) as boolean) ?? false;
	const annealingEnabled =
		(getByPath(startupPreset, ANNEALING_TOGGLE.path) as boolean) ?? false;

	return (
		<>
			<FieldGroup title="Fertility Layer">
				<ToggleRow
					field={FERTILITY_TOGGLE}
					id={`startup-${FERTILITY_TOGGLE.path.replaceAll(".", "-")}`}
					value={fertilityEnabled}
					disabled={false}
					onChange={updateStartupPreset}
					testId={FERTILITY_TOGGLE.testId}
				/>
				{fertilityEnabled &&
					FERTILITY_FIELDS.map((field) => (
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
			{fertilityEnabled && (
				<FieldGroup title="Warmup Annealing">
					<ToggleRow
						field={ANNEALING_TOGGLE}
						id={`startup-${ANNEALING_TOGGLE.path.replaceAll(".", "-")}`}
						value={annealingEnabled}
						disabled={false}
						onChange={updateStartupPreset}
						testId={ANNEALING_TOGGLE.testId}
					/>
					{annealingEnabled &&
						ANNEALING_FIELDS.map((field) => (
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
			)}
		</>
	);
}
