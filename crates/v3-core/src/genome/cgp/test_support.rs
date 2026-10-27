//! Fixture editing for tests (T25.F01): a def's lists are read-only outside
//! the container, so a fixture that reshapes a catalog or rewires nodes
//! freely edits plain drafts and rebuilds the def through the seedless
//! builder.

use super::{
    CgpGraphBackendDef, ComputeNode, ComputeNodeKind, EdgeOccurrence, EdgeSurface, GraphEdge,
    OutputSink, OutputSinkKind,
};
use crate::contracts::NodeId;
use crate::genome::{BackendDef, CreatureGenome, NodeGenome, PlasticityConfig};
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

/// Edit `def`'s sink catalog as drafts; the def keeps its compute nodes,
/// bundle records and seeds (a member stays a member: the caller keeps the
/// bundle invariants).
pub(crate) fn edit_sinks(def: &mut CgpGraphBackendDef, edit: impl FnOnce(&mut Vec<SinkDraft>)) {
    let mut drafts: Vec<SinkDraft> = def
        .output_sinks()
        .iter()
        .map(|sink| SinkDraft {
            kind: sink.kind,
            inputs: sink.inputs.to_vec(),
        })
        .collect();
    edit(&mut drafts);
    let sinks = drafts
        .into_iter()
        .map(|draft| OutputSink::new(draft.kind, draft.inputs))
        .collect();
    *def = def.clone().with_output_sinks(sinks);
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
pub(crate) fn with_plasticity(
    edges: Vec<GraphEdge>,
    config: Option<PlasticityConfig>,
) -> Vec<GraphEdge> {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs()
        .with_compute_nodes(vec![ComputeNode::new(ComputeNodeKind::Add, edges)]);
    let surface = EdgeSurface::ComputeInput(0);
    for index in 0..def.surface_edges(surface).len() {
        def.edge_mut(EdgeOccurrence::new(surface, index))
            .set_plasticity(config);
    }
    def.surface_edges(surface).to_vec()
}

/// A compute node of `kind` whose `inputs` each carry `config`: the former
/// per-node plasticity fixture, now on every input edge.
pub(crate) fn plastic_node(
    kind: ComputeNodeKind,
    inputs: Vec<GraphEdge>,
    config: Option<PlasticityConfig>,
) -> ComputeNode {
    ComputeNode::new(kind, with_plasticity(inputs, config))
}

/// Set `config` on every edge of `surface` of `def`.
pub(crate) fn set_surface_plasticity(
    def: &mut CgpGraphBackendDef,
    surface: EdgeSurface,
    config: Option<PlasticityConfig>,
) {
    for index in 0..def.surface_edges(surface).len() {
        def.edge_mut(EdgeOccurrence::new(surface, index))
            .set_plasticity(config);
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
    let genome = CreatureGenome {
        entry_node_id: NodeId(0),
        nodes: defs
            .iter()
            .enumerate()
            .map(|(id, def)| NodeGenome {
                node_id: NodeId(id as u32),
                input_refs: Vec::new(),
                backend_def: BackendDef::Graph((*def).clone()),
                targets: Vec::new(),
            })
            .collect(),
    };
    GraphRuntimeState::for_genome(&genome, Vec::new())
}

/// [`runtime_for_defs`] for one graph at mesh node 0.
pub(crate) fn runtime_for(def: &CgpGraphBackendDef) -> GraphRuntimeState {
    runtime_for_defs(&[def])
}

/// Append `sink` to `def`'s catalog, exactly as given.
pub(crate) fn push_sink(def: &mut CgpGraphBackendDef, sink: OutputSink) {
    let mut sinks = def.output_sinks().to_vec();
    sinks.push(sink);
    *def = def.clone().with_output_sinks(sinks);
}

/// Replace the edges of `def`'s first sink of `kind` with `edges`.
///
/// # Panics
///
/// If `def` carries no sink of `kind`.
pub(crate) fn set_sink_inputs(
    def: &mut CgpGraphBackendDef,
    kind: OutputSinkKind,
    edges: Vec<GraphEdge>,
) {
    edit_sinks(def, |sinks| {
        sinks
            .iter_mut()
            .find(|sink| sink.kind == kind)
            .expect("the catalog holds the sink")
            .inputs = edges;
    });
}

/// Replace the edges of compute node `idx` with `edges`. The rebuilt def
/// carries no seeds.
pub(crate) fn set_compute_inputs(def: &mut CgpGraphBackendDef, idx: usize, edges: Vec<GraphEdge>) {
    edit_compute_nodes(def, |nodes| nodes[idx].inputs = edges);
}
