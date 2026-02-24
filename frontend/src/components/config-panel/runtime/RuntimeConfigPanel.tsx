import { Section } from "../shared/Section.tsx";
import type { FieldDef, RuntimePanelProps } from "../shared/types.ts";
import { ENERGY_COSTS_FIELDS, EnergyCostsSection } from "./EnergyCostsSection.tsx";
import { ENERGY_LIFECYCLE_FIELDS, EnergyLifecycleSection } from "./EnergyLifecycleSection.tsx";
import { FOOD_PARAMETERS_FIELDS, FoodParametersSection } from "./FoodParametersSection.tsx";
import { MUTATION_FIELDS, MutationSection } from "./MutationSection.tsx";
import { POPULATION_FIELDS, PopulationSection } from "./PopulationSection.tsx";
import { RUNTIME_FIELDS, RuntimeSection } from "./RuntimeSection.tsx";
import { WORLD_TOPOLOGY_FIELDS, WorldTopologySection } from "./WorldTopologySection.tsx";

export const RUNTIME_PATCH_FIELDS: FieldDef[] = [
	...FOOD_PARAMETERS_FIELDS,
	...POPULATION_FIELDS,
	...WORLD_TOPOLOGY_FIELDS,
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
					<FoodParametersSection
						localDraft={localDraft}
						serverConfig={serverConfig}
						simState={simState}
						updateDraft={updateDraft}
					/>
					<PopulationSection
						localDraft={localDraft}
						serverConfig={serverConfig}
						simState={simState}
						updateDraft={updateDraft}
					/>
					<WorldTopologySection
						localDraft={localDraft}
						serverConfig={serverConfig}
						simState={simState}
						updateDraft={updateDraft}
					/>
					<EnergyLifecycleSection
						localDraft={localDraft}
						serverConfig={serverConfig}
						simState={simState}
						updateDraft={updateDraft}
					/>
					<EnergyCostsSection
						localDraft={localDraft}
						serverConfig={serverConfig}
						simState={simState}
						updateDraft={updateDraft}
					/>
					<RuntimeSection
						localDraft={localDraft}
						serverConfig={serverConfig}
						simState={simState}
						updateDraft={updateDraft}
					/>
					<MutationSection
						localDraft={localDraft}
						serverConfig={serverConfig}
						simState={simState}
						updateDraft={updateDraft}
					/>
				</>
			)}
		</Section>
	);
}
