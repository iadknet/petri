import type { FieldDef } from "../shared/types.ts";

export const POPULATION_FIELDS: FieldDef[] = [
	{
		path: "population.max_creatures",
		label: "Max Creatures",
		min: 1,
		max: 100000,
		step: 100,
		defaultValue: 100000,
	},
];
