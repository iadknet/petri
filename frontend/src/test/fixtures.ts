import type { SimulationConfig } from "../types/api.ts";

/** Shared mock config matching production defaults for most fields. */
export const MOCK_CONFIG: SimulationConfig = {
	population: { initial_creatures: 50, max_creatures: 1000 },
	world: {
		width: 400,
		height: 400,
		edge_mode: "Wrap",
		terrain: [],
		world_seed: null,
		food: {
			shared: {
				growth_rate: 0.25,
				initial_density: 1.0,
				initial_coverage: 0.15,
				spread_threshold_ratio: 0.75,
				spread_density_ratio: 0.25,
				recovery_spawn_rate: 0.02,
				recovery_floor_ratio: 0.03,
				max_density: 1.0,
				occupancy_depletion: {
					enabled: true,
					deposit_per_occupied_tick: 0.08,
				},
				grazing: {
					enabled: true,
					factor: 0.5,
					floor: 0.05,
					recovery_ticks: 1000,
				},
			},
			types: [
				{
					name: "Primary Food",
					color: "#22c55e",
					initial_density: 1.0,
					initial_coverage: 0.15,
					growth_inhibitor: 0.2,
				},
			],
			fertility: {
				enabled: false,
				min_fertility: 0.0,
				max_fertility: 2.0,
				layers: [],
			},
			annealing: {
				enabled: false,
				ramp_ticks: 5000,
				initial_min_fertility: 0.3,
				initial_max_fertility: 1.5,
			},
		},
	},
	energy: {
		lifecycle: {
			initial_energy: 20,
			max_energy: 100,
			energy_decay_per_tick: 0.01,
			genome_carry_cost_per_unit: 0.0001,
			genome_replication_cost_per_unit: 0.1,
			min_reproduce_energy: 1,
			default_offspring_energy: 8,
		},
		costs: {
			move_cost: 0.02,
			eat_cost: 0,
			eat_reward_per_food: 5,
			noop_cost: 0,
			reproduce_cost: 0.12,
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
		vm: { opcode_cost_multiplier: 0.5, step_ramp_allowance: 100, step_ramp_cost: 0.000001 },
		perception: { vision_radius: 5 },
	},
	mutation: {
		per_unit_supply_enabled: true,
		per_unit_rate: 0.005,
		mutation_probability: 0.01,
		per_birth_mutation_events_min: 1,
		per_birth_mutation_events_max: 4,
		per_birth_mutation_event_continuation_probability: 0.2,
		mesh_layer_probability: 0.2,
		genome_size_cap: 1200,
		genome_size_pressure_enabled: true,
		action_queue_cap: 4,
		phenotype: {
			channel_step: 1,
			channel_change_chance: 0.001,
			polarity_flip_chance: 0.0002,
		},
		reachable_bias: {
			topology: 0.7,
			vm: 0.7,
			graph: 0.7,
			input_ref: 0.5,
		},
		executed_bias: 0.9,
		executed_window_ticks: 100,
	},
	predation: {
		steal_cost_rate: 0.2,
		kill_complexity_bonus_multiplier: 0.05,
	},
	action_log: {
		capacity: 500,
	},
	shared_memory: {
		decay_rate: 0.0,
	},
	startup: {
		ramps: {
			failed_action_penalty: {
				enabled: false,
				start: 5,
				end: 5,
				target_tick: 1000,
			},
		},
	},
};
