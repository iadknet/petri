import type { HealthPayload, SimulationConfig } from "../types/api.ts";

/** Shared mock config matching production defaults for most fields. */
export const MOCK_CONFIG: SimulationConfig = {
	population: { initial_creatures: 50, max_creatures: 1000, founder_profile: "v3_alpha1" },
	world: {
		width: 400,
		height: 400,
		edge_mode: "Wrap",
		terrain: [],
		world_seed: null,
		food: {
			shared: {
				growth_rate: 0.25,
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
			gestation_ticks_per_founder_genome: 1.0,
			min_reproduce_energy: 1,
			min_reproduce_age: 20,
			age_reference_ticks: 500,
			default_offspring_energy: 8,
		},
		costs: {
			move_cost: 0.02,
			eat_cost: 0.05,
			eat_reward_per_food: 5,
			noop_cost: 0.05,
			reproduce_cost: 0.12,
			reproduce_attempt_cost: 0.05,
		},
		complexity_cost: {
			enabled: true,
			threshold: 50,
			scaling_factor: 0.002,
		},
		age_cost: {
			enabled: true,
			grace_ticks: 100,
			age_cap: 200,
			max_multiplier: 10.0,
		},
		capacity: {
			reference: 100,
			min: 50,
			founder: 100,
			move_cost_slope: 0.5,
			upkeep_per_unit: 0.0005,
			sensor_scale: "reference",
		},
	},
	runtime: {
		max_mesh_hops: 128,
		max_vm_steps: 1024,
		graph_node_base_cost: 0.05,
		hop_ramp_allowance: 32,
		hop_ramp_cost: 0.0001,
		max_actions_per_turn: 10,
		vm: { opcode_cost_multiplier: 0.5, step_ramp_allowance: 100, step_ramp_cost: 0.000001 },
		perception: { vision_radius: 5 },
	},
	mutation: {
		neutral_input_recruitment: "WholeFamily",
		per_unit_rate: 0.0060625,
		mesh_layer_probability: 0.2,
		large_copy_weight_percent: 25,
		genome_size_cap: 1200,
		genome_size_pressure_enabled: true,
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
		capacity: { chance: 0.1, log_step: 0.25 },
		share: { chance: 0.1, log_step: 0.25 },
	},
	predation: {
		kill_complexity_bonus_multiplier: 0.05,
	},
	exchange: {
		effort_rate: 0.2,
		share_min: 0.01,
		founder_share: 0.2,
	},
	action_log: {
		capacity: 500,
	},
	shared_memory: {
		decay_rate: 0.0,
	},
};

/** Health census fields at a fresh world's values; spread into health fixtures. */
export const MOCK_HEALTH_CENSUS = {
	mutation_outcome_summary: {
		carriers_observed_total: 0,
		helpful_total: 0,
		neutral_total: 0,
		detrimental_total: 0,
	},
	deaths_total: 0,
	deaths_total_by_cause: {},
	food_total_density_by_type: [],
	generation_mean: 0,
	generation_max: 0,
	genome_size_mean: 0,
	lineages_alive: 0,
	lineages_effective: 0,
	world_inputs_read_mean: 0,
	plasticity_changes_total: 0,
	shared_memory_writes_changed_total: 0,
	creature_ticks_total: 0,
} satisfies Partial<HealthPayload>;
