import type { FieldDef } from "../shared/types.ts";

export const MUTATION_FIELDS: FieldDef[] = [
	{
		path: "mutation.mutation_probability",
		label: "Mutation Prob.",
		min: 0,
		max: 1,
		step: 0.001,
		testId: "config-field-mutation-mutation-probability",
	},
	{
		path: "mutation.per_birth_mutation_events_min",
		label: "Min Events/Birth",
		min: 0,
		max: 20,
		step: 1,
		testId: "config-field-mutation-events-min",
	},
	{
		path: "mutation.per_birth_mutation_events_max",
		label: "Max Events/Birth",
		min: 0,
		max: 20,
		step: 1,
		testId: "config-field-mutation-events-max",
	},
	{
		path: "mutation.phenotype.channel_step",
		label: "Channel Step",
		min: 1,
		max: 50,
		step: 1,
		testId: "config-field-mutation-channel-step",
	},
	{
		path: "mutation.phenotype.polarity_flip_chance",
		label: "Polarity Flip",
		min: 0,
		max: 1,
		step: 0.001,
		testId: "config-field-mutation-polarity-flip",
	},
	{
		path: "mutation.phenotype.channel_change_chance",
		label: "Channel Switch",
		min: 0,
		max: 1,
		step: 0.001,
		testId: "config-field-mutation-channel-change",
	},
];
