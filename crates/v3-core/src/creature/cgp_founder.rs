//! CGP-style founder graph backend construction.
//!
//! Builds the two `CgpGraphBackendDef`s of the founder genome.
//!
//! Node 0: three compute nodes gate energy and age. Six custom outputs carry
//! local food, reproduction eligibility, and cardinal primary food densities.
//! The router gates remain unwired for evolution.
//!
//! Node 1 (T19.F04): the founder's decision as votes. It reads node 0's six
//! slots and the `ActionQueue` compound input and votes `Eat`, the cardinal
//! `Move` and `Reproduce` sinks, and `Terminate`, so the pass loop commits
//! exactly the queue the VM decision node emitted before votes existed.

use crate::creature::genome::cgp::{
    CgpGraphBackendDef, ComputeNode, ComputeNodeKind, GraphEdge, GraphSource, OutputSinkKind,
};
use crate::creature::genome::vote::{VoteKind, VoteSink};

/// The founder's age gate on the unit scale the brain reads `AgeTicks` on:
/// `(min_reproduce_age - 0.5) / age_reference_ticks`. Threshold uses strict
/// `>` semantics; the 0.5 offset makes every integer age at or above
/// `min_reproduce_age_ticks` pass and every one below fail, and the same
/// f32 division on both sides keeps the comparison exact (T17.F02).
pub(crate) fn age_gate_threshold(min_reproduce_age_ticks: u64, age_reference_ticks: u64) -> f32 {
    (min_reproduce_age_ticks as f32 - 0.5) / age_reference_ticks as f32
}

/// Build the founder sensor graph from primary food, energy, age, and occupancy.
///
/// `reproduce_energy_threshold` is a fraction of `max_energy` (the scale
/// `EnergyCurrent` reads on); the age gate is derived from the two tick
/// counts by `age_gate_threshold`.
#[must_use]
pub(crate) fn build_cgp_founder_graph_with_thresholds(
    reproduce_energy_threshold: f32,
    min_reproduce_age_ticks: u64,
    age_reference_ticks: u64,
) -> CgpGraphBackendDef {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();

    // CN0: energy threshold gate.
    def.compute_nodes.push(ComputeNode {
        kind: ComputeNodeKind::Threshold(reproduce_energy_threshold),
        inputs: vec![GraphEdge {
            source: GraphSource::InputLeaf {
                ref_idx: 1, // EnergyCurrent
                sub_idx: 0,
            },
            weight: 1.0,
        }],
        plasticity: None,
    });

    // CN1: age threshold gate.
    def.compute_nodes.push(ComputeNode {
        kind: ComputeNodeKind::Threshold(age_gate_threshold(
            min_reproduce_age_ticks,
            age_reference_ticks,
        )),
        inputs: vec![GraphEdge {
            source: GraphSource::InputLeaf {
                ref_idx: 2, // AgeTicks
                sub_idx: 0,
            },
            weight: 1.0,
        }],
        plasticity: None,
    });

    // CN2: can_reproduce = energy_gate * age_gate.
    def.compute_nodes.push(ComputeNode {
        kind: ComputeNodeKind::Multiply,
        inputs: vec![
            GraphEdge {
                source: GraphSource::ComputeNode(0),
                weight: 1.0,
            },
            GraphEdge {
                source: GraphSource::ComputeNode(1),
                weight: 1.0,
            },
        ],
        plasticity: None,
    });

    // Wire CustomOutput(0) ← primary food on the current cell.
    def.output_sinks[0].inputs.push(GraphEdge {
        source: GraphSource::InputLeaf {
            ref_idx: 0,
            sub_idx: 0,
        },
        weight: 1.0,
    });

    // Wire CustomOutput(1) ← can_reproduce gate (CN2 = energy/age gates)
    def.output_sinks[1].inputs.push(GraphEdge {
        source: GraphSource::ComputeNode(2),
        weight: 1.0,
    });

    // Wire CustomOutput(2..5) ← neighbor food N/E/S/W
    // Direction::to_index(): N=0, E=2, S=4, W=6
    for (slot, sub_idx) in [(2u8, 0u16), (3, 2), (4, 4), (5, 6)] {
        def.output_sinks[slot as usize].inputs.push(GraphEdge {
            source: GraphSource::InputLeaf {
                ref_idx: 3, // NeighborFoodRing
                sub_idx,
            },
            weight: 1.0,
        });
    }

    def
}

/// Node 1's input reference holding the `ActionQueue` compound input; refs
/// `0..6` are node 0's upstream slots.
pub(crate) const FOUNDER_QUEUE_REF: u16 = 6;

/// `ActionQueue` sub-value of queue slot 0's action type.
const QUEUE_SLOT0_TYPE: u16 = 0;

/// The founder's cardinal ring as (upstream slot, `Direction::ALL` index):
/// slots 2..=5 carry N, E, S, W at indices 0, 2, 4, 6, so the lowest-index
/// tie rule within a kind reproduces the first argmax over N, E, S, W.
const CARDINALS: [(u16, u8); 4] = [(2, 0), (3, 2), (4, 4), (5, 6)];

fn edge(source: GraphSource, weight: f32) -> GraphEdge {
    GraphEdge { source, weight }
}

fn leaf(ref_idx: u16, sub_idx: u16) -> GraphSource {
    GraphSource::InputLeaf { ref_idx, sub_idx }
}

fn compute(kind: ComputeNodeKind, inputs: Vec<GraphEdge>) -> ComputeNode {
    ComputeNode {
        kind,
        inputs,
        plasticity: None,
    }
}

/// Build the founder's decision graph (T19.F04 invariant 9). With
/// `f = [food_here > 0]`, `can` the reproduce gate on slot 1, `ring[d]` the
/// cardinal slots, `q = [queue slot 0 holds a Reproduce]`, and the profile's
/// reproduce gate `g` (`can` for V3Alpha1, `can · (1 - f)` for the
/// ForageFirst profiles):
///
/// - `Eat = f - 2g - 2q` (V3Alpha1) or `1 - 2g - 2q` (ForageFirst)
/// - `Move[d] = 0.5 + 0.4·ring[d] - 2g - 2q` on the cardinal sinks
/// - `Reproduce[d] = g·(0.5 + 0.4·ring[d])` on the cardinal sinks
/// - `Terminate = q`, `ActionParam(Reproduce, 1) = transfer_fraction`
#[must_use]
pub(crate) fn build_cgp_founder_decision_graph(
    forage_first: bool,
    transfer_fraction: f32,
) -> CgpGraphBackendDef {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    let bias = GraphSource::ComputeNode(0);
    let food = GraphSource::ComputeNode(1);
    let queued_reproduce = GraphSource::ComputeNode(4);
    let can = leaf(1, 0);
    let queue_type = leaf(FOUNDER_QUEUE_REF, QUEUE_SLOT0_TYPE);

    // CN0: the bias constant; edge weights carry the 0.5 and 1.0 terms.
    def.compute_nodes
        .push(compute(ComputeNodeKind::Constant(1.0), Vec::new()));
    // CN1: f = [food_here > 0].
    def.compute_nodes.push(compute(
        ComputeNodeKind::Threshold(0.0),
        vec![edge(leaf(0, 0), 1.0)],
    ));
    // CN2, CN3: queue slot 0's type above 2.5 and above 3.5; action type 3
    // is `Reproduce` and 4 `StealEnergy`.
    for threshold in [2.5, 3.5] {
        def.compute_nodes.push(compute(
            ComputeNodeKind::Threshold(threshold),
            vec![edge(queue_type, 1.0)],
        ));
    }
    // CN4: q = [type > 2.5] - [type > 3.5], exactly [type == Reproduce].
    def.compute_nodes.push(compute(
        ComputeNodeKind::WeightedSum,
        vec![
            edge(GraphSource::ComputeNode(2), 1.0),
            edge(GraphSource::ComputeNode(3), -1.0),
        ],
    ));
    // The reproduce gate. ForageFirst: CN5 = [can - f > 0.5], which is
    // `can · (1 - f)` on the 0/1 values both carry.
    let gate = if forage_first {
        def.compute_nodes.push(compute(
            ComputeNodeKind::Threshold(0.5),
            vec![edge(can, 1.0), edge(food, -1.0)],
        ));
        GraphSource::ComputeNode(5)
    } else {
        can
    };
    // One `g · ring[d]` product per cardinal direction.
    let mut gated_ring = Vec::with_capacity(CARDINALS.len());
    for (slot, _) in CARDINALS {
        gated_ring.push(GraphSource::ComputeNode(def.compute_nodes.len() as u16));
        def.compute_nodes.push(compute(
            ComputeNodeKind::Multiply,
            vec![edge(gate, 1.0), edge(leaf(slot, 0), 1.0)],
        ));
    }
    // The transfer fraction, bit-identical to the profile constant.
    let fraction = GraphSource::ComputeNode(def.compute_nodes.len() as u16);
    def.compute_nodes.push(compute(
        ComputeNodeKind::Constant(transfer_fraction),
        Vec::new(),
    ));

    let eat_drive = if forage_first {
        edge(bias, 1.0)
    } else {
        edge(food, 1.0)
    };
    wire(
        &mut def,
        OutputSinkKind::ActionVote(VoteSink::Eat),
        vec![eat_drive, edge(gate, -2.0), edge(queued_reproduce, -2.0)],
    );
    for ((slot, direction), product) in CARDINALS.into_iter().zip(gated_ring) {
        wire(
            &mut def,
            OutputSinkKind::ActionVote(VoteSink::Move(direction)),
            vec![
                edge(bias, 0.5),
                edge(leaf(slot, 0), 0.4),
                edge(gate, -2.0),
                edge(queued_reproduce, -2.0),
            ],
        );
        wire(
            &mut def,
            OutputSinkKind::ActionVote(VoteSink::Reproduce(direction)),
            vec![edge(gate, 0.5), edge(product, 0.4)],
        );
    }
    wire(
        &mut def,
        OutputSinkKind::ActionVote(VoteSink::Terminate),
        vec![edge(queued_reproduce, 1.0)],
    );
    wire(
        &mut def,
        OutputSinkKind::ActionParam(VoteKind::Reproduce, 1),
        vec![edge(fraction, 1.0)],
    );
    def
}

/// Append `edges` to the catalog sink of `kind`.
fn wire(def: &mut CgpGraphBackendDef, kind: OutputSinkKind, edges: Vec<GraphEdge>) {
    def.sink_mut(kind)
        .expect("the fixed catalog holds every vote and parameter sink")
        .inputs
        .extend(edges);
}
