import { EnumRow } from "../shared/EnumRow.tsx";
import { FieldGroup } from "../shared/FieldGroup.tsx";
import { FieldRow } from "../shared/FieldRow.tsx";
import { getByPath } from "../shared/pathUtils.ts";
import type { EnumFieldDef, FieldDef, StartupSectionProps } from "../shared/types.ts";

export const POPULATION_STARTUP_FIELDS: FieldDef[] = [
	{
		path: "population.initial_creatures",
		label: "Initial Creatures",
		min: 1,
		max: 1000000,
		step: 1,
		sliderMax: 20000,
		testId: "startup-field-population-initial-creatures",
		defaultValue: 10000,
		tooltip:
			"Number of founders spawned at world initialization; must not exceed the live Max Creatures",
	},
];

export const FOUNDER_PROFILE_FIELD: EnumFieldDef = {
	path: "population.founder_profile",
	label: "Founder Profile",
	testId: "startup-field-population-founder-profile",
	defaultValue: "v3_alpha1",
	options: [
		{ value: "v3_alpha1", label: "Canonical (v3 alpha1)" },
		{ value: "forage_first_sparse", label: "Forage first" },
		{ value: "forage_first_sparse_conservative", label: "Forage first, conservative" },
		{ value: "forage_first_sparse_rich_offspring", label: "Forage first, rich offspring" },
		{ value: "forage_first_sparse_balanced", label: "Forage first, balanced" },
	],
	tooltip:
		"Founder genome seeded at startup. The forage-first variants forage before breeding; conservative delays branching, rich offspring gives each child more energy, balanced sits between",
};

export function PopulationSection({ startupPreset, updateStartupPreset }: StartupSectionProps) {
	return (
		<FieldGroup title="Founders">
			{POPULATION_STARTUP_FIELDS.map((field) => (
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
			<EnumRow
				field={FOUNDER_PROFILE_FIELD}
				id="startup-population-founder_profile"
				value={startupPreset.population.founder_profile}
				disabled={false}
				onChange={updateStartupPreset}
				testId={FOUNDER_PROFILE_FIELD.testId}
			/>
		</FieldGroup>
	);
}
