export interface FoodConfig {
	growth_rate: number;
	initial_density: number;
	initial_coverage: number;
	spread_threshold_ratio: number;
	spread_density_ratio: number;
	recovery_spawn_rate: number;
	recovery_floor_ratio: number;
	max_density: number;
}

export type WorldEdgeMode = "Wrap" | "Bounded";

export interface WorldConfig {
	width: number;
	height: number;
	edge_mode: WorldEdgeMode;
	food: FoodConfig;
}

export interface PopulationConfig {
	initial_creatures: number;
	max_creatures: number;
}

export interface LifecycleEnergyConfig {
	initial_energy: number;
	max_energy: number;
	energy_decay_per_tick: number;
	min_reproduce_energy: number;
	default_offspring_energy: number;
}

export interface CostsConfig {
	move_cost: number;
	eat_cost: number;
	noop_cost: number;
	reproduce_cost: number;
	eat_reward_per_food: number;
	failed_action_penalty: number;
}

export interface ComplexityEnergyCostConfig {
	enabled: boolean;
	threshold: number;
	scaling_factor: number;
}

export interface AgeEnergyCostConfig {
	enabled: boolean;
	age_cap: number;
	max_multiplier: number;
}

export interface EnergyConfig {
	lifecycle: LifecycleEnergyConfig;
	costs: CostsConfig;
	complexity_cost: ComplexityEnergyCostConfig;
	age_cost: AgeEnergyCostConfig;
}

export interface VmConfig {
	opcode_cost_multiplier: number;
}

export interface PhenotypeConfig {
	channel_step: number;
	channel_change_chance: number;
	polarity_flip_chance: number;
}

export interface ReachableBiasConfig {
	topology: number;
	vm: number;
	graph: number;
	input_ref: number;
}

export interface TopologyNewNodeBirthConfig {
	graph_backend_chance: number;
	graph_initialized_chance: number;
	graph_compute_gate_chance: number;
}

export interface MutationConfig {
	mutation_probability: number;
	per_birth_mutation_events_min: number;
	per_birth_mutation_events_max: number;
	mesh_layer_probability: number;
	genome_size_cap: number;
	genome_size_pressure_enabled: boolean;
	action_queue_cap: number;
	phenotype: PhenotypeConfig;
	reachable_bias: ReachableBiasConfig;
	topology_new_node_birth: TopologyNewNodeBirthConfig;
}

export interface PerceptionRuntimeConfig {
	vision_radius: number;
}

export interface RuntimeConfig {
	max_mesh_hops: number;
	max_vm_steps: number;
	max_graph_relax_iters: number;
	graph_convergence_epsilon: number;
	graph_convergence_stable_passes: number;
	graph_node_base_cost: number;
	plasticity_update_cost: number;
	reward_learning_cost: number;
	max_actions_per_turn: number;
	vm: VmConfig;
	perception: PerceptionRuntimeConfig;
}

export interface PredationConfig {
	steal_cost_rate: number;
	kill_complexity_bonus_multiplier: number;
}

export interface ActionLogConfig {
	capacity: number;
}

export interface SharedMemoryConfig {
	decay_rate: number;
}

export interface FailedActionPenaltyRampConfig {
	enabled: boolean;
	start: number;
	end: number;
	target_tick: number;
}

export interface StartupRampsConfig {
	failed_action_penalty: FailedActionPenaltyRampConfig;
}

export interface StartupConfig {
	ramps: StartupRampsConfig;
}

export interface SimulationConfig {
	population: PopulationConfig;
	world: WorldConfig;
	energy: EnergyConfig;
	startup: StartupConfig;
	runtime: RuntimeConfig;
	mutation: MutationConfig;
	predation: PredationConfig;
	action_log: ActionLogConfig;
	shared_memory: SharedMemoryConfig;
}
