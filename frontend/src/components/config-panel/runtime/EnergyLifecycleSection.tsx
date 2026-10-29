import type { AnyFieldDef, EnumFieldDef, FieldDef } from "../shared/types.ts";

export const ENERGY_LIFECYCLE_FIELDS: FieldDef[] = [
	{
		path: "energy.lifecycle.max_energy",
		label: "Max Capacity",
		min: 1,
		max: 1000,
		step: 1,
		defaultValue: 800.0,
		tooltip:
			"Upper bound of the inherited energy capacity; each creature's own capacity caps its energy",
	},
	{
		path: "energy.lifecycle.energy_decay_per_tick",
		label: "Energy Decay / Tick",
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
			"Energy charged each tick per genome unit (each node, instruction, input, ungrouped edge; an edge group is one unit, its members free), junk included",
	},
	{
		path: "energy.lifecycle.genome_replication_cost_per_unit",
		label: "Replication Cost / Unit",
		min: 0,
		max: 1,
		step: 0.001,
		defaultValue: 0.1,
		tooltip:
			"Raises the parent's reproduce charge for big genomes: the charge is multiplied by 1 + rate \u00d7 (genome units above the founder's 85)",
	},
	{
		path: "energy.lifecycle.gestation_ticks_per_founder_genome",
		label: "Gestation Ticks / Founder Genome",
		min: 0,
		max: 10,
		step: 0.1,
		defaultValue: 1.0,
		tooltip:
			"Gestation ticks per founder-sized genome (85 units); bigger genomes gestate longer, rounded up. The parent cannot act until the birth, and its death loses the child. 0 delivers at once",
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
		path: "energy.lifecycle.min_reproduce_age",
		label: "Min Reproduce Age",
		min: 0,
		max: 1000000,
		step: 1,
		defaultValue: 20,
		sliderMax: 1000,
		testId: "config-field-energy-lifecycle-min-reproduce-age",
		tooltip: "Youngest age, in ticks, at which a creature may reproduce",
	},
	{
		path: "energy.lifecycle.age_reference_ticks",
		label: "Age Input Span",
		min: 1,
		max: 1000000,
		step: 1,
		defaultValue: 500,
		sliderMax: 5000,
		testId: "config-field-energy-lifecycle-age-reference-ticks",
		tooltip: "Ticks over which the brain's Age input rises from 0 to 1, then stays at 1",
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

/** The heritable energy capacity (T26.F02): range, cost rules and sensor scale. */
export const ENERGY_CAPACITY_NUMBER_FIELDS: FieldDef[] = [
	{
		path: "energy.capacity.reference",
		label: "Reference Capacity",
		min: 1,
		max: 1000,
		step: 1,
		defaultValue: 200.0,
		tooltip:
			"The neutral capacity: a body here pays the base move cost and no upkeep, and the energy sensors read on this scale",
	},
	{
		path: "energy.capacity.min",
		label: "Min Capacity",
		min: 1,
		max: 1000,
		step: 1,
		defaultValue: 50.0,
		tooltip: "Lower bound of the inherited capacity, at most the reference",
	},
	{
		path: "energy.capacity.founder",
		label: "Founder Capacity",
		min: 1,
		max: 1000,
		step: 1,
		defaultValue: 200.0,
		tooltip: "Every founder's capacity at seeding; takes effect on restart",
	},
	{
		path: "energy.capacity.move_cost_slope",
		label: "Move Cost Slope",
		min: 0,
		max: 1,
		step: 0.01,
		defaultValue: 1.0,
		tooltip:
			"How much a body's size scales its move cost: the base is multiplied by (1 \u2212 slope) + slope \u00d7 capacity / reference",
	},
	{
		path: "energy.capacity.upkeep_per_unit",
		label: "Upkeep / Capacity Unit",
		min: 0,
		max: 0.01,
		step: 0.0001,
		defaultValue: 0.002,
		slider: false,
		tooltip:
			"Energy charged each tick per capacity unit above the reference, and refunded against the tick's other upkeep below it",
	},
];

export const ENERGY_CAPACITY_ENUMS: EnumFieldDef[] = [
	{
		path: "energy.capacity.sensor_scale",
		label: "Energy Sensor Scale",
		testId: "config-field-energy-capacity-sensor-scale",
		defaultValue: "reference",
		options: [
			{ value: "reference", label: "Reference capacity" },
			{ value: "own", label: "Own capacity" },
		],
		tooltip:
			"What the energy sensors divide by: the reference capacity for every creature, or each creature's own capacity",
	},
];

export const ENERGY_CAPACITY_FIELDS: AnyFieldDef[] = [
	...ENERGY_CAPACITY_NUMBER_FIELDS,
	...ENERGY_CAPACITY_ENUMS,
];
