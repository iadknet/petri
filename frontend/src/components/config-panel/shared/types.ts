import type { StartupPreset } from "../../../stores/startupConfig.ts";
import type { FertilityLayerTarget } from "../../../types/generated/FertilityLayerTarget.ts";
import type { FoodTypeConfig } from "../../../types/generated/FoodTypeConfig.ts";
import type { SimulationConfig } from "../../../types/generated/SimulationConfig.ts";
import type { SimulationStatus } from "../../../types/generated/SimulationStatus.ts";

export interface FieldDef {
	path: string;
	label: string;
	/** Hard bounds: what the server accepts. Committed edits are clamped to them. */
	min: number;
	max: number;
	/** Arrow-key increment and slider granularity. */
	step: number;
	testId?: string;
	topologyField?: boolean;
	defaultValue?: number;
	tooltip?: string;
	/** Comfortable slider span when the hard bounds are far wider than useful values. */
	sliderMin?: number;
	sliderMax?: number;
	/** Hide the slider for values that span orders of magnitude (tiny costs, caps). */
	slider?: false;
}

export interface EnumFieldDef {
	path: string;
	label: string;
	options: readonly { value: string; label: string }[];
	testId?: string;
	defaultValue?: string;
	tooltip?: string;
}

export type AnyFieldDef = FieldDef | BooleanFieldDef | EnumFieldDef;

export function isNumberField(field: AnyFieldDef): field is FieldDef {
	return "min" in field;
}

export function isEnumField(field: AnyFieldDef): field is EnumFieldDef {
	return "options" in field;
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
	updateFertilityLayerTarget: (index: number, target: FertilityLayerTarget) => void;
	randomizeSeed: () => void;
}

export interface StartupSectionProps {
	startupPreset: StartupPreset;
	updateStartupPreset: StartupUpdater;
}

export interface RuntimePanelProps {
	localDraft: SimulationConfig;
	serverConfig: SimulationConfig;
	simState: SimulationStatus;
	updateDraft: RuntimeUpdater;
	/** Server reasons from the last rejected Apply, keyed by config path. */
	fieldErrors: Record<string, string>;
}
