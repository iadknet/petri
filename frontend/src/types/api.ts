// Protocol version
export const PROTOCOL_VERSION = "v3alpha1" as const;

// Simulation states
export type SimState = "idle" | "running" | "paused";

// Frame data
export interface Creature {
	id: number;
	x: number;
	y: number;
	energy: number;
	generation: number;
	phenotype_rgb: [number, number, number];
}

export interface FoodCell {
	x: number;
	y: number;
	density: number;
}

export interface Barrier {
	x: number;
	y: number;
}

export interface Frame {
	width: number;
	height: number;
	creatures: Creature[];
	food: FoodCell[];
	barriers: Barrier[];
}

// Status
export interface LastTickActions {
	move: number;
	eat: number;
	reproduce: number;
	noop: number;
}

export interface StatusPayload {
	state: SimState;
	population: number;
	mean_energy: number;
	last_tick_actions: LastTickActions;
	reproduction_actions_attempted_total: number;
	reproduction_actions_spawned_total: number;
	reproduction_actions_rejected_total: number;
	last_tick_compute_total_mean: number;
	last_tick_compute_total_min: number;
	last_tick_compute_total_max: number;
	last_tick_compute_vm_mean: number;
	last_tick_compute_graph_mean: number;
}

// Health
export interface HealthPayload {
	population: number;
	mean_energy: number;
	mutation_events_attempted_total: number;
	mutation_events_applied_total: number;
	mutation_events_skipped_total: number;
	reproduction_actions_attempted_total: number;
	reproduction_actions_spawned_total: number;
	reproduction_actions_rejected_total: number;
	reproduction_actions_rejected_total_by_reason: Record<string, number>;
	genome_complexity_mean: number;
	genome_complexity_min: number;
	genome_complexity_max: number;
}

// WebSocket binary frame (msgpack)
export interface WsFrame {
	tick: number;
	status: StatusPayload;
	frame: Frame;
	health: HealthPayload;
}

// Legacy WebSocket envelope (no longer used by WS, kept for reference)
export type WsEventType = "status" | "frame" | "health";

export interface WsEnvelope<T = unknown> {
	protocol_version: string;
	event: WsEventType;
	tick: number;
	payload: T;
}

// Config types - mirrors the full config structure
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
	polarity_flip_chance: number;
	channel_weight_min: number;
	channel_weight_max: number;
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

// HTTP response types
export interface LifecycleResponse {
	protocol_version: string;
	state: SimState;
	tick: number;
}

export interface StartupResponse extends LifecycleResponse {
	config_digest: string;
	seeded_creatures: number;
}

export interface StepResponse extends LifecycleResponse {
	steps_applied: number;
}

export interface StatusResponse extends StatusPayload {
	protocol_version: string;
	tick: number;
}

export interface FrameResponse extends Frame {
	protocol_version: string;
	tick: number;
}

export interface ConfigResponse {
	protocol_version: string;
	state: SimState;
	config: SimulationConfig;
}

// Request types
export interface StartupRequest {
	seed: number;
	population?: Partial<PopulationConfig>;
	world?: Partial<WorldConfig> & { food?: Partial<FoodConfig> };
	energy?: { lifecycle?: Partial<LifecycleEnergyConfig>; costs?: Partial<CostsConfig> };
	runtime?: Partial<Omit<RuntimeConfig, "vm">> & {
		vm?: Partial<VmConfig>;
	};
	mutation?: Partial<Omit<MutationConfig, "phenotype">> & {
		phenotype?: Partial<PhenotypeConfig>;
	};
}

export interface StepRequest {
	steps?: number;
}

// Error types
export interface FieldError {
	field: string;
	reason: string;
}

export interface ErrorDetails {
	endpoint?: string;
	field_errors?: FieldError[];
	expected_state?: SimState;
	current_state?: SimState;
}

export interface ApiError {
	protocol_version: string;
	error: {
		code: "invalid_request" | "invalid_state_transition" | "validation_rejected" | "internal_error";
		message: string;
		details?: ErrorDetails;
	};
}
