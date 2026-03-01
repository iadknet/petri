//! Post-convergence effect pass for graph-backend evaluation.
//!
//! Scans converged internal-node outputs and maps output-writer nodes
//! to their deferred effects. Processes in internal-node-index order.

use crate::creature::genome::{GraphBackendDef, GraphNodeKind};
use crate::runtime::types::NodeResult;

/// Apply post-convergence effects from graph evaluation.
///
/// Scans converged internal-node outputs and maps output-writer nodes
/// to their deferred effects. Processes in internal-node-index order.
///
/// - `CustomOutput(s)` writes `curr_outputs[i]` to `output_slots[s]` (s < 12 guard).
/// - `RouterOutput` writes `curr_outputs[i]` to `route_target_idx`.
#[inline]
pub(crate) fn apply_graph_effects(
    def: &GraphBackendDef,
    curr_outputs: &[f32],
    upstream_slots: &[f32; 12],
) -> NodeResult {
    let mut output_slots = *upstream_slots;
    let mut route_target_idx: f32 = 0.0;

    for (i, node) in def.internal_nodes.iter().enumerate() {
        match &node.kind {
            GraphNodeKind::CustomOutput(s) => {
                if (*s as usize) < 12 {
                    output_slots[*s as usize] = curr_outputs[i];
                }
            }
            GraphNodeKind::RouterOutput => {
                route_target_idx = curr_outputs[i];
            }
            _ => {}
        }
    }

    NodeResult {
        output_slots,
        route_target_idx,
        terminal: false,
        energy_exhausted: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::creature::genome::{GraphBackendDef, GraphInput, GraphInternalNode, GraphNodeKind};

    fn make_def(nodes: Vec<GraphInternalNode>) -> GraphBackendDef {
        GraphBackendDef {
            internal_nodes: nodes,
        }
    }

    fn node(kind: GraphNodeKind) -> GraphInternalNode {
        GraphInternalNode {
            kind,
            inputs: vec![],
            hebbian: None,
        }
    }

    fn node_with_input(kind: GraphNodeKind, source_idx: u16, weight: f32) -> GraphInternalNode {
        GraphInternalNode {
            kind,
            inputs: vec![GraphInput { source_idx, weight }],
            hebbian: None,
        }
    }

    #[test]
    fn custom_output_writes_slot() {
        let def = make_def(vec![
            node(GraphNodeKind::Constant(5.0)),
            node_with_input(GraphNodeKind::CustomOutput(3), 0, 1.0),
        ]);
        let curr_outputs = [5.0, 5.0];
        let upstream = [0.0f32; 12];

        let result = apply_graph_effects(&def, &curr_outputs, &upstream);

        assert!((result.output_slots[3] - 5.0).abs() < 1e-6);
        assert_eq!(result.output_slots[0], 0.0);
        assert!(!result.terminal);
        assert!(!result.energy_exhausted);
    }

    #[test]
    fn router_output_writes_route() {
        let def = make_def(vec![
            node(GraphNodeKind::Constant(2.5)),
            node_with_input(GraphNodeKind::RouterOutput, 0, 1.0),
        ]);
        let curr_outputs = [2.5, 2.5];
        let upstream = [0.0f32; 12];

        let result = apply_graph_effects(&def, &curr_outputs, &upstream);

        assert!((result.route_target_idx - 2.5).abs() < 1e-6);
    }

    #[test]
    fn out_of_range_custom_output_ignored() {
        let def = make_def(vec![node(GraphNodeKind::CustomOutput(200))]);
        let curr_outputs = [99.0];
        let upstream = [1.0f32; 12];

        let result = apply_graph_effects(&def, &curr_outputs, &upstream);

        // All slots should remain at upstream value
        assert_eq!(result.output_slots, upstream);
    }

    #[test]
    fn upstream_slots_passed_through_for_non_output_nodes() {
        let def = make_def(vec![node(GraphNodeKind::Add)]);
        let curr_outputs = [42.0];
        let upstream: [f32; 12] = [
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0,
        ];

        let result = apply_graph_effects(&def, &curr_outputs, &upstream);

        assert_eq!(result.output_slots, upstream);
        assert_eq!(result.route_target_idx, 0.0);
    }
}
