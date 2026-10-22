import type { FieldDef } from "../shared/types.ts";
import { EFFORT_RULE } from "./EnergyCostsSection.tsx";

/** The exchange acts' effort rate and heritable share range (T27.F03). */
export const EXCHANGE_FIELDS: FieldDef[] = [
	{
		path: "exchange.effort_rate",
		label: "Exchange Effort Rate",
		min: 0,
		max: 1,
		step: 0.01,
		testId: "config-field-exchange-effort-rate",
		defaultValue: 0.05,
		tooltip: `Fraction of the actor's own energy, times the act's intensity, a steal pays before the target resolves. ${EFFORT_RULE}; a steal with no victim or nothing to take pays it and nothing more`,
	},
	{
		path: "exchange.share_min",
		label: "Share Minimum",
		min: 0.001,
		max: 1,
		step: 0.001,
		testId: "config-field-exchange-share-min",
		defaultValue: 0.01,
		tooltip: "Lowest heritable steal share a newborn can carry; the highest is 1",
	},
	{
		path: "exchange.founder_share",
		label: "Founder Share",
		min: 0.001,
		max: 1,
		step: 0.01,
		testId: "config-field-exchange-founder-share",
		defaultValue: 0.2,
		tooltip:
			"Every founder's heritable steal share: the part of a neighbour's energy a steal takes at effort 1",
	},
];

export const PREDATION_FIELDS: FieldDef[] = [
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
