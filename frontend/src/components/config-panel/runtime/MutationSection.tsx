import type { BooleanFieldDef, FieldDef } from "../shared/types.ts";

export const MUTATION_FIELDS: FieldDef[] = [
	{
		path: "mutation.per_unit_rate",
		label: "Rate / Unit",
		min: 0,
		max: 0.1,
		step: 0.0001,
		testId: "config-field-mutation-per-unit-rate",
		defaultValue: 0.005,
		tooltip:
			"Chance that each genome unit requests one mutation event at birth; the founder's 97 units expect about 0.49 events. This field caps at 0.1; the runtime accepts up to 1.",
	},
	{
		path: "mutation.mesh_layer_probability",
		label: "Mesh Layer Prob.",
		min: 0,
		max: 1,
		step: 0.01,
		testId: "config-field-mutation-mesh-layer-probability",
		defaultValue: 0.2,
		tooltip: "Probability of selecting a topology mutation instead of a node-internal mutation",
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
		min: 1,
		max: 5000,
		step: 1,
		testId: "config-field-mutation-genome-size-cap",
		defaultValue: 1200,
		tooltip: "Maximum genome size before size pressure suppresses growth mutations",
	},
	{
		path: "mutation.phenotype.channel_step",
		label: "Channel Step",
		min: 1,
		max: 50,
		step: 1,
		testId: "config-field-mutation-channel-step",
		defaultValue: 1,
		tooltip: "RGB step size applied per tick to the active color channel",
	},
	{
		path: "mutation.phenotype.polarity_flip_chance",
		label: "Polarity Flip",
		min: 0,
		max: 1,
		step: 0.001,
		testId: "config-field-mutation-polarity-flip",
		defaultValue: 0.0002,
		tooltip: "Probability of reversing the drift direction of the active color channel",
	},
	{
		path: "mutation.phenotype.channel_change_chance",
		label: "Channel Switch",
		min: 0,
		max: 1,
		step: 0.001,
		testId: "config-field-mutation-channel-change",
		defaultValue: 0.001,
		tooltip: "Probability of switching to a different active color channel (R/G/B)",
	},
	{
		path: "mutation.executed_bias",
		label: "Bias: Executed",
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
		label: "Executed Window",
		min: 1,
		max: 10000,
		step: 1,
		testId: "config-field-mutation-executed-window-ticks",
		defaultValue: 100,
		tooltip: "How many ticks back a node dispatch still counts as recently executed",
	},
	{
		path: "mutation.reachable_bias.topology",
		label: "Bias: Topology",
		min: 0,
		max: 1,
		step: 0.01,
		testId: "config-field-mutation-reachable-bias-topology",
		defaultValue: 0,
		tooltip:
			"Chance to prefer reachable nodes; zero selects uniformly among eligible nodes for topology mutations",
	},
	{
		path: "mutation.reachable_bias.vm",
		label: "Bias: VM",
		min: 0,
		max: 1,
		step: 0.01,
		testId: "config-field-mutation-reachable-bias-vm",
		defaultValue: 0,
		tooltip:
			"Chance to prefer reachable nodes; zero selects uniformly among eligible nodes for VM mutations",
	},
	{
		path: "mutation.reachable_bias.graph",
		label: "Bias: Graph",
		min: 0,
		max: 1,
		step: 0.01,
		testId: "config-field-mutation-reachable-bias-graph",
		defaultValue: 0,
		tooltip:
			"Chance to prefer reachable nodes; zero selects uniformly among eligible nodes for graph mutations",
	},
	{
		path: "mutation.reachable_bias.input_ref",
		label: "Bias: Input Ref",
		min: 0,
		max: 1,
		step: 0.01,
		testId: "config-field-mutation-reachable-bias-input-ref",
		defaultValue: 0,
		tooltip:
			"Chance to prefer reachable nodes; zero selects uniformly among eligible nodes for input reference mutations",
	},
];

export const MUTATION_TOGGLES: BooleanFieldDef[] = [
	{
		path: "mutation.genome_size_pressure_enabled",
		label: "Genome Size Pressure",
		testId: "config-field-mutation-genome-size-pressure-enabled",
		defaultValue: false,
		tooltip: "When enabled, genomes near the size cap are less likely to gain growth mutations",
	},
];

export const MUTATION_ALL_FIELDS: (FieldDef | BooleanFieldDef)[] = [
	...MUTATION_FIELDS,
	...MUTATION_TOGGLES,
];
