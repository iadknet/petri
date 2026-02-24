import { create } from "zustand";
import type { SimulationConfig } from "../types/api.ts";

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
			initial_creatures: 50,
		},
		world: {
			width: 400,
			height: 400,
			food: {
				growth_rate: 0.02,
				initial_density: 80,
				initial_coverage: 0.3,
			},
		},
	};
}

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
