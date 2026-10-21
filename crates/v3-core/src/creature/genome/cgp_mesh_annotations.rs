//! CGP graph backend mesh annotation derivation.
//!
//! Derives read/write classes and stateful flags from the CGP three-layer model:
//! - Read classes from `GraphSource::InputLeaf` ref_idx in edges
//! - Write classes from `OutputSinkKind`; the vote and parameter sinks are
//!   class `Action`
//! - Stateful from `ComputeNodeKind` + plasticity

use std::collections::BTreeSet;

use crate::contracts::InputReference;
use crate::creature::genome::cgp::{
    CgpGraphBackendDef, ComputeNodeKind, EdgeSurface, GraphSource, OutputSinkKind,
};
use crate::creature::genome::mesh_annotations::{
    classify_input_ref, MeshReadClass, MeshWriteClass,
};

use super::cgp_analysis::{cgp_live_compute_indices, wired_surface_edges};

/// Derive read/write/stateful annotations for a CGP graph backend.
///
/// Returns (read_classes, write_classes, has_stateful_behavior, live_compute_indices).
pub(crate) fn derive_cgp_annotations(
    def: &CgpGraphBackendDef,
    input_refs: &[InputReference],
) -> (Vec<MeshReadClass>, Vec<MeshWriteClass>, bool, Vec<usize>) {
    let mut read_classes = BTreeSet::new();
    let mut write_classes = BTreeSet::new();

    let live_indices = cgp_live_compute_indices(def);

    // Learning (T25.F06): a plastic edge into a live compute node or a
    // sink (a sink with an edge is wired).
    let mut has_stateful_behavior = def.plastic_edges().any(|(site, _)| match site.surface {
        EdgeSurface::ComputeInput(i) => live_indices.contains(&i),
        EdgeSurface::SinkInput(_) => true,
    });

    // Stateful flags from live compute nodes
    for &idx in &live_indices {
        let node = &def.compute_nodes()[idx];
        match &node.kind {
            ComputeNodeKind::DecayIntegrator(_)
            | ComputeNodeKind::Momentum(_)
            | ComputeNodeKind::Oscillator(_)
            | ComputeNodeKind::AdaptiveGain => {
                has_stateful_behavior = true;
            }
            _ => {}
        }

        // Narrower than `sensor_census` on purpose: a shared-memory read
        // counts here only on a live compute node, where that census counts it
        // on the whole wired surface. Do not harmonize the two.
        has_stateful_behavior |= node
            .inputs
            .iter()
            .any(|edge| matches!(edge.source(), GraphSource::SharedMemory { .. }));
    }

    // Read classes from InputLeaf edges anywhere on the wired surface.
    for edge in wired_surface_edges(def, &live_indices) {
        if let GraphSource::InputLeaf { ref_idx, .. } = edge.source() {
            if let Some(input_ref) = input_refs.get(ref_idx as usize) {
                read_classes.insert(classify_input_ref(input_ref));
            }
        }
    }

    // Write classes from wired output sinks
    for sink in def.output_sinks() {
        if sink.inputs.is_empty() {
            continue;
        }
        match &sink.kind {
            OutputSinkKind::CustomOutput(_) => {
                write_classes.insert(MeshWriteClass::Payload);
            }
            OutputSinkKind::RouterGate(_) => {
                write_classes.insert(MeshWriteClass::Route);
            }
            OutputSinkKind::WriteSlot(_) | OutputSinkKind::ClearSlot(_) => {
                has_stateful_behavior = true;
                write_classes.insert(MeshWriteClass::Memory);
            }
            // The vote and parameter sinks write the action channel (T19.F04).
            OutputSinkKind::ActionVote(_) | OutputSinkKind::ActionParam(_) => {
                write_classes.insert(MeshWriteClass::Action);
            }
        }
    }

    (
        read_classes.into_iter().collect(),
        write_classes.into_iter().collect(),
        has_stateful_behavior,
        live_indices,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::OrdinaryFoodTypeId;
    use crate::contracts::WorldInputKey;
    use crate::creature::genome::cgp::{
        ComputeNode, ComputeNodeKind, GraphEdge, OutputSink, OutputSinkKind,
    };
    use crate::creature::genome::vote::DirectedVoteKind;
    use crate::creature::genome::vote::{ActionParamField, VoteSink};

    /// A node wired only through a group's members writes the action
    /// channel and reads its ring (T25.F03).
    #[test]
    fn a_member_only_node_is_an_action_writer() {
        use crate::creature::genome::cgp::Shape;
        use crate::creature::genome::vote::DirectedVoteKind;
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
        def.add_group(Shape::Ring { ring_ref: 0 }, DirectedVoteKind::Move, 5);
        let ring = InputReference::World(WorldInputKey::NeighborBarrierRing);
        let (reads, writes, stateful, live) = derive_cgp_annotations(&def, &[ring]);
        assert_eq!(writes, vec![MeshWriteClass::Action]);
        assert_eq!(reads, vec![MeshReadClass::Barrier]);
        assert!(!stateful);
        assert!(live.is_empty());
    }

    #[test]
    fn empty_graph_no_annotations() {
        let def = CgpGraphBackendDef::new_with_fixed_outputs()
            .with_compute_nodes(Vec::new())
            .with_output_sinks(Vec::new());
        let (reads, writes, stateful, live) = derive_cgp_annotations(&def, &[]);
        assert!(reads.is_empty());
        assert!(writes.is_empty());
        assert!(!stateful);
        assert!(live.is_empty());
    }

    #[test]
    fn wired_sink_produces_write_class() {
        let def = CgpGraphBackendDef::new_with_fixed_outputs()
            .with_compute_nodes(vec![ComputeNode::new(
                ComputeNodeKind::Add,
                vec![GraphEdge::new(
                    GraphSource::InputLeaf {
                        ref_idx: 0,
                        sub_idx: 0,
                    },
                    1.0,
                )],
            )])
            .with_output_sinks(vec![
                OutputSink::new(
                    OutputSinkKind::CustomOutput(0),
                    vec![GraphEdge::new(GraphSource::ComputeNode(0), 1.0)],
                ),
                OutputSink::new(OutputSinkKind::RouterGate(0), Vec::new()),
            ]);
        let input_refs = vec![InputReference::World(WorldInputKey::FoodHere {
            type_idx: OrdinaryFoodTypeId::default(),
        })];
        let (reads, writes, _, live) = derive_cgp_annotations(&def, &input_refs);

        assert!(writes.contains(&MeshWriteClass::Payload));
        assert!(!writes.contains(&MeshWriteClass::Route)); // unwired router
        assert!(reads.contains(&MeshReadClass::Food));
        assert!(live.contains(&0));
    }

    #[test]
    fn stateful_node_detected() {
        let def = CgpGraphBackendDef::new_with_fixed_outputs()
            .with_compute_nodes(vec![ComputeNode::new(
                ComputeNodeKind::DecayIntegrator(0.5),
                Vec::new(),
            )])
            .with_output_sinks(vec![OutputSink::new(
                OutputSinkKind::CustomOutput(0),
                vec![GraphEdge::new(GraphSource::ComputeNode(0), 1.0)],
            )]);
        let (_, _, stateful, _) = derive_cgp_annotations(&def, &[]);
        assert!(stateful);
    }

    #[test]
    fn vote_and_parameter_sinks_produce_action_writes() {
        for kind in [
            OutputSinkKind::ActionVote(VoteSink::Eat),
            OutputSinkKind::ActionParam(ActionParamField::ReproduceTransferFraction),
        ] {
            let def = CgpGraphBackendDef::new_with_fixed_outputs()
                .with_compute_nodes(vec![ComputeNode::new(
                    ComputeNodeKind::Constant(1.0),
                    Vec::new(),
                )])
                .with_output_sinks(vec![OutputSink::new(
                    kind,
                    vec![GraphEdge::new(GraphSource::ComputeNode(0), 1.0)],
                )]);
            let (_, writes, _, _) = derive_cgp_annotations(&def, &[]);
            assert_eq!(writes, vec![MeshWriteClass::Action], "{kind:?}");
        }
    }

    #[test]
    fn memory_sink_is_stateful() {
        let def = CgpGraphBackendDef::new_with_fixed_outputs()
            .with_compute_nodes(vec![ComputeNode::new(
                ComputeNodeKind::Constant(1.0),
                Vec::new(),
            )])
            .with_output_sinks(vec![OutputSink::new(
                OutputSinkKind::WriteSlot(0),
                vec![GraphEdge::new(GraphSource::ComputeNode(0), 1.0)],
            )]);
        let (_, writes, stateful, _) = derive_cgp_annotations(&def, &[]);
        assert!(stateful);
        assert!(writes.contains(&MeshWriteClass::Memory));
    }

    #[test]
    fn shared_memory_source_is_stateful() {
        let def = CgpGraphBackendDef::new_with_fixed_outputs()
            .with_compute_nodes(vec![ComputeNode::new(
                ComputeNodeKind::Add,
                vec![GraphEdge::new(
                    GraphSource::SharedMemory {
                        slot: 0,
                        previous: false,
                    },
                    1.0,
                )],
            )])
            .with_output_sinks(vec![OutputSink::new(
                OutputSinkKind::CustomOutput(0),
                vec![GraphEdge::new(GraphSource::ComputeNode(0), 1.0)],
            )]);
        let (_, _, stateful, _) = derive_cgp_annotations(&def, &[]);
        assert!(stateful);
    }

    #[test]
    fn read_class_from_vote_and_terminate_sink_edges() {
        let leaf_edge = |ref_idx| {
            GraphEdge::new(
                GraphSource::InputLeaf {
                    ref_idx,
                    sub_idx: 0,
                },
                1.0,
            )
        };
        let def = CgpGraphBackendDef::new_with_fixed_outputs()
            .with_compute_nodes(Vec::new())
            .with_output_sinks(vec![
                OutputSink::new(
                    OutputSinkKind::ActionVote(VoteSink::Directed(DirectedVoteKind::Move, 0)),
                    vec![leaf_edge(0)],
                ),
                OutputSink::new(
                    OutputSinkKind::ActionVote(VoteSink::Terminate),
                    vec![leaf_edge(1)],
                ),
            ]);
        let input_refs = vec![
            InputReference::World(WorldInputKey::FoodHere {
                type_idx: OrdinaryFoodTypeId::default(),
            }),
            InputReference::World(WorldInputKey::NeighborOccupiedRing),
        ];
        let (reads, writes, _, _) = derive_cgp_annotations(&def, &input_refs);

        assert!(reads.contains(&MeshReadClass::Food));
        assert!(reads.contains(&MeshReadClass::Occupancy));
        assert!(writes.contains(&MeshWriteClass::Action));
    }
}
