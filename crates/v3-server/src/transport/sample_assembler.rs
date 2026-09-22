//! Mapping from v3-core trace domain records to server sampler DTOs.

use v3_core::runtime::trace::domain as core_trace;

use crate::transport::sample_protocol::{
    BackendTracePayload, ExecutionSamplePayload, GateScorePayload, GraphNodeEvalTracePayload,
    GraphOutputSinkTracePayload, GraphPassTracePayload, GraphTracePayload, MeshHopTracePayload,
    MeshPassTracePayload, PassEndReasonPayload, PerceptionDebugSnapshotPayload,
    RouteDecisionPayload, SlotWritePayload, StaticInputsSnapshotPayload, TerminationReasonPayload,
    TickTracePayload, VmStepTracePayload, VmTracePayload,
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
        passes: tick
            .passes
            .into_iter()
            .map(assemble_pass)
            .collect::<Result<Vec<_>, _>>()?,
        final_actions: tick
            .final_actions
            .into_iter()
            .map(serialize_core_shape)
            .collect::<Result<Vec<_>, _>>()?,
        termination_reason: match tick.termination_reason {
            core_trace::TerminationReason::NoDecision => TerminationReasonPayload::NoDecision,
            core_trace::TerminationReason::TerminateVoted => {
                TerminationReasonPayload::TerminateVoted
            }
            core_trace::TerminationReason::ActionCapReached => {
                TerminationReasonPayload::ActionCapReached
            }
            core_trace::TerminationReason::EnergyExhausted => {
                TerminationReasonPayload::EnergyExhausted
            }
        },
        priority_bid: tick.priority_bid,
        commit_counts: tick.commit_counts,
    })
}

#[inline]
fn assemble_pass(
    pass: core_trace::MeshPassTrace,
) -> Result<MeshPassTracePayload, serde_json::Error> {
    Ok(MeshPassTracePayload {
        pass_index: pass.pass_index,
        end_reason: match pass.end_reason {
            core_trace::PassEndReason::Decided => PassEndReasonPayload::Decided,
            core_trace::PassEndReason::PassCapReached => PassEndReasonPayload::PassCapReached,
            core_trace::PassEndReason::NoTargets => PassEndReasonPayload::NoTargets,
            core_trace::PassEndReason::MissingNode => PassEndReasonPayload::MissingNode,
            core_trace::PassEndReason::EnergyExhausted => PassEndReasonPayload::EnergyExhausted,
        },
        votes: pass.votes,
        effective_votes: pass.effective_votes,
        committed: pass.committed.map(serialize_core_shape).transpose()?,
        hops: pass.hops,
    })
}

#[inline]
fn assemble_hop(hop: core_trace::MeshHopTrace) -> Result<MeshHopTracePayload, serde_json::Error> {
    Ok(MeshHopTracePayload {
        hop_index: hop.hop_index,
        pass_index: hop.pass_index,
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
        route: hop.route.as_ref().map(|r| RouteDecisionPayload {
            gate_scores: r
                .gate_scores
                .iter()
                .map(|gs| GateScorePayload {
                    slot: gs.slot,
                    target_id: gs.target_id.0,
                    gate_bias: gs.gate_bias,
                    runtime_score: gs.runtime_score,
                    effective_score: gs.effective_score,
                })
                .collect(),
            selected_target_idx: r.selected_target_idx,
            selected_target_id: r.selected_target_id.0,
        }),
        vote_contribution: hop.vote_contribution,
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
                })
            }
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use v3_core::creature::genome::vote::{VOTE_KIND_COUNT, VOTE_SINK_COUNT};
    use v3_core::runtime::trace::domain::{
        BackendTrace, ExecutionSample, GraphOutputSinkTrace, GraphTrace, MeshHopTrace,
        MeshPassTrace, PassEndReason, TickTrace, TraceGateScore, TraceRouteDecision, VmTrace,
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
                    age_ticks: 0.004,
                },
                debug_perception: None,
                hops: vec![MeshHopTrace {
                    hop_index: 0,
                    pass_index: 0,
                    node_id: v3_core::contracts::NodeId::new(1),
                    input_refs: vec![],
                    upstream_slots: [0.0; OUTPUT_SLOT_COUNT],
                    energy_before: 10.0,
                    energy_after: 9.5,
                    output_slots: [0.0; OUTPUT_SLOT_COUNT],
                    vote_contribution: [0.0; VOTE_SINK_COUNT],
                    route: Some(TraceRouteDecision {
                        gate_scores: vec![
                            TraceGateScore {
                                slot: 0,
                                target_id: v3_core::contracts::NodeId::new(10),
                                gate_bias: 0.0,
                                runtime_score: -1.0,
                                effective_score: -1.0,
                            },
                            TraceGateScore {
                                slot: 1,
                                target_id: v3_core::contracts::NodeId::new(11),
                                gate_bias: 0.5,
                                runtime_score: 0.0,
                                effective_score: 0.5,
                            },
                        ],
                        selected_target_idx: 1,
                        selected_target_id: v3_core::contracts::NodeId::new(11),
                    }),
                    backend_trace: BackendTrace::Vm(VmTrace {
                        register_count: 1,
                        constants: vec![],
                        steps: vec![],
                        final_registers: vec![0.0],
                        final_payload: [0.0; OUTPUT_SLOT_COUNT],
                        slot_writes: vec![],
                    }),
                }],
                passes: vec![],
                final_actions: vec![],
                termination_reason: core_trace::TerminationReason::NoDecision,
                priority_bid: 0.0,
                commit_counts: [0; VOTE_KIND_COUNT],
            }],
        };

        let assembled =
            assemble_execution_sample(sample).expect("valid core trace sample should assemble");
        let route = assembled.ticks[0].hops[0]
            .route
            .as_ref()
            .expect("route should be Some");
        assert_eq!(route.gate_scores.len(), 2);
        assert_eq!(route.selected_target_idx, 1);
        assert_eq!(route.selected_target_id, 11);
        assert!((route.gate_scores[0].effective_score - (-1.0)).abs() < 1e-6);
        assert!((route.gate_scores[1].effective_score - 0.5).abs() < 1e-6);
    }

    #[test]
    fn assembler_maps_graph_effect_phase_and_pass_records() {
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
                    age_ticks: 0.002,
                },
                debug_perception: None,
                hops: vec![MeshHopTrace {
                    hop_index: 0,
                    pass_index: 0,
                    node_id: v3_core::contracts::NodeId::new(2),
                    input_refs: vec![],
                    upstream_slots: [0.0; OUTPUT_SLOT_COUNT],
                    energy_before: 5.0,
                    energy_after: 4.5,
                    output_slots: [1.0; OUTPUT_SLOT_COUNT],
                    vote_contribution: [0.0; VOTE_SINK_COUNT],
                    route: Some(TraceRouteDecision {
                        gate_scores: vec![TraceGateScore {
                            slot: 0,
                            target_id: v3_core::contracts::NodeId::new(5),
                            gate_bias: 0.0,
                            runtime_score: 0.25,
                            effective_score: 0.25,
                        }],
                        selected_target_idx: 0,
                        selected_target_id: v3_core::contracts::NodeId::new(5),
                    }),
                    backend_trace: BackendTrace::Graph(GraphTrace {
                        temporal_committed: true,
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
                    }),
                }],
                passes: vec![
                    MeshPassTrace {
                        pass_index: 0,
                        end_reason: PassEndReason::Decided,
                        votes: [1.0; VOTE_SINK_COUNT],
                        effective_votes: [1.0; VOTE_KIND_COUNT],
                        committed: Some(v3_core::contracts::WorldAction::Eat {
                            type_idx: v3_core::config::OrdinaryFoodTypeId::default(),
                        }),
                        hops: 1,
                    },
                    MeshPassTrace {
                        pass_index: 1,
                        end_reason: PassEndReason::PassCapReached,
                        votes: [0.0; VOTE_SINK_COUNT],
                        effective_votes: [-1.0, 0.0, 0.0, 0.0],
                        committed: None,
                        hops: 64,
                    },
                ],
                final_actions: vec![v3_core::contracts::WorldAction::Eat {
                    type_idx: v3_core::config::OrdinaryFoodTypeId::default(),
                }],
                termination_reason: core_trace::TerminationReason::NoDecision,
                priority_bid: 0.0,
                commit_counts: [1, 0, 0, 0],
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
        let tick = &assembled.ticks[0];
        assert_eq!(tick.passes.len(), 2);
        assert!(matches!(
            tick.passes[0].end_reason,
            PassEndReasonPayload::Decided
        ));
        assert!(tick.passes[0].committed.is_some());
        assert!(matches!(
            tick.passes[1].end_reason,
            PassEndReasonPayload::PassCapReached
        ));
        assert!(tick.passes[1].committed.is_none());
        assert_eq!(tick.commit_counts, [1, 0, 0, 0]);
        let json = serde_json::to_value(tick).expect("pass records serialize");
        assert_eq!(json["passes"][0]["end_reason"], "Decided");
        assert_eq!(json["termination_reason"], "NoDecision");
    }
}
