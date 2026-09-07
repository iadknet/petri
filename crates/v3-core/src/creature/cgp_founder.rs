//! CGP-style founder graph backend construction.
//!
//! Builds the `CgpGraphBackendDef` used by Node 0 of the founder genome.
//!
//! Three compute nodes gate energy and age. Six custom outputs carry local food,
//! reproduction eligibility, and cardinal primary food densities. The action bank,
//! execute gate, and router gates remain unwired for evolution.

use crate::config::MutationConfig;
use crate::creature::genome::cgp::{
    CgpGraphBackendDef, ComputeNode, ComputeNodeKind, GraphEdge, GraphSource,
};

fn age_gate_threshold(min_reproduce_age_ticks: f32) -> f32 {
    // Threshold uses strict `>` semantics; offset by 0.5 so integer age ticks
    // satisfy `age >= min_reproduce_age_ticks`.
    (min_reproduce_age_ticks - 0.5).max(-0.5)
}

/// Build the founder sensor graph from primary food, energy, age, and occupancy.
#[must_use]
pub(crate) fn build_cgp_founder_graph_with_thresholds(
    config: &MutationConfig,
    reproduce_energy_threshold: f32,
    min_reproduce_age_ticks: f32,
) -> CgpGraphBackendDef {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs(config);

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
        kind: ComputeNodeKind::Threshold(age_gate_threshold(min_reproduce_age_ticks)),
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
