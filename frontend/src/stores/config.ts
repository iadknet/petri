import { create } from "zustand";
import type { SimState, SimulationConfig } from "../types/api.ts";

export interface ConfigState {
	/** Config as last received from server */
	serverConfig: SimulationConfig | null;
	/** Local draft with user edits */
	localDraft: SimulationConfig | null;
	/** Current sim state (for editability rules) */
	simState: SimState;
	/** Whether local draft differs from server config */
	isDirty: boolean;

	setServerConfig: (config: SimulationConfig, simState: SimState) => void;
	updateDraft: (path: string, value: number | string) => void;
	resetDraft: () => void;
	reset: () => void;
}

/** Deep-set a nested value by dot-separated path */
function deepSet<T extends Record<string, unknown>>(obj: T, path: string, value: unknown): T {
	const clone = structuredClone(obj);
	const keys = path.split(".");
	let current: Record<string, unknown> = clone;
	for (let i = 0; i < keys.length - 1; i++) {
		const key = keys[i]!;
		if (typeof current[key] !== "object" || current[key] === null) {
			current[key] = {};
		}
		current = current[key] as Record<string, unknown>;
	}
	current[keys[keys.length - 1]!] = value;
	return clone;
}

/** Deep equality check */
function deepEqual(a: unknown, b: unknown): boolean {
	return JSON.stringify(a) === JSON.stringify(b);
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
