use super::*;
use crate::config::{FounderProfile, OrdinaryFoodTypeId};
use crate::contracts::{NodeId, RouteTarget};
use crate::creature::founder::founder_genome;
use crate::creature::genome::analysis::mesh_reachable_nodes;
use crate::creature::genome::cgp::{GraphEdge, RingProjection};
use crate::creature::genome::vote::VoteSink;
use crate::creature::genome::NodeGenome;

fn barrier() -> InputReference {
    InputReference::World(WorldInputKey::NeighborBarrierRing)
}

fn census_of(genomes: &[CreatureGenome]) -> RingRuleCensus {
    let reachable: Vec<Vec<usize>> = genomes.iter().map(mesh_reachable_nodes).collect();
    RingRuleCensus::observe(
        genomes
            .iter()
            .zip(&reachable)
            .map(|(genome, reachable)| (genome, reachable.as_slice())),
    )
}

fn leaf(ref_idx: u16, sub_idx: u16, weight: f32) -> GraphEdge {
    GraphEdge {
        source: GraphSource::InputLeaf { ref_idx, sub_idx },
        weight,
    }
}

fn node(id: u32, refs: Vec<InputReference>, def: CgpGraphBackendDef) -> NodeGenome {
    NodeGenome {
        node_id: NodeId::new(id),
        input_refs: refs,
        backend_def: BackendDef::Graph(def),
        targets: vec![],
    }
}

fn wire(def: &mut CgpGraphBackendDef, kind: OutputSinkKind, edge: GraphEdge) {
    def.sink_mut(kind).expect("fixed catalog").inputs.push(edge);
}

fn single(def: CgpGraphBackendDef, refs: Vec<InputReference>) -> CreatureGenome {
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![node(0, refs, def)],
    }
}

#[test]
fn the_founder_carries_one_food_rule_covering_its_four_cardinals() {
    let census = census_of(&[founder_genome(FounderProfile::default())]);
    assert_eq!(census.creatures, 1);
    assert_eq!(
        census.rule_rows(),
        vec!["NeighborFoodRing:0 Move 0 + 0 0 0 1 0 0 0 0".to_string()]
    );
    assert_eq!(census.direction_general, 0);
    assert_eq!(census.direction_general_edges_only, 0);
    assert_eq!(census.projections, 0);
    assert!(census.projection_rows().is_empty());
}

#[test]
fn direct_edges_cover_a_rule_per_direction_and_ignore_tiny_weights() {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    // Avoid the barrier: channel d + 4 into Move(d), negative weight.
    for d in 0..6u8 {
        wire(
            &mut def,
            OutputSinkKind::ActionVote(VoteSink::Move(d)),
            leaf(0, u16::from((d + 4) % 8), -0.5),
        );
    }
    wire(
        &mut def,
        OutputSinkKind::ActionVote(VoteSink::Move(6)),
        leaf(0, 2, 1e-4),
    );
    let census = census_of(&[single(def, vec![barrier()])]);
    assert_eq!(
        census.rule_rows(),
        vec!["NeighborBarrierRing Move 4 - 0 0 0 0 0 1 0 0".to_string()]
    );
    assert_eq!(census.direction_general, 1);
    assert_eq!(census.direction_general_edges_only, 1);
}

#[test]
fn one_bus_hop_carries_the_ring_channel_with_the_product_sign() {
    let food = InputReference::World(WorldInputKey::neighbor_food_ring(OrdinaryFoodTypeId::new(
        1,
    )));
    let mut sender = CgpGraphBackendDef::new_with_fixed_outputs();
    wire(
        &mut sender,
        OutputSinkKind::CustomOutput(4),
        leaf(0, 3, -2.0),
    );
    let mut receiver = CgpGraphBackendDef::new_with_fixed_outputs();
    wire(
        &mut receiver,
        OutputSinkKind::ActionVote(VoteSink::StealEnergy(5)),
        leaf(0, 0, 0.5),
    );
    let mut entry = node(0, vec![food], sender);
    entry.targets.push(RouteTarget {
        target_id: NodeId::new(1),
        slot: 0,
        gate_bias: 1.0,
    });
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![
            entry,
            node(1, vec![InputReference::UpstreamSlot(4)], receiver),
        ],
    };
    let census = census_of(&[genome]);
    assert_eq!(
        census.rule_rows(),
        vec!["NeighborFoodRing:1 StealEnergy 2 - 1 0 0 0 0 0 0 0".to_string()]
    );
}

#[test]
fn a_projection_covers_every_direction_once_it_carries_weight() {
    let mut silent = CgpGraphBackendDef::new_with_fixed_outputs();
    silent
        .projections
        .push(RingProjection::new(0, DirectedVoteKind::Reproduce, 3, 0.0));
    let mut weighted = silent.clone();
    weighted.projections[0].weight = 0.25;
    // An incumbent edge path for the same rule in one direction.
    wire(
        &mut weighted,
        OutputSinkKind::ActionVote(VoteSink::Reproduce(3)),
        leaf(0, 0, 1.0),
    );
    let census = census_of(&[
        single(silent, vec![barrier()]),
        single(weighted, vec![barrier()]),
    ]);
    assert_eq!(census.creatures, 2);
    assert_eq!(census.projection_carriers, 2);
    assert_eq!(census.projections, 2);
    assert_eq!(census.projections_nonzero, 1);
    assert_eq!(census.projections_unreachable, 0);
    assert_eq!(
        census.projection_rows(),
        vec!["NeighborBarrierRing Reproduce 3 2 1 0".to_string()]
    );
    assert_eq!(
        census.rule_rows(),
        vec!["NeighborBarrierRing Reproduce 3 + 0 0 0 0 0 0 0 1".to_string()]
    );
    assert_eq!(census.direction_general, 1);
    assert_eq!(census.direction_general_edges_only, 0);
}

#[test]
fn a_projection_on_an_unreachable_node_is_counted_apart() {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    def.projections
        .push(RingProjection::new(0, DirectedVoteKind::Move, 0, 1.0));
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![
            node(0, vec![], CgpGraphBackendDef::new_with_fixed_outputs()),
            node(1, vec![barrier()], def),
        ],
    };
    let census = census_of(&[genome]);
    assert_eq!(census.projections, 0);
    assert_eq!(census.projection_carriers, 0);
    assert_eq!(census.projections_unreachable, 1);
    assert!(census.rule_rows().is_empty());
}
