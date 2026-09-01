import { Section } from "../shared/Section.tsx";
import type { StartupPanelProps } from "../shared/types.ts";
import { EnergySection } from "./EnergySection.tsx";
import { FertilitySection } from "./FertilitySection.tsx";
import { FoodTypesSection } from "./FoodTypesSection.tsx";
import { NutritionSection } from "./NutritionSection.tsx";
import { PopulationSection } from "./PopulationSection.tsx";
import { RunSettingsSection } from "./RunSettingsSection.tsx";
import { StartupRampsSection } from "./StartupRampsSection.tsx";
import { WorldTopologySection } from "./WorldTopologySection.tsx";

export function StartupConfigPanel({
	startupPreset,
	updateStartupPreset,
	addFoodType,
	removeFoodType,
	updateFoodType,
	addFertilityLayer,
	updateFertilityLayerTarget,
	randomizeSeed,
}: StartupPanelProps) {
	return (
		<Section
			title="Startup Config"
			description="These settings apply on Restart."
			collapsible
			sectionClassName="bg-emerald-950/10 border-l-2 border-l-emerald-500"
		>
			<RunSettingsSection
				seed={startupPreset.seed}
				updateSeed={(value) => updateStartupPreset("seed", value)}
				randomizeSeed={randomizeSeed}
			/>
			<PopulationSection startupPreset={startupPreset} updateStartupPreset={updateStartupPreset} />
			<WorldTopologySection
				startupPreset={startupPreset}
				updateStartupPreset={updateStartupPreset}
			/>
			<EnergySection startupPreset={startupPreset} updateStartupPreset={updateStartupPreset} />
			<NutritionSection startupPreset={startupPreset} updateStartupPreset={updateStartupPreset} />
			<FoodTypesSection
				startupPreset={startupPreset}
				addFoodType={addFoodType}
				removeFoodType={removeFoodType}
				updateFoodType={updateFoodType}
			/>
			<FertilitySection
				startupPreset={startupPreset}
				updateStartupPreset={updateStartupPreset}
				addFertilityLayer={addFertilityLayer}
				updateFertilityLayerTarget={updateFertilityLayerTarget}
			/>
			<StartupRampsSection
				startupPreset={startupPreset}
				updateStartupPreset={updateStartupPreset}
			/>
		</Section>
	);
}
