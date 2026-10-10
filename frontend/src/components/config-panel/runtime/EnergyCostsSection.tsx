import type { FieldDef } from "../shared/types.ts";

/** T16.F02: the shared pricing rule of every action's effort. */
export const EFFORT_RULE =
	"Charged on every attempt, whatever the outcome, and never below Noop Cost";

export const ENERGY_COSTS_FIELDS: FieldDef[] = [
	{
		path: "energy.costs.eat_reward_per_food",
		label: "Eat Reward",
		min: 0,
		max: 100,
		step: 0.1,
		defaultValue: 5.0,
		testId: "config-field-energy-costs-eat-reward-per-food",
		tooltip: "Energy gained per unit of any ordinary food consumed",
	},
	{
		path: "energy.costs.move_cost",
		label: "Move Cost",
		min: 0,
		max: 10,
		step: 0.01,
		testId: "config-field-energy-costs-move-cost",
		defaultValue: 1.6,
		tooltip: `Energy a move costs. ${EFFORT_RULE}; a blocked move pays it and nothing more`,
	},
	{
		path: "energy.costs.eat_cost",
		label: "Eat Cost",
		min: 0,
		max: 10,
		step: 0.01,
		defaultValue: 0.05,
		testId: "config-field-energy-costs-eat-cost",
		tooltip: `Energy an eat costs, paid after the food is credited. ${EFFORT_RULE}; an eat on an empty cell pays it and nothing more`,
	},
	{
		path: "energy.costs.noop_cost",
		label: "Noop Cost",
		min: 0.01,
		max: 10,
		step: 0.01,
		defaultValue: 0.4,
		testId: "config-field-energy-costs-noop-cost",
		tooltip: `Energy a no-op costs, the floor of every action's effort; must be above 0. ${EFFORT_RULE}`,
	},
	{
		path: "energy.costs.reproduce_cost",
		label: "Reproduce Cost",
		min: 0,
		max: 50,
		step: 0.1,
		defaultValue: 0.1,
		tooltip:
			"The replication cost, paid by the parent only on a conception, after every reproduction gate passes",
	},
	{
		path: "energy.costs.reproduce_attempt_cost",
		label: "Reproduce Attempt Cost",
		min: 0,
		max: 10,
		step: 0.01,
		defaultValue: 1.0,
		testId: "config-field-energy-costs-reproduce-attempt-cost",
		tooltip: `Energy a reproduce attempt costs, paid before any gate. ${EFFORT_RULE}; a rejected attempt pays it and nothing more`,
	},
];
