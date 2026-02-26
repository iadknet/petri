import { create } from "zustand";
import type { SimulationConfig } from "../types/api.ts";
import { deepSet } from "../utils/deepSet.ts";

export interface StartupPreset {
	seed: number;
	population: {
		initial_creatures: number;
	};
	world: {
		width: number;
		height: number;
		food: {
			initial_density: number;
			initial_coverage: number;
		};
	};
	energy: {
		initial_energy: number;
	};
}

export interface StartupConfigState {
	preset: StartupPreset;
	touched: boolean;
	hydrated: boolean;

	setPreset: (next: StartupPreset) => void;
	updatePreset: (path: string, value: number) => void;
	randomizeSeed: () => void;
	hydrateFromServerConfig: (config: SimulationConfig) => void;
	reset: () => void;
}

function randomSeed(): number {
	return Math.floor(Math.random() * 2 ** 32);
}

function buildDefaultPreset(): StartupPreset {
	return {
		seed: randomSeed(),
		population: { initial_creatures: 2000 },
		world: {
			width: 400,
			height: 400,
			food: {
				initial_density: 1.0,
				initial_coverage: 0.15,
			},
		},
		energy: { initial_energy: 20.0 },
	};
}

function fromServerConfig(config: SimulationConfig): StartupPreset {
	return {
		seed: randomSeed(),
		population: { initial_creatures: config.population.initial_creatures },
		world: {
			width: config.world.width,
			height: config.world.height,
			food: {
				initial_density: config.world.food.initial_density,
				initial_coverage: config.world.food.initial_coverage,
			},
		},
		energy: { initial_energy: config.energy.lifecycle.initial_energy },
	};
}

export const useStartupConfigStore = create<StartupConfigState>()((set) => ({
	preset: buildDefaultPreset(),
	touched: false,
	hydrated: false,

	setPreset: (preset) =>
		set({
			preset: structuredClone(preset),
			touched: true,
		}),

	updatePreset: (path, value) =>
		set((s) => ({
			preset: deepSet(
				s.preset as unknown as Record<string, unknown>,
				path,
				value,
			) as unknown as StartupPreset,
			touched: true,
		})),

	randomizeSeed: () =>
		set((s) => ({
			preset: {
				...s.preset,
				seed: randomSeed(),
			},
			touched: true,
		})),

	hydrateFromServerConfig: (config) =>
		set((s) => {
			if (s.hydrated || s.touched) {
				return s;
			}
			return {
				preset: fromServerConfig(config),
				hydrated: true,
			};
		}),

	reset: () =>
		set({
			preset: buildDefaultPreset(),
			touched: false,
			hydrated: false,
		}),
}));

/** Build the startup API request from the current preset */
export function buildStartupRequest(preset: StartupPreset) {
	return {
		seed: preset.seed,
		population: { initial_creatures: preset.population.initial_creatures },
		world: {
			width: preset.world.width,
			height: preset.world.height,
			food: { ...preset.world.food },
		},
		energy: {
			lifecycle: { initial_energy: preset.energy.initial_energy },
		},
	};
}
