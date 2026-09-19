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

/// The founder's age gate on the unit scale the brain reads `AgeTicks` on:
/// `(min_reproduce_age - 0.5) / age_reference_ticks`. Threshold uses strict
/// `>` semantics; the 0.5 offset makes every integer age at or above
/// `min_reproduce_age_ticks` pass and every one below fail, and the same
/// f32 division on both sides keeps the comparison exact (T17.F02).
pub(crate) fn age_gate_threshold(min_reproduce_age_ticks: u64, age_reference_ticks: u64) -> f32 {
    (min_reproduce_age_ticks as f32 - 0.5).max(-0.5) / age_reference_ticks as f32
}

/// Build the founder sensor graph from primary food, energy, age, and occupancy.
///
/// `reproduce_energy_threshold` is a fraction of `max_energy` (the scale
/// `EnergyCurrent` reads on); the age gate is derived from the two tick
/// counts by `age_gate_threshold`.
#[must_use]
pub(crate) fn build_cgp_founder_graph_with_thresholds(
    config: &MutationConfig,
    reproduce_energy_threshold: f32,
    min_reproduce_age_ticks: u64,
    age_reference_ticks: u64,
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
