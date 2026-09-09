import { beforeEach, describe, expect, it, vi } from "vitest";
import { MOCK_CONFIG } from "../test/fixtures.ts";
import type { SimulationConfig } from "../types/api.ts";
import { buildStartupRequest, useStartupConfigStore } from "./startupConfig.ts";

describe("StartupConfigStore", () => {
	beforeEach(() => {
		useStartupConfigStore.getState().reset();
		vi.restoreAllMocks();
	});

	it("normalizes an empty food catalog to the primary default", () => {
		const config = structuredClone(MOCK_CONFIG);
		config.world.food.types = [];
		useStartupConfigStore.getState().hydrateFromServerConfig(config);
		expect(buildStartupRequest(useStartupConfigStore.getState().preset).world?.food?.types).toEqual(
			[
				expect.objectContaining({
					name: "Primary Food",
					color: "#22c55e",
					initial_density: 1,
					initial_coverage: 0.54,
				}),
			],
		);
	});

	it("hydrates once from server config", () => {
		useStartupConfigStore.getState().hydrateFromServerConfig(MOCK_CONFIG);
		const state = useStartupConfigStore.getState();
		expect(state.preset.population.initial_creatures).toBe(50);
		expect(state.preset.world.width).toBe(400);
		expect(state.preset.world.food.shared.occupancy_depletion).toEqual({
			enabled: true,
			deposit_per_occupied_tick: 0.08,
		});
		expect(state.preset.world.food.types[0]).toMatchObject({
			name: "Primary Food",
			color: "#22c55e",
			initial_density: 1.0,
			initial_coverage: 0.15,
			growth_inhibitor: 0.2,
		});
		expect(state.preset.energy.initial_energy).toBe(20);
	});

	it("reset returns large-world fertility defaults", () => {
		useStartupConfigStore.getState().reset();
		const preset = useStartupConfigStore.getState().preset;

		expect(preset.population.initial_creatures).toBe(10000);
		expect(preset.world.width).toBe(1600);
		expect(preset.world.height).toBe(1600);
		expect(preset.world.food.shared.occupancy_depletion).toEqual({
			enabled: true,
			deposit_per_occupied_tick: 0.08,
		});
		expect(preset.world.food.types).toHaveLength(1);
		expect(preset.world.food.types[0]).toMatchObject({
			name: "Primary Food",
			color: "#22c55e",
			initial_density: 1.0,
			initial_coverage: 0.54,
			growth_inhibitor: 0.2,
		});

		expect(preset.world.food.fertility.enabled).toBe(true);
		expect(preset.world.food.fertility.layers).toEqual([
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
		]);
		expect(preset.startup.ramps.failed_action_penalty).toEqual({
			enabled: true,
			start: 0.0,
			end: 1.0,
			target_tick: 62680,
		});
	});

	it("serializes one primary food for restart", () => {
		const request = buildStartupRequest(useStartupConfigStore.getState().preset);
		expect(request.world?.food?.types).toEqual([
			expect.objectContaining({ name: "Primary Food", initial_coverage: 0.54 }),
		]);
		expect(request).not.toHaveProperty("nutrition");
	});

	it("does not overwrite user edits after touch", () => {
		useStartupConfigStore.getState().hydrateFromServerConfig(MOCK_CONFIG);
		useStartupConfigStore.getState().updatePreset("world.width", 512);

		useStartupConfigStore.getState().hydrateFromServerConfig({
			...MOCK_CONFIG,
			world: { ...MOCK_CONFIG.world, width: 300 },
		});

		expect(useStartupConfigStore.getState().preset.world.width).toBe(512);
	});

	it("randomizeSeed updates the seed", () => {
		vi.spyOn(Math, "random").mockReturnValue(0.123456);
		const before = useStartupConfigStore.getState().preset.seed;
		useStartupConfigStore.getState().randomizeSeed();
		const after = useStartupConfigStore.getState().preset.seed;
		expect(after).not.toBe(before);
		expect(after).toBe(Math.floor(0.123456 * 2 ** 32));
	});

	it("buildStartupRequest includes startup failed action penalty ramp settings", () => {
		useStartupConfigStore
			.getState()
			.updatePreset("startup.ramps.failed_action_penalty.enabled", true);
		useStartupConfigStore.getState().updatePreset("startup.ramps.failed_action_penalty.start", 5);
		useStartupConfigStore.getState().updatePreset("startup.ramps.failed_action_penalty.end", 30);
		useStartupConfigStore
			.getState()
			.updatePreset("startup.ramps.failed_action_penalty.target_tick", 1000);

		const req = buildStartupRequest(useStartupConfigStore.getState().preset);
		expect(req.startup?.ramps?.failed_action_penalty).toEqual({
			enabled: true,
			start: 5,
			end: 30,
			target_tick: 1000,
		});
	});

	it("hydrates fertility and annealing values from server config", () => {
		const configWithFertility: SimulationConfig = {
			...MOCK_CONFIG,
			world: {
				...MOCK_CONFIG.world,
				food: {
					...MOCK_CONFIG.world.food,
					fertility: {
						enabled: true,
						min_fertility: 0.4,
						max_fertility: 1.8,
						layers: [{ algorithm: { Uniform: { value: 0.0 } }, weight: 1.0 }],
					},
					annealing: {
						enabled: true,
						ramp_ticks: 9000,
						initial_min_fertility: 0.6,
						initial_max_fertility: 1.4,
					},
				},
			},
		};

		useStartupConfigStore.getState().hydrateFromServerConfig(configWithFertility);

		const preset = useStartupConfigStore.getState().preset;
		const [primaryType] = preset.world.food.types;
		expect(primaryType).toBeDefined();
		expect(primaryType!.initial_density).toBe(1.0);
		expect(primaryType!.initial_coverage).toBe(0.15);
		expect(preset.world.food.fertility.enabled).toBe(true);
		expect(preset.world.food.fertility.min_fertility).toBe(0.4);
		expect(preset.world.food.fertility.max_fertility).toBe(1.8);
		expect(preset.world.food.annealing.enabled).toBe(true);
		expect(preset.world.food.annealing.ramp_ticks).toBe(9000);
		expect(preset.world.food.annealing.initial_min_fertility).toBe(0.6);
		expect(preset.world.food.annealing.initial_max_fertility).toBe(1.4);
	});

	it("supports typed food type actions and retargets fertility layers on removal", () => {
		const store = useStartupConfigStore.getState();
		store.updatePreset("world.food.fertility.enabled", true);
		store.addFertilityLayer();
		store.updateFertilityLayerTarget(0, { SingleType: { type_idx: 0 } });
		store.addFoodType();
		store.updateFoodType(1, {
			name: "Blue Food",
			color: "#3b82f6",
			initial_density: 0.8,
			initial_coverage: 0.3,
			growth_inhibitor: 0.45,
		});

		expect(useStartupConfigStore.getState().preset.world.food.types).toHaveLength(2);
		expect(useStartupConfigStore.getState().preset.world.food.types[1]).toMatchObject({
			name: "Blue Food",
			color: "#3b82f6",
			initial_density: 0.8,
			initial_coverage: 0.3,
			growth_inhibitor: 0.45,
		});

		store.removeFoodType(0);

		const preset = useStartupConfigStore.getState().preset;
		expect(preset.world.food.types).toHaveLength(1);
		expect(preset.world.food.types[0]).toMatchObject({
			name: "Blue Food",
			color: "#3b82f6",
			initial_density: 0.8,
			initial_coverage: 0.3,
			growth_inhibitor: 0.45,
		});
		expect(preset.world.food.fertility.layers[0]!.target).toEqual("AllFoods");
	});

	it("defaults and clamps per-type growth inhibitor values", () => {
		const store = useStartupConfigStore.getState();
		store.addFoodType();

		expect(useStartupConfigStore.getState().preset.world.food.types[1]!.growth_inhibitor).toBe(0.2);

		store.updatePreset("world.food.types.0.growth_inhibitor", 4);
		store.updatePreset("world.food.types.1.growth_inhibitor", -0.5);

		const types = useStartupConfigStore.getState().preset.world.food.types;
		expect(types[0]!.growth_inhibitor).toBe(1);
		expect(types[1]!.growth_inhibitor).toBe(0);
	});

	it("serializes explicit fertility targets in startup requests", () => {
		const store = useStartupConfigStore.getState();
		store.addFoodType();
		store.addFertilityLayer();
		store.updateFertilityLayerTarget(0, { SingleType: { type_idx: 1 } });

		const req = buildStartupRequest(useStartupConfigStore.getState().preset);
		expect(req.world?.food?.fertility?.layers[0]?.target).toEqual({
			SingleType: { type_idx: 1 },
		});
	});

	it("retargets invalid single-type fertility targets to all_foods", () => {
		useStartupConfigStore.getState().updatePreset("world.food.fertility.layers", [
			{
				weight: 1.0,
				algorithm: { Uniform: { value: 0.5 } },
				target: { SingleType: { type_idx: 999 } },
			},
		]);

		const req = buildStartupRequest(useStartupConfigStore.getState().preset);
		expect(req.world?.food?.fertility?.layers[0]?.target).toBe("AllFoods");
	});

	it("buildStartupRequest includes split food, fertility, and annealing settings", () => {
		useStartupConfigStore.getState().updatePreset("world.food.fertility.enabled", true);
		useStartupConfigStore.getState().updatePreset("world.food.fertility.min_fertility", 0.5);
		useStartupConfigStore.getState().updatePreset("world.food.fertility.max_fertility", 1.6);
		useStartupConfigStore.getState().updatePreset("world.food.types.0.initial_density", 0.75);
		useStartupConfigStore.getState().updatePreset("world.food.types.0.initial_coverage", 0.4);
		useStartupConfigStore.getState().updatePreset("world.food.annealing.enabled", true);
		useStartupConfigStore.getState().updatePreset("world.food.annealing.ramp_ticks", 12000);
		useStartupConfigStore
			.getState()
			.updatePreset("world.food.annealing.initial_min_fertility", 0.7);
		useStartupConfigStore
			.getState()
			.updatePreset("world.food.annealing.initial_max_fertility", 1.3);
		useStartupConfigStore.getState().updatePreset("world.food.fertility.layers", [
			{
				weight: 0.65,
				algorithm: {
					Fbm: {
						octaves: 4,
						frequency: 0.07,
						lacunarity: 2.1,
						persistence: 0.48,
						seed: 42,
					},
				},
			},
		]);

		const req = buildStartupRequest(useStartupConfigStore.getState().preset);
		expect(req.world?.food).toMatchObject({
			shared: {
				occupancy_depletion: {
					enabled: true,
					deposit_per_occupied_tick: 0.08,
				},
			},
			types: expect.arrayContaining([
				expect.objectContaining({
					name: "Primary Food",
					color: "#22c55e",
					initial_density: 0.75,
					initial_coverage: 0.4,
					growth_inhibitor: 0.2,
				}),
			]),
			fertility: {
				enabled: true,
				min_fertility: 0.5,
				max_fertility: 1.6,
				layers: [
					{
						target: "AllFoods",
					},
				],
			},
			annealing: {
				enabled: true,
				ramp_ticks: 12000,
				initial_min_fertility: 0.7,
				initial_max_fertility: 1.3,
			},
		});
	});

	it("buildStartupRequest preserves fertility layer algorithm configuration", () => {
		useStartupConfigStore.getState().updatePreset("world.food.fertility.enabled", true);
		useStartupConfigStore.getState().updatePreset("world.food.fertility.layers", [
			{
				weight: 0.65,
				algorithm: {
					Fbm: {
						octaves: 4,
						frequency: 0.07,
						lacunarity: 2.1,
						persistence: 0.48,
						seed: 42,
					},
				},
			},
		]);

		const req = buildStartupRequest(useStartupConfigStore.getState().preset);
		expect(req.world?.food?.fertility?.layers).toEqual([
			{
				weight: 0.65,
				algorithm: {
					Fbm: {
						octaves: 4,
						frequency: 0.07,
						lacunarity: 2.1,
						persistence: 0.48,
						seed: 42,
					},
				},
				target: "AllFoods",
			},
		]);
	});

	it("ignores legacy startup food alias writes", () => {
		useStartupConfigStore.getState().updatePreset("world.food.initial_density", 0.81);
		useStartupConfigStore.getState().updatePreset("world.food.initial_coverage", 0.22);

		const preset = useStartupConfigStore.getState().preset;
		expect(preset.world.food.types[0]).toMatchObject({
			initial_density: 1.0,
			initial_coverage: 0.54,
		});
		expect((preset.world.food as unknown as { initial_density?: number }).initial_density).toBe(
			undefined,
		);
		expect((preset.world.food as unknown as { initial_coverage?: number }).initial_coverage).toBe(
			undefined,
		);
	});
});

it("energy-only startup defaults serialize one primary food", () => {
	useStartupConfigStore.getState().reset();
	const request = buildStartupRequest(useStartupConfigStore.getState().preset);
	expect(request.world?.food?.types).toEqual([
		expect.objectContaining({
			name: "Primary Food",
			color: "#22c55e",
			initial_density: 1,
			initial_coverage: 0.54,
		}),
	]);
	expect(request).not.toHaveProperty("nutrition");
	expect(request.world?.food?.types?.[0]).not.toHaveProperty("metabolic_energy_yield");
});

it("hydrates ordered terrain layers and optional map/layer seeds without aliasing", () => {
	useStartupConfigStore.getState().reset();
	const config = structuredClone(MOCK_CONFIG);
	config.world.world_seed = 123;
	config.world.terrain = [
		{
			params: { pattern_type: "Noise", density: 0.2, cluster_size: 3 },
			seed: 456,
			bounds: { x: 2, y: 3, width: 20, height: 30 },
		},
		{
			params: { pattern_type: "Maze", corridor_width: 2, wall_thickness: 1, open_center_radius: 0 },
			seed: null,
			bounds: null,
		},
	];
	useStartupConfigStore.getState().hydrateFromServerConfig(config);
	const request = buildStartupRequest(useStartupConfigStore.getState().preset);
	expect(request.world?.terrain).toEqual(config.world.terrain);
	expect(request.world?.world_seed).toBe(123);
	config.world.terrain.reverse();
	expect(request.world?.terrain?.[0]?.params.pattern_type).toBe("Noise");
});

it("rejects unsafe hydrated seeds before a normal restart", () => {
	useStartupConfigStore.getState().reset();
	const config = structuredClone(MOCK_CONFIG);
	config.world.world_seed = Number.MAX_SAFE_INTEGER + 1;
	useStartupConfigStore.getState().hydrateFromServerConfig(config);
	expect(() => buildStartupRequest(useStartupConfigStore.getState().preset)).toThrow(
		/safe integer/,
	);
});
it("refreshes recipe startup controls without changing the selected run seed", () => {
	useStartupConfigStore.getState().updatePreset("seed", 42);
	useStartupConfigStore.getState().applyRecipeConfig(MOCK_CONFIG);
	expect(useStartupConfigStore.getState().preset.seed).toBe(42);
	expect(useStartupConfigStore.getState().preset.world.width).toBe(MOCK_CONFIG.world.width);
});

it("retains overrides, zero and inheritance through hydration and startup requests", () => {
	useStartupConfigStore.getState().reset();
	const config = structuredClone(MOCK_CONFIG);
	Object.assign(config.world.food.types[0]!, {
		energy_per_unit: 12,
		growth_rate: null,
		recovery_spawn_rate: 0,
		initial_fertility_only: true,
	});
	useStartupConfigStore.getState().hydrateFromServerConfig(config);
	const first = buildStartupRequest(useStartupConfigStore.getState().preset).world?.food
		?.types?.[0];
	expect(first).toMatchObject({
		energy_per_unit: 12,
		growth_rate: null,
		recovery_spawn_rate: 0,
		initial_fertility_only: true,
	});
	useStartupConfigStore
		.getState()
		.updateFoodType(0, { energy_per_unit: null, growth_rate: Number.POSITIVE_INFINITY });
	expect(
		buildStartupRequest(useStartupConfigStore.getState().preset).world?.food?.types?.[0],
	).toMatchObject({ energy_per_unit: null, growth_rate: null, recovery_spawn_rate: 0 });
});
