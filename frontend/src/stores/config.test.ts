import { beforeEach, describe, expect, it } from "vitest";
import type { SimulationConfig } from "../types/api.ts";
import { useConfigStore } from "./config.ts";

const MOCK_CONFIG: SimulationConfig = {
	population: { initial_creatures: 50, max_creatures: 1000 },
	world: {
		width: 400,
		height: 400,
		edge_mode: "wrap",
		food: { growth_rate: 0.02, initial_density: 80, initial_coverage: 0.3 },
	},
	energy: {
		lifecycle: {
			initial_energy: 20,
			max_energy: 100,
			energy_decay_per_tick: 0.2,
			min_reproduce_energy: 24,
			default_offspring_energy: 20,
		},
		costs: {
			move_cost: 0.2,
			eat_cost: 0,
			noop_cost: 0,
			reproduce_cost: 2,
			eat_reward_per_food: 1,
		},
	},
	runtime: {
		max_mesh_hops: 128,
		max_vm_steps: 1024,
		max_graph_relax_iters: 4,
		graph_convergence_epsilon: 0.001,
		graph_convergence_stable_passes: 1,
		graph_node_base_cost: 1.0,
		vm: { opcode_cost_multiplier: 1.0 },
		mutation: {
			mutation_probability: 0.01,
			per_birth_mutation_events_min: 1,
			per_birth_mutation_events_max: 4,
			domain_selection_weights: { Topology: 1, Vm: 1, Graph: 1 },
			operator_selection_weights: {
				Topology: { AddNode: 1, RemoveNode: 1 },
				Vm: { VmInstructionMutation: 1, VmConstantMutation: 1 },
				Graph: { AddInternalGraphNode: 1, RemoveInternalGraphNode: 1 },
			},
			operator_modifier_scale: 1.0,
			phenotype: {
				channel_step: 2,
				polarity_flip_chance: 0.002,
				channel_weight_min: 0.05,
				channel_weight_max: 1.0,
			},
		},
	},
};

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
		useConfigStore.getState().updateDraft("runtime.mutation.phenotype.channel_step", 5);
		const draft = useConfigStore.getState().localDraft!;
		expect(draft.runtime.mutation.phenotype.channel_step).toBe(5);
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
