import { beforeEach, describe, expect, it, vi } from "vitest";
import { MOCK_CONFIG } from "../test/fixtures.ts";
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
});
