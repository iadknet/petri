//! Post-convergence effect pass for CGP-style graph backend evaluation.

use crate::config::OrdinaryFoodTypeId;
use crate::contracts::{Direction, InputReference, WorldAction};
use crate::creature::genome::cgp::{
    ActionSlotBehavior, CgpGraphBackendDef, GraphEdge, OutputSinkKind, WorldActionKind,
};
use crate::runtime::cgp::sources::{resolve_source, resolve_source_post_convergence};
use crate::runtime::inputs::ResolveCtx;
use crate::runtime::routing::RouteDecision;
use crate::runtime::trace::domain::{
    GraphActionSlotTrace, GraphExecuteGateTrace, GraphOutputSinkTrace,
};
use crate::runtime::types::{sanitize_f32, MeshSideOutputs, NodeResult, OUTPUT_SLOT_COUNT};

pub(crate) struct CgpEffectsTrace {
    pub(crate) output_sinks: Vec<GraphOutputSinkTrace>,
    pub(crate) action_slots: Vec<GraphActionSlotTrace>,
    pub(crate) execute_gate: GraphExecuteGateTrace,
}

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
fn decode_food_type_idx(raw: f32) -> OrdinaryFoodTypeId {
    if raw.is_finite() && raw >= 0.0 {
        OrdinaryFoodTypeId::new(raw.round().clamp(0.0, u16::MAX as f32) as u16)
    } else {
        OrdinaryFoodTypeId::default()
    }
}

#[inline]
fn decode_action_from_kind(kind: WorldActionKind, param_values: &[f32]) -> WorldAction {
    let p0 = param_values.first().copied().unwrap_or(0.0);
    let p1 = param_values.get(1).copied().unwrap_or(0.0);
    match kind {
        WorldActionKind::NoOp => WorldAction::NoOp,
        WorldActionKind::Eat => WorldAction::Eat {
            type_idx: decode_food_type_idx(p0),
        },
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
    upstream_slots: &[f32; OUTPUT_SLOT_COUNT],
    side_outputs: &mut MeshSideOutputs,
    shared_memory: &mut [f32; 16],
    prev_shared_memory: &[f32; 16],
) -> (NodeResult, CgpEffectsTrace) {
    let mut output_slots = *upstream_slots;
    let mut route_raw_value = 0.0f32;
    let compute_count = def.compute_nodes.len();
    let mut buf = Vec::with_capacity(8);
    let mut output_sink_traces = Vec::with_capacity(def.output_sinks.len());
    let mut action_slot_traces = Vec::with_capacity(def.action_bank.len());

    // Phase 1: value outputs
    for sink in &def.output_sinks {
        if sink.inputs.is_empty() {
            output_sink_traces.push(GraphOutputSinkTrace {
                wired: false,
                weighted_sum: 0.0,
                applied: false,
                applied_value: 0.0,
            });
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
        let mut applied = false;
        let mut applied_value = 0.0;

        match sink.kind {
            OutputSinkKind::CustomOutput(s) => {
                if (s as usize) < OUTPUT_SLOT_COUNT {
                    applied_value = sanitize_f32(wsum);
                    output_slots[s as usize] = applied_value;
                    applied = true;
                }
            }
            OutputSinkKind::RouterOutput => {
                applied_value = sanitize_f32(wsum);
                route_raw_value = applied_value;
                applied = true;
            }
            OutputSinkKind::WriteSlot(s) => {
                if (s as usize) < 16 {
                    applied_value = sanitize_f32(wsum);
                    shared_memory[s as usize] = applied_value;
                    applied = true;
                }
            }
            OutputSinkKind::ClearSlot(s) => {
                if (s as usize) < 16 {
                    shared_memory[s as usize] = 0.0;
                    applied_value = 0.0;
                    applied = true;
                }
            }
        }

        output_sink_traces.push(GraphOutputSinkTrace {
            wired: true,
            weighted_sum: sanitize_f32(wsum),
            applied,
            applied_value,
        });
    }

    // Phase 2: action bank scan
    for slot in &def.action_bank {
        let wired = !slot.gate_inputs.is_empty() || !slot.param_inputs.is_empty();
        let queue_len_before = side_outputs.action_queue.len();
        let gate_wsum = if !slot.gate_inputs.is_empty() {
            edges_wsum(
                &slot.gate_inputs,
                compute_count,
                curr_outputs,
                input_refs,
                resolve_ctx,
                shared_memory,
                prev_shared_memory,
                &mut buf,
            )
        } else {
            0.0
        };
        let fired = !slot.gate_inputs.is_empty() && gate_wsum > 0.0;
        let mut param_values = [0.0f32; 2];
        let mut emitted_action = None;

        if fired {
            match slot.behavior {
                ActionSlotBehavior::Pop => {
                    side_outputs.action_queue.pop();
                }
                ActionSlotBehavior::Emit(kind) => {
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
                    emitted_action = Some(action);
                    side_outputs.action_queue.push(action);
                }
            }
        }

        let param_values_trace = [sanitize_f32(param_values[0]), sanitize_f32(param_values[1])];

        action_slot_traces.push(GraphActionSlotTrace {
            wired,
            gate_weighted_sum: sanitize_f32(gate_wsum),
            fired,
            param_values: param_values_trace,
            queue_len_before,
            queue_len_after: side_outputs.action_queue.len(),
            emitted_action,
        });
    }

    // Phase 3: execute gate
    let execute_wired = !def.execute_gate.inputs.is_empty();
    let execute_wsum = if execute_wired {
        edges_wsum(
            &def.execute_gate.inputs,
            compute_count,
            curr_outputs,
            input_refs,
            resolve_ctx,
            shared_memory,
            prev_shared_memory,
            &mut buf,
        )
    } else {
        0.0
    };
    let queue_non_empty = !side_outputs.action_queue.is_empty();
    let terminal = execute_wired && execute_wsum > 0.0 && queue_non_empty;

    (
        NodeResult {
            output_slots,
            route: RouteDecision::CgpNormalized {
                raw_value: route_raw_value,
            },
            terminal,
            energy_exhausted: false,
        },
        CgpEffectsTrace {
            output_sinks: output_sink_traces,
            action_slots: action_slot_traces,
            execute_gate: GraphExecuteGateTrace {
                wired: execute_wired,
                weighted_sum: sanitize_f32(execute_wsum),
                queue_non_empty,
                fired: terminal,
            },
        },
    )
}

#[cfg(test)]
mod tests {
    use super::decode_action_from_kind;
    use crate::config::OrdinaryFoodTypeId;
    use crate::contracts::WorldAction;
    use crate::creature::genome::cgp::WorldActionKind;

    #[test]
    fn decode_action_from_kind_eat_uses_first_param_slot_for_type_idx() {
        let action = decode_action_from_kind(WorldActionKind::Eat, &[3.0, 8.0]);
        assert_eq!(
            action,
            WorldAction::Eat {
                type_idx: OrdinaryFoodTypeId::new(3),
            }
        );
    }
}
