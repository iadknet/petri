import { CollapsibleGroup } from "../shared/CollapsibleGroup.tsx";
import { FieldRow } from "../shared/FieldRow.tsx";
import { getByPath } from "../shared/pathUtils.ts";
import type { FieldDef, RuntimePanelProps } from "../shared/types.ts";

export const ENERGY_LIFECYCLE_FIELDS: FieldDef[] = [
	{
		path: "energy.lifecycle.initial_energy",
		label: "Initial Energy",
		min: 0,
		max: 500,
		step: 0.5,
	},
	{
		path: "energy.lifecycle.max_energy",
		label: "Max Energy",
		min: 1,
		max: 1000,
		step: 1,
	},
	{
		path: "energy.lifecycle.energy_decay_per_tick",
		label: "Decay / Tick",
		min: 0,
		max: 10,
		step: 0.01,
	},
	{
		path: "energy.lifecycle.min_reproduce_energy",
		label: "Min Reproduce Energy",
		min: 0,
		max: 500,
		step: 0.5,
	},
	{
		path: "energy.lifecycle.default_offspring_energy",
		label: "Offspring Energy",
		min: 0,
		max: 500,
		step: 0.5,
	},
];

export function EnergyLifecycleSection({
	localDraft,
	serverConfig,
	simState,
	updateDraft,
}: RuntimePanelProps) {
	return (
		<CollapsibleGroup title="Energy > Lifecycle">
			{ENERGY_LIFECYCLE_FIELDS.map((field) => (
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
