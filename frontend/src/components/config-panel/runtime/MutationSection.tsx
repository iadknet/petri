import type { AnyFieldDef, BooleanFieldDef, EnumFieldDef, FieldDef } from "../shared/types.ts";

export const MUTATION_FIELDS: FieldDef[] = [
	{
		path: "mutation.per_unit_rate",
		label: "Rate per Genome Unit",
		min: 0,
		max: 0.1,
		step: 0.0000625,
		testId: "config-field-mutation-per-unit-rate",
		defaultValue: 0.0060625,
		tooltip:
			"Chance that each genome unit requests one mutation event at birth; the founder's 80 units expect about 0.49 events. This field caps at 0.1; the runtime accepts up to 1.",
	},
	{
		path: "mutation.mesh_layer_probability",
		label: "Mesh Mutation Chance",
		min: 0,
		max: 1,
		step: 0.01,
		testId: "config-field-mutation-mesh-layer-probability",
		defaultValue: 0.2,
		tooltip:
			"Chance a mutation rewires the mesh (adds, removes, copies or reroutes nodes) instead of changing the inside of one node",
	},
	{
		path: "mutation.large_copy_weight_percent",
		label: "Large Copy Weight %",
		min: 0,
		max: 100,
		step: 1,
		testId: "config-field-mutation-large-copy-weight-percent",
		defaultValue: 25,
		tooltip:
			"Relative weight of copying a node or mesh slice: 25 quarters the base weight, 100 restores it, and 0 disables copying. This is not a per-birth probability; other mutation weights stay unchanged.",
	},
	{
		path: "mutation.genome_size_cap",
		label: "Genome Size Cap",
		min: 0,
		max: 100000,
		step: 1,
		testId: "config-field-mutation-genome-size-cap",
		defaultValue: 1200,
		sliderMax: 5000,
		tooltip:
			"Maximum genome size before size pressure suppresses growth mutations; 0 removes the cap",
	},
	{
		path: "mutation.phenotype.channel_step",
		label: "Channel Step",
		min: 1,
		max: 50,
		step: 1,
		testId: "config-field-mutation-channel-step",
		defaultValue: 1,
		tooltip: "RGB step size applied to the active color channel at each reproduction",
	},
	{
		path: "mutation.phenotype.polarity_flip_chance",
		label: "Polarity Flip",
		min: 0,
		max: 1,
		step: 0.0001,
		sliderMax: 0.01,
		testId: "config-field-mutation-polarity-flip",
		defaultValue: 0.0002,
		tooltip: "Probability of reversing the drift direction of the active color channel",
	},
	{
		path: "mutation.phenotype.channel_change_chance",
		label: "Channel Switch",
		min: 0,
		max: 1,
		step: 0.0001,
		sliderMax: 0.05,
		testId: "config-field-mutation-channel-change",
		defaultValue: 0.001,
		tooltip: "Probability of switching to a different active color channel (R/G/B)",
	},
	{
		path: "mutation.executed_bias",
		label: "Target Recently Run",
		min: 0,
		max: 1,
		step: 0.01,
		testId: "config-field-mutation-executed-bias",
		defaultValue: 0.9,
		tooltip:
			"Chance to target a node the parent's brain ran recently; the remainder draws from every eligible node",
	},
	{
		path: "mutation.executed_window_ticks",
		label: "Recently Run Window",
		min: 1,
		max: 10000,
		step: 1,
		testId: "config-field-mutation-executed-window-ticks",
		defaultValue: 100,
		tooltip: "How many ticks back a node run still counts as recent for Target Recently Run",
	},
	{
		path: "mutation.reachable_bias.topology",
		label: "Reachable Bias: Mesh",
		min: 0,
		max: 1,
		step: 0.01,
		testId: "config-field-mutation-reachable-bias-topology",
		defaultValue: 0,
		tooltip:
			"Chance a mesh mutation targets a node the brain can reach from its entry node; 0 picks evenly among all eligible nodes",
	},
	{
		path: "mutation.reachable_bias.vm",
		label: "Reachable Bias: VM",
		min: 0,
		max: 1,
		step: 0.01,
		testId: "config-field-mutation-reachable-bias-vm",
		defaultValue: 0,
		tooltip:
			"Chance a VM mutation targets a node the brain can reach from its entry node; 0 picks evenly among all eligible nodes",
	},
	{
		path: "mutation.reachable_bias.graph",
		label: "Reachable Bias: Graph",
		min: 0,
		max: 1,
		step: 0.01,
		testId: "config-field-mutation-reachable-bias-graph",
		defaultValue: 0,
		tooltip:
			"Chance a graph mutation targets a node the brain can reach from its entry node; 0 picks evenly among all eligible nodes",
	},
	{
		path: "mutation.reachable_bias.input_ref",
		label: "Reachable Bias: Inputs",
		min: 0,
		max: 1,
		step: 0.01,
		testId: "config-field-mutation-reachable-bias-input-ref",
		defaultValue: 0,
		tooltip:
			"Chance an input mutation targets a node the brain can reach from its entry node; 0 picks evenly among all eligible nodes",
	},
];

export const MUTATION_TOGGLES: BooleanFieldDef[] = [
	{
		path: "mutation.structured_heritable_refinement",
		label: "Structured Refinement",
		testId: "config-field-mutation-structured-heritable-refinement",
		defaultValue: true,
		tooltip: "Bounded semantic input-to-vote refinement at birth",
	},
	{
		path: "mutation.genome_size_pressure_enabled",
		label: "Genome Size Pressure",
		testId: "config-field-mutation-genome-size-pressure-enabled",
		defaultValue: false,
		tooltip: "When enabled, genomes near the size cap are less likely to gain growth mutations",
	},
];

/** Mutation settings that pick one mode rather than a number. */
export const MUTATION_ENUMS: EnumFieldDef[] = [
	{
		path: "mutation.neutral_input_recruitment",
		label: "Neutral Input Recruitment",
		testId: "config-field-mutation-neutral-input-recruitment",
		defaultValue: "WholeFamily",
		options: [
			{ value: "Off", label: "Off" },
			{ value: "SingleChannel", label: "Single channel" },
			{ value: "WholeFamily", label: "Whole family" },
		],
		tooltip:
			"Grows new input-to-vote links that start silent (weight 0), one sensor channel or a whole sensor family at a time",
	},
];

export const MUTATION_ALL_FIELDS: AnyFieldDef[] = [
	...MUTATION_FIELDS,
	...MUTATION_TOGGLES,
	...MUTATION_ENUMS,
];
