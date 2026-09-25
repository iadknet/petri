//! The structural consumers of a genome's inputs: every Graph edge whose
//! source is an input leaf or a shared-memory slot, and every VM instruction
//! that reads an input reference or a shared-memory slot, each with the
//! channel it addresses and whether it lies on the sensor census's live walk
//! (`creature::sensor_census`). Also the ablation genome rewrite.

use std::collections::BTreeSet;

use crate::creature::genome::cgp::{CgpGraphBackendDef, GraphEdge, GraphSource};
use crate::creature::genome::cgp_analysis::cgp_live_compute_indices;
use crate::creature::genome::mesh_annotations::collect_live_vm_instruction_indices;
use crate::creature::genome::{BackendDef, CreatureGenome, NodeGenome, VmInstruction};

use super::catalog::{addressed, shared_memory, Addressed, Channel, Declaration};

/// What a consumer reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Target {
    /// An input reference read at a resolvable `ref_idx`.
    Input(Addressed),
    /// A shared-memory slot fixed by the genome.
    Slot(Channel),
    /// A VM `LoadSlot`, whose slot a register chooses at run time.
    DynamicSlot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Consumer {
    pub(super) target: Target,
    pub(super) live: bool,
}

/// One node's consumers, positioned so a recorded read finds them.
#[derive(Debug, Clone, Default)]
pub(super) struct NodeConsumers {
    pub(super) reachable: bool,
    /// VM consumers by program counter.
    pub(super) vm: Vec<(usize, Consumer)>,
    /// Graph consumers of each compute node, in compute order.
    pub(super) compute: Vec<Vec<Consumer>>,
    /// Graph consumers on the output sinks.
    pub(super) sinks: Vec<Consumer>,
}

impl NodeConsumers {
    fn all(&self) -> impl Iterator<Item = &Consumer> {
        self.vm
            .iter()
            .map(|(_, consumer)| consumer)
            .chain(self.compute.iter().flatten())
            .chain(&self.sinks)
    }

    pub(super) fn vm_at(&self, pc: usize) -> Option<Consumer> {
        self.vm
            .binary_search_by_key(&pc, |(at, _)| *at)
            .ok()
            .map(|position| self.vm[position].1)
    }
}

/// The consumers of every node of a genome, in node order.
#[derive(Debug, Clone)]
pub(super) struct Inventory {
    pub(super) nodes: Vec<NodeConsumers>,
}

fn edge_target(node: &NodeGenome, edge: &GraphEdge) -> Option<Target> {
    match edge.source {
        GraphSource::InputLeaf { ref_idx, sub_idx } => node
            .input_refs
            .get(usize::from(ref_idx))
            .map(|reference| Target::Input(addressed(reference, sub_idx))),
        GraphSource::SharedMemory { slot, previous } if slot < 16 => {
            Some(Target::Slot(shared_memory(usize::from(slot), previous)))
        }
        GraphSource::SharedMemory { .. } | GraphSource::ComputeNode(_) => None,
    }
}

fn instruction_target(node: &NodeGenome, instruction: &VmInstruction) -> Option<Target> {
    match *instruction {
        VmInstruction::ReadInput {
            ref_idx, sub_idx, ..
        } => node
            .input_refs
            .get(usize::from(ref_idx))
            .map(|reference| Target::Input(addressed(reference, sub_idx))),
        VmInstruction::LoadSlotImm { slot_idx, .. } => {
            Some(Target::Slot(shared_memory(usize::from(slot_idx), false)))
        }
        VmInstruction::LoadSlotPrev { slot_idx, .. } => {
            Some(Target::Slot(shared_memory(usize::from(slot_idx), true)))
        }
        VmInstruction::LoadSlot { .. } => Some(Target::DynamicSlot),
        _ => None,
    }
}

fn graph_consumers(node: &NodeGenome, graph: &CgpGraphBackendDef) -> NodeConsumers {
    let live: BTreeSet<usize> = cgp_live_compute_indices(graph).into_iter().collect();
    let consumers = |edges: &[GraphEdge], live: bool| {
        edges
            .iter()
            .filter_map(|edge| edge_target(node, edge))
            .map(|target| Consumer { target, live })
            .collect::<Vec<_>>()
    };
    NodeConsumers {
        reachable: false,
        vm: Vec::new(),
        compute: graph
            .compute_nodes
            .iter()
            .enumerate()
            .map(|(index, compute)| consumers(&compute.inputs, live.contains(&index)))
            .collect(),
        // Every sink edge is on the wired surface: an unwired sink has none.
        sinks: graph
            .output_sinks
            .iter()
            .flat_map(|sink| consumers(&sink.inputs, true))
            .collect(),
    }
}

impl Inventory {
    /// Every node's consumers; `reachable` is `mesh_reachable_nodes`.
    pub(super) fn new(genome: &CreatureGenome, reachable: &[usize]) -> Self {
        let nodes = genome
            .nodes
            .iter()
            .enumerate()
            .map(|(index, node)| {
                let mut consumers = match &node.backend_def {
                    BackendDef::Graph(graph) => graph_consumers(node, graph),
                    BackendDef::Vm(vm) => {
                        let live: BTreeSet<usize> = collect_live_vm_instruction_indices(vm)
                            .into_iter()
                            .collect();
                        NodeConsumers {
                            vm: vm
                                .program
                                .iter()
                                .enumerate()
                                .filter_map(|(pc, instruction)| {
                                    instruction_target(node, instruction).map(|target| {
                                        (
                                            pc,
                                            Consumer {
                                                target,
                                                live: live.contains(&pc),
                                            },
                                        )
                                    })
                                })
                                .collect(),
                            ..NodeConsumers::default()
                        }
                    }
                };
                consumers.reachable = reachable.contains(&index);
                consumers
            })
            .collect();
        Self { nodes }
    }

    /// Every consumer on a reachable node.
    pub(super) fn reachable(&self) -> impl Iterator<Item = &Consumer> {
        self.nodes
            .iter()
            .filter(|node| node.reachable)
            .flat_map(NodeConsumers::all)
    }

    /// Channels a live consumer on a reachable node addresses: `connected`.
    pub(super) fn connected(&self) -> BTreeSet<Channel> {
        self.reachable()
            .filter(|consumer| consumer.live)
            .filter_map(|consumer| fixed_channel(consumer.target))
            .collect()
    }

    /// Input channels (never shared memory) any consumer on a reachable node
    /// addresses: the ablation candidates.
    pub(super) fn input_channels(&self) -> BTreeSet<Channel> {
        self.reachable()
            .filter_map(|consumer| match consumer.target {
                Target::Input(Addressed::Channel(channel)) => Some(channel),
                _ => None,
            })
            .collect()
    }
}

/// The channel a consumer addresses when the genome fixes it.
pub(super) fn fixed_channel(target: Target) -> Option<Channel> {
    match target {
        Target::Input(Addressed::Channel(channel)) | Target::Slot(channel) => Some(channel),
        Target::Input(Addressed::OutOfWidth(_)) | Target::DynamicSlot => None,
    }
}

/// Every declaration on a reachable node.
pub(super) fn declarations(genome: &CreatureGenome, reachable: &[usize]) -> BTreeSet<Declaration> {
    reachable
        .iter()
        .filter_map(|&index| genome.nodes.get(index))
        .flat_map(|node| node.input_refs.iter().map(Declaration::of))
        .collect()
}

/// `genome` with every read of a channel `ablate` selects returning 0.0:
/// each such Graph input leaf and VM `ReadInput` has its `ref_idx` moved out
/// of range, which both backends already resolve to 0.0
/// (`runtime/cgp/sources.rs`, `runtime/vm.rs`). Nothing else changes: the
/// same edges, weights, instructions and `genome_size()`, so execution keeps
/// its native semantics and charges.
pub(in crate::neighborhood) fn ablated(
    genome: &CreatureGenome,
    ablate: impl Fn(Channel) -> bool,
) -> CreatureGenome {
    let mut ablated = genome.clone();
    for node in &mut ablated.nodes {
        assert!(
            node.input_refs.len() < usize::from(u16::MAX),
            "ablation needs u16::MAX to be an out-of-range reference index"
        );
        let refs = &node.input_refs;
        let hit = |ref_idx: u16, sub_idx: u16| {
            refs.get(usize::from(ref_idx)).is_some_and(|reference| {
                matches!(addressed(reference, sub_idx), Addressed::Channel(c) if ablate(c))
            })
        };
        match &mut node.backend_def {
            BackendDef::Graph(graph) => {
                let edges = graph
                    .compute_nodes
                    .iter_mut()
                    .flat_map(|compute| compute.inputs.iter_mut())
                    .chain(
                        graph
                            .output_sinks
                            .iter_mut()
                            .flat_map(|sink| sink.inputs.iter_mut()),
                    );
                for edge in edges {
                    if let GraphSource::InputLeaf { ref_idx, sub_idx } = &mut edge.source {
                        if hit(*ref_idx, *sub_idx) {
                            *ref_idx = u16::MAX;
                        }
                    }
                }
            }
            BackendDef::Vm(vm) => {
                for instruction in &mut vm.program {
                    if let VmInstruction::ReadInput {
                        ref_idx, sub_idx, ..
                    } = instruction
                    {
                        if hit(*ref_idx, *sub_idx) {
                            *ref_idx = u16::MAX;
                        }
                    }
                }
            }
        }
    }
    ablated
}
