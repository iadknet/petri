//! Post-convergence effect pass for CGP-style graph backend evaluation.

use crate::contracts::{Direction, InputReference, WorldAction};
use crate::creature::genome::cgp::{
    ActionSlotBehavior, CgpGraphBackendDef, GraphEdge, OutputSinkKind, WorldActionKind,
};
use crate::runtime::cgp::sources::{resolve_source, resolve_source_post_convergence};
use crate::runtime::inputs::ResolveCtx;
use crate::runtime::routing::RouteDecision;
use crate::runtime::types::{sanitize_f32, MeshSideOutputs, NodeResult, OUTPUT_SLOT_COUNT};

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
        let v = resolve_source_post_convergence(
            &edge.source,
            compute_count,
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

#[inline]
fn decode_direction(raw: f32) -> Direction {
    let clamped = if raw.is_nan() {
        0.0
    } else {
        raw.round().clamp(0.0, 7.0)
    };
    Direction::ALL[clamped as usize]
}

#[inline]
fn clamp_non_negative_finite(v: f32) -> f32 {
    if v.is_finite() && v >= 0.0 {
        v
    } else {
        0.0
    }
}

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

/// Apply post-convergence effects from CGP graph evaluation.
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
    let mut route_raw_value = 0.0f32;
    let compute_count = def.compute_nodes.len();
    let mut buf = Vec::with_capacity(8);

    // Phase 1: value outputs
    for sink in &def.output_sinks {
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
                route_raw_value = sanitize_f32(wsum);
            }
            OutputSinkKind::WriteSlot(s) => {
                if (s as usize) < 16 {
                    shared_memory[s as usize] = sanitize_f32(wsum);
                }
            }
            OutputSinkKind::ClearSlot(s) => {
                if (s as usize) < 16 {
                    shared_memory[s as usize] = 0.0;
                }
            }
        }
    }

    // Phase 2: action bank scan
    for slot in &def.action_bank {
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
        route: RouteDecision::CgpNormalized {
            raw_value: route_raw_value,
        },
        terminal,
        energy_exhausted: false,
    }
}
