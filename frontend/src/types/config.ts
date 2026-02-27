export interface FoodConfig {
	growth_rate: number;
	initial_density: number;
	initial_coverage: number;
	spread_threshold_ratio: number;
	recovery_spawn_rate: number;
	recovery_floor_ratio: number;
	max_density: number;
}

export interface WorldConfig {
	width: number;
	height: number;
	edge_mode: string;
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
}

export interface EnergyConfig {
	lifecycle: LifecycleEnergyConfig;
	costs: CostsConfig;
}

export interface VmConfig {
	opcode_cost_multiplier: number;
}

export interface PhenotypeConfig {
	channel_step: number;
	channel_change_chance: number;
	polarity_flip_chance: number;
}

export interface MutationConfig {
	mutation_probability: number;
	per_birth_mutation_events_min: number;
	per_birth_mutation_events_max: number;
	phenotype: PhenotypeConfig;
}

export interface RuntimeConfig {
	max_mesh_hops: number;
	max_vm_steps: number;
	max_graph_relax_iters: number;
	graph_convergence_epsilon: number;
	graph_convergence_stable_passes: number;
	graph_node_base_cost: number;
	vm: VmConfig;
}

export interface SimulationConfig {
	population: PopulationConfig;
	world: WorldConfig;
	energy: EnergyConfig;
	runtime: RuntimeConfig;
	mutation: MutationConfig;
}
