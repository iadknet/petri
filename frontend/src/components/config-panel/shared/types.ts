import type { StartupPreset } from "../../../stores/startupConfig.ts";
import type { SimState, SimulationConfig } from "../../../types/api.ts";

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

export type StartupUpdater = (path: string, value: number) => void;
export type RuntimeUpdater = (path: string, value: number | string) => void;

export interface StartupPanelProps {
	startupPreset: StartupPreset;
	updateStartupPreset: StartupUpdater;
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
	updateDraft: RuntimeUpdater;
}
