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
		path: "energy.lifecycle.genome_carry_cost_per_unit",
		label: "Carry Cost / Unit",
		min: 0,
		max: 0.01,
		step: 0.00001,
		defaultValue: 0.0001,
		tooltip:
			"Energy charged each tick per unit of genome_size() \u2014 maintenance on carried structure, junk included",
	},
	{
		path: "energy.lifecycle.genome_replication_cost_per_unit",
		label: "Replication Cost / Unit",
		min: 0,
		max: 1,
		step: 0.001,
		defaultValue: 0.1,
		tooltip:
			"Per-birth multiplier on the parent's reproduce charge: 1 + rate \u00d7 units of genome_size() above the founder's 97",
	},
	{
		path: "energy.lifecycle.min_reproduce_energy",
		label: "Min Reproduce Energy",
		min: 0,
		max: 500,
		step: 0.5,
		defaultValue: 30.0,
		tooltip: "Minimum energy required for a creature to reproduce",
	},
	{
		path: "energy.lifecycle.default_offspring_energy",
		label: "Offspring Energy",
		min: 0,
		max: 500,
		step: 0.5,
		defaultValue: 100.0,
		tooltip: "Starting energy given to newborn creatures",
	},
];
