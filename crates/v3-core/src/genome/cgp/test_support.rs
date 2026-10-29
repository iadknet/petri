//! Fixture editing for tests (T25.F01): a def's lists are read-only outside
//! the container, so a fixture that rewires sink inputs or compute nodes
//! freely edits plain drafts and rebuilds the def through the seedless
//! builders; the sink catalog itself stays fixed.

use super::{
    BundleBirth, BundleId, BundleSource, CgpGraphBackendDef, ComputeNode, ComputeNodeKind,
    EdgeOccurrence, EdgeSurface, GraphEdge, OutputSinkKind,
};
use crate::contracts::{FieldSet, InputReference, NodeId, SensorDescriptor};
use crate::genome::{BackendDef, CreatureGenome, Learning, NodeGenome};
use crate::runtime::state::GraphRuntimeState;

/// An editable copy of one output sink.
#[derive(Debug, Clone)]
pub(crate) struct SinkDraft {
    pub kind: OutputSinkKind,
    pub inputs: Vec<GraphEdge>,
}

/// An editable copy of one compute node.
#[derive(Debug, Clone)]
pub(crate) struct NodeDraft {
    pub kind: ComputeNodeKind,
    pub inputs: Vec<GraphEdge>,
}

/// Edit the inputs of `def`'s sink catalog as drafts, in catalog order; the
/// kinds stay fixed, and the def keeps its compute nodes, bundle records and
/// seeds (a member stays a member: the caller keeps the bundle invariants).
///
/// # Panics
///
/// If `edit` changes a draft's kind.
pub(crate) fn edit_sinks(def: &mut CgpGraphBackendDef, edit: impl FnOnce(&mut [SinkDraft])) {
    let mut drafts: Vec<SinkDraft> = def
        .output_sinks()
        .iter()
        .map(|sink| SinkDraft {
            kind: sink.kind,
            inputs: sink.inputs.to_vec(),
        })
        .collect();
    edit(&mut drafts);
    *def = drafts.into_iter().zip(OutputSinkKind::catalog()).fold(
        def.clone(),
        |def, (draft, kind)| {
            assert_eq!(draft.kind, kind, "a fixture edits sink inputs only");
            def.with_sink_inputs(kind, draft.inputs)
        },
    );
}

/// Edit `def`'s compute nodes as drafts. The rebuilt def carries no seeds.
pub(crate) fn edit_compute_nodes(
    def: &mut CgpGraphBackendDef,
    edit: impl FnOnce(&mut Vec<NodeDraft>),
) {
    let mut drafts: Vec<NodeDraft> = def
        .compute_nodes()
        .iter()
        .map(|node| NodeDraft {
            kind: node.kind,
            inputs: node.inputs.to_vec(),
        })
        .collect();
    edit(&mut drafts);
    let nodes = drafts
        .into_iter()
        .map(|draft| ComputeNode::new(draft.kind, draft.inputs))
        .collect();
    *def = def.clone().with_compute_nodes(nodes);
}

/// `edges`, each carrying `config` (T25.F06), written through the only
/// writer, [`super::EdgeMut::set_plasticity`], on a scratch compute node.
pub(crate) fn with_plasticity(edges: Vec<GraphEdge>, config: Option<Learning>) -> Vec<GraphEdge> {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs()
        .with_compute_nodes(vec![ComputeNode::new(ComputeNodeKind::Add, edges)]);
    let surface = EdgeSurface::ComputeInput(0);
    for index in 0..def.surface_edges(surface).len() {
        def.edge_mut(EdgeOccurrence::new(surface, index))
            .set_learning(config);
    }
    def.surface_edges(surface).to_vec()
}

/// A compute node of `kind` whose `inputs` each carry `config`: the former
/// per-node plasticity fixture, now on every input edge.
pub(crate) fn plastic_node(
    kind: ComputeNodeKind,
    inputs: Vec<GraphEdge>,
    config: Option<Learning>,
) -> ComputeNode {
    ComputeNode::new(kind, with_plasticity(inputs, config))
}

/// Set `config` on every edge of `surface` of `def`.
pub(crate) fn set_surface_plasticity(
    def: &mut CgpGraphBackendDef,
    surface: EdgeSurface,
    config: Option<Learning>,
) {
    for index in 0..def.surface_edges(surface).len() {
        def.edge_mut(EdgeOccurrence::new(surface, index))
            .set_learning(config);
    }
}

/// Store `learning` on the edge at `at`: a lone edge's learning, or on a
/// member an `Own` override (`Some`) or `Static` (`None`).
pub(crate) fn store_learning(
    def: &mut CgpGraphBackendDef,
    at: EdgeOccurrence,
    learning: Option<Learning>,
) {
    let member = def.edge_at(at).membership().is_some();
    let mut edge = def.edge_mut(at);
    if member {
        edge.set_policy(learning.map_or(
            crate::genome::MemberPolicy::Static,
            crate::genome::MemberPolicy::Own,
        ));
    } else {
        edge.set_learning(learning);
    }
}

/// `edge` with its weight replaced.
pub(crate) fn reweighted(edge: GraphEdge, weight: f32) -> GraphEdge {
    GraphEdge::new(edge.source(), weight)
}

/// The runtime state of a genome whose mesh nodes `0..defs.len()` are
/// `defs`, each with no input references: a fixture over the only
/// constructor, [`GraphRuntimeState::for_genome`]. A graph with members
/// needs its input references; build its genome and call `for_genome`.
pub(crate) fn runtime_for_defs(defs: &[&CgpGraphBackendDef]) -> GraphRuntimeState {
    let genome = CreatureGenome::new(
        NodeId(0),
        defs.iter()
            .enumerate()
            .map(|(id, def)| {
                NodeGenome::new(
                    NodeId(id as u32),
                    Vec::new(),
                    BackendDef::Graph((*def).clone()),
                    Vec::new(),
                )
            })
            .collect(),
        crate::creature::founder::default_founder_body(),
    );
    GraphRuntimeState::for_genome(&genome, &crate::genome::learning::Seeds::default())
}

/// [`runtime_for_defs`] for one graph at mesh node 0.
pub(crate) fn runtime_for(def: &CgpGraphBackendDef) -> GraphRuntimeState {
    runtime_for_defs(&[def])
}

/// Replace the edges of `def`'s sink of `kind` with `edges`.
pub(crate) fn set_sink_inputs(
    def: &mut CgpGraphBackendDef,
    kind: OutputSinkKind,
    edges: Vec<GraphEdge>,
) {
    *def = def.clone().with_sink_inputs(kind, edges);
}

/// Replace the edges of compute node `idx` with `edges`. The rebuilt def
/// carries no seeds.
pub(crate) fn set_compute_inputs(def: &mut CgpGraphBackendDef, idx: usize, edges: Vec<GraphEdge>) {
    edit_compute_nodes(def, |nodes| nodes[idx].inputs = edges);
}

/// Declare `reference` on `node` with one silent bundle per field set, each
/// on its set's first admissible plan ([`super::plans`]), under the default
/// sensor context (a fixture). Returns the declaration's index.
///
/// # Panics
///
/// If a set has no admissible plan or the node rejects the edit.
pub(crate) fn add_first_plan_sensor(node: &mut NodeGenome, reference: InputReference) -> u16 {
    let context = crate::config::test_sensor_context(1);
    let births = match &node.backend_def {
        BackendDef::Graph(_) => SensorDescriptor::of(&reference, &context)
            .indexed_field_sets()
            .map(|(index, set)| first_plan_birth(index, &set))
            .collect(),
        BackendDef::Vm(_) => Vec::new(),
    };
    node.add_sensor(reference, &births, &context)
        .expect("a first-plan sensor fixture")
        .0
}

/// One silent bundle reading `source` on its set's first admissible plan
/// (a fixture).
///
/// # Panics
///
/// If the node is no Graph node, the source does not resolve or its set
/// has no admissible plan.
pub(crate) fn add_first_plan_bundle(node: &mut NodeGenome, source: BundleSource) -> BundleId {
    let context = crate::config::test_sensor_context(1);
    let set = node
        .field_set(source, &context)
        .expect("a resolvable source");
    let BackendDef::Graph(_) = &node.backend_def else {
        panic!("add_first_plan_bundle: a Graph node");
    };
    let birth = first_plan_birth(source.field_set(), &set);
    node.add_bundle(source, birth, &context)
        .expect("a first-plan bundle fixture")
        .0
}

/// `birth` reading `source` on a bare `def`, through
/// [`NodeGenome::add_bundle`] on a node declaring `refs` that hosts `def`
/// for the edit, under the default sensor context (a fixture).
///
/// # Panics
///
/// If the node rejects the edit.
pub(crate) fn add_bundle_on(
    def: &mut CgpGraphBackendDef,
    refs: &[InputReference],
    source: BundleSource,
    birth: BundleBirth,
) -> BundleId {
    let hosted = std::mem::replace(def, CgpGraphBackendDef::new_with_fixed_outputs());
    let mut node = NodeGenome::new(
        NodeId::new(0),
        refs.to_vec(),
        BackendDef::Graph(hosted),
        Vec::new(),
    );
    let id = node
        .add_bundle(source, birth, &crate::config::test_sensor_context(1))
        .expect("a bundle fixture")
        .0;
    let BackendDef::Graph(hosted) = node.backend_def else {
        unreachable!("the node hosts a Graph def")
    };
    *def = hosted;
    id
}

fn first_plan_birth(field_set: u8, set: &FieldSet) -> BundleBirth {
    BundleBirth {
        field_set,
        wiring: *super::plans(set).first().expect("an admissible plan"),
        scale: 0.0,
    }
}

/// A def on the fixed catalog whose compute nodes deserialize from
/// `compute_nodes`, the JSON of a compute-node list: the load boundary a
/// genome file crosses.
///
/// # Panics
///
/// If the JSON does not deserialize.
pub(crate) fn def_with_compute_json(compute_nodes: &str) -> CgpGraphBackendDef {
    let mut value = serde_json::to_value(CgpGraphBackendDef::new_with_fixed_outputs())
        .expect("a def serializes");
    value["compute_nodes"] = serde_json::from_str(compute_nodes).expect("compute-node JSON");
    serde_json::from_value(value).expect("a def on the fixed catalog")
}
