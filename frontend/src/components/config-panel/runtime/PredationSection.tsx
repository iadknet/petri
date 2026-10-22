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
		tooltip: `Fraction of the requested steal amount the attacker pays, before the target resolves. ${EFFORT_RULE}; a steal with no victim pays it and nothing more`,
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
