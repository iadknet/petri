import { FieldGroup } from "../shared/FieldGroup.tsx";
import { FieldRow } from "../shared/FieldRow.tsx";
import { getByPath } from "../shared/pathUtils.ts";
import type { FieldDef, RuntimePanelProps } from "../shared/types.ts";

export const ENERGY_COSTS_FIELDS: FieldDef[] = [
	{
		path: "energy.costs.move_cost",
		label: "Move Cost",
		min: 0,
		max: 10,
		step: 0.01,
		testId: "config-field-energy-costs-move-cost",
	},
	{
		path: "energy.costs.eat_cost",
		label: "Eat Cost",
		min: 0,
		max: 10,
		step: 0.01,
	},
	{
		path: "energy.costs.noop_cost",
		label: "Noop Cost",
		min: 0,
		max: 10,
		step: 0.01,
	},
	{
		path: "energy.costs.reproduce_cost",
		label: "Reproduce Cost",
		min: 0,
		max: 50,
		step: 0.1,
	},
	{
		path: "energy.costs.eat_reward_per_food",
		label: "Eat Reward",
		min: 0,
		max: 50,
		step: 0.1,
	},
];

export function EnergyCostsSection({
	localDraft,
	serverConfig,
	simState,
	updateDraft,
}: RuntimePanelProps) {
	return (
		<FieldGroup title="Energy > Costs">
			{ENERGY_COSTS_FIELDS.map((field) => (
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
