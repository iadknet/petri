import type { FieldDef } from "../shared/types.ts";

export const FOOD_PARAMETERS_FIELDS: FieldDef[] = [
	{
		path: "world.food.growth_rate",
		label: "Food Growth Rate",
		min: 0,
		max: 1,
		step: 0.001,
		defaultValue: 0.096,
	},
	{
		path: "world.food.spread_threshold_ratio",
		label: "Spread Threshold Ratio",
		min: 0,
		max: 1,
		step: 0.01,
		defaultValue: 0.8,
	},
	{
		path: "world.food.recovery_spawn_rate",
		label: "Recovery Spawn Rate",
		min: 0,
		max: 1,
		step: 0.01,
		defaultValue: 0.01,
	},
	{
		path: "world.food.recovery_floor_ratio",
		label: "Recovery Floor Ratio",
		min: 0,
		max: 1,
		step: 0.01,
		defaultValue: 0.01,
	},
	{
		path: "world.food.max_density",
		label: "Food Max Density",
		min: 0.1,
		max: 1,
		step: 0.01,
		defaultValue: 1.0,
	},
];
