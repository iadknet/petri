import type { FieldDef } from "../shared/types.ts";

export const ENERGY_LIFECYCLE_FIELDS: FieldDef[] = [
	{
		path: "energy.lifecycle.max_energy",
		label: "Max Energy",
		min: 1,
		max: 1000,
		step: 1,
		defaultValue: 200.0,
		tooltip: "Upper bound on creature energy \u2014 excess is clamped",
	},
	{
		path: "energy.lifecycle.energy_decay_per_tick",
		label: "Decay / Tick",
		min: 0,
		max: 10,
		step: 0.01,
		defaultValue: 0.5,
		tooltip: "Energy lost by every creature each tick (maintenance cost)",
	},
	{
		path: "energy.lifecycle.min_reproduce_energy",
		label: "Min Reproduce Energy",
		min: 0,
		max: 500,
		step: 0.5,
		defaultValue: 1.0,
		tooltip: "Minimum energy required for a creature to reproduce",
	},
	{
		path: "energy.lifecycle.default_offspring_energy",
		label: "Offspring Energy",
		min: 0,
		max: 500,
		step: 0.5,
		defaultValue: 8.0,
		tooltip: "Starting energy given to newborn creatures",
	},
];
