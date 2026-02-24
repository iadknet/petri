import { CollapsibleGroup } from "../shared/CollapsibleGroup.tsx";
import { FieldRow } from "../shared/FieldRow.tsx";
import { getByPath } from "../shared/pathUtils.ts";
import type { FieldDef, RuntimePanelProps } from "../shared/types.ts";

export const POPULATION_FIELDS: FieldDef[] = [
	{
		path: "population.max_creatures",
		label: "Max Creatures",
		min: 1,
		max: 100000,
		step: 100,
	},
];

export function PopulationSection({
	localDraft,
	serverConfig,
	simState,
	updateDraft,
}: RuntimePanelProps) {
	return (
		<CollapsibleGroup title="Population">
			{POPULATION_FIELDS.map((field) => (
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
