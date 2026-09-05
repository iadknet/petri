//! Structural companions: what a sampled genome's reachable mesh actually
//! contains, so a zero neighborhood reading can be attributed to absent
//! structure, inert (unreachable) structure, or a probe blind spot rather
//! than assumed to mean "no sensitivity" (T11.F01 Battery, "Structural
//! companions").

use crate::creature::genome::analysis::{functional_complexity, mesh_reachable_nodes};
use crate::creature::genome::cgp::{GraphSource, NodeClass, OutputSinkKind};
use crate::creature::genome::{BackendDef, CreatureGenome, VmInstruction};

/// Structural facts about a genome's reachable mesh, independent of any
/// battery execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructuralCompanions {
    pub functional_complexity: u32,
    pub reachable_node_count: usize,
    /// Any reachable VM instruction or graph edge reads shared memory
    /// (current or previous tick).
    pub reads_shared_memory: bool,
    /// Any reachable VM instruction or graph sink writes (or clears) shared
    /// memory.
    pub writes_shared_memory: bool,
    /// Any reachable graph compute node's kind is one of the stateful kinds
    /// (`DecayIntegrator`, `Momentum`, `Oscillator`, `AdaptiveGain`).
    pub has_stateful_compute_node: bool,
    /// Any reachable graph compute node carries a plasticity rule.
    pub has_plasticity: bool,
}

/// Derive the structural companions for `genome`. Scans every instruction,
/// edge, and sink of every mesh-reachable node (mesh-level reachability, not
/// VM-internal instruction liveness, which T11.F02 owns).
#[must_use]
pub fn structural_companions(genome: &CreatureGenome) -> StructuralCompanions {
    let reachable = mesh_reachable_nodes(genome);
    let mut reads_shared_memory = false;
    let mut writes_shared_memory = false;
    let mut has_stateful_compute_node = false;
    let mut has_plasticity = false;

    for &node_idx in &reachable {
        let Some(node) = genome.nodes.get(node_idx) else {
            continue;
        };
        match &node.backend_def {
            BackendDef::Vm(vm) => {
                for instruction in &vm.program {
                    match instruction {
                        VmInstruction::LoadSlot { .. }
                        | VmInstruction::LoadSlotImm { .. }
                        | VmInstruction::LoadSlotPrev { .. } => reads_shared_memory = true,
                        VmInstruction::StoreSlot { .. }
                        | VmInstruction::StoreSlotImm { .. }
                        | VmInstruction::ClearSlot { .. } => writes_shared_memory = true,
                        _ => {}
                    }
                }
            }
            BackendDef::Graph(graph) => {
                for compute_node in &graph.compute_nodes {
                    if compute_node.plasticity.is_some() {
                        has_plasticity = true;
                    }
                    if compute_node.kind.class() == NodeClass::Stateful {
                        has_stateful_compute_node = true;
                    }
                    for edge in &compute_node.inputs {
                        if matches!(edge.source, GraphSource::SharedMemory { .. }) {
                            reads_shared_memory = true;
                        }
                    }
                }
                for sink in &graph.output_sinks {
                    if sink.inputs.is_empty() {
                        continue;
                    }
                    if matches!(sink.kind, OutputSinkKind::WriteSlot(_) | OutputSinkKind::ClearSlot(_))
                    {
                        writes_shared_memory = true;
                    }
                }
            }
        }
    }

    StructuralCompanions {
        functional_complexity: functional_complexity(genome),
        reachable_node_count: reachable.len(),
        reads_shared_memory,
        writes_shared_memory,
        has_stateful_compute_node,
        has_plasticity,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::MutationConfig;
    use crate::contracts::NodeId;
    use crate::creature::genome::cgp::{CgpGraphBackendDef, ComputeNode, ComputeNodeKind, GraphEdge};
    use crate::creature::genome::{HebbianRule, NodeGenome, PlasticityConfig, VmBackendDef};

    fn vm_genome(program: Vec<VmInstruction>) -> CreatureGenome {
        CreatureGenome {
            entry_node_id: NodeId::new(1),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(1),
                input_refs: vec![],
                targets: vec![],
                backend_def: BackendDef::Vm(VmBackendDef { register_count: 2, constants: vec![], program }),
            }],
        }
    }

    fn graph_genome(graph: CgpGraphBackendDef) -> CreatureGenome {
        CreatureGenome {
            entry_node_id: NodeId::new(1),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(1),
                input_refs: vec![],
                targets: vec![],
                backend_def: BackendDef::Graph(graph),
            }],
        }
    }

    #[test]
    fn a_vm_genome_with_no_memory_instructions_reads_and_writes_nothing() {
        let genome = vm_genome(vec![VmInstruction::Noop, VmInstruction::Halt]);
        let companions = structural_companions(&genome);
        assert!(!companions.reads_shared_memory);
        assert!(!companions.writes_shared_memory);
        assert!(!companions.has_stateful_compute_node);
        assert!(!companions.has_plasticity);
    }

    #[test]
    fn a_vm_genome_with_load_slot_reads_shared_memory() {
        let genome = vm_genome(vec![VmInstruction::LoadSlotImm { dst: 0, slot_idx: 3 }]);
        let companions = structural_companions(&genome);
        assert!(companions.reads_shared_memory);
        assert!(!companions.writes_shared_memory);
    }

    #[test]
    fn a_vm_genome_with_store_slot_writes_shared_memory() {
        let genome = vm_genome(vec![VmInstruction::StoreSlotImm { slot_idx: 3, src: 0 }]);
        let companions = structural_companions(&genome);
        assert!(!companions.reads_shared_memory);
        assert!(companions.writes_shared_memory);
    }

    #[test]
    fn a_vm_genome_with_clear_slot_writes_shared_memory() {
        let genome = vm_genome(vec![VmInstruction::ClearSlot { slot_idx: 2 }]);
        assert!(structural_companions(&genome).writes_shared_memory);
    }

    #[test]
    fn a_graph_genome_with_no_stateful_structure_reports_nothing() {
        let graph = CgpGraphBackendDef::new_with_fixed_outputs(&MutationConfig::default());
        let genome = graph_genome(graph);
        let companions = structural_companions(&genome);
        assert!(!companions.reads_shared_memory);
        assert!(!companions.writes_shared_memory);
        assert!(!companions.has_stateful_compute_node);
        assert!(!companions.has_plasticity);
    }

    #[test]
    fn a_stateful_compute_node_kind_is_reported() {
        let mut graph = CgpGraphBackendDef::new_with_fixed_outputs(&MutationConfig::default());
        graph.compute_nodes.push(ComputeNode {
            kind: ComputeNodeKind::DecayIntegrator(0.5),
            inputs: vec![],
            plasticity: None,
        });
        let genome = graph_genome(graph);
        let companions = structural_companions(&genome);
        assert!(companions.has_stateful_compute_node);
        assert!(!companions.has_plasticity);
    }

    #[test]
    fn a_plastic_compute_node_is_reported() {
        let mut graph = CgpGraphBackendDef::new_with_fixed_outputs(&MutationConfig::default());
        graph.compute_nodes.push(ComputeNode {
            kind: ComputeNodeKind::Add,
            inputs: vec![],
            plasticity: Some(PlasticityConfig {
                rule: HebbianRule::Classic,
                learning_rate: 0.1,
                weight_clamp: 1.0,
                lamarckian: false,
                modulation: None,
            }),
        });
        let genome = graph_genome(graph);
        let companions = structural_companions(&genome);
        assert!(companions.has_plasticity);
        assert!(!companions.has_stateful_compute_node);
    }

    #[test]
    fn a_shared_memory_input_edge_reads_shared_memory() {
        let mut graph = CgpGraphBackendDef::new_with_fixed_outputs(&MutationConfig::default());
        graph.compute_nodes.push(ComputeNode {
            kind: ComputeNodeKind::Add,
            inputs: vec![GraphEdge {
                source: GraphSource::SharedMemory { slot: 0, previous: false },
                weight: 1.0,
            }],
            plasticity: None,
        });
        let genome = graph_genome(graph);
        assert!(structural_companions(&genome).reads_shared_memory);
    }

    #[test]
    fn a_wired_write_slot_sink_writes_shared_memory_but_an_unwired_one_does_not() {
        let mut graph = CgpGraphBackendDef::new_with_fixed_outputs(&MutationConfig::default());
        graph.compute_nodes.push(ComputeNode { kind: ComputeNodeKind::Add, inputs: vec![], plasticity: None });
        let sink = graph
            .output_sinks
            .iter_mut()
            .find(|sink| matches!(sink.kind, OutputSinkKind::WriteSlot(0)))
            .expect("the fixed catalog always has a WriteSlot(0) sink");
        sink.inputs.push(GraphEdge { source: GraphSource::ComputeNode(0), weight: 1.0 });
        let genome = graph_genome(graph);
        assert!(structural_companions(&genome).writes_shared_memory);

        let unwired = graph_genome(CgpGraphBackendDef::new_with_fixed_outputs(&MutationConfig::default()));
        assert!(!structural_companions(&unwired).writes_shared_memory);
    }
}
