pub mod analysis;

use crate::contracts::{InputReference, NodeId};

/// A single VM instruction. 39 opcodes per v3-vm-isa-spec.md.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum VmInstruction {
    // ── Arithmetic and Data Movement ─────────────────────────────────────────
    /// No operation.
    Noop,
    /// `dst = constants[const_idx]`
    LoadConst { dst: u8, const_idx: u8 },
    /// `dst = src`
    Move { dst: u8, src: u8 },
    /// `dst = a + b`
    Add { dst: u8, a: u8, b: u8 },
    /// `dst = a - b`
    Sub { dst: u8, a: u8, b: u8 },
    /// `dst = a * b`
    Mul { dst: u8, a: u8, b: u8 },
    /// `dst = if b == 0 { 0.0 } else { a / b }`
    Div { dst: u8, a: u8, b: u8 },
    /// `dst = min(a, b)`
    Min { dst: u8, a: u8, b: u8 },
    /// `dst = max(a, b)`
    Max { dst: u8, a: u8, b: u8 },
    /// `dst = abs(src)`
    Abs { dst: u8, src: u8 },
    /// `dst = -src`
    Neg { dst: u8, src: u8 },
    /// `dst = clamp(src, 0.0, 1.0)`
    Clamp01 { dst: u8, src: u8 },

    // ── Comparison and Logic ─────────────────────────────────────────────────
    /// `dst = if a > b { 1.0 } else { 0.0 }`
    CmpGt { dst: u8, a: u8, b: u8 },
    /// `dst = if a < b { 1.0 } else { 0.0 }`
    CmpLt { dst: u8, a: u8, b: u8 },
    /// `dst = if abs(a-b) <= clamp_eps(eps) { 1.0 } else { 0.0 }`
    CmpEq { dst: u8, a: u8, b: u8, eps: u8 },
    /// Boolean `and` using `>= 0.5` truthiness.
    And { dst: u8, a: u8, b: u8 },
    /// Boolean `or` using `>= 0.5` truthiness.
    Or { dst: u8, a: u8, b: u8 },
    /// Boolean `not` using `>= 0.5` truthiness.
    Not { dst: u8, src: u8 },

    // ── Type Conversion ───────────────────────────────────────────────────────
    /// Round ties-away-from-zero, store as f32.
    ToI32 { dst: u8, src: u8 },
    /// Clamp `[0, 255]`, round ties-away-from-zero, store as f32.
    ToU8 { dst: u8, src: u8 },
    /// `dst = if truthy(src) { 1.0 } else { 0.0 }`
    ToBool { dst: u8, src: u8 },

    // ── Control Flow ─────────────────────────────────────────────────────────
    /// If `!truthy(cond)` jump by signed offset (wraps via rem_euclid over program_len).
    JumpIfZero { cond: u8, offset: i32 },
    /// Unconditional jump by signed offset.
    Jump { offset: i32 },

    // ── Input Reads ───────────────────────────────────────────────────────────
    /// `dst = resolve(input_refs[ref_idx], sub_idx)`; invalid index yields `0.0`.
    ReadInput { dst: u8, ref_idx: u16, sub_idx: u16 },

    // ── Output and Routing Writes ─────────────────────────────────────────────
    /// Overwrite internal payload slot; invalid slot write ignored.
    WriteInternalPayload { slot_idx: u8, src: u8 },
    /// Write world-action metadata slot (0..7); invalid slot write ignored.
    WriteWorldActionMeta { slot_idx: u8, src: u8 },
    /// Write candidate route target value.
    WriteRouteTarget { src: u8 },

    // ── Action Queue ──────────────────────────────────────────────────────────
    /// Decode meta buffer and push action onto queue. Silent no-op if at cap.
    PushAction { action_type: u8 },
    /// Remove last action from queue. No-op if empty.
    PopAction,
    /// Write `queue.len() as f32` to register `dst`.
    ReadActionQueueLength { dst: u8 },
    /// Write action type discriminant at `queue[reg[index_src]]` to register `dst`.
    ReadActionQueueType { index_src: u8, dst: u8 },
    /// Write action param at `queue[reg[index_src]].param(param_slot)` to register `dst`.
    ReadActionQueueParam {
        index_src: u8,
        param_slot: u8,
        dst: u8,
    },
    /// Set the creature's priority bid for turn-order execution.
    /// Reads `regs[src]`, clamps to non-negative, deducts from energy.
    /// Last-write-wins if called multiple times.
    SetPriorityBid { src: u8 },
    /// Terminal: return accumulated action queue for execution.
    ExecuteActionQueue,

    // ── Halt and Memory ───────────────────────────────────────────────────────
    /// Stop VM execution without emitting a world action.
    Halt,
    /// `dst = memory[addr_reg % 1024]`
    LoadMem8 { dst: u8, addr_reg: u8 },
    /// `memory[addr_reg % 1024] = truncate(src)`
    StoreMem8 { addr_reg: u8, src: u8 },
    /// `dst = memory[imm_addr % 1024]`
    LoadMem8Imm { dst: u8, imm_addr: u16 },
    /// `memory[imm_addr % 1024] = truncate(src)`
    StoreMem8Imm { imm_addr: u16, src: u8 },
}

/// VM backend definition for a mesh node.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct VmBackendDef {
    /// Number of general-purpose f32 registers. If 0, VM halts immediately.
    pub register_count: u8,
    /// Constant pool; indexed by `LoadConst.const_idx` with rem_euclid wrapping.
    pub constants: Vec<f32>,
    /// Instruction sequence. PC starts at 0.
    pub program: Vec<VmInstruction>,
}

impl VmBackendDef {
    /// Returns `true` if any instruction in the program reads or writes memory.
    pub fn has_memory_ops(&self) -> bool {
        self.program.iter().any(|instr| {
            matches!(
                instr,
                VmInstruction::LoadMem8 { .. }
                    | VmInstruction::StoreMem8 { .. }
                    | VmInstruction::LoadMem8Imm { .. }
                    | VmInstruction::StoreMem8Imm { .. }
            )
        })
    }
}

// ── Graph backend types ───────────────────────────────────────────────────────

/// Hebbian learning rule variant controlling how edge weights adapt at runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum HebbianRule {
    /// dw = eta * pre * post
    Classic,
    /// dw = eta * post * (pre - w * post)  (normalizing)
    Oja,
    /// dw = -eta * pre * post
    AntiHebb,
    /// dw = eta * (pre - 0.5) * (post - 0.5)
    Covariance,
}

/// Per-node Hebbian learning configuration stored in the genome.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct HebbianConfig {
    pub rule: HebbianRule,
    /// Learning rate, clamped to [0.0, 1.0] at runtime.
    pub learning_rate: f32,
    /// Symmetric weight clamp magnitude, clamped to [0.01, 10.0] at runtime.
    pub weight_clamp: f32,
    /// If true, offspring inherit learned weights (Lamarckian); otherwise
    /// offspring start from genome birth weights (Darwinian).
    pub lamarckian: bool,
}

/// A weighted edge in a graph internal node's input list.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GraphInput {
    /// Source internal node index. Out-of-range -> 0.0 (soft default).
    pub source_idx: u16,
    pub weight: f32,
}

/// The kind of operation performed by a graph internal node.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum GraphNodeKind {
    /// Read from `NodeGenome.input_refs[ref_idx]` with `sub_idx` for compound inputs.
    InputRef {
        ref_idx: u16,
        sub_idx: u16,
    },
    /// Constant output value (ignores inputs).
    Constant(f32),
    /// Sum all weighted inputs.
    Add,
    /// Multiply all weighted inputs.
    Multiply,
    Negate,
    Abs,
    Min,
    Max,
    /// Threshold gate: 1.0 if input > threshold, else 0.0.
    Threshold(f32),
    GreaterThan,
    Sigmoid,
    Tanh,
    Relu,
    /// Ternary select using first input as condition.
    Select,
    Clamp01,
    WeightedSum,
    // ── Stateful operators (persist across ticks via graph_state) ─────────────
    DecayIntegrator(f32),
    Momentum(f32),
    Oscillator(f32),
    AdaptiveGain,
    // ── Output writers ────────────────────────────────────────────────────────
    /// Write computed value to `output_slots[u8]`.
    CustomOutput(u8),
    /// Write computed value to `route_target_idx`.
    RouterOutput,
}

/// A single internal node in the graph backend.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GraphInternalNode {
    pub kind: GraphNodeKind,
    pub inputs: Vec<GraphInput>,
    /// Per-node Hebbian learning config. `None` = immutable weights (default).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hebbian: Option<HebbianConfig>,
}

/// Graph backend definition for a mesh node.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GraphBackendDef {
    pub internal_nodes: Vec<GraphInternalNode>,
}

// ── Top-level genome types ────────────────────────────────────────────────────

/// Backend definition: either VM or Graph.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum BackendDef {
    Vm(VmBackendDef),
    Graph(GraphBackendDef),
}

impl BackendDef {
    /// Offset CustomOutput slot indices in Graph backends. No-op for VM.
    /// Only remaps in-range slots (0..12). Out-of-range slots are junk
    /// (runtime ignores writes to slot >= 12) and left untouched to
    /// avoid u8 overflow.
    pub fn remap_output_slots(&mut self, offset: u8) {
        if let BackendDef::Graph(ref mut g) = self {
            for node in &mut g.internal_nodes {
                if let GraphNodeKind::CustomOutput(ref mut slot) = node.kind {
                    if *slot < 12 {
                        *slot = (*slot + offset) % 12;
                    }
                }
            }
        }
    }
}

/// A single node in the creature genome.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NodeGenome {
    pub node_id: NodeId,
    /// Shared input slot references for both VM and Graph backends.
    pub input_refs: Vec<InputReference>,
    pub backend_def: BackendDef,
    /// Candidate routing targets. May contain dangling IDs (junk DNA).
    pub targets: Vec<NodeId>,
}

/// The complete genome of a creature.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CreatureGenome {
    /// Node to begin mesh execution from each tick.
    pub entry_node_id: NodeId,
    pub nodes: Vec<NodeGenome>,
}

impl CreatureGenome {
    /// Look up a node by its ID. O(n) — genomes are small.
    pub fn find_node(&self, id: NodeId) -> Option<&NodeGenome> {
        self.nodes.iter().find(|n| n.node_id == id)
    }

    /// Structural complexity score: sum of all discrete genome components.
    ///
    /// Counts every node, input ref, target, instruction, constant, and
    /// graph internal node (plus its inputs). Equal weights, includes junk DNA,
    /// no reachability analysis.
    pub fn complexity(&self) -> u32 {
        let mut score: u32 = 0;
        for node in &self.nodes {
            // Count the node itself
            score += 1;
            // Count input refs and targets
            score += node.input_refs.len() as u32;
            score += node.targets.len() as u32;
            // Count backend components
            match &node.backend_def {
                BackendDef::Vm(vm) => {
                    score += vm.program.len() as u32;
                    score += vm.constants.len() as u32;
                }
                BackendDef::Graph(graph) => {
                    score += graph.internal_nodes.len() as u32;
                    for inode in &graph.internal_nodes {
                        score += inode.inputs.len() as u32;
                    }
                }
            }
        }
        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::{DynamicIntrospectionKey, StaticIntrospectionKey, WorldInputKey};

    #[test]
    fn vm_instruction_all_39_variants_constructible() {
        let instructions: Vec<VmInstruction> = vec![
            VmInstruction::Noop,
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            },
            VmInstruction::Move { dst: 0, src: 1 },
            VmInstruction::Add { dst: 0, a: 1, b: 2 },
            VmInstruction::Sub { dst: 0, a: 1, b: 2 },
            VmInstruction::Mul { dst: 0, a: 1, b: 2 },
            VmInstruction::Div { dst: 0, a: 1, b: 2 },
            VmInstruction::Min { dst: 0, a: 1, b: 2 },
            VmInstruction::Max { dst: 0, a: 1, b: 2 },
            VmInstruction::Abs { dst: 0, src: 1 },
            VmInstruction::Neg { dst: 0, src: 1 },
            VmInstruction::Clamp01 { dst: 0, src: 1 },
            VmInstruction::CmpGt { dst: 0, a: 1, b: 2 },
            VmInstruction::CmpLt { dst: 0, a: 1, b: 2 },
            VmInstruction::CmpEq {
                dst: 0,
                a: 1,
                b: 2,
                eps: 3,
            },
            VmInstruction::And { dst: 0, a: 1, b: 2 },
            VmInstruction::Or { dst: 0, a: 1, b: 2 },
            VmInstruction::Not { dst: 0, src: 1 },
            VmInstruction::ToI32 { dst: 0, src: 1 },
            VmInstruction::ToU8 { dst: 0, src: 1 },
            VmInstruction::ToBool { dst: 0, src: 1 },
            VmInstruction::JumpIfZero { cond: 0, offset: 2 },
            VmInstruction::Jump { offset: -1 },
            VmInstruction::ReadInput {
                dst: 0,
                ref_idx: 0,
                sub_idx: 0,
            },
            VmInstruction::WriteInternalPayload {
                slot_idx: 0,
                src: 1,
            },
            VmInstruction::WriteWorldActionMeta {
                slot_idx: 0,
                src: 1,
            },
            VmInstruction::WriteRouteTarget { src: 0 },
            VmInstruction::PushAction { action_type: 1 },
            VmInstruction::PopAction,
            VmInstruction::ReadActionQueueLength { dst: 0 },
            VmInstruction::ReadActionQueueType {
                index_src: 0,
                dst: 0,
            },
            VmInstruction::ReadActionQueueParam {
                index_src: 0,
                param_slot: 0,
                dst: 0,
            },
            VmInstruction::SetPriorityBid { src: 0 },
            VmInstruction::ExecuteActionQueue,
            VmInstruction::Halt,
            VmInstruction::LoadMem8 {
                dst: 0,
                addr_reg: 1,
            },
            VmInstruction::StoreMem8 {
                addr_reg: 0,
                src: 1,
            },
            VmInstruction::LoadMem8Imm {
                dst: 0,
                imm_addr: 100,
            },
            VmInstruction::StoreMem8Imm {
                imm_addr: 200,
                src: 1,
            },
        ];
        assert_eq!(instructions.len(), 39, "must have exactly 39 opcodes");
    }

    #[test]
    fn vm_instruction_serde_roundtrip() {
        let instr = VmInstruction::CmpEq {
            dst: 0,
            a: 1,
            b: 2,
            eps: 3,
        };
        let json = serde_json::to_string(&instr).unwrap();
        let instr2: VmInstruction = serde_json::from_str(&json).unwrap();
        assert_eq!(instr, instr2);
    }

    #[test]
    fn graph_node_kinds_all_22_constructible() {
        let kinds: Vec<GraphNodeKind> = vec![
            GraphNodeKind::InputRef {
                ref_idx: 0,
                sub_idx: 0,
            },
            GraphNodeKind::Constant(1.0),
            GraphNodeKind::Add,
            GraphNodeKind::Multiply,
            GraphNodeKind::Negate,
            GraphNodeKind::Abs,
            GraphNodeKind::Min,
            GraphNodeKind::Max,
            GraphNodeKind::Threshold(0.5),
            GraphNodeKind::GreaterThan,
            GraphNodeKind::Sigmoid,
            GraphNodeKind::Tanh,
            GraphNodeKind::Relu,
            GraphNodeKind::Select,
            GraphNodeKind::Clamp01,
            GraphNodeKind::WeightedSum,
            GraphNodeKind::DecayIntegrator(0.9),
            GraphNodeKind::Momentum(0.1),
            GraphNodeKind::Oscillator(1.0),
            GraphNodeKind::AdaptiveGain,
            GraphNodeKind::CustomOutput(0),
            GraphNodeKind::RouterOutput,
        ];
        assert_eq!(kinds.len(), 22);
    }

    #[test]
    fn creature_genome_find_node() {
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![
                NodeGenome {
                    node_id: NodeId::new(0),
                    input_refs: vec![],
                    backend_def: BackendDef::Vm(VmBackendDef {
                        register_count: 1,
                        constants: vec![],
                        program: vec![VmInstruction::Halt],
                    }),
                    targets: vec![],
                },
                NodeGenome {
                    node_id: NodeId::new(1),
                    input_refs: vec![],
                    backend_def: BackendDef::Vm(VmBackendDef {
                        register_count: 1,
                        constants: vec![],
                        program: vec![
                            VmInstruction::PushAction { action_type: 0 },
                            VmInstruction::ExecuteActionQueue,
                        ],
                    }),
                    targets: vec![],
                },
            ],
        };
        assert!(genome.find_node(NodeId::new(0)).is_some());
        assert!(genome.find_node(NodeId::new(1)).is_some());
        assert!(genome.find_node(NodeId::new(99)).is_none());
    }

    #[test]
    fn complexity_empty_genome() {
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![],
        };
        assert_eq!(genome.complexity(), 0);
    }

    #[test]
    fn complexity_single_vm_node() {
        // 1 node + 2 input_refs + 1 target + 3 program + 2 constants = 9
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![
                    InputReference::DynamicIntrospection(DynamicIntrospectionKey::EnergyCurrent),
                    InputReference::StaticIntrospection(StaticIntrospectionKey::AgeTicks),
                ],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 4,
                    constants: vec![1.0, 2.0],
                    program: vec![
                        VmInstruction::LoadConst {
                            dst: 0,
                            const_idx: 0,
                        },
                        VmInstruction::Add { dst: 0, a: 0, b: 1 },
                        VmInstruction::Halt,
                    ],
                }),
                targets: vec![NodeId::new(1)],
            }],
        };
        assert_eq!(genome.complexity(), 9);
    }

    #[test]
    fn complexity_single_graph_node() {
        // 1 node + 1 input_ref + 0 targets + 2 internal_nodes + (1 + 2) inputs = 7
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![InputReference::World(WorldInputKey::FoodHere)],
                backend_def: BackendDef::Graph(GraphBackendDef {
                    internal_nodes: vec![
                        GraphInternalNode {
                            kind: GraphNodeKind::InputRef {
                                ref_idx: 0,
                                sub_idx: 0,
                            },
                            inputs: vec![GraphInput {
                                source_idx: 0,
                                weight: 1.0,
                            }],
                            hebbian: None,
                        },
                        GraphInternalNode {
                            kind: GraphNodeKind::Sigmoid,
                            inputs: vec![
                                GraphInput {
                                    source_idx: 0,
                                    weight: 0.5,
                                },
                                GraphInput {
                                    source_idx: 1,
                                    weight: -0.3,
                                },
                            ],
                            hebbian: None,
                        },
                    ],
                }),
                targets: vec![],
            }],
        };
        assert_eq!(genome.complexity(), 7);
    }

    #[test]
    fn complexity_multi_node_mixed() {
        // Node 0 (VM): 1 + 0 inputs + 1 target + 1 program + 0 constants = 3
        // Node 1 (Graph): 1 + 1 input + 0 targets + 1 internal + 0 graph_inputs = 3
        // Total = 6
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![
                NodeGenome {
                    node_id: NodeId::new(0),
                    input_refs: vec![],
                    backend_def: BackendDef::Vm(VmBackendDef {
                        register_count: 1,
                        constants: vec![],
                        program: vec![VmInstruction::Halt],
                    }),
                    targets: vec![NodeId::new(1)],
                },
                NodeGenome {
                    node_id: NodeId::new(1),
                    input_refs: vec![InputReference::World(WorldInputKey::FoodHere)],
                    backend_def: BackendDef::Graph(GraphBackendDef {
                        internal_nodes: vec![GraphInternalNode {
                            kind: GraphNodeKind::Constant(1.0),
                            inputs: vec![],
                            hebbian: None,
                        }],
                    }),
                    targets: vec![],
                },
            ],
        };
        assert_eq!(genome.complexity(), 6);
    }

    // ── Gap 5: remap_output_slots tests ──

    #[test]
    fn remap_output_slots_offsets_graph_custom_outputs() {
        let mut backend = BackendDef::Graph(GraphBackendDef {
            internal_nodes: vec![GraphInternalNode {
                kind: GraphNodeKind::CustomOutput(2),
                inputs: vec![],
                hebbian: None,
            }],
        });
        backend.remap_output_slots(3);
        if let BackendDef::Graph(ref g) = backend {
            assert_eq!(
                g.internal_nodes[0].kind,
                GraphNodeKind::CustomOutput(5),
                "2 + 3 = 5"
            );
        }
    }

    #[test]
    fn remap_output_slots_wraps_within_12() {
        let mut backend = BackendDef::Graph(GraphBackendDef {
            internal_nodes: vec![GraphInternalNode {
                kind: GraphNodeKind::CustomOutput(10),
                inputs: vec![],
                hebbian: None,
            }],
        });
        backend.remap_output_slots(5);
        if let BackendDef::Graph(ref g) = backend {
            assert_eq!(
                g.internal_nodes[0].kind,
                GraphNodeKind::CustomOutput(3),
                "10 + 5 = 15, 15 % 12 = 3"
            );
        }
    }

    #[test]
    fn remap_output_slots_skips_out_of_range_slots() {
        let mut backend = BackendDef::Graph(GraphBackendDef {
            internal_nodes: vec![GraphInternalNode {
                kind: GraphNodeKind::CustomOutput(200),
                inputs: vec![],
                hebbian: None,
            }],
        });
        backend.remap_output_slots(5);
        if let BackendDef::Graph(ref g) = backend {
            assert_eq!(
                g.internal_nodes[0].kind,
                GraphNodeKind::CustomOutput(200),
                "out-of-range slots must be left untouched"
            );
        }
    }

    #[test]
    fn remap_output_slots_noop_for_vm() {
        let mut backend = BackendDef::Vm(VmBackendDef {
            register_count: 1,
            constants: vec![],
            program: vec![VmInstruction::Halt],
        });
        let before = backend.clone();
        backend.remap_output_slots(5);
        assert_eq!(backend, before, "VM backend must be unchanged");
    }

    #[test]
    fn creature_genome_serde_roundtrip() {
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 2,
                    constants: vec![1.0, 2.0],
                    program: vec![
                        VmInstruction::Add { dst: 0, a: 1, b: 1 },
                        VmInstruction::Halt,
                    ],
                }),
                targets: vec![],
            }],
        };
        let json = serde_json::to_string(&genome).unwrap();
        let genome2: CreatureGenome = serde_json::from_str(&json).unwrap();
        assert_eq!(genome, genome2);
    }
}
