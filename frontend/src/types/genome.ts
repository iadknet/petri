import type { ActionLogEntry } from "./action-log.ts";

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
	| { UpstreamSlot: number }
	| "ActionQueue";

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
	genome_size: number;
	phenotype: CreaturePhenotype;
	/** Present unless excluded via `exclude=genome` query parameter. */
	genome?: CreatureGenome;
	/** Present unless excluded via `exclude=memory` query parameter. */
	memory?: number[];
	/** Present unless excluded via `exclude=action_log` query parameter. */
	action_log?: ActionLogEntry[];
	/** Tick of the most recent action_log entry (0 if log is empty). Always present. */
	latest_tick: number;
}
