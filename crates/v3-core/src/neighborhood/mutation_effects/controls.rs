//! Positive and negative controls of the coverage extension (T11.F26): each
//! is a fixed parent/child pair run in the same code as every sampled pair,
//! on every report.

use crate::contracts::votes::VoteSink;
use crate::contracts::{InputReference, NodeId, WorldInputKey};
use crate::genome::cgp::{
    CgpGraphBackendDef, ComputeNode, ComputeNodeKind, GraphEdge, GraphSource, OutputSinkKind,
};
use crate::genome::{BackendDef, CreatureGenome, NodeGenome};

/// A one-node Graph genome with `compute_nodes` and the given `Eat` vote
/// edges.
pub(super) fn graph_genome(
    input_refs: Vec<InputReference>,
    compute_nodes: Vec<ComputeNode>,
    eat_votes: Vec<GraphEdge>,
) -> CreatureGenome {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs().with_compute_nodes(compute_nodes);
    let eat = def.sink_surface(OutputSinkKind::ActionVote(VoteSink::Eat));
    for edge in eat_votes {
        def.push_edge(eat, edge);
    }
    let id = NodeId::new(0);
    CreatureGenome::new(
        id,
        vec![NodeGenome::new(
            id,
            input_refs,
            BackendDef::Graph(def),
            Vec::new(),
        )],
        crate::creature::founder::default_founder_body(),
    )
}

/// A barrier-dependent action edit: the child votes `Eat` on any neighbor
/// barrier, which `neighborhood-v2` never shows; the parent votes nothing.
pub(super) fn barrier_pair() -> (CreatureGenome, CreatureGenome) {
    let refs = vec![InputReference::World(WorldInputKey::NeighborBarrierRing)];
    let votes = (0..8)
        .map(|direction| {
            GraphEdge::new(
                GraphSource::InputLeaf {
                    ref_idx: 0,
                    sub_idx: direction,
                },
                1.0,
            )
        })
        .collect();
    (
        graph_genome(refs.clone(), Vec::new(), Vec::new()),
        graph_genome(refs, Vec::new(), votes),
    )
}

/// The integrator's rate and threshold: after `k` visits it holds
/// `1 - 0.9^k`, which stays below 0.38 through four visits and crosses it at
/// the fifth, so the child's vote appears only after tick 4 of a sequence.
const INTEGRATOR_RATE: f32 = 0.1;
const INTEGRATOR_THRESHOLD: f32 = 0.38;

/// A slow integrator crossing its threshold after tick 4: the child votes
/// `Eat` from the threshold; the parent computes the same and votes nothing.
pub(super) fn integrator_pair() -> (CreatureGenome, CreatureGenome) {
    let edge = |source: u16| GraphEdge::new(GraphSource::ComputeNode(source), 1.0);
    let nodes = vec![
        ComputeNode::new(ComputeNodeKind::Constant(1.0), Vec::new()),
        ComputeNode::new(
            ComputeNodeKind::DecayIntegrator(INTEGRATOR_RATE),
            vec![edge(0)],
        ),
        ComputeNode::new(
            ComputeNodeKind::Threshold(INTEGRATOR_THRESHOLD),
            vec![edge(1)],
        ),
    ];
    (
        graph_genome(Vec::new(), nodes.clone(), Vec::new()),
        graph_genome(Vec::new(), nodes, vec![edge(2)]),
    )
}
