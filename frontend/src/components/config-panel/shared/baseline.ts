import { createContext, useContext } from "react";
import type { SimulationConfig } from "../../../types/api.ts";
import { getByPath } from "./pathUtils.ts";

/**
 * The config the running world was started with. Restart-tab rows compare the
 * startup preset against it to show what the next Restart will change.
 */
export const StartupBaselineContext = createContext<SimulationConfig | null>(null);

/** Startup preset paths that live elsewhere in the server config. */
const PRESET_TO_CONFIG: Record<string, string | null> = {
	seed: null,
	"energy.initial_energy": "energy.lifecycle.initial_energy",
};

export function baselineValueAt(config: SimulationConfig | null, presetPath: string): unknown {
	if (!config) return undefined;
	const path = presetPath in PRESET_TO_CONFIG ? PRESET_TO_CONFIG[presetPath] : presetPath;
	return path ? getByPath(config, path) : undefined;
}

/** The applied value a row compares against: given explicitly, or the startup baseline. */
export function useAppliedValue<T>(path: string, explicit: T | undefined): T | undefined {
	const baseline = useContext(StartupBaselineContext);
	if (explicit !== undefined) return explicit;
	const value = baselineValueAt(baseline, path);
	return value === undefined ? undefined : (value as T);
}
