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
	  }
	| {
			Channels: {
				octaves: number;
				frequency: number;
				lacunarity: number;
				persistence: number;
				width: number;
				value: number;
				seed?: number | null;
			};
	  };

/** Omitted means `WeightedAverage`, matching the server's serialization. */
export type FertilityBlend = "WeightedAverage" | "Max";

export interface FertilityLayer {
	algorithm: FertilityAlgorithm;
	weight: number;
	target?: FoodFertilityLayerTarget;
	blend?: FertilityBlend;
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
	blend?: FertilityBlend;
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

/** Founder genome profile seeded at startup (`FounderProfile`, snake_case on the wire). */
export type FounderProfile =
	| "v3_alpha1"
	| "forage_first_sparse"
	| "forage_first_sparse_conservative"
	| "forage_first_sparse_rich_offspring"
	| "forage_first_sparse_balanced";

export interface PopulationConfig {
	initial_creatures: number;
	max_creatures: number;
	founder_profile: FounderProfile;
}

export interface LifecycleEnergyConfig {
	initial_energy: number;
	max_energy: number;
	energy_decay_per_tick: number;
	min_reproduce_energy: number;
	min_reproduce_age: number;
	age_reference_ticks: number;
	default_offspring_energy: number;
	genome_carry_cost_per_unit: number;
	genome_replication_cost_per_unit: number;
	gestation_ticks_per_founder_genome: number;
}

export interface CostsConfig {
	move_cost: number;
	eat_cost: number;
	eat_reward_per_food: number;
	noop_cost: number;
	reproduce_cost: number;
	reproduce_attempt_cost: number;
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

/** Which denominator the energy sensor reads divide by (T26.F02). */
export type SensorScale = "reference" | "own";

/** The heritable energy capacity's range and cost rules (T26.F02). */
export interface EnergyCapacityConfig {
	reference: number;
	min: number;
	founder: number;
	move_cost_slope: number;
	upkeep_per_unit: number;
	sensor_scale: SensorScale;
}

export interface EnergyConfig {
	lifecycle: LifecycleEnergyConfig;
	costs: CostsConfig;
	complexity_cost: ComplexityEnergyCostConfig;
	age_cost: AgeEnergyCostConfig;
	capacity: EnergyCapacityConfig;
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

/** Native Graph input recruitment (T20.F04); `WholeFamily` is the default. */
export type NeutralInputRecruitment = "Off" | "SingleChannel" | "WholeFamily";

/** The capacity mutation drawn on every birth (T26.F02). */
export interface CapacityMutationConfig {
	chance: number;
	log_step: number;
}

export interface MutationConfig {
	neutral_input_recruitment: NeutralInputRecruitment;
	per_unit_rate: number;
	mesh_layer_probability: number;
	large_copy_weight_percent: number;
	genome_size_cap: number;
	genome_size_pressure_enabled: boolean;
	phenotype: PhenotypeConfig;
	reachable_bias: ReachableBiasConfig;
	executed_bias: number;
	executed_window_ticks: number;
	capacity: CapacityMutationConfig;
}

export interface PerceptionRuntimeConfig {
	vision_radius: number;
}

export interface RuntimeConfig {
	max_mesh_hops: number;
	max_vm_steps: number;
	graph_node_base_cost: number;
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

export interface SimulationConfig {
	population: PopulationConfig;
	world: WorldConfig;
	energy: EnergyConfig;
	runtime: RuntimeConfig;
	mutation: MutationConfig;
	predation: PredationConfig;
	action_log: ActionLogConfig;
	shared_memory: SharedMemoryConfig;
}
