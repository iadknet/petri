//! Authored controllers: the canonical founder plus one declared reference
//! on its vote node (node 1) and ordinary Graph structure feeding existing
//! vote or parameter sinks. The lab's comparators compose them; they are
//! instruments, never production genomes, mutation targets or fitness
//! signals.
//!
//! Every weight and threshold is a fixed constant, recorded at T20.F01 and
//! never tuned on assay outcomes.

use v3_core::config::OrdinaryFoodTypeId;
use v3_core::contracts::food_idx;
use v3_core::contracts::votes::DirectedVoteKind;
use v3_core::contracts::votes::VoteSink;
use v3_core::contracts::{InputReference, WorldInputKey};
use v3_core::genome::cgp::{
    CgpGraphBackendDef, ComputeNode, ComputeNodeKind, GraphEdge, GraphSource, OutputSinkKind,
};
use v3_core::genome::{BackendDef, CreatureGenome};

/// The founder's vote node (its decision graph) and the reference index a
/// controller authored on the founder appends to it (the founder's vote
/// node holds seven).
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
/// `A_ring`: the cardinal moves the Ring controller inhibits: N, E, S, W.
pub const RING_MOVES: [u8; 4] = [0, 2, 4, 6];
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
/// The two controller families, one per input shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
    /// `NeighborBarrierRing`, cyclic 8.
    Ring,
    /// `AreaFoodSummary(0)`, heterogeneous 7.
    Vector,
}

impl Family {
    /// The declared reference.
    #[must_use]
    pub fn reference(self) -> InputReference {
        InputReference::World(match self {
            Self::Ring => WorldInputKey::NeighborBarrierRing,
            Self::Vector => WorldInputKey::area_food_summary(OrdinaryFoodTypeId::new(0)),
        })
    }
}

/// Appends authored structure to the vote graph, reading the family's
/// reference at `ref_idx`.
struct Author<'a> {
    graph: &'a mut CgpGraphBackendDef,
    ref_idx: u16,
}

impl Author<'_> {
    fn leaf(&self, sub_idx: u16) -> GraphSource {
        GraphSource::InputLeaf {
            ref_idx: self.ref_idx,
            sub_idx,
        }
    }

    fn node(&mut self, kind: ComputeNodeKind, inputs: &[(GraphSource, f32)]) -> GraphSource {
        let inputs = inputs
            .iter()
            .map(|&(source, weight)| GraphEdge::new(source, weight))
            .collect();
        self.graph
            .append_compute_node(ComputeNode::new(kind, inputs));
        GraphSource::ComputeNode((self.graph.compute_nodes().len() - 1) as u16)
    }

    fn wire(&mut self, sink: OutputSinkKind, source: GraphSource, weight: f32) {
        let edge = GraphEdge::new(source, weight);
        let surface = self
            .graph
            .sink_surface(sink)
            .expect("the fixed catalog holds every vote and parameter sink");
        self.graph.push_edge(surface, edge);
    }
}

fn move_sink(direction: u8) -> OutputSinkKind {
    OutputSinkKind::ActionVote(VoteSink::Directed(DirectedVoteKind::Move, direction))
}

/// Appends `family`'s reference at the vote node's next index and wires its
/// structure reading that index.
///
/// # Panics
///
/// If `genome`'s vote node is not a Graph node.
pub fn append_controller(genome: &mut CreatureGenome, family: Family) {
    let node = &mut genome.nodes[VOTE_NODE];
    let ref_idx = u16::try_from(node.input_refs.len()).expect("a vote node's references fit u16");
    node.input_refs.push(family.reference());
    let BackendDef::Graph(graph) = &mut node.backend_def else {
        panic!("the founder's vote node is a Graph node");
    };
    let mut author = Author { graph, ref_idx };
    match family {
        Family::Ring => {
            for direction in RING_MOVES {
                let barrier = author.leaf(u16::from(direction));
                author.wire(move_sink(direction), barrier, RING_INHIBITION);
            }
        }
        Family::Vector => {
            let nearest_dist = author.leaf(food_idx::NEAREST_DIST as u16);
            let nearest_dx = author.leaf(food_idx::NEAREST_DX as u16);
            let nearest_dy = author.leaf(food_idx::NEAREST_DY as u16);
            let far = author.node(
                ComputeNodeKind::Threshold(VECTOR_FAR_THRESHOLD),
                &[(nearest_dist, 1.0)],
            );
            let gate = author.node(
                ComputeNodeKind::Threshold(VECTOR_GATE_THRESHOLD),
                &[(far, 1.0), (CAN_REPRODUCE, -1.0), (PRIMARY_FOOD_HERE, -1.0)],
            );
            let dx = author.node(ComputeNodeKind::Multiply, &[(gate, 1.0), (nearest_dx, 1.0)]);
            let dy = author.node(ComputeNodeKind::Multiply, &[(gate, 1.0), (nearest_dy, 1.0)]);
            // Direction indices: N 0, E 2, S 4, W 6; `dy` grows southward.
            author.wire(move_sink(2), dx, VECTOR_WEIGHT);
            author.wire(move_sink(6), dx, -VECTOR_WEIGHT);
            author.wire(move_sink(4), dy, VECTOR_WEIGHT);
            author.wire(move_sink(0), dy, -VECTOR_WEIGHT);
        }
    }
}

/// `founder` with family `family`'s controller, its reference at
/// [`AUTHORED_REF`].
///
/// # Panics
///
/// If `founder`'s vote node is not the founder's seven-reference Graph node.
#[must_use]
pub fn controller(founder: &CreatureGenome, family: Family) -> CreatureGenome {
    let mut genome = founder.clone();
    assert_eq!(
        genome.nodes[VOTE_NODE].input_refs.len(),
        usize::from(AUTHORED_REF),
        "the founder's vote node holds seven references"
    );
    append_controller(&mut genome, family);
    genome
}
