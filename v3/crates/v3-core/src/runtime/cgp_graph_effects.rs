//! Post-convergence effect pass for CGP-style graph backend evaluation.
//!
//! Three-phase deferred effect pass over converged compute-node outputs:
//!
//! 1. **Value outputs:** iterate `output_sinks` — write to output_slots,
//!    route_target_idx, shared_memory.
//! 2. **Action bank scan:** iterate `action_bank` — fire gated slots, decode
//!    world actions from param edges.
//! 3. **Execute gate:** check if execute gate fires AND queue is non-empty.

use crate::contracts::{Direction, InputReference, WorldAction};
use crate::creature::genome::cgp::{
    ActionSlotBehavior, CgpGraphBackendDef, GraphEdge, OutputSinkKind, WorldActionKind,
};
use crate::runtime::cgp_graph::resolve_source;
use crate::runtime::inputs::ResolveCtx;
use crate::runtime::types::{sanitize_f32, MeshSideOutputs, NodeResult, OUTPUT_SLOT_COUNT};

// ─── Edge wsum helper ───────────────────────────────────────────────────────

/// Compute the weighted sum of resolved edge sources.
///
/// Reuses the caller-provided `buf` to avoid per-call allocation.
#[inline]
#[allow(clippy::too_many_arguments)]
fn edges_wsum(
    edges: &[GraphEdge],
    compute_count: usize,
    curr_outputs: &[f32],
    input_refs: &[InputReference],
    resolve_ctx: &ResolveCtx<'_>,
    shared_memory: &[f32; 16],
    prev_shared_memory: &[f32; 16],
    buf: &mut Vec<f32>,
) -> f32 {
    buf.clear();
    buf.extend(edges.iter().map(|edge| {
        let v = resolve_source(
            &edge.source,
            // Post-convergence: all compute nodes are "evaluated" — use
            // compute_count as current_idx so every ComputeNode source reads
            // from curr_outputs.
            compute_count,
            compute_count,
            // prev_outputs is unused post-convergence (current_idx == node_count
            // means every source index < current_idx), but we must pass a valid
            // slice. curr_outputs serves as both.
            curr_outputs,
            curr_outputs,
            input_refs,
            resolve_ctx,
            shared_memory,
            prev_shared_memory,
        );
        v * edge.weight
    }));
    buf.iter().copied().sum()
}

// ─── Action decoding ────────────────────────────────────────────────────────

/// Decode a direction from a raw f32 param value.
/// Rounds to nearest integer, clamps to [0, 7], indexes into Direction::ALL.
#[inline]
fn decode_direction(raw: f32) -> Direction {
    let clamped = if raw.is_nan() {
        0.0
    } else {
        raw.round().clamp(0.0, 7.0)
    };
    Direction::ALL[clamped as usize]
}

/// Clamp to non-negative finite: NaN/Inf/negative → 0.0.
#[inline]
fn clamp_non_negative_finite(v: f32) -> f32 {
    if v.is_finite() && v >= 0.0 {
        v
    } else {
        0.0
    }
}

/// Decode a `WorldAction` from a `WorldActionKind` and param wsum values.
///
/// `param_values[0]` → direction (for Move, Reproduce, StealEnergy).
/// `param_values[1]` → energy/amount (for Reproduce, StealEnergy).
#[inline]
fn decode_action_from_kind(kind: WorldActionKind, param_values: &[f32]) -> WorldAction {
    let p0 = param_values.first().copied().unwrap_or(0.0);
    let p1 = param_values.get(1).copied().unwrap_or(0.0);
    match kind {
        WorldActionKind::NoOp => WorldAction::NoOp,
        WorldActionKind::Eat => WorldAction::Eat,
        WorldActionKind::Move => WorldAction::Move(decode_direction(p0)),
        WorldActionKind::Reproduce => WorldAction::Reproduce {
            direction: decode_direction(p0),
            energy_transfer: clamp_non_negative_finite(p1),
        },
        WorldActionKind::StealEnergy => WorldAction::StealEnergy {
            direction: decode_direction(p0),
            amount: clamp_non_negative_finite(p1),
        },
    }
}

// ─── Main effects pass ──────────────────────────────────────────────────────

/// Apply post-convergence effects from CGP graph evaluation.
///
/// Three-phase deferred effect pass:
///
/// **Phase 1 — Value outputs** (output_sinks order):
/// - Inert sinks (empty inputs) are skipped — they don't write.
/// - `CustomOutput(s)` → `output_slots[s] = wsum` (s < 12 guard)
/// - `RouterOutput` → `route_target_idx = clamp01(wsum) * target_count` (normalized binning)
/// - `WriteSlot(s)` → `shared_memory[s % 16] = sanitize_f32(wsum)`
/// - `ClearSlot(s)` → `shared_memory[s % 16] = 0.0` (only when wired)
///
/// **Phase 2 — Action bank scan** (action_bank order):
/// - Each slot with `gate wsum > 0.0` fires.
/// - `Pop` → remove last queued action.
/// - `Emit(kind)` → decode world action from kind + param edges, push to queue.
///
/// **Phase 3 — Execute gate:**
/// - `terminal = true` if `execute_gate wsum > 0.0` AND queue is non-empty.
#[inline]
#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_cgp_graph_effects(
    def: &CgpGraphBackendDef,
    curr_outputs: &[f32],
    input_refs: &[InputReference],
    resolve_ctx: &ResolveCtx<'_>,
    upstream_slots: &[f32; 12],
    side_outputs: &mut MeshSideOutputs,
    shared_memory: &mut [f32; 16],
    prev_shared_memory: &[f32; 16],
) -> NodeResult {
    let mut output_slots = *upstream_slots;
    let mut route_target_idx: f32 = 0.0;
    let compute_count = def.compute_nodes.len();
    let mut buf = Vec::with_capacity(8);

    // Phase 1: value outputs
    for sink in &def.output_sinks {
        // Inert sinks don't write
        if sink.inputs.is_empty() {
            continue;
        }

        let wsum = edges_wsum(
            &sink.inputs,
            compute_count,
            curr_outputs,
            input_refs,
            resolve_ctx,
            shared_memory,
            prev_shared_memory,
            &mut buf,
        );

        match sink.kind {
            OutputSinkKind::CustomOutput(s) => {
                if (s as usize) < OUTPUT_SLOT_COUNT {
                    output_slots[s as usize] = sanitize_f32(wsum);
                }
            }
            OutputSinkKind::RouterOutput => {
                route_target_idx = sanitize_f32(wsum);
            }
            OutputSinkKind::WriteSlot(s) => {
                shared_memory[(s as usize) % 16] = sanitize_f32(wsum);
            }
            OutputSinkKind::ClearSlot(s) => {
                shared_memory[(s as usize) % 16] = 0.0;
            }
        }
    }

    // Phase 2: action bank scan
    for slot in &def.action_bank {
        // Empty gate → wsum = 0.0 → doesn't fire
        if slot.gate_inputs.is_empty() {
            continue;
        }

        let gate_wsum = edges_wsum(
            &slot.gate_inputs,
            compute_count,
            curr_outputs,
            input_refs,
            resolve_ctx,
            shared_memory,
            prev_shared_memory,
            &mut buf,
        );

        if gate_wsum > 0.0 {
            match slot.behavior {
                ActionSlotBehavior::Pop => {
                    side_outputs.action_queue.pop();
                }
                ActionSlotBehavior::Emit(kind) => {
                    // First two param edges map to param[0] (direction) and
                    // param[1] (energy/amount). Each is resolved independently.
                    let mut param_values = [0.0f32; 2];
                    for (i, edge) in slot.param_inputs.iter().enumerate() {
                        if i >= 2 {
                            break;
                        }
                        let v = resolve_source(
                            &edge.source,
                            compute_count,
                            compute_count,
                            curr_outputs,
                            curr_outputs,
                            input_refs,
                            resolve_ctx,
                            shared_memory,
                            prev_shared_memory,
                        );
                        param_values[i] = v * edge.weight;
                    }

                    let action = decode_action_from_kind(kind, &param_values);
                    side_outputs.action_queue.push(action);
                }
            }
        }
    }

    // Phase 3: execute gate
    let terminal = if def.execute_gate.inputs.is_empty() {
        false
    } else {
        let gate_wsum = edges_wsum(
            &def.execute_gate.inputs,
            compute_count,
            curr_outputs,
            input_refs,
            resolve_ctx,
            shared_memory,
            prev_shared_memory,
            &mut buf,
        );
        gate_wsum > 0.0 && !side_outputs.action_queue.is_empty()
    };

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
    use crate::creature::genome::cgp::{
        ActionSlot, ActionSlotBehavior, CgpGraphBackendDef, ComputeNode, ComputeNodeKind,
        ExecuteGate, GraphEdge, GraphSource, OutputSink, OutputSinkKind, WorldActionKind,
    };
    use crate::runtime::inputs::ResolveCtx;
    use crate::runtime::types::MeshSideOutputs;

    fn empty_def() -> CgpGraphBackendDef {
        CgpGraphBackendDef {
            compute_nodes: Vec::new(),
            output_sinks: Vec::new(),
            action_bank: Vec::new(),
            execute_gate: ExecuteGate { inputs: Vec::new() },
        }
    }

    fn side_outputs() -> MeshSideOutputs {
        MeshSideOutputs::new(8)
    }

    fn apply(
        def: &CgpGraphBackendDef,
        curr_outputs: &[f32],
        upstream: &[f32; 12],
        so: &mut MeshSideOutputs,
        shared_mem: &mut [f32; 16],
    ) -> NodeResult {
        let prev_shared = *shared_mem;
        apply_cgp_graph_effects(
            def,
            curr_outputs,
            &[],
            &ResolveCtx::dummy(),
            upstream,
            so,
            shared_mem,
            &prev_shared,
        )
    }

    // ── Phase 1: value outputs ──────────────────────────────────────────────

    #[test]
    fn inert_sink_does_not_write() {
        let mut def = empty_def();
        def.output_sinks.push(OutputSink {
            kind: OutputSinkKind::CustomOutput(0),
            inputs: Vec::new(), // inert
        });
        let mut so = side_outputs();
        let upstream = [7.0f32; 12];
        let result = apply(&def, &[], &upstream, &mut so, &mut [0.0; 16]);
        // Should preserve upstream value, not overwrite with 0.0
        assert_eq!(result.output_slots[0], 7.0);
    }

    #[test]
    fn custom_output_writes_wsum() {
        let mut def = empty_def();
        // One compute node: Constant(5.0)
        def.compute_nodes.push(ComputeNode {
            kind: ComputeNodeKind::Constant(5.0),
            inputs: Vec::new(),
            plasticity: None,
        });
        // Sink: CustomOutput(3) sourced from compute node 0 with weight 1.0
        def.output_sinks.push(OutputSink {
            kind: OutputSinkKind::CustomOutput(3),
            inputs: vec![GraphEdge {
                source: GraphSource::ComputeNode(0),
                weight: 1.0,
            }],
        });

        let curr_outputs = [5.0]; // compute node 0 converged to 5.0
        let upstream = [0.0f32; 12];
        let mut so = side_outputs();
        let result = apply(&def, &curr_outputs, &upstream, &mut so, &mut [0.0; 16]);

        assert!((result.output_slots[3] - 5.0).abs() < 1e-6);
        assert_eq!(result.output_slots[0], 0.0);
    }

    #[test]
    fn router_output_writes_route_value() {
        let mut def = empty_def();
        def.compute_nodes.push(ComputeNode {
            kind: ComputeNodeKind::Constant(0.75),
            inputs: Vec::new(),
            plasticity: None,
        });
        def.output_sinks.push(OutputSink {
            kind: OutputSinkKind::RouterOutput,
            inputs: vec![GraphEdge {
                source: GraphSource::ComputeNode(0),
                weight: 1.0,
            }],
        });

        let curr_outputs = [0.75];
        let upstream = [0.0f32; 12];
        let mut so = side_outputs();
        let result = apply(&def, &curr_outputs, &upstream, &mut so, &mut [0.0; 16]);

        assert!((result.route_target_idx - 0.75).abs() < 1e-6);
    }

    #[test]
    fn write_slot_commits_to_shared_memory() {
        let mut def = empty_def();
        def.compute_nodes.push(ComputeNode {
            kind: ComputeNodeKind::Constant(4.5),
            inputs: Vec::new(),
            plasticity: None,
        });
        def.output_sinks.push(OutputSink {
            kind: OutputSinkKind::WriteSlot(2),
            inputs: vec![GraphEdge {
                source: GraphSource::ComputeNode(0),
                weight: 1.0,
            }],
        });

        let curr_outputs = [4.5];
        let upstream = [0.0f32; 12];
        let mut so = side_outputs();
        let mut shared_mem = [0.0f32; 16];
        let _ = apply(&def, &curr_outputs, &upstream, &mut so, &mut shared_mem);

        assert!((shared_mem[2] - 4.5).abs() < 1e-6);
    }

    #[test]
    fn clear_slot_zeros_shared_memory() {
        let mut def = empty_def();
        def.compute_nodes.push(ComputeNode {
            kind: ComputeNodeKind::Constant(1.0),
            inputs: Vec::new(),
            plasticity: None,
        });
        // ClearSlot with a wired edge — should clear
        def.output_sinks.push(OutputSink {
            kind: OutputSinkKind::ClearSlot(5),
            inputs: vec![GraphEdge {
                source: GraphSource::ComputeNode(0),
                weight: 1.0,
            }],
        });

        let curr_outputs = [1.0];
        let upstream = [0.0f32; 12];
        let mut so = side_outputs();
        let mut shared_mem = [0.0f32; 16];
        shared_mem[5] = 99.0;
        let _ = apply(&def, &curr_outputs, &upstream, &mut so, &mut shared_mem);

        assert_eq!(shared_mem[5], 0.0);
    }

    #[test]
    fn write_slot_sanitizes_nan() {
        let mut def = empty_def();
        def.compute_nodes.push(ComputeNode {
            kind: ComputeNodeKind::Constant(0.0), // won't actually matter
            inputs: Vec::new(),
            plasticity: None,
        });
        def.output_sinks.push(OutputSink {
            kind: OutputSinkKind::WriteSlot(0),
            inputs: vec![GraphEdge {
                source: GraphSource::ComputeNode(0),
                weight: f32::NAN, // weight is NaN → wsum is NaN → sanitize to 0.0
            }],
        });

        let curr_outputs = [1.0];
        let upstream = [0.0f32; 12];
        let mut so = side_outputs();
        let mut shared_mem = [0.0f32; 16];
        shared_mem[0] = 1.0;
        let _ = apply(&def, &curr_outputs, &upstream, &mut so, &mut shared_mem);

        assert_eq!(shared_mem[0], 0.0);
    }

    #[test]
    fn out_of_range_custom_output_ignored() {
        let mut def = empty_def();
        def.compute_nodes.push(ComputeNode {
            kind: ComputeNodeKind::Constant(99.0),
            inputs: Vec::new(),
            plasticity: None,
        });
        def.output_sinks.push(OutputSink {
            kind: OutputSinkKind::CustomOutput(200),
            inputs: vec![GraphEdge {
                source: GraphSource::ComputeNode(0),
                weight: 1.0,
            }],
        });

        let curr_outputs = [99.0];
        let upstream = [1.0f32; 12];
        let mut so = side_outputs();
        let result = apply(&def, &curr_outputs, &upstream, &mut so, &mut [0.0; 16]);

        assert_eq!(result.output_slots, upstream);
    }

    // ── Phase 2: action bank ────────────────────────────────────────────────

    #[test]
    fn action_slot_fires_when_gate_positive() {
        let mut def = empty_def();
        def.compute_nodes.push(ComputeNode {
            kind: ComputeNodeKind::Constant(1.0), // gate source
            inputs: Vec::new(),
            plasticity: None,
        });
        def.action_bank.push(ActionSlot {
            behavior: ActionSlotBehavior::Emit(WorldActionKind::Eat),
            gate_inputs: vec![GraphEdge {
                source: GraphSource::ComputeNode(0),
                weight: 1.0,
            }],
            param_inputs: Vec::new(),
        });

        let curr_outputs = [1.0];
        let upstream = [0.0f32; 12];
        let mut so = side_outputs();
        let _ = apply(&def, &curr_outputs, &upstream, &mut so, &mut [0.0; 16]);

        let actions = so.action_queue.into_actions_or_noop();
        assert_eq!(actions, vec![WorldAction::Eat]);
    }

    #[test]
    fn action_slot_does_not_fire_when_gate_zero() {
        let mut def = empty_def();
        def.compute_nodes.push(ComputeNode {
            kind: ComputeNodeKind::Constant(0.0),
            inputs: Vec::new(),
            plasticity: None,
        });
        def.action_bank.push(ActionSlot {
            behavior: ActionSlotBehavior::Emit(WorldActionKind::Eat),
            gate_inputs: vec![GraphEdge {
                source: GraphSource::ComputeNode(0),
                weight: 1.0,
            }],
            param_inputs: Vec::new(),
        });

        let curr_outputs = [0.0];
        let upstream = [0.0f32; 12];
        let mut so = side_outputs();
        let _ = apply(&def, &curr_outputs, &upstream, &mut so, &mut [0.0; 16]);

        let actions = so.action_queue.into_actions_or_noop();
        assert_eq!(actions, vec![WorldAction::NoOp]); // nothing was pushed
    }

    #[test]
    fn action_slot_empty_gate_does_not_fire() {
        let mut def = empty_def();
        def.action_bank.push(ActionSlot {
            behavior: ActionSlotBehavior::Emit(WorldActionKind::Eat),
            gate_inputs: Vec::new(), // empty gate
            param_inputs: Vec::new(),
        });

        let mut so = side_outputs();
        let _ = apply(&def, &[], &[0.0; 12], &mut so, &mut [0.0; 16]);

        let actions = so.action_queue.into_actions_or_noop();
        assert_eq!(actions, vec![WorldAction::NoOp]);
    }

    #[test]
    fn pop_behavior_removes_last_action() {
        let mut def = empty_def();
        // Constant(1.0) for gate
        def.compute_nodes.push(ComputeNode {
            kind: ComputeNodeKind::Constant(1.0),
            inputs: Vec::new(),
            plasticity: None,
        });
        // First slot: Emit(Eat)
        def.action_bank.push(ActionSlot {
            behavior: ActionSlotBehavior::Emit(WorldActionKind::Eat),
            gate_inputs: vec![GraphEdge {
                source: GraphSource::ComputeNode(0),
                weight: 1.0,
            }],
            param_inputs: Vec::new(),
        });
        // Second slot: Pop
        def.action_bank.push(ActionSlot {
            behavior: ActionSlotBehavior::Pop,
            gate_inputs: vec![GraphEdge {
                source: GraphSource::ComputeNode(0),
                weight: 1.0,
            }],
            param_inputs: Vec::new(),
        });

        let curr_outputs = [1.0];
        let upstream = [0.0f32; 12];
        let mut so = side_outputs();
        let _ = apply(&def, &curr_outputs, &upstream, &mut so, &mut [0.0; 16]);

        let actions = so.action_queue.into_actions_or_noop();
        assert_eq!(actions, vec![WorldAction::NoOp]); // Eat was pushed then popped
    }

    #[test]
    fn move_action_decodes_direction_from_params() {
        let mut def = empty_def();
        // Compute node 0: gate (1.0)
        def.compute_nodes.push(ComputeNode {
            kind: ComputeNodeKind::Constant(1.0),
            inputs: Vec::new(),
            plasticity: None,
        });
        // Compute node 1: direction param (2.0 = E)
        def.compute_nodes.push(ComputeNode {
            kind: ComputeNodeKind::Constant(2.0),
            inputs: Vec::new(),
            plasticity: None,
        });
        def.action_bank.push(ActionSlot {
            behavior: ActionSlotBehavior::Emit(WorldActionKind::Move),
            gate_inputs: vec![GraphEdge {
                source: GraphSource::ComputeNode(0),
                weight: 1.0,
            }],
            param_inputs: vec![GraphEdge {
                source: GraphSource::ComputeNode(1),
                weight: 1.0, // param[0] = 2.0 → E
            }],
        });

        let curr_outputs = [1.0, 2.0];
        let upstream = [0.0f32; 12];
        let mut so = side_outputs();
        let _ = apply(&def, &curr_outputs, &upstream, &mut so, &mut [0.0; 16]);

        let actions = so.action_queue.into_actions_or_noop();
        assert_eq!(actions, vec![WorldAction::Move(Direction::E)]);
    }

    #[test]
    fn reproduce_action_decodes_direction_and_energy() {
        let mut def = empty_def();
        def.compute_nodes.push(ComputeNode {
            kind: ComputeNodeKind::Constant(1.0),
            inputs: Vec::new(),
            plasticity: None,
        });
        def.compute_nodes.push(ComputeNode {
            kind: ComputeNodeKind::Constant(4.0), // direction = S
            inputs: Vec::new(),
            plasticity: None,
        });
        def.compute_nodes.push(ComputeNode {
            kind: ComputeNodeKind::Constant(15.0), // energy
            inputs: Vec::new(),
            plasticity: None,
        });
        def.action_bank.push(ActionSlot {
            behavior: ActionSlotBehavior::Emit(WorldActionKind::Reproduce),
            gate_inputs: vec![GraphEdge {
                source: GraphSource::ComputeNode(0),
                weight: 1.0,
            }],
            param_inputs: vec![
                GraphEdge {
                    source: GraphSource::ComputeNode(1),
                    weight: 1.0,
                },
                GraphEdge {
                    source: GraphSource::ComputeNode(2),
                    weight: 1.0,
                },
            ],
        });

        let curr_outputs = [1.0, 4.0, 15.0];
        let upstream = [0.0f32; 12];
        let mut so = side_outputs();
        let _ = apply(&def, &curr_outputs, &upstream, &mut so, &mut [0.0; 16]);

        let actions = so.action_queue.into_actions_or_noop();
        assert_eq!(actions.len(), 1);
        if let WorldAction::Reproduce {
            direction,
            energy_transfer,
        } = &actions[0]
        {
            assert_eq!(*direction, Direction::S);
            assert!((energy_transfer - 15.0).abs() < 1e-6);
        } else {
            panic!("expected Reproduce, got {:?}", actions[0]);
        }
    }

    #[test]
    fn multiple_action_slots_fire_in_order() {
        let mut def = empty_def();
        def.compute_nodes.push(ComputeNode {
            kind: ComputeNodeKind::Constant(1.0),
            inputs: Vec::new(),
            plasticity: None,
        });
        // Slot 0: Eat
        def.action_bank.push(ActionSlot {
            behavior: ActionSlotBehavior::Emit(WorldActionKind::Eat),
            gate_inputs: vec![GraphEdge {
                source: GraphSource::ComputeNode(0),
                weight: 1.0,
            }],
            param_inputs: Vec::new(),
        });
        // Slot 1: NoOp
        def.action_bank.push(ActionSlot {
            behavior: ActionSlotBehavior::Emit(WorldActionKind::NoOp),
            gate_inputs: vec![GraphEdge {
                source: GraphSource::ComputeNode(0),
                weight: 1.0,
            }],
            param_inputs: Vec::new(),
        });

        let curr_outputs = [1.0];
        let upstream = [0.0f32; 12];
        let mut so = side_outputs();
        let _ = apply(&def, &curr_outputs, &upstream, &mut so, &mut [0.0; 16]);

        let actions = so.action_queue.into_actions_or_noop();
        assert_eq!(actions, vec![WorldAction::Eat, WorldAction::NoOp]);
    }

    // ── Phase 3: execute gate ───────────────────────────────────────────────

    #[test]
    fn execute_gate_fires_when_positive_and_queue_nonempty() {
        let mut def = empty_def();
        def.compute_nodes.push(ComputeNode {
            kind: ComputeNodeKind::Constant(1.0),
            inputs: Vec::new(),
            plasticity: None,
        });
        // Wire action slot to put something in queue
        def.action_bank.push(ActionSlot {
            behavior: ActionSlotBehavior::Emit(WorldActionKind::Eat),
            gate_inputs: vec![GraphEdge {
                source: GraphSource::ComputeNode(0),
                weight: 1.0,
            }],
            param_inputs: Vec::new(),
        });
        // Wire execute gate
        def.execute_gate = ExecuteGate {
            inputs: vec![GraphEdge {
                source: GraphSource::ComputeNode(0),
                weight: 1.0,
            }],
        };

        let curr_outputs = [1.0];
        let upstream = [0.0f32; 12];
        let mut so = side_outputs();
        let result = apply(&def, &curr_outputs, &upstream, &mut so, &mut [0.0; 16]);

        assert!(result.terminal);
    }

    #[test]
    fn execute_gate_does_not_fire_with_empty_queue() {
        let mut def = empty_def();
        def.compute_nodes.push(ComputeNode {
            kind: ComputeNodeKind::Constant(1.0),
            inputs: Vec::new(),
            plasticity: None,
        });
        // No action slots → queue is empty
        def.execute_gate = ExecuteGate {
            inputs: vec![GraphEdge {
                source: GraphSource::ComputeNode(0),
                weight: 1.0,
            }],
        };

        let curr_outputs = [1.0];
        let upstream = [0.0f32; 12];
        let mut so = side_outputs();
        let result = apply(&def, &curr_outputs, &upstream, &mut so, &mut [0.0; 16]);

        assert!(!result.terminal);
    }

    #[test]
    fn execute_gate_does_not_fire_when_empty_inputs() {
        let def = empty_def(); // execute_gate has empty inputs

        let mut so = side_outputs();
        let result = apply(&def, &[], &[0.0; 12], &mut so, &mut [0.0; 16]);

        assert!(!result.terminal);
    }

    #[test]
    fn execute_gate_does_not_fire_when_wsum_zero() {
        let mut def = empty_def();
        def.compute_nodes.push(ComputeNode {
            kind: ComputeNodeKind::Constant(0.0),
            inputs: Vec::new(),
            plasticity: None,
        });
        def.action_bank.push(ActionSlot {
            behavior: ActionSlotBehavior::Emit(WorldActionKind::Eat),
            gate_inputs: vec![GraphEdge {
                source: GraphSource::ComputeNode(0),
                weight: 1.0, // wsum = 0.0 → gate doesn't fire, but let's test execute gate
            }],
            param_inputs: Vec::new(),
        });
        def.execute_gate = ExecuteGate {
            inputs: vec![GraphEdge {
                source: GraphSource::ComputeNode(0),
                weight: 1.0, // wsum = 0.0
            }],
        };

        let curr_outputs = [0.0];
        let upstream = [0.0f32; 12];
        let mut so = side_outputs();
        let result = apply(&def, &curr_outputs, &upstream, &mut so, &mut [0.0; 16]);

        assert!(!result.terminal);
    }

    // ── Cross-phase tests ───────────────────────────────────────────────────

    #[test]
    fn upstream_preserved_when_no_sinks_wired() {
        let def = empty_def();
        let upstream: [f32; 12] = [
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0,
        ];
        let mut so = side_outputs();
        let result = apply(&def, &[], &upstream, &mut so, &mut [0.0; 16]);

        assert_eq!(result.output_slots, upstream);
        assert_eq!(result.route_target_idx, 0.0);
        assert!(!result.terminal);
    }

    #[test]
    fn pop_on_empty_queue_is_noop() {
        let mut def = empty_def();
        def.compute_nodes.push(ComputeNode {
            kind: ComputeNodeKind::Constant(1.0),
            inputs: Vec::new(),
            plasticity: None,
        });
        def.action_bank.push(ActionSlot {
            behavior: ActionSlotBehavior::Pop,
            gate_inputs: vec![GraphEdge {
                source: GraphSource::ComputeNode(0),
                weight: 1.0,
            }],
            param_inputs: Vec::new(),
        });

        let curr_outputs = [1.0];
        let mut so = side_outputs();
        // Should not panic
        let _ = apply(&def, &curr_outputs, &[0.0; 12], &mut so, &mut [0.0; 16]);

        let actions = so.action_queue.into_actions_or_noop();
        assert_eq!(actions, vec![WorldAction::NoOp]);
    }
}
