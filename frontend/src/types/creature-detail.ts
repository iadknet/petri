import type { ActionLogEntry } from "./action-log.ts";
import type { CreatureGenome, CreaturePhenotype } from "./genome.ts";

export type MeshReadClass =
	| "food"
	| "neighbor"
	| "barrier"
	| "occupancy"
	| "introspection"
	| "upstream"
	| "action_queue"
	| "decision";

export type MeshWriteClass = "route" | "action" | "memory" | "payload";

export interface CreatureMeshAnnotation {
	node_id: number;
	reachable: boolean;
	read_classes: MeshReadClass[];
	write_classes: MeshWriteClass[];
	has_stateful_behavior: boolean;
	live_instruction_indices?: number[];
	live_internal_node_indices?: number[];
}

export interface CreatureCurrentInputsDiagnostics {
	food_here: number;
	neighbor_food: number[];
	neighbor_barrier: number[];
	neighbor_occupied: number[];
}

export interface CreatureLiveCircuitDiagnostics {
	reachable_node_count: number;
	stateful_reachable_node_count: number;
	barrier_reader_reachable_node_count: number;
	distinct_upstream_slots_read: number[];
	distinct_payload_slots_written: number[];
	distinct_custom_output_slots_written: number[];
	reachable_read_class_counts: Record<string, number>;
	reachable_write_class_counts: Record<string, number>;
}

export interface CreatureRecentActionsDiagnostics {
	sampled_entries: number;
	blocked_move_count: number;
	invalid_target_reproduce_count: number;
	by_action_result: Record<string, number>;
}

export interface CreatureDiagnostics {
	current_inputs: CreatureCurrentInputsDiagnostics;
	live_circuit: CreatureLiveCircuitDiagnostics;
	recent_actions: CreatureRecentActionsDiagnostics;
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
	genome_size: number;
	phenotype: CreaturePhenotype;
	/** Present unless excluded via `exclude=genome` query parameter. */
	genome?: CreatureGenome;
	/** Present whenever `genome` is included in the response. */
	mesh_annotations?: CreatureMeshAnnotation[];
	/** Present unless excluded via `exclude=shared_memory` query parameter. */
	shared_memory?: number[];
	/** Present unless excluded via `exclude=action_log` query parameter. */
	action_log?: ActionLogEntry[];
	/** Present unless excluded via `exclude=diagnostics` query parameter. */
	diagnostics?: CreatureDiagnostics;
	/** Tick of the most recent action_log entry (0 if log is empty). Always present. */
	latest_tick: number;
}
