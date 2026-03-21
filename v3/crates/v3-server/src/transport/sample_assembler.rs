//! Mapping from v3-core trace domain records to server sampler DTOs.

use v3_core::runtime::trace::domain as core_trace;

use crate::transport::sample_protocol::{
    BackendTracePayload, ExecutionSamplePayload, GraphActionSlotTracePayload,
    GraphExecuteGateTracePayload, GraphNodeEvalTracePayload, GraphOutputSinkTracePayload,
    GraphPassTracePayload, GraphTracePayload, MeshHopTracePayload, PerceptionDebugSnapshotPayload,
    RouteDecisionPayload, RouteKindPayload, SlotWritePayload, StaticInputsSnapshotPayload,
    TerminationReasonPayload, TickTracePayload, VmStepTracePayload, VmTracePayload,
};

#[inline]
fn serialize_core_shape<T: serde::Serialize>(
    value: T,
) -> Result<serde_json::Value, serde_json::Error> {
    serde_json::to_value(value)
}

pub fn assemble_execution_sample(
    sample: core_trace::ExecutionSample,
) -> Result<ExecutionSamplePayload, serde_json::Error> {
    Ok(ExecutionSamplePayload {
        creature_id: sample.creature_id,
        ticks: sample
            .ticks
            .into_iter()
            .map(assemble_tick)
            .collect::<Result<Vec<_>, _>>()?,
    })
}

#[inline]
fn assemble_tick(tick: core_trace::TickTrace) -> Result<TickTracePayload, serde_json::Error> {
    Ok(TickTracePayload {
        tick_number: tick.tick_number,
        energy_before: tick.energy_before,
        energy_after: tick.energy_after,
        static_inputs: StaticInputsSnapshotPayload {
            food_here: tick.static_inputs.food_here,
            neighbor_food: tick.static_inputs.neighbor_food,
            neighbor_barrier: tick.static_inputs.neighbor_barrier,
            neighbor_occupied: tick.static_inputs.neighbor_occupied,
            generation: tick.static_inputs.generation,
            age_ticks: tick.static_inputs.age_ticks,
        },
        debug_perception: tick
            .debug_perception
            .map(|p| PerceptionDebugSnapshotPayload {
                area_food: p.area_food,
                area_barrier: p.area_barrier,
                area_occupancy: p.area_occupancy,
                nearby_core: p.nearby_core,
                nearby_vitals: p.nearby_vitals,
                nearby_identity: p.nearby_identity,
            }),
        hops: tick
            .hops
            .into_iter()
            .map(assemble_hop)
            .collect::<Result<Vec<_>, _>>()?,
        final_actions: tick
            .final_actions
            .into_iter()
            .map(serialize_core_shape)
            .collect::<Result<Vec<_>, _>>()?,
        termination_reason: match tick.termination_reason {
            core_trace::TerminationReason::ActionEmitted => TerminationReasonPayload::ActionEmitted,
            core_trace::TerminationReason::EnergyExhausted => {
                TerminationReasonPayload::EnergyExhausted
            }
            core_trace::TerminationReason::MaxHopsReached => {
                TerminationReasonPayload::MaxHopsReached
            }
            core_trace::TerminationReason::NoTargets => TerminationReasonPayload::NoTargets,
            core_trace::TerminationReason::MissingNode => TerminationReasonPayload::MissingNode,
        },
        priority_bid: tick.priority_bid,
    })
}

#[inline]
fn assemble_hop(hop: core_trace::MeshHopTrace) -> Result<MeshHopTracePayload, serde_json::Error> {
    Ok(MeshHopTracePayload {
        hop_index: hop.hop_index,
        node_id: hop.node_id.0 as u64,
        input_refs: hop
            .input_refs
            .into_iter()
            .map(serialize_core_shape)
            .collect::<Result<Vec<_>, _>>()?,
        upstream_slots: hop.upstream_slots,
        energy_before: hop.energy_before,
        energy_after: hop.energy_after,
        output_slots: hop.output_slots,
        route: RouteDecisionPayload {
            kind: match hop.route.kind {
                core_trace::TraceRouteKind::VmWrap => RouteKindPayload::VmWrap,
                core_trace::TraceRouteKind::CgpNormalized => RouteKindPayload::CgpNormalized,
            },
            raw_value: hop.route.raw_value,
            resolved_target_index: hop.resolved_target_index,
        },
        backend_trace: match hop.backend_trace {
            core_trace::BackendTrace::Vm(vm) => BackendTracePayload::Vm(VmTracePayload {
                register_count: vm.register_count,
                constants: vm.constants,
                steps: vm
                    .steps
                    .into_iter()
                    .map(|s| {
                        Ok(VmStepTracePayload {
                            pc: s.pc,
                            instruction: serialize_core_shape(s.instruction)?,
                            energy_cost: s.energy_cost,
                            energy_after: s.energy_after,
                            register_changes: s.register_changes,
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?,
                final_registers: vm.final_registers,
                final_payload: vm.final_payload,
                final_meta: vm.final_meta,
                final_route_value: vm.final_route_value,
                slot_writes: vm
                    .slot_writes
                    .into_iter()
                    .map(|w| SlotWritePayload {
                        slot_idx: w.slot_idx,
                        old_value: w.old_value,
                        new_value: w.new_value,
                    })
                    .collect(),
            }),
            core_trace::BackendTrace::Graph(graph) => {
                BackendTracePayload::Graph(GraphTracePayload {
                    passes: graph
                        .passes
                        .into_iter()
                        .map(|p| GraphPassTracePayload {
                            pass_index: p.pass_index,
                            energy_cost: p.energy_cost,
                            energy_after: p.energy_after,
                            node_evaluations: p
                                .node_evaluations
                                .into_iter()
                                .map(|n| GraphNodeEvalTracePayload {
                                    node_index: n.node_index,
                                    kind: n.kind,
                                    weighted_inputs: n.weighted_inputs,
                                    weighted_sum: n.weighted_sum,
                                    state_before: n.state_before,
                                    state_after: n.state_after,
                                    output: n.output,
                                })
                                .collect(),
                            max_delta: p.max_delta,
                        })
                        .collect(),
                    converged: graph.converged,
                    stable_passes_count: graph.stable_passes_count,
                    final_outputs: graph.final_outputs,
                    output_sinks: graph
                        .output_sinks
                        .into_iter()
                        .map(|s| GraphOutputSinkTracePayload {
                            wired: s.wired,
                            weighted_sum: s.weighted_sum,
                            applied: s.applied,
                            applied_value: s.applied_value,
                        })
                        .collect(),
                    action_slots: graph
                        .action_slots
                        .into_iter()
                        .map(|slot| {
                            Ok(GraphActionSlotTracePayload {
                                wired: slot.wired,
                                gate_weighted_sum: slot.gate_weighted_sum,
                                fired: slot.fired,
                                param_values: slot.param_values,
                                queue_len_before: slot.queue_len_before,
                                queue_len_after: slot.queue_len_after,
                                emitted_action: slot
                                    .emitted_action
                                    .map(serialize_core_shape)
                                    .transpose()?,
                            })
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                    execute_gate: GraphExecuteGateTracePayload {
                        wired: graph.execute_gate.wired,
                        weighted_sum: graph.execute_gate.weighted_sum,
                        queue_non_empty: graph.execute_gate.queue_non_empty,
                        fired: graph.execute_gate.fired,
                    },
                })
            }
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use v3_core::runtime::trace::domain::{
        BackendTrace, ExecutionSample, GraphActionSlotTrace, GraphExecuteGateTrace,
        GraphOutputSinkTrace, GraphTrace, MeshHopTrace, TickTrace, TraceRouteDecision,
        TraceRouteKind, VmTrace,
    };
    use v3_core::runtime::OUTPUT_SLOT_COUNT;

    #[test]
    fn assembler_preserves_route_shape() {
        let sample = ExecutionSample {
            creature_id: 42,
            ticks: vec![TickTrace {
                tick_number: 7,
                energy_before: 10.0,
                energy_after: 9.5,
                static_inputs: core_trace::StaticInputsSnapshot {
                    food_here: 0.0,
                    neighbor_food: [0.0; 8],
                    neighbor_barrier: [0.0; 8],
                    neighbor_occupied: [0.0; 8],
                    generation: 1.0,
                    age_ticks: 2.0,
                },
                debug_perception: None,
                hops: vec![MeshHopTrace {
                    hop_index: 0,
                    node_id: v3_core::contracts::NodeId::new(1),
                    input_refs: vec![],
                    upstream_slots: [0.0; OUTPUT_SLOT_COUNT],
                    energy_before: 10.0,
                    energy_after: 9.5,
                    output_slots: [0.0; OUTPUT_SLOT_COUNT],
                    route: TraceRouteDecision {
                        kind: TraceRouteKind::VmWrap,
                        raw_value: 3.0,
                    },
                    resolved_target_index: 1,
                    backend_trace: BackendTrace::Vm(VmTrace {
                        register_count: 1,
                        constants: vec![],
                        steps: vec![],
                        final_registers: vec![0.0],
                        final_payload: [0.0; OUTPUT_SLOT_COUNT],
                        final_meta: [0.0; 8],
                        final_route_value: 3.0,
                        slot_writes: vec![],
                    }),
                }],
                final_actions: vec![],
                termination_reason: core_trace::TerminationReason::NoTargets,
                priority_bid: 0.0,
            }],
        };

        let assembled =
            assemble_execution_sample(sample).expect("valid core trace sample should assemble");
        let route = &assembled.ticks[0].hops[0].route;
        assert!(matches!(route.kind, RouteKindPayload::VmWrap));
        assert!((route.raw_value - 3.0).abs() < 1e-6);
        assert_eq!(route.resolved_target_index, 1);
    }

    #[test]
    fn assembler_maps_graph_effect_phase_trace_fields() {
        let sample = ExecutionSample {
            creature_id: 7,
            ticks: vec![TickTrace {
                tick_number: 1,
                energy_before: 5.0,
                energy_after: 4.5,
                static_inputs: core_trace::StaticInputsSnapshot {
                    food_here: 0.0,
                    neighbor_food: [0.0; 8],
                    neighbor_barrier: [0.0; 8],
                    neighbor_occupied: [0.0; 8],
                    generation: 1.0,
                    age_ticks: 1.0,
                },
                debug_perception: None,
                hops: vec![MeshHopTrace {
                    hop_index: 0,
                    node_id: v3_core::contracts::NodeId::new(2),
                    input_refs: vec![],
                    upstream_slots: [0.0; OUTPUT_SLOT_COUNT],
                    energy_before: 5.0,
                    energy_after: 4.5,
                    output_slots: [1.0; OUTPUT_SLOT_COUNT],
                    route: TraceRouteDecision {
                        kind: TraceRouteKind::CgpNormalized,
                        raw_value: 0.25,
                    },
                    resolved_target_index: 0,
                    backend_trace: BackendTrace::Graph(GraphTrace {
                        passes: vec![],
                        converged: true,
                        stable_passes_count: 2,
                        final_outputs: vec![1.0],
                        output_sinks: vec![GraphOutputSinkTrace {
                            wired: true,
                            weighted_sum: 1.0,
                            applied: true,
                            applied_value: 1.0,
                        }],
                        action_slots: vec![GraphActionSlotTrace {
                            wired: true,
                            gate_weighted_sum: 1.0,
                            fired: true,
                            param_values: [0.0, 0.0],
                            queue_len_before: 0,
                            queue_len_after: 1,
                            emitted_action: Some(v3_core::contracts::WorldAction::Eat {
                                type_idx: v3_core::config::OrdinaryFoodTypeId::default(),
                            }),
                        }],
                        execute_gate: GraphExecuteGateTrace {
                            wired: true,
                            weighted_sum: 1.0,
                            queue_non_empty: true,
                            fired: true,
                        },
                    }),
                }],
                final_actions: vec![v3_core::contracts::WorldAction::Eat {
                    type_idx: v3_core::config::OrdinaryFoodTypeId::default(),
                }],
                termination_reason: core_trace::TerminationReason::ActionEmitted,
                priority_bid: 0.0,
            }],
        };

        let assembled =
            assemble_execution_sample(sample).expect("valid core trace sample should assemble");

        let graph = match &assembled.ticks[0].hops[0].backend_trace {
            BackendTracePayload::Graph(graph) => graph,
            BackendTracePayload::Vm(_) => panic!("expected graph payload"),
        };
        assert_eq!(graph.output_sinks.len(), 1);
        assert!(graph.output_sinks[0].wired);
        assert!(graph.output_sinks[0].applied);
        assert_eq!(graph.action_slots.len(), 1);
        assert!(graph.action_slots[0].fired);
        assert!(graph.action_slots[0].emitted_action.is_some());
        assert!(graph.execute_gate.fired);
        assert!(graph.execute_gate.queue_non_empty);
    }
}
