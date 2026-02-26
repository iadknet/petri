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
			growth_rate: number;
			initial_density: number;
			initial_coverage: number;
			spread_threshold_ratio: number;
			recovery_spawn_rate: number;
			recovery_floor_ratio: number;
			max_density: number;
		};
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
		population: {
			initial_creatures: 2000,
		},
		world: {
			width: 400,
			height: 400,
			food: {
				growth_rate: 0.096,
				initial_density: 1.0,
				initial_coverage: 0.15,
				spread_threshold_ratio: 0.8,
				recovery_spawn_rate: 0.01,
				recovery_floor_ratio: 0.01,
				max_density: 1.0,
			},
		},
	};
}

function fromServerConfig(config: SimulationConfig): StartupPreset {
	return {
		seed: randomSeed(),
		population: {
			initial_creatures: config.population.initial_creatures,
		},
		world: {
			width: config.world.width,
			height: config.world.height,
			food: {
				growth_rate: config.world.food.growth_rate,
				initial_density: config.world.food.initial_density,
				initial_coverage: config.world.food.initial_coverage,
				spread_threshold_ratio: config.world.food.spread_threshold_ratio,
				recovery_spawn_rate: config.world.food.recovery_spawn_rate,
				recovery_floor_ratio: config.world.food.recovery_floor_ratio,
				max_density: config.world.food.max_density,
			},
		},
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
