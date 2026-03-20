import { beforeEach, describe, expect, it, vi } from "vitest";
import { MOCK_CONFIG } from "../test/fixtures.ts";
import type { SimulationConfig } from "../types/api.ts";
import { buildStartupRequest, useStartupConfigStore } from "./startupConfig.ts";

describe("StartupConfigStore", () => {
	beforeEach(() => {
		useStartupConfigStore.getState().reset();
		vi.restoreAllMocks();
	});

	it("hydrates once from server config", () => {
		useStartupConfigStore.getState().hydrateFromServerConfig(MOCK_CONFIG);
		const state = useStartupConfigStore.getState();
		expect(state.preset.population.initial_creatures).toBe(50);
		expect(state.preset.world.width).toBe(400);
		expect(state.preset.world.food.initial_density).toBe(1.0);
		expect(state.preset.energy.initial_energy).toBe(20);
	});

	it("reset returns large-world fertility defaults", () => {
		useStartupConfigStore.getState().reset();
		const preset = useStartupConfigStore.getState().preset;

		expect(preset.population.initial_creatures).toBe(10000);
		expect(preset.world.width).toBe(1600);
		expect(preset.world.height).toBe(1600);
		expect(preset.world.food.initial_coverage).toBe(0.54);
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
		expect(preset.world.food.fertility.enabled).toBe(true);
		expect(preset.world.food.fertility.min_fertility).toBe(0.4);
		expect(preset.world.food.fertility.max_fertility).toBe(1.8);
		expect(preset.world.food.annealing.enabled).toBe(true);
		expect(preset.world.food.annealing.ramp_ticks).toBe(9000);
		expect(preset.world.food.annealing.initial_min_fertility).toBe(0.6);
		expect(preset.world.food.annealing.initial_max_fertility).toBe(1.4);
	});

	it("buildStartupRequest includes fertility and annealing settings", () => {
		useStartupConfigStore.getState().updatePreset("world.food.fertility.enabled", true);
		useStartupConfigStore.getState().updatePreset("world.food.fertility.min_fertility", 0.5);
		useStartupConfigStore.getState().updatePreset("world.food.fertility.max_fertility", 1.6);
		useStartupConfigStore.getState().updatePreset("world.food.annealing.enabled", true);
		useStartupConfigStore.getState().updatePreset("world.food.annealing.ramp_ticks", 12000);
		useStartupConfigStore
			.getState()
			.updatePreset("world.food.annealing.initial_min_fertility", 0.7);
		useStartupConfigStore
			.getState()
			.updatePreset("world.food.annealing.initial_max_fertility", 1.3);

		const req = buildStartupRequest(useStartupConfigStore.getState().preset);
		expect(req.world?.food).toMatchObject({
			fertility: {
				enabled: true,
				min_fertility: 0.5,
				max_fertility: 1.6,
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
			},
		]);
	});
});
