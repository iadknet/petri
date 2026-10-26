//! T11.F08 graph tests: a copied compute node or cluster is placed so every
//! copied edge reads the same evaluation phase its original reads, so the copy
//! reproduces its original when a later mutation makes something read it; and
//! `split_existing_edge` splits every introspection edge function-preservingly
//! (T25.F02 retired the T11.F08 exclusion with the learning charge).

use crate::genome::vote::DirectedVoteKind;
use proptest::prelude::*;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use super::operators::{
    ample_runtime_config, assert_neutral, base_def, base_input_refs, plasticity_def,
    plasticity_input_refs, scenarios,
};
use crate::config::RuntimeConfig;
use crate::contracts::{DynamicIntrospectionKey, InputReference, WorldAction};
use crate::genome::cgp::{
    CgpGraphBackendDef, ComputeNode, ComputeNodeKind, EdgeOccurrence, EdgeSurface, GraphEdge,
    GraphSource, OutputSink, OutputSinkKind,
};
use crate::genome::vote::{ActionParamField, VoteSink};
use crate::genome::PlasticityConfig;
use crate::mutation::graph::operators::{
    copy_cgp_subgraph, copy_compute_node, split_existing_edge,
};
use crate::mutation::types::MutationSkipReason;
use crate::runtime::cgp::execute_graph_node;
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
            let mut graph_runtime = crate::genome::cgp::test_support::runtime_for(def);
            (0..SEQUENCE_TICKS)
                .map(|_| {
                    let prev_shared_memory = shared_memory;
                    let mut energy = 1.0e6f32;
                    let mut side_outputs = MeshSideOutputs::new(8);
                    graph_runtime.begin_tick(0);
                    let result = execute_graph_node(
                        def,
                        input_refs,
                        &[0.0f32; OUTPUT_SLOT_COUNT],
                        &mut energy,
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

/// Activate the copies made by `duplicate_compute_nodes(sources)`:
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
    let rewired: Vec<(EdgeOccurrence, u16)> = def
        .edge_sites()
        .filter(|occurrence| {
            !matches!(occurrence.surface, EdgeSurface::ComputeInput(i) if duplicated.contains(&i))
        })
        .filter_map(|occurrence| {
            let GraphSource::ComputeNode(idx) = def.edge_at(occurrence).source() else {
                return None;
            };
            retarget
                .iter()
                .find(|(original, _)| *original == idx)
                .map(|&(_, copy)| (occurrence, copy))
        })
        .collect();
    for (occurrence, copy) in rewired {
        def.edge_mut(occurrence)
            .set_source(GraphSource::ComputeNode(copy));
    }
}

/// A graph built around evaluation phase: node 1 reads the higher-index node 2
/// (a frozen tick-start read), node 0 reads node 1 backwards, node 2 reads
/// node 1 forwards, and a non-compute surface reads node 1 directly. Copying
/// node 1 anywhere but immediately after itself changes what the copy reads,
/// what a consumer of the copy reads, or both.
pub(super) fn phase_def() -> CgpGraphBackendDef {
    CgpGraphBackendDef::new_with_fixed_outputs()
        .with_compute_nodes(vec![
            ComputeNode::new(
                ComputeNodeKind::WeightedSum,
                vec![
                    GraphEdge::new(
                        GraphSource::InputLeaf {
                            ref_idx: 0,
                            sub_idx: 3,
                        },
                        0.5,
                    ),
                    GraphEdge::new(GraphSource::ComputeNode(1), 0.7),
                ],
            ),
            ComputeNode::new(
                ComputeNodeKind::WeightedSum,
                vec![
                    GraphEdge::new(GraphSource::ComputeNode(0), 0.6),
                    GraphEdge::new(GraphSource::ComputeNode(2), 0.9),
                ],
            ),
            ComputeNode::new(
                ComputeNodeKind::DecayIntegrator(0.4),
                vec![GraphEdge::new(GraphSource::ComputeNode(1), 1.0)],
            ),
        ])
        .with_output_sinks(vec![
            OutputSink::new(
                OutputSinkKind::CustomOutput(0),
                vec![GraphEdge::new(GraphSource::ComputeNode(2), 1.0)],
            ),
            OutputSink::new(
                OutputSinkKind::ActionVote(VoteSink::Directed(DirectedVoteKind::Move, 2)),
                vec![GraphEdge::new(GraphSource::ComputeNode(0), 1.0)],
            ),
            OutputSink::new(
                OutputSinkKind::ActionParam(ActionParamField::ReproduceTransferFraction),
                vec![GraphEdge::new(GraphSource::ComputeNode(1), 1.0)],
            ),
            OutputSink::new(
                OutputSinkKind::ActionVote(VoteSink::Terminate),
                vec![GraphEdge::new(GraphSource::ComputeNode(0), 1.0)],
            ),
        ])
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
    // `base_def`'s three nodes carry distinct kinds, so the copy's position
    // identifies the source it was taken from.
    let parent = base_def();
    let mut seen = [false; 3];
    for seed in 0u64..48 {
        let mut child = parent.clone();
        copy_compute_node(&mut child, &mut SmallRng::seed_from_u64(seed)).unwrap();
        assert_eq!(
            child.compute_nodes().len(),
            parent.compute_nodes().len() + 1
        );
        let source = (0..parent.compute_nodes().len())
            .find(|&i| child.compute_nodes()[i + 1].kind == parent.compute_nodes()[i].kind)
            .expect("the copy sits directly after one of the original nodes");
        seen[source] = true;
        assert_eq!(
            child.compute_nodes()[source + 1].kind,
            parent.compute_nodes()[source].kind,
        );
        assert_eq!(
            child.compute_nodes()[source + 1]
                .inputs
                .iter()
                .map(GraphEdge::plasticity)
                .collect::<Vec<_>>(),
            parent.compute_nodes()[source]
                .inputs
                .iter()
                .map(GraphEdge::plasticity)
                .collect::<Vec<_>>(),
        );
    }
    assert!(seen.iter().all(|&s| s), "every source index is reachable");
}

#[test]
fn copy_internal_node_shifts_the_copied_inputs_and_follows_a_self_edge() {
    let mut child = base_def();
    // Node 2 carries a self-loop and node 1 reads the lower-index node 0.
    child.duplicate_compute_nodes(&[1]);
    assert_eq!(
        child.compute_nodes()[2].inputs[0].source(),
        GraphSource::ComputeNode(0),
        "a source below the original stays below the copy"
    );
    assert_eq!(
        child.compute_nodes()[3].inputs[0].source(),
        GraphSource::ComputeNode(3),
        "the self-loop node itself shifted with the insertion"
    );

    let mut self_child = base_def();
    self_child.duplicate_compute_nodes(&[2]);
    assert_eq!(
        self_child.compute_nodes()[3].inputs[0].source(),
        GraphSource::ComputeNode(3),
        "a self-edge on the copy reads the copy, not the original"
    );
    assert_eq!(
        self_child.compute_nodes()[2].inputs[0].source(),
        GraphSource::ComputeNode(2),
        "the original keeps reading itself"
    );
}

#[test]
fn copy_subgraph_inserts_every_copy_directly_after_its_member() {
    let mut child = phase_def();
    child.duplicate_compute_nodes(&[0, 2]);
    let kinds: Vec<ComputeNodeKind> = child.compute_nodes().iter().map(|n| n.kind).collect();
    let parent = phase_def();
    assert_eq!(
        kinds,
        vec![
            parent.compute_nodes()[0].kind,
            parent.compute_nodes()[0].kind,
            parent.compute_nodes()[1].kind,
            parent.compute_nodes()[2].kind,
            parent.compute_nodes()[2].kind,
        ],
        "each copy sits at c_i + i + 1"
    );
    // Node 1 (now index 2) still reads node 0 (index 0) and node 2 (index 3).
    assert_eq!(
        child.compute_nodes()[2].inputs[0].source(),
        GraphSource::ComputeNode(0)
    );
    assert_eq!(
        child.compute_nodes()[2].inputs[1].source(),
        GraphSource::ComputeNode(3)
    );
    // The copy of node 2 (index 4) reads the copy of node 0's consumer chain
    // through the surviving external source, shifted to its final index.
    assert_eq!(
        child.compute_nodes()[4].inputs[0].source(),
        GraphSource::ComputeNode(2)
    );
    // The copy of node 0 (index 1) keeps its intra-cluster external read of
    // node 1 at its shifted index.
    assert_eq!(
        child.compute_nodes()[1].inputs[1].source(),
        GraphSource::ComputeNode(2)
    );
}

#[test]
fn copy_operators_skip_when_the_copy_would_exceed_the_index_space() {
    // One below the limit: the copy lands at index `u16::MAX - 1`, the
    // highest index that is not `remove_compute_node`'s sentinel, so it
    // still applies.
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs()
        .with_compute_nodes(vec![
            ComputeNode::new(ComputeNodeKind::Add, Vec::new());
            u16::MAX as usize - 1
        ])
        .with_output_sinks(Vec::new());
    assert_eq!(
        copy_compute_node(&mut def, &mut SmallRng::seed_from_u64(1)),
        Ok(())
    );
    assert_eq!(def.compute_nodes().len(), u16::MAX as usize);
    // At the limit, both copy operators skip rather than address the sentinel.
    assert_eq!(
        copy_compute_node(&mut def, &mut SmallRng::seed_from_u64(1)),
        Err(MutationSkipReason::NoApplicableTarget)
    );
    assert_eq!(
        copy_cgp_subgraph(&mut def, &mut SmallRng::seed_from_u64(1)),
        Err(MutationSkipReason::NoApplicableTarget)
    );
    assert_eq!(def.compute_nodes().len(), u16::MAX as usize);
}

// ── Activation equivalence ─────────────────────────────────────────────────

#[test]
fn activating_a_copy_read_by_a_backward_edge_reproduces_the_original() {
    let parent = phase_def();
    let refs = base_input_refs();
    let mut child = parent.clone();
    child.duplicate_compute_nodes(&[1]);
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
            let len = parent.compute_nodes().len();
            let single = vec![rng.gen_range(0..len)];
            for sources in [single, random_cluster(len, &mut rng)] {
                let mut child = parent.clone();
                child.duplicate_compute_nodes(&sources);
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

// ── Introspection edges split (T25.F02 retires the T11.F08 exclusion) ─────

/// A graph whose only splittable edge is one live introspection reference
/// read directly by `sink` (or, with `on_compute_input`, by the compute node).
fn introspection_edge_def(
    plasticity: bool,
    on_compute_input: bool,
    sink: OutputSinkKind,
) -> CgpGraphBackendDef {
    let edge = GraphEdge::new(
        GraphSource::InputLeaf {
            ref_idx: 0,
            sub_idx: 0,
        },
        1.0,
    );
    CgpGraphBackendDef::new_with_fixed_outputs()
        .with_compute_nodes(vec![crate::genome::cgp::test_support::plastic_node(
            ComputeNodeKind::WeightedSum,
            if on_compute_input {
                vec![edge]
            } else {
                Vec::new()
            },
            plasticity.then_some(PlasticityConfig {
                rule: crate::genome::HebbianRule::Classic,
                learning_rate: 0.1,
                weight_clamp: 5.0,
                lamarckian: false,
                modulation: None,
                retention: 1.0,
            }),
        )])
        .with_output_sinks(vec![OutputSink::new(
            sink,
            if on_compute_input {
                Vec::new()
            } else {
                vec![edge]
            },
        )])
}

fn introspection_refs() -> Vec<InputReference> {
    vec![InputReference::DynamicIntrospection(
        DynamicIntrospectionKey::EnergyCurrent,
    )]
}

/// The shape T11.F08 excluded (a sink reading `EnergyCurrent` directly on a
/// graph carrying plasticity) splits now that no learning charge sits between
/// the evaluation and effects contexts, and the sink commits the same value
/// through the identity node as it did directly. Node evaluation is free here
/// so parent and child evaluate at the same energy: the split's extra node
/// otherwise legitimately costs one `graph_node_base_cost`.
#[test]
fn split_preserves_a_live_introspection_sink_edge_under_plasticity() {
    let parent = introspection_edge_def(true, false, OutputSinkKind::CustomOutput(0));
    let refs = introspection_refs();
    let config = RuntimeConfig {
        graph_node_base_cost: 0.0,
        ..ample_runtime_config()
    };
    for seed in 0u64..16 {
        let mut child = parent.clone();
        assert_eq!(
            split_existing_edge(&mut child, &[], &mut SmallRng::seed_from_u64(seed)),
            Ok(()),
            "seed {seed}"
        );
        assert_eq!(child.compute_nodes().len(), 2, "seed {seed}");
        assert_eq!(
            child.output_sinks()[0].inputs[0].source(),
            GraphSource::ComputeNode(1),
            "seed {seed}: the sink reads the inserted identity node"
        );
        assert_sequences_equivalent(
            &format!("seed {seed}"),
            &parent,
            &refs,
            &child,
            &refs,
            &config,
        );
        // Below the cap the read is unsaturated: 50 of 200 max energy.
        for def in [&parent, &child] {
            assert_eq!(
                committed_at_energy(def, &refs, &config, 50.0),
                0.25,
                "seed {seed}: the sink commits the evaluation energy"
            );
        }
    }
}

/// The value `def`'s `CustomOutput(0)` sink commits on one visit of the
/// first scenario starting at `energy`.
fn committed_at_energy(
    def: &CgpGraphBackendDef,
    refs: &[InputReference],
    config: &RuntimeConfig,
    mut energy: f32,
) -> f32 {
    let mut graph_runtime = crate::genome::cgp::test_support::runtime_for(def);
    graph_runtime.begin_tick(0);
    let mut shared_memory = [0.0f32; 16];
    execute_graph_node(
        def,
        refs,
        &[0.0f32; OUTPUT_SLOT_COUNT],
        &mut energy,
        0.0,
        0,
        &mut graph_runtime,
        &scenarios()[0],
        config,
        &mut MeshSideOutputs::new(8),
        &mut shared_memory,
        &[0.0f32; 16],
    )
    .output_slots[0]
}

/// The split reads neither the input references nor plasticity: every
/// former exclusion condition splits alike.
#[test]
fn split_applies_to_every_introspection_edge_shape() {
    let param = OutputSinkKind::ActionParam(ActionParamField::ReproduceTransferFraction);
    for (label, def) in [
        (
            "plasticity, sink consumer",
            introspection_edge_def(true, false, param),
        ),
        ("no plasticity", introspection_edge_def(false, false, param)),
        (
            "compute consumer",
            introspection_edge_def(true, true, param),
        ),
    ] {
        let mut child = def.clone();
        assert_eq!(
            split_existing_edge(&mut child, &[], &mut SmallRng::seed_from_u64(3)),
            Ok(()),
            "{label}: the split applies"
        );
        assert_eq!(child.compute_nodes().len(), def.compute_nodes().len() + 1);
    }
}
