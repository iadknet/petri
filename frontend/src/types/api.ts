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

// Creature detail types (genome, phenotype, memory)

export interface CreaturePhenotype {
	channels: [number, number, number, number, number, number];
	active_channel: number;
	polarity: [boolean, boolean, boolean, boolean, boolean, boolean];
	rgb: [number, number, number];
}

export interface GraphInput {
	source_idx: number;
	weight: number;
}

export type GraphNodeKind =
	| { InputRef: number }
	| { Constant: number }
	| "Add"
	| "Multiply"
	| "Negate"
	| "Abs"
	| "Min"
	| "Max"
	| { Threshold: number }
	| "GreaterThan"
	| "Sigmoid"
	| "Tanh"
	| "Relu"
	| "Select"
	| "Clamp01"
	| "WeightedSum"
	| { DecayIntegrator: number }
	| { Momentum: number }
	| { Oscillator: number }
	| "AdaptiveGain"
	| { CustomOutput: number }
	| "RouterOutput";

export interface GraphInternalNode {
	kind: GraphNodeKind;
	inputs: GraphInput[];
}

export interface GraphBackendDef {
	internal_nodes: GraphInternalNode[];
}

export type VmInstruction =
	| "Noop"
	| { LoadConst: { dst: number; const_idx: number } }
	| { Move: { dst: number; src: number } }
	| { Add: { dst: number; a: number; b: number } }
	| { Sub: { dst: number; a: number; b: number } }
	| { Mul: { dst: number; a: number; b: number } }
	| { Div: { dst: number; a: number; b: number } }
	| { Min: { dst: number; a: number; b: number } }
	| { Max: { dst: number; a: number; b: number } }
	| { Abs: { dst: number; src: number } }
	| { Neg: { dst: number; src: number } }
	| { Clamp01: { dst: number; src: number } }
	| { CmpGt: { dst: number; a: number; b: number } }
	| { CmpLt: { dst: number; a: number; b: number } }
	| { CmpEq: { dst: number; a: number; b: number; eps: number } }
	| { And: { dst: number; a: number; b: number } }
	| { Or: { dst: number; a: number; b: number } }
	| { Not: { dst: number; src: number } }
	| { ToI32: { dst: number; src: number } }
	| { ToU8: { dst: number; src: number } }
	| { ToBool: { dst: number; src: number } }
	| { JumpIfZero: { cond: number; offset: number } }
	| { Jump: { offset: number } }
	| { ReadInput: { dst: number; input_idx: number } }
	| { WriteInternalPayload: { slot_idx: number; src: number } }
	| { WriteWorldActionMeta: { slot_idx: number; src: number } }
	| { EmitWorldAction: { action_type: number } }
	| { WriteRouteTarget: { src: number } }
	| "Halt"
	| { LoadMem8: { dst: number; addr_reg: number } }
	| { StoreMem8: { addr_reg: number; src: number } }
	| { LoadMem8Imm: { dst: number; imm_addr: number } }
	| { StoreMem8Imm: { imm_addr: number; src: number } };

export interface VmBackendDef {
	register_count: number;
	constants: number[];
	program: VmInstruction[];
}

export type BackendDef = { Vm: VmBackendDef } | { Graph: GraphBackendDef };

export type InputReference =
	| {
			World:
				| string
				| { NeighborCellFood: string }
				| { NeighborCellBarrier: string }
				| { NeighborCellOccupied: string };
	  }
	| { StaticIntrospection: string }
	| { DynamicIntrospection: string }
	| { UpstreamSlot: number };

export interface NodeGenome {
	node_id: number;
	input_refs: InputReference[];
	backend_def: BackendDef;
	targets: number[];
}

export interface CreatureGenome {
	entry_node_id: number;
	nodes: NodeGenome[];
}

export interface CreatureDetail {
	protocol_version: string;
	id: number;
	position: { x: number; y: number };
	energy: number;
	max_energy: number;
	age: number;
	generation: number;
	complexity: number;
	phenotype: CreaturePhenotype;
	genome: CreatureGenome;
	memory: number[];
}

// Paint types
export type PaintTool = "food" | "barrier" | "erase_food" | "erase_barrier";

export interface PaintPoint {
	x: number;
	y: number;
}

export interface PaintRequest {
	tool: PaintTool;
	brush_half_extent: 0 | 1 | 2;
	points: PaintPoint[];
}

export interface PaintStats {
	affected_cells: number;
	food_set_cells: number;
	food_cleared_cells: number;
	barrier_set_cells: number;
	barrier_cleared_cells: number;
	creatures_removed: number;
}

export interface PaintResponse {
	protocol_version: string;
	stats: PaintStats;
	frame: WsFrame;
}

// Request types
export interface StartupRequest {
	seed: number;
	population?: Partial<PopulationConfig>;
	world?: Partial<Omit<WorldConfig, "food">> & { food?: Partial<FoodConfig> };
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

// ── Execution Sampler types ──────────────────────────────────────────────────

export interface StaticInputsSnapshot {
	food_here: number;
	neighbor_food: number[];
	neighbor_barrier: number[];
	neighbor_occupied: number[];
	generation: number;
	age_ticks: number;
}

export interface VmStepTrace {
	pc: number;
	instruction: VmInstruction;
	energy_cost: number;
	energy_after: number;
	register_changes: [number, number][];
}

export interface MemoryWriteTrace {
	address: number;
	old_value: number;
	new_value: number;
}

export interface VmTrace {
	register_count: number;
	constants: number[];
	steps: VmStepTrace[];
	final_registers: number[];
	final_payload: number[];
	final_meta: number[];
	final_route_target: number;
	memory_writes: MemoryWriteTrace[];
}

export interface GraphNodeEvalTrace {
	node_index: number;
	kind: string;
	weighted_inputs: number[];
	weighted_sum: number;
	state_before: number;
	state_after: number;
	output: number;
}

export interface GraphPassTrace {
	pass_index: number;
	energy_cost: number;
	energy_after: number;
	node_evaluations: GraphNodeEvalTrace[];
	max_delta: number;
}

export interface GraphTrace {
	passes: GraphPassTrace[];
	converged: boolean;
	stable_passes_count: number;
	final_outputs: number[];
}

export type BackendTrace = { Vm: VmTrace } | { Graph: GraphTrace };

export interface MeshHopTrace {
	hop_index: number;
	node_id: number;
	input_refs: InputReference[];
	upstream_slots: number[];
	energy_before: number;
	energy_after: number;
	output_slots: number[];
	route_target_idx: number;
	backend_trace: BackendTrace;
}

export type TerminationReason =
	| "ActionEmitted"
	| "EnergyExhausted"
	| "MaxHopsReached"
	| "NoTargets"
	| "MissingNode";

export type WorldAction =
	| "NoOp"
	| "Eat"
	| { Move: string }
	| { Reproduce: { direction: string; offered_energy: number } };

export interface TickTrace {
	tick_number: number;
	energy_before: number;
	energy_after: number;
	static_inputs: StaticInputsSnapshot;
	hops: MeshHopTrace[];
	final_action: WorldAction;
	termination_reason: TerminationReason;
}

export interface ExecutionSample {
	creature_id: number;
	ticks: TickTrace[];
}

export type SampleResponse =
	| { protocol_version: string; status: "idle" }
	| {
			protocol_version: string;
			status: "recording";
			ticks_completed: number;
			ticks_remaining: number;
	  }
	| { protocol_version: string; status: "complete"; sample: ExecutionSample };

export interface StartSampleResponse {
	protocol_version: string;
	status: "recording";
	ticks_requested: number;
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
