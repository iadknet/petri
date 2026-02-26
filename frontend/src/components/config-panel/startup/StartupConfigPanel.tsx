import { Section } from "../shared/Section.tsx";
import type { StartupPanelProps } from "../shared/types.ts";
import { EnergySection } from "./EnergySection.tsx";
import { FoodParametersSection } from "./FoodParametersSection.tsx";
import { PopulationSection } from "./PopulationSection.tsx";
import { RunSettingsSection } from "./RunSettingsSection.tsx";
import { WorldTopologySection } from "./WorldTopologySection.tsx";

export function StartupConfigPanel({
	startupPreset,
	updateStartupPreset,
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
			<FoodParametersSection
				startupPreset={startupPreset}
				updateStartupPreset={updateStartupPreset}
			/>
		</Section>
	);
}
