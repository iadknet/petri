import type { BooleanFieldDef, FieldDef } from "../shared/types.ts";

export const AGE_COST_FIELDS: FieldDef[] = [
	{
		path: "energy.age_cost.age_cap",
		label: "Age Cap",
		min: 1,
		max: 5000,
		step: 10,
		testId: "config-field-age-cost-age-cap",
		defaultValue: 500,
		tooltip: "Age in ticks at which the maximum cost multiplier applies",
	},
	{
		path: "energy.age_cost.max_multiplier",
		label: "Max Multiplier",
		min: 1,
		max: 50,
		step: 0.5,
		testId: "config-field-age-cost-max-multiplier",
		defaultValue: 10,
		tooltip: "Maximum energy cost multiplier applied to creatures at or beyond age cap",
	},
];

export const AGE_COST_TOGGLES: BooleanFieldDef[] = [
	{
		path: "energy.age_cost.enabled",
		label: "Age Cost",
		testId: "config-field-age-cost-enabled",
		defaultValue: true,
		tooltip: "When enabled, older creatures pay higher energy costs for all actions",
	},
];

export const AGE_COST_ALL_FIELDS: (FieldDef | BooleanFieldDef)[] = [
	...AGE_COST_FIELDS,
	...AGE_COST_TOGGLES,
];
