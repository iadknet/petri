//! CGP-style founder graph backend construction.
//!
//! Builds the `CgpGraphBackendDef` used by Node 0 of the founder genome.
//!
//! The CGP founder graph has:
//! - 6 ComputeNodes:
//!   - Threshold(energy_threshold)
//!   - Threshold(min_reproduce_age_ticks - 0.5)
//!   - Multiply(energy_gate, age_gate)
//!   - Threshold(predecessor(reproductive_reserve_cost))
//!   - Multiply(energy_age_gate, reserve_gate)
//!   - Max(primary_food, reproductive_food)
//! - Full fixed output catalog (45 sinks), with 8 CustomOutput sinks wired
//! - RouterGate sinks unwired (default routing to single target)
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

fn reserve_gate_threshold(reproductive_reserve_cost: f32) -> f32 {
    // Threshold uses strict `>` semantics. The immediate representable
    // predecessor makes the graph predicate exactly `reserve >= cost`, even
    // for fractional costs below 0.5 (where a fixed half-unit margin would
    // incorrectly admit an unaffordable reserve).
    if reproductive_reserve_cost.is_finite() && reproductive_reserve_cost > 0.0 {
        f32::from_bits(reproductive_reserve_cost.to_bits() - 1)
    } else {
        f32::NEG_INFINITY
    }
}

/// Build the CGP graph backend for the founder's sensor aggregator node.
///
/// input_refs layout (same as existing founder):
///   0: FoodHere
///   1: EnergyCurrent
///   2: AgeTicks
///   3: NeighborFoodRing
///   4: NeighborOccupiedRing
///   5: FoodHere (reproductive-food type)
///   6: ReproductiveReserveCurrent
///   7: NeighborFoodRing (reproductive-food type)
///
/// Compute nodes:
///   CN0: Threshold(energy_threshold) — gated by EnergyCurrent (input_ref 1)
///   CN1: Threshold(min_reproduce_age_ticks - 0.5) — gated by AgeTicks (input_ref 2)
///   CN2: Multiply(CN0, CN1) — energy/age gate
///   CN3: Threshold(predecessor(reproductive_reserve_cost)) — reserve gate
///   CN4: Multiply(CN2, CN3) — can_reproduce gate
///   CN5: Max(primary_food, reproductive_food) — forage gate
///
/// Wired output sinks:
///   CustomOutput(0) ← ComputeNode(5) — any typed food here
///   CustomOutput(1) ← ComputeNode(4) — can_reproduce gate
///   CustomOutput(2) ← InputLeaf(3,0) — food_N
///   CustomOutput(3) ← InputLeaf(3,2) — food_E
///   CustomOutput(4) ← InputLeaf(3,4) — food_S
///   CustomOutput(5) ← InputLeaf(3,6) — food_W
///   CustomOutput(6) ← InputLeaf(5,0) — reproductive food
///   CustomOutput(7) ← InputLeaf(6,0) — current reproductive reserve
///   CustomOutput(8) ← InputLeaf(1,0) — current energy
///   CustomOutput(9..12) ← InputLeaf(7,N/E/S/W) — reproductive food neighbors
/// Build the founder graph with a runtime-configured reserve threshold.
#[must_use]
pub(crate) fn build_cgp_founder_graph_with_thresholds_and_reserve(
    config: &MutationConfig,
    reproduce_energy_threshold: f32,
    min_reproduce_age_ticks: f32,
    reproductive_reserve_cost: f32,
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

    // CN3: reserve threshold gate.
    def.compute_nodes.push(ComputeNode {
        kind: ComputeNodeKind::Threshold(reserve_gate_threshold(reproductive_reserve_cost)),
        inputs: vec![GraphEdge {
            source: GraphSource::InputLeaf {
                ref_idx: 6, // ReproductiveReserveCurrent
                sub_idx: 0,
            },
            weight: 1.0,
        }],
        plasticity: None,
    });

    // CN4: can_reproduce = energy/age gate * reserve gate.
    def.compute_nodes.push(ComputeNode {
        kind: ComputeNodeKind::Multiply,
        inputs: vec![
            GraphEdge {
                source: GraphSource::ComputeNode(2),
                weight: 1.0,
            },
            GraphEdge {
                source: GraphSource::ComputeNode(3),
                weight: 1.0,
            },
        ],
        plasticity: None,
    });

    // CN5: any food on the current cell. The founder still receives both
    // typed-food inputs below; this aggregate gate lets it forage for either
    // complementary resource when one type is absent from the cell.
    def.compute_nodes.push(ComputeNode {
        kind: ComputeNodeKind::Max,
        inputs: vec![
            GraphEdge {
                source: GraphSource::InputLeaf {
                    ref_idx: 0,
                    sub_idx: 0,
                },
                weight: 1.0,
            },
            GraphEdge {
                source: GraphSource::InputLeaf {
                    ref_idx: 5,
                    sub_idx: 0,
                },
                weight: 1.0,
            },
        ],
        plasticity: None,
    });

    // Wire CustomOutput(0) ← any typed food on the current cell.
    def.output_sinks[0].inputs.push(GraphEdge {
        source: GraphSource::ComputeNode(5),
        weight: 1.0,
    });

    // Wire CustomOutput(1) ← can_reproduce gate (CN4 = energy/age/reserve gates)
    def.output_sinks[1].inputs.push(GraphEdge {
        source: GraphSource::ComputeNode(4),
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

    // CustomOutput(6) carries reproductive-food density and CustomOutput(7)
    // carries the live reserve input for founder policy evolution.
    for (slot, ref_idx) in [(6usize, 5u16), (7, 6)] {
        def.output_sinks[slot].inputs.push(GraphEdge {
            source: GraphSource::InputLeaf {
                ref_idx,
                sub_idx: 0,
            },
            weight: 1.0,
        });
    }

    // Expose the live energy and independent reproductive-food neighborhood to
    // the decision node. These are ordinary genome inputs, not hidden founder
    // state or a server-side policy.
    def.output_sinks[8].inputs.push(GraphEdge {
        source: GraphSource::InputLeaf {
            ref_idx: 1,
            sub_idx: 0,
        },
        weight: 1.0,
    });
    for (slot, sub_idx) in [(9usize, 0u16), (10, 2), (11, 4), (12, 6)] {
        def.output_sinks[slot].inputs.push(GraphEdge {
            source: GraphSource::InputLeaf {
                ref_idx: 7,
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
        let def = build_cgp_founder_graph_with_thresholds_and_reserve(&config, 30.0, 10.0, 4.0);

        assert_eq!(def.compute_nodes.len(), 6);
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
        assert_eq!(
            def.compute_nodes[3].kind,
            ComputeNodeKind::Threshold(f32::from_bits(4.0f32.to_bits() - 1))
        );
        assert_eq!(def.compute_nodes[4].kind, ComputeNodeKind::Multiply);
        assert_eq!(def.compute_nodes[5].kind, ComputeNodeKind::Max);
    }

    #[test]
    fn cgp_founder_custom_thresholds() {
        let config = MutationConfig::default();
        let def = build_cgp_founder_graph_with_thresholds_and_reserve(&config, 40.0, 12.0, 4.0);
        assert_eq!(def.compute_nodes.len(), 6);
        assert_eq!(def.compute_nodes[0].kind, ComputeNodeKind::Threshold(40.0));
        assert_eq!(def.compute_nodes[1].kind, ComputeNodeKind::Threshold(11.5));
    }

    #[test]
    fn reserve_gate_threshold_is_immediate_predecessor_for_fractional_costs() {
        for cost in [0.125_f32, 0.25, 0.75, 4.25] {
            let threshold = reserve_gate_threshold(cost);
            assert!(threshold < cost);
            assert!(f32::from_bits(threshold.to_bits() + 1) == cost);
        }
    }

    #[test]
    fn reserve_gate_rejects_just_below_cost_and_accepts_cost_below_half() {
        let cost = 0.25_f32;
        let threshold = reserve_gate_threshold(cost);
        let mut state = 0.0;
        assert_eq!(
            crate::runtime::cgp::eval::evaluate_compute_kind(
                &ComputeNodeKind::Threshold(threshold),
                &[f32::from_bits(cost.to_bits() - 1)],
                f32::from_bits(cost.to_bits() - 1),
                &mut state,
            ),
            0.0
        );
        assert_eq!(
            crate::runtime::cgp::eval::evaluate_compute_kind(
                &ComputeNodeKind::Threshold(threshold),
                &[cost],
                cost,
                &mut state,
            ),
            1.0
        );
    }

    #[test]
    fn cgp_founder_sink_catalog() {
        let config = MutationConfig::default();
        let def = build_cgp_founder_graph_with_thresholds_and_reserve(&config, 30.0, 10.0, 4.0);

        assert_eq!(def.output_sinks.len(), FIXED_SINK_COUNT);
        assert_eq!(def.action_bank.len(), 4);
        assert!(def.execute_gate.inputs.is_empty());
    }

    #[test]
    fn cgp_founder_wired_outputs() {
        let config = MutationConfig::default();
        let def = build_cgp_founder_graph_with_thresholds_and_reserve(&config, 30.0, 10.0, 4.0);

        // CustomOutput sinks 0-12 are wired to the founder's typed sensor and
        // live-state outputs; the remaining fixed catalog stays unwired.
        for i in 0..13u8 {
            assert!(
                !def.output_sinks[i as usize].inputs.is_empty(),
                "CustomOutput({i}) should be wired"
            );
        }
        for i in 13..CUSTOM_OUTPUT_COUNT {
            assert!(
                def.output_sinks[i as usize].inputs.is_empty(),
                "CustomOutput({i}) should be unwired"
            );
        }

        // RouterGate sinks (after CustomOutput sinks) all unwired
        for i in 0..crate::contracts::MAX_GATE_SLOTS {
            let router_idx = CUSTOM_OUTPUT_COUNT as usize + i;
            assert_eq!(
                def.output_sinks[router_idx].kind,
                OutputSinkKind::RouterGate(i as u8)
            );
            assert!(def.output_sinks[router_idx].inputs.is_empty());
        }
    }

    #[test]
    fn cgp_founder_output_sources_correct() {
        let config = MutationConfig::default();
        let def = build_cgp_founder_graph_with_thresholds_and_reserve(&config, 30.0, 10.0, 4.0);

        // CustomOutput(0) ← ComputeNode(5) any typed food here
        assert_eq!(
            def.output_sinks[0].inputs[0].source,
            GraphSource::ComputeNode(5)
        );

        // CustomOutput(1) ← ComputeNode(4) can_reproduce
        assert_eq!(
            def.output_sinks[1].inputs[0].source,
            GraphSource::ComputeNode(4)
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
        let def = build_cgp_founder_graph_with_thresholds_and_reserve(&config, 30.0, 10.0, 4.0);

        for slot in &def.action_bank {
            assert!(slot.gate_inputs.is_empty());
            assert!(slot.param_inputs.is_empty());
        }
    }
}
