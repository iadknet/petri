import { FieldGroup } from "../shared/FieldGroup.tsx";
import { FieldRow } from "../shared/FieldRow.tsx";
import { ToggleRow } from "../shared/ToggleRow.tsx";
import { getByPath } from "../shared/pathUtils.ts";
import type { BooleanFieldDef, FieldDef, RuntimePanelProps } from "../shared/types.ts";

export const MUTATION_FIELDS: FieldDef[] = [
	{
		path: "mutation.mutation_probability",
		label: "Mutation Prob.",
		min: 0,
		max: 1,
		step: 0.001,
		testId: "config-field-mutation-mutation-probability",
		defaultValue: 0.303,
		tooltip: "Probability that a newborn genome undergoes mutation",
	},
	{
		path: "mutation.per_birth_mutation_events_min",
		label: "Min Events/Birth",
		min: 0,
		max: 20,
		step: 1,
		testId: "config-field-mutation-events-min",
		defaultValue: 1,
		tooltip: "Minimum number of mutation events per birth when mutation triggers",
	},
	{
		path: "mutation.per_birth_mutation_events_max",
		label: "Max Events/Birth",
		min: 0,
		max: 20,
		step: 1,
		testId: "config-field-mutation-events-max",
		defaultValue: 10,
		tooltip: "Maximum number of mutation events per birth when mutation triggers",
	},
	{
		path: "mutation.mesh_layer_probability",
		label: "Mesh Layer Prob.",
		min: 0,
		max: 1,
		step: 0.01,
		defaultValue: 0.2,
		tooltip: "Probability of adding a mesh layer during genome mutation",
	},
	{
		path: "mutation.complexity_cap",
		label: "Complexity Cap",
		min: 1,
		max: 5000,
		step: 1,
		defaultValue: 1200,
		tooltip: "Maximum genome complexity before complexity pressure suppresses growth mutations",
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
];

export const MUTATION_TOGGLES: BooleanFieldDef[] = [
	{
		path: "mutation.complexity_pressure_enabled",
		label: "Complexity Pressure",
		testId: "config-field-mutation-complexity-pressure-enabled",
		defaultValue: true,
		tooltip:
			"When enabled, genomes near the complexity cap are less likely to gain growth mutations",
	},
];

export const MUTATION_ALL_FIELDS: (FieldDef | BooleanFieldDef)[] = [
	...MUTATION_FIELDS,
	...MUTATION_TOGGLES,
];

export function MutationFieldGroup({
	localDraft,
	serverConfig,
	simState,
	updateDraft,
}: RuntimePanelProps) {
	return (
		<FieldGroup title="Mutation">
			{MUTATION_FIELDS.map((field) => (
				<FieldRow
					key={`runtime-${field.path}`}
					field={field}
					id={`runtime-${field.path.replaceAll(".", "-")}`}
					value={getByPath(localDraft, field.path) as number}
					serverValue={getByPath(serverConfig, field.path) as number}
					disabled={simState === "running"}
					onChange={updateDraft}
					testId={field.testId}
				/>
			))}
			{MUTATION_TOGGLES.map((field) => (
				<ToggleRow
					key={`runtime-${field.path}`}
					field={field}
					id={`runtime-${field.path.replaceAll(".", "-")}`}
					value={getByPath(localDraft, field.path) as boolean}
					serverValue={getByPath(serverConfig, field.path) as boolean}
					disabled={simState === "running"}
					onChange={updateDraft}
					testId={field.testId}
				/>
			))}
		</FieldGroup>
	);
}
