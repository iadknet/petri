import { Section } from "../shared/Section.tsx";
import type { FieldDef, RuntimePanelProps } from "../shared/types.ts";
import { ENERGY_COSTS_FIELDS } from "./EnergyCostsSection.tsx";
import { ENERGY_LIFECYCLE_FIELDS } from "./EnergyLifecycleSection.tsx";
import { FOOD_PARAMETERS_FIELDS } from "./FoodParametersSection.tsx";
import { MUTATION_FIELDS } from "./MutationSection.tsx";
import { POPULATION_FIELDS } from "./PopulationSection.tsx";
import { RUNTIME_FIELDS } from "./RuntimeSection.tsx";
import { RuntimeFieldGroup } from "./RuntimeFieldGroup.tsx";

export const RUNTIME_PATCH_FIELDS: FieldDef[] = [
	...FOOD_PARAMETERS_FIELDS,
	...POPULATION_FIELDS,
	...ENERGY_LIFECYCLE_FIELDS,
	...ENERGY_COSTS_FIELDS,
	...RUNTIME_FIELDS,
	...MUTATION_FIELDS,
];

interface RuntimeConfigPanelProps {
	localDraft: RuntimePanelProps["localDraft"] | null;
	serverConfig: RuntimePanelProps["serverConfig"] | null;
	simState: RuntimePanelProps["simState"];
	updateDraft: RuntimePanelProps["updateDraft"];
}

export function RuntimeConfigPanel({
	localDraft,
	serverConfig,
	simState,
	updateDraft,
}: RuntimeConfigPanelProps) {
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
				<>
					<RuntimeFieldGroup title="Food Parameters" fields={FOOD_PARAMETERS_FIELDS} localDraft={localDraft} serverConfig={serverConfig} simState={simState} updateDraft={updateDraft} />
					<RuntimeFieldGroup title="Population" fields={POPULATION_FIELDS} localDraft={localDraft} serverConfig={serverConfig} simState={simState} updateDraft={updateDraft} />
					<RuntimeFieldGroup title="Energy > Lifecycle" fields={ENERGY_LIFECYCLE_FIELDS} localDraft={localDraft} serverConfig={serverConfig} simState={simState} updateDraft={updateDraft} />
					<RuntimeFieldGroup title="Energy > Costs" fields={ENERGY_COSTS_FIELDS} localDraft={localDraft} serverConfig={serverConfig} simState={simState} updateDraft={updateDraft} />
					<RuntimeFieldGroup title="Runtime" fields={RUNTIME_FIELDS} localDraft={localDraft} serverConfig={serverConfig} simState={simState} updateDraft={updateDraft} />
					<RuntimeFieldGroup title="Mutation" fields={MUTATION_FIELDS} localDraft={localDraft} serverConfig={serverConfig} simState={simState} updateDraft={updateDraft} />
				</>
			)}
		</Section>
	);
}
