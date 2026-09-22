//! CGP-style graph backend types.
//!
//! Three-layer architecture: implicit inputs (GraphSource), mutable compute
//! nodes (ComputeNodeKind), and the fixed output sink catalog (OutputSink),
//! which includes the action-vote and parameter sinks (T19.F03, T19.F04).

use crate::contracts::MAX_GATE_SLOTS;
use crate::creature::genome::vote::{
    VoteKind, VoteSink, VOTE_KIND_COUNT, VOTE_PARAM_SLOTS, VOTE_SINK_COUNT,
};

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
    /// Contribute to the pass vote `votes[sink.index()]` (T19.F03). 27 sinks;
    /// the pass end commits from them (T19.F04).
    ActionVote(VoteSink),
    /// Overwrite `action_params[kind][slot]` (T19.F03), `slot` in `0..2`,
    /// read when a pass commits `kind` (T19.F04).
    ActionParam(VoteKind, u8),
}

/// Fixed structural output — one per target slot.
/// Kind is NOT mutated; only inputs (edges) change via mutation.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OutputSink {
    pub kind: OutputSinkKind,
    pub inputs: Vec<GraphEdge>,
}

// ── Top-level graph backend ─────────────────────────────────────────────────

/// CGP-style graph backend definition.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct CgpGraphBackendDef {
    /// Birth-only available learned values, aligned with compute input occurrences.
    /// Excluded from genome identity and removed before the newborn is constructed.
    #[serde(skip)]
    #[doc(hidden)]
    pub birth_weights: Option<Vec<Vec<Option<f32>>>>,
    pub compute_nodes: Vec<ComputeNode>,
    /// Fixed set — structurally immutable. Only edges are evolvable.
    pub output_sinks: Vec<OutputSink>,
}

// Keep temporary birth state out of diagnostic genome identity as well as
// serialization and equality. Existing state fingerprints use this format.
impl std::fmt::Debug for CgpGraphBackendDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CgpGraphBackendDef")
            .field("compute_nodes", &self.compute_nodes)
            .field("output_sinks", &self.output_sinks)
            .finish()
    }
}

impl PartialEq for CgpGraphBackendDef {
    fn eq(&self, other: &Self) -> bool {
        self.compute_nodes == other.compute_nodes && self.output_sinks == other.output_sinks
    }
}

/// Number of CustomOutput sinks in the fixed catalog.
pub const CUSTOM_OUTPUT_COUNT: u8 = 24;
/// Number of shared memory slots (WriteSlot + ClearSlot each).
pub const SHARED_MEMORY_SLOTS: u8 = 16;
/// Catalog index of the first `ActionVote` sink, which is also the count of
/// the sinks before it: N CustomOutput + 8 RouterGate + 16 WriteSlot +
/// 16 ClearSlot. The vote sinks run from here in `VoteSink` index order, and
/// the `ActionParam` sinks follow kind-major.
pub const FIRST_ACTION_VOTE_SINK: usize = CUSTOM_OUTPUT_COUNT as usize // 24 CustomOutput slots
    + MAX_GATE_SLOTS               // 8 RouterGate sinks
    + SHARED_MEMORY_SLOTS as usize // 16 WriteSlot sinks
    + SHARED_MEMORY_SLOTS as usize; // 16 ClearSlot sinks
const _: () = assert!(FIRST_ACTION_VOTE_SINK == 64);
/// Total fixed sink count: the sinks before the vote catalog + 27 ActionVote +
/// 8 ActionParam (T19.F03).
pub const FIXED_SINK_COUNT: usize =
    FIRST_ACTION_VOTE_SINK + VOTE_SINK_COUNT + VOTE_KIND_COUNT * VOTE_PARAM_SLOTS as usize;
const _: () = assert!(FIXED_SINK_COUNT == 99);

impl CgpGraphBackendDef {
    /// The output sink of `kind`, when the graph carries it.
    #[must_use]
    pub fn sink(&self, kind: OutputSinkKind) -> Option<&OutputSink> {
        self.output_sinks.iter().find(|sink| sink.kind == kind)
    }

    /// The output sink of `kind`, mutably, when the graph carries it.
    pub fn sink_mut(&mut self, kind: OutputSinkKind) -> Option<&mut OutputSink> {
        self.output_sinks.iter_mut().find(|sink| sink.kind == kind)
    }

    /// Construct a new graph backend with the full fixed output catalog.
    /// All sinks start with empty edges (inert).
    pub fn new_with_fixed_outputs() -> Self {
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

        // 27 ActionVote sinks in `VoteSink` index order (T19.F03).
        for sink in VoteSink::all() {
            output_sinks.push(OutputSink {
                kind: OutputSinkKind::ActionVote(sink),
                inputs: Vec::new(),
            });
        }

        // 8 ActionParam sinks, kind-major (T19.F03).
        for kind in VoteKind::ALL {
            for slot in 0..VOTE_PARAM_SLOTS {
                output_sinks.push(OutputSink {
                    kind: OutputSinkKind::ActionParam(kind, slot),
                    inputs: Vec::new(),
                });
            }
        }

        Self {
            compute_nodes: Vec::new(),
            output_sinks,
            birth_weights: None,
        }
    }

    /// Whether a visit to this graph enters evaluation and the effects pass.
    ///
    /// True with at least one compute node, and true for a zero-compute graph
    /// with a wired output sink. A graph with neither computes nothing and
    /// applies nothing, so its visit is free.
    /// Derived from the genome on every visit and stored nowhere.
    #[must_use]
    pub fn enters_visit(&self) -> bool {
        !self.compute_nodes.is_empty()
            || self.output_sinks.iter().any(|sink| !sink.inputs.is_empty())
    }

    /// Remove a compute node at `idx`. Remaps `GraphSource::ComputeNode`
    /// indices across both edge containers: compute inputs and sink inputs.
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
        if let Some(weights) = &mut self.birth_weights {
            weights.remove(idx);
            for (node, values) in self.compute_nodes.iter().zip(weights) {
                for (edge, inherited) in node.inputs.iter().zip(values) {
                    if edge.source == GraphSource::ComputeNode(idx as u16) {
                        *inherited = None;
                    }
                }
            }
        }
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

    /// Insert `node` at compute index `idx`, the inverse of
    /// [`Self::remove_compute_node_at`]. Every `GraphSource::ComputeNode(i)`
    /// reference at or above `idx`, across both edge-bearing surfaces
    /// (including edges belonging to `node` itself, resolved by the caller
    /// before insertion), is incremented by one so every surviving edge keeps
    /// pointing at the same logical node after the shift.
    pub fn insert_compute_node_at(&mut self, idx: usize, node: ComputeNode) {
        debug_assert!(
            idx <= self.compute_nodes.len(),
            "insert_compute_node_at: idx {} out of bounds (len {})",
            idx,
            self.compute_nodes.len()
        );
        let at = idx as u16;
        self.remap_compute_sources(|src_idx| {
            // `u16::MAX` is `remove_compute_node_at`'s dangling-reference
            // sentinel, never a real index; leave it alone rather than
            // overflow.
            if src_idx != u16::MAX && src_idx >= at {
                src_idx + 1
            } else {
                src_idx
            }
        });
        if let Some(weights) = &mut self.birth_weights {
            weights.insert(idx, vec![None; node.inputs.len()]);
        }
        self.compute_nodes.insert(idx, node);
    }

    /// Duplicate the compute nodes at `sources`, inserting each copy directly
    /// after its original so the copy reads every source in the same
    /// evaluation phase its original reads: a source below the original stays
    /// below the copy and is read from this visit, a source at or above it
    /// stays above and is read from the last committed outputs
    /// (`runtime/cgp/sources.rs`, T11.F06). The `i`-th source ends at index
    /// `sources[i] + i` and its copy at `sources[i] + i + 1`.
    ///
    /// An edge from one duplicated node to another, including a self-edge,
    /// points at the corresponding copy, so the duplicated set's internal
    /// wiring and per-node state are its own. Every other reference, in the
    /// copies and across both edge-bearing surfaces, is remapped to its
    /// shifted index, so no surviving edge changes what it reads.
    ///
    /// `sources` must be strictly ascending and in range; callers must keep
    /// `compute_nodes.len() + sources.len()` within `u16::MAX`.
    pub fn duplicate_compute_nodes_in_place(&mut self, sources: &[usize]) {
        let old_len = self.compute_nodes.len();
        debug_assert!(
            sources.windows(2).all(|pair| pair[0] < pair[1])
                && sources.last().is_none_or(|&last| last < old_len),
            "duplicate_compute_nodes_in_place: sources must be strictly ascending and in range"
        );
        let originals: Vec<ComputeNode> = sources
            .iter()
            .map(|&idx| self.compute_nodes[idx].clone())
            .collect();
        // A surviving reference moves up by the number of duplicated nodes
        // below it. Out-of-range indices, including `remove_compute_node_at`'s
        // `u16::MAX` sentinel, are left alone.
        let shift = |src_idx: u16| {
            if (src_idx as usize) >= old_len {
                src_idx
            } else {
                src_idx + sources.partition_point(|&c| c < src_idx as usize) as u16
            }
        };
        if let Some(weights) = &mut self.birth_weights {
            for &source in sources.iter().rev() {
                weights.insert(source + 1, weights[source].clone());
            }
        }
        self.remap_compute_sources(shift);
        for (i, mut copy) in originals.into_iter().enumerate().rev() {
            for edge in &mut copy.inputs {
                if let GraphSource::ComputeNode(ref mut src_idx) = edge.source {
                    *src_idx = match sources.binary_search(&(*src_idx as usize)) {
                        Ok(position) => (sources[position] + position + 1) as u16,
                        Err(_) => shift(*src_idx),
                    };
                }
            }
            self.compute_nodes.insert(sources[i] + 1, copy);
        }
    }

    /// After an input_ref is removed at `removed_ref_idx`, update all
    /// `GraphSource::InputLeaf { ref_idx }` across all edge containers.
    /// Matching ref_idx edges are removed. Higher ref_idx values are decremented.
    pub fn reindex_input_refs_after_removal(&mut self, removed_ref_idx: u16) {
        self.retain_edges(|edge| {
            if let GraphSource::InputLeaf { ref_idx, .. } = &mut edge.source {
                if *ref_idx == removed_ref_idx {
                    return false;
                }
                if *ref_idx > removed_ref_idx {
                    *ref_idx -= 1;
                }
            }
            true
        });
    }

    /// Remove edges outside the replacement input reference's width.
    pub fn clamp_sub_idx_after_swap(&mut self, ref_idx: u16, new_width: u16) {
        self.retain_edges(|edge| {
            !matches!(edge.source,
            GraphSource::InputLeaf { ref_idx: r, sub_idx } if r == ref_idx && sub_idx >= new_width)
        });
    }

    /// Every edge on every container `retain_edges` walks: all compute
    /// nodes (live or not), then the output sinks.
    pub fn edges(&self) -> impl Iterator<Item = &GraphEdge> {
        self.compute_nodes
            .iter()
            .flat_map(|node| &node.inputs)
            .chain(self.output_sinks.iter().flat_map(|sink| &sink.inputs))
    }

    fn retain_edges(&mut self, mut keep: impl FnMut(&mut GraphEdge) -> bool) {
        for (idx, node) in self.compute_nodes.iter_mut().enumerate() {
            let mut values = self.birth_weights.as_mut().map(|rows| &mut rows[idx]);
            let mut read = 0;
            let mut written = 0;
            node.inputs.retain_mut(|edge| {
                let retained = keep(edge);
                if retained {
                    if let Some(values) = &mut values {
                        values[written] = values[read];
                    }
                    written += 1;
                }
                read += 1;
                retained
            });
            if let Some(values) = values {
                values.truncate(written);
            }
        }
        for sink in &mut self.output_sinks {
            sink.inputs.retain_mut(&mut keep);
        }
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Construction tests ──────────────────────────────────────────────────

    #[test]
    fn new_with_fixed_outputs_creates_correct_catalog() {
        let def = CgpGraphBackendDef::new_with_fixed_outputs();

        assert_eq!(def.output_sinks.len(), FIXED_SINK_COUNT);
        assert!(def.compute_nodes.is_empty());

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
        // The vote sinks occupy 64..91 in `VoteSink` index order (T19.F03).
        assert_eq!(FIRST_ACTION_VOTE_SINK, 64);
        for (offset, sink) in VoteSink::all().enumerate() {
            assert_eq!(
                def.output_sinks[FIRST_ACTION_VOTE_SINK + offset].kind,
                OutputSinkKind::ActionVote(sink)
            );
        }
        // The parameter sinks occupy 91..99, kind-major.
        let first_param = FIRST_ACTION_VOTE_SINK + VOTE_SINK_COUNT;
        assert_eq!(first_param, 91);
        for (kind_index, kind) in VoteKind::ALL.into_iter().enumerate() {
            for slot in 0..VOTE_PARAM_SLOTS {
                let index = first_param + kind_index * VOTE_PARAM_SLOTS as usize + slot as usize;
                assert_eq!(
                    def.output_sinks[index].kind,
                    OutputSinkKind::ActionParam(kind, slot)
                );
            }
        }
        assert_eq!(def.output_sinks.len(), 99);

        // All sinks start with empty edges
        for sink in &def.output_sinks {
            assert!(sink.inputs.is_empty());
        }
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
    fn full_backend_def_serde_roundtrip() {
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();

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
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();

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

        // Vote sink has edge to node 0 (Add)
        def.output_sinks[FIRST_ACTION_VOTE_SINK]
            .inputs
            .push(GraphEdge {
                source: GraphSource::ComputeNode(0),
                weight: 1.0,
            });

        // Second vote sink has edge to node 1 (Sigmoid)
        def.output_sinks[FIRST_ACTION_VOTE_SINK + 1]
            .inputs
            .push(GraphEdge {
                source: GraphSource::ComputeNode(1),
                weight: 1.0,
            });

        // Parameter sink has edge to node 2 (Relu)
        def.output_sinks[FIXED_SINK_COUNT - 1]
            .inputs
            .push(GraphEdge {
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
            def.output_sinks[FIRST_ACTION_VOTE_SINK].inputs[0].source,
            GraphSource::ComputeNode(u16::MAX)
        );

        // Action param edge to Sigmoid: was CN(1), now CN(0)
        assert_eq!(
            def.output_sinks[FIRST_ACTION_VOTE_SINK + 1].inputs[0].source,
            GraphSource::ComputeNode(0)
        );

        // Last sink (ActionParam(StealEnergy, 1)) edge to Relu: was CN(2), now CN(1)
        assert_eq!(
            def.output_sinks[FIXED_SINK_COUNT - 1].inputs[0].source,
            GraphSource::ComputeNode(1)
        );
    }

    #[test]
    fn remove_compute_node_preserves_non_compute_sources() {
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();

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

    // ── Structural insertion tests (T11.F03 split insert) ───────────────────

    #[test]
    fn insert_compute_node_at_remaps_edges_across_all_containers() {
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();

        // Two compute nodes: 0 (Sigmoid), 1 (Relu), Relu reads Sigmoid.
        def.compute_nodes.push(ComputeNode {
            kind: ComputeNodeKind::Sigmoid,
            inputs: Vec::new(),
            plasticity: None,
        });
        def.compute_nodes.push(ComputeNode {
            kind: ComputeNodeKind::Relu,
            inputs: vec![GraphEdge {
                source: GraphSource::ComputeNode(0),
                weight: 1.0,
            }],
            plasticity: None,
        });
        def.output_sinks[0].inputs.push(GraphEdge {
            source: GraphSource::ComputeNode(1),
            weight: 0.5,
        });
        def.output_sinks[FIRST_ACTION_VOTE_SINK]
            .inputs
            .push(GraphEdge {
                source: GraphSource::ComputeNode(0),
                weight: 1.0,
            });
        def.output_sinks[FIXED_SINK_COUNT - 1]
            .inputs
            .push(GraphEdge {
                source: GraphSource::ComputeNode(1),
                weight: 1.0,
            });

        // Insert a new node at index 1 (between Sigmoid and Relu).
        def.insert_compute_node_at(
            1,
            ComputeNode {
                kind: ComputeNodeKind::Add,
                inputs: Vec::new(),
                plasticity: None,
            },
        );

        assert_eq!(def.compute_nodes.len(), 3);
        assert_eq!(def.compute_nodes[0].kind, ComputeNodeKind::Sigmoid);
        assert_eq!(def.compute_nodes[1].kind, ComputeNodeKind::Add);
        assert_eq!(def.compute_nodes[2].kind, ComputeNodeKind::Relu);

        // Relu's edge to Sigmoid: was CN(0), unaffected (below insertion point).
        assert_eq!(
            def.compute_nodes[2].inputs[0].source,
            GraphSource::ComputeNode(0)
        );
        // Sink edge to Relu: was CN(1), now CN(2).
        assert_eq!(
            def.output_sinks[0].inputs[0].source,
            GraphSource::ComputeNode(2)
        );
        // Action gate edge to Sigmoid: was CN(0), unaffected.
        assert_eq!(
            def.output_sinks[FIRST_ACTION_VOTE_SINK].inputs[0].source,
            GraphSource::ComputeNode(0)
        );
        // Last sink (ActionParam(StealEnergy, 1)) edge to Relu: was CN(1), now CN(2).
        assert_eq!(
            def.output_sinks[FIXED_SINK_COUNT - 1].inputs[0].source,
            GraphSource::ComputeNode(2)
        );
    }

    #[test]
    fn insert_compute_node_at_shifts_self_referencing_edge() {
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
        def.compute_nodes.push(ComputeNode {
            kind: ComputeNodeKind::DecayIntegrator(0.5),
            inputs: vec![GraphEdge {
                source: GraphSource::ComputeNode(0),
                weight: 1.0,
            }],
            plasticity: None,
        });

        def.insert_compute_node_at(
            0,
            ComputeNode {
                kind: ComputeNodeKind::Add,
                inputs: Vec::new(),
                plasticity: None,
            },
        );

        assert_eq!(def.compute_nodes.len(), 2);
        // The self-loop now refers to the shifted node's own new index (1).
        assert_eq!(
            def.compute_nodes[1].inputs[0].source,
            GraphSource::ComputeNode(1)
        );
    }

    #[test]
    fn insert_compute_node_at_and_remove_compute_node_at_are_inverse_on_indices() {
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
        for kind in [ComputeNodeKind::Add, ComputeNodeKind::Sigmoid] {
            def.compute_nodes.push(ComputeNode {
                kind,
                inputs: Vec::new(),
                plasticity: None,
            });
        }
        let before = def.clone();

        def.insert_compute_node_at(
            1,
            ComputeNode {
                kind: ComputeNodeKind::Relu,
                inputs: Vec::new(),
                plasticity: None,
            },
        );
        def.remove_compute_node_at(1);

        assert_eq!(def, before);
    }

    // ── Input ref reindexing tests ──────────────────────────────────────────

    #[test]
    fn reindex_input_refs_removes_matching_and_decrements_higher() {
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();

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
    fn reindex_across_vote_and_param_sinks() {
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();

        def.output_sinks[FIRST_ACTION_VOTE_SINK]
            .inputs
            .push(GraphEdge {
                source: GraphSource::InputLeaf {
                    ref_idx: 2,
                    sub_idx: 0,
                },
                weight: 1.0,
            });
        def.output_sinks[FIRST_ACTION_VOTE_SINK + 2]
            .inputs
            .push(GraphEdge {
                source: GraphSource::InputLeaf {
                    ref_idx: 0,
                    sub_idx: 0,
                },
                weight: 1.0,
            });
        def.output_sinks[FIXED_SINK_COUNT - 1]
            .inputs
            .push(GraphEdge {
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
            def.output_sinks[FIRST_ACTION_VOTE_SINK].inputs[0].source,
            GraphSource::InputLeaf {
                ref_idx: 1,
                sub_idx: 0,
            }
        );
        // Param: ref_idx 0 unchanged
        assert_eq!(
            def.output_sinks[FIRST_ACTION_VOTE_SINK + 2].inputs[0].source,
            GraphSource::InputLeaf {
                ref_idx: 0,
                sub_idx: 0,
            }
        );
        // Last sink (ActionParam(StealEnergy, 1)): ref_idx 3 -> 2
        assert_eq!(
            def.output_sinks[FIXED_SINK_COUNT - 1].inputs[0].source,
            GraphSource::InputLeaf {
                ref_idx: 2,
                sub_idx: 0,
            }
        );
    }

    // ── Sub-idx clamping tests ──────────────────────────────────────────────

    #[test]
    fn clamp_sub_idx_removes_out_of_range_edges() {
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();

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
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();

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

        let cnk = ComputeNodeKind::Add;
        let _copy = cnk;
        let _another = cnk;
    }
}
