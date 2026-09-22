//! CGP graph backend analysis: backward/forward slicing and functional complexity.
//!
//! Backward slicing anchors on every wired output sink, the action-vote and
//! parameter sinks included.
//! Slicing traverses `GraphSource::ComputeNode` references, stopping at
//! `InputLeaf` and `SharedMemory` (implicit sources, not nodes).

use std::collections::{HashSet, VecDeque};

use super::cgp::{CgpGraphBackendDef, GraphEdge, GraphSource};

// ── Backward slicing ────────────────────────────────────────────────────────

/// Collect all compute node indices referenced as `GraphSource::ComputeNode`
/// in the given edges (one level only, not recursive).
fn enqueue_compute_sources<'a>(
    edges: impl IntoIterator<Item = &'a GraphEdge>,
    compute_count: usize,
    visited: &mut [bool],
    queue: &mut VecDeque<usize>,
) {
    for edge in edges {
        if let GraphSource::ComputeNode(idx) = edge.source {
            let i = idx as usize;
            if i < compute_count && !visited[i] {
                visited[i] = true;
                queue.push_back(i);
            }
        }
    }
}

/// Backward-slice from all wired behavioral surfaces in the CGP graph.
///
/// Anchors: wired output sinks. A sink is "wired" if it has at least one edge.
/// Traverses `GraphSource::ComputeNode` references recursively.
/// Returns compute node indices that are backward-reachable from any wired surface.
#[must_use]
pub(crate) fn cgp_live_compute_indices(def: &CgpGraphBackendDef) -> Vec<usize> {
    let n = def.compute_nodes.len();
    if n == 0 {
        return Vec::new();
    }
    let mut visited = vec![false; n];
    let mut queue = VecDeque::with_capacity(n);

    // Anchor: wired output sinks
    for sink in &def.output_sinks {
        if !sink.inputs.is_empty() {
            enqueue_compute_sources(&sink.inputs, n, &mut visited, &mut queue);
        }
    }

    // BFS through compute node inputs
    while let Some(idx) = queue.pop_front() {
        enqueue_compute_sources(&def.compute_nodes[idx].inputs, n, &mut visited, &mut queue);
    }

    visited
        .iter()
        .enumerate()
        .filter(|(_, &v)| v)
        .map(|(i, _)| i)
        .collect()
}

/// Every edge on the graph's wired surface: the inputs of the given live
/// compute nodes, then all output sinks.
///
/// `live` is a `cgp_live_compute_indices` result. An unwired sink carries no
/// edge, so no wiredness test is needed here. Edges
/// are yielded once each, in traversal order; every caller reduces them into
/// an order-independent result, and each keeps its own filtering — the census
/// in `sensor_census` deliberately reads `SharedMemory` on the whole surface
/// while `derive_cgp_annotations` reads it on live compute nodes alone.
pub(crate) fn wired_surface_edges<'a>(
    def: &'a CgpGraphBackendDef,
    live: &'a [usize],
) -> impl Iterator<Item = &'a GraphEdge> {
    live.iter()
        .flat_map(|&idx| &def.compute_nodes[idx].inputs)
        .chain(def.output_sinks.iter().flat_map(|sink| &sink.inputs))
}

// ── Functional complexity ───────────────────────────────────────────────────

/// Collect consumed `InputLeaf` ref_idx values from the wired surface edges of
/// a graph with these live compute nodes.
fn collect_consumed_input_refs(def: &CgpGraphBackendDef, live: &[usize]) -> HashSet<u16> {
    wired_surface_edges(def, live)
        .filter_map(|edge| match edge.source {
            GraphSource::InputLeaf { ref_idx, .. } => Some(ref_idx),
            GraphSource::SharedMemory { .. } | GraphSource::ComputeNode(_) => None,
        })
        .collect()
}

/// Count the functional complexity of a CGP graph backend.
///
/// Scoring (per the three-tier liveness model):
/// - +1 per wired output sink (has at least one edge)
/// - +1 per live compute node (backward-reachable from wired surfaces)
/// - +edge_count per live compute node
/// - +1 per consumed InputLeaf ref_idx (across all live edges)
///
/// Dormant (unwired) sinks are NOT counted.
#[must_use]
pub(crate) fn cgp_functional_complexity(def: &CgpGraphBackendDef) -> u32 {
    let mut score: u32 = 0;

    // Count wired output sinks
    for sink in &def.output_sinks {
        if !sink.inputs.is_empty() {
            score += 1;
        }
    }

    // Live compute nodes (backward-reachable from wired surfaces)
    let live_indices = cgp_live_compute_indices(def);

    for &idx in &live_indices {
        score += 1; // the compute node
        score += def.compute_nodes[idx].inputs.len() as u32;
    }

    // Consumed input refs across all live edges
    let consumed = collect_consumed_input_refs(def, &live_indices);
    score += consumed.len() as u32;

    score
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::creature::genome::cgp::{ComputeNode, ComputeNodeKind, OutputSink, OutputSinkKind};
    use crate::creature::genome::vote::VoteSink;

    fn empty_def() -> CgpGraphBackendDef {
        CgpGraphBackendDef {
            birth_weights: None,
            compute_nodes: Vec::new(),
            output_sinks: Vec::new(),
        }
    }

    fn wired_def() -> CgpGraphBackendDef {
        // CN0: Add, inputs from CN1 + InputLeaf(0,0)
        // CN1: Constant(1.0), no inputs
        // Sink(CustomOutput(0)) wired to CN0
        // Sink(ActionVote(Eat)) wired to CN1
        // Sink(ActionVote(Terminate)) wired to CN0
        CgpGraphBackendDef {
            birth_weights: None,
            compute_nodes: vec![
                ComputeNode {
                    kind: ComputeNodeKind::Add,
                    inputs: vec![
                        GraphEdge {
                            source: GraphSource::ComputeNode(1),
                            weight: 1.0,
                        },
                        GraphEdge {
                            source: GraphSource::InputLeaf {
                                ref_idx: 0,
                                sub_idx: 0,
                            },
                            weight: 0.5,
                        },
                    ],
                    plasticity: None,
                },
                ComputeNode {
                    kind: ComputeNodeKind::Constant(1.0),
                    inputs: Vec::new(),
                    plasticity: None,
                },
                ComputeNode {
                    kind: ComputeNodeKind::Sigmoid,
                    inputs: Vec::new(), // isolated — not reachable
                    plasticity: None,
                },
            ],
            output_sinks: vec![
                OutputSink {
                    kind: OutputSinkKind::CustomOutput(0),
                    inputs: vec![GraphEdge {
                        source: GraphSource::ComputeNode(0),
                        weight: 1.0,
                    }],
                },
                OutputSink {
                    kind: OutputSinkKind::ActionVote(VoteSink::Eat),
                    inputs: vec![GraphEdge {
                        source: GraphSource::ComputeNode(1),
                        weight: 1.0,
                    }],
                },
                OutputSink {
                    kind: OutputSinkKind::ActionVote(VoteSink::Terminate),
                    inputs: vec![GraphEdge {
                        source: GraphSource::ComputeNode(0),
                        weight: 1.0,
                    }],
                },
            ],
        }
    }

    // ── Live compute indices ────────────────────────────────────────────────

    #[test]
    fn live_indices_empty_graph() {
        let def = empty_def();
        assert!(cgp_live_compute_indices(&def).is_empty());
    }

    #[test]
    fn live_indices_wired_graph() {
        let def = wired_def();
        let live = cgp_live_compute_indices(&def);
        // CN0 and CN1 are reachable, CN2 is isolated
        assert!(live.contains(&0));
        assert!(live.contains(&1));
        assert!(!live.contains(&2));
    }

    #[test]
    fn live_indices_unwired_sinks_dont_anchor() {
        let def = CgpGraphBackendDef::new_with_fixed_outputs();
        // All sinks are empty → no live compute nodes
        assert!(cgp_live_compute_indices(&def).is_empty());
    }

    // ── Functional complexity ───────────────────────────────────────────────

    #[test]
    fn complexity_empty_graph_is_zero() {
        let def = empty_def();
        assert_eq!(cgp_functional_complexity(&def), 0);
    }

    #[test]
    fn complexity_unwired_fixed_outputs_is_zero() {
        let def = CgpGraphBackendDef::new_with_fixed_outputs();
        assert_eq!(cgp_functional_complexity(&def), 0);
    }

    #[test]
    fn complexity_wired_graph() {
        let def = wired_def();
        let score = cgp_functional_complexity(&def);

        // Expected:
        // Wired sinks: 3 (CustomOutput(0), ActionVote(Eat), ActionVote(Terminate))
        // Live compute nodes: CN0 (2 edges) + CN1 (0 edges) = 2 nodes + 2 edges
        // Consumed refs: InputLeaf(ref_idx=0) = 1
        // Total: 3 + 2 + 2 + 1 = 8
        assert_eq!(score, 8);
    }
}
