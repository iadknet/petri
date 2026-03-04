import { FieldGroup } from "../shared/FieldGroup.tsx";
import { FieldRow } from "../shared/FieldRow.tsx";
import { ToggleRow } from "../shared/ToggleRow.tsx";
import { getByPath } from "../shared/pathUtils.ts";
import type { BooleanFieldDef, FieldDef, RuntimePanelProps } from "../shared/types.ts";

export const COMPLEXITY_COST_FIELDS: FieldDef[] = [
	{
		path: "energy.complexity_cost.threshold",
		label: "Threshold",
		min: 0,
		max: 500,
		step: 10,
		testId: "config-field-complexity-cost-threshold",
		defaultValue: 50,
		tooltip: "Genome complexity below which there is no energy cost penalty",
	},
	{
		path: "energy.complexity_cost.scaling_factor",
		label: "Scaling Factor",
		min: 0,
		max: 0.02,
		step: 0.001,
		testId: "config-field-complexity-cost-scaling-factor",
		defaultValue: 0.002,
		tooltip:
			"Energy cost multiplier increase per unit of complexity above threshold",
	},
];

export const COMPLEXITY_COST_TOGGLES: BooleanFieldDef[] = [
	{
		path: "energy.complexity_cost.enabled",
		label: "Complexity Cost",
		testId: "config-field-complexity-cost-enabled",
		defaultValue: true,
		tooltip:
			"When enabled, creatures with complex genomes pay higher energy costs for all actions",
	},
];

export const COMPLEXITY_COST_ALL_FIELDS: (FieldDef | BooleanFieldDef)[] = [
	...COMPLEXITY_COST_FIELDS,
	...COMPLEXITY_COST_TOGGLES,
];

export function ComplexityEnergyCostFieldGroup({
	localDraft,
	serverConfig,
	simState,
	updateDraft,
}: RuntimePanelProps) {
	return (
		<FieldGroup title="Energy > Complexity Cost">
			{COMPLEXITY_COST_TOGGLES.map((field) => (
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
			{COMPLEXITY_COST_FIELDS.map((field) => (
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
		</FieldGroup>
	);
}
