//! Traced mesh execution — identical routing logic to [`super::mesh::execute_creature_mesh`]
//! but records per-hop trace data for the Execution Sampler.
//!
//! **Maintenance note:** This module duplicates the mesh routing loop from `mesh.rs`
//! with trace recording. When updating mesh routing logic, apply the same changes
//! here and verify with equivalence tests.

use std::collections::HashMap;

use crate::config::RuntimeConfig;
use crate::contracts::{NodeId, WorldAction};
use crate::creature::genome::{BackendDef, CreatureGenome};
use crate::creature::state::GraphRuntimeState;
use crate::runtime::trace::{BackendTrace, MeshHopTrace, TerminationReason};
use crate::runtime::traced_graph::execute_graph_node_traced;
use crate::runtime::traced_vm::execute_vm_node_traced;
use crate::runtime::types::ComputeCostReport;
use crate::sensors::static_inputs::StaticInputs;

/// Execute the creature's mesh chain with trace recording.
///
/// Identical routing behavior to [`super::mesh::execute_creature_mesh`] but
/// returns additional trace data: per-hop `MeshHopTrace` and `TerminationReason`.
#[allow(clippy::too_many_arguments)]
pub fn execute_creature_mesh_traced(
    genome: &CreatureGenome,
    static_inputs: &StaticInputs,
    energy: &mut f32,
    memory: &mut [u8; 1024],
    graph_runtime: &mut GraphRuntimeState,
    config: &RuntimeConfig,
) -> (
    WorldAction,
    ComputeCostReport,
    Vec<MeshHopTrace>,
    TerminationReason,
) {
    let mut current_node_id = genome.entry_node_id;
    let mut upstream_slots = [0.0f32; 12];
    let mut hops: usize = 0;
    let max_hops = config.max_mesh_hops.max(1) as usize;
    let start_energy = *energy;
    let mut report = ComputeCostReport::default();

    let node_index: HashMap<NodeId, usize> = genome
        .nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (n.node_id, i))
        .collect();

    let mut hop_traces: Vec<MeshHopTrace> = Vec::with_capacity(max_hops);

    if !node_index.contains_key(&current_node_id) {
        return (
            WorldAction::NoOp,
            report,
            hop_traces,
            TerminationReason::MissingNode,
        );
    }

    loop {
        if hops >= max_hops {
            return (
                WorldAction::NoOp,
                report,
                hop_traces,
                TerminationReason::MaxHopsReached,
            );
        }

        let node = &genome.nodes[node_index[&current_node_id]];
        let energy_consumed = (start_energy - *energy).max(0.0);
        let node_energy_before = *energy;
        let current_idx = node_index[&current_node_id];

        let (result, backend_trace) = match &node.backend_def {
            BackendDef::Vm(def) => {
                let (result, vm_trace) = execute_vm_node_traced(
                    def,
                    &node.input_refs,
                    &upstream_slots,
                    energy,
                    energy_consumed,
                    memory,
                    static_inputs,
                    config,
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
                    static_inputs,
                    config,
                );
                (result, BackendTrace::Graph(graph_trace))
            }
        };

        let node_cost = (node_energy_before - *energy).max(0.0);
        match &node.backend_def {
            BackendDef::Vm(_) => report.vm_cost += node_cost,
            BackendDef::Graph(_) => report.graph_cost += node_cost,
        }

        hop_traces.push(MeshHopTrace {
            hop_index: hops,
            node_id: current_node_id,
            input_refs: node.input_refs.clone(),
            upstream_slots,
            energy_before: node_energy_before,
            energy_after: *energy,
            output_slots: result.output_slots,
            route_target_idx: result.route_target_idx,
            backend_trace,
        });

        if result.energy_exhausted {
            return (
                WorldAction::NoOp,
                report,
                hop_traces,
                TerminationReason::EnergyExhausted,
            );
        }

        if let Some(action) = result.world_action {
            return (action, report, hop_traces, TerminationReason::ActionEmitted);
        }

        if node.targets.is_empty() {
            return (
                WorldAction::NoOp,
                report,
                hop_traces,
                TerminationReason::NoTargets,
            );
        }

        let route_target_idx = result.route_target_idx;
        let route_idx_i64: i64 = if route_target_idx.is_nan() {
            -1
        } else if route_target_idx == f32::INFINITY {
            i64::MAX
        } else if route_target_idx == f32::NEG_INFINITY {
            i64::MIN
        } else {
            route_target_idx
                .clamp(i64::MIN as f32, i64::MAX as f32)
                .floor() as i64
        };

        let target_pos = route_idx_i64.rem_euclid(node.targets.len() as i64) as usize;
        let target_id = node.targets[target_pos];

        if !node_index.contains_key(&target_id) {
            return (
                WorldAction::NoOp,
                report,
                hop_traces,
                TerminationReason::MissingNode,
            );
        }

        upstream_slots = result.output_slots;
        current_node_id = target_id;
        hops += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RuntimeConfig;
    use crate::contracts::{InputReference, NodeId, WorldAction};
    use crate::creature::genome::{
        BackendDef, CreatureGenome, GraphBackendDef, GraphInput, GraphInternalNode, GraphNodeKind,
        NodeGenome, VmBackendDef, VmInstruction,
    };
    use crate::creature::state::GraphRuntimeState;
    use crate::runtime::mesh::execute_creature_mesh;
    use crate::sensors::static_inputs::StaticInputs;

    fn default_config() -> RuntimeConfig {
        RuntimeConfig::default()
    }

    fn empty_si() -> StaticInputs {
        StaticInputs {
            food_here: 0.0,
            neighbor_food: [0.0; 8],
            neighbor_barrier: [0.0; 8],
            neighbor_occupied: [0.0; 8],
            generation: 0.0,
            age_ticks: 0.0,
        }
    }

    fn vm_emit_node(node_id: NodeId, action_type: u8, targets: Vec<NodeId>) -> NodeGenome {
        NodeGenome {
            node_id,
            input_refs: vec![],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 1,
                constants: vec![],
                program: vec![VmInstruction::EmitWorldAction { action_type }],
            }),
            targets,
        }
    }

    /// Multi-hop mesh (Graph → VM → Eat) produces same action and correct hop trace.
    #[test]
    fn multi_hop_graph_vm_eat() {
        let id_graph = NodeId::new(0);
        let id_vm = NodeId::new(1);

        let graph_node = NodeGenome {
            node_id: id_graph,
            input_refs: vec![],
            backend_def: BackendDef::Graph(GraphBackendDef {
                internal_nodes: vec![
                    GraphInternalNode {
                        kind: GraphNodeKind::Constant(0.0),
                        inputs: vec![],
                        hebbian: None,
                    },
                    GraphInternalNode {
                        kind: GraphNodeKind::RouterOutput,
                        inputs: vec![GraphInput {
                            source_idx: 0,
                            weight: 1.0,
                        }],
                        hebbian: None,
                    },
                ],
            }),
            targets: vec![id_vm],
        };

        let vm_node = vm_emit_node(id_vm, 1, vec![]);

        let genome = CreatureGenome {
            entry_node_id: id_graph,
            nodes: vec![graph_node, vm_node],
        };
        let si = empty_si();
        let config = default_config();

        // Run non-traced
        let mut energy_a = 100.0f32;
        let mut memory_a = [0u8; 1024];
        let mut gr_a = GraphRuntimeState::new();
        let (action_a, report_a) = execute_creature_mesh(
            &genome,
            &si,
            &mut energy_a,
            &mut memory_a,
            &mut gr_a,
            &config,
        );

        // Run traced
        let mut energy_b = 100.0f32;
        let mut memory_b = [0u8; 1024];
        let mut gr_b = GraphRuntimeState::new();
        let (action_b, report_b, hops, reason) = execute_creature_mesh_traced(
            &genome,
            &si,
            &mut energy_b,
            &mut memory_b,
            &mut gr_b,
            &config,
        );

        assert_eq!(action_a, action_b);
        assert_eq!(action_b, WorldAction::Eat);
        assert!(
            (energy_a - energy_b).abs() < 1e-6,
            "energy: {energy_a} vs {energy_b}"
        );
        assert!((report_a.vm_cost - report_b.vm_cost).abs() < 1e-6);
        assert!((report_a.graph_cost - report_b.graph_cost).abs() < 1e-6);

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

        // Graph writes 9.0 to slot 5
        let graph_node = NodeGenome {
            node_id: id_graph,
            input_refs: vec![],
            backend_def: BackendDef::Graph(GraphBackendDef {
                internal_nodes: vec![
                    GraphInternalNode {
                        kind: GraphNodeKind::Constant(9.0),
                        inputs: vec![],
                        hebbian: None,
                    },
                    GraphInternalNode {
                        kind: GraphNodeKind::CustomOutput(5),
                        inputs: vec![GraphInput {
                            source_idx: 0,
                            weight: 1.0,
                        }],
                        hebbian: None,
                    },
                    GraphInternalNode {
                        kind: GraphNodeKind::RouterOutput,
                        inputs: vec![],
                        hebbian: None,
                    },
                ],
            }),
            targets: vec![id_vm],
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
                    VmInstruction::EmitWorldAction { action_type: 1 },
                ],
            }),
            targets: vec![],
        };

        let genome = CreatureGenome {
            entry_node_id: id_graph,
            nodes: vec![graph_node, vm_node],
        };
        let si = empty_si();
        let config = default_config();
        let mut energy = 1000.0f32;
        let mut memory = [0u8; 1024];
        let mut gr = GraphRuntimeState::new();

        let (action, _, hops, _) =
            execute_creature_mesh_traced(&genome, &si, &mut energy, &mut memory, &mut gr, &config);

        assert_eq!(action, WorldAction::Eat);
        // Hop 1 (VM) should have upstream_slots[5] = 9.0
        assert!((hops[1].upstream_slots[5] - 9.0).abs() < 1e-5);
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
        let si = empty_si();
        // Boost opcode cost multiplier so Noop actually exhausts energy
        let mut config = default_config();
        config.vm.opcode_cost_multiplier = 1.0;
        let mut energy = 0.01f32;
        let mut memory = [0u8; 1024];
        let mut gr = GraphRuntimeState::new();

        let (action, _, hops, reason) =
            execute_creature_mesh_traced(&genome, &si, &mut energy, &mut memory, &mut gr, &config);

        assert_eq!(action, WorldAction::NoOp);
        assert!(
            matches!(reason, TerminationReason::EnergyExhausted),
            "expected EnergyExhausted, got {:?}",
            reason
        );
        assert_eq!(hops.len(), 1);
    }
}
