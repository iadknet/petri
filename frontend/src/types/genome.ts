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
	| { InputRef: { ref_idx: number; sub_idx: number } }
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
	| "RouterOutput"
	| { WriteActionMeta: number }
	| { PushAction: number }
	| "PopAction"
	| "ExecuteActionQueue"
	| { ReadSlot: number }
	| { ReadSlotPrev: number }
	| { WriteSlot: number }
	| { ClearSlot: number };

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
	| { WriteRouteTarget: { src: number } }
	| { PushAction: { action_type: number } }
	| "PopAction"
	| { ReadActionQueueLength: { dst: number } }
	| { ReadActionQueueType: { index_src: number; dst: number } }
	| { ReadActionQueueParam: { index_src: number; param_slot: number; dst: number } }
	| { SetPriorityBid: { src: number } }
	| "ExecuteActionQueue"
	| "Halt"
	| { LoadSlot: { dst: number; slot_reg: number } }
	| { StoreSlot: { slot_reg: number; src: number } }
	| { LoadSlotImm: { dst: number; slot_idx: number } }
	| { StoreSlotImm: { slot_idx: number; src: number } }
	| { LoadSlotPrev: { dst: number; slot_idx: number } }
	| { ClearSlot: { slot_idx: number } };

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
