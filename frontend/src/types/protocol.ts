export const PROTOCOL_VERSION = "v3alpha2" as const;

export type SimState = "idle" | "running" | "paused";
export type ZoomTier = "overview" | "detail" | "inspect";
export type ByteArrayLike = Uint8Array | number[];

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
	type_idx: number;
	density: number;
}

export interface DetailFoodCellPayload {
	x: number;
	y: number;
	type_idx: number;
	density: number;
}

export interface OverviewFoodCellPayload {
	bucket_x: number;
	bucket_y: number;
	type_idx: number;
	density: number;
}

export interface FoodTypeMetadata {
	type_idx: number;
	name: string;
	color: string;
	growth_inhibitor: number;
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

export interface LastTickActions {
	move: number;
	eat: number;
	reproduce: number;
	noop: number;
	steal: number;
	predation_kills: number;
}

export interface PerfPayload {
	projection_publish_ms: number;
	ws_frame_publish_ms: number;
	subscriber_count: number;
}

export interface MutationTargetReachabilityTotalPayload {
	reachable: number;
	unreachable: number;
	/** Targets on nodes the parent executed recently (T11.F17); not displayed. */
	executed: number;
	not_applicable: number;
}

export interface StatusPayload {
	state: SimState;
	population: number;
	mean_energy: number;
	last_tick_actions: LastTickActions;
	reproduction_actions_attempted_total: number;
	reproduction_actions_spawned_total: number;
	reproduction_actions_rejected_total: number;
	predation_actions_attempted_total: number;
	predation_actions_transferred_total: number;
	predation_actions_rejected_total: number;
	predation_kills_total: number;
	predation_actions_by_result: Record<string, number>;
	mutation_events_attempted_total: number;
	mutation_events_applied_total: number;
	mutation_events_skipped_total: number;
	mutation_events_attempted_total_by_domain: Record<string, number>;
	mutation_events_applied_total_by_domain: Record<string, number>;
	mutation_events_attempted_total_by_operator: Record<string, number>;
	mutation_events_applied_total_by_operator: Record<string, number>;
	mutation_events_skipped_total_by_operator?: Record<string, number>;
	mutation_added_node_input_classes_total_by_operator?: Record<string, Record<string, number>>;
	mutation_added_node_world_inputs_total_by_operator?: Record<string, Record<string, number>>;
	mutation_target_reachability_total?: MutationTargetReachabilityTotalPayload;
	move_actions_blocked_total_by_cause?: Record<string, number>;
	reproduction_actions_rejected_invalid_target_total_by_cause?: Record<string, number>;
	vm_live_read_world_inputs_current?: Record<string, number>;
	last_tick_compute_energy_total_mean: number;
	last_tick_compute_energy_total_min: number;
	last_tick_compute_energy_total_max: number;
	last_tick_compute_energy_vm_mean: number;
	last_tick_compute_energy_graph_mean: number;
	perf: PerfPayload;
}

export interface HealthPayload {
	population: number;
	mean_energy: number;
	mutation_events_attempted_total: number;
	mutation_events_applied_total: number;
	mutation_events_skipped_total: number;
	mutation_events_attempted_total_by_domain: Record<string, number>;
	mutation_events_applied_total_by_domain: Record<string, number>;
	mutation_events_attempted_total_by_operator: Record<string, number>;
	mutation_events_applied_total_by_operator: Record<string, number>;
	mutation_events_skipped_total_by_operator?: Record<string, number>;
	mutation_added_node_input_classes_total_by_operator?: Record<string, Record<string, number>>;
	mutation_added_node_world_inputs_total_by_operator?: Record<string, Record<string, number>>;
	mutation_target_reachability_total?: MutationTargetReachabilityTotalPayload;
	move_actions_blocked_total_by_cause?: Record<string, number>;
	reproduction_actions_rejected_invalid_target_total_by_cause?: Record<string, number>;
	vm_live_read_world_inputs_current?: Record<string, number>;
	reproduction_actions_attempted_total: number;
	reproduction_actions_spawned_total: number;
	reproduction_actions_rejected_total: number;
	reproduction_actions_rejected_total_by_reason: Record<string, number>;
	mutation_events_skipped_total_by_reason: Record<string, number>;
	predation_actions_attempted_total: number;
	predation_actions_transferred_total: number;
	predation_actions_rejected_total: number;
	predation_kills_total: number;
	predation_actions_by_result: Record<string, number>;
	genome_complexity_mean: number;
	genome_complexity_min: number;
	genome_complexity_max: number;
}

export interface PredationEvent {
	attacker_x: number;
	attacker_y: number;
	victim_x: number;
	victim_y: number;
	energy_stolen: number;
	killed: boolean;
}

export interface WorldStaticPayload {
	width: number;
	height: number;
	barrier_mask: ByteArrayLike;
	food_types: FoodTypeMetadata[];
	/** Quantized fertility overlay for primary food type (`type_idx = 0`). */
	food_fertility_u8?: ByteArrayLike;
}

export interface ViewRect {
	x: number;
	y: number;
	width: number;
	height: number;
}

export interface ViewOverviewPayload {
	rect: ViewRect;
	grid_width: number;
	grid_height: number;
	food: OverviewFoodCellPayload[];
	food_fertility_u8?: ByteArrayLike;
	creature_count_u16: number[];
}

export interface ViewDetailPayload {
	rect: ViewRect;
	width: number;
	height: number;
	food: DetailFoodCellPayload[];
	food_fertility_u8?: ByteArrayLike;
	creatures: Creature[];
	predation_events: PredationEvent[];
}

export type SnapshotView =
	| ({ kind: "overview" } & ViewOverviewPayload)
	| ({ kind: "detail" } & ViewDetailPayload);

export interface SubscribeViewMessage {
	type: "subscribe_view";
	request_id: number;
	x: number;
	y: number;
	width: number;
	height: number;
	canvas_width: number;
	canvas_height: number;
	zoom_tier: ZoomTier;
}

export interface UnsubscribeViewMessage {
	type: "unsubscribe_view";
}

export type ClientMessage = SubscribeViewMessage | UnsubscribeViewMessage;

interface ServerMessageBase {
	protocol_version: string;
	projection_revision: number;
	world_static_revision: number;
	tick: number;
}

export interface StatusServerMessage extends ServerMessageBase {
	type: "status";
	payload: StatusPayload;
}

export interface HealthServerMessage extends ServerMessageBase {
	type: "health";
	payload: HealthPayload;
}

export interface WorldStaticServerMessage extends ServerMessageBase {
	type: "world_static";
	payload: WorldStaticPayload;
}

export interface ViewOverviewServerMessage extends ServerMessageBase {
	type: "view_overview";
	request_id: number;
	payload: ViewOverviewPayload;
}

export interface ViewDetailServerMessage extends ServerMessageBase {
	type: "view_detail";
	request_id: number;
	payload: ViewDetailPayload;
}

export type ServerMessage =
	| StatusServerMessage
	| HealthServerMessage
	| WorldStaticServerMessage
	| ViewOverviewServerMessage
	| ViewDetailServerMessage;
