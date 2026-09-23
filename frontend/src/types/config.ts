import type { PatternBounds, PatternParams } from "./pattern.ts";

export type FertilityAlgorithm =
	| { Uniform: { value: number } }
	| {
			Fbm: {
				octaves: number;
				frequency: number;
				lacunarity: number;
				persistence: number;
				seed?: number | null;
			};
	  }
	| {
			PoissonBlobs: {
				blob_count: number;
				min_radius: number;
				max_radius: number;
				falloff: number;
				seed?: number | null;
			};
	  };

export interface FertilityLayer {
	algorithm: FertilityAlgorithm;
	weight: number;
	target?: FoodFertilityLayerTarget;
}

export interface FertilityConfig {
	enabled: boolean;
	min_fertility: number;
	max_fertility: number;
	layers: FertilityLayer[];
}

export type FoodFertilityLayerTarget = "AllFoods" | { SingleType: { type_idx: number } };

export interface FoodSharedConfig {
	growth_rate: number;
	initial_density: number;
	initial_coverage: number;
	spread_threshold_ratio: number;
	spread_density_ratio: number;
	recovery_spawn_rate: number;
	recovery_floor_ratio: number;
	max_density: number;
	occupancy_depletion: OccupancyDepletionConfig;
	grazing: GrazingConfig;
}

export interface FoodTypeConfig {
	name: string;
	color: string;
	initial_density: number;
	initial_coverage: number;
	growth_inhibitor: number;
	energy_per_unit?: number | null;
	growth_rate?: number | null;
	recovery_spawn_rate?: number | null;
	initial_fertility_only?: boolean;
}

export interface StartupFoodRequestLayer {
	algorithm: FertilityAlgorithm;
	weight: number;
	target: FoodFertilityLayerTarget;
}

export interface StartupFoodRequestFertilityConfig {
	enabled: boolean;
	min_fertility: number;
	max_fertility: number;
	layers: StartupFoodRequestLayer[];
}

export interface StartupFoodRequest {
	shared: FoodSharedConfig;
	types: FoodTypeConfig[];
	fertility: StartupFoodRequestFertilityConfig;
	annealing: AnnealingConfig;
}

export interface StartupFoodConfig extends Omit<StartupFoodRequest, "fertility"> {
	fertility: FertilityConfig;
}

export interface AnnealingConfig {
	enabled: boolean;
	ramp_ticks: number;
	initial_min_fertility: number;
	initial_max_fertility: number;
}

export interface OccupancyDepletionConfig {
	enabled: boolean;
	deposit_per_occupied_tick: number;
}

/** Grazing recovery and overuse (T02.F04): `world.food.shared.grazing`. */
export interface GrazingConfig {
	enabled: boolean;
	factor: number;
	floor: number;
	recovery_ticks: number;
}

export interface FoodConfig {
	shared: FoodSharedConfig;
	types: FoodTypeConfig[];
	fertility: FertilityConfig;
	annealing: AnnealingConfig;
}

export type WorldEdgeMode = "Wrap" | "Bounded";

export interface TerrainLayer {
	params: PatternParams;
	bounds?: PatternBounds | null;
	seed?: number | null;
}

export interface WorldConfig {
	width: number;
	height: number;
	edge_mode: WorldEdgeMode;
	terrain: TerrainLayer[];
	world_seed: number | null;
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
	genome_carry_cost_per_unit: number;
	genome_replication_cost_per_unit: number;
}

export interface CostsConfig {
	move_cost: number;
	eat_cost: number;
	eat_reward_per_food: number;
	noop_cost: number;
	reproduce_cost: number;
	failed_action_penalty: number;
}

export interface ComplexityEnergyCostConfig {
	enabled: boolean;
	threshold: number;
	scaling_factor: number;
}

export interface AgeEnergyCostConfig {
	enabled: boolean;
	grace_ticks: number;
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
	step_ramp_allowance: number;
	step_ramp_cost: number;
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

export interface MutationConfig {
	per_unit_supply_enabled: boolean;
	per_unit_rate: number;
	mutation_probability: number;
	per_birth_mutation_events_min: number;
	per_birth_mutation_events_max: number;
	per_birth_mutation_event_continuation_probability: number;
	mesh_layer_probability: number;
	large_copy_weight_percent: number;
	genome_size_cap: number;
	genome_size_pressure_enabled: boolean;
	phenotype: PhenotypeConfig;
	reachable_bias: ReachableBiasConfig;
	executed_bias: number;
	executed_window_ticks: number;
}

export interface PerceptionRuntimeConfig {
	vision_radius: number;
}

export interface RuntimeConfig {
	max_mesh_hops: number;
	max_vm_steps: number;
	graph_node_base_cost: number;
	plasticity_update_cost: number;
	reward_learning_cost: number;
	hop_ramp_allowance: number;
	hop_ramp_cost: number;
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
