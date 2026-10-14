//! T25.F02 unit rule: `genome_size()` charges one unit per edge on both graph
//! surfaces, nothing for a wired sink itself, and nine units per ring
//! projection (its record plus its eight implied synapses).
use proptest::prelude::*;

use crate::config::OrdinaryFoodTypeId;
use crate::contracts::{InputReference, NodeId, RouteTarget, WorldInputKey};
use crate::creature::genome::cgp::{
    CgpGraphBackendDef, ComputeNode, ComputeNodeKind, GraphEdge, GraphSource, OutputSink,
    OutputSinkKind, RingProjection,
};
use crate::creature::genome::vote::DirectedVoteKind;
use crate::creature::genome::{BackendDef, CreatureGenome, NodeGenome};

fn leaf(ref_idx: u16) -> GraphSource {
    GraphSource::InputLeaf {
        ref_idx,
        sub_idx: 0,
    }
}

fn graph_genome(def: CgpGraphBackendDef, refs: usize, targets: u8) -> CreatureGenome {
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![
                InputReference::World(WorldInputKey::FoodHere {
                    type_idx: OrdinaryFoodTypeId::default(),
                });
                refs
            ],
            backend_def: BackendDef::Graph(def),
            targets: (0..targets)
                .map(|slot| RouteTarget {
                    target_id: NodeId::new(1),
                    slot,
                    gate_bias: 0.0,
                })
                .collect(),
        }],
    }
}

/// A sink with two edges costs its two edges (3 units under the old rule, 2
/// now); an unwired sink costs nothing under either rule.
#[test]
fn genome_size_charges_no_wired_sink_unit() {
    let unwired = graph_genome(CgpGraphBackendDef::new_with_fixed_outputs(), 1, 0);
    // 1 mesh node + 1 input reference; the fixed outputs are unwired.
    assert_eq!(unwired.genome_size(), 2);

    let wired = graph_genome(
        CgpGraphBackendDef::new_with_fixed_outputs().with_output_sinks(vec![OutputSink::new(
            OutputSinkKind::CustomOutput(0),
            vec![GraphEdge::new(leaf(0), 1.0), GraphEdge::new(leaf(0), -0.5)],
        )]),
        1,
        0,
    );
    assert_eq!(wired.genome_size(), 2 + 2);

    let empty_sink = graph_genome(
        CgpGraphBackendDef::new_with_fixed_outputs().with_output_sinks(vec![OutputSink::new(
            OutputSinkKind::CustomOutput(0),
            Vec::new(),
        )]),
        1,
        0,
    );
    assert_eq!(empty_sink.genome_size(), 2);
}

fn edges_strategy() -> impl Strategy<Value = Vec<GraphEdge>> {
    prop::collection::vec(
        (0u16..4, -4.0f32..4.0).prop_map(|(ref_idx, weight)| GraphEdge::new(leaf(ref_idx), weight)),
        0..4,
    )
}

fn def_strategy() -> impl Strategy<Value = CgpGraphBackendDef> {
    (
        prop::collection::vec(edges_strategy(), 0..5),
        prop::collection::vec((0u8..24, edges_strategy()), 0..6),
        prop::collection::vec((0usize..3, any::<u8>(), -4.0f32..4.0), 0..4),
    )
        .prop_map(|(nodes, sinks, projections)| {
            let mut def = CgpGraphBackendDef::new_with_fixed_outputs()
                .with_compute_nodes(
                    nodes
                        .into_iter()
                        .map(|inputs| ComputeNode::new(ComputeNodeKind::Add, inputs, None))
                        .collect(),
                )
                .with_output_sinks(
                    sinks
                        .into_iter()
                        .map(|(slot, inputs)| {
                            OutputSink::new(OutputSinkKind::CustomOutput(slot), inputs)
                        })
                        .collect(),
                );
            def.projections = projections
                .into_iter()
                .map(|(kind, offset, weight)| {
                    RingProjection::new(0, DirectedVoteKind::ALL[kind], offset, weight)
                })
                .collect();
            def
        })
}

proptest! {
    /// `genome_size` is the unit rule's sum: mesh node + input references +
    /// route targets + compute nodes + edges on both surfaces + 9 per
    /// projection, whatever the generated def.
    #[test]
    fn genome_size_is_the_unit_rule_sum(
        def in def_strategy(),
        refs in 0usize..5,
        targets in 0u8..3,
    ) {
        let compute_nodes = def.compute_nodes().len();
        let compute_edges: usize = def.compute_nodes().iter().map(|n| n.inputs.len()).sum();
        let sink_edges: usize = def.output_sinks().iter().map(|s| s.inputs.len()).sum();
        let projections = def.projections.len();
        let expected = 1 + refs + usize::from(targets) + compute_nodes + compute_edges + sink_edges
            + 9 * projections;
        let genome = graph_genome(def, refs, targets);
        prop_assert_eq!(genome.genome_size() as usize, expected);
    }
}
