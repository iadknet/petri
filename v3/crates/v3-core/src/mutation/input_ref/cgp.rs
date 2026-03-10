//! CGP-specific input_ref lifecycle operations.
//!
//! In the CGP model, input_refs are implicit sources (GraphSource::InputLeaf),
//! NOT physical nodes. Adding an input_ref just pushes to the Vec. Removing
//! or swapping walks all edge containers to update/remove affected edges.

use rand::Rng;

use crate::config::MutationConfig;
use crate::creature::genome::cgp::CgpGraphBackendDef;
use crate::mutation::compound;

/// After adding a new input_ref, optionally create a bootstrap edge from a
/// random edge-bearing surface to the new InputLeaf source.
///
/// Unlike the old model, no leaf nodes are created — InputLeaf sources are
/// implicit. The bootstrap edge gives evolution a starting point.
pub(crate) fn cgp_bootstrap_new_input_ref(
    def: &mut CgpGraphBackendDef,
    ref_idx: u16,
    sub_value_count: u16,
    auto_connect_chance: f32,
    rng: &mut impl Rng,
) {
    use crate::creature::genome::cgp::{GraphEdge, GraphSource};

    if !rng.gen_bool(auto_connect_chance as f64) {
        return;
    }

    // Pick sub_idx 0 for the bootstrap edge (most common scalar case).
    let sub_idx = if sub_value_count > 1 {
        rng.gen_range(0..sub_value_count)
    } else {
        0
    };
    let source = GraphSource::InputLeaf { ref_idx, sub_idx };
    let weight = rng.gen_range(-1.0f32..=1.0);
    let edge = GraphEdge { source, weight };

    // Try to add to a random compute node first, then fall back to sinks.
    if !def.compute_nodes.is_empty() && rng.gen_bool(0.7) {
        let idx = rng.gen_range(0..def.compute_nodes.len());
        def.compute_nodes[idx].inputs.push(edge);
    } else if !def.output_sinks.is_empty() {
        let idx = rng.gen_range(0..def.output_sinks.len());
        def.output_sinks[idx].inputs.push(edge);
    }
}

/// After removing input_ref at `removed_idx`, reindex all InputLeaf edges.
///
/// Delegates to `CgpGraphBackendDef::reindex_input_refs_after_removal` which
/// walks all 5 edge containers: compute inputs, sink inputs, action gate/param
/// inputs, and execute gate inputs.
pub(crate) fn cgp_reindex_after_removal(def: &mut CgpGraphBackendDef, removed_idx: u16) {
    def.reindex_input_refs_after_removal(removed_idx);
}

/// After swapping input_ref at `ref_idx` to a new reference with different
/// sub_value_count, remove edges with out-of-range sub_idx values.
///
/// Delegates to `CgpGraphBackendDef::clamp_sub_idx_after_swap`.
pub(crate) fn cgp_clamp_after_swap(
    def: &mut CgpGraphBackendDef,
    ref_idx: u16,
    new_ref: &crate::contracts::InputReference,
    config: &MutationConfig,
) {
    let new_width = compound::sub_value_count(new_ref, config);
    def.clamp_sub_idx_after_swap(ref_idx, new_width);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::InputReference;
    use crate::creature::genome::cgp::{
        ComputeNode, ComputeNodeKind, ExecuteGate, GraphEdge, GraphSource, OutputSink,
        OutputSinkKind,
    };
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    fn rng(seed: u64) -> SmallRng {
        SmallRng::seed_from_u64(seed)
    }

    fn test_def() -> CgpGraphBackendDef {
        CgpGraphBackendDef {
            compute_nodes: vec![ComputeNode {
                kind: ComputeNodeKind::Add,
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
                            ref_idx: 1,
                            sub_idx: 3,
                        },
                        weight: 0.5,
                    },
                    GraphEdge {
                        source: GraphSource::InputLeaf {
                            ref_idx: 2,
                            sub_idx: 0,
                        },
                        weight: 0.8,
                    },
                ],
                plasticity: None,
            }],
            output_sinks: vec![OutputSink {
                kind: OutputSinkKind::CustomOutput(0),
                inputs: vec![GraphEdge {
                    source: GraphSource::InputLeaf {
                        ref_idx: 1,
                        sub_idx: 0,
                    },
                    weight: 1.0,
                }],
            }],
            action_bank: Vec::new(),
            execute_gate: ExecuteGate { inputs: Vec::new() },
        }
    }

    #[test]
    fn bootstrap_adds_edge_on_connect() {
        let mut def = test_def();
        let mut r = rng(42);
        let initial_edges: usize = def
            .compute_nodes
            .iter()
            .map(|n| n.inputs.len())
            .sum::<usize>()
            + def
                .output_sinks
                .iter()
                .map(|s| s.inputs.len())
                .sum::<usize>();

        // auto_connect_chance = 1.0 guarantees connection
        cgp_bootstrap_new_input_ref(&mut def, 3, 1, 1.0, &mut r);

        let new_edges: usize = def
            .compute_nodes
            .iter()
            .map(|n| n.inputs.len())
            .sum::<usize>()
            + def
                .output_sinks
                .iter()
                .map(|s| s.inputs.len())
                .sum::<usize>();

        assert_eq!(new_edges, initial_edges + 1);
    }

    #[test]
    fn bootstrap_skips_on_zero_chance() {
        let mut def = test_def();
        let mut r = rng(42);

        let before: usize = def
            .compute_nodes
            .iter()
            .map(|n| n.inputs.len())
            .sum::<usize>()
            + def
                .output_sinks
                .iter()
                .map(|s| s.inputs.len())
                .sum::<usize>();

        cgp_bootstrap_new_input_ref(&mut def, 3, 1, 0.0, &mut r);

        let after: usize = def
            .compute_nodes
            .iter()
            .map(|n| n.inputs.len())
            .sum::<usize>()
            + def
                .output_sinks
                .iter()
                .map(|s| s.inputs.len())
                .sum::<usize>();

        assert_eq!(before, after);
    }

    #[test]
    fn reindex_removes_matching_and_decrements_higher() {
        let mut def = test_def();

        // Remove ref_idx 1
        cgp_reindex_after_removal(&mut def, 1);

        // Compute node: ref_idx 0 kept, ref_idx 1 removed, ref_idx 2 -> 1
        assert_eq!(def.compute_nodes[0].inputs.len(), 2);
        assert_eq!(
            def.compute_nodes[0].inputs[0].source,
            GraphSource::InputLeaf {
                ref_idx: 0,
                sub_idx: 0,
            }
        );
        assert_eq!(
            def.compute_nodes[0].inputs[1].source,
            GraphSource::InputLeaf {
                ref_idx: 1,
                sub_idx: 0,
            }
        );

        // Sink edge to InputLeaf(1) was removed
        assert!(def.output_sinks[0].inputs.is_empty());
    }

    #[test]
    fn clamp_after_swap_removes_out_of_range() {
        let mut def = test_def();
        let config = MutationConfig::default();

        // ref_idx 1 has sub_idx 3 in compute_nodes and sub_idx 0 in output_sinks.
        // Swap to a scalar input (width 1) — sub_idx 3 should be removed.
        let new_ref = InputReference::StaticIntrospection(
            crate::contracts::StaticIntrospectionKey::Generation,
        );
        cgp_clamp_after_swap(&mut def, 1, &new_ref, &config);

        // compute_nodes[0]: edge with sub_idx 3 removed, others kept
        assert_eq!(def.compute_nodes[0].inputs.len(), 2);
        // ref_idx 0 sub_idx 0 kept
        assert_eq!(
            def.compute_nodes[0].inputs[0].source,
            GraphSource::InputLeaf {
                ref_idx: 0,
                sub_idx: 0,
            }
        );
        // ref_idx 2 sub_idx 0 kept
        assert_eq!(
            def.compute_nodes[0].inputs[1].source,
            GraphSource::InputLeaf {
                ref_idx: 2,
                sub_idx: 0,
            }
        );

        // Sink: ref_idx 1 sub_idx 0 is still valid (0 < 1)
        assert_eq!(def.output_sinks[0].inputs.len(), 1);
    }
}
