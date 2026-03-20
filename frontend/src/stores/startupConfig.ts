import { create } from "zustand";
import type { SimulationConfig } from "../types/api.ts";
import type { FertilityLayer, WorldEdgeMode } from "../types/config.ts";
import { deepSet } from "../utils/deepSet.ts";

interface StartupFertilityConfig {
	enabled: boolean;
	min_fertility: number;
	max_fertility: number;
	layers: FertilityLayer[];
}

interface StartupAnnealingConfig {
	enabled: boolean;
	ramp_ticks: number;
	initial_min_fertility: number;
	initial_max_fertility: number;
}

export interface StartupPreset {
	seed: number;
	population: {
		initial_creatures: number;
	};
	world: {
		width: number;
		height: number;
		edge_mode: WorldEdgeMode;
		food: {
			initial_density: number;
			initial_coverage: number;
			fertility: StartupFertilityConfig;
			annealing: StartupAnnealingConfig;
		};
	};
	energy: {
		initial_energy: number;
	};
	startup: {
		ramps: {
			failed_action_penalty: {
				enabled: boolean;
				start: number;
				end: number;
				target_tick: number;
			};
		};
	};
}

export interface StartupConfigState {
	preset: StartupPreset;
	touched: boolean;
	hydrated: boolean;

	setPreset: (next: StartupPreset) => void;
	updatePreset: (path: string, value: unknown) => void;
	randomizeSeed: () => void;
	hydrateFromServerConfig: (config: SimulationConfig) => void;
	reset: () => void;
}

function randomSeed(): number {
	return Math.floor(Math.random() * 2 ** 32);
}

function defaultPoissonLayer(): FertilityLayer {
	return {
		weight: 1.0,
		algorithm: {
			PoissonBlobs: {
				blob_count: 900,
				min_radius: 5.0,
				max_radius: 15.0,
				falloff: 0.5,
			},
		},
	};
}

function buildDefaultPreset(): StartupPreset {
	return {
		seed: randomSeed(),
		population: { initial_creatures: 10000 },
		world: {
			width: 1600,
			height: 1600,
			edge_mode: "Wrap",
			food: {
				initial_density: 1.0,
				initial_coverage: 0.54,
				fertility: {
					enabled: true,
					min_fertility: 0.0,
					max_fertility: 2.0,
					layers: [defaultPoissonLayer(), defaultPoissonLayer()],
				},
				annealing: {
					enabled: false,
					ramp_ticks: 5000,
					initial_min_fertility: 0.3,
					initial_max_fertility: 1.5,
				},
			},
		},
		energy: { initial_energy: 20.0 },
		startup: {
			ramps: {
				failed_action_penalty: {
					enabled: true,
					start: 0.0,
					end: 1.0,
					target_tick: 62680,
				},
			},
		},
	};
}

function fromServerConfig(config: SimulationConfig): StartupPreset {
	return {
		seed: randomSeed(),
		population: { initial_creatures: config.population.initial_creatures },
		world: {
			width: config.world.width,
			height: config.world.height,
			edge_mode: config.world.edge_mode,
			food: {
				initial_density: config.world.food.initial_density,
				initial_coverage: config.world.food.initial_coverage,
				fertility: {
					enabled: config.world.food.fertility?.enabled ?? false,
					min_fertility: config.world.food.fertility?.min_fertility ?? 0.0,
					max_fertility: config.world.food.fertility?.max_fertility ?? 2.0,
					layers:
						config.world.food.fertility?.layers ??
						[defaultPoissonLayer(), defaultPoissonLayer()],
				},
				annealing: {
					enabled: config.world.food.annealing?.enabled ?? false,
					ramp_ticks: config.world.food.annealing?.ramp_ticks ?? 5000,
					initial_min_fertility: config.world.food.annealing?.initial_min_fertility ?? 0.3,
					initial_max_fertility: config.world.food.annealing?.initial_max_fertility ?? 1.5,
				},
			},
		},
		energy: { initial_energy: config.energy.lifecycle.initial_energy },
		startup: {
			ramps: {
				failed_action_penalty: {
					enabled: config.startup.ramps.failed_action_penalty.enabled,
					start: config.startup.ramps.failed_action_penalty.start,
					end: config.startup.ramps.failed_action_penalty.end,
					target_tick: config.startup.ramps.failed_action_penalty.target_tick,
				},
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

/** Build the startup API request from the current preset */
export function buildStartupRequest(preset: StartupPreset) {
	return {
		seed: preset.seed,
		population: { initial_creatures: preset.population.initial_creatures },
		world: {
			width: preset.world.width,
			height: preset.world.height,
			edge_mode: preset.world.edge_mode,
			food: {
				initial_density: preset.world.food.initial_density,
				initial_coverage: preset.world.food.initial_coverage,
				fertility: {
					...preset.world.food.fertility,
					layers: [...preset.world.food.fertility.layers],
				},
				annealing: { ...preset.world.food.annealing },
			},
		},
		energy: {
			lifecycle: { initial_energy: preset.energy.initial_energy },
		},
		startup: {
			ramps: {
				failed_action_penalty: {
					...preset.startup.ramps.failed_action_penalty,
				},
			},
		},
	};
}
