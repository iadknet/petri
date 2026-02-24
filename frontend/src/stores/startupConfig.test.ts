import { beforeEach, describe, expect, it, vi } from "vitest";
import type { SimulationConfig } from "../types/api.ts";
import { useStartupConfigStore } from "./startupConfig.ts";

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
		expect(state.preset.world.food.initial_density).toBe(80);
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
});
