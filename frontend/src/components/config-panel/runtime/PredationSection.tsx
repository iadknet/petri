import type { FieldDef } from "../shared/types.ts";

export const PREDATION_FIELDS: FieldDef[] = [
	{
		path: "predation.steal_cost_rate",
		label: "Steal Cost Rate",
		min: 0,
		max: 1,
		step: 0.01,
		testId: "config-field-predation-steal-cost-rate",
		defaultValue: 0.2,
		tooltip: "Fraction of attempted steal amount paid as attacker energy cost",
	},
	{
		path: "predation.kill_complexity_bonus_multiplier",
		label: "Kill Complexity Bonus",
		min: 0,
		max: 1,
		step: 0.01,
		testId: "config-field-predation-kill-complexity-bonus",
		defaultValue: 0.05,
		tooltip: "Energy bonus per unit of victim genome complexity on kill",
	},
];
