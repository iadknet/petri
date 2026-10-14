//! Golden Lamarckian corpus (T25.F01 oracle).
//!
//! Each scenario captures a fixed parent's learned weights on a seeded draft
//! of a fixed two-node graph genome with Lamarckian compute nodes, applies one
//! named operator (or one engine event), builds the child's weights and hashes
//! the child genome's `Debug` text with the built weights' bits. The hashes
//! were recorded on main before the refactor; any change in what a birth
//! inherits moves one. The scenarios search seeds deterministically for the
//! branch they cover and assert it, so coverage does not depend on luck.
//!
//! Only the adapter section below touches the birth-payload API.

use rand::rngs::SmallRng;
use rand::SeedableRng;
use sha2::{Digest, Sha256};

use super::cgp_reproduction;
use crate::config::{FounderProfile, MutationConfig};
use crate::contracts::{InputReference, NodeId, RouteTarget, WorldInputKey};
use crate::creature::founder::founder_genome;
use crate::creature::genome::cgp::{
    CgpGraphBackendDef, GraphEdge, GraphSource, OutputSinkKind,
};
use crate::creature::genome::vote::VoteSink;
use crate::creature::genome::{BackendDef, CreatureGenome, HebbianRule, PlasticityConfig};
use crate::mutation::graph::{GraphMutator, GraphOperator};
use crate::mutation::input_ref::{InputRefMutator, InputRefOperator};
use crate::mutation::reachability::{ParentExecuted, TargetSelector};
use crate::mutation::topology::{TopologyMutator, TopologyOperator};
use crate::mutation::{MutationEngine, MutationSkipReason};

type Weights = Vec<Vec<Box<[f32]>>>;
type Seeds = Vec<Option<Vec<Vec<Option<f32>>>>>;

// ── Adapter: the only code that names the birth-payload API ────────────────

/// Capture `parent` on every graph node of the draft, as reproduction does.
fn capture(genome: &mut CreatureGenome, parent: &Weights) {
    for (idx, node) in genome.nodes.iter_mut().enumerate() {
        if let BackendDef::Graph(def) = &mut node.backend_def {
            cgp_reproduction::capture_birth_weights(
                def,
                parent.get(idx).map_or(&[], Vec::as_slice),
            );
        }
    }
}

/// Consume the draft's payload into the newborn's weights.
fn build(genome: &mut CreatureGenome) -> Weights {
    let mut result = Vec::new();
    for (idx, node) in genome.nodes.iter_mut().enumerate() {
        if let BackendDef::Graph(def) = &mut node.backend_def {
            result.resize_with(idx + 1, Vec::new);
            result[idx] = cgp_reproduction::build_cgp_child_plasticity_weights(def);
        }
    }
    result
}

/// The payload rows of every graph node, `None` where a node carries none.
fn seeds(genome: &CreatureGenome) -> Seeds {
    genome
        .nodes
        .iter()
        .filter_map(|node| match &node.backend_def {
            BackendDef::Graph(def) => Some(def.birth_weights.clone()),
            BackendDef::Vm(_) => None,
        })
        .collect()
}

// ── Fixture ─────────────────────────────────────────────────────────────────

fn graph(genome: &CreatureGenome, node: usize) -> &CgpGraphBackendDef {
    match &genome.nodes[node].backend_def {
        BackendDef::Graph(def) => def,
        BackendDef::Vm(_) => panic!("node {node} is a graph node"),
    }
}

fn graph_mut(genome: &mut CreatureGenome, node: usize) -> &mut CgpGraphBackendDef {
    match &mut genome.nodes[node].backend_def {
        BackendDef::Graph(def) => def,
        BackendDef::Vm(_) => panic!("node {node} is a graph node"),
    }
}

fn plasticity(compute: usize) -> Option<PlasticityConfig> {
    (compute % 3 != 2).then_some(PlasticityConfig {
        rule: HebbianRule::Classic,
        learning_rate: 0.1,
        weight_clamp: 5.0,
        lamarckian: compute % 3 == 0,
        modulation: None,
    })
}

/// The founder's two graph nodes with: an unaddressed reference at node 0's
/// index 0 (`Prune`'s target, every other index one higher), a ring group on
/// node 1's move sinks (`RefineHeritableStructure`'s target), a
/// shared-memory edge on a compute node, and plasticity on two of every
/// three compute nodes, half of those Lamarckian.
fn fixture() -> CreatureGenome {
    let mut genome = founder_genome(FounderProfile::V3Alpha1);
    genome.nodes[0]
        .input_refs
        .insert(0, InputReference::World(WorldInputKey::NeighborBarrierRing));
    {
        let def = graph_mut(&mut genome, 0);
        let edges = def
            .compute_nodes
            .iter_mut()
            .flat_map(|node| node.inputs.iter_mut())
            .chain(def.output_sinks.iter_mut().flat_map(|sink| sink.inputs.iter_mut()));
        for edge in edges {
            if let GraphSource::InputLeaf { ref_idx, .. } = &mut edge.source {
                *ref_idx += 1;
            }
        }
        for projection in &mut def.projections {
            projection.ref_idx += 1;
        }
    }
    let ring = u16::try_from(genome.nodes[1].input_refs.len()).unwrap();
    genome.nodes[1]
        .input_refs
        .push(InputReference::World(WorldInputKey::NeighborOccupiedRing));
    {
        let def = graph_mut(&mut genome, 1);
        for direction in [0u8, 2] {
            def.sink_mut(OutputSinkKind::ActionVote(VoteSink::Move(direction)))
                .unwrap()
                .inputs
                .push(GraphEdge {
                    source: GraphSource::InputLeaf {
                        ref_idx: ring,
                        sub_idx: u16::from(direction),
                    },
                    weight: 0.3,
                });
        }
        def.compute_nodes[1].inputs.push(GraphEdge {
            source: GraphSource::SharedMemory {
                slot: 3,
                previous: false,
            },
            weight: 0.25,
        });
    }
    for node in 0..2 {
        let def = graph_mut(&mut genome, node);
        for compute in 0..def.compute_nodes.len() {
            def.compute_nodes[compute].plasticity = plasticity(compute);
        }
    }
    // A third node, a copy of node 1 that node 1 routes to, so a middle node
    // can be removed.
    let mut third = genome.nodes[1].clone();
    third.node_id = NodeId::new(2);
    genome.nodes[1].targets = vec![RouteTarget {
        target_id: NodeId::new(2),
        slot: 0,
        gate_bias: 0.0,
    }];
    genome.nodes.push(third);
    genome
}

/// A parent's learned weights for `genome`: full rows, short rows (one value
/// fewer than the node's inputs) and empty rows.
fn parent(genome: &CreatureGenome) -> Weights {
    (0..genome.nodes.len())
        .map(|node| {
            graph(genome, node)
                .compute_nodes
                .iter()
                .enumerate()
                .map(|(compute, cn)| {
                    let len = if compute % 5 == 4 {
                        0
                    } else if compute % 4 == 3 {
                        cn.inputs.len().saturating_sub(1)
                    } else {
                        cn.inputs.len()
                    };
                    (0..len)
                        .map(|edge| (100 * node + 10 * compute + edge) as f32 + 0.5)
                        .collect()
                })
                .collect()
        })
        .collect()
}

// ── Branch observations ─────────────────────────────────────────────────────

fn compute_edges(def: &CgpGraphBackendDef) -> usize {
    def.compute_nodes.iter().map(|node| node.inputs.len()).sum()
}

fn sink_edges(def: &CgpGraphBackendDef) -> usize {
    def.output_sinks.iter().map(|sink| sink.inputs.len()).sum()
}

fn router_gate_edges(def: &CgpGraphBackendDef) -> usize {
    def.output_sinks
        .iter()
        .filter(|sink| matches!(sink.kind, OutputSinkKind::RouterGate(_)))
        .map(|sink| sink.inputs.len())
        .sum()
}

fn all_sources(def: &CgpGraphBackendDef) -> Vec<GraphSource> {
    def.edges().map(|edge| edge.source).collect()
}

/// How many edges, on either surface, read compute node `idx`.
fn readers(def: &CgpGraphBackendDef, idx: usize) -> usize {
    def.edges()
        .filter(|edge| edge.source == GraphSource::ComputeNode(idx as u16))
        .count()
}

/// The one edge whose source changed, when the shapes agree.
fn changed_source(before: &CgpGraphBackendDef, after: &CgpGraphBackendDef) -> Option<(GraphSource, GraphSource)> {
    let (b, a) = (all_sources(before), all_sources(after));
    if b.len() != a.len() {
        return None;
    }
    let changed: Vec<_> = b.into_iter().zip(a).filter(|(x, y)| x != y).collect();
    (changed.len() == 1).then(|| changed[0])
}

/// The compute-node count grew by one and the new last node is (`appended`)
/// or is not a fresh append, with `inputs` inputs and `read` readers.
fn added_node(before: &CgpGraphBackendDef, after: &CgpGraphBackendDef, inputs: usize, read: bool) -> bool {
    let len = after.compute_nodes.len();
    len == before.compute_nodes.len() + 1
        && after.compute_nodes[len - 1].inputs.len() == inputs
        && (readers(after, len - 1) > 0) == read
}

type Branch = fn(&CgpGraphBackendDef, &CgpGraphBackendDef) -> bool;

#[derive(Clone, Copy)]
enum Event {
    Graph(GraphOperator, usize, Branch),
    InputRef(InputRefOperator, usize, Branch),
    Topology(TopologyOperator),
}

fn any(_: &CgpGraphBackendDef, _: &CgpGraphBackendDef) -> bool {
    true
}

const SCENARIOS: &[(&str, Event)] = &[
    ("remove compute node, readers cleared", Event::Graph(GraphOperator::RemoveInternalGraphNode, 1, |b, a| {
        a.compute_nodes.len() + 1 == b.compute_nodes.len()
            && all_sources(a).contains(&GraphSource::ComputeNode(u16::MAX))
    })),
    // Node 1's last compute node is the transfer constant, read by its
    // parameter sink: an append makes an unread last node, a split onto a
    // sink an `Add` read by the sink, a split onto a compute input leaves the
    // constant last.
    ("append: disconnected form", Event::Graph(GraphOperator::AddInternalGraphNode, 1, |b, a| added_node(b, a, 0, false))),
    ("append: bootstrap form", Event::Graph(GraphOperator::AddInternalGraphNode, 1, |b, a| added_node(b, a, 1, false))),
    ("split: sink consumer (append)", Event::Graph(GraphOperator::AddInternalGraphNode, 1, |b, a| added_node(b, a, 1, true))),
    ("split: compute-input consumer (insert)", Event::Graph(GraphOperator::AddInternalGraphNode, 1, |b, a| added_node(b, a, 0, true))),
    ("duplicate: copy internal node", Event::Graph(GraphOperator::CopyInternalNode, 1, any)),
    ("duplicate: copy subgraph", Event::Graph(GraphOperator::CopySubgraph, 1, any)),
    ("push edge: compute surface", Event::Graph(GraphOperator::AddGraphEdge, 1, |b, a| compute_edges(a) == compute_edges(b) + 1)),
    ("push edge: sink surface", Event::Graph(GraphOperator::AddGraphEdge, 1, |b, a| sink_edges(a) == sink_edges(b) + 1)),
    ("remove edge: compute surface", Event::Graph(GraphOperator::RemoveGraphEdge, 1, |b, a| compute_edges(a) + 1 == compute_edges(b))),
    ("remove edge: sink surface", Event::Graph(GraphOperator::RemoveGraphEdge, 1, |b, a| sink_edges(a) + 1 == sink_edges(b))),
    ("retarget: source changed", Event::Graph(GraphOperator::RetargetGraphEdge, 1, |b, a| b != a)),
    ("retarget: same source drawn", Event::Graph(GraphOperator::RetargetGraphEdge, 1, |b, a| b == a)),
    ("raw field: InputLeaf ref", Event::Graph(GraphOperator::GraphRawFieldMutation, 1, |b, a| matches!(
        changed_source(b, a),
        Some((GraphSource::InputLeaf { ref_idx: r0, sub_idx: s0 }, GraphSource::InputLeaf { ref_idx: r1, sub_idx: s1 })) if r0 != r1 && s0 == s1
    ))),
    ("raw field: InputLeaf sub", Event::Graph(GraphOperator::GraphRawFieldMutation, 1, |b, a| matches!(
        changed_source(b, a),
        Some((GraphSource::InputLeaf { ref_idx: r0, sub_idx: s0 }, GraphSource::InputLeaf { ref_idx: r1, sub_idx: s1 })) if r0 == r1 && s0 != s1
    ))),
    ("raw field: SharedMemory slot", Event::Graph(GraphOperator::GraphRawFieldMutation, 1, |b, a| matches!(
        changed_source(b, a),
        Some((GraphSource::SharedMemory { slot: s0, .. }, GraphSource::SharedMemory { slot: s1, .. })) if s0 != s1
    ))),
    ("raw field: SharedMemory previous flag", Event::Graph(GraphOperator::GraphRawFieldMutation, 1, |b, a| matches!(
        changed_source(b, a),
        Some((GraphSource::SharedMemory { previous: p0, .. }, GraphSource::SharedMemory { previous: p1, .. })) if p0 != p1
    ))),
    ("raw field: ComputeNode index", Event::Graph(GraphOperator::GraphRawFieldMutation, 1, |b, a| matches!(
        changed_source(b, a),
        Some((GraphSource::ComputeNode(_), GraphSource::ComputeNode(_)))
    ))),
    ("alter weight", Event::Graph(GraphOperator::AlterGraphEdgeWeight, 1, |b, a| b != a)),
    ("refinement weight write", Event::Graph(GraphOperator::RefineHeritableStructure, 1, |b, a| b != a)),
    ("copy edge bundle", Event::Graph(GraphOperator::CopyEdgeBundle, 1, any)),
    ("compute kind: swap operator", Event::Graph(GraphOperator::SwapGraphOperator, 1, any)),
    ("compute kind: operator param", Event::Graph(GraphOperator::MutateGraphOperatorParam, 1, any)),
    ("plasticity: enable", Event::Graph(GraphOperator::EnableHebbian, 1, any)),
    ("plasticity: disable", Event::Graph(GraphOperator::DisableHebbian, 1, any)),
    ("plasticity: rule", Event::Graph(GraphOperator::MutateHebbianRule, 1, any)),
    ("plasticity: rate", Event::Graph(GraphOperator::MutateHebbianRate, 1, any)),
    ("plasticity: Lamarckian toggle", Event::Graph(GraphOperator::ToggleHebbianLamarckian, 1, any)),
    ("plasticity: enable modulation", Event::Graph(GraphOperator::EnableRewardModulation, 1, any)),
    ("recruitment: neutral input", Event::Graph(GraphOperator::RecruitNeutralInput, 1, |b, a| b != a)),
    ("Prune reindex", Event::InputRef(InputRefOperator::Prune, 0, |b, a| b != a)),
    ("recruitment: declared input sink push", Event::InputRef(InputRefOperator::Add, 1, |b, a| sink_edges(a) == sink_edges(b) + 1)),
    ("routing: gate sink push", Event::Topology(TopologyOperator::AddRouteTarget)),
    ("topology: CopyNode", Event::Topology(TopologyOperator::CopyNode)),
    ("topology: CopyMeshBackwardSlice", Event::Topology(TopologyOperator::CopyMeshBackwardSlice)),
    ("topology: CopyMeshForwardSlice", Event::Topology(TopologyOperator::CopyMeshForwardSlice)),
    ("topology: RemoveNode", Event::Topology(TopologyOperator::RemoveNode)),
    ("topology: Swap", Event::Topology(TopologyOperator::SwapNodeBackend)),
];

/// Recorded on main before the refactor; scenario order as in [`SCENARIOS`],
/// then the rollback and the all-empty parent.
const PINNED: &[&str] = &[
    "66eba85cba7ab6a1d4438b01fb7521c4ecf90af85df5eb57a7d80d5dd0c8a179",
    "0263adcba61cfdfba5eb973d778c344ecaadc484d5b00a13434f3479ffc0a3b9",
    "ba5a82abd41f60482e941d1472d4e79ae5687cd61971f28e913b8a0a892faf4a",
    "0cefd7df3cf9d0abd4a9df859a5c8341fcf3d8daa2f1142eabcda0518228c0da",
    "4d0a19e20f7a95e70db1490485359f56a08337abcc306ea1e07ee1f2f5815505",
    "b055d7b3415816b30553d15a0db09b795b8c6edf74835ce272a373a852fcb0c9",
    "d5ebfb4d77a62f7978682e327dc209bff91e69c1dd4eb0a3a296218f5c3649ce",
    "fdf5d08134bc88e29288eb2133bf6270ea54c19adc1d7b38f35c0712ede50ba9",
    "e00263c7d2d97c20af2418e0112c2b70fd83cf111b92c11bf0135335994e0276",
    "722afaf421e941553883617ecf31450e06efbb78e40c7c5ffb558058791448e1",
    "571b865cbfa12f1e666758d0a904747a22d53b02736fb0fd5030d0cac5bb9634",
    "4e238377c0ca0a9545072e45da8eb0872c4b103778484fdca56e08515b96bfa5",
    "2551f1a0936d1572937f91e00c117fd36f71519105a4d7f2ab9e25dba9e9649c",
    "d9c60d48c29598ea043c61d9e9bcb42da6645d51a90e6abbca32354f3c5ad365",
    "9419a6949d7850c223c585b765eb30224496a4bf6ebd29ce9ffed1b0d5dae9d3",
    "e79b4abf4a6ebbeca6137e5e87215e3894ded7aeae9c2edeb14991d8a23e4ef5",
    "397f0974951266fb916849d6e18ca567597f72b19c99484b28f8d56c7707f419",
    "9a73453edd3a3009da635379d75df3c1668cf04eec0927a9bc1ea4d97b937a6f",
    "0d6351550167aa4ba30b4f2e8a31fb435186e01f3f1dbfd4f9a541ec5b5029b0",
    "cf16b266637af983eeb828c82cb36575ea0aef9658326e166dd20a12944d541d",
    "d299bddad6a97bf1d8a3513ed3bc6762ef5d6938b828d5f2ed9415d1d7f109bd",
    "3f9adc0c00022a6ecc3228d5e139a5d60b41d61923bfdd997c6be0206ac6789b",
    "65980bf00a5961fd6247f37a3a01f873afa646c93254c88f7abe4ff78f2ef729",
    "2c9cde80427ff3ba7944c490fcb9d07aa55f134890614e45bc2faf47afcc75d7",
    "d886e97689e9c0cf6377395efd9bacf7b07a05a1c9438c9cf1301e50df26ff9e",
    "404b2bc91cf3940c1b5c1911e70bb073f55f3fe4562feb7f0621dbb78199a655",
    "508e4d412f7e5c858df7e482f26f7de8e686bbd7be52394004ddb5e50f1107d5",
    "45c0811f4bfea9805ff48e2677dc6988f12e1e26ac1da9787bcac78b1515767f",
    "4d5fa40728206d0eca9129d43179e375994613c1ff29102d59cfb92962310158",
    "ca9d26a6d753c9b5c2a3db90af81f76b0f5516f6c0526b26abe0869e7d8efb3f",
    "311123d7c50a98a0c4e31a147e3a3fe29fe1d2a39114609bd6d9d9bcddb5bdd6",
    "3bc0ed6cd178a4ae02c67724f42649b107a44eb2324060fb9f5d13c931823b58",
    "497205998a3088c2359aff7380f744913f6be4dca8cac8bfcec267065115c051",
    "0979eb7df983dadb940528c9e31194b517386a32efc314439f9c6d4a4f8de20e",
    "b349db97072f620fe6d8a23b88cf2b5ec3e94705c7083f6991f122ad27dea39d",
    "e57c264e2d146909de26a8c3f07bbfc69c015d6d8136d312483b8b8df1f9671e",
    "36e46b4a55ce65d778996d87a4a3c036cbd7d80fa95bb1bee05c35e83445dc24",
    "c6f8cb55a60b652af3275062f0e8e26cc64992e8555228e3c6ecb3ab195e7c89",
    "264b85816dfc0c3c70bc223b6dd1b9ffbd6abb0eecb0fced5e37f6efb601facc",
    "c2683ab3b31ef478321db898c94e907f1958b686c4ffc0610d104bbf558af178",
];

fn apply(event: Event, genome: &mut CreatureGenome, rng: &mut SmallRng) -> bool {
    let config = MutationConfig::default();
    let before = genome.clone();
    match event {
        Event::Graph(op, node, branch) => {
            GraphMutator::apply_to_node(genome, op, node, rng, &config, 1).is_ok()
                && branch(graph(&before, node), graph(genome, node))
        }
        Event::InputRef(op, node, branch) => {
            InputRefMutator::apply_to_node(genome, op, node, rng, 1).is_ok()
                && branch(graph(&before, node), graph(genome, node))
        }
        Event::Topology(op) => {
            let ok = TopologyMutator::apply(
                genome,
                op,
                &mut TargetSelector::reachable_only(&[0, 1, 2], 0.0),
                rng,
                &config,
            )
            .is_ok();
            ok && (op != TopologyOperator::AddRouteTarget
                || router_gate_edges(graph(genome, 0)) == router_gate_edges(graph(&before, 0)) + 1)
        }
    }
}

fn digest(seed: u64, genome: &CreatureGenome, weights: &Weights) -> String {
    let mut hash = Sha256::new();
    hash.update(seed.to_le_bytes());
    hash.update(format!("{genome:?}").as_bytes());
    hash.update((weights.len() as u64).to_le_bytes());
    for node in weights {
        hash.update((node.len() as u64).to_le_bytes());
        for row in node {
            hash.update((row.len() as u64).to_le_bytes());
            for value in row.iter() {
                hash.update(value.to_bits().to_le_bytes());
            }
        }
    }
    hex::encode(hash.finalize())
}

fn scenario_digest(name: &str, event: Event, base: &CreatureGenome, parent: &Weights) -> String {
    for seed in 0..4096 {
        let mut child = base.clone();
        capture(&mut child, parent);
        assert!(
            seeds(&child).iter().all(Option::is_some),
            "{name}: every graph node carries a payload"
        );
        if apply(event, &mut child, &mut SmallRng::seed_from_u64(seed)) {
            let weights = build(&mut child);
            assert!(
                weights.iter().flatten().flat_map(|row| row.iter()).any(|&v| v >= 10.0),
                "{name}: the child inherits learned values"
            );
            return digest(seed, &child, &weights);
        }
    }
    panic!("{name}: no seed below 4096 applies the scenario's branch");
}

fn rollback_digest(base: &CreatureGenome, parent: &Weights) -> String {
    let mut config = MutationConfig::default();
    config.per_unit_rate = 1.0;
    let mut broken = base.clone();
    broken.nodes.push(broken.nodes[0].clone());
    let mut parent = parent.clone();
    parent.push(parent[0].clone());
    for seed in 0..4096 {
        let mut child = broken.clone();
        capture(&mut child, &parent);
        let before = child.clone();
        let captured = seeds(&child);
        assert!(captured.iter().all(Option::is_some));
        let summary = MutationEngine::apply_mutations_with_food_type_count(
            &mut child,
            &config,
            &[0, 1, 2, 3],
            ParentExecuted::Indices(&[]),
            &mut SmallRng::seed_from_u64(seed),
            1,
        );
        if summary
            .skip_reasons
            .get(&MutationSkipReason::ParseabilityViolation)
            .is_some_and(|&n| n > 0)
        {
            assert_eq!(summary.applied_events, 0);
            assert_eq!(child, before);
            assert_eq!(seeds(&child), captured, "rolled-back seeds survive");
            let weights = build(&mut child);
            return digest(seed, &child, &weights);
        }
    }
    panic!("rollback: no seed below 4096 draws a rejected event");
}

fn all_empty_digest(base: &CreatureGenome) -> String {
    let mut child = base.clone();
    capture(&mut child, &Vec::new());
    let weights = build(&mut child);
    digest(0, &child, &weights)
}

#[test]
fn golden_lamarckian_corpus_is_pinned() {
    let base = fixture();
    let parent = parent(&base);
    let mut digests: Vec<String> = SCENARIOS
        .iter()
        .map(|&(name, event)| scenario_digest(name, event, &base, &parent))
        .collect();
    digests.push(rollback_digest(&base, &parent));
    digests.push(all_empty_digest(&base));
    let rendered: Vec<String> = SCENARIOS
        .iter()
        .map(|(name, _)| *name)
        .chain(["rollback (ParseabilityViolation)", "all-empty parent"])
        .zip(&digests)
        .map(|(name, digest)| format!("{name}: {digest}"))
        .collect();
    assert_eq!(
        digests.iter().map(String::as_str).collect::<Vec<_>>(),
        PINNED,
        "\n{}",
        rendered.join("\n")
    );
}
