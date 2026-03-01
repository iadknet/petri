//! Post-convergence effect pass for graph-backend evaluation.
//!
//! Scans converged internal-node outputs and maps output-writer nodes
//! to their deferred effects. Processes in three phases, each scanning
//! internal nodes in index order:
//!
//! 1. **Staged-value writes:** `CustomOutput`, `RouterOutput`, `WriteActionMeta`
//! 2. **Queue mutations:** `PushAction`, `PopAction`
//! 3. **Terminal check:** `ExecuteActionQueue`

use crate::creature::genome::{GraphBackendDef, GraphNodeKind};
use crate::runtime::action_decode::decode_world_action;
use crate::runtime::types::{MeshSideOutputs, NodeResult};

/// Apply post-convergence effects from graph evaluation.
///
/// Three-phase deferred effect pass over converged internal-node outputs:
///
/// **Phase 1 — Staged-value writes** (node-index order):
/// - `CustomOutput(s)` → `output_slots[s] = curr_outputs[i]` (s < 12 guard)
/// - `RouterOutput` → `route_target_idx = curr_outputs[i]`
/// - `WriteActionMeta(slot)` → `action_meta[slot] = curr_outputs[i]` (slot < 8 guard)
///
/// **Phase 2 — Queue mutations** (node-index order):
/// - `PushAction(action_type)` → decode action from meta buffer, push onto queue
/// - `PopAction` → remove most recently queued action
///
/// **Phase 3 — Terminal check:**
/// - `terminal = true` if any node is `ExecuteActionQueue`
///
/// The `action_meta` buffer is scoped per graph-node evaluation; it does not
/// persist across mesh hops. Each graph mesh node starts with a zeroed meta
/// buffer and writes to it independently.
#[inline]
pub(crate) fn apply_graph_effects(
    def: &GraphBackendDef,
    curr_outputs: &[f32],
    upstream_slots: &[f32; 12],
    side_outputs: &mut MeshSideOutputs,
) -> NodeResult {
    let mut output_slots = *upstream_slots;
    let mut route_target_idx: f32 = 0.0;
    let mut action_meta = [0.0f32; 8];

    // Phase 1: staged-value writes
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
            GraphNodeKind::WriteActionMeta(slot) => {
                if (*slot as usize) < 8 {
                    action_meta[*slot as usize] = curr_outputs[i];
                }
            }
            _ => {}
        }
    }

    // Phase 2: queue mutations
    for node in &def.internal_nodes {
        match &node.kind {
            GraphNodeKind::PushAction(action_type) => {
                let action = decode_world_action(*action_type, &action_meta);
                side_outputs.action_queue.push(action);
            }
            GraphNodeKind::PopAction => {
                side_outputs.action_queue.pop();
            }
            _ => {}
        }
    }

    // Phase 3: terminal check
    let terminal = def
        .internal_nodes
        .iter()
        .any(|n| matches!(n.kind, GraphNodeKind::ExecuteActionQueue));

    NodeResult {
        output_slots,
        route_target_idx,
        terminal,
        energy_exhausted: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::WorldAction;
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

    fn side_outputs() -> MeshSideOutputs {
        MeshSideOutputs::new(8)
    }

    // ── Phase 1 tests (existing + new) ──────────────────────────────────────

    #[test]
    fn custom_output_writes_slot() {
        let def = make_def(vec![
            node(GraphNodeKind::Constant(5.0)),
            node_with_input(GraphNodeKind::CustomOutput(3), 0, 1.0),
        ]);
        let curr_outputs = [5.0, 5.0];
        let upstream = [0.0f32; 12];
        let mut so = side_outputs();

        let result = apply_graph_effects(&def, &curr_outputs, &upstream, &mut so);

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
        let mut so = side_outputs();

        let result = apply_graph_effects(&def, &curr_outputs, &upstream, &mut so);

        assert!((result.route_target_idx - 2.5).abs() < 1e-6);
    }

    #[test]
    fn out_of_range_custom_output_ignored() {
        let def = make_def(vec![node(GraphNodeKind::CustomOutput(200))]);
        let curr_outputs = [99.0];
        let upstream = [1.0f32; 12];
        let mut so = side_outputs();

        let result = apply_graph_effects(&def, &curr_outputs, &upstream, &mut so);

        assert_eq!(result.output_slots, upstream);
    }

    #[test]
    fn upstream_slots_passed_through_for_non_output_nodes() {
        let def = make_def(vec![node(GraphNodeKind::Add)]);
        let curr_outputs = [42.0];
        let upstream: [f32; 12] = [
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0,
        ];
        let mut so = side_outputs();

        let result = apply_graph_effects(&def, &curr_outputs, &upstream, &mut so);

        assert_eq!(result.output_slots, upstream);
        assert_eq!(result.route_target_idx, 0.0);
    }

    #[test]
    fn write_action_meta_populates_buffer() {
        // WriteActionMeta(0) with Constant(3.0) → PushAction(2) → Move decoded with meta[0]=3.0
        // Direction index 3 = SE
        let def = make_def(vec![
            node(GraphNodeKind::Constant(3.0)),
            node_with_input(GraphNodeKind::WriteActionMeta(0), 0, 1.0),
            node(GraphNodeKind::PushAction(2)), // action_type 2 = Move
        ]);
        let curr_outputs = [3.0, 3.0, 0.0];
        let upstream = [0.0f32; 12];
        let mut so = side_outputs();

        let _result = apply_graph_effects(&def, &curr_outputs, &upstream, &mut so);

        let actions = so.action_queue.into_actions_or_noop();
        assert_eq!(actions.len(), 1);
        if let WorldAction::Move(dir) = &actions[0] {
            assert_eq!(*dir, crate::contracts::Direction::SE);
        } else {
            panic!("expected Move, got {:?}", actions[0]);
        }
    }

    // ── Phase 2 tests ───────────────────────────────────────────────────────

    #[test]
    fn push_action_decodes_and_queues() {
        let def = make_def(vec![node(GraphNodeKind::PushAction(1))]); // action_type 1 = Eat
        let curr_outputs = [0.0];
        let upstream = [0.0f32; 12];
        let mut so = side_outputs();

        let _result = apply_graph_effects(&def, &curr_outputs, &upstream, &mut so);

        let actions = so.action_queue.into_actions_or_noop();
        assert_eq!(actions, vec![WorldAction::Eat]);
    }

    #[test]
    fn pop_action_removes_from_queue() {
        let def = make_def(vec![
            node(GraphNodeKind::PushAction(1)), // Push Eat
            node(GraphNodeKind::PopAction),     // Pop it
        ]);
        let curr_outputs = [0.0, 0.0];
        let upstream = [0.0f32; 12];
        let mut so = side_outputs();

        let _result = apply_graph_effects(&def, &curr_outputs, &upstream, &mut so);

        let actions = so.action_queue.into_actions_or_noop();
        assert_eq!(actions, vec![WorldAction::NoOp]);
    }

    // ── Phase 3 tests ───────────────────────────────────────────────────────

    #[test]
    fn execute_sets_terminal() {
        let def = make_def(vec![node(GraphNodeKind::ExecuteActionQueue)]);
        let curr_outputs = [0.0];
        let upstream = [0.0f32; 12];
        let mut so = side_outputs();

        let result = apply_graph_effects(&def, &curr_outputs, &upstream, &mut so);

        assert!(result.terminal);
    }

    #[test]
    fn no_execute_means_not_terminal() {
        let def = make_def(vec![node(GraphNodeKind::PushAction(1))]);
        let curr_outputs = [0.0];
        let upstream = [0.0f32; 12];
        let mut so = side_outputs();

        let result = apply_graph_effects(&def, &curr_outputs, &upstream, &mut so);

        assert!(!result.terminal);
    }

    // ── Cross-phase ordering tests ──────────────────────────────────────────

    #[test]
    fn meta_buffer_is_local_zeroed() {
        // PushAction(2) without WriteActionMeta → direction = N (meta[0]=0.0 → index 0 → N)
        let def = make_def(vec![node(GraphNodeKind::PushAction(2))]);
        let curr_outputs = [0.0];
        let upstream = [0.0f32; 12];
        let mut so = side_outputs();

        let _result = apply_graph_effects(&def, &curr_outputs, &upstream, &mut so);

        let actions = so.action_queue.into_actions_or_noop();
        assert_eq!(
            actions,
            vec![WorldAction::Move(crate::contracts::Direction::N)]
        );
    }

    #[test]
    fn phase_ordering_meta_before_push() {
        // WriteActionMeta at index 2 (higher than PushAction at index 1)
        // but Phase 1 runs before Phase 2, so meta IS written before push reads it
        let def = make_def(vec![
            node(GraphNodeKind::Constant(1.0)),
            node(GraphNodeKind::PushAction(2)), // index 1: push Move
            node_with_input(GraphNodeKind::WriteActionMeta(0), 0, 1.0), // index 2: meta[0]=1.0
        ]);
        // curr_outputs: node0=1.0, node1=irrelevant, node2=1.0
        let curr_outputs = [1.0, 0.0, 1.0];
        let upstream = [0.0f32; 12];
        let mut so = side_outputs();

        let _result = apply_graph_effects(&def, &curr_outputs, &upstream, &mut so);

        let actions = so.action_queue.into_actions_or_noop();
        // meta[0]=1.0 → direction index 1 = NE
        if let WorldAction::Move(dir) = &actions[0] {
            assert_eq!(*dir, crate::contracts::Direction::NE);
        } else {
            panic!("expected Move, got {:?}", actions[0]);
        }
    }

    #[test]
    fn multiple_push_actions_in_index_order() {
        let def = make_def(vec![
            node(GraphNodeKind::PushAction(1)), // Eat (index 0)
            node(GraphNodeKind::PushAction(0)), // NoOp (index 1)
        ]);
        let curr_outputs = [0.0, 0.0];
        let upstream = [0.0f32; 12];
        let mut so = side_outputs();

        let _result = apply_graph_effects(&def, &curr_outputs, &upstream, &mut so);

        let actions = so.action_queue.into_actions_or_noop();
        assert_eq!(actions, vec![WorldAction::Eat, WorldAction::NoOp]);
    }

    #[test]
    fn out_of_range_meta_slot_ignored() {
        let def = make_def(vec![node(GraphNodeKind::WriteActionMeta(200))]);
        let curr_outputs = [99.0];
        let upstream = [0.0f32; 12];
        let mut so = side_outputs();

        let result = apply_graph_effects(&def, &curr_outputs, &upstream, &mut so);

        // Should not panic or write anywhere
        assert!(!result.terminal);
    }
}
