import type { FieldDef } from "../shared/types.ts";
import { EFFORT_RULE } from "./EnergyCostsSection.tsx";

export const PREDATION_FIELDS: FieldDef[] = [
	{
		path: "predation.steal_cost_rate",
		label: "Steal Cost Rate",
		min: 0,
		max: 1,
		step: 0.01,
		testId: "config-field-predation-steal-cost-rate",
		defaultValue: 0.05,
		tooltip: `Fraction of the exchange bite the attacker pays as effort, before the target resolves. ${EFFORT_RULE}; a steal with no victim, or one with nothing to take, pays it and nothing more`,
	},
	{
		path: "exchange.bite",
		label: "Exchange Bite",
		min: 0.01,
		max: 1000,
		step: 0.1,
		sliderMax: 50,
		testId: "config-field-exchange-bite",
		defaultValue: 8.0,
		tooltip: "Energy one steal takes from the neighbour, at most what it holds",
	},
	{
		path: "predation.kill_complexity_bonus_multiplier",
		label: "Kill Complexity Bonus",
		min: 0,
		max: 100,
		step: 0.01,
		sliderMax: 1,
		testId: "config-field-predation-kill-complexity-bonus",
		defaultValue: 0.0,
		tooltip: "Energy bonus per unit of victim genome complexity on kill",
	},
];
