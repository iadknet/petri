//! Bounded mesh execution and static-successor knockout measurements.

use super::{Battery, Signature};
use crate::config::RuntimeConfig;
use crate::contracts::NodeId;
use crate::creature::genome::{analysis::mesh_reachable_nodes, BackendDef, CreatureGenome};
use crate::runtime::mesh::{MeshObservation, ObservedMeshExecution};
use crate::runtime::routing::{resolve_gated_route, RouteGateMap};
use crate::runtime::trace::domain::TerminationReason;
use std::collections::{BTreeMap, BTreeSet};

pub const MESH_EXECUTION_VERSION: &str = "mesh-execution-v1";
pub const KNOCKOUT_METHOD: &str = "static-successor-bypass-v1";

/// Counts from the complete fixed battery; knockout equality concerns action queues only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MeshExecutionReading {
    pub backends: MeshBackendCounts,
    pub total_node_count: usize,
    pub reachable_node_count: usize,
    pub executed_node_count: usize,
    pub knockout_count: usize,
    pub route_varies_with_input: bool,
    pub hop_cap_hits: usize,
}

/// A [`MeshExecutionReading`] with the node ids behind two of its counts.
///
/// `contributing` is always a subset of `executed`: a node contributes only
/// when it was dispatched and its bypass changed the battery signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeshExecutionSets {
    pub reading: MeshExecutionReading,
    pub executed: BTreeSet<NodeId>,
    pub contributing: BTreeSet<NodeId>,
}

/// Additive battery-specific node counts, not mutation creation counts.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BackendNodeCounts {
    pub total: u64,
    pub executed: u64,
    pub contributing: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MeshBackendCounts {
    pub graph: BackendNodeCounts,
    pub vm: BackendNodeCounts,
}

type Routes = BTreeMap<NodeId, BTreeSet<usize>>;

fn snapshot_routes(observation: &MeshObservation) -> Routes {
    let mut routes = Routes::new();
    for &(node, position) in &observation.hops {
        if let Some(position) = position {
            routes.entry(node).or_default().insert(position);
        }
    }
    routes
}

fn route_varies_with_input(snapshots: &[Routes]) -> bool {
    let mut prior = BTreeMap::new();
    for routes in snapshots {
        for (&node, positions) in routes {
            if positions.is_empty() {
                continue;
            }
            if let Some(previous) = prior.insert(node, positions) {
                // Different nonempty sets necessarily have at least two positions in their union.
                if previous != positions {
                    return true;
                }
            }
        }
    }
    false
}

/// Remove only the chosen node and redirect references through its static winner.
pub(crate) fn static_successor_bypass(genome: &CreatureGenome, removed: NodeId) -> CreatureGenome {
    let successor = genome
        .nodes
        .iter()
        .find(|node| node.node_id == removed)
        .and_then(|node| resolve_gated_route(&node.targets, &RouteGateMap::default()))
        .map_or(removed, |(_, id)| id);
    let mut bypass = genome.clone();
    bypass.nodes.retain(|node| node.node_id != removed);
    if bypass.entry_node_id == removed {
        bypass.entry_node_id = successor;
    }
    for node in &mut bypass.nodes {
        for target in &mut node.targets {
            if target.target_id == removed {
                target.target_id = successor;
            }
        }
    }
    bypass
}

/// Everything one observed battery pass yields about a genome's execution.
struct BatteryObservation {
    executed: BTreeSet<NodeId>,
    hop_cap_hits: usize,
    routes: Vec<Routes>,
    baseline: Signature,
}

/// Sorted ascending indices of the genome's nodes carrying one of `ids`.
#[must_use]
pub fn indices_for_node_ids(genome: &CreatureGenome, ids: &BTreeSet<NodeId>) -> Vec<usize> {
    genome
        .nodes
        .iter()
        .enumerate()
        .filter(|(_, node)| ids.contains(&node.node_id))
        .map(|(index, _)| index)
        .collect()
}

impl Battery {
    /// Run the complete battery under hop observation once.
    fn observe(
        &self,
        genome: &CreatureGenome,
        runtime: &RuntimeConfig,
        decay_rate: f32,
    ) -> BatteryObservation {
        let (snapshots, sequences) =
            self.execute_with_mode(genome, runtime, decay_rate, ObservedMeshExecution::default);
        let routes: Vec<_> = snapshots
            .iter()
            .map(|(_, observation)| snapshot_routes(observation))
            .collect();
        let mut executed = BTreeSet::new();
        let mut hop_cap_hits = 0;
        for (_, observation) in snapshots.iter().chain(sequences.iter().flatten()) {
            executed.extend(observation.hops.iter().map(|(id, _)| *id));
            if matches!(
                observation.termination_reason,
                TerminationReason::MaxHopsReached
            ) {
                hop_cap_hits += 1;
            }
        }
        let baseline = Signature {
            snapshots: snapshots
                .into_iter()
                .map(|(output, _)| output.actions)
                .collect(),
            sequences: sequences
                .into_iter()
                .map(|sequence| {
                    sequence
                        .into_iter()
                        .map(|(output, _)| output.actions)
                        .collect()
                })
                .collect(),
        };
        BatteryObservation {
            executed,
            hop_cap_hits,
            routes,
            baseline,
        }
    }

    /// The node ids this genome dispatches anywhere in the battery. Used as
    /// the observation stand-in for a live creature's dispatch record when a
    /// harness mutates genomes that never lived in the world (T11.F17).
    #[must_use]
    pub fn executed_node_ids(
        &self,
        genome: &CreatureGenome,
        runtime: &RuntimeConfig,
        decay_rate: f32,
    ) -> BTreeSet<NodeId> {
        self.observe(genome, runtime, decay_rate).executed
    }

    /// The same set as this genome's sorted mesh node indices.
    #[must_use]
    pub fn executed_indices(
        &self,
        genome: &CreatureGenome,
        runtime: &RuntimeConfig,
        decay_rate: f32,
    ) -> Vec<usize> {
        indices_for_node_ids(genome, &self.executed_node_ids(genome, runtime, decay_rate))
    }

    /// Observe an unmutated subject, then separately bypass each executed node from fresh state.
    #[must_use]
    pub fn mesh_execution(
        &self,
        genome: &CreatureGenome,
        runtime: &RuntimeConfig,
        decay_rate: f32,
    ) -> MeshExecutionReading {
        self.mesh_execution_sets(genome, runtime, decay_rate)
            .reading
    }

    /// [`Battery::mesh_execution`] with the node ids behind two of its counts:
    /// the nodes the battery dispatched and, among those, the nodes whose
    /// static-successor bypass changed the complete signature (T13.F01).
    /// Same single battery pass and same counts; nothing extra is executed.
    #[must_use]
    pub fn mesh_execution_sets(
        &self,
        genome: &CreatureGenome,
        runtime: &RuntimeConfig,
        decay_rate: f32,
    ) -> MeshExecutionSets {
        let BatteryObservation {
            executed,
            hop_cap_hits,
            routes,
            baseline,
        } = self.observe(genome, runtime, decay_rate);
        let mut backends = MeshBackendCounts::default();
        let mut knockout_count = 0;
        let mut contributing = BTreeSet::new();
        for node in &genome.nodes {
            let counts = match node.backend_def {
                BackendDef::Graph(_) => &mut backends.graph,
                BackendDef::Vm(_) => &mut backends.vm,
            };
            counts.total += 1;
            if executed.contains(&node.node_id) {
                counts.executed += 1;
                if self.signature(
                    &static_successor_bypass(genome, node.node_id),
                    runtime,
                    decay_rate,
                ) == baseline
                {
                    knockout_count += 1;
                } else {
                    counts.contributing += 1;
                    contributing.insert(node.node_id);
                }
            }
        }
        MeshExecutionSets {
            reading: MeshExecutionReading {
                backends,
                total_node_count: genome.nodes.len(),
                reachable_node_count: mesh_reachable_nodes(genome).len(),
                executed_node_count: executed.len(),
                knockout_count,
                route_varies_with_input: route_varies_with_input(&routes),
                hop_cap_hits,
            },
            executed,
            contributing,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RuntimeConfig;
    use crate::contracts::{NodeId, RouteTarget};
    use crate::creature::genome::{
        BackendDef, CreatureGenome, NodeGenome, VmBackendDef, VmInstruction,
    };
    use crate::neighborhood::Battery;
    use proptest::prelude::*;

    fn node(id: u32, targets: &[u32], action: bool) -> NodeGenome {
        NodeGenome {
            node_id: NodeId::new(id),
            input_refs: vec![],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 1,
                constants: vec![],
                program: if action {
                    vec![
                        VmInstruction::PushAction { action_type: 1 },
                        VmInstruction::ExecuteActionQueue,
                    ]
                } else {
                    vec![VmInstruction::Halt]
                },
            }),
            targets: targets
                .iter()
                .enumerate()
                .map(|(slot, &id)| RouteTarget {
                    target_id: NodeId::new(id),
                    slot: slot as u8,
                    gate_bias: 0.0,
                })
                .collect(),
        }
    }
    fn genome(nodes: Vec<NodeGenome>) -> CreatureGenome {
        CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes,
        }
    }
    fn reading(g: &CreatureGenome) -> MeshExecutionReading {
        Battery::generate(2).mesh_execution(g, &RuntimeConfig::default(), 0.0)
    }

    #[test]
    fn f18_backend_counts_partition_silent_contributing_and_unreachable_nodes() {
        for graph in [false, true] {
            let mut silent = node(0, &[1], false);
            let mut unreachable = node(2, &[], false);
            if graph {
                let blank =
                    crate::creature::genome::cgp::CgpGraphBackendDef::new_with_fixed_outputs(
                        &crate::config::MutationConfig::default(),
                    );
                silent.backend_def = BackendDef::Graph(blank.clone());
                unreachable.backend_def = BackendDef::Graph(blank);
            }
            let r = reading(&genome(vec![silent, node(1, &[], true), unreachable]));
            assert_eq!(r.backends.graph.total, if graph { 2 } else { 0 });
            assert_eq!(r.backends.graph.executed, u64::from(graph));
            assert_eq!(r.backends.graph.contributing, 0);
            assert_eq!(r.backends.vm.total, if graph { 1 } else { 3 });
            assert_eq!(r.backends.vm.executed, if graph { 1 } else { 2 });
            assert_eq!(r.backends.vm.contributing, 1);
        }
    }

    #[test]
    fn graph_bus_producer_contributes_to_snapshot_actions() {
        use crate::contracts::{InputReference, WorldInputKey};
        use crate::creature::genome::cgp::{
            CgpGraphBackendDef, ComputeNode, ComputeNodeKind, GraphEdge, GraphSource,
            OutputSinkKind,
        };
        let mut producer = node(0, &[1], false);
        producer.input_refs = vec![InputReference::World(WorldInputKey::FoodHere {
            type_idx: Default::default(),
        })];
        let mut graph =
            CgpGraphBackendDef::new_with_fixed_outputs(&crate::config::MutationConfig::default());
        graph.compute_nodes.push(ComputeNode {
            kind: ComputeNodeKind::WeightedSum,
            inputs: vec![GraphEdge {
                source: GraphSource::InputLeaf {
                    ref_idx: 0,
                    sub_idx: 0,
                },
                weight: 7.0,
            }],
            plasticity: None,
        });
        graph
            .output_sinks
            .iter_mut()
            .find(|s| s.kind == OutputSinkKind::CustomOutput(0))
            .unwrap()
            .inputs
            .push(GraphEdge {
                source: GraphSource::ComputeNode(0),
                weight: 1.0,
            });
        producer.backend_def = BackendDef::Graph(graph);
        let mut consumer = node(1, &[], false);
        consumer.input_refs = vec![InputReference::UpstreamSlot(0)];
        if let BackendDef::Vm(vm) = &mut consumer.backend_def {
            vm.program = vec![
                VmInstruction::ReadInput {
                    dst: 0,
                    ref_idx: 0,
                    sub_idx: 0,
                },
                VmInstruction::WriteWorldActionMeta {
                    slot_idx: 0,
                    src: 0,
                },
                VmInstruction::PushAction { action_type: 1 },
                VmInstruction::ExecuteActionQueue,
            ];
        }
        let g = genome(vec![producer, consumer]);
        let battery = Battery::generate(2);
        let runtime = RuntimeConfig::default();
        assert_ne!(
            battery.signature(&g, &runtime, 0.0).snapshots,
            battery
                .signature(&static_successor_bypass(&g, NodeId::new(0)), &runtime, 0.0)
                .snapshots
        );
        let r = reading(&g);
        assert_eq!(r.backends.graph.contributing, 1);
        assert_eq!(r.backends.vm.contributing, 1);
    }

    #[test]
    fn static_chain_counts_executed_silent_and_losing_structure() {
        let g = genome(vec![
            node(0, &[1, 2], false),
            node(1, &[], true),
            node(2, &[], true),
            node(3, &[], false),
        ]);
        let before = g.clone();
        let r = reading(&g);
        assert_eq!(
            (
                r.total_node_count,
                r.reachable_node_count,
                r.executed_node_count,
                r.knockout_count
            ),
            (4, 3, 2, 1)
        );
        assert!(!r.route_varies_with_input);
        assert_eq!(r.hop_cap_hits, 0);
        assert_eq!(g, before);
    }

    /// The executed set the harnesses feed the mutation engine as a stand-in
    /// for a live parent's dispatch record (T11.F17): exactly the nodes the
    /// battery dispatches, by id and as this genome's node indices.
    #[test]
    fn executed_node_ids_and_indices_name_only_the_dispatched_nodes() {
        // Node 0 routes to 1; node 2 is unreachable and node 3 is dangling.
        let g = genome(vec![
            node(0, &[1], false),
            node(1, &[], true),
            node(2, &[], true),
            node(3, &[], false),
        ]);
        let battery = Battery::generate(2);
        let runtime = RuntimeConfig::default();
        assert_eq!(
            battery.executed_node_ids(&g, &runtime, 0.0),
            BTreeSet::from([NodeId::new(0), NodeId::new(1)])
        );
        assert_eq!(battery.executed_indices(&g, &runtime, 0.0), vec![0, 1]);

        // Index mapping follows node order, not node id.
        let reordered = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![
                node(3, &[], false),
                node(2, &[], true),
                node(1, &[], true),
                node(0, &[1], false),
            ],
        };
        assert_eq!(
            battery.executed_indices(&reordered, &runtime, 0.0),
            vec![2, 3]
        );
        assert_eq!(
            indices_for_node_ids(&reordered, &BTreeSet::from([NodeId::new(3)])),
            vec![0]
        );
        assert!(indices_for_node_ids(&reordered, &BTreeSet::new()).is_empty());
        assert!(indices_for_node_ids(&reordered, &BTreeSet::from([NodeId::new(77)])).is_empty());

        // A genome whose entry node is missing dispatches nothing at all.
        let dead = genome(vec![node(9, &[], true)]);
        assert!(battery.executed_node_ids(&dead, &runtime, 0.0).is_empty());
        assert!(battery.executed_indices(&dead, &runtime, 0.0).is_empty());
    }

    #[test]
    fn bypass_preserves_incoming_metadata_and_uses_bias_then_position() {
        let mut g = genome(vec![
            node(0, &[1, 1], false),
            node(1, &[2, 3, 4], false),
            node(2, &[], true),
        ]);
        g.nodes[1].targets[1].gate_bias = 2.0;
        g.nodes[1].targets[2].gate_bias = 2.0;
        let before = g.clone();
        let removed = static_successor_bypass(&g, NodeId::new(1));
        assert_eq!(removed.nodes.len(), 2);
        for (actual, old) in removed.nodes[0].targets.iter().zip(&g.nodes[0].targets) {
            assert_eq!(
                *actual,
                RouteTarget {
                    target_id: NodeId::new(3),
                    ..*old
                }
            );
        }
        assert_eq!(g, before);
        let entry = static_successor_bypass(&g, NodeId::new(0));
        assert_eq!(entry.entry_node_id, NodeId::new(1));
    }

    #[test]
    fn bypass_no_self_and_missing_successors_remain_dangling() {
        for targets in [vec![], vec![1], vec![99]] {
            let g = genome(vec![node(0, &[1], false), node(1, &targets, false)]);
            let removed = static_successor_bypass(&g, NodeId::new(1));
            assert_eq!(
                removed.nodes[0].targets[0].target_id,
                NodeId::new(if targets == vec![99] { 99 } else { 1 })
            );
            assert_eq!(reading(&removed).executed_node_count, 1);
        }
    }

    #[test]
    fn missing_nodes_and_cap_termination_are_not_inferred_from_visits() {
        assert_eq!(reading(&genome(vec![])).executed_node_count, 0);
        assert_eq!(
            reading(&genome(vec![node(0, &[99], false)])).hop_cap_hits,
            0
        );
        let battery = Battery::generate(2);
        let config = RuntimeConfig {
            max_mesh_hops: 2,
            ..RuntimeConfig::default()
        };
        let looping = genome(vec![node(0, &[0], false)]);
        let r = battery.mesh_execution(&looping, &config, 0.0);
        assert_eq!(r.executed_node_count, 1);
        assert_eq!(r.hop_cap_hits, 0);
        let chain = genome(vec![
            node(0, &[1], false),
            node(1, &[2], false),
            node(2, &[], false),
        ]);
        assert_eq!(
            battery.mesh_execution(&chain, &config, 0.0).hop_cap_hits,
            80
        );
        let terminal = genome(vec![node(0, &[1], false), node(1, &[0], true)]);
        assert_eq!(
            battery.mesh_execution(&terminal, &config, 0.0).hop_cap_hits,
            0
        );
    }

    #[test]
    fn conditional_routes_use_positions_even_when_destinations_match() {
        use crate::contracts::{InputReference, WorldInputKey};
        let mut router = node(0, &[1, 1], false);
        router.input_refs = vec![InputReference::World(WorldInputKey::FoodHere {
            type_idx: Default::default(),
        })];
        if let BackendDef::Vm(vm) = &mut router.backend_def {
            vm.program = vec![
                VmInstruction::ReadInput {
                    dst: 0,
                    ref_idx: 0,
                    sub_idx: 0,
                },
                VmInstruction::WriteRouteGate { slot: 1, src: 0 },
                VmInstruction::Halt,
            ];
        }
        let mut g = genome(vec![router, node(1, &[], true), node(2, &[], false)]);
        assert!(reading(&g).route_varies_with_input);
        g.nodes[0].targets[1].target_id = NodeId::new(2);
        let r = reading(&g);
        assert!(r.route_varies_with_input);
        assert_eq!(r.knockout_count, 1); // the silent losing terminal only
                                         // Scores on a terminal backend do not establish applied variation.
        if let BackendDef::Vm(vm) = &mut g.nodes[0].backend_def {
            vm.program.push(VmInstruction::ExecuteActionQueue);
            vm.program.remove(2);
        }
        assert!(!reading(&g).route_varies_with_input);
    }

    #[test]
    fn repeated_route_sets_and_visit_absence_do_not_establish_input_variation() {
        let id = NodeId::new(0);
        let both = BTreeMap::from([(id, BTreeSet::from([0, 1]))]);
        assert!(!route_varies_with_input(&[
            both.clone(),
            Routes::new(),
            both.clone()
        ]));
        assert!(route_varies_with_input(&[
            both,
            BTreeMap::from([(id, BTreeSet::from([0]))])
        ]));
        assert!(!route_varies_with_input(&[
            BTreeMap::from([(id, BTreeSet::new())]),
            BTreeMap::from([(id, BTreeSet::from([1]))])
        ]));
    }

    #[test]
    fn graph_and_vm_memory_producers_and_sequence_only_effects_contribute() {
        use crate::contracts::{InputReference, WorldInputKey};
        use crate::creature::genome::cgp::{
            CgpGraphBackendDef, ComputeNode, ComputeNodeKind, GraphEdge, GraphSource, OutputSink,
            OutputSinkKind,
        };
        let mut reader = node(0, &[1], false);
        if let BackendDef::Vm(vm) = &mut reader.backend_def {
            vm.program = vec![
                VmInstruction::LoadSlotImm {
                    dst: 0,
                    slot_idx: 0,
                },
                VmInstruction::WriteWorldActionMeta {
                    slot_idx: 0,
                    src: 0,
                },
                VmInstruction::PushAction { action_type: 1 },
                VmInstruction::Halt,
            ];
        }
        let mut writer = node(1, &[], false);
        if let BackendDef::Vm(vm) = &mut writer.backend_def {
            vm.constants = vec![3.0];
            vm.program = vec![
                VmInstruction::LoadConst {
                    dst: 0,
                    const_idx: 0,
                },
                VmInstruction::StoreSlotImm {
                    slot_idx: 0,
                    src: 0,
                },
                VmInstruction::Halt,
            ];
        }
        let battery = Battery::generate(2);
        let runtime = RuntimeConfig::default();
        let g = genome(vec![reader.clone(), writer]);
        let baseline = battery.signature(&g, &runtime, 0.0);
        let bypass = battery.signature(&static_successor_bypass(&g, NodeId::new(1)), &runtime, 0.0);
        assert_eq!(baseline.snapshots, bypass.snapshots);
        assert_ne!(baseline.sequences, bypass.sequences);
        assert_eq!(reading(&g).knockout_count, 0);
        assert_eq!(reading(&g).backends.vm.contributing, 2);
        let mut graph =
            CgpGraphBackendDef::new_with_fixed_outputs(&crate::config::MutationConfig::default());
        graph.compute_nodes.push(ComputeNode {
            kind: ComputeNodeKind::WeightedSum,
            inputs: vec![],
            plasticity: None,
        });
        graph.output_sinks = vec![OutputSink {
            kind: OutputSinkKind::WriteSlot(0),
            inputs: vec![GraphEdge {
                source: GraphSource::InputLeaf {
                    ref_idx: 0,
                    sub_idx: 0,
                },
                weight: 7.0,
            }],
        }];
        let graph_writer = NodeGenome {
            node_id: NodeId::new(1),
            input_refs: vec![InputReference::World(WorldInputKey::FoodHere {
                type_idx: Default::default(),
            })],
            backend_def: BackendDef::Graph(graph),
            targets: vec![],
        };
        let graph_genome = genome(vec![reader, graph_writer]);
        let r = reading(&graph_genome);
        assert_eq!(r.knockout_count, 0);
        assert_eq!(r.backends.graph.contributing, 1);
        assert_eq!(r.backends.vm.contributing, 1);
        let baseline = battery.signature(&graph_genome, &runtime, 0.0);
        let bypass = battery.signature(
            &static_successor_bypass(&graph_genome, NodeId::new(1)),
            &runtime,
            0.0,
        );
        assert_eq!(baseline.snapshots, bypass.snapshots);
        assert_ne!(baseline.sequences, bypass.sequences);
    }

    #[test]
    fn bypass_forwards_predecessor_upstream_and_compares_action_payloads() {
        use crate::contracts::InputReference;
        let mut producer = node(0, &[1], false);
        if let BackendDef::Vm(vm) = &mut producer.backend_def {
            vm.constants = vec![3.0];
            vm.program = vec![
                VmInstruction::LoadConst {
                    dst: 0,
                    const_idx: 0,
                },
                VmInstruction::WriteInternalPayload {
                    slot_idx: 0,
                    src: 0,
                },
                VmInstruction::Halt,
            ];
        }
        let mut middle = producer.clone();
        middle.node_id = NodeId::new(1);
        middle.targets[0].target_id = NodeId::new(2);
        if let BackendDef::Vm(vm) = &mut middle.backend_def {
            vm.constants[0] = 5.0;
        }
        let mut consumer = node(2, &[], true);
        if let BackendDef::Vm(vm) = &mut consumer.backend_def {
            vm.program[0] = VmInstruction::PushAction { action_type: 2 };
        }
        consumer.input_refs = vec![InputReference::UpstreamSlot(0)];
        if let BackendDef::Vm(vm) = &mut consumer.backend_def {
            vm.program.splice(
                0..0,
                [
                    VmInstruction::ReadInput {
                        dst: 0,
                        ref_idx: 0,
                        sub_idx: 0,
                    },
                    VmInstruction::WriteWorldActionMeta {
                        slot_idx: 0,
                        src: 0,
                    },
                ],
            );
        }
        let g = genome(vec![producer, middle, consumer]);
        let battery = Battery::generate(2);
        let runtime = RuntimeConfig::default();
        let baseline = battery.signature(&g, &runtime, 0.0);
        let bypass = static_successor_bypass(&g, NodeId::new(1));
        let actual = battery.signature(&bypass, &runtime, 0.0);
        assert_ne!(baseline, actual);
        assert_eq!(
            actual.snapshots[0],
            vec![crate::contracts::WorldAction::Move(
                crate::contracts::Direction::SE
            )]
        );
        assert_eq!(reading(&g).knockout_count, 1); // overwritten producer is silent
    }

    proptest! {
        #[test]
        fn counts_and_bypass_preserve_bounds_and_source(targets in prop::collection::vec(prop::collection::vec(0u32..8, 0..3), 1..8), removed in 0u32..8) {
            for graph_parity in [0, 1] {
            let g = genome(targets.iter().enumerate().map(|(i,t)| {
                let mut n = node(i as u32,t,false);
                if i % 2 == graph_parity { n.backend_def = BackendDef::Graph(crate::creature::genome::cgp::CgpGraphBackendDef::new_with_fixed_outputs(&crate::config::MutationConfig::default())); }
                n
            }).collect());
            let before = g.clone();
            let r = reading(&g);
            prop_assert_eq!(r.backends.graph.total + r.backends.vm.total, r.total_node_count as u64);
            prop_assert_eq!(r.backends.graph.executed + r.backends.vm.executed, r.executed_node_count as u64);
            prop_assert_eq!(r.backends.graph.contributing + r.backends.vm.contributing, (r.executed_node_count - r.knockout_count) as u64);
            for counts in [r.backends.graph, r.backends.vm] { prop_assert!(counts.contributing <= counts.executed && counts.executed <= counts.total); }
            prop_assert!(r.knockout_count <= r.executed_node_count);
            prop_assert!(r.executed_node_count <= r.reachable_node_count);
            prop_assert!(r.reachable_node_count <= r.total_node_count);
            let reachable: BTreeSet<_> = mesh_reachable_nodes(&g).iter().map(|&index| g.nodes[index].node_id).collect();
            let (snapshots, sequences) = Battery::generate(2).execute_with_mode(&g, &RuntimeConfig::default(), 0.0, ObservedMeshExecution::default);
            let executed: BTreeSet<_> = snapshots.iter().chain(sequences.iter().flatten()).flat_map(|(_, observation)| observation.hops.iter().map(|(id, _)| *id)).collect();
            prop_assert!(executed.is_subset(&reachable));
            prop_assert_eq!(executed.len(), r.executed_node_count);

            let bypass = static_successor_bypass(&g, NodeId::new(removed));
            prop_assert!(!bypass.nodes.iter().any(|n| n.node_id == NodeId::new(removed)));
            for original in &g.nodes {
                if original.node_id != NodeId::new(removed) {
                    let retained = bypass.nodes.iter().find(|n| n.node_id == original.node_id).unwrap();
                    prop_assert_eq!(&retained.backend_def, &original.backend_def);
                    prop_assert_eq!(&retained.input_refs, &original.input_refs);
                    prop_assert_eq!(retained.targets.len(), original.targets.len());
                    for (a,b) in retained.targets.iter().zip(&original.targets) {
                        prop_assert_eq!((a.slot,a.gate_bias), (b.slot,b.gate_bias));
                        if b.target_id != NodeId::new(removed) { prop_assert_eq!(a.target_id,b.target_id); }
                    }
                }
            }
            prop_assert_eq!(g,before);
            }
        }
    }

    /// The sibling reading exposes ids without moving any count: the executed
    /// and contributing sets have exactly the sizes the counts report, and
    /// contributing is a subset of executed.
    #[test]
    fn mesh_execution_sets_carry_the_ids_behind_the_unchanged_counts() {
        let config = RuntimeConfig::default();
        let battery = Battery::generate(2);
        let genome =
            crate::creature::founder::founder_genome(crate::config::FounderProfile::V3Alpha1);
        let sets = battery.mesh_execution_sets(&genome, &config, 0.0);
        assert_eq!(sets.reading, battery.mesh_execution(&genome, &config, 0.0));
        assert_eq!(sets.executed.len(), sets.reading.executed_node_count);
        assert_eq!(
            sets.contributing.len(),
            sets.reading.executed_node_count - sets.reading.knockout_count
        );
        assert!(sets.contributing.is_subset(&sets.executed));
        assert!(!sets.executed.is_empty(), "the founder dispatches nodes");
        let ids: BTreeSet<_> = genome.nodes.iter().map(|node| node.node_id).collect();
        assert!(sets.executed.is_subset(&ids));
    }
}
