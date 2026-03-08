import type { ActionLogEntry } from "./action-log.ts";
import type { CreatureGenome, CreaturePhenotype } from "./genome.ts";

export type MeshReadClass =
	| "food"
	| "neighbor"
	| "barrier"
	| "occupancy"
	| "introspection"
	| "upstream"
	| "action_queue";

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
	/** Tick of the most recent action_log entry (0 if log is empty). Always present. */
	latest_tick: number;
}
