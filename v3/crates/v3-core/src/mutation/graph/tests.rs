use super::*;
use crate::contracts::NodeId;
use crate::creature::founder::v3alpha1_founder_genome;
use crate::creature::genome::{
    BackendDef, CreatureGenome, GraphBackendDef, GraphInput, GraphInternalNode, NodeGenome,
    VmBackendDef, VmInstruction,
};
use crate::creature::parseability::ParseabilityGate;
use rand::rngs::SmallRng;
use rand::SeedableRng;

mod copy;
mod extensions;
mod operators;

fn rng(seed: u64) -> SmallRng {
    SmallRng::seed_from_u64(seed)
}

/// Founder genome has a Graph node (node 0) with internal nodes including `Threshold(24.0)`.
fn graph_node_internal_count(genome: &CreatureGenome) -> usize {
    genome
        .nodes
        .iter()
        .filter_map(|n| {
            if let BackendDef::Graph(ref g) = n.backend_def {
                Some(g.internal_nodes.len())
            } else {
                None
            }
        })
        .sum()
}

fn graph_only_genome(internal_nodes: Vec<GraphInternalNode>) -> CreatureGenome {
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![],
            backend_def: BackendDef::Graph(GraphBackendDef { internal_nodes }),
            targets: vec![],
        }],
    }
}
