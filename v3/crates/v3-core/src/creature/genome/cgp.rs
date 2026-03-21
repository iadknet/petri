//! CGP-style graph backend types.
//!
//! Three-layer architecture: implicit inputs (GraphSource), mutable compute
//! nodes (ComputeNodeKind), and fixed structural outputs (OutputSink,
//! ActionSlot, ExecuteGate).

use crate::config::MutationConfig;
use crate::contracts::MAX_GATE_SLOTS;

// ── Edge addressing ─────────────────────────────────────────────────────────

/// Tagged source for edge addressing. Replaces bare `source_idx: u16`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum GraphSource {
    /// Read from `input_refs[ref_idx]` with sub-value index `sub_idx`.
    InputLeaf { ref_idx: u16, sub_idx: u16 },
    /// Read shared memory slot (current or previous tick).
    SharedMemory { slot: u8, previous: bool },
    /// Read output of `compute_nodes[idx]`.
    ComputeNode(u16),
}

/// Weighted edge in the graph.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GraphEdge {
    pub source: GraphSource,
    pub weight: f32,
}

// ── Compute layer ───────────────────────────────────────────────────────────

/// Node class for frontend visualization and categorization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeClass {
    Arithmetic,
    Activation,
    Logic,
    Stateful,
    Constant,
}

/// Computation-only node kinds (17 variants).
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ComputeNodeKind {
    // Arithmetic
    Add,
    Multiply,
    Negate,
    Abs,
    Min,
    Max,
    WeightedSum,
    // Activation
    Sigmoid,
    Tanh,
    Relu,
    Clamp01,
    Threshold(f32),
    // Logic
    GreaterThan,
    Select,
    // Stateful
    DecayIntegrator(f32),
    Momentum(f32),
    Oscillator(f32),
    AdaptiveGain,
    // Constant
    Constant(f32),
}

impl ComputeNodeKind {
    /// Returns the high-level class for this compute node kind.
    pub fn class(&self) -> NodeClass {
        match self {
            Self::Add
            | Self::Multiply
            | Self::Negate
            | Self::Abs
            | Self::Min
            | Self::Max
            | Self::WeightedSum => NodeClass::Arithmetic,

            Self::Sigmoid | Self::Tanh | Self::Relu | Self::Clamp01 | Self::Threshold(_) => {
                NodeClass::Activation
            }

            Self::GreaterThan | Self::Select => NodeClass::Logic,

            Self::DecayIntegrator(_)
            | Self::Momentum(_)
            | Self::Oscillator(_)
            | Self::AdaptiveGain => NodeClass::Stateful,

            Self::Constant(_) => NodeClass::Constant,
        }
    }
}

/// A single compute node in the graph backend.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ComputeNode {
    pub kind: ComputeNodeKind,
    pub inputs: Vec<GraphEdge>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plasticity: Option<super::PlasticityConfig>,
}

// ── Fixed structural outputs ────────────────────────────────────────────────

/// Output sink kinds — fixed catalog, not evolvable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum OutputSinkKind {
    /// Write to `output_slots[slot]`. 12 slots, indices 0-11.
    CustomOutput(u8),
    /// Write to `route_gates.scores[slot]`. 8 slots, indices 0-7.
    RouterGate(u8),
    /// Write to `shared_memory[slot]`. 16 slots, indices 0-15.
    WriteSlot(u8),
    /// Clear `shared_memory[slot]` to 0.0. 16 slots, indices 0-15.
    ClearSlot(u8),
}

/// Fixed structural output — one per target slot.
/// Kind is NOT mutated; only inputs (edges) change via mutation.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OutputSink {
    pub kind: OutputSinkKind,
    pub inputs: Vec<GraphEdge>,
}

// ── Action bank ─────────────────────────────────────────────────────────────

/// What a slot does when it fires.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ActionSlotBehavior {
    /// Queue-program-control: remove last queued action.
    Pop,
    /// Emit a world action into the queue.
    Emit(WorldActionKind),
}

/// World action kinds — maps 1:1 to WorldAction variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum WorldActionKind {
    Eat,
    Move,
    Reproduce,
    StealEnergy,
    NoOp,
}

/// Action slot in the fixed action bank.
/// Each slot is fully self-contained: own gate, own params, fixed behavior.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ActionSlot {
    /// Evolvable via raw field mutation.
    pub behavior: ActionSlotBehavior,
    /// Weighted sum > 0.0 means fire.
    pub gate_inputs: Vec<GraphEdge>,
    /// Decoded per WorldActionKind (ignored for Pop).
    pub param_inputs: Vec<GraphEdge>,
}

// ── Execute gate ────────────────────────────────────────────────────────────

/// Separate gated output controlling mesh termination.
/// When gate fires AND queue non-empty, mesh hop terminates.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ExecuteGate {
    pub inputs: Vec<GraphEdge>,
}

// ── Top-level graph backend ─────────────────────────────────────────────────

/// CGP-style graph backend definition.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CgpGraphBackendDef {
    pub compute_nodes: Vec<ComputeNode>,
    /// Fixed set — structurally immutable. Only edges are evolvable.
    pub output_sinks: Vec<OutputSink>,
    /// Fixed-length bank (size == config.action_queue_cap at creation time).
    pub action_bank: Vec<ActionSlot>,
    /// Separate terminal gate.
    pub execute_gate: ExecuteGate,
}

/// Number of CustomOutput sinks in the fixed catalog.
pub const CUSTOM_OUTPUT_COUNT: u8 = 24;
/// Number of shared memory slots (WriteSlot + ClearSlot each).
pub const SHARED_MEMORY_SLOTS: u8 = 16;
/// Total fixed sink count: N CustomOutput + 8 RouterGate + 16 WriteSlot + 16 ClearSlot.
pub const FIXED_SINK_COUNT: usize = CUSTOM_OUTPUT_COUNT as usize     // 24 CustomOutput slots
    + MAX_GATE_SLOTS                 // 8 RouterGate sinks
    + SHARED_MEMORY_SLOTS as usize   // 16 WriteSlot sinks
    + SHARED_MEMORY_SLOTS as usize; // 16 ClearSlot sinks
const _: () = assert!(FIXED_SINK_COUNT == 64);

impl CgpGraphBackendDef {
    /// Construct a new graph backend with the full fixed output catalog.
    /// All sinks and action slots start with empty edges (inert).
    pub fn new_with_fixed_outputs(config: &MutationConfig) -> Self {
        let mut output_sinks = Vec::with_capacity(FIXED_SINK_COUNT);

        // CustomOutput sinks
        for slot in 0..CUSTOM_OUTPUT_COUNT {
            output_sinks.push(OutputSink {
                kind: OutputSinkKind::CustomOutput(slot),
                inputs: Vec::new(),
            });
        }

        // 8 RouterGate sinks
        for slot in 0..MAX_GATE_SLOTS as u8 {
            output_sinks.push(OutputSink {
                kind: OutputSinkKind::RouterGate(slot),
                inputs: Vec::new(),
            });
        }

        // 16 WriteSlot sinks
        for slot in 0..SHARED_MEMORY_SLOTS {
            output_sinks.push(OutputSink {
                kind: OutputSinkKind::WriteSlot(slot),
                inputs: Vec::new(),
            });
        }

        // 16 ClearSlot sinks
        for slot in 0..SHARED_MEMORY_SLOTS {
            output_sinks.push(OutputSink {
                kind: OutputSinkKind::ClearSlot(slot),
                inputs: Vec::new(),
            });
        }

        // Action bank sized to config
        let bank_size = config.action_queue_cap;
        let mut action_bank = Vec::with_capacity(bank_size);
        for _ in 0..bank_size {
            action_bank.push(ActionSlot {
                behavior: ActionSlotBehavior::Emit(WorldActionKind::NoOp),
                gate_inputs: Vec::new(),
                param_inputs: Vec::new(),
            });
        }

        Self {
            compute_nodes: Vec::new(),
            output_sinks,
            action_bank,
            execute_gate: ExecuteGate { inputs: Vec::new() },
        }
    }

    /// Remove a compute node at `idx`. Remaps `GraphSource::ComputeNode`
    /// indices across ALL edge containers: compute inputs, sink inputs,
    /// action gate/param inputs, and execute gate inputs.
    ///
    /// Edges pointing to the removed node get `ComputeNode(u16::MAX)`.
    /// Edges pointing above the removed index are decremented.
    pub fn remove_compute_node_at(&mut self, idx: usize) {
        debug_assert!(
            idx < self.compute_nodes.len(),
            "remove_compute_node_at: idx {} out of bounds (len {})",
            idx,
            self.compute_nodes.len()
        );
        self.compute_nodes.remove(idx);
        let removed = idx as u16;
        self.remap_compute_sources(|src_idx| {
            if src_idx == removed {
                u16::MAX
            } else if src_idx > removed {
                src_idx - 1
            } else {
                src_idx
            }
        });
    }

    /// After an input_ref is removed at `removed_ref_idx`, update all
    /// `GraphSource::InputLeaf { ref_idx }` across all edge containers.
    /// Matching ref_idx edges are removed. Higher ref_idx values are decremented.
    pub fn reindex_input_refs_after_removal(&mut self, removed_ref_idx: u16) {
        self.for_each_edge_vec_mut(|edges| {
            edges.retain_mut(|edge| {
                if let GraphSource::InputLeaf { ref_idx, .. } = &mut edge.source {
                    if *ref_idx == removed_ref_idx {
                        return false; // remove edge
                    }
                    if *ref_idx > removed_ref_idx {
                        *ref_idx -= 1;
                    }
                }
                true
            });
        });
    }

    /// After an input_ref is swapped, clamp sub_idx values that exceed the
    /// new width for the given ref_idx. Edges with out-of-range sub_idx
    /// are removed.
    pub fn clamp_sub_idx_after_swap(&mut self, ref_idx: u16, new_width: u16) {
        if new_width == 0 {
            // Remove all edges pointing to this ref_idx
            self.for_each_edge_vec_mut(|edges| {
                edges.retain(|edge| {
                    !matches!(edge.source, GraphSource::InputLeaf { ref_idx: r, .. } if r == ref_idx)
                });
            });
            return;
        }
        self.for_each_edge_vec_mut(|edges| {
            edges.retain_mut(|edge| {
                if let GraphSource::InputLeaf {
                    ref_idx: r,
                    sub_idx,
                } = &mut edge.source
                {
                    if *r == ref_idx && *sub_idx >= new_width {
                        return false; // remove out-of-range edge
                    }
                }
                true
            });
        });
    }

    // ── Internal helpers ────────────────────────────────────────────────────

    /// Apply a remapping function to all `GraphSource::ComputeNode(idx)` edges
    /// across every edge container.
    fn remap_compute_sources(&mut self, remap: impl Fn(u16) -> u16) {
        self.for_each_edge_mut(|edge| {
            if let GraphSource::ComputeNode(idx) = &mut edge.source {
                *idx = remap(*idx);
            }
        });
    }

    /// Apply a closure to every `GraphEdge` across all containers.
    fn for_each_edge_mut(&mut self, mut f: impl FnMut(&mut GraphEdge)) {
        for node in &mut self.compute_nodes {
            for edge in &mut node.inputs {
                f(edge);
            }
        }
        for sink in &mut self.output_sinks {
            for edge in &mut sink.inputs {
                f(edge);
            }
        }
        for slot in &mut self.action_bank {
            for edge in &mut slot.gate_inputs {
                f(edge);
            }
            for edge in &mut slot.param_inputs {
                f(edge);
            }
        }
        for edge in &mut self.execute_gate.inputs {
            f(edge);
        }
    }

    /// Apply a closure to every `Vec<GraphEdge>` across all containers.
    /// Used for retain-style operations that need mutable access to the Vec.
    fn for_each_edge_vec_mut(&mut self, mut f: impl FnMut(&mut Vec<GraphEdge>)) {
        for node in &mut self.compute_nodes {
            f(&mut node.inputs);
        }
        for sink in &mut self.output_sinks {
            f(&mut sink.inputs);
        }
        for slot in &mut self.action_bank {
            f(&mut slot.gate_inputs);
            f(&mut slot.param_inputs);
        }
        f(&mut self.execute_gate.inputs);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Construction tests ──────────────────────────────────────────────────

    #[test]
    fn new_with_fixed_outputs_creates_correct_catalog() {
        let config = MutationConfig::default(); // action_queue_cap = 4
        let def = CgpGraphBackendDef::new_with_fixed_outputs(&config);

        assert_eq!(def.output_sinks.len(), FIXED_SINK_COUNT);
        assert_eq!(def.action_bank.len(), 4);
        assert!(def.compute_nodes.is_empty());
        assert!(def.execute_gate.inputs.is_empty());

        // Verify sink ordering
        for i in 0..CUSTOM_OUTPUT_COUNT {
            assert_eq!(
                def.output_sinks[i as usize].kind,
                OutputSinkKind::CustomOutput(i)
            );
        }
        for i in 0..MAX_GATE_SLOTS {
            assert_eq!(
                def.output_sinks[CUSTOM_OUTPUT_COUNT as usize + i].kind,
                OutputSinkKind::RouterGate(i as u8)
            );
        }
        for i in 0..SHARED_MEMORY_SLOTS {
            assert_eq!(
                def.output_sinks[CUSTOM_OUTPUT_COUNT as usize + MAX_GATE_SLOTS + i as usize].kind,
                OutputSinkKind::WriteSlot(i)
            );
        }
        for i in 0..SHARED_MEMORY_SLOTS {
            assert_eq!(
                def.output_sinks[CUSTOM_OUTPUT_COUNT as usize
                    + MAX_GATE_SLOTS
                    + SHARED_MEMORY_SLOTS as usize
                    + i as usize]
                    .kind,
                OutputSinkKind::ClearSlot(i)
            );
        }

        // All sinks start with empty edges
        for sink in &def.output_sinks {
            assert!(sink.inputs.is_empty());
        }

        // All action slots start with empty edges
        for slot in &def.action_bank {
            assert!(slot.gate_inputs.is_empty());
            assert!(slot.param_inputs.is_empty());
            assert_eq!(
                slot.behavior,
                ActionSlotBehavior::Emit(WorldActionKind::NoOp)
            );
        }
    }

    #[test]
    fn new_with_custom_queue_cap() {
        let config = MutationConfig {
            action_queue_cap: 8,
            ..MutationConfig::default()
        };
        let def = CgpGraphBackendDef::new_with_fixed_outputs(&config);
        assert_eq!(def.action_bank.len(), 8);
    }

    // ── Serde roundtrip tests ───────────────────────────────────────────────

    #[test]
    fn graph_source_serde_roundtrip() {
        let sources = vec![
            GraphSource::InputLeaf {
                ref_idx: 3,
                sub_idx: 7,
            },
            GraphSource::SharedMemory {
                slot: 5,
                previous: true,
            },
            GraphSource::ComputeNode(42),
        ];
        let json = serde_json::to_string(&sources).unwrap();
        let decoded: Vec<GraphSource> = serde_json::from_str(&json).unwrap();
        assert_eq!(sources, decoded);
    }

    #[test]
    fn compute_node_kind_serde_roundtrip() {
        let kinds = vec![
            ComputeNodeKind::Add,
            ComputeNodeKind::Threshold(0.5),
            ComputeNodeKind::DecayIntegrator(0.9),
            ComputeNodeKind::Constant(3.25),
        ];
        let json = serde_json::to_string(&kinds).unwrap();
        let decoded: Vec<ComputeNodeKind> = serde_json::from_str(&json).unwrap();
        assert_eq!(kinds, decoded);
    }

    #[test]
    fn action_slot_behavior_serde_roundtrip() {
        let behaviors = vec![
            ActionSlotBehavior::Pop,
            ActionSlotBehavior::Emit(WorldActionKind::Eat),
            ActionSlotBehavior::Emit(WorldActionKind::Move),
            ActionSlotBehavior::Emit(WorldActionKind::Reproduce),
            ActionSlotBehavior::Emit(WorldActionKind::StealEnergy),
            ActionSlotBehavior::Emit(WorldActionKind::NoOp),
        ];
        let json = serde_json::to_string(&behaviors).unwrap();
        let decoded: Vec<ActionSlotBehavior> = serde_json::from_str(&json).unwrap();
        assert_eq!(behaviors, decoded);
    }

    #[test]
    fn full_backend_def_serde_roundtrip() {
        let config = MutationConfig::default();
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs(&config);

        // Add a compute node with edges
        def.compute_nodes.push(ComputeNode {
            kind: ComputeNodeKind::Threshold(24.0),
            inputs: vec![GraphEdge {
                source: GraphSource::InputLeaf {
                    ref_idx: 1,
                    sub_idx: 0,
                },
                weight: 1.0,
            }],
            plasticity: None,
        });

        // Wire an output sink
        def.output_sinks[0].inputs.push(GraphEdge {
            source: GraphSource::ComputeNode(0),
            weight: 1.0,
        });

        let json = serde_json::to_string(&def).unwrap();
        let decoded: CgpGraphBackendDef = serde_json::from_str(&json).unwrap();
        assert_eq!(def, decoded);
    }

    // ── ComputeNodeKind::class() tests ──────────────────────────────────────

    #[test]
    fn compute_node_class_categorization() {
        assert_eq!(ComputeNodeKind::Add.class(), NodeClass::Arithmetic);
        assert_eq!(ComputeNodeKind::WeightedSum.class(), NodeClass::Arithmetic);
        assert_eq!(ComputeNodeKind::Sigmoid.class(), NodeClass::Activation);
        assert_eq!(
            ComputeNodeKind::Threshold(0.5).class(),
            NodeClass::Activation
        );
        assert_eq!(ComputeNodeKind::GreaterThan.class(), NodeClass::Logic);
        assert_eq!(ComputeNodeKind::Select.class(), NodeClass::Logic);
        assert_eq!(
            ComputeNodeKind::DecayIntegrator(0.9).class(),
            NodeClass::Stateful
        );
        assert_eq!(ComputeNodeKind::AdaptiveGain.class(), NodeClass::Stateful);
        assert_eq!(ComputeNodeKind::Constant(1.0).class(), NodeClass::Constant);
    }

    // ── Structural removal tests ────────────────────────────────────────────

    #[test]
    fn remove_compute_node_remaps_edges_across_all_containers() {
        let config = MutationConfig::default();
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs(&config);

        // Add 3 compute nodes
        for kind in [
            ComputeNodeKind::Add,
            ComputeNodeKind::Sigmoid,
            ComputeNodeKind::Relu,
        ] {
            def.compute_nodes.push(ComputeNode {
                kind,
                inputs: Vec::new(),
                plasticity: None,
            });
        }

        // Compute node 2 (Relu) has edge to node 1 (Sigmoid)
        def.compute_nodes[2].inputs.push(GraphEdge {
            source: GraphSource::ComputeNode(1),
            weight: 1.0,
        });

        // Output sink has edge to node 2 (Relu)
        def.output_sinks[0].inputs.push(GraphEdge {
            source: GraphSource::ComputeNode(2),
            weight: 0.5,
        });

        // Action slot gate has edge to node 0 (Add)
        def.action_bank[0].gate_inputs.push(GraphEdge {
            source: GraphSource::ComputeNode(0),
            weight: 1.0,
        });

        // Action slot param has edge to node 1 (Sigmoid)
        def.action_bank[0].param_inputs.push(GraphEdge {
            source: GraphSource::ComputeNode(1),
            weight: 1.0,
        });

        // Execute gate has edge to node 2 (Relu)
        def.execute_gate.inputs.push(GraphEdge {
            source: GraphSource::ComputeNode(2),
            weight: 1.0,
        });

        // Remove node 0 (Add). Nodes 1,2 become 0,1.
        def.remove_compute_node_at(0);

        assert_eq!(def.compute_nodes.len(), 2);
        assert_eq!(def.compute_nodes[0].kind, ComputeNodeKind::Sigmoid);
        assert_eq!(def.compute_nodes[1].kind, ComputeNodeKind::Relu);

        // Relu's edge to Sigmoid: was CN(1), now CN(0)
        assert_eq!(
            def.compute_nodes[1].inputs[0].source,
            GraphSource::ComputeNode(0)
        );

        // Sink edge to Relu: was CN(2), now CN(1)
        assert_eq!(
            def.output_sinks[0].inputs[0].source,
            GraphSource::ComputeNode(1)
        );

        // Action gate edge to removed Add: CN(0) -> CN(u16::MAX)
        assert_eq!(
            def.action_bank[0].gate_inputs[0].source,
            GraphSource::ComputeNode(u16::MAX)
        );

        // Action param edge to Sigmoid: was CN(1), now CN(0)
        assert_eq!(
            def.action_bank[0].param_inputs[0].source,
            GraphSource::ComputeNode(0)
        );

        // Execute gate edge to Relu: was CN(2), now CN(1)
        assert_eq!(
            def.execute_gate.inputs[0].source,
            GraphSource::ComputeNode(1)
        );
    }

    #[test]
    fn remove_compute_node_preserves_non_compute_sources() {
        let config = MutationConfig::default();
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs(&config);

        def.compute_nodes.push(ComputeNode {
            kind: ComputeNodeKind::Add,
            inputs: vec![GraphEdge {
                source: GraphSource::InputLeaf {
                    ref_idx: 2,
                    sub_idx: 0,
                },
                weight: 1.0,
            }],
            plasticity: None,
        });

        def.compute_nodes.push(ComputeNode {
            kind: ComputeNodeKind::Sigmoid,
            inputs: vec![GraphEdge {
                source: GraphSource::SharedMemory {
                    slot: 3,
                    previous: false,
                },
                weight: 0.5,
            }],
            plasticity: None,
        });

        // Remove node 0 (Add)
        def.remove_compute_node_at(0);

        // Sigmoid's SharedMemory edge should be preserved unchanged
        assert_eq!(
            def.compute_nodes[0].inputs[0].source,
            GraphSource::SharedMemory {
                slot: 3,
                previous: false,
            }
        );
    }

    // ── Input ref reindexing tests ──────────────────────────────────────────

    #[test]
    fn reindex_input_refs_removes_matching_and_decrements_higher() {
        let config = MutationConfig::default();
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs(&config);

        def.compute_nodes.push(ComputeNode {
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
                        sub_idx: 0,
                    },
                    weight: 1.0,
                },
                GraphEdge {
                    source: GraphSource::InputLeaf {
                        ref_idx: 2,
                        sub_idx: 3,
                    },
                    weight: 1.0,
                },
                GraphEdge {
                    source: GraphSource::ComputeNode(0),
                    weight: 0.5,
                },
            ],
            plasticity: None,
        });

        // Wire a sink to InputLeaf(1)
        def.output_sinks[0].inputs.push(GraphEdge {
            source: GraphSource::InputLeaf {
                ref_idx: 1,
                sub_idx: 0,
            },
            weight: 1.0,
        });

        // Remove input_ref at index 1
        def.reindex_input_refs_after_removal(1);

        // Compute node edges: ref_idx 0 kept, ref_idx 1 removed, ref_idx 2 -> 1
        assert_eq!(def.compute_nodes[0].inputs.len(), 3);
        assert_eq!(
            def.compute_nodes[0].inputs[0].source,
            GraphSource::InputLeaf {
                ref_idx: 0,
                sub_idx: 0,
            }
        );
        assert_eq!(
            def.compute_nodes[0].inputs[1].source,
            GraphSource::InputLeaf {
                ref_idx: 1,
                sub_idx: 3,
            }
        );
        assert_eq!(
            def.compute_nodes[0].inputs[2].source,
            GraphSource::ComputeNode(0)
        );

        // Sink edge to InputLeaf(1) was removed
        assert!(def.output_sinks[0].inputs.is_empty());
    }

    #[test]
    fn reindex_across_action_bank_and_execute_gate() {
        let config = MutationConfig::default();
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs(&config);

        def.action_bank[0].gate_inputs.push(GraphEdge {
            source: GraphSource::InputLeaf {
                ref_idx: 2,
                sub_idx: 0,
            },
            weight: 1.0,
        });
        def.action_bank[1].param_inputs.push(GraphEdge {
            source: GraphSource::InputLeaf {
                ref_idx: 0,
                sub_idx: 0,
            },
            weight: 1.0,
        });
        def.execute_gate.inputs.push(GraphEdge {
            source: GraphSource::InputLeaf {
                ref_idx: 3,
                sub_idx: 0,
            },
            weight: 1.0,
        });

        // Remove input_ref at index 1
        def.reindex_input_refs_after_removal(1);

        // Gate: ref_idx 2 -> 1
        assert_eq!(
            def.action_bank[0].gate_inputs[0].source,
            GraphSource::InputLeaf {
                ref_idx: 1,
                sub_idx: 0,
            }
        );
        // Param: ref_idx 0 unchanged
        assert_eq!(
            def.action_bank[1].param_inputs[0].source,
            GraphSource::InputLeaf {
                ref_idx: 0,
                sub_idx: 0,
            }
        );
        // Execute gate: ref_idx 3 -> 2
        assert_eq!(
            def.execute_gate.inputs[0].source,
            GraphSource::InputLeaf {
                ref_idx: 2,
                sub_idx: 0,
            }
        );
    }

    // ── Sub-idx clamping tests ──────────────────────────────────────────────

    #[test]
    fn clamp_sub_idx_removes_out_of_range_edges() {
        let config = MutationConfig::default();
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs(&config);

        def.compute_nodes.push(ComputeNode {
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
                        ref_idx: 0,
                        sub_idx: 7,
                    },
                    weight: 1.0,
                },
                GraphEdge {
                    source: GraphSource::InputLeaf {
                        ref_idx: 1,
                        sub_idx: 5,
                    },
                    weight: 1.0,
                },
            ],
            plasticity: None,
        });

        // Swap ref_idx 0 from compound (width 8) to scalar (width 1)
        def.clamp_sub_idx_after_swap(0, 1);

        assert_eq!(def.compute_nodes[0].inputs.len(), 2);
        // sub_idx 0 is still valid
        assert_eq!(
            def.compute_nodes[0].inputs[0].source,
            GraphSource::InputLeaf {
                ref_idx: 0,
                sub_idx: 0,
            }
        );
        // ref_idx 1 edge is untouched
        assert_eq!(
            def.compute_nodes[0].inputs[1].source,
            GraphSource::InputLeaf {
                ref_idx: 1,
                sub_idx: 5,
            }
        );
    }

    #[test]
    fn clamp_sub_idx_zero_width_removes_all_edges_for_ref() {
        let config = MutationConfig::default();
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs(&config);

        def.compute_nodes.push(ComputeNode {
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
                        sub_idx: 0,
                    },
                    weight: 1.0,
                },
            ],
            plasticity: None,
        });

        def.clamp_sub_idx_after_swap(0, 0);

        // Only ref_idx 1 edge survives
        assert_eq!(def.compute_nodes[0].inputs.len(), 1);
        assert_eq!(
            def.compute_nodes[0].inputs[0].source,
            GraphSource::InputLeaf {
                ref_idx: 1,
                sub_idx: 0,
            }
        );
    }

    // ── Copy trait verification ─────────────────────────────────────────────

    #[test]
    fn copy_types_are_copy() {
        // These should compile — verifies Copy is derived
        let src = GraphSource::ComputeNode(0);
        let _copy = src;
        let _another = src; // still valid after copy

        let edge = GraphEdge {
            source: src,
            weight: 1.0,
        };
        let _copy = edge;
        let _another = edge;

        let kind = OutputSinkKind::CustomOutput(0);
        let _copy = kind;
        let _another = kind;

        let behavior = ActionSlotBehavior::Pop;
        let _copy = behavior;
        let _another = behavior;

        let wak = WorldActionKind::Eat;
        let _copy = wak;
        let _another = wak;

        let cnk = ComputeNodeKind::Add;
        let _copy = cnk;
        let _another = cnk;
    }
}
