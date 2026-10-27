//! Deterministic sensor analysis of a genome: what its references declare,
//! what a live reference wires, which snapshots it needs assembled, whether it
//! reads barriers and which world keys its live VM reads resolve. Pure
//! structural reads over the genome and its reachable node indices; no brain
//! is executed and no RNG is consumed. The creature record caches these at
//! birth; readers and telemetry read the caches or call here.
//!
//! Liveness is the analysis already in the tree, applied at key resolution
//! instead of class resolution: [`collect_live_vm_instruction_indices`] for the
//! VM backend, and the shared [`wired_surface_edges`] walk for the graph
//! backend. That walk covers live compute nodes *and* the wired output sinks,
//! vote sinks included, because a founder's world inputs are wired straight
//! to its output sinks.

use std::collections::{BTreeMap, BTreeSet};

use crate::contracts::{
    DynamicIntrospectionKey, InputReference, PerceptionRequirements, WorldInputKey,
};
use crate::genome::cgp::{CgpGraphBackendDef, GraphSource, NodeClass};
use crate::genome::cgp_analysis::{cgp_live_compute_indices, wired_surface_edges};
use crate::genome::mesh_annotations::{
    collect_live_vm_instruction_indices, derive_mesh_annotations_with_reachable_indices,
    MeshReadClass,
};
use crate::genome::{BackendDef, CreatureGenome, VmInstruction};

/// One decision-state input (T19.F05), in catalog order: the census key of
/// the four decision references and `DynamicIntrospection(HopsThisTick)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DecisionInputKey {
    ActionVotes,
    PreviousPassVotes,
    CommitCounts,
    HopsThisTick,
    PreviousOutcome,
}

impl DecisionInputKey {
    /// Every key in catalog order.
    pub const ALL: [Self; 5] = [
        Self::ActionVotes,
        Self::PreviousPassVotes,
        Self::CommitCounts,
        Self::HopsThisTick,
        Self::PreviousOutcome,
    ];

    /// The key of a decision-state reference; `None` for every other one.
    #[must_use]
    pub fn from_input_reference(reference: &InputReference) -> Option<Self> {
        match reference {
            InputReference::ActionVotes => Some(Self::ActionVotes),
            InputReference::PreviousPassVotes => Some(Self::PreviousPassVotes),
            InputReference::CommitCounts => Some(Self::CommitCounts),
            InputReference::DynamicIntrospection(DynamicIntrospectionKey::HopsThisTick) => {
                Some(Self::HopsThisTick)
            }
            InputReference::PreviousOutcome => Some(Self::PreviousOutcome),
            _ => None,
        }
    }

    /// Report label: the variant name.
    #[must_use]
    pub const fn as_key(self) -> &'static str {
        match self {
            Self::ActionVotes => "ActionVotes",
            Self::PreviousPassVotes => "PreviousPassVotes",
            Self::CommitCounts => "CommitCounts",
            Self::HopsThisTick => "HopsThisTick",
            Self::PreviousOutcome => "PreviousOutcome",
        }
    }
}

/// The structural reach of one creature: the world input keys and
/// decision-state inputs a live reference resolves to, and whether it reads
/// shared memory or holds a stateful compute node.
///
/// A key appears at most once however many instructions or edges reference it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SensorCensus {
    /// World input keys reached by a live reference, in `WorldInputKey` order.
    pub world_inputs: BTreeSet<WorldInputKey>,
    /// Decision-state inputs reached by a live reference (T19.F05), in
    /// catalog order.
    pub decision_inputs: BTreeSet<DecisionInputKey>,
    /// A live, reachable node reads a shared-memory slot. `shared_memory`
    /// persists across ticks, so a current-tick read is a stateful read too.
    pub reads_shared_memory: bool,
    /// A live, reachable graph node carries persisted node state
    /// (`NodeClass::Stateful`). Plasticity is not counted here — T14.F05 owns it.
    pub holds_stateful_node: bool,
}

impl SensorCensus {
    /// The creature reads at least one stateful source.
    #[must_use]
    pub const fn reads_any_stateful(&self) -> bool {
        self.reads_shared_memory || self.holds_stateful_node
    }
}

/// Census one creature from its genome and its cached reachable node indices.
///
/// `reachable_indices` is the creature's `reachable_nodes()`, so no second
/// mesh reachability walk is paid.
#[must_use]
pub fn sensor_census(genome: &CreatureGenome, reachable_indices: &[usize]) -> SensorCensus {
    let mut census = SensorCensus::default();
    for &node_idx in reachable_indices {
        let Some(node) = genome.nodes.get(node_idx) else {
            continue;
        };
        match &node.backend_def {
            BackendDef::Vm(vm) => {
                for instruction_idx in collect_live_vm_instruction_indices(vm) {
                    match vm.program.get(instruction_idx) {
                        Some(VmInstruction::ReadInput { ref_idx, .. }) => {
                            census.insert_input(node.input_refs.get(*ref_idx as usize));
                        }
                        Some(
                            VmInstruction::LoadSlot { .. }
                            | VmInstruction::LoadSlotImm { .. }
                            | VmInstruction::LoadSlotPrev { .. },
                        ) => census.reads_shared_memory = true,
                        _ => {}
                    }
                }
            }
            BackendDef::Graph(graph) => census_graph(graph, &node.input_refs, &mut census),
        }
    }
    census
}

/// The world input keys a creature carries: every `World` reference in a
/// reachable node's `input_refs`, wired or not. It is a superset of
/// [`sensor_census`]'s `world_inputs`, since every live read resolves
/// through a reachable node's own references. Out-of-range indices are skipped.
#[must_use]
pub fn declared_world_keys(
    genome: &CreatureGenome,
    reachable_indices: &[usize],
) -> BTreeSet<WorldInputKey> {
    reachable_indices
        .iter()
        .filter_map(|&node_idx| genome.nodes.get(node_idx))
        .flat_map(|node| &node.input_refs)
        .filter_map(|reference| match reference {
            InputReference::World(key) => Some(*key),
            _ => None,
        })
        .collect()
}

/// The snapshots `genome` needs assembled: the union of
/// [`PerceptionRequirements::of_key`] over every node's `World` references,
/// reachable or not.
#[must_use]
pub fn perception_requirements(genome: &CreatureGenome) -> PerceptionRequirements {
    genome
        .nodes
        .iter()
        .flat_map(|node| &node.input_refs)
        .filter_map(|reference| match reference {
            InputReference::World(key) => Some(PerceptionRequirements::of_key(key)),
            _ => None,
        })
        .fold(PerceptionRequirements::NONE, PerceptionRequirements::union)
}

/// Whether any reachable node has a live barrier read path.
#[must_use]
pub fn has_barrier_reader(genome: &CreatureGenome, reachable_indices: &[usize]) -> bool {
    derive_mesh_annotations_with_reachable_indices(genome, reachable_indices)
        .iter()
        .any(|node| {
            node.reachable
                && node
                    .read_classes
                    .iter()
                    .any(|class| matches!(class, MeshReadClass::Barrier))
        })
}

/// The world keys live VM `ReadInput` instructions on reachable nodes
/// resolve, each with its instruction count, in `WorldInputKey` order.
#[must_use]
pub fn live_vm_world_reads(
    genome: &CreatureGenome,
    reachable_indices: &[usize],
) -> Box<[(WorldInputKey, u16)]> {
    let mut counts = BTreeMap::<WorldInputKey, u16>::new();
    for &node_idx in reachable_indices {
        let Some(node) = genome.nodes.get(node_idx) else {
            continue;
        };
        let BackendDef::Vm(vm) = &node.backend_def else {
            continue;
        };
        for instruction_idx in collect_live_vm_instruction_indices(vm) {
            let Some(VmInstruction::ReadInput { ref_idx, .. }) = vm.program.get(instruction_idx)
            else {
                continue;
            };
            let Some(InputReference::World(key)) = node.input_refs.get(*ref_idx as usize) else {
                continue;
            };
            *counts.entry(*key).or_insert(0) += 1;
        }
    }
    counts.into_iter().collect::<Vec<_>>().into_boxed_slice()
}

fn census_graph(
    graph: &CgpGraphBackendDef,
    input_refs: &[InputReference],
    census: &mut SensorCensus,
) {
    let live = cgp_live_compute_indices(graph);
    census.holds_stateful_node |= live
        .iter()
        .any(|&idx| graph.compute_nodes()[idx].kind.class() == NodeClass::Stateful);
    for edge in wired_surface_edges(graph, &live) {
        match edge.source() {
            GraphSource::InputLeaf { ref_idx, .. } => {
                census.insert_input(input_refs.get(ref_idx as usize));
            }
            // Deliberate and spec-mandated: the stateful table counts a
            // shared-memory read on every wired surface, not on live compute
            // nodes alone as `derive_cgp_annotations` does. Do not harmonize
            // the two — a founder wires its edges straight onto its sinks.
            GraphSource::SharedMemory { .. } => census.reads_shared_memory = true,
            GraphSource::ComputeNode(_) => {}
        }
    }
}

impl SensorCensus {
    /// Record the reference a live read addresses, if it is a world key or
    /// a decision-state input.
    fn insert_input(&mut self, input_ref: Option<&InputReference>) {
        let Some(reference) = input_ref else {
            return;
        };
        if let InputReference::World(key) = reference {
            self.world_inputs.insert(*key);
        } else if let Some(key) = DecisionInputKey::from_input_reference(reference) {
            self.decision_inputs.insert(key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::votes::DirectedVoteKind;
    use crate::contracts::world_key_strategy;
    use crate::contracts::{DynamicIntrospectionKey, NodeId, OrdinaryFoodTypeId};
    use crate::creature::founder::v3alpha1_founder_genome;
    use crate::genome::analysis::mesh_reachable_nodes;
    use crate::genome::cgp::test_support::push_sink;
    use crate::genome::cgp::{ComputeNode, ComputeNodeKind, GraphEdge, OutputSink, OutputSinkKind};
    use crate::genome::{HebbianRule, NodeGenome, PlasticityConfig, VmBackendDef, VmInstruction};
    use proptest::prelude::*;

    const FOOD_0: OrdinaryFoodTypeId = OrdinaryFoodTypeId::new(0);

    fn graph_node(graph: CgpGraphBackendDef, input_refs: Vec<InputReference>) -> CreatureGenome {
        CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(0),
                input_refs,
                backend_def: BackendDef::Graph(graph),
                targets: vec![],
            }],
        }
    }

    fn vm_node(program: Vec<VmInstruction>, input_refs: Vec<InputReference>) -> CreatureGenome {
        CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(0),
                input_refs,
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 4,
                    constants: vec![0.0],
                    program,
                }),
                targets: vec![],
            }],
        }
    }

    fn empty_graph() -> CgpGraphBackendDef {
        CgpGraphBackendDef::new_with_fixed_outputs()
            .with_compute_nodes(Vec::new())
            .with_output_sinks(Vec::new())
    }

    fn edge(source: GraphSource) -> GraphEdge {
        GraphEdge::new(source, 1.0)
    }

    fn leaf(ref_idx: u16) -> GraphEdge {
        edge(GraphSource::InputLeaf {
            ref_idx,
            sub_idx: 0,
        })
    }

    fn census_of(genome: &CreatureGenome) -> SensorCensus {
        let reachable = mesh_reachable_nodes(genome);
        sensor_census(genome, &reachable)
    }

    /// The founder wires its world inputs straight to output sinks, not
    /// through a compute node: a census that walked only live compute nodes
    /// would report the whole founder population as sensing nothing.
    #[test]
    fn the_founder_genome_references_its_world_inputs_through_graph_sinks() {
        let census = census_of(&v3alpha1_founder_genome());

        assert!(
            census
                .world_inputs
                .contains(&WorldInputKey::food_here(FOOD_0)),
            "the founder senses food on its own cell: {census:?}"
        );
        assert!(
            census
                .world_inputs
                .contains(&WorldInputKey::neighbor_food_ring(FOOD_0)),
            "the founder senses the neighbor food ring: {census:?}"
        );
    }

    #[test]
    fn a_key_referenced_by_several_instructions_and_edges_is_counted_once() {
        let key = WorldInputKey::NeighborOccupiedRing;
        let graph = {
            let mut graph = empty_graph();
            graph.append_compute_node(ComputeNode::new(
                ComputeNodeKind::Add,
                vec![leaf(0), leaf(0)],
            ));
            push_sink(
                &mut graph,
                OutputSink::new(
                    OutputSinkKind::CustomOutput(0),
                    vec![edge(GraphSource::ComputeNode(0)), leaf(0)],
                ),
            );
            graph
        };
        let census = census_of(&graph_node(graph, vec![InputReference::World(key)]));

        assert_eq!(census.world_inputs, BTreeSet::from([key]));

        let vm = census_of(&vm_node(
            vec![
                VmInstruction::ReadInput {
                    ref_idx: 0,
                    sub_idx: 0,
                    dst: 1,
                },
                VmInstruction::ReadInput {
                    ref_idx: 0,
                    sub_idx: 1,
                    dst: 2,
                },
                VmInstruction::Add { dst: 3, a: 1, b: 2 },
                VmInstruction::WriteInternalPayload {
                    slot_idx: 0,
                    src: 3,
                },
                VmInstruction::Halt,
            ],
            vec![InputReference::World(key)],
        ));

        assert_eq!(vm.world_inputs, BTreeSet::from([key]));
    }

    #[test]
    fn a_reference_on_an_unreachable_graph_compute_node_is_not_counted() {
        let graph = {
            let mut graph = empty_graph();
            // CN0 feeds nothing: no sink wires it.
            graph.append_compute_node(ComputeNode::new(ComputeNodeKind::Add, vec![leaf(0)]));
            graph
        };
        let census = census_of(&graph_node(
            graph,
            vec![InputReference::World(WorldInputKey::AreaBarrierSummary)],
        ));

        assert!(census.world_inputs.is_empty(), "{census:?}");
    }

    #[test]
    fn a_dead_vm_read_input_is_not_counted_but_a_live_one_is() {
        let live_only = vm_node(
            vec![
                // Dead: r1 is never read by an output instruction.
                VmInstruction::ReadInput {
                    ref_idx: 0,
                    sub_idx: 0,
                    dst: 1,
                },
                VmInstruction::ReadInput {
                    ref_idx: 1,
                    sub_idx: 0,
                    dst: 2,
                },
                VmInstruction::WriteInternalPayload {
                    slot_idx: 0,
                    src: 2,
                },
                VmInstruction::Halt,
            ],
            vec![
                InputReference::World(WorldInputKey::AreaBarrierSummary),
                InputReference::World(WorldInputKey::AreaOccupancySummary),
            ],
        );
        let census = census_of(&live_only);

        assert_eq!(
            census.world_inputs,
            BTreeSet::from([WorldInputKey::AreaOccupancySummary])
        );
    }

    #[test]
    fn a_reference_on_a_node_outside_the_reachable_set_is_not_counted() {
        let genome = graph_node(
            {
                let mut graph = empty_graph();
                push_sink(
                    &mut graph,
                    OutputSink::new(OutputSinkKind::CustomOutput(0), vec![leaf(0)]),
                );
                graph
            },
            vec![InputReference::World(WorldInputKey::NearbyCreature)],
        );

        assert!(
            sensor_census(&genome, &[]).world_inputs.is_empty(),
            "an empty reachable set reaches no reference"
        );
        assert!(
            !census_of(&genome).world_inputs.is_empty(),
            "the same genome does carry the reference when the node is reachable"
        );
    }

    #[test]
    fn vm_slot_loads_are_stateful_reads_and_slot_writes_are_not() {
        let loads = [
            VmInstruction::LoadSlot {
                dst: 1,
                slot_reg: 0,
            },
            VmInstruction::LoadSlotImm {
                dst: 1,
                slot_idx: 0,
            },
            VmInstruction::LoadSlotPrev {
                dst: 1,
                slot_idx: 0,
            },
        ];
        for load in loads {
            let census = census_of(&vm_node(
                vec![
                    load.clone(),
                    VmInstruction::WriteInternalPayload {
                        slot_idx: 0,
                        src: 1,
                    },
                    VmInstruction::Halt,
                ],
                vec![],
            ));
            assert!(census.reads_shared_memory, "{load:?} reads shared memory");
            assert!(census.reads_any_stateful());
        }

        let writes = [
            VmInstruction::StoreSlot {
                slot_reg: 0,
                src: 1,
            },
            VmInstruction::StoreSlotImm {
                slot_idx: 0,
                src: 1,
            },
            VmInstruction::ClearSlot { slot_idx: 0 },
        ];
        for write in writes {
            let census = census_of(&vm_node(vec![write.clone(), VmInstruction::Halt], vec![]));
            assert!(
                !census.reads_any_stateful(),
                "{write:?} is a write, not a read"
            );
        }
    }

    #[test]
    fn a_shared_memory_edge_is_a_stateful_read_for_either_previous_flag() {
        for previous in [false, true] {
            let mut graph = empty_graph();
            push_sink(
                &mut graph,
                OutputSink::new(
                    OutputSinkKind::CustomOutput(0),
                    vec![edge(GraphSource::SharedMemory { slot: 0, previous })],
                ),
            );
            let census = census_of(&graph_node(graph, vec![]));

            assert!(
                census.reads_shared_memory,
                "a same-tick shared-memory read is stateful too: previous={previous}"
            );
            assert!(!census.holds_stateful_node);
        }
    }

    #[test]
    fn a_live_stateful_compute_node_is_stateful_and_plasticity_alone_is_not() {
        let stateful_kinds = [
            ComputeNodeKind::DecayIntegrator(0.5),
            ComputeNodeKind::Momentum(0.5),
            ComputeNodeKind::Oscillator(0.5),
            ComputeNodeKind::AdaptiveGain,
        ];
        for kind in stateful_kinds {
            let mut graph = empty_graph();
            graph.append_compute_node(ComputeNode::new(kind, vec![]));
            push_sink(
                &mut graph,
                OutputSink::new(
                    OutputSinkKind::CustomOutput(0),
                    vec![edge(GraphSource::ComputeNode(0))],
                ),
            );
            let census = census_of(&graph_node(graph, vec![]));

            assert!(census.holds_stateful_node, "{kind:?} persists node state");
            assert!(!census.reads_shared_memory);
            assert!(census.reads_any_stateful());
        }

        let mut graph = empty_graph();
        graph.append_compute_node(crate::genome::cgp::test_support::plastic_node(
            ComputeNodeKind::Add,
            vec![leaf(0)],
            Some(PlasticityConfig {
                rule: HebbianRule::Classic,
                learning_rate: 0.1,
                weight_clamp: 1.0,
                lamarckian: false,
                modulation: None,
                retention: 1.0,
            }),
        ));
        push_sink(
            &mut graph,
            OutputSink::new(
                OutputSinkKind::CustomOutput(0),
                vec![edge(GraphSource::ComputeNode(0))],
            ),
        );
        assert!(
            graph.plastic_edges().next().is_some(),
            "the fixture carries a plastic edge from a world input"
        );
        let census = census_of(&graph_node(
            graph,
            vec![InputReference::World(WorldInputKey::NeighborOccupiedRing)],
        ));

        assert!(
            !census.reads_any_stateful(),
            "plasticity is T14.F05's reading, not this one: {census:?}"
        );
    }

    #[test]
    fn an_unreachable_stateful_compute_node_is_not_counted() {
        let mut graph = empty_graph();
        graph.append_compute_node(ComputeNode::new(
            ComputeNodeKind::DecayIntegrator(0.5),
            vec![],
        ));
        let census = census_of(&graph_node(graph, vec![]));

        assert!(!census.reads_any_stateful(), "{census:?}");
    }

    #[test]
    fn world_inputs_reach_vote_and_terminate_sink_edges() {
        use crate::contracts::votes::VoteSink;
        use crate::genome::cgp::{OutputSink, OutputSinkKind};
        let mut graph = empty_graph();
        push_sink(
            &mut graph,
            OutputSink::new(
                OutputSinkKind::ActionVote(VoteSink::Directed(DirectedVoteKind::Move, 0)),
                vec![leaf(0)],
            ),
        );
        push_sink(
            &mut graph,
            OutputSink::new(
                OutputSinkKind::ActionVote(VoteSink::Terminate),
                vec![leaf(1)],
            ),
        );
        let census = census_of(&graph_node(
            graph,
            vec![
                InputReference::World(WorldInputKey::NearbyCreature),
                InputReference::World(WorldInputKey::AreaBarrierSummary),
            ],
        ));

        assert_eq!(
            census.world_inputs,
            BTreeSet::from([
                WorldInputKey::NearbyCreature,
                WorldInputKey::AreaBarrierSummary,
            ])
        );
    }

    /// T19.F05: every decision-state reference a live read addresses is
    /// counted once however often it is read; a dead read is not counted.
    #[test]
    fn each_decision_state_input_is_counted_once_per_reference() {
        let refs = vec![
            InputReference::ActionVotes,
            InputReference::PreviousPassVotes,
            InputReference::CommitCounts,
            InputReference::DynamicIntrospection(DynamicIntrospectionKey::HopsThisTick),
            InputReference::PreviousOutcome,
            InputReference::DynamicIntrospection(DynamicIntrospectionKey::EnergyCurrent),
        ];
        let mut program = Vec::new();
        for ref_idx in (0..6u16).chain(0..5) {
            program.push(VmInstruction::ReadInput {
                ref_idx,
                sub_idx: ref_idx,
                dst: 1,
            });
            program.push(VmInstruction::WriteInternalPayload {
                slot_idx: 0,
                src: 1,
            });
        }
        program.push(VmInstruction::Halt);
        let census = census_of(&vm_node(program, refs.clone()));
        assert_eq!(
            census.decision_inputs,
            BTreeSet::from(DecisionInputKey::ALL),
            "{census:?}"
        );
        assert!(census.world_inputs.is_empty());

        let mut graph = empty_graph();
        push_sink(
            &mut graph,
            OutputSink::new(
                OutputSinkKind::CustomOutput(0),
                vec![leaf(2), leaf(2), leaf(4)],
            ),
        );
        let census = census_of(&graph_node(graph, refs.clone()));
        assert_eq!(
            census.decision_inputs,
            BTreeSet::from([
                DecisionInputKey::CommitCounts,
                DecisionInputKey::PreviousOutcome
            ])
        );

        // A read into a register no output instruction reads is dead.
        let dead = census_of(&vm_node(
            vec![
                VmInstruction::ReadInput {
                    ref_idx: 0,
                    sub_idx: 0,
                    dst: 2,
                },
                VmInstruction::WriteInternalPayload {
                    slot_idx: 0,
                    src: 1,
                },
                VmInstruction::Halt,
            ],
            refs,
        ));
        assert!(dead.decision_inputs.is_empty(), "{dead:?}");
    }

    /// The `decision_inputs` report rows are keyed by these labels, so they
    /// are the stored-summary contract: the variant names, in catalog order.
    #[test]
    fn decision_input_report_labels_are_the_variant_names_in_catalog_order() {
        assert_eq!(
            DecisionInputKey::ALL.map(DecisionInputKey::as_key),
            [
                "ActionVotes",
                "PreviousPassVotes",
                "CommitCounts",
                "HopsThisTick",
                "PreviousOutcome",
            ]
        );
    }

    /// A group's members are wired sink edges: a member-only graph consumes
    /// its ring (T25.F03).
    #[test]
    fn a_ring_group_is_a_live_consumer_of_its_ring() {
        use crate::contracts::votes::DirectedVoteKind;
        use crate::genome::cgp::Shape;
        let mut graph = CgpGraphBackendDef::new_with_fixed_outputs();
        graph.add_group(Shape::Ring { ring_ref: 1 }, DirectedVoteKind::Move, 0);
        let refs = vec![
            InputReference::World(WorldInputKey::food_here(FOOD_0)),
            InputReference::World(WorldInputKey::NeighborOccupiedRing),
        ];
        let census = census_of(&graph_node(graph, refs));
        assert_eq!(
            census.world_inputs,
            BTreeSet::from([WorldInputKey::NeighborOccupiedRing])
        );
    }

    #[test]
    fn a_non_world_reference_is_not_a_world_input() {
        let mut graph = empty_graph();
        push_sink(
            &mut graph,
            OutputSink::new(OutputSinkKind::CustomOutput(0), vec![leaf(0)]),
        );
        let census = census_of(&graph_node(
            graph,
            vec![InputReference::DynamicIntrospection(
                DynamicIntrospectionKey::EnergyCurrent,
            )],
        ));

        assert!(census.world_inputs.is_empty(), "{census:?}");
    }

    #[test]
    fn the_founder_declares_the_occupied_ring_without_wiring_it() {
        let founder = v3alpha1_founder_genome();
        let reachable = mesh_reachable_nodes(&founder);
        let declared = declared_world_keys(&founder, &reachable);

        assert!(
            declared.contains(&WorldInputKey::NeighborOccupiedRing),
            "the founder's node 0 lists the occupied ring: {declared:?}"
        );
        assert!(
            !census_of(&founder)
                .world_inputs
                .contains(&WorldInputKey::NeighborOccupiedRing),
            "no founder edge or projection reads it"
        );
    }

    #[test]
    fn declared_inputs_are_the_world_refs_of_reachable_nodes_only_once_each() {
        let refs = vec![
            InputReference::World(WorldInputKey::NearbyCreature),
            InputReference::World(WorldInputKey::NearbyCreature),
            InputReference::DynamicIntrospection(DynamicIntrospectionKey::EnergyCurrent),
            InputReference::ActionVotes,
        ];
        let genome = graph_node(empty_graph(), refs);

        assert_eq!(
            declared_world_keys(&genome, &mesh_reachable_nodes(&genome)),
            BTreeSet::from([WorldInputKey::NearbyCreature]),
            "an unwired reference is declared; a non-world reference is not"
        );
        assert!(
            declared_world_keys(&genome, &[]).is_empty(),
            "an unreachable node declares nothing"
        );
        assert!(
            declared_world_keys(&genome, &[7]).is_empty(),
            "an out-of-range reachable index is skipped"
        );
    }

    /// A graph whose wired sink reads the drawn refs: the census is exactly the
    /// set of world keys behind the drawn indices, whatever was drawn.
    fn graph_reading_refs(refs: &[InputReference], indices: &[u16]) -> CreatureGenome {
        let mut graph = empty_graph();
        push_sink(
            &mut graph,
            OutputSink::new(
                OutputSinkKind::CustomOutput(0),
                indices.iter().copied().map(leaf).collect(),
            ),
        );
        graph_node(graph, refs.to_vec())
    }

    proptest! {
        /// The census is the set of world keys the drawn live references
        /// resolve to — out-of-range indices and non-world references
        /// contribute nothing, and duplication changes nothing.
        #[test]
        fn the_graph_census_is_exactly_the_world_keys_behind_the_live_refs(
            refs in proptest::collection::vec(
                prop_oneof![
                    world_key_strategy().prop_map(InputReference::World),
                    Just(InputReference::ActionQueue),
                    Just(InputReference::ActionVotes),
                    Just(InputReference::PreviousPassVotes),
                    Just(InputReference::CommitCounts),
                    Just(InputReference::DynamicIntrospection(
                        DynamicIntrospectionKey::HopsThisTick,
                    )),
                    Just(InputReference::PreviousOutcome),
                ],
                0..6usize,
            ),
            indices in proptest::collection::vec(0u16..8, 0..10usize),
        ) {
            let expected: BTreeSet<WorldInputKey> = indices
                .iter()
                .filter_map(|&i| match refs.get(i as usize) {
                    Some(InputReference::World(key)) => Some(*key),
                    _ => None,
                })
                .collect();

            let expected_decision: BTreeSet<DecisionInputKey> = indices
                .iter()
                .filter_map(|&i| refs.get(i as usize))
                .filter_map(DecisionInputKey::from_input_reference)
                .collect();

            let census = census_of(&graph_reading_refs(&refs, &indices));
            prop_assert_eq!(&census.world_inputs, &expected);
            prop_assert_eq!(&census.decision_inputs, &expected_decision);

            let doubled: Vec<u16> = indices.iter().chain(indices.iter()).copied().collect();
            let doubled_census = census_of(&graph_reading_refs(&refs, &doubled));
            prop_assert_eq!(&doubled_census.world_inputs, &expected);
            prop_assert!(!census.reads_any_stateful());
        }

        /// Whatever a creature wires it also declares: a graph leaf (a group
        /// member is one) and a VM read each resolve through the node's own
        /// `input_refs`. The declared set is exactly the world references of
        /// the reachable nodes, the oracle built here from the refs alone.
        #[test]
        fn wired_inputs_are_always_declared(
            graph_refs in proptest::collection::vec(
                prop_oneof![
                    world_key_strategy().prop_map(InputReference::World),
                    Just(InputReference::ActionVotes),
                ],
                0..6usize,
            ),
            vm_refs in proptest::collection::vec(
                world_key_strategy().prop_map(InputReference::World),
                0..4usize,
            ),
            leaves in proptest::collection::vec(0u16..8, 0..8usize),
            reads in proptest::collection::vec(0u16..6, 0..6usize),
        ) {
            let mut genome = graph_reading_refs(&graph_refs, &leaves);
            let mut program = Vec::new();
            for &ref_idx in &reads {
                program.push(VmInstruction::ReadInput { ref_idx, sub_idx: 0, dst: 1 });
                program.push(VmInstruction::WriteInternalPayload { slot_idx: 0, src: 1 });
            }
            program.push(VmInstruction::Halt);
            let mut vm = vm_node(program, vm_refs.clone());
            vm.nodes[0].node_id = NodeId::new(1);
            genome.nodes[0].targets = vec![crate::contracts::RouteTarget {
                target_id: NodeId::new(1),
                slot: 0,
                gate_bias: 0.0,
            }];
            genome.nodes.push(vm.nodes.remove(0));

            let reachable = mesh_reachable_nodes(&genome);
            let declared = declared_world_keys(&genome, &reachable);
            let oracle: BTreeSet<WorldInputKey> = graph_refs
                .iter()
                .chain(vm_refs.iter())
                .filter_map(|reference| match reference {
                    InputReference::World(key) => Some(*key),
                    _ => None,
                })
                .collect();

            prop_assert_eq!(&declared, &oracle);
            let wired = sensor_census(&genome, &reachable).world_inputs;
            prop_assert!(wired.is_subset(&declared), "{:?} not within {:?}", wired, declared);
        }

        /// The requirements are the fold of `of_key` over every `World`
        /// reference of every node, whatever the node order or reachability.
        #[test]
        fn perception_requirements_fold_every_world_reference(
            refs in proptest::collection::vec(
                proptest::collection::vec(
                    prop_oneof![
                        world_key_strategy().prop_map(InputReference::World),
                        Just(InputReference::ActionVotes),
                    ],
                    0..4usize,
                ),
                1..4usize,
            ),
        ) {
            let template = vm_node(vec![VmInstruction::Halt], Vec::new()).nodes.remove(0);
            let mut genome = CreatureGenome {
                entry_node_id: NodeId::new(0),
                nodes: refs
                    .iter()
                    .enumerate()
                    .map(|(i, input_refs)| NodeGenome {
                        node_id: NodeId::new(i as u32),
                        input_refs: input_refs.clone(),
                        ..template.clone()
                    })
                    .collect(),
            };
            let keys: Vec<PerceptionRequirements> = refs
                .iter()
                .flatten()
                .filter_map(|reference| match reference {
                    InputReference::World(key) => Some(PerceptionRequirements::of_key(key)),
                    _ => None,
                })
                .collect();
            let expected = PerceptionRequirements {
                typed_local_food: keys.iter().any(|r| r.typed_local_food),
                extended_perception: keys.iter().any(|r| r.extended_perception),
            };
            prop_assert_eq!(perception_requirements(&genome), expected);
            genome.nodes.reverse();
            prop_assert_eq!(perception_requirements(&genome), expected);
        }
    }

    #[test]
    fn perception_requirements_follow_the_referenced_families() {
        let food = OrdinaryFoodTypeId::default();
        let reads = |refs: Vec<WorldInputKey>| {
            perception_requirements(&vm_node(
                vec![VmInstruction::Halt],
                refs.into_iter().map(InputReference::World).collect(),
            ))
        };
        let typed = PerceptionRequirements {
            typed_local_food: true,
            extended_perception: false,
        };
        let extended = PerceptionRequirements {
            typed_local_food: false,
            extended_perception: true,
        };
        assert_eq!(reads(vec![WorldInputKey::food_here(food)]), typed);
        assert_eq!(
            reads(vec![WorldInputKey::neighbor_food_ring(
                OrdinaryFoodTypeId::new(2)
            )]),
            typed
        );
        assert_eq!(
            reads(vec![
                WorldInputKey::food_here(food),
                WorldInputKey::area_food_summary(food),
            ]),
            PerceptionRequirements::ALL
        );
        assert_eq!(reads(vec![WorldInputKey::AreaBarrierSummary]), extended);
        assert_eq!(
            reads(vec![
                WorldInputKey::NeighborBarrierRing,
                WorldInputKey::NeighborOccupiedRing,
            ]),
            PerceptionRequirements::NONE
        );
        assert_eq!(reads(Vec::new()), PerceptionRequirements::NONE);
    }

    #[test]
    fn perception_requirements_count_an_unreachable_node() {
        let mut genome = vm_node(vec![VmInstruction::Halt], Vec::new());
        let mut detached = genome.nodes[0].clone();
        detached.node_id = NodeId::new(1);
        detached.input_refs = vec![InputReference::World(WorldInputKey::NearbyCreature)];
        genome.nodes.push(detached);
        assert_eq!(mesh_reachable_nodes(&genome), vec![0]);
        assert!(perception_requirements(&genome).extended_perception);
    }
}
