//! The authored controllers of `input-opportunity-v1`: the canonical founder
//! plus one declared reference on its vote node (node 1) and ordinary Graph
//! structure feeding existing vote or parameter sinks. `Z_k` is `A_k` with
//! every authored edge weight 0.0: the same references, nodes,
//! `genome_size()` and perception assembly, and no signal.
//!
//! Every weight and threshold is a fixed constant, recorded in the readings
//! before the pilot and never tuned on assay outcomes.

use crate::config::OrdinaryFoodTypeId;
use crate::contracts::{InputReference, WorldInputKey};
use crate::creature::genome::cgp::{
    CgpGraphBackendDef, ComputeNode, ComputeNodeKind, GraphEdge, GraphSource, OutputSinkKind,
};
use crate::creature::genome::vote::{ActionParamField, VoteSink};
use crate::creature::genome::{BackendDef, CreatureGenome};
use crate::sensors::perception::food_idx;

use super::super::input_use::catalog::{addressed, Addressed, Channel};

/// The founder's vote node (its decision graph) and the reference index the
/// controllers append to it (the founder's vote node holds seven).
pub const VOTE_NODE: usize = 1;
pub const AUTHORED_REF: u16 = 7;
/// The founder's `can reproduce` upstream read and its primary-food flag
/// compute node on the vote node (`creature::cgp_founder`).
const CAN_REPRODUCE: GraphSource = GraphSource::InputLeaf {
    ref_idx: 1,
    sub_idx: 0,
};
const PRIMARY_FOOD_HERE: GraphSource = GraphSource::ComputeNode(1);

/// `A_ring`: the weight of `barrier[d]` into `Move(d)` for the four cardinal
/// moves the founder votes (below the founder's largest move vote, 0.9).
pub const RING_INHIBITION: f32 = -2.0;
/// `A_vector`: `nearest_dist` above this is off the four cardinal ring cells
/// the founder reads. At vision radius 5 those lie `1 / (5 sqrt(2)) = 0.141`
/// away and the diagonal ring cells, which do not gate, `0.2`.
pub const VECTOR_FAR_THRESHOLD: f32 = 0.17;
/// `A_vector`: the gate opens above this (far, not able to reproduce, no
/// primary food here).
pub const VECTOR_GATE_THRESHOLD: f32 = 0.5;
/// `A_vector`: the weight of a gated nearest-food component into the move
/// toward it (and its negation into the move away).
pub const VECTOR_WEIGHT: f32 = 1.0;
/// `A_scalar`: the fruit-only flag (fruit here, no primary food here) opens
/// above this; its weight into `Eat` lies above the founder's largest move
/// vote (0.9), and the fruit-here flag's weight into `EatFoodType` names
/// type 1. With grass here the founder already eats once, so only the type
/// changes.
pub const SCALAR_ONLY_THRESHOLD: f32 = 0.5;
pub const SCALAR_EAT_WEIGHT: f32 = 1.0;
pub const SCALAR_TYPE_WEIGHT: f32 = 1.0;

/// The three families the assay competes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Family {
    /// `NeighborBarrierRing`, cyclic 8.
    Ring,
    /// `AreaFoodSummary(0)`, heterogeneous 7.
    Vector,
    /// `FoodHere(1)`, scalar.
    Scalar,
}

impl Family {
    pub const ALL: [Self; 3] = [Self::Ring, Self::Vector, Self::Scalar];

    #[must_use]
    pub const fn index(self) -> u64 {
        self as u64
    }

    #[must_use]
    pub const fn as_key(self) -> &'static str {
        match self {
            Self::Ring => "ring",
            Self::Vector => "vector",
            Self::Scalar => "scalar",
        }
    }

    /// The declared reference.
    #[must_use]
    pub fn reference(self) -> InputReference {
        InputReference::World(match self {
            Self::Ring => WorldInputKey::NeighborBarrierRing,
            Self::Vector => WorldInputKey::area_food_summary(OrdinaryFoodTypeId::new(0)),
            Self::Scalar => WorldInputKey::food_here(OrdinaryFoodTypeId::new(1)),
        })
    }

    /// The sub-indices the authored edges read.
    #[must_use]
    pub fn authored_sub_indices(self) -> &'static [u16] {
        const CARDINALS: [u16; 4] = [0, 2, 4, 6];
        const VECTOR: [u16; 3] = [
            food_idx::NEAREST_DX as u16,
            food_idx::NEAREST_DY as u16,
            food_idx::NEAREST_DIST as u16,
        ];
        match self {
            Self::Ring => &CARDINALS,
            Self::Vector => &VECTOR,
            Self::Scalar => &[0],
        }
    }

    /// The authored channels, in the input-use catalog.
    #[must_use]
    pub fn authored_channels(self) -> Vec<Channel> {
        let reference = self.reference();
        self.authored_sub_indices()
            .iter()
            .map(|&sub_idx| match addressed(&reference, sub_idx) {
                Addressed::Channel(channel) => channel,
                Addressed::OutOfWidth(_) => unreachable!("world keys always address a channel"),
            })
            .collect()
    }
}

fn leaf(sub_idx: u16) -> GraphSource {
    GraphSource::InputLeaf {
        ref_idx: AUTHORED_REF,
        sub_idx,
    }
}

/// Appends authored structure to the vote graph, scaling every authored edge
/// weight by `scale` (1 for `A_k`, 0 for `Z_k`).
struct Author<'a> {
    graph: &'a mut CgpGraphBackendDef,
    scale: f32,
    edges: u32,
    nodes: u32,
}

impl Author<'_> {
    fn edge(&mut self, source: GraphSource, weight: f32) -> GraphEdge {
        self.edges += 1;
        GraphEdge {
            source,
            weight: weight * self.scale,
        }
    }

    fn node(&mut self, kind: ComputeNodeKind, inputs: &[(GraphSource, f32)]) -> GraphSource {
        let inputs = inputs
            .iter()
            .map(|&(source, weight)| self.edge(source, weight))
            .collect();
        self.nodes += 1;
        self.graph.compute_nodes.push(ComputeNode {
            kind,
            inputs,
            plasticity: None,
        });
        GraphSource::ComputeNode((self.graph.compute_nodes.len() - 1) as u16)
    }

    fn wire(&mut self, sink: OutputSinkKind, source: GraphSource, weight: f32) {
        let edge = self.edge(source, weight);
        self.graph
            .sink_mut(sink)
            .expect("the fixed catalog holds every vote and parameter sink")
            .inputs
            .push(edge);
    }
}

fn move_sink(direction: u8) -> OutputSinkKind {
    OutputSinkKind::ActionVote(VoteSink::Move(direction))
}

/// The count of authored structure a controller adds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct AuthoredStructure {
    pub edges: u32,
    pub compute_nodes: u32,
}

/// `founder` with family `family`'s controller; `inert` builds `Z_k`.
///
/// # Panics
///
/// If `founder`'s vote node is not the founder's seven-reference Graph node.
#[must_use]
pub fn controller(
    founder: &CreatureGenome,
    family: Family,
    inert: bool,
) -> (CreatureGenome, AuthoredStructure) {
    let mut genome = founder.clone();
    let node = &mut genome.nodes[VOTE_NODE];
    assert_eq!(
        node.input_refs.len(),
        usize::from(AUTHORED_REF),
        "the founder's vote node holds seven references"
    );
    node.input_refs.push(family.reference());
    let BackendDef::Graph(graph) = &mut node.backend_def else {
        panic!("the founder's vote node is a Graph node");
    };
    let mut author = Author {
        graph,
        scale: if inert { 0.0 } else { 1.0 },
        edges: 0,
        nodes: 0,
    };
    match family {
        Family::Ring => {
            for direction in [0u8, 2, 4, 6] {
                author.wire(
                    move_sink(direction),
                    leaf(u16::from(direction)),
                    RING_INHIBITION,
                );
            }
        }
        Family::Vector => {
            let far = author.node(
                ComputeNodeKind::Threshold(VECTOR_FAR_THRESHOLD),
                &[(leaf(food_idx::NEAREST_DIST as u16), 1.0)],
            );
            let gate = author.node(
                ComputeNodeKind::Threshold(VECTOR_GATE_THRESHOLD),
                &[(far, 1.0), (CAN_REPRODUCE, -1.0), (PRIMARY_FOOD_HERE, -1.0)],
            );
            let dx = author.node(
                ComputeNodeKind::Multiply,
                &[(gate, 1.0), (leaf(food_idx::NEAREST_DX as u16), 1.0)],
            );
            let dy = author.node(
                ComputeNodeKind::Multiply,
                &[(gate, 1.0), (leaf(food_idx::NEAREST_DY as u16), 1.0)],
            );
            // Direction indices: N 0, E 2, S 4, W 6; `dy` grows southward.
            author.wire(move_sink(2), dx, VECTOR_WEIGHT);
            author.wire(move_sink(6), dx, -VECTOR_WEIGHT);
            author.wire(move_sink(4), dy, VECTOR_WEIGHT);
            author.wire(move_sink(0), dy, -VECTOR_WEIGHT);
        }
        Family::Scalar => {
            let fruit = author.node(ComputeNodeKind::Threshold(0.0), &[(leaf(0), 1.0)]);
            // Fruit is the only food here: the founder does not already eat.
            let fruit_only = author.node(
                ComputeNodeKind::Threshold(SCALAR_ONLY_THRESHOLD),
                &[(fruit, 1.0), (PRIMARY_FOOD_HERE, -1.0)],
            );
            author.wire(
                OutputSinkKind::ActionVote(VoteSink::Eat),
                fruit_only,
                SCALAR_EAT_WEIGHT,
            );
            author.wire(
                OutputSinkKind::ActionParam(ActionParamField::EatFoodType),
                fruit,
                SCALAR_TYPE_WEIGHT,
            );
        }
    }
    let structure = AuthoredStructure {
        edges: author.edges,
        compute_nodes: author.nodes,
    };
    (genome, structure)
}
