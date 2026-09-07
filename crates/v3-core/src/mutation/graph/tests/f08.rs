//! T11.F08 graph tests: a copied compute node or cluster is placed so every
//! copied edge reads the same evaluation phase its original reads, so the copy
//! reproduces its original when a later mutation makes something read it; and
//! `split_existing_edge` skips the one edge shape that cannot be split
//! function-preservingly, replacing T11.F03's documented exception.

use proptest::prelude::*;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use super::operators::{
    ample_runtime_config, assert_neutral, base_def, base_input_refs, plasticity_def,
    plasticity_input_refs, scenarios,
};
use crate::config::RuntimeConfig;
use crate::contracts::{DynamicIntrospectionKey, InputReference, WorldAction, WorldInputKey};
use crate::creature::genome::cgp::{
    ActionSlot, ActionSlotBehavior, CgpGraphBackendDef, ComputeNode, ComputeNodeKind, ExecuteGate,
    GraphEdge, GraphSource, OutputSink, OutputSinkKind, WorldActionKind,
};
use crate::creature::genome::PlasticityConfig;
use crate::creature::state::GraphRuntimeState;
use crate::mutation::graph::operators::{copy_cgp_subgraph, copy_compute_node, split_existing_edge};
use crate::mutation::types::MutationSkipReason;
use crate::runtime::cgp::execute_graph_node_with_reserve;
use crate::runtime::types::{MeshSideOutputs, NodeResult, OUTPUT_SLOT_COUNT};

/// World ticks per scenario. Backward and self edges read frozen tick-start
/// outputs (T11.F06), which are all zero on the first tick, so a phase error
/// is only visible from the second tick on.
const SEQUENCE_TICKS: usize = 4;

type TickOutcome = (NodeResult, Vec<WorldAction>, [f32; 16]);

/// Execute `def` for `SEQUENCE_TICKS` world ticks per scenario from fresh
/// runtime state, crossing the tick boundary through
/// `GraphRuntimeState::begin_tick` so persistent outputs and state advance on
/// the world clock. Energy is refreshed each tick: this compares structure,
/// not the per-node energy charge a copy legitimately adds.
fn run_tick_sequences(
    def: &CgpGraphBackendDef,
    input_refs: &[InputReference],
    config: &RuntimeConfig,
) -> Vec<Vec<TickOutcome>> {
    scenarios()
        .iter()
        .map(|sensors| {
            let mut shared_memory = [0.0f32; 16];
            let mut graph_runtime = GraphRuntimeState::new();
            (0..SEQUENCE_TICKS)
                .map(|_| {
                    let prev_shared_memory = shared_memory;
                    let mut energy = 1.0e6f32;
                    let mut side_outputs = MeshSideOutputs::new(8);
                    graph_runtime.begin_tick(&[]);
                    let result = execute_graph_node_with_reserve(
                        def,
                        input_refs,
                        &[0.0f32; OUTPUT_SLOT_COUNT],
                        &mut energy,
                        0.0,
                        0.0,
                        0,
                        &mut graph_runtime,
                        sensors,
                        config,
                        &mut side_outputs,
                        &mut shared_memory,
                        &prev_shared_memory,
                    );
                    (
                        result,
                        side_outputs.action_queue.clone().into_actions(),
                        shared_memory,
                    )
                })
                .collect()
        })
        .collect()
}

/// Assert `child` reproduces `parent` on every scenario and every tick of the
/// multi-tick sequence.
fn assert_sequences_equivalent(
    label: &str,
    parent: &CgpGraphBackendDef,
    parent_refs: &[InputReference],
    child: &CgpGraphBackendDef,
    child_refs: &[InputReference],
    config: &RuntimeConfig,
) {
    let before = run_tick_sequences(parent, parent_refs, config);
    let after = run_tick_sequences(child, child_refs, config);
    for (scenario, (b, a)) in before.iter().zip(after.iter()).enumerate() {
        for (tick, ((b_result, b_actions, b_mem), (a_result, a_actions, a_mem))) in
            b.iter().zip(a.iter()).enumerate()
        {
            assert_eq!(
                b_actions, a_actions,
                "{label}: scenario {scenario} tick {tick} actions differ"
            );
            assert_eq!(
                b_result, a_result,
                "{label}: scenario {scenario} tick {tick} node result differs"
            );
            assert_eq!(
                b_mem, a_mem,
                "{label}: scenario {scenario} tick {tick} shared memory differs"
            );
        }
    }
}

/// Final index of the `i`-th duplicated source and of its copy.
fn placement(sources: &[usize], i: usize) -> (usize, usize) {
    (sources[i] + i, sources[i] + i + 1)
}

/// Activate the copies made by `duplicate_compute_nodes_in_place(sources)`:
/// every edge outside the duplicated set that read a member now reads that
/// member's copy. Edges belonging to a member or to a copy are untouched, so
/// the duplicated set keeps its own internal wiring.
fn activate_copies(def: &mut CgpGraphBackendDef, sources: &[usize]) {
    let mut retarget: Vec<(u16, u16)> = Vec::with_capacity(sources.len());
    let mut duplicated: Vec<usize> = Vec::with_capacity(sources.len() * 2);
    for i in 0..sources.len() {
        let (original, copy) = placement(sources, i);
        retarget.push((original as u16, copy as u16));
        duplicated.push(original);
        duplicated.push(copy);
    }
    let remap = |edge: &mut GraphEdge| {
        if let GraphSource::ComputeNode(idx) = &mut edge.source {
            if let Some(&(_, copy)) = retarget.iter().find(|(original, _)| original == idx) {
                *idx = copy;
            }
        }
    };
    for (i, node) in def.compute_nodes.iter_mut().enumerate() {
        if !duplicated.contains(&i) {
            node.inputs.iter_mut().for_each(remap);
        }
    }
    for sink in &mut def.output_sinks {
        sink.inputs.iter_mut().for_each(remap);
    }
    for slot in &mut def.action_bank {
        slot.gate_inputs.iter_mut().for_each(remap);
        slot.param_inputs.iter_mut().for_each(remap);
    }
    def.execute_gate.inputs.iter_mut().for_each(remap);
}

/// A graph built around evaluation phase: node 1 reads the higher-index node 2
/// (a frozen tick-start read), node 0 reads node 1 backwards, node 2 reads
/// node 1 forwards, and a non-compute surface reads node 1 directly. Copying
/// node 1 anywhere but immediately after itself changes what the copy reads,
/// what a consumer of the copy reads, or both.
pub(super) fn phase_def() -> CgpGraphBackendDef {
    CgpGraphBackendDef {
        compute_nodes: vec![
            ComputeNode {
                kind: ComputeNodeKind::WeightedSum,
                inputs: vec![
                    GraphEdge {
                        source: GraphSource::InputLeaf {
                            ref_idx: 0,
                            sub_idx: 3,
                        },
                        weight: 0.5,
                    },
                    // Backward external edge onto the copy target.
                    GraphEdge {
                        source: GraphSource::ComputeNode(1),
                        weight: 0.7,
                    },
                ],
                plasticity: None,
            },
            ComputeNode {
                kind: ComputeNodeKind::WeightedSum,
                inputs: vec![
                    GraphEdge {
                        source: GraphSource::ComputeNode(0),
                        weight: 0.6,
                    },
                    // Backward input: read from the frozen tick-start outputs.
                    GraphEdge {
                        source: GraphSource::ComputeNode(2),
                        weight: 0.9,
                    },
                ],
                plasticity: None,
            },
            ComputeNode {
                kind: ComputeNodeKind::DecayIntegrator(0.4),
                inputs: vec![GraphEdge {
                    source: GraphSource::ComputeNode(1),
                    weight: 1.0,
                }],
                plasticity: None,
            },
        ],
        output_sinks: vec![OutputSink {
            kind: OutputSinkKind::CustomOutput(0),
            inputs: vec![GraphEdge {
                source: GraphSource::ComputeNode(2),
                weight: 1.0,
            }],
        }],
        action_bank: vec![ActionSlot {
            behavior: ActionSlotBehavior::Emit(WorldActionKind::Move),
            gate_inputs: vec![GraphEdge {
                source: GraphSource::ComputeNode(0),
                weight: 1.0,
            }],
            param_inputs: vec![GraphEdge {
                source: GraphSource::ComputeNode(1),
                weight: 1.0,
            }],
        }],
        execute_gate: ExecuteGate {
            inputs: vec![GraphEdge {
                source: GraphSource::ComputeNode(0),
                weight: 1.0,
            }],
        },
    }
}

fn activation_fixtures() -> [(&'static str, CgpGraphBackendDef, Vec<InputReference>); 3] {
    [
        ("base", base_def(), base_input_refs()),
        ("plasticity", plasticity_def(), plasticity_input_refs()),
        ("phase", phase_def(), base_input_refs()),
    ]
}

// ── Placement ──────────────────────────────────────────────────────────────

#[test]
fn copy_internal_node_inserts_the_copy_directly_after_its_source() {
    // The two nodes carry different kinds, so the copy identifies its source.
    let parent = phase_def();
    let mut seen = [false; 3];
    for seed in 0u64..48 {
        let mut child = parent.clone();
        copy_compute_node(&mut child, &mut SmallRng::seed_from_u64(seed)).unwrap();
        assert_eq!(child.compute_nodes.len(), parent.compute_nodes.len() + 1);
        let source = (0..parent.compute_nodes.len())
            .find(|&i| child.compute_nodes[i + 1].kind == parent.compute_nodes[i].kind)
            .expect("the copy sits directly after one of the original nodes");
        seen[source] = true;
        assert_eq!(
            child.compute_nodes[source + 1].kind,
            parent.compute_nodes[source].kind,
        );
        assert_eq!(
            child.compute_nodes[source + 1].plasticity,
            parent.compute_nodes[source].plasticity,
        );
    }
    assert!(seen.iter().all(|&s| s), "every source index is reachable");
}

#[test]
fn copy_internal_node_shifts_the_copied_inputs_and_follows_a_self_edge() {
    let mut child = base_def();
    // Node 2 carries a self-loop and node 1 reads the lower-index node 0.
    child.duplicate_compute_nodes_in_place(&[1]);
    assert_eq!(
        child.compute_nodes[2].inputs[0].source,
        GraphSource::ComputeNode(0),
        "a source below the original stays below the copy"
    );
    assert_eq!(
        child.compute_nodes[3].inputs[0].source,
        GraphSource::ComputeNode(3),
        "the self-loop node itself shifted with the insertion"
    );

    let mut self_child = base_def();
    self_child.duplicate_compute_nodes_in_place(&[2]);
    assert_eq!(
        self_child.compute_nodes[3].inputs[0].source,
        GraphSource::ComputeNode(3),
        "a self-edge on the copy reads the copy, not the original"
    );
    assert_eq!(
        self_child.compute_nodes[2].inputs[0].source,
        GraphSource::ComputeNode(2),
        "the original keeps reading itself"
    );
}

#[test]
fn copy_subgraph_inserts_every_copy_directly_after_its_member() {
    let mut child = phase_def();
    child.duplicate_compute_nodes_in_place(&[0, 2]);
    let kinds: Vec<ComputeNodeKind> = child.compute_nodes.iter().map(|n| n.kind).collect();
    let parent = phase_def();
    assert_eq!(
        kinds,
        vec![
            parent.compute_nodes[0].kind,
            parent.compute_nodes[0].kind,
            parent.compute_nodes[1].kind,
            parent.compute_nodes[2].kind,
            parent.compute_nodes[2].kind,
        ],
        "each copy sits at c_i + i + 1"
    );
    // Node 1 (now index 2) still reads node 0 (index 0) and node 2 (index 3).
    assert_eq!(
        child.compute_nodes[2].inputs[0].source,
        GraphSource::ComputeNode(0)
    );
    assert_eq!(
        child.compute_nodes[2].inputs[1].source,
        GraphSource::ComputeNode(3)
    );
    // The copy of node 2 (index 4) reads the copy of node 0's consumer chain
    // through the surviving external source, shifted to its final index.
    assert_eq!(
        child.compute_nodes[4].inputs[0].source,
        GraphSource::ComputeNode(2)
    );
    // The copy of node 0 (index 1) keeps its intra-cluster external read of
    // node 1 at its shifted index.
    assert_eq!(
        child.compute_nodes[1].inputs[1].source,
        GraphSource::ComputeNode(2)
    );
}

#[test]
fn copy_operators_skip_when_the_copy_would_exceed_the_index_space() {
    let mut def = CgpGraphBackendDef {
        compute_nodes: vec![
            ComputeNode {
                kind: ComputeNodeKind::Add,
                inputs: Vec::new(),
                plasticity: None,
            };
            u16::MAX as usize
        ],
        output_sinks: Vec::new(),
        action_bank: Vec::new(),
        execute_gate: ExecuteGate { inputs: Vec::new() },
    };
    assert_eq!(
        copy_compute_node(&mut def, &mut SmallRng::seed_from_u64(1)),
        Err(MutationSkipReason::NoApplicableTarget)
    );
    assert_eq!(
        copy_cgp_subgraph(&mut def, &mut SmallRng::seed_from_u64(1)),
        Err(MutationSkipReason::NoApplicableTarget)
    );
    assert_eq!(def.compute_nodes.len(), u16::MAX as usize);
}

// ── Activation equivalence ─────────────────────────────────────────────────

#[test]
fn activating_a_copy_read_by_a_backward_edge_reproduces_the_original() {
    let parent = phase_def();
    let refs = base_input_refs();
    let mut child = parent.clone();
    child.duplicate_compute_nodes_in_place(&[1]);
    activate_copies(&mut child, &[1]);
    assert_sequences_equivalent(
        "phase_backward_edge",
        &parent,
        &refs,
        &child,
        &refs,
        &ample_runtime_config(),
    );
}

/// Pick an ascending, distinct cluster of compute indices from `rng`,
/// always non-empty.
fn random_cluster(len: usize, rng: &mut impl Rng) -> Vec<usize> {
    let cluster: Vec<usize> = (0..len).filter(|_| rng.gen_bool(0.5)).collect();
    if cluster.is_empty() {
        vec![rng.gen_range(0..len)]
    } else {
        cluster
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    /// Duplicating a node or a cluster is silent when it fires, and stays
    /// equivalent once every outside edge that read a member is retargeted to
    /// that member's copy across all five surfaces.
    #[test]
    fn copy_activation_is_equivalent_over_fixtures_and_seeds(seed in any::<u64>()) {
        for (label, parent, refs) in activation_fixtures() {
            let mut rng = SmallRng::seed_from_u64(seed);
            let len = parent.compute_nodes.len();
            let single = vec![rng.gen_range(0..len)];
            for sources in [single, random_cluster(len, &mut rng)] {
                let mut child = parent.clone();
                child.duplicate_compute_nodes_in_place(&sources);
                assert_neutral(
                    &format!("duplicate[{label}]{sources:?}"),
                    &parent,
                    &refs,
                    &child,
                    &refs,
                    &ample_runtime_config(),
                );
                activate_copies(&mut child, &sources);
                assert_sequences_equivalent(
                    &format!("activate[{label}]{sources:?}"),
                    &parent,
                    &refs,
                    &child,
                    &refs,
                    &ample_runtime_config(),
                );
            }
        }
    }
}

// ── Split exclusion (replacing T11.F03's documented exception) ─────────────

/// A graph whose only splittable edge is one live introspection reference
/// read directly by an action slot's parameter surface.
fn introspection_edge_def(plasticity: bool, on_compute_input: bool) -> CgpGraphBackendDef {
    let edge = GraphEdge {
        source: GraphSource::InputLeaf {
            ref_idx: 0,
            sub_idx: 0,
        },
        weight: 1.0,
    };
    CgpGraphBackendDef {
        compute_nodes: vec![ComputeNode {
            kind: ComputeNodeKind::WeightedSum,
            inputs: if on_compute_input {
                vec![edge.clone()]
            } else {
                Vec::new()
            },
            plasticity: plasticity.then(|| PlasticityConfig {
                rule: crate::creature::genome::HebbianRule::Classic,
                learning_rate: 0.1,
                weight_clamp: 5.0,
                lamarckian: false,
                modulation: None,
            }),
        }],
        output_sinks: Vec::new(),
        action_bank: vec![ActionSlot {
            behavior: ActionSlotBehavior::Emit(WorldActionKind::Move),
            gate_inputs: Vec::new(),
            param_inputs: if on_compute_input {
                Vec::new()
            } else {
                vec![edge]
            },
        }],
        execute_gate: ExecuteGate { inputs: Vec::new() },
    }
}

fn introspection_refs() -> Vec<InputReference> {
    vec![InputReference::DynamicIntrospection(
        DynamicIntrospectionKey::EnergyCurrent,
    )]
}

#[test]
fn split_skips_a_live_introspection_edge_into_a_non_compute_consumer_under_plasticity() {
    let parent = introspection_edge_def(true, false);
    for seed in 0u64..16 {
        let mut child = parent.clone();
        assert_eq!(
            split_existing_edge(
                &mut child,
                &introspection_refs(),
                &mut SmallRng::seed_from_u64(seed)
            ),
            Err(MutationSkipReason::NoApplicableTarget),
            "seed {seed}"
        );
        assert_eq!(child, parent, "seed {seed}: the skip changes nothing");
    }
}

#[test]
fn split_exclusion_is_scoped_to_all_three_of_its_conditions() {
    let world_refs = vec![InputReference::World(WorldInputKey::NeighborBarrierRing)];
    for (label, def, refs) in [
        (
            "no plasticity",
            introspection_edge_def(false, false),
            introspection_refs(),
        ),
        (
            "compute consumer",
            introspection_edge_def(true, true),
            introspection_refs(),
        ),
        (
            "not introspection",
            introspection_edge_def(true, false),
            world_refs,
        ),
    ] {
        let mut child = def.clone();
        assert_eq!(
            split_existing_edge(&mut child, &refs, &mut SmallRng::seed_from_u64(3)),
            Ok(()),
            "{label}: outside the exclusion, the split still applies"
        );
        assert_eq!(child.compute_nodes.len(), def.compute_nodes.len() + 1);
    }
}
