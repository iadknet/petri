//! Post-convergence effect pass for CGP-style graph backend evaluation.

use crate::contracts::{InputReference, WorldAction, MAX_GATE_SLOTS};
use crate::creature::genome::cgp::{
    ActionSlot, ActionSlotBehavior, CgpGraphBackendDef, DirectionBidEdge, GraphEdge,
    OutputSinkKind, WorldActionKind,
};
use crate::runtime::action_decode::{decode_world_action, DirectionBank, DIRECTION_BANK_SLOTS};
use crate::runtime::cgp::sources::resolve_source_post_convergence;
use crate::runtime::inputs::ResolveCtx;
use crate::runtime::routing::RouteGateMap;
use crate::runtime::trace::domain::{
    GraphActionSlotTrace, GraphExecuteGateTrace, GraphOutputSinkTrace,
};
use crate::runtime::types::{
    sanitize_f32, shared_memory_write_changed, MeshSideOutputs, NodeResult, OUTPUT_SLOT_COUNT,
};

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

/// The direction bank a movement slot writes (T11.F21): bid `d` is the
/// weighted sum of the edges with `direction == d`, and the bank exists once
/// any edge lands in `0..8`. `Eat`, `NoOp`, and out-of-range edges write
/// nothing.
#[allow(clippy::too_many_arguments)]
fn direction_bank(
    kind: WorldActionKind,
    bids: &[DirectionBidEdge],
    compute_count: usize,
    curr_outputs: &[f32],
    input_refs: &[InputReference],
    resolve_ctx: &ResolveCtx<'_>,
    shared_memory: &[f32; 16],
    prev_shared_memory: &[f32; 16],
) -> Option<DirectionBank> {
    let movement = matches!(
        kind,
        WorldActionKind::Move | WorldActionKind::Reproduce | WorldActionKind::StealEnergy
    );
    let mut bank: Option<DirectionBank> = None;
    for bid in bids
        .iter()
        .filter(|bid| movement && (bid.direction as usize) < DIRECTION_BANK_SLOTS)
    {
        let v = resolve_source_post_convergence(
            &bid.edge.source,
            compute_count,
            curr_outputs,
            input_refs,
            resolve_ctx,
            shared_memory,
            prev_shared_memory,
        );
        bank.get_or_insert([0.0; DIRECTION_BANK_SLOTS])[bid.direction as usize] +=
            v * bid.edge.weight;
    }
    bank
}

/// A fired `Emit(kind)` slot's decoded action beside the values it decoded
/// from: the two parameter sums and the direction bank, if one was written.
struct EmittedAction {
    param_values: [f32; 2],
    bank: Option<DirectionBank>,
    action: WorldAction,
}

/// Resolve a fired `Emit(kind)` slot: `param_inputs[i]` fills `param[i]` for
/// `i < 2`, the direction bank sums `direction_bids`, and the shared decode
/// table turns both into the action.
#[allow(clippy::too_many_arguments)]
fn emit_action(
    kind: WorldActionKind,
    slot: &ActionSlot,
    compute_count: usize,
    curr_outputs: &[f32],
    input_refs: &[InputReference],
    resolve_ctx: &ResolveCtx<'_>,
    shared_memory: &[f32; 16],
    prev_shared_memory: &[f32; 16],
) -> EmittedAction {
    let mut param_values = [0.0f32; 2];
    for (value, edge) in param_values.iter_mut().zip(&slot.param_inputs) {
        let v = resolve_source_post_convergence(
            &edge.source,
            compute_count,
            curr_outputs,
            input_refs,
            resolve_ctx,
            shared_memory,
            prev_shared_memory,
        );
        *value = v * edge.weight;
    }
    let bank = direction_bank(
        kind,
        &slot.direction_bids,
        compute_count,
        curr_outputs,
        input_refs,
        resolve_ctx,
        shared_memory,
        prev_shared_memory,
    );
    let action = decode_action_from_kind(kind, &param_values, bank.as_ref());
    EmittedAction {
        param_values,
        bank,
        action,
    }
}

/// The `Direction::ALL` index a movement action commits; `None` for the rest.
fn movement_direction(action: &WorldAction) -> Option<u8> {
    match action {
        WorldAction::Move(direction)
        | WorldAction::Reproduce { direction, .. }
        | WorldAction::StealEnergy { direction, .. } => Some(direction.to_index() as u8),
        WorldAction::NoOp | WorldAction::Eat { .. } => None,
    }
}

/// The `action_type` discriminant of the shared decode table
/// (v3-vm-isa-spec.md Section 7) for a slot's emit kind.
#[inline]
fn action_type_of(kind: WorldActionKind) -> u8 {
    match kind {
        WorldActionKind::NoOp => 0,
        WorldActionKind::Eat => 1,
        WorldActionKind::Move => 2,
        WorldActionKind::Reproduce => 3,
        WorldActionKind::StealEnergy => 4,
    }
}

/// Decode a slot's action through the same table the VM uses: the two param
/// values fill meta slots 0 and 1, and the bank (when written) selects the
/// direction of a movement action.
#[inline]
fn decode_action_from_kind(
    kind: WorldActionKind,
    param_values: &[f32; 2],
    bank: Option<&DirectionBank>,
) -> WorldAction {
    let mut meta = [0.0f32; 8];
    meta[..2].copy_from_slice(param_values);
    decode_world_action(action_type_of(kind), &meta, bank)
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
    let mut route_gates = RouteGateMap::default();
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
            OutputSinkKind::RouterGate(slot) => {
                let s = slot as usize;
                if s < MAX_GATE_SLOTS {
                    let val = sanitize_f32(wsum);
                    route_gates.scores[s] = val;
                    applied_value = val;
                    applied = true;
                }
            }
            OutputSinkKind::WriteSlot(s) => {
                if (s as usize) < 16 {
                    applied_value = sanitize_f32(wsum);
                    side_outputs.work_counters.shared_memory_writes_changed += u32::from(
                        shared_memory_write_changed(shared_memory[s as usize], applied_value),
                    );
                    shared_memory[s as usize] = applied_value;
                    applied = true;
                }
            }
            OutputSinkKind::ClearSlot(s) => {
                if (s as usize) < 16 {
                    side_outputs.work_counters.shared_memory_writes_changed +=
                        u32::from(shared_memory_write_changed(shared_memory[s as usize], 0.0));
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
        let wired = slot.is_wired();
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
        let mut bank = None;
        let mut chosen_direction = None;

        if fired {
            match slot.behavior {
                ActionSlotBehavior::Pop => {
                    side_outputs.action_queue.pop();
                }
                ActionSlotBehavior::Emit(kind) => {
                    let emitted = emit_action(
                        kind,
                        slot,
                        compute_count,
                        curr_outputs,
                        input_refs,
                        resolve_ctx,
                        shared_memory,
                        prev_shared_memory,
                    );
                    param_values = emitted.param_values;
                    bank = emitted.bank;
                    chosen_direction = bank.and_then(|_| movement_direction(&emitted.action));
                    emitted_action = Some(emitted.action);
                    side_outputs.action_queue.push(emitted.action);
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
            direction_bids: bank.map(|bids| bids.map(sanitize_f32)),
            chosen_direction,
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
            route_gates,
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
    use crate::contracts::{Direction, WorldAction};
    use crate::creature::genome::cgp::WorldActionKind;

    #[test]
    fn decode_action_from_kind_move_reads_the_bank_when_written() {
        let bank = [0.0, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0];
        assert_eq!(
            decode_action_from_kind(WorldActionKind::Move, &[1.0, 0.0], Some(&bank)),
            WorldAction::Move(Direction::ALL[5])
        );
        assert_eq!(
            decode_action_from_kind(WorldActionKind::Move, &[1.0, 0.0], None),
            WorldAction::Move(Direction::ALL[1])
        );
        assert_eq!(
            decode_action_from_kind(WorldActionKind::Eat, &[1.0, 0.0], Some(&bank)),
            decode_action_from_kind(WorldActionKind::Eat, &[1.0, 0.0], None)
        );
    }

    #[test]
    fn decode_action_from_kind_eat_uses_first_param_slot_for_type_idx() {
        let action = decode_action_from_kind(WorldActionKind::Eat, &[3.0, 8.0], None);
        assert_eq!(
            action,
            WorldAction::Eat {
                type_idx: OrdinaryFoodTypeId::new(3),
            }
        );
    }
}
