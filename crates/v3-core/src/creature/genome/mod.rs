pub mod analysis;
pub mod cgp;
pub(crate) mod cgp_analysis;
pub(crate) mod cgp_mesh_annotations;
pub mod companions;
pub mod mesh_annotations;
pub mod vote;

pub use companions::{structural_companions, StructuralCompanions};
pub use vote::{VoteKind, VoteSink, VoteVector, VOTE_KIND_COUNT, VOTE_SINK_COUNT};

use crate::contracts::{InputReference, NodeId, RouteTarget};

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
    /// Overwrite parameter-surface slot `params[slot_idx / 2][slot_idx % 2]`
    /// (`slot_idx` in `0..8`, T19.F04); an invalid slot write is ignored.
    WriteActionParam { slot_idx: u8, src: u8 },
    /// Add `regs[src]` to this dispatch's vote for `VoteSink::from_index(sink)`
    /// (T19.F03); a sink at or above the catalog count writes nothing and still
    /// costs. The dispatch commits its vote at every exit but energy
    /// exhaustion; the pass end reads the summed votes (T19.F04).
    AddVote { sink: u8, src: u8 },
    /// Write a route gate score for a specific target slot.
    WriteRouteGate { slot: u8, src: u8 },

    // ── Action Queue Reads ────────────────────────────────────────────────────
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
    /// Reads `regs[src]`, clamps to non-negative, and records it; the mesh
    /// settles the bid once at evaluation end (T19.F02).
    /// Last-write-wins if called multiple times.
    SetPriorityBid { src: u8 },

    // ── Halt and Shared Memory Slots ─────────────────────────────────────────
    /// End the dispatch; the mesh routes on the written gates.
    Halt,
    /// `dst = shared_memory[regs[slot_reg] % 16]`
    LoadSlot { dst: u8, slot_reg: u8 },
    /// `shared_memory[regs[slot_reg] % 16] = sanitize_f32(regs[src])`
    StoreSlot { slot_reg: u8, src: u8 },
    /// `dst = shared_memory[slot_idx % 16]`
    LoadSlotImm { dst: u8, slot_idx: u8 },
    /// `shared_memory[slot_idx % 16] = sanitize_f32(regs[src])`
    StoreSlotImm { slot_idx: u8, src: u8 },
    /// `dst = prev_shared_memory[slot_idx % 16]`
    LoadSlotPrev { dst: u8, slot_idx: u8 },
    /// `shared_memory[slot_idx % 16] = 0.0`
    ClearSlot { slot_idx: u8 },
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

/// Outcome channel for reward-modulated learning.
///
/// Describes **what happened** (not what should matter). Evolution discovers
/// which circuits listen to which signals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum OutcomeChannel {
    EnergyDelta = 0,
    ActionSuccess = 1,
    DamageDelta = 2,
    OffspringSuccess = 3,
}

/// Number of outcome channels.
pub const OUTCOME_CHANNEL_COUNT: usize = 4;

/// Reward modulation config for three-factor plasticity.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RewardModulationConfig {
    pub reward_source: OutcomeChannel,
    /// Eligibility retention per world tick, clamped to [0.0, 1.0] at runtime.
    pub trace_decay: f32,
}

/// Per-node plasticity configuration stored in the genome.
///
/// Governs both pure Hebbian and reward-modulated (three-factor) learning.
/// When `modulation` is `None`, this is pure Hebbian learning. When `Some`,
/// activity without the learning-rate factor replaces the tick contribution to an eligibility trace; weight
/// updates are deferred to the reward-modulated learning pass.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PlasticityConfig {
    pub rule: HebbianRule,
    /// Learning rate, applied once per weight update and clamped to [0.0, 1.0].
    pub learning_rate: f32,
    /// Symmetric weight clamp magnitude, clamped to [0.01, 10.0] at runtime.
    pub weight_clamp: f32,
    /// If true, offspring inherit learned weights (Lamarckian); otherwise
    /// offspring start from genome birth weights (Darwinian).
    pub lamarckian: bool,
    /// Reward modulation config. `None` = pure Hebbian (default).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modulation: Option<RewardModulationConfig>,
}

// ── Top-level genome types ────────────────────────────────────────────────────

/// Backend definition: either VM or Graph (CGP three-layer model).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum BackendDef {
    Vm(VmBackendDef),
    Graph(cgp::CgpGraphBackendDef),
}

impl BackendDef {
    /// After an input_ref is removed at `removed_ref_idx`, update all internal
    /// references. Graph: walks all edge containers (InputLeaf ref_idx).
    /// VM: walks ReadInput instructions.
    /// Matching ref_idx edges removed (Graph) or converted to Noop (VM).
    /// Above → decremented.
    pub fn reindex_input_refs_after_removal(&mut self, removed_ref_idx: u16) {
        match self {
            BackendDef::Graph(gd) => {
                gd.reindex_input_refs_after_removal(removed_ref_idx);
            }
            BackendDef::Vm(vm) => {
                for instr in &mut vm.program {
                    if let VmInstruction::ReadInput { ref_idx, .. } = instr {
                        if *ref_idx == removed_ref_idx {
                            // Convert orphaned read to Noop instead of leaving a
                            // permanent u16::MAX zombie that silently reads 0.0.
                            // This parallels the Graph backend which removes the edge.
                            *instr = VmInstruction::Noop;
                        } else if *ref_idx > removed_ref_idx {
                            *ref_idx -= 1;
                        }
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
    pub targets: Vec<RouteTarget>,
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

    /// Total structural size: sum of all discrete genome components.
    ///
    /// Counts every node, input ref, target, instruction, constant, and
    /// graph internal node (plus its inputs). Equal weights, includes junk DNA,
    /// no reachability analysis. Used for the mutation pressure gate
    /// (`genome_size_cap`).
    pub fn genome_size(&self) -> u32 {
        let mut score: u32 = 0;
        for node in &self.nodes {
            score += 1;
            score += node.input_refs.len() as u32;
            score += node.targets.len() as u32;
            match &node.backend_def {
                BackendDef::Vm(vm) => {
                    score += vm.program.len() as u32;
                    score += vm.constants.len() as u32;
                }
                BackendDef::Graph(graph) => {
                    // Count compute nodes + their edges
                    score += graph.compute_nodes.len() as u32;
                    for cn in &graph.compute_nodes {
                        score += cn.inputs.len() as u32;
                    }
                    // Count wired output sinks (unwired are structural scaffolding)
                    for sink in &graph.output_sinks {
                        if !sink.inputs.is_empty() {
                            score += 1 + sink.inputs.len() as u32;
                        }
                    }
                }
            }
        }
        score
    }

    /// Functional complexity: reachability-aware score counting only
    /// live components (reachable mesh nodes, output-connected backend
    /// components, consumed input refs).
    ///
    /// Used for action energy costs. Creatures are not penalized for
    /// junk DNA that has no functional impact.
    pub fn complexity(&self) -> u32 {
        analysis::functional_complexity(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::{DynamicIntrospectionKey, StaticIntrospectionKey, WorldInputKey};
    use proptest::prelude::*;
    use proptest::strategy::Union;

    #[test]
    fn vm_instruction_all_42_variants_constructible() {
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
            VmInstruction::WriteActionParam {
                slot_idx: 0,
                src: 1,
            },
            VmInstruction::WriteRouteGate { slot: 0, src: 0 },
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
            VmInstruction::Halt,
            VmInstruction::LoadSlot {
                dst: 0,
                slot_reg: 1,
            },
            VmInstruction::StoreSlot {
                slot_reg: 0,
                src: 1,
            },
            VmInstruction::LoadSlotImm {
                dst: 0,
                slot_idx: 3,
            },
            VmInstruction::StoreSlotImm {
                slot_idx: 5,
                src: 1,
            },
            VmInstruction::LoadSlotPrev {
                dst: 0,
                slot_idx: 7,
            },
            VmInstruction::ClearSlot { slot_idx: 2 },
            VmInstruction::AddVote { sink: 0, src: 1 },
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

    fn nullary(instr: VmInstruction) -> BoxedStrategy<VmInstruction> {
        Just(instr).boxed()
    }

    fn unary(make: fn(u8) -> VmInstruction) -> BoxedStrategy<VmInstruction> {
        any::<u8>().prop_map(make).boxed()
    }

    fn binary(make: fn(u8, u8) -> VmInstruction) -> BoxedStrategy<VmInstruction> {
        (any::<u8>(), any::<u8>())
            .prop_map(move |(x, y)| make(x, y))
            .boxed()
    }

    fn ternary(make: fn(u8, u8, u8) -> VmInstruction) -> BoxedStrategy<VmInstruction> {
        (any::<u8>(), any::<u8>(), any::<u8>())
            .prop_map(move |(x, y, z)| make(x, y, z))
            .boxed()
    }

    /// Every one of the 39 opcodes with arbitrary operands.
    fn any_vm_instruction() -> impl Strategy<Value = VmInstruction> {
        use VmInstruction::*;
        Union::new(vec![
            nullary(Noop),
            binary(|dst, const_idx| LoadConst { dst, const_idx }),
            binary(|dst, src| Move { dst, src }),
            ternary(|dst, a, b| Add { dst, a, b }),
            ternary(|dst, a, b| Sub { dst, a, b }),
            ternary(|dst, a, b| Mul { dst, a, b }),
            ternary(|dst, a, b| Div { dst, a, b }),
            ternary(|dst, a, b| Min { dst, a, b }),
            ternary(|dst, a, b| Max { dst, a, b }),
            binary(|dst, src| Abs { dst, src }),
            binary(|dst, src| Neg { dst, src }),
            binary(|dst, src| Clamp01 { dst, src }),
            ternary(|dst, a, b| CmpGt { dst, a, b }),
            ternary(|dst, a, b| CmpLt { dst, a, b }),
            (any::<u8>(), any::<u8>(), any::<u8>(), any::<u8>())
                .prop_map(|(dst, a, b, eps)| CmpEq { dst, a, b, eps })
                .boxed(),
            ternary(|dst, a, b| And { dst, a, b }),
            ternary(|dst, a, b| Or { dst, a, b }),
            binary(|dst, src| Not { dst, src }),
            binary(|dst, src| ToI32 { dst, src }),
            binary(|dst, src| ToU8 { dst, src }),
            binary(|dst, src| ToBool { dst, src }),
            (any::<u8>(), any::<i32>())
                .prop_map(|(cond, offset)| JumpIfZero { cond, offset })
                .boxed(),
            any::<i32>().prop_map(|offset| Jump { offset }).boxed(),
            (any::<u8>(), any::<u16>(), any::<u16>())
                .prop_map(|(dst, ref_idx, sub_idx)| ReadInput {
                    dst,
                    ref_idx,
                    sub_idx,
                })
                .boxed(),
            binary(|slot_idx, src| WriteInternalPayload { slot_idx, src }),
            binary(|slot_idx, src| WriteActionParam { slot_idx, src }),
            binary(|sink, src| AddVote { sink, src }),
            binary(|slot, src| WriteRouteGate { slot, src }),
            unary(|dst| ReadActionQueueLength { dst }),
            binary(|index_src, dst| ReadActionQueueType { index_src, dst }),
            ternary(|index_src, param_slot, dst| ReadActionQueueParam {
                index_src,
                param_slot,
                dst,
            }),
            unary(|src| SetPriorityBid { src }),
            nullary(Halt),
            binary(|dst, slot_reg| LoadSlot { dst, slot_reg }),
            binary(|slot_reg, src| StoreSlot { slot_reg, src }),
            binary(|dst, slot_idx| LoadSlotImm { dst, slot_idx }),
            binary(|slot_idx, src| StoreSlotImm { slot_idx, src }),
            binary(|dst, slot_idx| LoadSlotPrev { dst, slot_idx }),
            unary(|slot_idx| ClearSlot { slot_idx }),
        ])
    }

    proptest! {
        /// Serde round-trip is the identity for every opcode and operand value.
        #[test]
        fn vm_instruction_serde_roundtrip_holds_for_any_opcode(instr in any_vm_instruction()) {
            let json = serde_json::to_string(&instr).unwrap();
            let decoded: VmInstruction = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(decoded, instr);
        }
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
                            VmInstruction::AddVote { sink: 0, src: 0 },
                            VmInstruction::Halt,
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
    fn genome_size_empty_genome() {
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![],
        };
        assert_eq!(genome.genome_size(), 0);
    }

    #[test]
    fn genome_size_single_vm_node() {
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
                targets: vec![RouteTarget {
                    target_id: NodeId::new(1),
                    slot: 0,
                    gate_bias: 0.0,
                }],
            }],
        };
        assert_eq!(genome.genome_size(), 9);
    }

    #[test]
    fn genome_size_single_graph_node() {
        use crate::creature::genome::cgp::{
            CgpGraphBackendDef, ComputeNode, ComputeNodeKind, GraphEdge, GraphSource,
        };
        // 1 node + 1 input_ref + 0 targets + 2 compute_nodes + (1 + 2) compute edges = 7
        // (unwired sinks add 0)
        let mut cgp = CgpGraphBackendDef::new_with_fixed_outputs();
        cgp.compute_nodes = vec![
            ComputeNode {
                kind: ComputeNodeKind::Add,
                inputs: vec![GraphEdge {
                    source: GraphSource::InputLeaf {
                        ref_idx: 0,
                        sub_idx: 0,
                    },
                    weight: 1.0,
                }],
                plasticity: None,
            },
            ComputeNode {
                kind: ComputeNodeKind::Sigmoid,
                inputs: vec![
                    GraphEdge {
                        source: GraphSource::ComputeNode(0),
                        weight: 0.5,
                    },
                    GraphEdge {
                        source: GraphSource::InputLeaf {
                            ref_idx: 0,
                            sub_idx: 0,
                        },
                        weight: -0.3,
                    },
                ],
                plasticity: None,
            },
        ];
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![InputReference::World(WorldInputKey::FoodHere {
                    type_idx: crate::config::OrdinaryFoodTypeId::default(),
                })],
                backend_def: BackendDef::Graph(cgp),
                targets: vec![],
            }],
        };
        assert_eq!(genome.genome_size(), 7);
    }

    #[test]
    fn genome_size_multi_node_mixed() {
        use crate::creature::genome::cgp::{CgpGraphBackendDef, ComputeNode, ComputeNodeKind};
        // Node 0 (VM): 1 + 0 inputs + 1 target + 1 program + 0 constants = 3
        // Node 1 (Graph): 1 + 1 input + 0 targets + 1 compute_node + 0 edges = 3
        // Total = 6
        let mut cgp = CgpGraphBackendDef::new_with_fixed_outputs();
        cgp.compute_nodes = vec![ComputeNode {
            kind: ComputeNodeKind::Constant(1.0),
            inputs: vec![],
            plasticity: None,
        }];
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
                    targets: vec![RouteTarget {
                        target_id: NodeId::new(1),
                        slot: 0,
                        gate_bias: 0.0,
                    }],
                },
                NodeGenome {
                    node_id: NodeId::new(1),
                    input_refs: vec![InputReference::World(WorldInputKey::FoodHere {
                        type_idx: crate::config::OrdinaryFoodTypeId::default(),
                    })],
                    backend_def: BackendDef::Graph(cgp),
                    targets: vec![],
                },
            ],
        };
        assert_eq!(genome.genome_size(), 6);
    }

    #[test]
    fn complexity_excludes_unreachable_mesh_nodes() {
        // Node 0 (entry) → Node 1. Node 2 is unreachable.
        // Node 0: VM with output instructions (functional)
        // Node 2: also has output instructions but is unreachable
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![
                NodeGenome {
                    node_id: NodeId::new(0),
                    input_refs: vec![InputReference::World(WorldInputKey::FoodHere {
                        type_idx: crate::config::OrdinaryFoodTypeId::default(),
                    })],
                    backend_def: BackendDef::Vm(VmBackendDef {
                        register_count: 4,
                        constants: vec![],
                        program: vec![
                            VmInstruction::ReadInput {
                                dst: 0,
                                ref_idx: 0,
                                sub_idx: 0,
                            },
                            VmInstruction::WriteRouteGate { slot: 0, src: 0 },
                        ],
                    }),
                    targets: vec![RouteTarget {
                        target_id: NodeId::new(1),
                        slot: 0,
                        gate_bias: 0.0,
                    }],
                },
                NodeGenome {
                    node_id: NodeId::new(1),
                    input_refs: vec![],
                    backend_def: BackendDef::Vm(VmBackendDef {
                        register_count: 4,
                        constants: vec![],
                        program: vec![
                            VmInstruction::AddVote { sink: 25, src: 0 },
                            VmInstruction::WriteActionParam {
                                slot_idx: 0,
                                src: 0,
                            },
                        ],
                    }),
                    targets: vec![],
                },
                NodeGenome {
                    node_id: NodeId::new(2),
                    input_refs: vec![InputReference::DynamicIntrospection(
                        DynamicIntrospectionKey::EnergyCurrent,
                    )],
                    backend_def: BackendDef::Vm(VmBackendDef {
                        register_count: 4,
                        constants: vec![],
                        program: vec![
                            VmInstruction::ReadInput {
                                dst: 0,
                                ref_idx: 0,
                                sub_idx: 0,
                            },
                            VmInstruction::AddVote { sink: 25, src: 0 },
                            VmInstruction::WriteActionParam {
                                slot_idx: 0,
                                src: 0,
                            },
                        ],
                    }),
                    targets: vec![],
                },
            ],
        };
        // Node 0: 1 + 1 target + 2 live instrs + 0 consts + 1 consumed ref = 5
        // Node 1: 1 + 0 targets + 2 live instrs + 0 consts + 0 refs = 3
        // Node 2: excluded (unreachable)
        assert_eq!(genome.complexity(), 8);
        // genome_size counts everything including Node 2
        assert_eq!(genome.genome_size(), 13);
        assert!(genome.complexity() < genome.genome_size());
    }

    #[test]
    fn complexity_equals_genome_size_when_fully_connected() {
        // All nodes reachable, all instructions live, all input_refs consumed
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![InputReference::World(WorldInputKey::FoodHere {
                    type_idx: crate::config::OrdinaryFoodTypeId::default(),
                })],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 4,
                    constants: vec![],
                    program: vec![
                        VmInstruction::ReadInput {
                            dst: 0,
                            ref_idx: 0,
                            sub_idx: 0,
                        },
                        VmInstruction::WriteRouteGate { slot: 0, src: 0 },
                    ],
                }),
                targets: vec![],
            }],
        };
        // Node: 1 + 0 targets + 2 live instrs + 0 consts + 1 consumed ref = 4
        // genome_size: 1 + 1 input_ref + 0 targets + 2 instrs + 0 consts = 4
        assert_eq!(genome.complexity(), genome.genome_size());
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

    #[test]
    fn plasticity_config_serde_roundtrip_with_modulation() {
        let cfg = PlasticityConfig {
            rule: HebbianRule::Oja,
            learning_rate: 0.05,
            weight_clamp: 3.0,
            lamarckian: true,
            modulation: Some(RewardModulationConfig {
                reward_source: OutcomeChannel::EnergyDelta,
                trace_decay: 0.9,
            }),
        };
        let json = serde_json::to_string(&cfg).unwrap();
        let cfg2: PlasticityConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(cfg, cfg2);
    }

    #[test]
    fn plasticity_config_serde_roundtrip_without_modulation() {
        let cfg = PlasticityConfig {
            rule: HebbianRule::Classic,
            learning_rate: 0.1,
            weight_clamp: 5.0,
            lamarckian: false,
            modulation: None,
        };
        let json = serde_json::to_string(&cfg).unwrap();
        // modulation: None should be skipped in serialization
        assert!(!json.contains("modulation"));
        let cfg2: PlasticityConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(cfg, cfg2);
    }

    #[test]
    fn outcome_channel_serde_roundtrip() {
        let channels = [
            OutcomeChannel::EnergyDelta,
            OutcomeChannel::ActionSuccess,
            OutcomeChannel::DamageDelta,
            OutcomeChannel::OffspringSuccess,
        ];
        for ch in channels {
            let json = serde_json::to_string(&ch).unwrap();
            let ch2: OutcomeChannel = serde_json::from_str(&json).unwrap();
            assert_eq!(ch, ch2);
        }
    }

    #[test]
    fn reindex_input_refs_after_removal_graph() {
        // CGP Graph backend with InputLeaf edges at ref_idx 0, 1, 2.
        // Remove input_ref at index 1:
        // - ref_idx 0 → unchanged
        // - ref_idx 1 → removed (edge dropped)
        // - ref_idx 2 → 1 (decremented)

        use crate::creature::genome::cgp::{
            CgpGraphBackendDef, ComputeNode, ComputeNodeKind, GraphEdge, GraphSource,
        };
        let mut cgp = CgpGraphBackendDef::new_with_fixed_outputs();
        cgp.compute_nodes = vec![ComputeNode {
            kind: ComputeNodeKind::Add,
            inputs: vec![
                GraphEdge {
                    source: GraphSource::InputLeaf {
                        ref_idx: 0,
                        sub_idx: 0,
                    },
                    weight: 1.0,
                },
                GraphEdge {
                    source: GraphSource::InputLeaf {
                        ref_idx: 1,
                        sub_idx: 3,
                    },
                    weight: 1.0,
                },
                GraphEdge {
                    source: GraphSource::InputLeaf {
                        ref_idx: 2,
                        sub_idx: 0,
                    },
                    weight: 1.0,
                },
                GraphEdge {
                    source: GraphSource::ComputeNode(0),
                    weight: 0.5,
                },
            ],
            plasticity: None,
        }];
        let mut backend = BackendDef::Graph(cgp);
        backend.reindex_input_refs_after_removal(1);
        if let BackendDef::Graph(ref g) = backend {
            let edges = &g.compute_nodes[0].inputs;
            // ref_idx 1 edge removed, 3 edges remain (0, 2→1, ComputeNode)
            assert_eq!(edges.len(), 3, "edge at ref_idx 1 must be removed");
            // ref_idx 0 → unchanged
            assert_eq!(
                edges[0].source,
                GraphSource::InputLeaf {
                    ref_idx: 0,
                    sub_idx: 0,
                }
            );
            // ref_idx 2 → decremented to 1
            assert_eq!(
                edges[1].source,
                GraphSource::InputLeaf {
                    ref_idx: 1,
                    sub_idx: 0,
                }
            );
            // Non-InputLeaf edge unchanged
            assert_eq!(edges[2].source, GraphSource::ComputeNode(0));
        } else {
            panic!("expected Graph");
        }
    }

    #[test]
    fn reindex_input_refs_after_removal_vm() {
        // VM backend with ReadInput instructions at ref_idx 0, 1, 2.
        // Remove input_ref at index 1.
        let mut backend = BackendDef::Vm(VmBackendDef {
            register_count: 4,
            constants: vec![],
            program: vec![
                VmInstruction::ReadInput {
                    dst: 0,
                    ref_idx: 0,
                    sub_idx: 0,
                },
                VmInstruction::ReadInput {
                    dst: 1,
                    ref_idx: 1,
                    sub_idx: 0,
                },
                VmInstruction::ReadInput {
                    dst: 2,
                    ref_idx: 2,
                    sub_idx: 5,
                },
                VmInstruction::Halt,
            ],
        });
        backend.reindex_input_refs_after_removal(1);
        if let BackendDef::Vm(ref vm) = backend {
            assert!(matches!(
                vm.program[0],
                VmInstruction::ReadInput {
                    ref_idx: 0,
                    sub_idx: 0,
                    ..
                }
            ));
            assert!(
                matches!(vm.program[1], VmInstruction::Noop),
                "orphaned ReadInput must become Noop, got {:?}",
                vm.program[1]
            );
            assert!(matches!(
                vm.program[2],
                VmInstruction::ReadInput {
                    ref_idx: 1,
                    sub_idx: 5,
                    ..
                }
            ));
            assert!(matches!(vm.program[3], VmInstruction::Halt));
        } else {
            panic!("expected VM");
        }
    }
}
