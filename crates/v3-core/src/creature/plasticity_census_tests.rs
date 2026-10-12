use std::collections::BTreeSet;

use super::*;
use crate::config::FounderProfile;
use crate::contracts::NodeId;
use crate::creature::founder::founder_genome;
use crate::creature::genome::analysis::mesh_reachable_nodes;
use crate::creature::genome::cgp::{
    CgpGraphBackendDef, ComputeNode, ComputeNodeKind, GraphEdge, GraphSource,
};
use crate::creature::genome::{NodeGenome, OutcomeChannel, RewardModulationConfig};

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
    ComputeNode {
        kind: ComputeNodeKind::Add,
        inputs: weights
            .iter()
            .enumerate()
            .map(|(slot, &weight)| GraphEdge {
                source: GraphSource::SharedMemory {
                    slot: slot as u8,
                    previous: false,
                },
                weight,
            })
            .collect(),
        plasticity,
    }
}

fn graph_genome(compute_nodes: Vec<ComputeNode>) -> CreatureGenome {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    def.compute_nodes = compute_nodes;
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

/// Learned weights for a one-node genome, one slice per compute node.
fn learned(per_compute_node: &[&[f32]]) -> Vec<Vec<Box<[f32]>>> {
    vec![per_compute_node
        .iter()
        .map(|weights| weights.to_vec().into_boxed_slice())
        .collect()]
}

fn census_of(
    genome: &CreatureGenome,
    reachable: &[usize],
    weights: &LearnedWeights,
) -> PlasticityCensus {
    PlasticityCensus::observe([(genome, reachable, weights)])
}

fn kind(rule: HebbianRule, reward_modulated: bool, lamarckian: bool) -> usize {
    PlasticityKind {
        rule,
        reward_modulated,
        lamarckian,
    }
    .index()
}

#[test]
fn every_kind_sits_at_its_index_under_distinct_keys() {
    let mut keys = BTreeSet::new();
    for (i, kind) in PlasticityKind::ALL.iter().enumerate() {
        assert_eq!(kind.index(), i);
        keys.insert((kind.rule_key(), kind.learning_key(), kind.inheritance_key()));
    }
    assert_eq!(keys.len(), PlasticityKind::ALL.len());
}

#[test]
fn the_founder_carries_no_plasticity() {
    let genome = founder_genome(FounderProfile::default());
    let reachable = mesh_reachable_nodes(&genome);
    let census = census_of(&genome, &reachable, &[]);
    assert_eq!(census.creatures, 1);
    assert_eq!(census.carriers, 0);
    assert!(census
        .kinds
        .iter()
        .all(|row| *row == PlasticEdges::default()));
}

#[test]
fn drift_and_clamp_read_the_learned_weights() {
    let config = plastic(HebbianRule::Classic, false, false);
    let genome = graph_genome(vec![node(Some(config), &[0.5, -0.25, 1.0])]);
    let census = census_of(&genome, &[0], &learned(&[&[0.75, -2.0, 1.0]]));

    let row = census.kinds[kind(HebbianRule::Classic, false, false)];
    assert_eq!(census.carriers, 1);
    assert_eq!(row.edges, 3);
    // 0.25 + 1.75 + 0; every term is exact in binary.
    assert_eq!(row.drift_sum, 2.0);
    assert_eq!(row.at_clamp, 1);
}

#[test]
fn an_uninitialized_edge_reads_its_genome_weight() {
    // Beyond the clamp in the genome, but never learned: no drift, not at clamp.
    let config = plastic(HebbianRule::Classic, false, false);
    let genome = graph_genome(vec![node(Some(config), &[5.0])]);
    for weights in [vec![], learned(&[&[]])] {
        let census = census_of(&genome, &[0], &weights);
        let row = census.kinds[kind(HebbianRule::Classic, false, false)];
        assert_eq!((census.carriers, row.edges, row.at_clamp), (1, 1, 0));
        assert_eq!(row.drift_sum, 0.0);
    }
}

#[test]
fn the_clamp_is_the_runtime_clamp_not_the_genome_value() {
    let mut config = plastic(HebbianRule::Oja, false, false);
    config.weight_clamp = 50.0;
    let genome = graph_genome(vec![node(Some(config), &[0.0, 0.0])]);
    let census = census_of(&genome, &[0], &learned(&[&[-10.0, 9.0]]));
    assert_eq!(
        census.kinds[kind(HebbianRule::Oja, false, false)].at_clamp,
        1
    );
}

#[test]
fn edges_land_in_their_kind_and_plain_nodes_are_not_plastic() {
    let genome = graph_genome(vec![
        node(
            Some(plastic(HebbianRule::Covariance, true, true)),
            &[0.1, 0.2],
        ),
        node(None, &[0.3, 0.4, 0.5]),
        node(Some(plastic(HebbianRule::Oja, false, false)), &[0.6]),
    ]);
    let census = census_of(&genome, &[0], &[]);
    let covariance = kind(HebbianRule::Covariance, true, true);
    let oja = kind(HebbianRule::Oja, false, false);
    assert_eq!(census.carriers, 1);
    for (i, row) in census.kinds.iter().enumerate() {
        let expected = match i {
            _ if i == covariance => 2,
            _ if i == oja => 1,
            _ => 0,
        };
        assert_eq!(row.edges, expected, "{:?}", PlasticityKind::ALL[i]);
    }
}

#[test]
fn unreachable_nodes_are_not_read() {
    let config = plastic(HebbianRule::Classic, false, false);
    let genome = graph_genome(vec![node(Some(config), &[0.5])]);
    let census = census_of(&genome, &[], &learned(&[&[2.0]]));
    assert_eq!((census.creatures, census.carriers), (1, 0));
    assert!(census
        .kinds
        .iter()
        .all(|row| *row == PlasticEdges::default()));
}

#[test]
fn carriers_count_creatures_not_edges() {
    let config = plastic(HebbianRule::AntiHebb, false, true);
    let carrier = graph_genome(vec![node(Some(config), &[0.1, 0.2])]);
    let plain = graph_genome(vec![node(None, &[0.1])]);
    let census = PlasticityCensus::observe([
        (&carrier, [0].as_slice(), [].as_slice()),
        (&carrier, [0].as_slice(), [].as_slice()),
        (&plain, [0].as_slice(), [].as_slice()),
    ]);
    assert_eq!((census.creatures, census.carriers), (3, 2));
    assert_eq!(
        census.kinds[kind(HebbianRule::AntiHebb, false, true)].edges,
        4
    );
}
