import { CollapsibleGroup } from "../shared/CollapsibleGroup.tsx";
import { FieldRow } from "../shared/FieldRow.tsx";
import { getByPath } from "../shared/pathUtils.ts";
import type { FieldDef, RuntimePanelProps } from "../shared/types.ts";

export const MUTATION_FIELDS: FieldDef[] = [
	{
		path: "runtime.mutation.mutation_probability",
		label: "Mutation Prob.",
		min: 0,
		max: 1,
		step: 0.001,
	},
	{
		path: "runtime.mutation.per_birth_mutation_events_min",
		label: "Min Events/Birth",
		min: 0,
		max: 20,
		step: 1,
	},
	{
		path: "runtime.mutation.per_birth_mutation_events_max",
		label: "Max Events/Birth",
		min: 0,
		max: 20,
		step: 1,
	},
	{
		path: "runtime.mutation.operator_modifier_scale",
		label: "Modifier Scale",
		min: 0,
		max: 10,
		step: 0.1,
	},
	{
		path: "runtime.mutation.phenotype.channel_step",
		label: "Channel Step",
		min: 1,
		max: 50,
		step: 1,
	},
	{
		path: "runtime.mutation.phenotype.polarity_flip_chance",
		label: "Polarity Flip",
		min: 0,
		max: 1,
		step: 0.001,
	},
	{
		path: "runtime.mutation.phenotype.channel_weight_min",
		label: "Weight Min",
		min: 0,
		max: 1,
		step: 0.01,
	},
	{
		path: "runtime.mutation.phenotype.channel_weight_max",
		label: "Weight Max",
		min: 0,
		max: 10,
		step: 0.01,
	},
];

export function MutationSection({
	localDraft,
	serverConfig,
	simState,
	updateDraft,
}: RuntimePanelProps) {
	return (
		<CollapsibleGroup title="Mutation">
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
		</CollapsibleGroup>
	);
}
