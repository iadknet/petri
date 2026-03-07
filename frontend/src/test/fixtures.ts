import type { SimulationConfig } from "../types/api.ts";

/** Shared mock config matching production defaults for most fields. */
export const MOCK_CONFIG: SimulationConfig = {
	population: { initial_creatures: 50, max_creatures: 1000 },
	world: {
		width: 400,
		height: 400,
		edge_mode: "Wrap",
		food: {
			growth_rate: 0.25,
			initial_density: 1.0,
			initial_coverage: 0.15,
			spread_threshold_ratio: 0.75,
			spread_density_ratio: 0.25,
			recovery_spawn_rate: 0.02,
			recovery_floor_ratio: 0.03,
			max_density: 1.0,
		},
	},
	energy: {
		lifecycle: {
			initial_energy: 20,
			max_energy: 100,
			energy_decay_per_tick: 0.01,
			min_reproduce_energy: 1,
			default_offspring_energy: 8,
		},
		costs: {
			move_cost: 0.02,
			eat_cost: 0,
			noop_cost: 0,
			reproduce_cost: 0.12,
			eat_reward_per_food: 12,
			failed_action_penalty: 5,
		},
		complexity_cost: {
			enabled: true,
			threshold: 50,
			scaling_factor: 0.002,
		},
		age_cost: {
			enabled: true,
			age_cap: 500,
			max_multiplier: 10.0,
		},
	},
	runtime: {
		max_mesh_hops: 128,
		max_vm_steps: 1024,
		max_graph_relax_iters: 4,
		graph_convergence_epsilon: 0.001,
		graph_convergence_stable_passes: 1,
		graph_node_base_cost: 0.05,
		plasticity_update_cost: 0.0,
		reward_learning_cost: 0.0,
		max_actions_per_turn: 10,
		vm: { opcode_cost_multiplier: 0.5 },
		perception: { vision_radius: 5 },
	},
	mutation: {
		mutation_probability: 0.01,
		per_birth_mutation_events_min: 1,
		per_birth_mutation_events_max: 4,
		mesh_layer_probability: 0.2,
		genome_size_cap: 1200,
		genome_size_pressure_enabled: true,
		action_queue_cap: 4,
		phenotype: {
			channel_step: 1,
			channel_change_chance: 0.001,
			polarity_flip_chance: 0.0002,
		},
	},
	predation: {
		steal_cost_rate: 0.2,
		kill_complexity_bonus_multiplier: 0.05,
	},
};
