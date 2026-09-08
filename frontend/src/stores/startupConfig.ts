import { create } from "zustand";
import type { SimulationConfig } from "../types/api.ts";
import type {
	FertilityLayer,
	FoodFertilityLayerTarget,
	FoodSharedConfig,
	FoodTypeConfig,
	StartupFoodConfig,
	StartupFoodRequest,
	TerrainLayer,
	WorldEdgeMode,
} from "../types/config.ts";
import type { StartupRequest } from "../types/http.ts";
import { deepSet } from "../utils/deepSet.ts";

export interface StartupPreset {
	seed: number;
	population: {
		initial_creatures: number;
	};
	world: {
		width: number;
		height: number;
		edge_mode: WorldEdgeMode;
		terrain: TerrainLayer[];
		world_seed: number | null;
		food: StartupFoodConfig;
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
	addFoodType: () => void;
	removeFoodType: (index: number) => void;
	updateFoodType: (index: number, patch: Partial<FoodTypeConfig>) => void;
	addFertilityLayer: () => void;
	updateFertilityLayerTarget: (index: number, target: FoodFertilityLayerTarget) => void;
	randomizeSeed: () => void;
	hydrateFromServerConfig: (config: SimulationConfig) => void;
	reset: () => void;
}

function randomSeed(): number {
	return Math.floor(Math.random() * 2 ** 32);
}

const FOOD_TYPE_COLORS = ["#22c55e", "#38bdf8", "#f97316", "#e879f9", "#facc15", "#2dd4bf"];

function createFoodType(index: number): FoodTypeConfig {
	return {
		name: index === 0 ? "Primary Food" : `Food ${index + 1}`,
		color: FOOD_TYPE_COLORS[index % FOOD_TYPE_COLORS.length] ?? "#22c55e",
		initial_density: 1.0,
		initial_coverage: 0.54,
		growth_inhibitor: 0.2,
	};
}

function defaultFoodShared(): FoodSharedConfig {
	return {
		growth_rate: 0.09,
		initial_density: 1.0,
		initial_coverage: 0.54,
		spread_threshold_ratio: 0.8,
		spread_density_ratio: 0.25,
		recovery_spawn_rate: 0.01,
		recovery_floor_ratio: 0.01,
		max_density: 1.0,
		occupancy_depletion: {
			enabled: true,
			deposit_per_occupied_tick: 0.08,
		},
	};
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

function cloneFoodType(type: FoodTypeConfig): FoodTypeConfig {
	return { ...type };
}

function clampFinite(value: number, min: number, max: number, fallback: number): number {
	if (!Number.isFinite(value)) {
		return fallback;
	}
	if (value < min) {
		return min;
	}
	if (value > max) {
		return max;
	}
	return value;
}

function normalizeFoodType(
	type: FoodTypeConfig,
	index: number,
	maxDensity: number,
): FoodTypeConfig {
	const defaults = createFoodType(index);
	return {
		name: type.name,
		color: type.color,
		initial_density: clampFinite(type.initial_density, 0, maxDensity, defaults.initial_density),
		initial_coverage: clampFinite(type.initial_coverage, 0, 1, defaults.initial_coverage),
		growth_inhibitor: clampFinite(type.growth_inhibitor, 0, 1, defaults.growth_inhibitor),
	};
}

function cloneFertilityLayer(layer: FertilityLayer): FertilityLayer {
	return structuredClone(layer);
}

function cloneFertilityTarget(
	target: FoodFertilityLayerTarget | undefined,
): FoodFertilityLayerTarget | undefined {
	if (!target) return undefined;
	if (target === "AllFoods") return "AllFoods";
	return { SingleType: { type_idx: target.SingleType.type_idx } };
}

function normalizeFertilityTarget(
	target: FoodFertilityLayerTarget | undefined,
	typeCount: number,
): FoodFertilityLayerTarget | undefined {
	const cloned = cloneFertilityTarget(target);
	if (!cloned || cloned === "AllFoods") {
		return cloned;
	}

	const typeIndex = cloned.SingleType.type_idx;
	if (!Number.isInteger(typeIndex) || typeIndex < 0 || typeIndex >= typeCount) {
		return "AllFoods";
	}

	return cloned;
}

function normalizeFoodConfig(food: StartupFoodConfig): StartupFoodConfig {
	const sharedDefaults = defaultFoodShared();
	const shared = {
		...sharedDefaults,
		...food.shared,
		occupancy_depletion: {
			...sharedDefaults.occupancy_depletion,
			...food.shared.occupancy_depletion,
		},
	};
	const types =
		food.types.length > 0
			? food.types.map((type, index) =>
					normalizeFoodType(cloneFoodType(type), index, shared.max_density),
				)
			: [createFoodType(0)];
	const primaryType = types[0]!;

	return {
		shared: {
			growth_rate: shared.growth_rate,
			initial_density: primaryType.initial_density,
			initial_coverage: primaryType.initial_coverage,
			spread_threshold_ratio: shared.spread_threshold_ratio,
			spread_density_ratio: shared.spread_density_ratio,
			recovery_spawn_rate: shared.recovery_spawn_rate,
			recovery_floor_ratio: shared.recovery_floor_ratio,
			max_density: shared.max_density,
			occupancy_depletion: {
				...shared.occupancy_depletion,
			},
		},
		types,
		fertility: {
			...food.fertility,
			layers: food.fertility.layers.map((layer) => {
				const cloned = cloneFertilityLayer(layer);
				const target = normalizeFertilityTarget(layer.target, types.length);
				return target ? { ...cloned, target } : cloned;
			}),
		},
		annealing: {
			...food.annealing,
		},
	};
}

function buildDefaultFoodConfig(): StartupFoodConfig {
	return normalizeFoodConfig({
		shared: defaultFoodShared(),
		types: [createFoodType(0)],
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
	});
}

function fromServerFoodConfig(config: SimulationConfig["world"]["food"]): StartupFoodConfig {
	return normalizeFoodConfig({
		shared: {
			growth_rate: config.shared.growth_rate,
			initial_density: config.shared.initial_density,
			initial_coverage: config.shared.initial_coverage,
			spread_threshold_ratio: config.shared.spread_threshold_ratio,
			spread_density_ratio: config.shared.spread_density_ratio,
			recovery_spawn_rate: config.shared.recovery_spawn_rate,
			recovery_floor_ratio: config.shared.recovery_floor_ratio,
			max_density: config.shared.max_density,
			occupancy_depletion: {
				enabled: config.shared.occupancy_depletion.enabled,
				deposit_per_occupied_tick: config.shared.occupancy_depletion.deposit_per_occupied_tick,
			},
		},
		types: config.types.length > 0 ? config.types.map(cloneFoodType) : [createFoodType(0)],
		fertility: {
			enabled: config.fertility.enabled,
			min_fertility: config.fertility.min_fertility,
			max_fertility: config.fertility.max_fertility,
			layers: config.fertility.layers,
		},
		annealing: {
			enabled: config.annealing.enabled,
			ramp_ticks: config.annealing.ramp_ticks,
			initial_min_fertility: config.annealing.initial_min_fertility,
			initial_max_fertility: config.annealing.initial_max_fertility,
		},
	});
}

function updateFoodConfig(
	preset: StartupPreset,
	updater: (food: StartupFoodConfig) => StartupFoodConfig,
): StartupPreset {
	return withFoodPreset(preset, updater(normalizeFoodConfig(preset.world.food)));
}

function updateFoodTypeTargetsAfterRemoval(
	layers: FertilityLayer[],
	removedIndex: number,
): FertilityLayer[] {
	return layers.map((layer) => {
		if (!layer.target || layer.target === "AllFoods") {
			return { ...cloneFertilityLayer(layer) };
		}

		const currentIndex = layer.target.SingleType.type_idx;
		if (currentIndex === removedIndex) {
			return {
				...cloneFertilityLayer(layer),
				target: "AllFoods",
			};
		}

		if (currentIndex > removedIndex) {
			return {
				...cloneFertilityLayer(layer),
				target: {
					SingleType: {
						type_idx: currentIndex - 1,
					},
				},
			};
		}

		return { ...cloneFertilityLayer(layer) };
	});
}

function buildDefaultPreset(): StartupPreset {
	return {
		seed: randomSeed(),
		population: { initial_creatures: 10000 },
		world: {
			width: 1600,
			height: 1600,
			edge_mode: "Wrap",
			terrain: [],
			world_seed: null,
			food: buildDefaultFoodConfig(),
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
			terrain: structuredClone(config.world.terrain),
			world_seed: config.world.world_seed,
			food: fromServerFoodConfig(config.world.food),
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

function withFoodPreset(preset: StartupPreset, food: StartupFoodConfig): StartupPreset {
	return {
		...preset,
		world: {
			...preset.world,
			food: normalizeFoodConfig(food),
		},
	};
}

function buildStartupFoodRequest(food: StartupFoodConfig): StartupFoodRequest {
	const normalized = normalizeFoodConfig(food);
	return {
		shared: {
			growth_rate: normalized.shared.growth_rate,
			initial_density: normalized.shared.initial_density,
			initial_coverage: normalized.shared.initial_coverage,
			spread_threshold_ratio: normalized.shared.spread_threshold_ratio,
			spread_density_ratio: normalized.shared.spread_density_ratio,
			recovery_spawn_rate: normalized.shared.recovery_spawn_rate,
			recovery_floor_ratio: normalized.shared.recovery_floor_ratio,
			max_density: normalized.shared.max_density,
			occupancy_depletion: {
				...normalized.shared.occupancy_depletion,
			},
		},
		types: normalized.types.map(cloneFoodType),
		fertility: {
			enabled: normalized.fertility.enabled,
			min_fertility: normalized.fertility.min_fertility,
			max_fertility: normalized.fertility.max_fertility,
			layers: normalized.fertility.layers.map((layer) => ({
				algorithm: layer.algorithm,
				weight: layer.weight,
				target: layer.target ?? "AllFoods",
			})),
		},
		annealing: {
			...normalized.annealing,
		},
	};
}

export const useStartupConfigStore = create<StartupConfigState>()((set) => ({
	preset: buildDefaultPreset(),
	touched: false,
	hydrated: false,

	setPreset: (preset) =>
		set({
			preset: withFoodPreset(preset, preset.world.food),
			touched: true,
		}),

	updatePreset: (path, value) =>
		set((s) => {
			const next = deepSet(
				s.preset as unknown as Record<string, unknown>,
				path,
				value,
			) as unknown as StartupPreset;

			return {
				preset: withFoodPreset(next, next.world.food),
				touched: true,
			};
		}),

	addFoodType: () =>
		set((s) => ({
			preset: updateFoodConfig(s.preset, (food) => ({
				...food,
				types: [...food.types, createFoodType(food.types.length)],
			})),
			touched: true,
		})),

	removeFoodType: (index) =>
		set((s) => {
			const food = normalizeFoodConfig(s.preset.world.food);
			if (food.types.length <= 1 || index < 0 || index >= food.types.length) {
				return s;
			}

			return {
				preset: updateFoodConfig(s.preset, (currentFood) => ({
					...currentFood,
					types: currentFood.types.filter((_, currentIndex) => currentIndex !== index),
					fertility: {
						...currentFood.fertility,
						layers: updateFoodTypeTargetsAfterRemoval(currentFood.fertility.layers, index),
					},
				})),
				touched: true,
			};
		}),

	updateFoodType: (index, patch) =>
		set((s) => {
			const food = normalizeFoodConfig(s.preset.world.food);
			if (index < 0 || index >= food.types.length) {
				return s;
			}

			return {
				preset: updateFoodConfig(s.preset, (currentFood) => ({
					...currentFood,
					types: currentFood.types.map((type, currentIndex) =>
						currentIndex === index ? { ...type, ...patch } : type,
					),
				})),
				touched: true,
			};
		}),

	addFertilityLayer: () =>
		set((s) => ({
			preset: updateFoodConfig(s.preset, (food) => ({
				...food,
				fertility: {
					...food.fertility,
					layers: [
						...food.fertility.layers,
						{
							weight: 1.0,
							algorithm: {
								PoissonBlobs: {
									blob_count: 900,
									min_radius: 5.0,
									max_radius: 15.0,
									falloff: 0.5,
								},
							},
						},
					],
				},
			})),
			touched: true,
		})),

	updateFertilityLayerTarget: (index, target) =>
		set((s) => {
			const food = normalizeFoodConfig(s.preset.world.food);
			if (index < 0 || index >= food.fertility.layers.length) {
				return s;
			}

			return {
				preset: updateFoodConfig(s.preset, (currentFood) => ({
					...currentFood,
					fertility: {
						...currentFood.fertility,
						layers: currentFood.fertility.layers.map((layer, currentIndex) =>
							currentIndex === index
								? {
										...cloneFertilityLayer(layer),
										target: cloneFertilityTarget(target),
									}
								: layer,
						),
					},
				})),
				touched: true,
			};
		}),

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
export function buildStartupRequest(preset: StartupPreset): StartupRequest {
	return {
		seed: preset.seed,
		population: { initial_creatures: preset.population.initial_creatures },
		world: {
			width: preset.world.width,
			height: preset.world.height,
			edge_mode: preset.world.edge_mode,
			terrain: structuredClone(preset.world.terrain),
			world_seed: preset.world.world_seed,
			food: buildStartupFoodRequest(preset.world.food),
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
