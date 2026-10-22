//! CGP-style graph backend types.
//!
//! Three-layer architecture: implicit inputs (GraphSource), mutable compute
//! nodes (ComputeNodeKind), and the fixed output sink catalog (OutputSink),
//! which includes the action-vote and parameter sinks (T19.F03, T19.F04).

use crate::contracts::MAX_GATE_SLOTS;
use crate::creature::genome::vote::{
    ActionParamField, DirectedVoteKind, VoteSink, ACTION_PARAM_FIELD_COUNT, VOTE_SINK_COUNT,
};

mod container;
mod edge;
mod group;
#[cfg(test)]
pub(crate) mod test_support;

pub use container::{
    validate_graph, BirthSeeds, CgpGraphBackendDef, ComputeNode, ComputeNodes, EdgeList, EdgeMut,
    EdgeOccurrence, EdgeSurface, GraphInvariantViolation, GroupMut, OutputSink, OutputSinks,
};
pub use edge::{GraphEdge, GraphSource, Membership, Scope};
pub use group::{
    compose, detach, pair_birth_members, pair_relative, ring_offset, Group, GroupId, MemberSite,
    Shape,
};

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

// ── Fixed structural outputs ────────────────────────────────────────────────

/// Output sink kinds — fixed catalog, not evolvable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum OutputSinkKind {
    /// Write to `output_slots[slot]`. 24 slots, indices 0-23.
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
    /// Overwrite `action_params[field.index()]` (T11.F27), read when a pass
    /// commits `field.kind()` (T19.F04).
    ActionParam(ActionParamField),
}

impl OutputSinkKind {
    /// The `(kind, direction)` of a directed vote sink, `None` for any other
    /// sink.
    #[must_use]
    pub const fn direction(self) -> Option<(DirectedVoteKind, u8)> {
        match self {
            Self::ActionVote(vote) => DirectedVoteKind::of_sink(vote),
            _ => None,
        }
    }
}

/// Number of CustomOutput sinks in the fixed catalog.
pub const CUSTOM_OUTPUT_COUNT: u8 = 24;
/// Number of shared memory slots (WriteSlot + ClearSlot each).
pub const SHARED_MEMORY_SLOTS: u8 = 16;
/// Catalog index of the first `ActionVote` sink, which is also the count of
/// the sinks before it: N CustomOutput + 8 RouterGate + 16 WriteSlot +
/// 16 ClearSlot. The vote sinks run from here in `VoteSink` index order, and
/// the `ActionParam` sinks follow in `ActionParamField::ALL` order.
pub const FIRST_ACTION_VOTE_SINK: usize = CUSTOM_OUTPUT_COUNT as usize // 24 CustomOutput slots
    + MAX_GATE_SLOTS               // 8 RouterGate sinks
    + SHARED_MEMORY_SLOTS as usize // 16 WriteSlot sinks
    + SHARED_MEMORY_SLOTS as usize; // 16 ClearSlot sinks
const _: () = assert!(FIRST_ACTION_VOTE_SINK == 64);
/// Total fixed sink count: the sinks before the vote catalog + 39 ActionVote +
/// 5 ActionParam (T11.F27, T02.F08, T27.F03).
pub const FIXED_SINK_COUNT: usize =
    FIRST_ACTION_VOTE_SINK + VOTE_SINK_COUNT + ACTION_PARAM_FIELD_COUNT;
const _: () = assert!(FIXED_SINK_COUNT == 108);

#[cfg(test)]
mod tests {
    use super::*;

    // ── Construction tests ──────────────────────────────────────────────────

    #[test]
    fn new_with_fixed_outputs_creates_correct_catalog() {
        let def = CgpGraphBackendDef::new_with_fixed_outputs();

        assert_eq!(def.output_sinks().len(), FIXED_SINK_COUNT);
        assert!(def.compute_nodes().is_empty());

        // Verify sink ordering
        for i in 0..CUSTOM_OUTPUT_COUNT {
            assert_eq!(
                def.output_sinks()[i as usize].kind,
                OutputSinkKind::CustomOutput(i)
            );
        }
        for i in 0..MAX_GATE_SLOTS {
            assert_eq!(
                def.output_sinks()[CUSTOM_OUTPUT_COUNT as usize + i].kind,
                OutputSinkKind::RouterGate(i as u8)
            );
        }
        for i in 0..SHARED_MEMORY_SLOTS {
            assert_eq!(
                def.output_sinks()[CUSTOM_OUTPUT_COUNT as usize + MAX_GATE_SLOTS + i as usize].kind,
                OutputSinkKind::WriteSlot(i)
            );
        }
        for i in 0..SHARED_MEMORY_SLOTS {
            assert_eq!(
                def.output_sinks()[CUSTOM_OUTPUT_COUNT as usize
                    + MAX_GATE_SLOTS
                    + SHARED_MEMORY_SLOTS as usize
                    + i as usize]
                    .kind,
                OutputSinkKind::ClearSlot(i)
            );
        }
        // The vote sinks occupy 64..103 in `VoteSink` index order (T19.F03, T11.F30).
        assert_eq!(FIRST_ACTION_VOTE_SINK, 64);
        for (offset, sink) in VoteSink::all().enumerate() {
            assert_eq!(
                def.output_sinks()[FIRST_ACTION_VOTE_SINK + offset].kind,
                OutputSinkKind::ActionVote(sink)
            );
        }
        // The five parameter sinks occupy 103..108 in `ActionParamField::ALL`
        // order (T11.F27).
        let first_param = FIRST_ACTION_VOTE_SINK + VOTE_SINK_COUNT;
        assert_eq!(first_param, 103);
        let params: Vec<OutputSinkKind> = def.output_sinks()[first_param..]
            .iter()
            .map(|sink| sink.kind)
            .collect();
        assert_eq!(
            params,
            ActionParamField::ALL.map(OutputSinkKind::ActionParam)
        );
        assert_eq!(def.output_sinks().len(), 108);

        // All sinks start with empty edges
        for sink in def.output_sinks() {
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
        def.append_compute_node(ComputeNode::new(
            ComputeNodeKind::Threshold(24.0),
            vec![GraphEdge::new(
                GraphSource::InputLeaf {
                    ref_idx: 1,
                    sub_idx: 0,
                },
                1.0,
            )],
        ));

        // Wire an output sink
        def.push_edge(
            EdgeSurface::SinkInput(0),
            GraphEdge::new(GraphSource::ComputeNode(0), 1.0),
        );

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
            def.append_compute_node(ComputeNode::new(kind, Vec::new()));
        }

        // Compute node 2 (Relu) has edge to node 1 (Sigmoid)
        def.push_edge(
            EdgeSurface::ComputeInput(2),
            GraphEdge::new(GraphSource::ComputeNode(1), 1.0),
        );

        // Output sink has edge to node 2 (Relu)
        def.push_edge(
            EdgeSurface::SinkInput(0),
            GraphEdge::new(GraphSource::ComputeNode(2), 0.5),
        );

        // Vote sink has edge to node 0 (Add)
        def.push_edge(
            EdgeSurface::SinkInput(FIRST_ACTION_VOTE_SINK),
            GraphEdge::new(GraphSource::ComputeNode(0), 1.0),
        );

        // Second vote sink has edge to node 1 (Sigmoid)
        def.push_edge(
            EdgeSurface::SinkInput(FIRST_ACTION_VOTE_SINK + 1),
            GraphEdge::new(GraphSource::ComputeNode(1), 1.0),
        );

        // Parameter sink has edge to node 2 (Relu)
        def.push_edge(
            EdgeSurface::SinkInput(FIXED_SINK_COUNT - 1),
            GraphEdge::new(GraphSource::ComputeNode(2), 1.0),
        );

        // Remove node 0 (Add). Nodes 1,2 become 0,1.
        def.remove_compute_node(0);

        assert_eq!(def.compute_nodes().len(), 2);
        assert_eq!(def.compute_nodes()[0].kind, ComputeNodeKind::Sigmoid);
        assert_eq!(def.compute_nodes()[1].kind, ComputeNodeKind::Relu);

        // Relu's edge to Sigmoid: was CN(1), now CN(0)
        assert_eq!(
            def.compute_nodes()[1].inputs[0].source,
            GraphSource::ComputeNode(0)
        );

        // Sink edge to Relu: was CN(2), now CN(1)
        assert_eq!(
            def.output_sinks()[0].inputs[0].source,
            GraphSource::ComputeNode(1)
        );

        // Action gate edge to removed Add: CN(0) -> CN(u16::MAX)
        assert_eq!(
            def.output_sinks()[FIRST_ACTION_VOTE_SINK].inputs[0].source,
            GraphSource::ComputeNode(u16::MAX)
        );

        // Action param edge to Sigmoid: was CN(1), now CN(0)
        assert_eq!(
            def.output_sinks()[FIRST_ACTION_VOTE_SINK + 1].inputs[0].source,
            GraphSource::ComputeNode(0)
        );

        // Last sink (the final parameter sink) edge to Relu: was CN(2), now CN(1)
        assert_eq!(
            def.output_sinks()[FIXED_SINK_COUNT - 1].inputs[0].source,
            GraphSource::ComputeNode(1)
        );
    }

    #[test]
    fn remove_compute_node_preserves_non_compute_sources() {
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();

        def.append_compute_node(ComputeNode::new(
            ComputeNodeKind::Add,
            vec![GraphEdge::new(
                GraphSource::InputLeaf {
                    ref_idx: 2,
                    sub_idx: 0,
                },
                1.0,
            )],
        ));

        def.append_compute_node(ComputeNode::new(
            ComputeNodeKind::Sigmoid,
            vec![GraphEdge::new(
                GraphSource::SharedMemory {
                    slot: 3,
                    previous: false,
                },
                0.5,
            )],
        ));

        // Remove node 0 (Add)
        def.remove_compute_node(0);

        // Sigmoid's SharedMemory edge should be preserved unchanged
        assert_eq!(
            def.compute_nodes()[0].inputs[0].source,
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
        def.append_compute_node(ComputeNode::new(ComputeNodeKind::Sigmoid, Vec::new()));
        def.append_compute_node(ComputeNode::new(
            ComputeNodeKind::Relu,
            vec![GraphEdge::new(GraphSource::ComputeNode(0), 1.0)],
        ));
        def.push_edge(
            EdgeSurface::SinkInput(0),
            GraphEdge::new(GraphSource::ComputeNode(1), 0.5),
        );
        def.push_edge(
            EdgeSurface::SinkInput(FIRST_ACTION_VOTE_SINK),
            GraphEdge::new(GraphSource::ComputeNode(0), 1.0),
        );
        def.push_edge(
            EdgeSurface::SinkInput(FIXED_SINK_COUNT - 1),
            GraphEdge::new(GraphSource::ComputeNode(1), 1.0),
        );

        // Insert a new node at index 1 (between Sigmoid and Relu).
        def.insert_compute_node(1, ComputeNode::new(ComputeNodeKind::Add, Vec::new()));

        assert_eq!(def.compute_nodes().len(), 3);
        assert_eq!(def.compute_nodes()[0].kind, ComputeNodeKind::Sigmoid);
        assert_eq!(def.compute_nodes()[1].kind, ComputeNodeKind::Add);
        assert_eq!(def.compute_nodes()[2].kind, ComputeNodeKind::Relu);

        // Relu's edge to Sigmoid: was CN(0), unaffected (below insertion point).
        assert_eq!(
            def.compute_nodes()[2].inputs[0].source,
            GraphSource::ComputeNode(0)
        );
        // Sink edge to Relu: was CN(1), now CN(2).
        assert_eq!(
            def.output_sinks()[0].inputs[0].source,
            GraphSource::ComputeNode(2)
        );
        // Action gate edge to Sigmoid: was CN(0), unaffected.
        assert_eq!(
            def.output_sinks()[FIRST_ACTION_VOTE_SINK].inputs[0].source,
            GraphSource::ComputeNode(0)
        );
        // Last sink (the final parameter sink) edge to Relu: was CN(1), now CN(2).
        assert_eq!(
            def.output_sinks()[FIXED_SINK_COUNT - 1].inputs[0].source,
            GraphSource::ComputeNode(2)
        );
    }

    #[test]
    fn insert_compute_node_at_shifts_self_referencing_edge() {
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
        def.append_compute_node(ComputeNode::new(
            ComputeNodeKind::DecayIntegrator(0.5),
            vec![GraphEdge::new(GraphSource::ComputeNode(0), 1.0)],
        ));

        def.insert_compute_node(0, ComputeNode::new(ComputeNodeKind::Add, Vec::new()));

        assert_eq!(def.compute_nodes().len(), 2);
        // The self-loop now refers to the shifted node's own new index (1).
        assert_eq!(
            def.compute_nodes()[1].inputs[0].source,
            GraphSource::ComputeNode(1)
        );
    }

    #[test]
    fn insert_compute_node_at_and_remove_compute_node_at_are_inverse_on_indices() {
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
        for kind in [ComputeNodeKind::Add, ComputeNodeKind::Sigmoid] {
            def.append_compute_node(ComputeNode::new(kind, Vec::new()));
        }
        let before = def.clone();

        def.insert_compute_node(1, ComputeNode::new(ComputeNodeKind::Relu, Vec::new()));
        def.remove_compute_node(1);

        assert_eq!(def, before);
    }

    // ── Input ref reindexing tests ──────────────────────────────────────────

    #[test]
    fn reindex_input_refs_decrements_higher_and_keeps_lower() {
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
        def.append_compute_node(ComputeNode::new(
            ComputeNodeKind::Add,
            vec![
                GraphEdge::new(
                    GraphSource::InputLeaf {
                        ref_idx: 0,
                        sub_idx: 0,
                    },
                    1.0,
                ),
                GraphEdge::new(
                    GraphSource::InputLeaf {
                        ref_idx: 2,
                        sub_idx: 3,
                    },
                    1.0,
                ),
                GraphEdge::new(GraphSource::ComputeNode(0), 0.5),
            ],
        ));
        def.push_edge(
            EdgeSurface::SinkInput(0),
            GraphEdge::new(
                GraphSource::InputLeaf {
                    ref_idx: 3,
                    sub_idx: 0,
                },
                1.0,
            ),
        );

        // `Prune` removed index 1, which nothing addresses.
        def.reindex_input_refs_after_removal(1);

        // Nothing is removed: ref_idx 0 is kept, 2 -> 1 and 3 -> 2.
        let sources: Vec<GraphSource> = def.edges().map(|edge| edge.source).collect();
        assert_eq!(
            sources,
            vec![
                GraphSource::InputLeaf {
                    ref_idx: 0,
                    sub_idx: 0,
                },
                GraphSource::InputLeaf {
                    ref_idx: 1,
                    sub_idx: 3,
                },
                GraphSource::ComputeNode(0),
                GraphSource::InputLeaf {
                    ref_idx: 2,
                    sub_idx: 0,
                },
            ]
        );
    }

    #[test]
    fn reindex_across_vote_and_param_sinks() {
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
        let leaf = |ref_idx| {
            GraphEdge::new(
                GraphSource::InputLeaf {
                    ref_idx,
                    sub_idx: 0,
                },
                1.0,
            )
        };
        def.push_edge(EdgeSurface::SinkInput(FIRST_ACTION_VOTE_SINK), leaf(2));
        def.push_edge(EdgeSurface::SinkInput(FIRST_ACTION_VOTE_SINK + 2), leaf(0));
        def.push_edge(EdgeSurface::SinkInput(FIXED_SINK_COUNT - 1), leaf(3));

        def.reindex_input_refs_after_removal(1);

        // Vote: ref_idx 2 -> 1.
        assert_eq!(
            def.output_sinks()[FIRST_ACTION_VOTE_SINK].inputs[0],
            leaf(1)
        );
        // Vote: ref_idx 0 unchanged.
        assert_eq!(
            def.output_sinks()[FIRST_ACTION_VOTE_SINK + 2].inputs[0],
            leaf(0)
        );
        // Last sink (an `ActionParam`): ref_idx 3 -> 2.
        assert_eq!(def.output_sinks()[FIXED_SINK_COUNT - 1].inputs[0], leaf(2));
    }

    /// A removed index some edge still addresses is a caller bug: `Prune`
    /// only removes unaddressed indices.
    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "a pruned input reference is addressed by no edge")]
    fn reindex_of_an_addressed_index_is_a_debug_assertion() {
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
        def.push_edge(
            EdgeSurface::SinkInput(0),
            GraphEdge::new(
                GraphSource::InputLeaf {
                    ref_idx: 1,
                    sub_idx: 0,
                },
                1.0,
            ),
        );
        def.reindex_input_refs_after_removal(1);
    }

    // ── Copy trait verification ─────────────────────────────────────────────

    #[test]
    fn copy_types_are_copy() {
        // These should compile — verifies Copy is derived
        let src = GraphSource::ComputeNode(0);
        let _copy = src;
        let _another = src; // still valid after copy

        let edge = GraphEdge::new(src, 1.0);
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
