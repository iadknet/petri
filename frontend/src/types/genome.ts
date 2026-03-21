export interface CreaturePhenotype {
	channels: [number, number, number, number, number, number];
	active_channel: number;
	polarity: [boolean, boolean, boolean, boolean, boolean, boolean];
	rgb: [number, number, number];
}

// ── CGP Graph Backend Types ─────────────────────────────────────────────────

export type GraphSource =
	| { InputLeaf: { ref_idx: number; sub_idx: number } }
	| { SharedMemory: { slot: number; previous: boolean } }
	| { ComputeNode: number };

export interface GraphEdge {
	source: GraphSource;
	weight: number;
}

export type ComputeNodeKind =
	| "Add"
	| "Multiply"
	| "Negate"
	| "Abs"
	| "Min"
	| "Max"
	| "WeightedSum"
	| "Sigmoid"
	| "Tanh"
	| "Relu"
	| "Clamp01"
	| { Threshold: number }
	| "GreaterThan"
	| "Select"
	| { DecayIntegrator: number }
	| { Momentum: number }
	| { Oscillator: number }
	| "AdaptiveGain"
	| { Constant: number };

export type HebbianRule = "Classic" | "Oja" | "AntiHebb" | "Covariance";
export type OutcomeChannel = "EnergyDelta" | "ActionSuccess" | "DamageDelta" | "OffspringSuccess";

export interface RewardModulationConfig {
	reward_source: OutcomeChannel;
	trace_decay: number;
}

export interface PlasticityConfig {
	rule: HebbianRule;
	learning_rate: number;
	weight_clamp: number;
	lamarckian: boolean;
	modulation?: RewardModulationConfig | null;
}

export interface ComputeNode {
	kind: ComputeNodeKind;
	inputs: GraphEdge[];
	plasticity?: PlasticityConfig | null;
}

export type OutputSinkKind =
	| { CustomOutput: number }
	| { RouterGate: number }
	| { WriteSlot: number }
	| { ClearSlot: number };

export interface OutputSink {
	kind: OutputSinkKind;
	inputs: GraphEdge[];
}

export type WorldActionKind = "Eat" | "Move" | "Reproduce" | "StealEnergy" | "NoOp";

export type ActionSlotBehavior = "Pop" | { Emit: WorldActionKind };

export interface ActionSlot {
	behavior: ActionSlotBehavior;
	gate_inputs: GraphEdge[];
	param_inputs: GraphEdge[];
}

export interface ExecuteGate {
	inputs: GraphEdge[];
}

export interface GraphBackendDef {
	compute_nodes: ComputeNode[];
	output_sinks: OutputSink[];
	action_bank: ActionSlot[];
	execute_gate: ExecuteGate;
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
	| { WriteRouteGate: { slot: number; src: number } }
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
	| { World: string }
	| { StaticIntrospection: string }
	| { DynamicIntrospection: string }
	| { UpstreamSlot: number }
	| "ActionQueue";

export interface RouteTarget {
	target_id: number;
	slot: number;
	gate_bias: number;
}

export interface NodeGenome {
	node_id: number;
	input_refs: InputReference[];
	backend_def: BackendDef;
	targets: RouteTarget[];
}

export interface CreatureGenome {
	entry_node_id: number;
	nodes: NodeGenome[];
}
