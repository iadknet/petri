//! Fixture editing for tests (T25.F01): a def's lists are read-only outside
//! the container, so a fixture that reshapes a catalog or rewires nodes
//! freely edits plain drafts and rebuilds the def through the seedless
//! builder.

use super::{
    CgpGraphBackendDef, ComputeNode, ComputeNodeKind, GraphEdge, OutputSink, OutputSinkKind,
};
use crate::creature::genome::PlasticityConfig;

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
    pub plasticity: Option<PlasticityConfig>,
}

/// Edit `def`'s sink catalog as drafts; the def keeps its compute nodes,
/// projections and seeds.
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
            plasticity: node.plasticity,
        })
        .collect();
    edit(&mut drafts);
    let nodes = drafts
        .into_iter()
        .map(|draft| ComputeNode::new(draft.kind, draft.inputs, draft.plasticity))
        .collect();
    *def = def.clone().with_compute_nodes(nodes);
}

/// `edge` with its weight replaced.
pub(crate) fn reweighted(edge: GraphEdge, weight: f32) -> GraphEdge {
    GraphEdge::new(edge.source(), weight)
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
