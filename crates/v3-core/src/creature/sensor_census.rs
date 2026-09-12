//! Per-creature sensor usage census — pure structural reads.
//!
//! Answers two questions about one creature, from its genome and its cached
//! mesh reachability alone: which world input keys a live reference reaches,
//! and whether it reads any stateful source. No brain is executed, no RNG is
//! consumed, and nothing here is cached on the hot path.
//!
//! Liveness is the analysis already in the tree, applied at key resolution
//! instead of class resolution: [`collect_live_vm_instruction_indices`] for the
//! VM backend, and the wired-surface edge walk `derive_cgp_annotations` uses
//! for the graph backend. The graph walk covers live compute nodes *and* the
//! wired output sinks, action-bank slots and execute gate, because a founder's
//! world inputs are wired straight to its output sinks.

use std::collections::BTreeSet;

use crate::contracts::{InputReference, OrdinaryFoodTypeId, WorldInputKey};
use crate::creature::genome::cgp::{CgpGraphBackendDef, GraphEdge, GraphSource, NodeClass};
use crate::creature::genome::cgp_analysis::cgp_live_compute_indices;
use crate::creature::genome::mesh_annotations::collect_live_vm_instruction_indices;
use crate::creature::genome::{BackendDef, CreatureGenome, VmInstruction};

/// The structural reach of one creature: the world input keys a live reference
/// resolves to, and whether it reads shared memory or holds a stateful compute
/// node.
///
/// A key appears at most once however many instructions or edges reference it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CreatureSensorCensus {
    /// World input keys reached by a live reference, in `WorldInputKey` order.
    pub world_inputs: BTreeSet<WorldInputKey>,
    /// A live, reachable node reads a shared-memory slot. `shared_memory`
    /// persists across ticks, so a current-tick read is a stateful read too.
    pub reads_shared_memory: bool,
    /// A live, reachable graph node carries persisted node state
    /// (`NodeClass::Stateful`). Plasticity is not counted here — T14.F05 owns it.
    pub holds_stateful_node: bool,
}

impl CreatureSensorCensus {
    /// The creature reads at least one stateful source.
    #[must_use]
    pub const fn reads_any_stateful(&self) -> bool {
        self.reads_shared_memory || self.holds_stateful_node
    }
}

/// Census one creature from its genome and its cached reachable node indices.
///
/// `reachable_indices` is the creature's `cached_reachable_nodes`, so no second
/// mesh reachability walk is paid.
#[must_use]
pub fn creature_sensor_census(
    genome: &CreatureGenome,
    reachable_indices: &[usize],
) -> CreatureSensorCensus {
    let mut census = CreatureSensorCensus::default();
    for &node_idx in reachable_indices {
        let Some(node) = genome.nodes.get(node_idx) else {
            continue;
        };
        match &node.backend_def {
            BackendDef::Vm(vm) => {
                for instruction_idx in collect_live_vm_instruction_indices(vm) {
                    match vm.program.get(instruction_idx) {
                        Some(VmInstruction::ReadInput { ref_idx, .. }) => {
                            insert_world_key(
                                &mut census.world_inputs,
                                node.input_refs.get(*ref_idx as usize),
                            );
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

fn census_graph(
    graph: &CgpGraphBackendDef,
    input_refs: &[InputReference],
    census: &mut CreatureSensorCensus,
) {
    for idx in cgp_live_compute_indices(graph) {
        let node = &graph.compute_nodes[idx];
        if node.kind.class() == NodeClass::Stateful {
            census.holds_stateful_node = true;
        }
        census_edges(&node.inputs, input_refs, census);
    }
    for sink in &graph.output_sinks {
        census_edges(&sink.inputs, input_refs, census);
    }
    for slot in &graph.action_bank {
        census_edges(&slot.gate_inputs, input_refs, census);
        census_edges(&slot.param_inputs, input_refs, census);
    }
    census_edges(&graph.execute_gate.inputs, input_refs, census);
}

fn census_edges(
    edges: &[GraphEdge],
    input_refs: &[InputReference],
    census: &mut CreatureSensorCensus,
) {
    for edge in edges {
        match edge.source {
            GraphSource::InputLeaf { ref_idx, .. } => {
                insert_world_key(&mut census.world_inputs, input_refs.get(ref_idx as usize));
            }
            GraphSource::SharedMemory { .. } => census.reads_shared_memory = true,
            GraphSource::ComputeNode(_) => {}
        }
    }
}

fn insert_world_key(keys: &mut BTreeSet<WorldInputKey>, input_ref: Option<&InputReference>) {
    if let Some(InputReference::World(key)) = input_ref {
        keys.insert(*key);
    }
}

/// Every world input key a world with these ordinary food types can present:
/// the seven unparameterized keys plus the three food-parameterized families,
/// once per food type. A key absent from a census is then distinguishable from
/// a key this world does not have.
#[must_use]
pub fn world_input_key_universe(
    food_types: impl IntoIterator<Item = OrdinaryFoodTypeId>,
) -> BTreeSet<WorldInputKey> {
    let mut keys = BTreeSet::from([
        WorldInputKey::NeighborBarrierRing,
        WorldInputKey::NeighborOccupiedRing,
        WorldInputKey::AreaBarrierSummary,
        WorldInputKey::AreaOccupancySummary,
        WorldInputKey::NearbyCreatureCore,
        WorldInputKey::NearbyCreatureVitals,
        WorldInputKey::NearbyCreatureIdentity,
    ]);
    for type_idx in food_types {
        keys.insert(WorldInputKey::food_here(type_idx));
        keys.insert(WorldInputKey::neighbor_food_ring(type_idx));
        keys.insert(WorldInputKey::area_food_summary(type_idx));
    }
    keys
}

/// Report label for a world input key: the transport key, with the food type
/// appended for the food-parameterized families so they stay distinct.
#[must_use]
pub fn world_input_key_label(key: WorldInputKey) -> String {
    match key.food_type_idx() {
        Some(type_idx) => format!("{}:{}", key.as_key(), type_idx.get()),
        None => key.as_key().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::{DynamicIntrospectionKey, NodeId};
    use crate::creature::founder::v3alpha1_founder_genome;
    use crate::creature::genome::analysis::mesh_reachable_nodes;
    use crate::creature::genome::cgp::{
        ActionSlot, ActionSlotBehavior, ComputeNode, ComputeNodeKind, ExecuteGate, OutputSink,
        OutputSinkKind, WorldActionKind,
    };
    use crate::creature::genome::{
        HebbianRule, NodeGenome, PlasticityConfig, VmBackendDef, VmInstruction,
    };
    use proptest::prelude::*;

    const FOOD_0: OrdinaryFoodTypeId = OrdinaryFoodTypeId::new(0);
    const FOOD_1: OrdinaryFoodTypeId = OrdinaryFoodTypeId::new(1);

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
        CgpGraphBackendDef {
            birth_weights: None,
            compute_nodes: Vec::new(),
            output_sinks: Vec::new(),
            action_bank: Vec::new(),
            execute_gate: ExecuteGate { inputs: Vec::new() },
        }
    }

    fn edge(source: GraphSource) -> GraphEdge {
        GraphEdge {
            source,
            weight: 1.0,
        }
    }

    fn leaf(ref_idx: u16) -> GraphEdge {
        edge(GraphSource::InputLeaf {
            ref_idx,
            sub_idx: 0,
        })
    }

    fn census_of(genome: &CreatureGenome) -> CreatureSensorCensus {
        let reachable = mesh_reachable_nodes(genome);
        creature_sensor_census(genome, &reachable)
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
            graph.compute_nodes.push(ComputeNode {
                kind: ComputeNodeKind::Add,
                inputs: vec![leaf(0), leaf(0)],
                plasticity: None,
            });
            graph.output_sinks.push(OutputSink {
                kind: OutputSinkKind::CustomOutput(0),
                inputs: vec![edge(GraphSource::ComputeNode(0)), leaf(0)],
            });
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
            // CN0 feeds nothing: no sink, action slot or execute gate wires it.
            graph.compute_nodes.push(ComputeNode {
                kind: ComputeNodeKind::Add,
                inputs: vec![leaf(0)],
                plasticity: None,
            });
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
                graph.output_sinks.push(OutputSink {
                    kind: OutputSinkKind::CustomOutput(0),
                    inputs: vec![leaf(0)],
                });
                graph
            },
            vec![InputReference::World(WorldInputKey::NearbyCreatureCore)],
        );

        assert!(
            creature_sensor_census(&genome, &[]).world_inputs.is_empty(),
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
            graph.output_sinks.push(OutputSink {
                kind: OutputSinkKind::CustomOutput(0),
                inputs: vec![edge(GraphSource::SharedMemory { slot: 0, previous })],
            });
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
            graph.compute_nodes.push(ComputeNode {
                kind,
                inputs: vec![],
                plasticity: None,
            });
            graph.output_sinks.push(OutputSink {
                kind: OutputSinkKind::CustomOutput(0),
                inputs: vec![edge(GraphSource::ComputeNode(0))],
            });
            let census = census_of(&graph_node(graph, vec![]));

            assert!(census.holds_stateful_node, "{kind:?} persists node state");
            assert!(!census.reads_shared_memory);
            assert!(census.reads_any_stateful());
        }

        let mut graph = empty_graph();
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
        graph.output_sinks.push(OutputSink {
            kind: OutputSinkKind::CustomOutput(0),
            inputs: vec![edge(GraphSource::ComputeNode(0))],
        });
        let census = census_of(&graph_node(graph, vec![]));

        assert!(
            !census.reads_any_stateful(),
            "plasticity is T14.F05's reading, not this one: {census:?}"
        );
    }

    #[test]
    fn an_unreachable_stateful_compute_node_is_not_counted() {
        let mut graph = empty_graph();
        graph.compute_nodes.push(ComputeNode {
            kind: ComputeNodeKind::DecayIntegrator(0.5),
            inputs: vec![],
            plasticity: None,
        });
        let census = census_of(&graph_node(graph, vec![]));

        assert!(!census.reads_any_stateful(), "{census:?}");
    }

    #[test]
    fn world_inputs_reach_action_slot_and_execute_gate_edges() {
        let mut graph = empty_graph();
        graph.action_bank.push(ActionSlot {
            behavior: ActionSlotBehavior::Emit(WorldActionKind::Move),
            gate_inputs: vec![leaf(0)],
            param_inputs: vec![],
        });
        graph.execute_gate.inputs.push(leaf(1));
        let census = census_of(&graph_node(
            graph,
            vec![
                InputReference::World(WorldInputKey::NearbyCreatureVitals),
                InputReference::World(WorldInputKey::NearbyCreatureIdentity),
            ],
        ));

        assert_eq!(
            census.world_inputs,
            BTreeSet::from([
                WorldInputKey::NearbyCreatureVitals,
                WorldInputKey::NearbyCreatureIdentity,
            ])
        );
    }

    #[test]
    fn a_non_world_reference_is_not_a_world_input() {
        let mut graph = empty_graph();
        graph.output_sinks.push(OutputSink {
            kind: OutputSinkKind::CustomOutput(0),
            inputs: vec![leaf(0)],
        });
        let census = census_of(&graph_node(
            graph,
            vec![InputReference::DynamicIntrospection(
                DynamicIntrospectionKey::EnergyCurrent,
            )],
        ));

        assert!(census.world_inputs.is_empty(), "{census:?}");
    }

    #[test]
    fn the_key_universe_is_seven_plus_three_per_food_type_with_labelled_families() {
        let one = world_input_key_universe([FOOD_0]);
        assert_eq!(one.len(), 10);

        let two = world_input_key_universe([FOOD_0, FOOD_1]);
        assert_eq!(two.len(), 13);
        assert!(two.is_superset(&one));

        assert_eq!(
            world_input_key_label(WorldInputKey::food_here(FOOD_1)),
            "FoodHere:1"
        );
        assert_eq!(
            world_input_key_label(WorldInputKey::AreaBarrierSummary),
            "AreaBarrierSummary"
        );
        let labels: BTreeSet<String> = two.iter().copied().map(world_input_key_label).collect();
        assert_eq!(labels.len(), two.len(), "no two keys share a label");
    }

    fn world_key_strategy() -> impl Strategy<Value = WorldInputKey> {
        prop_oneof![
            (0u16..3).prop_map(|i| WorldInputKey::food_here(OrdinaryFoodTypeId::new(i))),
            (0u16..3).prop_map(|i| WorldInputKey::neighbor_food_ring(OrdinaryFoodTypeId::new(i))),
            (0u16..3).prop_map(|i| WorldInputKey::area_food_summary(OrdinaryFoodTypeId::new(i))),
            Just(WorldInputKey::NeighborBarrierRing),
            Just(WorldInputKey::NeighborOccupiedRing),
            Just(WorldInputKey::AreaBarrierSummary),
            Just(WorldInputKey::AreaOccupancySummary),
            Just(WorldInputKey::NearbyCreatureCore),
            Just(WorldInputKey::NearbyCreatureVitals),
            Just(WorldInputKey::NearbyCreatureIdentity),
        ]
    }

    /// A graph whose wired sink reads the drawn refs: the census is exactly the
    /// set of world keys behind the drawn indices, whatever was drawn.
    fn graph_reading_refs(refs: &[InputReference], indices: &[u16]) -> CreatureGenome {
        let mut graph = empty_graph();
        graph.output_sinks.push(OutputSink {
            kind: OutputSinkKind::CustomOutput(0),
            inputs: indices.iter().copied().map(leaf).collect(),
        });
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

            let census = census_of(&graph_reading_refs(&refs, &indices));
            prop_assert_eq!(&census.world_inputs, &expected);

            let doubled: Vec<u16> = indices.iter().chain(indices.iter()).copied().collect();
            let doubled_census = census_of(&graph_reading_refs(&refs, &doubled));
            prop_assert_eq!(&doubled_census.world_inputs, &expected);
            prop_assert!(!census.reads_any_stateful());
        }

        /// The key universe holds every key the label map distinguishes, and
        /// contains every food-parameterized family exactly once per type.
        #[test]
        fn the_key_universe_covers_every_food_type_once(count in 1usize..6) {
            let types: Vec<OrdinaryFoodTypeId> =
                (0..count).map(|i| OrdinaryFoodTypeId::new(i as u16)).collect();
            let universe = world_input_key_universe(types.iter().copied());

            prop_assert_eq!(universe.len(), 7 + 3 * count);
            for type_idx in types {
                prop_assert!(universe.contains(&WorldInputKey::food_here(type_idx)));
                prop_assert!(universe.contains(&WorldInputKey::neighbor_food_ring(type_idx)));
                prop_assert!(universe.contains(&WorldInputKey::area_food_summary(type_idx)));
            }
            let labels: BTreeSet<String> =
                universe.iter().copied().map(world_input_key_label).collect();
            prop_assert_eq!(labels.len(), universe.len());
        }
    }
}
