import type { StartupPreset } from "../../../stores/startupConfig.ts";
import type { SimState, SimulationConfig } from "../../../types/api.ts";
import type { FoodFertilityLayerTarget, FoodTypeConfig } from "../../../types/config.ts";

export interface FieldDef {
	path: string;
	label: string;
	min: number;
	max: number;
	step: number;
	testId?: string;
	topologyField?: boolean;
	defaultValue?: number;
	tooltip?: string;
}

export interface BooleanFieldDef {
	path: string;
	label: string;
	testId?: string;
	defaultValue?: boolean;
	tooltip?: string;
}

export type StartupUpdater = (path: string, value: unknown) => void;
export type RuntimeUpdater = (path: string, value: number | string | boolean) => void;

export interface StartupPanelProps {
	startupPreset: StartupPreset;
	updateStartupPreset: StartupUpdater;
	addFoodType: () => void;
	removeFoodType: (index: number) => void;
	updateFoodType: (index: number, patch: Partial<FoodTypeConfig>) => void;
	addFertilityLayer: () => void;
	updateFertilityLayerTarget: (index: number, target: FoodFertilityLayerTarget) => void;
	randomizeSeed: () => void;
}

export interface StartupSectionProps {
	startupPreset: StartupPreset;
	updateStartupPreset: StartupUpdater;
}

export interface RuntimePanelProps {
	localDraft: SimulationConfig;
	serverConfig: SimulationConfig;
	simState: SimState;
	tick: number;
	updateDraft: RuntimeUpdater;
}
