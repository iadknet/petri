import { beforeEach, describe, expect, it } from "vitest";
import type { SimulationConfig } from "../types/api.ts";
import { MOCK_CONFIG } from "../test/fixtures.ts";
import { useConfigStore } from "./config.ts";

describe("ConfigStore", () => {
	beforeEach(() => {
		useConfigStore.getState().reset();
	});

	it("starts with null config", () => {
		const state = useConfigStore.getState();
		expect(state.serverConfig).toBeNull();
		expect(state.localDraft).toBeNull();
		expect(state.isDirty).toBe(false);
	});

	it("sets server config and creates draft copy", () => {
		useConfigStore.getState().setServerConfig(MOCK_CONFIG, "idle");
		const state = useConfigStore.getState();
		expect(state.serverConfig).toBeTruthy();
		expect(state.localDraft).toBeTruthy();
		expect(state.isDirty).toBe(false);
		// Draft should be a deep clone, not same reference
		expect(state.localDraft).not.toBe(state.serverConfig);
		expect(state.localDraft).toEqual(state.serverConfig);
	});

	it("marks dirty when draft diverges from server", () => {
		useConfigStore.getState().setServerConfig(MOCK_CONFIG, "idle");
		useConfigStore.getState().updateDraft("population.initial_creatures", 100);
		expect(useConfigStore.getState().isDirty).toBe(true);
	});

	it("clears dirty when draft matches server", () => {
		useConfigStore.getState().setServerConfig(MOCK_CONFIG, "idle");
		useConfigStore.getState().updateDraft("population.initial_creatures", 100);
		expect(useConfigStore.getState().isDirty).toBe(true);
		useConfigStore.getState().updateDraft("population.initial_creatures", 50);
		expect(useConfigStore.getState().isDirty).toBe(false);
	});

	it("resetDraft reverts to server config", () => {
		useConfigStore.getState().setServerConfig(MOCK_CONFIG, "idle");
		useConfigStore.getState().updateDraft("population.initial_creatures", 999);
		expect(useConfigStore.getState().isDirty).toBe(true);

		useConfigStore.getState().resetDraft();
		expect(useConfigStore.getState().isDirty).toBe(false);
		expect(useConfigStore.getState().localDraft).toEqual(MOCK_CONFIG);
	});

	it("updates deeply nested fields", () => {
		useConfigStore.getState().setServerConfig(MOCK_CONFIG, "idle");
		useConfigStore.getState().updateDraft("mutation.phenotype.channel_step", 5);
		const draft = useConfigStore.getState().localDraft!;
		expect(draft.mutation.phenotype.channel_step).toBe(5);
	});

	it("commitServerConfig syncs draft to server and clears dirty", () => {
		useConfigStore.getState().setServerConfig(MOCK_CONFIG, "idle");
		useConfigStore.getState().updateDraft("energy.costs.move_cost", 0.3);
		expect(useConfigStore.getState().isDirty).toBe(true);

		const committedConfig: SimulationConfig = {
			...MOCK_CONFIG,
			energy: {
				...MOCK_CONFIG.energy,
				costs: {
					...MOCK_CONFIG.energy.costs,
					move_cost: 0.30000001192092896,
				},
			},
		};
		useConfigStore.getState().commitServerConfig(committedConfig, "paused");

		const state = useConfigStore.getState();
		expect(state.isDirty).toBe(false);
		expect(state.serverConfig).toEqual(committedConfig);
		expect(state.localDraft).toEqual(committedConfig);
	});
});
