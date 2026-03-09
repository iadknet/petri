use std::collections::BTreeSet;

use crate::contracts::{InputReference, NodeId, WorldInputKey};

use super::analysis::{
    graph_backward_slice, graph_is_output_node, mesh_reachable_nodes, vm_backward_slice,
    vm_is_output_instruction,
};
use super::{BackendDef, CreatureGenome, GraphBackendDef, GraphNodeKind, NodeGenome, VmBackendDef};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum MeshReadClass {
    Food,
    Neighbor,
    Barrier,
    Occupancy,
    Introspection,
    Upstream,
    ActionQueue,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum MeshWriteClass {
    Route,
    Action,
    Memory,
    Payload,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MeshNodeAnnotation {
    pub node_id: NodeId,
    pub reachable: bool,
    pub read_classes: Vec<MeshReadClass>,
    pub write_classes: Vec<MeshWriteClass>,
    pub has_stateful_behavior: bool,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub live_instruction_indices: Vec<usize>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub live_internal_node_indices: Vec<usize>,
}

#[must_use]
pub fn derive_mesh_annotations(genome: &CreatureGenome) -> Vec<MeshNodeAnnotation> {
    let reachable_indices = mesh_reachable_nodes(genome);
    derive_mesh_annotations_with_reachable_indices(genome, &reachable_indices)
}

#[must_use]
pub fn derive_mesh_annotations_with_reachable_indices(
    genome: &CreatureGenome,
    reachable_indices: &[usize],
) -> Vec<MeshNodeAnnotation> {
    let reachable_ids: BTreeSet<NodeId> = reachable_indices
        .iter()
        .filter_map(|&idx| genome.nodes.get(idx).map(|node| node.node_id))
        .collect();

    genome
        .nodes
        .iter()
        .map(|node| derive_node_annotation(node, reachable_ids.contains(&node.node_id)))
        .collect()
}

fn derive_node_annotation(node: &NodeGenome, reachable: bool) -> MeshNodeAnnotation {
    let mut read_classes = BTreeSet::new();
    let mut write_classes = BTreeSet::new();
    let mut has_stateful_behavior = false;
    let mut live_instruction_indices = Vec::new();
    let mut live_internal_node_indices = Vec::new();

    match &node.backend_def {
        BackendDef::Vm(vm) => {
            live_instruction_indices = collect_live_vm_instruction_indices(vm);
            for index in &live_instruction_indices {
                let Some(instruction) = vm.program.get(*index) else {
                    continue;
                };
                match instruction {
                    super::VmInstruction::ReadInput { ref_idx, .. } => {
                        if let Some(input_ref) = node.input_refs.get(*ref_idx as usize) {
                            read_classes.insert(classify_input_ref(input_ref));
                        }
                    }
                    super::VmInstruction::ReadActionQueueLength { .. }
                    | super::VmInstruction::ReadActionQueueType { .. }
                    | super::VmInstruction::ReadActionQueueParam { .. } => {
                        read_classes.insert(MeshReadClass::ActionQueue);
                    }
                    super::VmInstruction::LoadSlot { .. }
                    | super::VmInstruction::LoadSlotImm { .. }
                    | super::VmInstruction::LoadSlotPrev { .. } => {
                        has_stateful_behavior = true;
                    }
                    super::VmInstruction::StoreSlot { .. }
                    | super::VmInstruction::StoreSlotImm { .. }
                    | super::VmInstruction::ClearSlot { .. } => {
                        has_stateful_behavior = true;
                        write_classes.insert(MeshWriteClass::Memory);
                    }
                    super::VmInstruction::WriteInternalPayload { .. } => {
                        write_classes.insert(MeshWriteClass::Payload);
                    }
                    super::VmInstruction::WriteWorldActionMeta { .. }
                    | super::VmInstruction::PushAction { .. }
                    | super::VmInstruction::PopAction
                    | super::VmInstruction::ExecuteActionQueue
                    | super::VmInstruction::SetPriorityBid { .. } => {
                        write_classes.insert(MeshWriteClass::Action);
                    }
                    super::VmInstruction::WriteRouteTarget { .. } => {
                        write_classes.insert(MeshWriteClass::Route);
                    }
                    _ => {}
                }
            }
        }
        BackendDef::Graph(graph) => {
            live_internal_node_indices = collect_live_graph_node_indices(graph);
            for index in &live_internal_node_indices {
                let Some(internal_node) = graph.internal_nodes.get(*index) else {
                    continue;
                };
                if internal_node.plasticity.is_some() {
                    has_stateful_behavior = true;
                }

                match &internal_node.kind {
                    GraphNodeKind::InputRef { ref_idx, .. } => {
                        if let Some(input_ref) = node.input_refs.get(*ref_idx as usize) {
                            read_classes.insert(classify_input_ref(input_ref));
                        }
                    }
                    GraphNodeKind::RouterOutput => {
                        write_classes.insert(MeshWriteClass::Route);
                    }
                    GraphNodeKind::CustomOutput(_) => {
                        write_classes.insert(MeshWriteClass::Payload);
                    }
                    GraphNodeKind::WriteActionMeta(_)
                    | GraphNodeKind::PushAction(_)
                    | GraphNodeKind::PopAction
                    | GraphNodeKind::ExecuteActionQueue => {
                        write_classes.insert(MeshWriteClass::Action);
                    }
                    GraphNodeKind::ReadSlot(_)
                    | GraphNodeKind::ReadSlotPrev(_)
                    | GraphNodeKind::WriteSlot(_)
                    | GraphNodeKind::ClearSlot(_) => {
                        has_stateful_behavior = true;
                        if matches!(
                            &internal_node.kind,
                            GraphNodeKind::WriteSlot(_) | GraphNodeKind::ClearSlot(_)
                        ) {
                            write_classes.insert(MeshWriteClass::Memory);
                        }
                    }
                    GraphNodeKind::DecayIntegrator(_)
                    | GraphNodeKind::Momentum(_)
                    | GraphNodeKind::Oscillator(_)
                    | GraphNodeKind::AdaptiveGain => {
                        has_stateful_behavior = true;
                    }
                    _ => {}
                }
            }
        }
    }

    MeshNodeAnnotation {
        node_id: node.node_id,
        reachable,
        read_classes: read_classes.into_iter().collect(),
        write_classes: write_classes.into_iter().collect(),
        has_stateful_behavior,
        live_instruction_indices,
        live_internal_node_indices,
    }
}

fn collect_live_vm_instruction_indices(vm: &VmBackendDef) -> Vec<usize> {
    let mut live = BTreeSet::new();
    for (index, instruction) in vm.program.iter().enumerate() {
        if vm_is_output_instruction(instruction) {
            if let Some(gene) = vm_backward_slice(&vm.program, index) {
                live.extend(gene.indices);
            }
        }
    }
    live.into_iter().collect()
}

fn collect_live_graph_node_indices(graph: &GraphBackendDef) -> Vec<usize> {
    let mut live = BTreeSet::new();
    for (index, internal_node) in graph.internal_nodes.iter().enumerate() {
        if graph_is_output_node(&internal_node.kind) {
            if let Some(gene) =
                graph_backward_slice(&graph.internal_nodes, index, graph.internal_nodes.len())
            {
                live.extend(gene.indices);
            }
        }
    }
    live.into_iter().collect()
}

fn classify_input_ref(input_ref: &InputReference) -> MeshReadClass {
    match input_ref {
        InputReference::World(key) => classify_world_input(key),
        InputReference::StaticIntrospection(_) | InputReference::DynamicIntrospection(_) => {
            MeshReadClass::Introspection
        }
        InputReference::UpstreamSlot(_) => MeshReadClass::Upstream,
        InputReference::ActionQueue => MeshReadClass::ActionQueue,
    }
}

fn classify_world_input(key: &WorldInputKey) -> MeshReadClass {
    match key {
        WorldInputKey::FoodHere
        | WorldInputKey::NeighborFoodRing
        | WorldInputKey::AreaFoodSummary => MeshReadClass::Food,
        WorldInputKey::NeighborBarrierRing | WorldInputKey::AreaBarrierSummary => {
            MeshReadClass::Barrier
        }
        WorldInputKey::NeighborOccupiedRing | WorldInputKey::AreaOccupancySummary => {
            MeshReadClass::Occupancy
        }
        WorldInputKey::NearbyCreatureCore
        | WorldInputKey::NearbyCreatureVitals
        | WorldInputKey::NearbyCreatureIdentity => MeshReadClass::Neighbor,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::{DynamicIntrospectionKey, InputReference};
    use crate::creature::genome::analysis::mesh_reachable_nodes;
    use crate::creature::genome::{
        CreatureGenome, GraphBackendDef, GraphInput, GraphInternalNode, GraphNodeKind, NodeGenome,
        VmBackendDef, VmInstruction,
    };

    #[test]
    fn derives_factual_vm_annotations_from_live_instructions() {
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(1),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(1),
                input_refs: vec![
                    InputReference::World(WorldInputKey::FoodHere),
                    InputReference::DynamicIntrospection(DynamicIntrospectionKey::EnergyCurrent),
                ],
                targets: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 2,
                    constants: vec![],
                    program: vec![
                        VmInstruction::ReadInput {
                            dst: 0,
                            ref_idx: 0,
                            sub_idx: 0,
                        },
                        VmInstruction::LoadSlotImm {
                            dst: 1,
                            slot_idx: 3,
                        },
                        VmInstruction::WriteRouteTarget { src: 0 },
                        VmInstruction::StoreSlotImm {
                            slot_idx: 4,
                            src: 1,
                        },
                        VmInstruction::ReadInput {
                            dst: 0,
                            ref_idx: 1,
                            sub_idx: 0,
                        },
                    ],
                }),
            }],
        };

        let annotations = derive_mesh_annotations(&genome);
        let annotation = &annotations[0];

        assert!(annotation.reachable);
        assert_eq!(annotation.read_classes, vec![MeshReadClass::Food]);
        assert_eq!(
            annotation.write_classes,
            vec![MeshWriteClass::Route, MeshWriteClass::Memory]
        );
        assert!(annotation.has_stateful_behavior);
        assert_eq!(annotation.live_instruction_indices, vec![0, 1, 2, 3]);
        assert!(annotation.live_internal_node_indices.is_empty());
    }

    #[test]
    fn derives_graph_annotations_and_marks_unreachable_nodes() {
        let reachable_graph = NodeGenome {
            node_id: NodeId::new(1),
            input_refs: vec![InputReference::World(WorldInputKey::NearbyCreatureCore)],
            targets: vec![],
            backend_def: BackendDef::Graph(GraphBackendDef {
                internal_nodes: vec![
                    GraphInternalNode {
                        kind: GraphNodeKind::InputRef {
                            ref_idx: 0,
                            sub_idx: 0,
                        },
                        inputs: vec![],
                        plasticity: None,
                    },
                    GraphInternalNode {
                        kind: GraphNodeKind::AdaptiveGain,
                        inputs: vec![GraphInput {
                            source_idx: 0,
                            weight: 1.0,
                        }],
                        plasticity: None,
                    },
                    GraphInternalNode {
                        kind: GraphNodeKind::RouterOutput,
                        inputs: vec![GraphInput {
                            source_idx: 1,
                            weight: 1.0,
                        }],
                        plasticity: None,
                    },
                    GraphInternalNode {
                        kind: GraphNodeKind::Constant(1.0),
                        inputs: vec![],
                        plasticity: None,
                    },
                ],
            }),
        };

        let unreachable_vm = NodeGenome {
            node_id: NodeId::new(2),
            input_refs: vec![InputReference::ActionQueue],
            targets: vec![],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 1,
                constants: vec![],
                program: vec![
                    VmInstruction::ReadActionQueueLength { dst: 0 },
                    VmInstruction::SetPriorityBid { src: 0 },
                ],
            }),
        };

        let genome = CreatureGenome {
            entry_node_id: NodeId::new(1),
            nodes: vec![reachable_graph, unreachable_vm],
        };

        let annotations = derive_mesh_annotations(&genome);
        let graph_annotation = annotations
            .iter()
            .find(|annotation| annotation.node_id == NodeId::new(1))
            .expect("graph annotation");
        let unreachable_annotation = annotations
            .iter()
            .find(|annotation| annotation.node_id == NodeId::new(2))
            .expect("vm annotation");

        assert!(graph_annotation.reachable);
        assert_eq!(graph_annotation.read_classes, vec![MeshReadClass::Neighbor]);
        assert_eq!(graph_annotation.write_classes, vec![MeshWriteClass::Route]);
        assert!(graph_annotation.has_stateful_behavior);
        assert_eq!(graph_annotation.live_internal_node_indices, vec![0, 1, 2]);

        assert!(!unreachable_annotation.reachable);
        assert_eq!(
            unreachable_annotation.read_classes,
            vec![MeshReadClass::ActionQueue]
        );
        assert_eq!(
            unreachable_annotation.write_classes,
            vec![MeshWriteClass::Action]
        );
    }

    #[test]
    fn helper_matches_wrapper_for_cached_reachable_indices() {
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(1),
            nodes: vec![
                NodeGenome {
                    node_id: NodeId::new(1),
                    input_refs: vec![],
                    targets: vec![NodeId::new(2)],
                    backend_def: BackendDef::Vm(VmBackendDef {
                        register_count: 1,
                        constants: vec![],
                        program: vec![
                            VmInstruction::LoadConst {
                                dst: 0,
                                const_idx: 0,
                            },
                            VmInstruction::WriteRouteTarget { src: 0 },
                        ],
                    }),
                },
                NodeGenome {
                    node_id: NodeId::new(2),
                    input_refs: vec![InputReference::World(WorldInputKey::FoodHere)],
                    targets: vec![],
                    backend_def: BackendDef::Vm(VmBackendDef {
                        register_count: 1,
                        constants: vec![],
                        program: vec![
                            VmInstruction::ReadInput {
                                dst: 0,
                                ref_idx: 0,
                                sub_idx: 0,
                            },
                            VmInstruction::WriteInternalPayload {
                                slot_idx: 0,
                                src: 0,
                            },
                        ],
                    }),
                },
                NodeGenome {
                    node_id: NodeId::new(3),
                    input_refs: vec![InputReference::ActionQueue],
                    targets: vec![],
                    backend_def: BackendDef::Vm(VmBackendDef {
                        register_count: 1,
                        constants: vec![],
                        program: vec![
                            VmInstruction::ReadActionQueueLength { dst: 0 },
                            VmInstruction::SetPriorityBid { src: 0 },
                        ],
                    }),
                },
            ],
        };

        let reachable_indices = mesh_reachable_nodes(&genome);
        let via_wrapper = derive_mesh_annotations(&genome);
        let via_cached =
            derive_mesh_annotations_with_reachable_indices(&genome, &reachable_indices);

        assert_eq!(via_cached, via_wrapper);
    }

    #[test]
    fn helper_matches_wrapper_for_empty_genome() {
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: Vec::new(),
        };

        let reachable_indices = mesh_reachable_nodes(&genome);
        let via_wrapper = derive_mesh_annotations(&genome);
        let via_cached =
            derive_mesh_annotations_with_reachable_indices(&genome, &reachable_indices);

        assert_eq!(reachable_indices, Vec::<usize>::new());
        assert_eq!(via_cached, via_wrapper);
    }

    #[test]
    fn helper_matches_wrapper_for_cycles_and_dangling_targets() {
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(10),
            nodes: vec![
                NodeGenome {
                    node_id: NodeId::new(10),
                    input_refs: vec![],
                    targets: vec![NodeId::new(11)],
                    backend_def: BackendDef::Vm(VmBackendDef {
                        register_count: 1,
                        constants: vec![],
                        program: vec![VmInstruction::Halt],
                    }),
                },
                NodeGenome {
                    node_id: NodeId::new(11),
                    input_refs: vec![],
                    targets: vec![NodeId::new(10), NodeId::new(999)],
                    backend_def: BackendDef::Vm(VmBackendDef {
                        register_count: 1,
                        constants: vec![],
                        program: vec![VmInstruction::Halt],
                    }),
                },
                NodeGenome {
                    node_id: NodeId::new(12),
                    input_refs: vec![],
                    targets: vec![],
                    backend_def: BackendDef::Vm(VmBackendDef {
                        register_count: 1,
                        constants: vec![],
                        program: vec![VmInstruction::Halt],
                    }),
                },
            ],
        };

        let reachable_indices = mesh_reachable_nodes(&genome);
        let via_wrapper = derive_mesh_annotations(&genome);
        let via_cached =
            derive_mesh_annotations_with_reachable_indices(&genome, &reachable_indices);

        assert_eq!(reachable_indices, vec![0, 1]);
        assert_eq!(via_cached, via_wrapper);
    }
}
