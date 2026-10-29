//! T25.F02 unit rule: `brain_units()` charges one unit per edge on both graph
//! surfaces, nothing for a wired sink itself, and (T25.F03, T25.F10) one unit
//! per edge-bundle record and nothing for its members, so a gene of either
//! shape costs one unit; `genome_size()` adds the five body genes (T29.F02).
use proptest::prelude::*;

use crate::config::OrdinaryFoodTypeId;
use crate::contracts::votes::DirectedVoteKind;
use crate::contracts::{InputReference, NodeId, RouteTarget, WorldInputKey};
use crate::genome::cgp::{
    detach, CgpGraphBackendDef, ComputeNode, ComputeNodeKind, EdgeSurface, GraphEdge, GraphSource,
    OutputSinkKind,
};
use crate::genome::{BackendDef, CreatureGenome, NodeGenome};

fn leaf(ref_idx: u16) -> GraphSource {
    GraphSource::InputLeaf {
        ref_idx,
        sub_idx: 0,
    }
}

fn graph_genome(def: CgpGraphBackendDef, refs: usize, targets: u8) -> CreatureGenome {
    CreatureGenome::new(
        NodeId::new(0),
        vec![NodeGenome::new(
            NodeId::new(0),
            vec![
                InputReference::World(WorldInputKey::FoodHere {
                    type_idx: OrdinaryFoodTypeId::default(),
                });
                refs
            ],
            BackendDef::Graph(def),
            (0..targets)
                .map(|slot| RouteTarget {
                    target_id: NodeId::new(1),
                    slot,
                    gate_bias: 0.0,
                })
                .collect(),
        )],
        crate::creature::founder::default_founder_body(),
    )
}

/// A sink with two edges costs its two edges (3 units under the old rule, 2
/// now); an unwired sink costs nothing under either rule.
#[test]
fn genome_size_charges_no_wired_sink_unit() {
    let unwired = graph_genome(CgpGraphBackendDef::new_with_fixed_outputs(), 1, 0);
    // 1 mesh node + 1 input reference; the fixed outputs are unwired.
    assert_eq!(unwired.brain_units(), 2);

    let wired = graph_genome(
        CgpGraphBackendDef::new_with_fixed_outputs().with_sink_inputs(
            OutputSinkKind::CustomOutput(0),
            vec![GraphEdge::new(leaf(0), 1.0), GraphEdge::new(leaf(0), -0.5)],
        ),
        1,
        0,
    );
    assert_eq!(wired.brain_units(), 2 + 2);
}

/// T25.F10 invariant 1: a `Ring` gene costs one unit, a pair gene with its
/// fact members one, a hole nothing, and a detached member one as an
/// ordinary edge (its gene's last member's detach removes the record's
/// unit, so the total holds).
#[test]
fn a_gene_costs_one_unit_and_its_members_nothing() {
    let area = InputReference::World(WorldInputKey::area_food_summary(
        OrdinaryFoodTypeId::default(),
    ));
    let ring_ref = InputReference::World(WorldInputKey::NeighborBarrierRing);
    let refs = [ring_ref, area.clone()];
    let size = |def: &CgpGraphBackendDef| {
        let mut genome = graph_genome(def.clone(), 0, 0);
        genome.nodes[0].input_refs = refs.to_vec();
        genome.brain_units()
    };
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    let base = size(&def);
    let ring = def.add_ring_bundle(0, DirectedVoteKind::Move, 1);
    assert_eq!(size(&def), base + 1, "a Ring gene of eight members");
    let layout = crate::contracts::pair_layout(&area, 1);
    let pair = def.add_pair_bundle(1, 1, layout, DirectedVoteKind::StealEnergy, 4);
    assert_eq!(def.members(pair).count(), 12 + 3);
    assert_eq!(size(&def), base + 2, "a pair gene with its facts");
    let hole = def.members(pair).next().unwrap();
    def.remove_edge(hole);
    assert_eq!(size(&def), base + 2, "a hole costs nothing");
    let member = def.members(ring).next().unwrap();
    detach(&mut def, member, &refs);
    assert_eq!(
        size(&def),
        base + 3,
        "a detached member is one ordinary edge"
    );
    let fact = def.members(pair).last().unwrap();
    detach(&mut def, fact, &refs);
    assert_eq!(size(&def), base + 4, "a detached fact member too");

    let mut sole = CgpGraphBackendDef::new_with_fixed_outputs();
    let id = sole.add_ring_bundle(0, DirectedVoteKind::Move, 1);
    while sole.members(id).count() > 1 {
        let next = sole.members(id).next().unwrap();
        sole.remove_edge(next);
    }
    let units = size(&sole);
    let last = sole.members(id).next().unwrap();
    detach(&mut sole, last, &refs);
    assert!(sole.bundles().is_empty());
    assert_eq!(
        size(&sole),
        units,
        "the last member's detach trades the record for an edge"
    );
}

fn edges_strategy() -> impl Strategy<Value = Vec<GraphEdge>> {
    prop::collection::vec(
        (0u16..4, -4.0f32..4.0).prop_map(|(ref_idx, weight)| GraphEdge::new(leaf(ref_idx), weight)),
        0..4,
    )
}

/// A def with compute nodes, independent `CustomOutput` edges and genes
/// (`Ring`, or a `DirectionPair` gene with its fact members), each gene
/// losing the members its mask drops (a gene losing every member goes).
fn def_strategy() -> impl Strategy<Value = CgpGraphBackendDef> {
    (
        prop::collection::vec(edges_strategy(), 0..5),
        prop::collection::vec((0usize..24, edges_strategy()), 0..6),
        prop::collection::vec((0usize..3, 0u8..8, 1u16..=u16::MAX, any::<bool>()), 0..4),
    )
        .prop_map(|(nodes, sinks, bundles)| {
            let mut def = CgpGraphBackendDef::new_with_fixed_outputs().with_compute_nodes(
                nodes
                    .into_iter()
                    .map(|inputs| ComputeNode::new(ComputeNodeKind::Add, inputs))
                    .collect(),
            );
            for (slot, inputs) in sinks {
                for edge in inputs {
                    def.push_edge(EdgeSurface::SinkInput(slot), edge);
                }
            }
            let area = InputReference::World(WorldInputKey::AreaOccupancySummary);
            let layout = crate::contracts::pair_layout(&area, 0);
            for (kind, offset, keep, pair) in bundles {
                let kind = DirectedVoteKind::ALL[kind];
                let id = if pair {
                    def.add_pair_bundle(0, 0, layout, kind, offset)
                } else {
                    def.add_ring_bundle(0, kind, offset)
                };
                let members: Vec<_> = def.members(id).collect();
                for (index, occurrence) in members.into_iter().enumerate().rev() {
                    if keep & (1 << index) == 0 {
                        def.remove_edge(occurrence);
                    }
                }
            }
            def
        })
}

proptest! {
    /// `brain_units` is the unit rule's sum: mesh node + input references +
    /// route targets + compute nodes + compute edges + sink edges with no
    /// membership + 1 per bundle record (T25.F10), whatever the generated
    /// def, and `genome_size` adds one unit per body gene (T29.F02).
    #[test]
    fn genome_size_is_the_unit_rule_sum(
        def in def_strategy(),
        refs in 0usize..5,
        targets in 0u8..3,
    ) {
        let compute_nodes = def.compute_nodes().len();
        let compute_edges: usize = def.compute_nodes().iter().map(|n| n.inputs.len()).sum();
        let sink_edges = def
            .output_sinks()
            .iter()
            .flat_map(|sink| sink.inputs.iter())
            .filter(|edge| edge.membership().is_none())
            .count();
        let bundles = def.bundles().len();
        let expected = 1 + refs + usize::from(targets) + compute_nodes + compute_edges + sink_edges
            + bundles;
        let genome = graph_genome(def, refs, targets);
        prop_assert_eq!(genome.brain_units() as usize, expected);
        prop_assert_eq!(genome.genome_size() as usize, expected + 5);
    }
}
