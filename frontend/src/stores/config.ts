import { create } from "zustand";
import type { SimulationConfig } from "../types/generated/SimulationConfig.ts";
import type { SimulationStatus } from "../types/generated/SimulationStatus.ts";
import { deepSet } from "../utils/deepSet.ts";

export interface ConfigState {
	/** Config as last received from server */
	serverConfig: SimulationConfig | null;
	/** Local draft with user edits */
	localDraft: SimulationConfig | null;
	/** Current sim state (for editability rules) */
	simState: SimulationStatus;
	/** Whether local draft differs from server config */
	isDirty: boolean;

	setServerConfig: (config: SimulationConfig, simState: SimulationStatus) => void;
	commitServerConfig: (config: SimulationConfig, simState: SimulationStatus) => void;
	updateDraft: (path: string, value: number | string | boolean) => void;
	resetDraft: () => void;
	reset: () => void;
}

/**
 * Deep equality that treats numbers equal after an f32 round trip: the server
 * stores f32, so a value typed exactly as displayed (0.3) must not read as a
 * change from what it sent (0.30000001192092896).
 */
function deepEqual(a: unknown, b: unknown): boolean {
	if (typeof a === "number" && typeof b === "number") {
		return a === b || Math.fround(a) === Math.fround(b);
	}
	if (a === null || b === null || typeof a !== "object" || typeof b !== "object") {
		return a === b;
	}
	if (Array.isArray(a) !== Array.isArray(b)) return false;
	const aKeys = Object.keys(a);
	const bKeys = Object.keys(b);
	if (aKeys.length !== bKeys.length) return false;
	return aKeys.every((key) =>
		deepEqual((a as Record<string, unknown>)[key], (b as Record<string, unknown>)[key]),
	);
}

export const useConfigStore = create<ConfigState>()((set) => ({
	serverConfig: null,
	localDraft: null,
	simState: "idle",
	isDirty: false,

	setServerConfig: (config, simState) =>
		set((s) => ({
			serverConfig: config,
			simState,
			localDraft: s.isDirty ? s.localDraft : structuredClone(config),
			isDirty: s.isDirty ? !deepEqual(s.localDraft, config) : false,
		})),

	commitServerConfig: (config, simState) =>
		set({
			serverConfig: config,
			simState,
			localDraft: structuredClone(config),
			isDirty: false,
		}),

	updateDraft: (path, value) =>
		set((s) => {
			if (!s.localDraft) return s;
			const next = deepSet(s.localDraft as unknown as Record<string, unknown>, path, value);
			return {
				localDraft: next as unknown as SimulationConfig,
				isDirty: !deepEqual(next, s.serverConfig),
			};
		}),

	resetDraft: () =>
		set((s) => ({
			localDraft: s.serverConfig ? structuredClone(s.serverConfig) : null,
			isDirty: false,
		})),

	reset: () =>
		set({
			serverConfig: null,
			localDraft: null,
			simState: "idle",
			isDirty: false,
		}),
}));
