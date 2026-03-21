//! Traced mesh execution — identical routing logic to [`super::mesh::execute_creature_mesh`]
//! but records per-hop trace data for the Execution Sampler.
//!
//! **Maintenance note:** This module duplicates the mesh routing loop from `mesh.rs`
//! with trace recording. When updating mesh routing logic, apply the same changes
//! here and verify with equivalence tests.

use crate::config::RuntimeConfig;
use crate::contracts::{NodeId, WorldAction, MAX_GATE_SLOTS};
use crate::creature::genome::{BackendDef, CreatureGenome, NodeGenome};
use crate::creature::state::GraphRuntimeState;
use crate::runtime::cgp::execute_graph_node_traced;
use crate::runtime::routing::resolve_gated_route;
use crate::runtime::trace::domain::{
    BackendTrace, MeshHopTrace, TerminationReason, TraceGateScore, TraceRouteDecision,
};
use crate::runtime::traced_vm::execute_vm_node_traced;
use crate::runtime::types::{ComputeCostReport, MeshOutput, MeshSideOutputs, OUTPUT_SLOT_COUNT};
use crate::sensors::perception::SensorSnapshot;

/// Execute the creature's mesh chain with trace recording.
///
/// Identical routing behavior to [`super::mesh::execute_creature_mesh`] but
/// returns additional trace data: per-hop `MeshHopTrace` and `TerminationReason`.
#[allow(clippy::too_many_arguments)]
pub fn execute_creature_mesh_traced(
    genome: &CreatureGenome,
    sensors: &SensorSnapshot,
    energy: &mut f32,
    shared_memory: &mut [f32; 16],
    prev_shared_memory: &[f32; 16],
    graph_runtime: &mut GraphRuntimeState,
    config: &RuntimeConfig,
) -> (MeshOutput, Vec<MeshHopTrace>, TerminationReason) {
    let mut current_node_id = genome.entry_node_id;
    let mut upstream_slots = [0.0f32; OUTPUT_SLOT_COUNT];
    let mut hops: usize = 0;
    let max_hops = config.max_mesh_hops.max(1) as usize;
    let start_energy = *energy;
    let mut report = ComputeCostReport::default();

    let mut side_outputs = MeshSideOutputs::new(config.max_actions_per_turn);
    let mut hop_traces: Vec<MeshHopTrace> = Vec::with_capacity(max_hops);

    macro_rules! mesh_output {
        ($actions:expr) => {
            MeshOutput {
                actions: $actions,
                cost_report: report,
                priority_bid: side_outputs.priority_bid,
            }
        };
    }

    if find_node_index(&genome.nodes, current_node_id).is_none() {
        return (
            mesh_output!(vec![WorldAction::NoOp]),
            hop_traces,
            TerminationReason::MissingNode,
        );
    }

    loop {
        if hops >= max_hops {
            return (
                mesh_output!(side_outputs.action_queue.into_actions_or_noop()),
                hop_traces,
                TerminationReason::MaxHopsReached,
            );
        }

        let current_idx =
            find_node_index(&genome.nodes, current_node_id).expect("node must exist in genome");
        let node = &genome.nodes[current_idx];
        let energy_consumed = (start_energy - *energy).max(0.0);
        let node_energy_before = *energy;

        let (result, backend_trace) = match &node.backend_def {
            BackendDef::Vm(def) => {
                let (result, vm_trace) = execute_vm_node_traced(
                    def,
                    &node.input_refs,
                    &upstream_slots,
                    energy,
                    energy_consumed,
                    shared_memory,
                    prev_shared_memory,
                    sensors,
                    config,
                    &mut side_outputs,
                );
                (result, BackendTrace::Vm(vm_trace))
            }
            BackendDef::Graph(def) => {
                let (result, graph_trace) = execute_graph_node_traced(
                    def,
                    &node.input_refs,
                    &upstream_slots,
                    energy,
                    energy_consumed,
                    current_idx,
                    graph_runtime,
                    sensors,
                    config,
                    &mut side_outputs,
                    shared_memory,
                    prev_shared_memory,
                );
                (result, BackendTrace::Graph(graph_trace))
            }
        };

        let node_cost = (node_energy_before - *energy).max(0.0);
        match &node.backend_def {
            BackendDef::Vm(_) => report.vm_cost += node_cost,
            BackendDef::Graph(_) => report.graph_cost += node_cost,
        }

        // Resolve routing via per-target gate scoring.
        let route_result = resolve_gated_route(&node.targets, &result.route_gates);

        // Build trace route decision with per-target gate scores.
        let trace_route = route_result.map(|(winning_idx, id)| {
            let gate_scores: Vec<TraceGateScore> = node
                .targets
                .iter()
                .map(|t| {
                    let runtime = result.route_gates.score_for_slot(t.slot);
                    TraceGateScore {
                        slot: t.slot,
                        target_id: t.target_id,
                        gate_bias: t.gate_bias,
                        runtime_score: runtime,
                        effective_score: t.gate_bias + runtime,
                    }
                })
                .collect();
            TraceRouteDecision {
                gate_scores,
                selected_target_idx: winning_idx,
                selected_target_id: id,
            }
        });

        hop_traces.push(MeshHopTrace {
            hop_index: hops,
            node_id: current_node_id,
            input_refs: node.input_refs.clone(),
            upstream_slots,
            energy_before: node_energy_before,
            energy_after: *energy,
            output_slots: result.output_slots,
            route: trace_route,
            backend_trace,
        });

        if result.energy_exhausted {
            return (
                mesh_output!(vec![WorldAction::NoOp]),
                hop_traces,
                TerminationReason::EnergyExhausted,
            );
        }

        if result.terminal {
            return (
                mesh_output!(side_outputs.action_queue.into_actions_or_noop()),
                hop_traces,
                TerminationReason::ActionEmitted,
            );
        }

        match route_result {
            Some((_idx, id)) => {
                if find_node_index(&genome.nodes, id).is_none() {
                    return (
                        mesh_output!(side_outputs.action_queue.into_actions_or_noop()),
                        hop_traces,
                        TerminationReason::MissingNode,
                    );
                }
                upstream_slots = result.output_slots;
                current_node_id = id;
                hops += 1;
            }
            None => {
                return (
                    mesh_output!(side_outputs.action_queue.into_actions_or_noop()),
                    hop_traces,
                    TerminationReason::NoTargets,
                );
            }
        }
    }
}

/// Find the index of a node by its `NodeId` via linear scan.
///
/// For typical genomes (2-10 nodes), linear scan is faster than HashMap
/// due to cache locality and zero heap allocation.
#[inline]
fn find_node_index(nodes: &[NodeGenome], id: NodeId) -> Option<usize> {
    nodes.iter().position(|n| n.node_id == id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RuntimeConfig;
    use crate::contracts::{InputReference, NodeId, RouteTarget, WorldAction};
    use crate::creature::genome::cgp::{
        CgpGraphBackendDef, ComputeNode, ComputeNodeKind, ExecuteGate, GraphEdge, GraphSource,
        OutputSink, OutputSinkKind,
    };
    use crate::creature::genome::{
        BackendDef, CreatureGenome, HebbianRule, NodeGenome, PlasticityConfig, VmBackendDef,
        VmInstruction,
    };
    use crate::creature::state::GraphRuntimeState;
    use crate::runtime::mesh::execute_creature_mesh;
    use crate::runtime::trace::domain::BackendTrace;
    use crate::sensors::perception::{PerceptionSnapshot, SensorSnapshot};
    use crate::sensors::static_inputs::StaticInputs;

    fn default_config() -> RuntimeConfig {
        RuntimeConfig::default()
    }

    fn empty_ss() -> SensorSnapshot {
        SensorSnapshot {
            local: StaticInputs {
                food_here: 0.0,
                neighbor_food: [0.0; 8],
                neighbor_barrier: [0.0; 8],
                neighbor_occupied: [0.0; 8],
                generation: 0.0,
                age_ticks: 0.0,
            },
            perception: PerceptionSnapshot::zero(),
        }
    }

    fn wrap_targets(ids: Vec<NodeId>) -> Vec<RouteTarget> {
        ids.into_iter()
            .enumerate()
            .map(|(i, id)| RouteTarget {
                target_id: id,
                slot: i as u8,
                gate_bias: 0.0,
            })
            .collect()
    }

    fn vm_emit_node(node_id: NodeId, action_type: u8, targets: Vec<NodeId>) -> NodeGenome {
        NodeGenome {
            node_id,
            input_refs: vec![],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 1,
                constants: vec![],
                program: vec![
                    VmInstruction::PushAction { action_type },
                    VmInstruction::ExecuteActionQueue,
                ],
            }),
            targets: wrap_targets(targets),
        }
    }

    /// Multi-hop mesh (Graph → VM → Eat) produces same action and correct hop trace.
    #[test]
    fn multi_hop_graph_vm_eat() {
        let id_graph = NodeId::new(0);
        let id_vm = NodeId::new(1);

        // CGP graph: Constant(0.0) → RouterGate(0) sink (routes to target 0)
        let graph_node = NodeGenome {
            node_id: id_graph,
            input_refs: vec![],
            backend_def: BackendDef::Graph(CgpGraphBackendDef {
                compute_nodes: vec![ComputeNode {
                    kind: ComputeNodeKind::Constant(0.0),
                    inputs: vec![],
                    plasticity: None,
                }],
                output_sinks: vec![OutputSink {
                    kind: OutputSinkKind::RouterGate(0),
                    inputs: vec![GraphEdge {
                        source: GraphSource::ComputeNode(0),
                        weight: 1.0,
                    }],
                }],
                action_bank: vec![],
                execute_gate: ExecuteGate { inputs: vec![] },
            }),
            targets: wrap_targets(vec![id_vm]),
        };

        let vm_node = vm_emit_node(id_vm, 1, vec![]);

        let genome = CreatureGenome {
            entry_node_id: id_graph,
            nodes: vec![graph_node, vm_node],
        };
        let ss = empty_ss();
        let config = default_config();

        // Run non-traced
        let mut energy_a = 100.0f32;
        let mut smem_a = [0.0f32; 16];
        let prev_a = [0.0f32; 16];
        let mut gr_a = GraphRuntimeState::new();
        let output_a = execute_creature_mesh(
            &genome,
            &ss,
            &mut energy_a,
            &mut smem_a,
            &prev_a,
            &mut gr_a,
            &config,
        );

        // Run traced
        let mut energy_b = 100.0f32;
        let mut smem_b = [0.0f32; 16];
        let prev_b = [0.0f32; 16];
        let mut gr_b = GraphRuntimeState::new();
        let (output_b, hops, reason) = execute_creature_mesh_traced(
            &genome,
            &ss,
            &mut energy_b,
            &mut smem_b,
            &prev_b,
            &mut gr_b,
            &config,
        );

        assert_eq!(output_a.actions, output_b.actions);
        assert_eq!(output_b.actions, vec![WorldAction::Eat]);
        assert!(
            (energy_a - energy_b).abs() < 1e-6,
            "energy: {energy_a} vs {energy_b}"
        );
        assert!((output_a.cost_report.vm_cost - output_b.cost_report.vm_cost).abs() < 1e-6);
        assert!((output_a.cost_report.graph_cost - output_b.cost_report.graph_cost).abs() < 1e-6);

        // 2 hops: Graph(hop 0) → VM(hop 1)
        assert_eq!(hops.len(), 2);
        assert_eq!(hops[0].node_id, id_graph);
        assert!(matches!(hops[0].backend_trace, BackendTrace::Graph(_)));
        assert_eq!(hops[1].node_id, id_vm);
        assert!(matches!(hops[1].backend_trace, BackendTrace::Vm(_)));
        assert!(matches!(reason, TerminationReason::ActionEmitted));
    }

    /// Upstream slots correctly propagated between hops in trace.
    #[test]
    fn upstream_slots_propagated() {
        let id_graph = NodeId::new(0);
        let id_vm = NodeId::new(1);

        // CGP graph writes 9.0 to CustomOutput(5), RouterGate(0) unwired (default route)
        let graph_node = NodeGenome {
            node_id: id_graph,
            input_refs: vec![],
            backend_def: BackendDef::Graph(CgpGraphBackendDef {
                compute_nodes: vec![ComputeNode {
                    kind: ComputeNodeKind::Constant(9.0),
                    inputs: vec![],
                    plasticity: None,
                }],
                output_sinks: vec![
                    OutputSink {
                        kind: OutputSinkKind::CustomOutput(5),
                        inputs: vec![GraphEdge {
                            source: GraphSource::ComputeNode(0),
                            weight: 1.0,
                        }],
                    },
                    OutputSink {
                        kind: OutputSinkKind::RouterGate(0),
                        inputs: vec![], // unwired = default route
                    },
                ],
                action_bank: vec![],
                execute_gate: ExecuteGate { inputs: vec![] },
            }),
            targets: wrap_targets(vec![id_vm]),
        };

        // VM reads upstream slot 5
        let vm_node = NodeGenome {
            node_id: id_vm,
            input_refs: vec![InputReference::UpstreamSlot(5)],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 1,
                constants: vec![],
                program: vec![
                    VmInstruction::ReadInput {
                        dst: 0,
                        ref_idx: 0,
                        sub_idx: 0,
                    },
                    VmInstruction::PushAction { action_type: 1 },
                    VmInstruction::ExecuteActionQueue,
                ],
            }),
            targets: vec![],
        };

        let genome = CreatureGenome {
            entry_node_id: id_graph,
            nodes: vec![graph_node, vm_node],
        };
        let ss = empty_ss();
        let config = default_config();
        let mut energy = 1000.0f32;
        let mut smem = [0.0f32; 16];
        let prev_smem = [0.0f32; 16];
        let mut gr = GraphRuntimeState::new();

        let (output, hops, _) = execute_creature_mesh_traced(
            &genome,
            &ss,
            &mut energy,
            &mut smem,
            &prev_smem,
            &mut gr,
            &config,
        );

        assert_eq!(output.actions, vec![WorldAction::Eat]);
        // Hop 1 (VM) should have upstream_slots[5] = 9.0
        assert!((hops[1].upstream_slots[5] - 9.0).abs() < 1e-5);
    }

    #[test]
    fn graph_effect_trace_captures_sinks_action_bank_and_execute_gate() {
        let id0 = NodeId::new(0);
        let genome = CreatureGenome {
            entry_node_id: id0,
            nodes: vec![NodeGenome {
                node_id: id0,
                input_refs: vec![],
                backend_def: BackendDef::Graph(CgpGraphBackendDef {
                    compute_nodes: vec![ComputeNode {
                        kind: ComputeNodeKind::Constant(1.0),
                        inputs: vec![],
                        plasticity: None,
                    }],
                    output_sinks: vec![
                        OutputSink {
                            kind: OutputSinkKind::CustomOutput(0),
                            inputs: vec![GraphEdge {
                                source: GraphSource::ComputeNode(0),
                                weight: 1.0,
                            }],
                        },
                        OutputSink {
                            kind: OutputSinkKind::RouterGate(0),
                            inputs: vec![],
                        },
                    ],
                    action_bank: vec![
                        crate::creature::genome::cgp::ActionSlot {
                            behavior: crate::creature::genome::cgp::ActionSlotBehavior::Emit(
                                crate::creature::genome::cgp::WorldActionKind::Eat,
                            ),
                            gate_inputs: vec![GraphEdge {
                                source: GraphSource::ComputeNode(0),
                                weight: 1.0,
                            }],
                            param_inputs: vec![],
                        },
                        crate::creature::genome::cgp::ActionSlot {
                            behavior: crate::creature::genome::cgp::ActionSlotBehavior::Pop,
                            gate_inputs: vec![GraphEdge {
                                source: GraphSource::ComputeNode(0),
                                weight: 1.0,
                            }],
                            param_inputs: vec![],
                        },
                    ],
                    execute_gate: ExecuteGate {
                        inputs: vec![GraphEdge {
                            source: GraphSource::ComputeNode(0),
                            weight: 1.0,
                        }],
                    },
                }),
                targets: vec![],
            }],
        };

        let ss = empty_ss();
        let mut energy = 100.0f32;
        let mut smem = [0.0f32; 16];
        let prev_smem = [0.0f32; 16];
        let mut gr = GraphRuntimeState::new();
        let config = default_config();

        let (output, hops, reason) = execute_creature_mesh_traced(
            &genome,
            &ss,
            &mut energy,
            &mut smem,
            &prev_smem,
            &mut gr,
            &config,
        );

        assert_eq!(hops.len(), 1);
        assert!(matches!(reason, TerminationReason::NoTargets));
        assert_eq!(output.actions, vec![WorldAction::NoOp]);

        let BackendTrace::Graph(graph) = &hops[0].backend_trace else {
            panic!("expected graph backend trace");
        };

        assert_eq!(graph.output_sinks.len(), 2);
        assert!(graph.output_sinks[0].wired);
        assert!(graph.output_sinks[0].applied);
        assert!((graph.output_sinks[0].applied_value - 1.0).abs() < 1e-6);
        assert!(!graph.output_sinks[1].wired);
        assert!(!graph.output_sinks[1].applied);

        assert_eq!(graph.action_slots.len(), 2);
        assert!(graph.action_slots[0].wired);
        assert!(graph.action_slots[0].fired);
        assert_eq!(graph.action_slots[0].queue_len_before, 0);
        assert_eq!(graph.action_slots[0].queue_len_after, 1);
        assert_eq!(graph.action_slots[0].emitted_action, Some(WorldAction::Eat));
        assert!(graph.action_slots[1].wired);
        assert!(graph.action_slots[1].fired);
        assert_eq!(graph.action_slots[1].queue_len_before, 1);
        assert_eq!(graph.action_slots[1].queue_len_after, 0);
        assert!(graph.action_slots[1].emitted_action.is_none());

        assert!(graph.execute_gate.wired);
        assert!(!graph.execute_gate.queue_non_empty);
        assert!(!graph.execute_gate.fired);
    }

    #[test]
    fn graph_effect_trace_execute_gate_fires_only_with_non_empty_queue() {
        let id0 = NodeId::new(0);
        let genome = CreatureGenome {
            entry_node_id: id0,
            nodes: vec![NodeGenome {
                node_id: id0,
                input_refs: vec![],
                backend_def: BackendDef::Graph(CgpGraphBackendDef {
                    compute_nodes: vec![ComputeNode {
                        kind: ComputeNodeKind::Constant(1.0),
                        inputs: vec![],
                        plasticity: None,
                    }],
                    output_sinks: vec![],
                    action_bank: vec![crate::creature::genome::cgp::ActionSlot {
                        behavior: crate::creature::genome::cgp::ActionSlotBehavior::Emit(
                            crate::creature::genome::cgp::WorldActionKind::Eat,
                        ),
                        gate_inputs: vec![GraphEdge {
                            source: GraphSource::ComputeNode(0),
                            weight: 1.0,
                        }],
                        param_inputs: vec![],
                    }],
                    execute_gate: ExecuteGate {
                        inputs: vec![GraphEdge {
                            source: GraphSource::ComputeNode(0),
                            weight: 1.0,
                        }],
                    },
                }),
                targets: vec![],
            }],
        };

        let ss = empty_ss();
        let mut energy = 100.0f32;
        let mut smem = [0.0f32; 16];
        let prev_smem = [0.0f32; 16];
        let mut gr = GraphRuntimeState::new();
        let config = default_config();

        let (output, hops, reason) = execute_creature_mesh_traced(
            &genome,
            &ss,
            &mut energy,
            &mut smem,
            &prev_smem,
            &mut gr,
            &config,
        );

        assert_eq!(hops.len(), 1);
        assert!(matches!(reason, TerminationReason::ActionEmitted));
        assert_eq!(output.actions, vec![WorldAction::Eat]);

        let BackendTrace::Graph(graph) = &hops[0].backend_trace else {
            panic!("expected graph backend trace");
        };

        assert_eq!(graph.output_sinks.len(), 0);
        assert_eq!(graph.action_slots.len(), 1);
        assert!(graph.action_slots[0].fired);
        assert_eq!(graph.action_slots[0].queue_len_before, 0);
        assert_eq!(graph.action_slots[0].queue_len_after, 1);
        assert_eq!(graph.action_slots[0].emitted_action, Some(WorldAction::Eat));
        assert!(graph.execute_gate.wired);
        assert!(graph.execute_gate.queue_non_empty);
        assert!(graph.execute_gate.fired);
    }

    #[test]
    fn graph_effect_trace_sanitizes_non_finite_values() {
        let id0 = NodeId::new(0);
        let genome = CreatureGenome {
            entry_node_id: id0,
            nodes: vec![NodeGenome {
                node_id: id0,
                input_refs: vec![],
                backend_def: BackendDef::Graph(CgpGraphBackendDef {
                    compute_nodes: vec![ComputeNode {
                        kind: ComputeNodeKind::Constant(1.0),
                        inputs: vec![],
                        plasticity: None,
                    }],
                    output_sinks: vec![OutputSink {
                        kind: OutputSinkKind::CustomOutput(0),
                        inputs: vec![GraphEdge {
                            source: GraphSource::ComputeNode(0),
                            weight: f32::NAN,
                        }],
                    }],
                    action_bank: vec![crate::creature::genome::cgp::ActionSlot {
                        behavior: crate::creature::genome::cgp::ActionSlotBehavior::Emit(
                            crate::creature::genome::cgp::WorldActionKind::Move,
                        ),
                        gate_inputs: vec![GraphEdge {
                            source: GraphSource::ComputeNode(0),
                            weight: 1.0,
                        }],
                        param_inputs: vec![GraphEdge {
                            source: GraphSource::ComputeNode(0),
                            weight: f32::INFINITY,
                        }],
                    }],
                    execute_gate: ExecuteGate {
                        inputs: vec![GraphEdge {
                            source: GraphSource::ComputeNode(0),
                            weight: f32::NAN,
                        }],
                    },
                }),
                targets: vec![],
            }],
        };

        let ss = empty_ss();
        let mut energy = 100.0f32;
        let mut smem = [0.0f32; 16];
        let prev_smem = [0.0f32; 16];
        let mut gr = GraphRuntimeState::new();
        let config = default_config();

        let (_output, hops, _reason) = execute_creature_mesh_traced(
            &genome,
            &ss,
            &mut energy,
            &mut smem,
            &prev_smem,
            &mut gr,
            &config,
        );

        let BackendTrace::Graph(graph) = &hops[0].backend_trace else {
            panic!("expected graph backend trace");
        };

        assert_eq!(graph.output_sinks.len(), 1);
        assert_eq!(graph.output_sinks[0].weighted_sum, 0.0);
        assert_eq!(graph.output_sinks[0].applied_value, 0.0);

        assert_eq!(graph.action_slots.len(), 1);
        assert_eq!(graph.action_slots[0].param_values[0], 1_000_000_000.0);
        assert!(graph.action_slots[0].gate_weighted_sum.is_finite());

        assert!(graph.execute_gate.weighted_sum.is_finite());
    }

    /// Energy exhaustion captured with correct termination reason.
    #[test]
    fn energy_exhaustion_termination() {
        let id0 = NodeId::new(0);
        let genome = CreatureGenome {
            entry_node_id: id0,
            nodes: vec![NodeGenome {
                node_id: id0,
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![],
                    program: vec![VmInstruction::Noop, VmInstruction::Halt],
                }),
                targets: vec![],
            }],
        };
        let ss = empty_ss();
        // Boost opcode cost multiplier so Noop actually exhausts energy
        let mut config = default_config();
        config.vm.opcode_cost_multiplier = 1.0;
        let mut energy = 0.01f32;
        let mut smem = [0.0f32; 16];
        let prev_smem = [0.0f32; 16];
        let mut gr = GraphRuntimeState::new();

        let (output, hops, reason) = execute_creature_mesh_traced(
            &genome,
            &ss,
            &mut energy,
            &mut smem,
            &prev_smem,
            &mut gr,
            &config,
        );

        assert_eq!(output.actions, vec![WorldAction::NoOp]);
        assert!(
            matches!(reason, TerminationReason::EnergyExhausted),
            "expected EnergyExhausted, got {:?}",
            reason
        );
        assert_eq!(hops.len(), 1);
    }

    #[test]
    fn graph_energy_exhaustion_traces_cgp_route_kind() {
        let id0 = NodeId::new(0);
        let genome = CreatureGenome {
            entry_node_id: id0,
            nodes: vec![NodeGenome {
                node_id: id0,
                input_refs: vec![],
                backend_def: BackendDef::Graph(CgpGraphBackendDef {
                    compute_nodes: vec![ComputeNode {
                        kind: ComputeNodeKind::Constant(1.0),
                        inputs: vec![],
                        plasticity: None,
                    }],
                    output_sinks: vec![],
                    action_bank: vec![],
                    execute_gate: ExecuteGate { inputs: vec![] },
                }),
                targets: vec![],
            }],
        };

        let ss = empty_ss();
        let mut config = default_config();
        config.graph_node_base_cost = 1.0;
        let mut energy = 0.1f32;
        let mut smem = [0.0f32; 16];
        let prev_smem = [0.0f32; 16];
        let mut gr = GraphRuntimeState::new();

        let (output, hops, reason) = execute_creature_mesh_traced(
            &genome,
            &ss,
            &mut energy,
            &mut smem,
            &prev_smem,
            &mut gr,
            &config,
        );

        assert_eq!(output.actions, vec![WorldAction::NoOp]);
        assert!(matches!(reason, TerminationReason::EnergyExhausted));
        assert_eq!(hops.len(), 1);
        // No targets on this node, so route is None.
        assert!(hops[0].route.is_none());
    }

    #[test]
    fn graph_plasticity_cost_exhaustion_stops_effects() {
        let id0 = NodeId::new(0);
        let genome = CreatureGenome {
            entry_node_id: id0,
            nodes: vec![NodeGenome {
                node_id: id0,
                input_refs: vec![],
                backend_def: BackendDef::Graph(CgpGraphBackendDef {
                    compute_nodes: vec![ComputeNode {
                        kind: ComputeNodeKind::Constant(1.0),
                        inputs: vec![GraphEdge {
                            source: GraphSource::SharedMemory {
                                slot: 0,
                                previous: false,
                            },
                            weight: 1.0,
                        }],
                        plasticity: Some(PlasticityConfig {
                            rule: HebbianRule::Classic,
                            learning_rate: 0.5,
                            weight_clamp: 1.0,
                            lamarckian: false,
                            modulation: None,
                        }),
                    }],
                    output_sinks: vec![],
                    action_bank: vec![crate::creature::genome::cgp::ActionSlot {
                        behavior: crate::creature::genome::cgp::ActionSlotBehavior::Emit(
                            crate::creature::genome::cgp::WorldActionKind::Eat,
                        ),
                        gate_inputs: vec![GraphEdge {
                            source: GraphSource::ComputeNode(0),
                            weight: 1.0,
                        }],
                        param_inputs: vec![],
                    }],
                    execute_gate: ExecuteGate {
                        inputs: vec![GraphEdge {
                            source: GraphSource::ComputeNode(0),
                            weight: 1.0,
                        }],
                    },
                }),
                targets: vec![],
            }],
        };

        let ss = empty_ss();
        let mut config = default_config();
        config.graph_node_base_cost = 0.1;
        config.plasticity_update_cost = 2.0;
        let mut energy = 1.0f32;
        let mut smem = [0.0f32; 16];
        let prev_smem = [0.0f32; 16];
        let mut gr = GraphRuntimeState::new();

        let (output, hops, reason) = execute_creature_mesh_traced(
            &genome,
            &ss,
            &mut energy,
            &mut smem,
            &prev_smem,
            &mut gr,
            &config,
        );

        assert_eq!(output.actions, vec![WorldAction::NoOp]);
        assert!(matches!(reason, TerminationReason::EnergyExhausted));
        assert_eq!(hops.len(), 1);
        // No targets on this node, so route is None.
        assert!(hops[0].route.is_none());
    }

    /// Traced and non-traced paths produce identical MeshOutput when priority bid is used.
    #[test]
    fn traced_and_untraced_priority_bid_equivalence() {
        use crate::runtime::mesh::execute_creature_mesh;

        let id0 = NodeId::new(0);
        // Genome that loads 3.0, sets priority bid, then emits Eat.
        let genome = CreatureGenome {
            entry_node_id: id0,
            nodes: vec![NodeGenome {
                node_id: id0,
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![3.0],
                    program: vec![
                        VmInstruction::LoadConst {
                            dst: 0,
                            const_idx: 0,
                        },
                        VmInstruction::SetPriorityBid { src: 0 },
                        VmInstruction::PushAction { action_type: 1 },
                        VmInstruction::ExecuteActionQueue,
                    ],
                }),
                targets: vec![],
            }],
        };

        let ss = empty_ss();
        let config = default_config();

        // Run non-traced path.
        let mut energy_a = 100.0f32;
        let mut smem_a = [0.0f32; 16];
        let prev_a = [0.0f32; 16];
        let mut gr_a = GraphRuntimeState::new();
        let output_a = execute_creature_mesh(
            &genome,
            &ss,
            &mut energy_a,
            &mut smem_a,
            &prev_a,
            &mut gr_a,
            &config,
        );

        // Run traced path.
        let mut energy_b = 100.0f32;
        let mut smem_b = [0.0f32; 16];
        let prev_b = [0.0f32; 16];
        let mut gr_b = GraphRuntimeState::new();
        let (output_b, _, _) = execute_creature_mesh_traced(
            &genome,
            &ss,
            &mut energy_b,
            &mut smem_b,
            &prev_b,
            &mut gr_b,
            &config,
        );

        // Assert full MeshOutput equivalence.
        assert_eq!(output_a.actions, output_b.actions, "actions must match");
        assert!(
            (output_a.cost_report.vm_cost - output_b.cost_report.vm_cost).abs() < 1e-6,
            "VM cost must match: {} vs {}",
            output_a.cost_report.vm_cost,
            output_b.cost_report.vm_cost
        );
        assert_eq!(
            output_a.priority_bid, output_b.priority_bid,
            "priority_bid must match: {} vs {}",
            output_a.priority_bid, output_b.priority_bid
        );
        assert_eq!(output_a.priority_bid, 3.0, "bid should be 3.0");
        assert!(
            (energy_a - energy_b).abs() < 1e-6,
            "final energy must match: {} vs {}",
            energy_a,
            energy_b
        );
    }
}
