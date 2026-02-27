export const PROTOCOL_VERSION = "v3alpha1" as const;

export type SimState = "idle" | "running" | "paused";

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
	mutation_events_applied_total_semantic_noop: number;
	mutation_events_applied_total_semantic_change: number;
	reproduction_actions_attempted_total: number;
	reproduction_actions_spawned_total: number;
	reproduction_actions_rejected_total: number;
	reproduction_actions_rejected_total_by_reason: Record<string, number>;
	mutation_events_skipped_total_by_reason: Record<string, number>;
	genome_complexity_mean: number;
	genome_complexity_min: number;
	genome_complexity_max: number;
}

export interface WsFrame {
	tick: number;
	status: StatusPayload;
	frame: Frame;
	health: HealthPayload;
}

export type WsEventType = "status" | "frame" | "health";

export interface WsEnvelope<T = unknown> {
	protocol_version: string;
	event: WsEventType;
	tick: number;
	payload: T;
}
