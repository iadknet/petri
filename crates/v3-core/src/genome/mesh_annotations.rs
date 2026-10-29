use std::collections::BTreeSet;

use crate::contracts::{InputReference, NodeId, WorldInputKey, OUTPUT_SLOT_COUNT};

use super::analysis::{mesh_reachable_nodes, vm_backward_slice, vm_is_output_instruction};
use super::cgp::{GraphSource, OutputSinkKind};
use super::cgp_analysis::wired_surface_edges;
use super::{BackendDef, CreatureGenome, NodeGenome, VmBackendDef, VmInstruction};

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
    /// The in-tick decision state (T19.F05): `ActionVotes`,
    /// `PreviousPassVotes`, `CommitCounts`.
    Decision,
}

impl MeshReadClass {
    /// Every read class, in declaration order.
    pub const ALL: [Self; 8] = [
        Self::Food,
        Self::Neighbor,
        Self::Barrier,
        Self::Occupancy,
        Self::Introspection,
        Self::Upstream,
        Self::ActionQueue,
        Self::Decision,
    ];
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

impl MeshWriteClass {
    /// Every write class, in declaration order.
    pub const ALL: [Self; 4] = [Self::Route, Self::Action, Self::Memory, Self::Payload];
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
                    super::VmInstruction::WriteActionParam { .. }
                    | super::VmInstruction::SetPriorityBid { .. }
                    | super::VmInstruction::AddVote { .. } => {
                        write_classes.insert(MeshWriteClass::Action);
                    }
                    super::VmInstruction::WriteRouteGate { .. } => {
                        write_classes.insert(MeshWriteClass::Route);
                    }
                    _ => {}
                }
            }
        }
        BackendDef::Graph(graph) => {
            let (cgp_reads, cgp_writes, cgp_stateful, cgp_live) =
                super::cgp_mesh_annotations::derive_cgp_annotations(graph, &node.input_refs);
            read_classes.extend(cgp_reads);
            write_classes.extend(cgp_writes);
            has_stateful_behavior = cgp_stateful;
            live_internal_node_indices = cgp_live;
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

pub(crate) fn collect_live_vm_instruction_indices(vm: &VmBackendDef) -> Vec<usize> {
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

/// The bus slots one node's live circuit touches: the upstream bus slots
/// its live reads reach and the payload (output bus) slots its live writes
/// reach.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NodeBusSlots {
    pub upstream_read: BTreeSet<usize>,
    pub payload_written: BTreeSet<usize>,
}

/// The bus slots `node` reads and writes under `annotation`'s liveness, its
/// own [`MeshNodeAnnotation`], which this reads and never re-derives.
///
/// A live read is a `ReadInput` at a live instruction index (VM) or an
/// `InputLeaf` edge on the wired surface of the live compute nodes (Graph);
/// its slot is [`InputReference::bus_slot`] of the reference it names. A
/// live write is a `WriteInternalPayload` at a live instruction index (VM)
/// or a wired `CustomOutput` sink (Graph), counted below
/// [`OUTPUT_SLOT_COUNT`], the bound the runtime applies.
#[must_use]
pub fn node_bus_slots(node: &NodeGenome, annotation: &MeshNodeAnnotation) -> NodeBusSlots {
    debug_assert_eq!(node.node_id, annotation.node_id);
    let read = |ref_idx: usize, sub_idx: u16| node.input_refs.get(ref_idx)?.bus_slot(sub_idx);
    let on_bus = |slot: usize| (slot < OUTPUT_SLOT_COUNT).then_some(slot);
    let mut slots = NodeBusSlots::default();
    match &node.backend_def {
        BackendDef::Vm(vm) => {
            let live = annotation
                .live_instruction_indices
                .iter()
                .filter_map(|&index| vm.program.get(index));
            for instruction in live {
                match *instruction {
                    VmInstruction::ReadInput {
                        ref_idx, sub_idx, ..
                    } => slots
                        .upstream_read
                        .extend(read(usize::from(ref_idx), sub_idx)),
                    VmInstruction::WriteInternalPayload { slot_idx, .. } => {
                        slots.payload_written.extend(on_bus(usize::from(slot_idx)))
                    }
                    _ => {}
                }
            }
        }
        BackendDef::Graph(def) => {
            let live = &annotation.live_internal_node_indices;
            for edge in wired_surface_edges(def, live) {
                if let GraphSource::InputLeaf { ref_idx, sub_idx } = edge.source() {
                    slots
                        .upstream_read
                        .extend(read(usize::from(ref_idx), sub_idx));
                }
            }
            let wired = def
                .output_sinks()
                .iter()
                .filter(|sink| !sink.inputs.is_empty());
            for sink in wired {
                if let OutputSinkKind::CustomOutput(slot) = sink.kind {
                    slots.payload_written.extend(on_bus(usize::from(slot)));
                }
            }
        }
    }
    slots
}

pub(crate) fn classify_input_ref(input_ref: &InputReference) -> MeshReadClass {
    match input_ref {
        InputReference::World(key) => classify_world_input(key),
        InputReference::StaticIntrospection(_)
        | InputReference::DynamicIntrospection(_)
        | InputReference::PreviousOutcome => MeshReadClass::Introspection,
        InputReference::ActionVotes
        | InputReference::PreviousPassVotes
        | InputReference::CommitCounts => MeshReadClass::Decision,
        InputReference::UpstreamSlot(_) | InputReference::UpstreamBus => MeshReadClass::Upstream,
        InputReference::ActionQueue => MeshReadClass::ActionQueue,
    }
}

fn classify_world_input(key: &WorldInputKey) -> MeshReadClass {
    match key {
        WorldInputKey::FoodHere { .. }
        | WorldInputKey::NeighborFoodRing { .. }
        | WorldInputKey::AreaFoodSummary { .. } => MeshReadClass::Food,
        WorldInputKey::NeighborBarrierRing | WorldInputKey::AreaBarrierSummary => {
            MeshReadClass::Barrier
        }
        WorldInputKey::NeighborOccupiedRing | WorldInputKey::AreaOccupancySummary => {
            MeshReadClass::Occupancy
        }
        WorldInputKey::NearbyCreature => MeshReadClass::Neighbor,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::OrdinaryFoodTypeId;
    use crate::contracts::{DynamicIntrospectionKey, InputReference, RouteTarget};
    use crate::genome::analysis::mesh_reachable_nodes;
    use crate::genome::cgp::{
        CgpGraphBackendDef, ComputeNode, ComputeNodeKind, GraphEdge, GraphSource, OutputSink,
        OutputSinkKind,
    };
    use crate::genome::{CreatureGenome, NodeGenome, VmBackendDef, VmInstruction};

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

    #[test]
    fn derives_factual_vm_annotations_from_live_instructions() {
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(1),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(1),
                input_refs: vec![
                    InputReference::World(WorldInputKey::food_here(OrdinaryFoodTypeId::default())),
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
                        VmInstruction::WriteRouteGate { slot: 0, src: 0 },
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
        // CGP graph: CN0 (AdaptiveGain) with InputLeaf(0,0) → RouterGate(0) sink
        // CN1 (Constant) disconnected (dead)
        let reachable_graph = NodeGenome {
            node_id: NodeId::new(1),
            input_refs: vec![InputReference::World(WorldInputKey::NearbyCreature)],
            targets: vec![],
            backend_def: BackendDef::Graph(
                CgpGraphBackendDef::new_with_fixed_outputs()
                    .with_compute_nodes(vec![
                        ComputeNode::new(
                            ComputeNodeKind::AdaptiveGain,
                            vec![GraphEdge::new(
                                GraphSource::InputLeaf {
                                    ref_idx: 0,
                                    sub_idx: 0,
                                },
                                1.0,
                            )],
                        ),
                        ComputeNode::new(ComputeNodeKind::Constant(1.0), vec![]),
                    ])
                    .with_output_sinks(vec![OutputSink::new(
                        OutputSinkKind::RouterGate(0),
                        vec![GraphEdge::new(GraphSource::ComputeNode(0), 1.0)],
                    )]),
            ),
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
        assert!(graph_annotation.has_stateful_behavior); // AdaptiveGain is stateful
                                                         // Only CN0 is live (CN1 is dead)
        assert_eq!(graph_annotation.live_internal_node_indices, vec![0]);

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

    /// T19.F03: `AddVote` is a world-action write on the VM side, classified
    /// the same way as the graph side's vote sinks.
    #[test]
    fn classifies_add_vote_as_an_action_write() {
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(1),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(1),
                input_refs: vec![],
                targets: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![1.0],
                    program: vec![
                        VmInstruction::LoadConst {
                            dst: 0,
                            const_idx: 0,
                        },
                        VmInstruction::AddVote { sink: 0, src: 0 },
                    ],
                }),
            }],
        };

        let annotation = &derive_mesh_annotations(&genome)[0];

        assert_eq!(annotation.write_classes, vec![MeshWriteClass::Action]);
        assert!(annotation.read_classes.is_empty());
        assert!(!annotation.has_stateful_behavior);
        assert_eq!(annotation.live_instruction_indices, vec![0, 1]);
    }

    #[test]
    fn classifies_write_internal_payload_as_a_payload_write() {
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(1),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(1),
                input_refs: vec![],
                targets: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![1.0],
                    program: vec![
                        VmInstruction::LoadConst {
                            dst: 0,
                            const_idx: 0,
                        },
                        VmInstruction::WriteInternalPayload {
                            slot_idx: 0,
                            src: 0,
                        },
                    ],
                }),
            }],
        };

        let annotation = &derive_mesh_annotations(&genome)[0];

        assert_eq!(annotation.write_classes, vec![MeshWriteClass::Payload]);
        assert!(!annotation.has_stateful_behavior);
        assert_eq!(annotation.live_instruction_indices, vec![0, 1]);
    }

    #[test]
    fn helper_matches_wrapper_for_cached_reachable_indices() {
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(1),
            nodes: vec![
                NodeGenome {
                    node_id: NodeId::new(1),
                    input_refs: vec![],
                    targets: wrap_targets(vec![NodeId::new(2)]),
                    backend_def: BackendDef::Vm(VmBackendDef {
                        register_count: 1,
                        constants: vec![],
                        program: vec![
                            VmInstruction::LoadConst {
                                dst: 0,
                                const_idx: 0,
                            },
                            VmInstruction::WriteRouteGate { slot: 0, src: 0 },
                        ],
                    }),
                },
                NodeGenome {
                    node_id: NodeId::new(2),
                    input_refs: vec![InputReference::World(WorldInputKey::food_here(
                        OrdinaryFoodTypeId::default(),
                    ))],
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
                    targets: wrap_targets(vec![NodeId::new(11)]),
                    backend_def: BackendDef::Vm(VmBackendDef {
                        register_count: 1,
                        constants: vec![],
                        program: vec![VmInstruction::Halt],
                    }),
                },
                NodeGenome {
                    node_id: NodeId::new(11),
                    input_refs: vec![],
                    targets: wrap_targets(vec![NodeId::new(10), NodeId::new(999)]),
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

#[cfg(test)]
mod bus_slot_tests {
    use super::*;
    use crate::contracts::OUTPUT_SLOT_COUNT;
    use crate::genome::cgp::{
        CgpGraphBackendDef, ComputeNode, ComputeNodeKind, GraphEdge, GraphSource, OutputSink,
        OutputSinkKind,
    };
    use crate::genome::VmInstruction;
    use proptest::prelude::{any, Strategy};

    /// `node`'s bus slots under its own annotation, as a reachable node.
    fn slots_of(node: &NodeGenome) -> NodeBusSlots {
        node_bus_slots(node, &derive_node_annotation(node, true))
    }

    fn vm_node(input_refs: Vec<InputReference>, program: Vec<VmInstruction>) -> NodeGenome {
        NodeGenome {
            node_id: NodeId::new(0),
            input_refs,
            targets: Vec::new(),
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 2,
                constants: vec![1.0],
                program,
            }),
        }
    }

    fn graph_node(input_refs: Vec<InputReference>, def: CgpGraphBackendDef) -> NodeGenome {
        NodeGenome {
            node_id: NodeId::new(0),
            input_refs,
            targets: Vec::new(),
            backend_def: BackendDef::Graph(def),
        }
    }

    fn leaf(sub_idx: u16) -> GraphEdge {
        GraphEdge::new(
            GraphSource::InputLeaf {
                ref_idx: 0,
                sub_idx,
            },
            1.0,
        )
    }

    fn sink(slot: u8, inputs: Vec<GraphEdge>) -> OutputSink {
        OutputSink::new(OutputSinkKind::CustomOutput(slot), inputs)
    }

    /// A Graph node whose input leaf `sub_idx` of `reference` feeds a
    /// `CustomOutput(0)` sink directly.
    fn graph_reading(reference: InputReference, sub_idx: u16) -> NodeGenome {
        let def = CgpGraphBackendDef::new_with_fixed_outputs()
            .with_compute_nodes(Vec::new())
            .with_output_sinks(vec![sink(0, vec![leaf(sub_idx)])]);
        graph_node(vec![reference], def)
    }

    /// A VM node whose live `ReadInput` of `reference` at `sub_idx` feeds a
    /// payload write.
    fn vm_reading(reference: InputReference, sub_idx: u16) -> NodeGenome {
        vm_node(
            vec![reference],
            vec![
                VmInstruction::ReadInput {
                    dst: 0,
                    ref_idx: 0,
                    sub_idx,
                },
                VmInstruction::WriteInternalPayload {
                    slot_idx: 0,
                    src: 0,
                },
            ],
        )
    }

    /// A VM node writing a constant to payload slot `slot_idx`.
    fn vm_payload_write(input_refs: Vec<InputReference>, slot_idx: u8) -> NodeGenome {
        vm_node(
            input_refs,
            vec![
                VmInstruction::LoadConst {
                    dst: 1,
                    const_idx: 0,
                },
                VmInstruction::WriteInternalPayload { slot_idx, src: 1 },
            ],
        )
    }

    #[test]
    fn a_declared_but_unread_upstream_slot_reads_no_slot() {
        // Arrange
        let node = vm_payload_write(vec![InputReference::UpstreamSlot(3)], 0);

        // Act
        let slots = slots_of(&node);

        // Assert
        assert!(slots.upstream_read.is_empty(), "{slots:?}");
    }

    #[test]
    fn a_dead_vm_read_input_reads_no_slot() {
        // Arrange: the read's register reaches no output instruction.
        let mut node = vm_payload_write(vec![InputReference::UpstreamSlot(3)], 0);
        let BackendDef::Vm(vm) = &mut node.backend_def else {
            unreachable!("a VM node")
        };
        vm.program.insert(
            0,
            VmInstruction::ReadInput {
                dst: 0,
                ref_idx: 0,
                sub_idx: 0,
            },
        );

        // Act
        let slots = slots_of(&node);

        // Assert
        assert!(slots.upstream_read.is_empty(), "{slots:?}");
        assert_eq!(slots.payload_written, BTreeSet::from([0]));
    }

    #[test]
    fn an_input_leaf_into_a_non_live_compute_node_reads_no_slot() {
        // Arrange: compute node 0 reads slot 2; only `live` wires it to a sink.
        let dead = CgpGraphBackendDef::new_with_fixed_outputs()
            .with_compute_nodes(vec![ComputeNode::new(ComputeNodeKind::Add, vec![leaf(0)])])
            .with_output_sinks(Vec::new());
        let live = dead.clone().with_output_sinks(vec![sink(
            1,
            vec![GraphEdge::new(GraphSource::ComputeNode(0), 1.0)],
        )]);
        let refs = vec![InputReference::UpstreamSlot(2)];

        // Act
        let dead_slots = slots_of(&graph_node(refs.clone(), dead));
        let live_slots = slots_of(&graph_node(refs, live));

        // Assert
        assert!(dead_slots.upstream_read.is_empty(), "{dead_slots:?}");
        assert_eq!(live_slots.upstream_read, BTreeSet::from([2]));
    }

    /// A bus field read through `UpstreamBus[j]` is slot `j` read, as
    /// `UpstreamSlot(j)` reads it, on either backend; a read past the bus
    /// reads no slot.
    #[test]
    fn a_bus_field_read_is_the_slot_its_upstream_slot_twin_reads() {
        let past = u16::try_from(OUTPUT_SLOT_COUNT).expect("a small bus");
        for build in [graph_reading, vm_reading] {
            // Act
            let via_slot = slots_of(&build(InputReference::UpstreamSlot(3), 0)).upstream_read;
            let via_bus = slots_of(&build(InputReference::UpstreamBus, 3)).upstream_read;
            let off_bus = slots_of(&build(InputReference::UpstreamBus, past)).upstream_read;

            // Assert
            assert_eq!(via_slot, BTreeSet::from([3]));
            assert_eq!(via_bus, via_slot);
            assert!(off_bus.is_empty(), "{off_bus:?}");
        }
    }

    #[test]
    fn an_upstream_slot_read_through_any_field_reads_its_slot() {
        for build in [graph_reading, vm_reading] {
            // Act
            let slots = slots_of(&build(InputReference::UpstreamSlot(5), 7));

            // Assert
            assert_eq!(slots.upstream_read, BTreeSet::from([5]));
        }
    }

    #[test]
    fn a_vm_payload_write_writes_its_slot_below_the_bus_bound() {
        // Arrange
        let past = u8::try_from(OUTPUT_SLOT_COUNT).expect("a small bus");

        // Act
        let within = slots_of(&vm_payload_write(Vec::new(), 4));
        let beyond = slots_of(&vm_payload_write(Vec::new(), past));

        // Assert
        assert_eq!(within.payload_written, BTreeSet::from([4]));
        assert!(beyond.payload_written.is_empty(), "{beyond:?}");
    }

    /// The payload slots a lone wired write to `slot` reaches: `slot` when it
    /// is on the bus, none past it.
    fn expected_payload(slot: u8) -> BTreeSet<usize> {
        let slot = usize::from(slot);
        if slot < OUTPUT_SLOT_COUNT {
            BTreeSet::from([slot])
        } else {
            BTreeSet::new()
        }
    }

    proptest::proptest! {
        /// A live `UpstreamSlot(k)` read, through any sub-index, and a live
        /// `UpstreamBus[k]` read both read upstream slot `k` on either backend.
        #[test]
        fn a_live_upstream_slot_read_and_its_bus_field_twin_read_one_slot(
            slot in 0..OUTPUT_SLOT_COUNT,
            any_sub in any::<u16>(),
        ) {
            let bus_field = u16::try_from(slot).expect("a small bus");
            for build in [graph_reading, vm_reading] {
                let via_slot =
                    slots_of(&build(InputReference::UpstreamSlot(slot), any_sub)).upstream_read;
                let via_bus = slots_of(&build(InputReference::UpstreamBus, bus_field)).upstream_read;

                proptest::prop_assert_eq!(&via_slot, &BTreeSet::from([slot]));
                proptest::prop_assert_eq!(&via_bus, &via_slot);
            }
        }

        /// A live `UpstreamBus` read past the bus reads no slot on either
        /// backend.
        #[test]
        fn a_live_bus_field_read_past_the_bus_reads_no_slot(
            sub_idx in (OUTPUT_SLOT_COUNT..=usize::from(u16::MAX))
                .prop_map(|sub| u16::try_from(sub).expect("a u16 range")),
        ) {
            for build in [graph_reading, vm_reading] {
                let slots = slots_of(&build(InputReference::UpstreamBus, sub_idx));

                proptest::prop_assert!(slots.upstream_read.is_empty(), "{:?}", slots);
            }
        }

        /// A live payload write reaches its slot below `OUTPUT_SLOT_COUNT`
        /// and no slot at or above it, on either backend.
        #[test]
        fn a_live_payload_write_reaches_only_on_bus_slots(slot in any::<u8>()) {
            let vm = slots_of(&vm_payload_write(Vec::new(), slot));
            let graph = slots_of(&graph_node(
                vec![InputReference::UpstreamSlot(0)],
                CgpGraphBackendDef::new_with_fixed_outputs()
                    .with_compute_nodes(Vec::new())
                    .with_output_sinks(vec![sink(slot, vec![leaf(0)])]),
            ));

            proptest::prop_assert_eq!(&vm.payload_written, &expected_payload(slot));
            proptest::prop_assert_eq!(&graph.payload_written, &expected_payload(slot));
        }
    }

    #[test]
    fn a_wired_custom_output_writes_its_slot_and_an_unwired_one_does_not() {
        // Arrange
        let past = u8::try_from(OUTPUT_SLOT_COUNT).expect("a small bus");
        let def = CgpGraphBackendDef::new_with_fixed_outputs()
            .with_compute_nodes(Vec::new())
            .with_output_sinks(vec![
                sink(6, vec![leaf(0)]),
                sink(7, Vec::new()),
                sink(past, vec![leaf(0)]),
            ]);
        let node = graph_node(vec![InputReference::UpstreamSlot(0)], def);

        // Act
        let slots = slots_of(&node);

        // Assert
        assert_eq!(slots.payload_written, BTreeSet::from([6]));
    }
}
