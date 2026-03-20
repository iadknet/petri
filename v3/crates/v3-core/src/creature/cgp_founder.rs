//! CGP-style founder graph backend construction.
//!
//! Builds the `CgpGraphBackendDef` used by Node 0 of the founder genome.
//!
//! The CGP founder graph has:
//! - 3 ComputeNodes:
//!   - Threshold(energy_threshold)
//!   - Threshold(min_reproduce_age_ticks - 0.5)
//!   - Multiply(energy_gate, age_gate)
//! - Full fixed output catalog (45 sinks), with 6 CustomOutput sinks wired
//! - RouterOutput unwired (default routing to single target)
//! - Action bank + ExecuteGate start unwired (blank slate for evolution)

use crate::config::MutationConfig;
use crate::creature::genome::cgp::{
    CgpGraphBackendDef, ComputeNode, ComputeNodeKind, GraphEdge, GraphSource,
};

fn age_gate_threshold(min_reproduce_age_ticks: f32) -> f32 {
    // Threshold uses strict `>` semantics; offset by 0.5 so integer age ticks
    // satisfy `age >= min_reproduce_age_ticks`.
    (min_reproduce_age_ticks - 0.5).max(-0.5)
}

/// Build the CGP graph backend for the founder's sensor aggregator node.
///
/// input_refs layout (same as existing founder):
///   0: FoodHere
///   1: EnergyCurrent
///   2: AgeTicks
///   3: NeighborFoodRing
///   4: NeighborOccupiedRing
///
/// Compute nodes:
///   CN0: Threshold(energy_threshold) — gated by EnergyCurrent (input_ref 1)
///   CN1: Threshold(min_reproduce_age_ticks - 0.5) — gated by AgeTicks (input_ref 2)
///   CN2: Multiply(CN0, CN1) — can_reproduce gate (both conditions must be 1.0)
///
/// Wired output sinks:
///   CustomOutput(0) ← InputLeaf(0,0) — food_here passthrough
///   CustomOutput(1) ← ComputeNode(2) — can_reproduce gate
///   CustomOutput(2) ← InputLeaf(3,0) — food_N
///   CustomOutput(3) ← InputLeaf(3,2) — food_E
///   CustomOutput(4) ← InputLeaf(3,4) — food_S
///   CustomOutput(5) ← InputLeaf(3,6) — food_W
#[must_use]
pub(crate) fn build_cgp_founder_graph(
    config: &MutationConfig,
    min_reproduce_age_ticks: f32,
) -> CgpGraphBackendDef {
    build_cgp_founder_graph_with_thresholds(config, 30.0, min_reproduce_age_ticks)
}

/// Build the CGP graph backend for the founder's sensor aggregator node
/// with configurable reproduction thresholds.
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

    // Wire CustomOutput(0) ← food_here signal
    def.output_sinks[0].inputs.push(GraphEdge {
        source: GraphSource::InputLeaf {
            ref_idx: 0,
            sub_idx: 0,
        },
        weight: 1.0,
    });

    // Wire CustomOutput(1) ← can_reproduce gate (CN2 = energy_gate * age_gate)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::creature::genome::cgp::{OutputSinkKind, CUSTOM_OUTPUT_COUNT, FIXED_SINK_COUNT};

    #[test]
    fn cgp_founder_compute_node() {
        let config = MutationConfig::default();
        let def = build_cgp_founder_graph(&config, 10.0);

        assert_eq!(def.compute_nodes.len(), 3);
        assert_eq!(def.compute_nodes[0].kind, ComputeNodeKind::Threshold(30.0));
        assert_eq!(def.compute_nodes[0].inputs.len(), 1);
        assert_eq!(
            def.compute_nodes[0].inputs[0].source,
            GraphSource::InputLeaf {
                ref_idx: 1,
                sub_idx: 0,
            }
        );
        assert_eq!(def.compute_nodes[1].kind, ComputeNodeKind::Threshold(9.5));
        assert_eq!(
            def.compute_nodes[1].inputs[0].source,
            GraphSource::InputLeaf {
                ref_idx: 2,
                sub_idx: 0,
            }
        );
        assert_eq!(def.compute_nodes[2].kind, ComputeNodeKind::Multiply);
    }

    #[test]
    fn cgp_founder_custom_thresholds() {
        let config = MutationConfig::default();
        let def = build_cgp_founder_graph_with_thresholds(&config, 40.0, 12.0);
        assert_eq!(def.compute_nodes.len(), 3);
        assert_eq!(def.compute_nodes[0].kind, ComputeNodeKind::Threshold(40.0));
        assert_eq!(def.compute_nodes[1].kind, ComputeNodeKind::Threshold(11.5));
    }

    #[test]
    fn cgp_founder_sink_catalog() {
        let config = MutationConfig::default();
        let def = build_cgp_founder_graph(&config, 10.0);

        assert_eq!(def.output_sinks.len(), FIXED_SINK_COUNT);
        assert_eq!(def.action_bank.len(), 4);
        assert!(def.execute_gate.inputs.is_empty());
    }

    #[test]
    fn cgp_founder_wired_outputs() {
        let config = MutationConfig::default();
        let def = build_cgp_founder_graph(&config, 10.0);

        // 6 CustomOutput sinks wired (0-5), rest unwired
        for i in 0..6u8 {
            assert!(
                !def.output_sinks[i as usize].inputs.is_empty(),
                "CustomOutput({i}) should be wired"
            );
        }
        for i in 6..CUSTOM_OUTPUT_COUNT {
            assert!(
                def.output_sinks[i as usize].inputs.is_empty(),
                "CustomOutput({i}) should be unwired"
            );
        }

        // RouterOutput (after CustomOutput sinks) unwired
        let router_idx = CUSTOM_OUTPUT_COUNT as usize;
        assert_eq!(
            def.output_sinks[router_idx].kind,
            OutputSinkKind::RouterOutput
        );
        assert!(def.output_sinks[router_idx].inputs.is_empty());
    }

    #[test]
    fn cgp_founder_output_sources_correct() {
        let config = MutationConfig::default();
        let def = build_cgp_founder_graph(&config, 10.0);

        // CustomOutput(0) ← InputLeaf(0,0) food_here
        assert_eq!(
            def.output_sinks[0].inputs[0].source,
            GraphSource::InputLeaf {
                ref_idx: 0,
                sub_idx: 0,
            }
        );

        // CustomOutput(1) ← ComputeNode(2) can_reproduce
        assert_eq!(
            def.output_sinks[1].inputs[0].source,
            GraphSource::ComputeNode(2)
        );

        // CustomOutput(2..5) ← NeighborFoodRing sub_idx N/E/S/W
        let expected_sub_idxs = [0u16, 2, 4, 6];
        for (i, &expected_sub) in expected_sub_idxs.iter().enumerate() {
            assert_eq!(
                def.output_sinks[2 + i].inputs[0].source,
                GraphSource::InputLeaf {
                    ref_idx: 3,
                    sub_idx: expected_sub,
                }
            );
        }
    }

    #[test]
    fn cgp_founder_action_bank_unwired() {
        let config = MutationConfig::default();
        let def = build_cgp_founder_graph(&config, 10.0);

        for slot in &def.action_bank {
            assert!(slot.gate_inputs.is_empty());
            assert!(slot.param_inputs.is_empty());
        }
    }
}
