import type { BooleanFieldDef, FieldDef } from "../shared/types.ts";

export const FOOD_PARAMETERS_FIELDS: FieldDef[] = [
	{
		path: "world.food.shared.growth_rate",
		label: "Food Growth Rate",
		min: 0,
		max: 1,
		step: 0.001,
		defaultValue: 0.09,
		tooltip: "Rate at which existing food cells regenerate density each tick",
	},
	{
		path: "world.food.shared.spread_threshold_ratio",
		label: "Spread Threshold Ratio",
		min: 0,
		max: 1,
		step: 0.01,
		defaultValue: 0.8,
		tooltip: "Minimum neighbor density ratio to trigger food spread to empty cells",
	},
	{
		path: "world.food.shared.spread_density_ratio",
		label: "Spread Density Ratio",
		min: 0,
		max: 1,
		step: 0.01,
		defaultValue: 0.25,
		tooltip: "Fraction of growth delta deposited to neighbor during spread",
	},
	{
		path: "world.food.shared.recovery_spawn_rate",
		label: "Recovery Spawn Rate",
		min: 0,
		max: 1,
		step: 0.01,
		defaultValue: 0.01,
		tooltip: "Probability of spontaneous food spawn on empty cells each tick",
	},
	{
		path: "world.food.shared.recovery_floor_ratio",
		label: "Recovery Floor Ratio",
		min: 0,
		max: 1,
		step: 0.01,
		defaultValue: 0.01,
		tooltip: "Minimum population-to-capacity ratio below which recovery spawning activates",
	},
	{
		path: "world.food.shared.max_density",
		label: "Food Max Density",
		min: 0.1,
		max: 1,
		step: 0.01,
		defaultValue: 1.0,
		tooltip: "Maximum food density per cell (0-1 scale)",
	},
];

export const FOOD_OCCUPANCY_DEPLETION_TOGGLES: BooleanFieldDef[] = [
	{
		path: "world.food.shared.occupancy_depletion.enabled",
		label: "Occupancy Depletion Enabled",
		defaultValue: true,
		tooltip: "Toggle occupancy-driven food suppression during live food growth",
		testId: "config-field-food-occupancy-depletion-enabled",
	},
];

export const FOOD_OCCUPANCY_DEPLETION_FIELDS: FieldDef[] = [
	{
		path: "world.food.shared.occupancy_depletion.deposit_per_occupied_tick",
		label: "Occupancy Depletion Rate",
		min: 0,
		max: 1,
		step: 0.001,
		defaultValue: 0.08,
		tooltip: "Fraction of depletion deposited into each occupied cell per tick",
		testId: "config-field-food-occupancy-depletion-deposit-per-occupied-tick",
	},
];

export const FOOD_GRAZING_TOGGLES: BooleanFieldDef[] = [
	{
		path: "world.food.shared.grazing.enabled",
		label: "Grazing Enabled",
		defaultValue: true,
		tooltip:
			"Toggle the per-cell grazing fertility modifier: each bite slows regrowth where it landed until the cell rests",
		testId: "config-field-food-grazing-enabled",
	},
];

export const FOOD_GRAZING_FIELDS: FieldDef[] = [
	{
		path: "world.food.shared.grazing.factor",
		label: "Grazing Bite Factor",
		min: 0,
		max: 1,
		step: 0.01,
		defaultValue: 0.5,
		tooltip: "Multiplier a consuming bite applies to the cell's fertility modifier",
		testId: "config-field-food-grazing-factor",
	},
	{
		path: "world.food.shared.grazing.floor",
		label: "Grazing Floor",
		min: 0,
		max: 1,
		step: 0.01,
		defaultValue: 0.05,
		tooltip: "Lowest fertility modifier repeated bites can drive a cell to",
		testId: "config-field-food-grazing-floor",
	},
	{
		path: "world.food.shared.grazing.recovery_ticks",
		label: "Grazing Recovery Ticks",
		min: 1,
		max: 10000,
		step: 1,
		defaultValue: 1000,
		tooltip: "Ticks a fully floored cell needs to recover its fertility modifier from 0 to 1",
		testId: "config-field-food-grazing-recovery-ticks",
	},
];
