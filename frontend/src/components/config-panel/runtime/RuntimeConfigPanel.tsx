import { Section } from "../shared/Section.tsx";
import type { BooleanFieldDef, FieldDef, RuntimePanelProps } from "../shared/types.ts";
import { ACTION_LOG_FIELDS } from "./ActionLogSection.tsx";
import { AGE_COST_ALL_FIELDS, AGE_COST_FIELDS, AGE_COST_TOGGLES } from "./AgeEnergyCostSection.tsx";
import {
	COMPLEXITY_COST_ALL_FIELDS,
	COMPLEXITY_COST_FIELDS,
	COMPLEXITY_COST_TOGGLES,
} from "./ComplexityEnergyCostSection.tsx";
import { ENERGY_COSTS_FIELDS } from "./EnergyCostsSection.tsx";
import { ENERGY_LIFECYCLE_FIELDS } from "./EnergyLifecycleSection.tsx";
import {
	FOOD_OCCUPANCY_DEPLETION_FIELDS,
	FOOD_OCCUPANCY_DEPLETION_TOGGLES,
	FOOD_PARAMETERS_FIELDS,
} from "./FoodParametersSection.tsx";
import { MUTATION_ALL_FIELDS, MUTATION_FIELDS, MUTATION_TOGGLES } from "./MutationSection.tsx";
import { POPULATION_FIELDS } from "./PopulationSection.tsx";
import { PREDATION_FIELDS } from "./PredationSection.tsx";
import { RuntimeFieldGroup } from "./RuntimeFieldGroup.tsx";
import { RUNTIME_FIELDS } from "./RuntimeSection.tsx";
import { SHARED_MEMORY_FIELDS } from "./SharedMemorySection.tsx";

export const RUNTIME_PATCH_FIELDS: (FieldDef | BooleanFieldDef)[] = [
	...FOOD_OCCUPANCY_DEPLETION_TOGGLES,
	...FOOD_OCCUPANCY_DEPLETION_FIELDS,
	...FOOD_PARAMETERS_FIELDS,
	...POPULATION_FIELDS,
	...ENERGY_LIFECYCLE_FIELDS,
	...ENERGY_COSTS_FIELDS,
	...COMPLEXITY_COST_ALL_FIELDS,
	...AGE_COST_ALL_FIELDS,
	...RUNTIME_FIELDS,
	...MUTATION_ALL_FIELDS,
	...PREDATION_FIELDS,
	...ACTION_LOG_FIELDS,
	...SHARED_MEMORY_FIELDS,
];

/** Groups rendered in panel order, each with its numeric fields and toggles. */
const RUNTIME_GROUPS: { title: string; fields: FieldDef[]; toggles?: BooleanFieldDef[] }[] = [
	{
		title: "Food > Occupancy Depletion",
		fields: FOOD_OCCUPANCY_DEPLETION_FIELDS,
		toggles: FOOD_OCCUPANCY_DEPLETION_TOGGLES,
	},
	{ title: "Food Parameters", fields: FOOD_PARAMETERS_FIELDS },
	{ title: "Population", fields: POPULATION_FIELDS },
	{ title: "Energy > Lifecycle", fields: ENERGY_LIFECYCLE_FIELDS },
	{ title: "Energy > Costs", fields: ENERGY_COSTS_FIELDS },
	{
		title: "Energy > Complexity Cost",
		fields: COMPLEXITY_COST_FIELDS,
		toggles: COMPLEXITY_COST_TOGGLES,
	},
	{ title: "Energy > Age Cost", fields: AGE_COST_FIELDS, toggles: AGE_COST_TOGGLES },
	{ title: "Runtime", fields: RUNTIME_FIELDS },
	{ title: "Mutation", fields: MUTATION_FIELDS, toggles: MUTATION_TOGGLES },
	{ title: "Predation", fields: PREDATION_FIELDS },
	{ title: "Action Log", fields: ACTION_LOG_FIELDS },
	{ title: "Shared Memory", fields: SHARED_MEMORY_FIELDS },
];

interface RuntimeConfigPanelProps extends Omit<RuntimePanelProps, "localDraft" | "serverConfig"> {
	localDraft: RuntimePanelProps["localDraft"] | null;
	serverConfig: RuntimePanelProps["serverConfig"] | null;
}

export function RuntimeConfigPanel({ localDraft, serverConfig, ...rest }: RuntimeConfigPanelProps) {
	return (
		<Section
			title="Runtime (Live) Config"
			description="These settings apply to the current simulation when allowed by lifecycle state."
			collapsible
			sectionClassName="bg-sky-950/10 border-l-2 border-l-sky-500"
		>
			{!localDraft || !serverConfig ? (
				<div className="p-3 text-sm text-slate-500">
					Runtime config unavailable. Restart to initialize the simulation.
				</div>
			) : (
				RUNTIME_GROUPS.map((group) => (
					<RuntimeFieldGroup
						key={group.title}
						{...group}
						{...rest}
						localDraft={localDraft}
						serverConfig={serverConfig}
					/>
				))
			)}
		</Section>
	);
}
