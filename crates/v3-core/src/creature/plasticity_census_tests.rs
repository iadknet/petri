use std::collections::BTreeSet;

use super::*;
use crate::config::FounderProfile;
use crate::contracts::NodeId;
use crate::creature::founder::founder_genome;
use crate::creature::genome::analysis::mesh_reachable_nodes;
use crate::creature::genome::cgp::test_support::plastic_node;
use crate::creature::genome::cgp::{
    validate_graph, CgpGraphBackendDef, ComputeNode, ComputeNodeKind, EdgeOccurrence, GraphEdge,
    GraphInvariantViolation, GraphSource, OutputSinkKind,
};
use crate::creature::genome::vote::VoteSink;
use crate::creature::genome::{NodeGenome, OutcomeChannel, RewardModulationConfig};
use crate::creature::state::GraphRuntimeState;

fn plastic(rule: HebbianRule, reward_modulated: bool, lamarckian: bool) -> PlasticityConfig {
    PlasticityConfig {
        rule,
        learning_rate: 0.1,
        weight_clamp: 2.0,
        lamarckian,
        modulation: reward_modulated.then_some(RewardModulationConfig {
            reward_source: OutcomeChannel::EnergyDelta,
            trace_decay: 0.5,
        }),
    }
}

fn node(plasticity: Option<PlasticityConfig>, weights: &[f32]) -> ComputeNode {
    plastic_node(
        ComputeNodeKind::Add,
        weights
            .iter()
            .enumerate()
            .map(|(slot, &weight)| {
                GraphEdge::new(
                    GraphSource::SharedMemory {
                        slot: slot as u8,
                        previous: false,
                    },
                    weight,
                )
            })
            .collect(),
        plasticity,
    )
}

fn genome_of(def: CgpGraphBackendDef) -> CreatureGenome {
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![],
            backend_def: BackendDef::Graph(def),
            targets: vec![],
        }],
    }
}

fn graph_genome(compute_nodes: Vec<ComputeNode>) -> CreatureGenome {
    genome_of(CgpGraphBackendDef::new_with_fixed_outputs().with_compute_nodes(compute_nodes))
}

/// The learned state of `genome` with its plastic slots of mesh node 0, in
/// canonical order, set to `learned` (the backend visited), or unvisited
/// when `learned` is `None`.
fn learned_state(genome: &CreatureGenome, learned: Option<&[f32]>) -> GraphRuntimeState {
    let mut state = GraphRuntimeState::for_genome(genome, Vec::new());
    for (slot, &value) in learned.unwrap_or(&[]).iter().enumerate() {
        state.learned.set_slot(0, slot, value, 0.0);
    }
    state
}

fn census_of(
    genome: &CreatureGenome,
    reachable: &[usize],
    learned: Option<&[f32]>,
) -> PlasticityCensus {
    let state = learned_state(genome, learned);
    PlasticityCensus::observe([(genome, reachable, state.learned())])
}

const HEBBIAN: usize = 0;
const REWARD: usize = 1;
const COMPUTE: usize = 0;
const SINK: usize = 1;

#[test]
fn every_attribute_key_is_distinct() {
    let rules: BTreeSet<_> = RULES.iter().map(|&rule| rule_key(rule)).collect();
    assert_eq!(rules.len(), RULES.len());
    for keys in [LEARNING_KEYS, SURFACE_KEYS, INHERITANCE_KEYS] {
        assert_ne!(keys[0], keys[1]);
    }
    for (index, &rule) in RULES.iter().enumerate() {
        let kind = PlasticityKind::of(&plastic(rule, false, false));
        assert_eq!(kind.rule_index(), index);
    }
}

#[test]
fn the_founder_carries_no_plasticity() {
    let genome = founder_genome(FounderProfile::default());
    let reachable = mesh_reachable_nodes(&genome);
    let census = census_of(&genome, &reachable, None);
    assert_eq!(census.creatures, 1);
    assert_eq!(census.carriers, 0);
    assert_eq!(
        census,
        PlasticityCensus {
            creatures: 1,
            ..PlasticityCensus::default()
        }
    );
}

#[test]
fn drift_and_clamp_read_the_learned_weights() {
    let config = plastic(HebbianRule::Classic, false, false);
    let genome = graph_genome(vec![node(Some(config), &[0.5, -0.25, 1.0])]);
    let census = census_of(&genome, &[0], Some(&[0.75, -2.0, 1.0]));

    let row = census.edges[HEBBIAN][COMPUTE];
    assert_eq!(census.carriers, 1);
    assert_eq!(row.edges, 3);
    // 0.25 + 1.75 + 0; every term is exact in binary.
    assert_eq!(row.drift_sum, 2.0);
    assert_eq!(row.at_clamp, 1);
}

/// An uninitialized edge (its backend never visited, no seed) is counted
/// but exposes no learned value: no drift and never at its clamp, even
/// when its genomic weight is beyond the clamp.
#[test]
fn an_uninitialized_edge_has_no_drift_and_is_never_at_its_clamp() {
    let config = plastic(HebbianRule::Classic, false, false);
    let genome = graph_genome(vec![node(Some(config), &[5.0])]);
    let census = census_of(&genome, &[0], None);
    let row = census.edges[HEBBIAN][COMPUTE];
    assert_eq!((census.carriers, row.edges, row.at_clamp), (1, 1, 0));
    assert_eq!(row.drift_sum, 0.0);
}

/// The census reads the stored clamp, which I1 keeps in `[0.01, 10]`: a
/// clamp beyond it is rejected at the load boundary instead of being
/// normalized by the reader.
#[test]
fn an_out_of_range_clamp_is_rejected_at_the_load_boundary() {
    let json = r#"{"compute_nodes":[{"kind":"Add","inputs":[{"source":{"ComputeNode":0},"own_weight":0.0,"plasticity":{"rule":"Oja","learning_rate":0.1,"weight_clamp":50.0,"lamarckian":false}}]}],"output_sinks":[]}"#;
    let def: CgpGraphBackendDef = serde_json::from_str(json).unwrap();
    let node = genome_of(def).nodes.remove(0);
    assert_eq!(
        validate_graph(&node),
        Err(GraphInvariantViolation::PlasticityOutOfRange)
    );
}

#[test]
fn edges_land_in_their_pathway_surface_rule_and_inheritance() {
    let mut genome = graph_genome(vec![
        node(
            Some(plastic(HebbianRule::Covariance, true, true)),
            &[0.1, 0.2],
        ),
        node(None, &[0.3, 0.4, 0.5]),
        node(Some(plastic(HebbianRule::Oja, false, false)), &[0.6]),
    ]);
    let BackendDef::Graph(def) = &mut genome.nodes[0].backend_def else {
        unreachable!()
    };
    let vote = def
        .sink_surface(OutputSinkKind::ActionVote(VoteSink::Eat))
        .unwrap();
    def.push_edge(vote, GraphEdge::new(GraphSource::ComputeNode(0), 1.0));
    def.edge_mut(EdgeOccurrence::new(vote, 0))
        .set_plasticity(Some(plastic(HebbianRule::Oja, true, false)));
    let census = census_of(&genome, &[0], None);
    assert_eq!(census.carriers, 1);
    assert_eq!(census.edges[REWARD][COMPUTE].edges, 2);
    assert_eq!(census.edges[HEBBIAN][COMPUTE].edges, 1);
    assert_eq!(census.edges[REWARD][SINK].edges, 1);
    assert_eq!(census.edges[HEBBIAN][SINK].edges, 0);
    assert_eq!(census.edges_by_rule, [0, 2, 0, 2]);
    assert_eq!(census.edges_by_inheritance, [2, 2]);
}

#[test]
fn unreachable_nodes_are_not_read() {
    let config = plastic(HebbianRule::Classic, false, false);
    let genome = graph_genome(vec![node(Some(config), &[0.5])]);
    let census = census_of(&genome, &[], Some(&[2.0]));
    assert_eq!(
        census,
        PlasticityCensus {
            creatures: 1,
            ..PlasticityCensus::default()
        }
    );
}

#[test]
fn carriers_count_creatures_not_edges() {
    let config = plastic(HebbianRule::AntiHebb, false, true);
    let carrier = graph_genome(vec![node(Some(config), &[0.1, 0.2])]);
    let plain = graph_genome(vec![node(None, &[0.1])]);
    let carrier_state = learned_state(&carrier, None);
    let plain_state = learned_state(&plain, None);
    let census = PlasticityCensus::observe([
        (&carrier, [0].as_slice(), carrier_state.learned()),
        (&carrier, [0].as_slice(), carrier_state.learned()),
        (&plain, [0].as_slice(), plain_state.learned()),
    ]);
    assert_eq!((census.creatures, census.carriers), (3, 2));
    assert_eq!(census.edges[HEBBIAN][COMPUTE].edges, 4);
    assert_eq!(census.edges_by_inheritance, [0, 4]);
}
