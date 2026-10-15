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

/** A sink edge's membership in an edge group (T25.F03). */
export interface Membership {
	group: number;
}

export interface GraphEdge {
	source: GraphSource;
	/** The genomic weight of an ungrouped edge; a member's correction. */
	own_weight: number;
	/** Omitted by the server when the edge belongs to no group. */
	membership?: Membership;
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

/** One vote sink in the catalog; the number is a `Direction::ALL` index. A
 * `Drive` votes for its directed kind without naming a direction (T11.F30). */
export type VoteSink =
	| "Eat"
	| { Move: number }
	| { Reproduce: number }
	| { StealEnergy: number }
	| { Drive: DirectedVoteKind }
	| "Terminate"
	| "Decide";

/** An action kind carrying votes and a bar. */
export type VoteKind = "Eat" | "Move" | "Reproduce" | "StealEnergy";

/** One action-parameter field the commit decoder reads (T11.F27, T02.F08). */
export type ActionParamField =
	| "EatFoodType"
	| "EatLeaveShare"
	| "ReproduceTransferFraction"
	| "StealEnergyAmount";

/** Catalog order: the Graph sink order and the VM `field_idx` addressing. */
export const ACTION_PARAM_FIELDS: readonly ActionParamField[] = Object.freeze([
	"EatFoodType",
	"EatLeaveShare",
	"ReproduceTransferFraction",
	"StealEnergyAmount",
]);

export type OutputSinkKind =
	| { CustomOutput: number }
	| { RouterGate: number }
	| { WriteSlot: number }
	| { ClearSlot: number }
	| { ActionVote: VoteSink }
	| { ActionParam: ActionParamField };

export interface OutputSink {
	kind: OutputSinkKind;
	inputs: GraphEdge[];
}

/** A directed vote kind an edge group votes for (T25.F03). */
export type DirectedVoteKind = "Move" | "Reproduce" | "StealEnergy";

/** The geometry an edge group's members fit (T25.F03). */
export type GroupShape = { Ring: { ring_ref: number } };

/**
 * One edge-group record (T25.F03). A `Ring` group's members are sink edges:
 * ring cell `c` of `input_refs[ring_ref]` into `kind`'s direction
 * `(c + offset) mod 8`, each at genomic weight `scale + own_weight`.
 */
export interface EdgeGroup {
	shape: GroupShape;
	kind: DirectedVoteKind;
	offset: number;
	scale: number;
}

export interface GraphBackendDef {
	compute_nodes: ComputeNode[];
	output_sinks: OutputSink[];
	/** Edge-group records by `GroupId`; omitted by the server when empty. */
	groups?: EdgeGroup[];
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
	| { ReadInput: { dst: number; ref_idx: number; sub_idx: number } }
	| { WriteInternalPayload: { slot_idx: number; src: number } }
	| { WriteActionParam: { field_idx: number; src: number } }
	| { AddVote: { sink: number; src: number } }
	| { WriteRouteGate: { slot: number; src: number } }
	| { ReadActionQueueLength: { dst: number } }
	| { ReadActionQueueType: { index_src: number; dst: number } }
	| { ReadActionQueueParam: { index_src: number; param_slot: number; dst: number } }
	| { SetPriorityBid: { src: number } }
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

export type WorldInputReference =
	| string
	| { FoodHere: { type_idx: number } }
	| { NeighborFoodRing: { type_idx: number } }
	| { AreaFoodSummary: { type_idx: number } }
	| Record<string, unknown>;

export type InputReference =
	| { World: WorldInputReference }
	| { StaticIntrospection: string }
	| { DynamicIntrospection: string }
	| { UpstreamSlot: number }
	| "ActionQueue"
	| DecisionStateReference;

/** The in-tick decision state (T19.F05); `PreviousOutcome` is frozen perception. */
export type DecisionStateReference =
	| "ActionVotes"
	| "PreviousPassVotes"
	| "CommitCounts"
	| "PreviousOutcome";

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
