//! CGP graph backend analysis: backward/forward slicing and functional complexity.
//!
//! Backward slicing anchors on all wired behavioral surfaces:
//! output_sinks + action_bank gate/param edges + execute_gate inputs.
//! Slicing traverses `GraphSource::ComputeNode` references, stopping at
//! `InputLeaf` and `SharedMemory` (implicit sources, not nodes).

use std::collections::{HashSet, VecDeque};

use rand::Rng;

use super::analysis::DetectedGene;
use super::cgp::{CgpGraphBackendDef, GraphEdge, GraphSource};

// ── Backward slicing ────────────────────────────────────────────────────────

/// Collect all compute node indices referenced as `GraphSource::ComputeNode`
/// in the given edges (one level only, not recursive).
fn enqueue_compute_sources(
    edges: &[GraphEdge],
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
/// Anchors: wired output_sinks, wired action_bank (gate + param), wired execute_gate.
/// An item is "wired" if it has at least one edge.
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

    // Anchor: wired action bank (gate + param edges)
    for slot in &def.action_bank {
        if !slot.gate_inputs.is_empty() {
            enqueue_compute_sources(&slot.gate_inputs, n, &mut visited, &mut queue);
        }
        if !slot.param_inputs.is_empty() {
            enqueue_compute_sources(&slot.param_inputs, n, &mut visited, &mut queue);
        }
    }

    // Anchor: wired execute gate
    if !def.execute_gate.inputs.is_empty() {
        enqueue_compute_sources(&def.execute_gate.inputs, n, &mut visited, &mut queue);
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

/// Backward-slice from a specific compute node (used as anchor).
///
/// BFS backward through `GraphSource::ComputeNode` edges. Returns sorted
/// indices capped at `max_size`.
#[must_use]
#[allow(dead_code)] // wired into mutation gene detection in a future phase
pub(crate) fn cgp_backward_slice(
    def: &CgpGraphBackendDef,
    anchor_idx: usize,
    max_size: usize,
) -> Option<DetectedGene> {
    let n = def.compute_nodes.len();
    if anchor_idx >= n {
        return None;
    }

    let mut visited = vec![false; n];
    visited[anchor_idx] = true;
    let mut queue = VecDeque::new();
    let mut indices = Vec::with_capacity(max_size.min(n));
    indices.push(anchor_idx);

    if indices.len() < max_size {
        for edge in &def.compute_nodes[anchor_idx].inputs {
            if let GraphSource::ComputeNode(idx) = edge.source {
                let i = idx as usize;
                if i < n && !visited[i] {
                    visited[i] = true;
                    queue.push_back(i);
                }
            }
        }
    }

    while let Some(idx) = queue.pop_front() {
        indices.push(idx);
        if indices.len() >= max_size {
            break;
        }
        for edge in &def.compute_nodes[idx].inputs {
            if let GraphSource::ComputeNode(src) = edge.source {
                let i = src as usize;
                if i < n && !visited[i] {
                    visited[i] = true;
                    queue.push_back(i);
                }
            }
        }
    }

    indices.sort_unstable();
    Some(DetectedGene { indices })
}

/// Backward-slice from a randomly chosen compute node.
#[must_use]
#[allow(dead_code)] // wired into mutation gene detection in a future phase
pub(crate) fn cgp_backward_slice_random(
    def: &CgpGraphBackendDef,
    rng: &mut impl Rng,
    max_size: usize,
) -> Option<DetectedGene> {
    if def.compute_nodes.is_empty() {
        return None;
    }
    let anchor = rng.gen_range(0..def.compute_nodes.len());
    cgp_backward_slice(def, anchor, max_size)
}

// ── Forward slicing ─────────────────────────────────────────────────────────

/// Forward-slice from a seed compute node.
///
/// Fixpoint expansion: starts from seed, iteratively includes any compute node
/// whose inputs reference an already-included node via `GraphSource::ComputeNode`.
/// Returns sorted indices capped at `max_size`.
#[must_use]
#[allow(dead_code)] // wired into mutation gene detection in a future phase
pub(crate) fn cgp_forward_slice(
    def: &CgpGraphBackendDef,
    seed_idx: usize,
    max_size: usize,
) -> Option<DetectedGene> {
    let n = def.compute_nodes.len();
    if seed_idx >= n {
        return None;
    }

    let mut included = vec![false; n];
    included[seed_idx] = true;
    let mut count = 1usize;

    loop {
        let mut changed = false;
        for i in 0..n {
            if included[i] || count >= max_size {
                continue;
            }
            let refs_included = def.compute_nodes[i].inputs.iter().any(|edge| {
                if let GraphSource::ComputeNode(idx) = edge.source {
                    (idx as usize) < n && included[idx as usize]
                } else {
                    false
                }
            });
            if refs_included {
                included[i] = true;
                count += 1;
                changed = true;
                if count >= max_size {
                    break;
                }
            }
        }
        if !changed {
            break;
        }
    }

    let indices: Vec<usize> = included
        .iter()
        .enumerate()
        .filter(|(_, &inc)| inc)
        .map(|(i, _)| i)
        .collect();
    Some(DetectedGene { indices })
}

/// Forward-slice from a randomly chosen compute node.
#[must_use]
#[allow(dead_code)] // wired into mutation gene detection in a future phase
pub(crate) fn cgp_forward_slice_random(
    def: &CgpGraphBackendDef,
    rng: &mut impl Rng,
    max_size: usize,
) -> Option<DetectedGene> {
    if def.compute_nodes.is_empty() {
        return None;
    }
    let seed = rng.gen_range(0..def.compute_nodes.len());
    cgp_forward_slice(def, seed, max_size)
}

// ── Functional complexity ───────────────────────────────────────────────────

/// Collect consumed `InputLeaf` ref_idx values from edges in a set of live
/// compute nodes, plus all wired surface edges.
fn collect_consumed_input_refs(def: &CgpGraphBackendDef, live: &HashSet<usize>) -> HashSet<u16> {
    let mut refs = HashSet::new();

    // From live compute nodes
    for &idx in live {
        for edge in &def.compute_nodes[idx].inputs {
            if let GraphSource::InputLeaf { ref_idx, .. } = edge.source {
                refs.insert(ref_idx);
            }
        }
    }

    // From wired output sinks
    for sink in &def.output_sinks {
        for edge in &sink.inputs {
            if let GraphSource::InputLeaf { ref_idx, .. } = edge.source {
                refs.insert(ref_idx);
            }
        }
    }

    // From wired action bank
    for slot in &def.action_bank {
        for edge in slot.gate_inputs.iter().chain(slot.param_inputs.iter()) {
            if let GraphSource::InputLeaf { ref_idx, .. } = edge.source {
                refs.insert(ref_idx);
            }
        }
    }

    // From execute gate
    for edge in &def.execute_gate.inputs {
        if let GraphSource::InputLeaf { ref_idx, .. } = edge.source {
            refs.insert(ref_idx);
        }
    }

    refs
}

/// Count the functional complexity of a CGP graph backend.
///
/// Scoring (per the three-tier liveness model):
/// - +1 per wired output sink (has at least one edge)
/// - +1 per wired action slot (gate or param has at least one edge)
/// - +1 if execute gate is wired
/// - +1 per live compute node (backward-reachable from wired surfaces)
/// - +edge_count per live compute node
/// - +1 per consumed InputLeaf ref_idx (across all live edges)
///
/// Dormant (unwired) sinks, slots, and execute gate are NOT counted.
#[must_use]
pub(crate) fn cgp_functional_complexity(def: &CgpGraphBackendDef) -> u32 {
    let mut score: u32 = 0;

    // Count wired output sinks
    for sink in &def.output_sinks {
        if !sink.inputs.is_empty() {
            score += 1;
        }
    }

    // Count wired action slots
    for slot in &def.action_bank {
        if !slot.gate_inputs.is_empty() || !slot.param_inputs.is_empty() {
            score += 1;
        }
    }

    // Count wired execute gate
    if !def.execute_gate.inputs.is_empty() {
        score += 1;
    }

    // Live compute nodes (backward-reachable from wired surfaces)
    let live_indices = cgp_live_compute_indices(def);
    let live_set: HashSet<usize> = live_indices.iter().copied().collect();

    for &idx in &live_indices {
        score += 1; // the compute node
        score += def.compute_nodes[idx].inputs.len() as u32;
    }

    // Consumed input refs across all live edges
    let consumed = collect_consumed_input_refs(def, &live_set);
    score += consumed.len() as u32;

    score
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::MutationConfig;
    use crate::creature::genome::cgp::{
        ActionSlot, ActionSlotBehavior, ComputeNode, ComputeNodeKind, ExecuteGate, OutputSink,
        OutputSinkKind, WorldActionKind,
    };

    fn empty_def() -> CgpGraphBackendDef {
        CgpGraphBackendDef {
            compute_nodes: Vec::new(),
            output_sinks: Vec::new(),
            action_bank: Vec::new(),
            execute_gate: ExecuteGate { inputs: Vec::new() },
        }
    }

    fn wired_def() -> CgpGraphBackendDef {
        // CN0: Add, inputs from CN1 + InputLeaf(0,0)
        // CN1: Constant(1.0), no inputs
        // Sink(CustomOutput(0)) wired to CN0
        // ActionSlot gate wired to CN1
        // ExecuteGate wired to CN0
        CgpGraphBackendDef {
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
            output_sinks: vec![OutputSink {
                kind: OutputSinkKind::CustomOutput(0),
                inputs: vec![GraphEdge {
                    source: GraphSource::ComputeNode(0),
                    weight: 1.0,
                }],
            }],
            action_bank: vec![ActionSlot {
                behavior: ActionSlotBehavior::Emit(WorldActionKind::Eat),
                gate_inputs: vec![GraphEdge {
                    source: GraphSource::ComputeNode(1),
                    weight: 1.0,
                }],
                param_inputs: Vec::new(),
            }],
            execute_gate: ExecuteGate {
                inputs: vec![GraphEdge {
                    source: GraphSource::ComputeNode(0),
                    weight: 1.0,
                }],
            },
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
        let config = MutationConfig::default();
        let def = CgpGraphBackendDef::new_with_fixed_outputs(&config);
        // All sinks are empty → no live compute nodes
        assert!(cgp_live_compute_indices(&def).is_empty());
    }

    // ── Backward slicing ────────────────────────────────────────────────────

    #[test]
    fn backward_slice_includes_transitive_deps() {
        let def = wired_def();
        let gene = cgp_backward_slice(&def, 0, 10).unwrap();
        // CN0 depends on CN1 — both should be in slice
        assert!(gene.indices.contains(&0));
        assert!(gene.indices.contains(&1));
        assert!(!gene.indices.contains(&2)); // isolated
    }

    #[test]
    fn backward_slice_respects_max_size() {
        let def = wired_def();
        let gene = cgp_backward_slice(&def, 0, 1).unwrap();
        assert_eq!(gene.indices.len(), 1);
    }

    #[test]
    fn backward_slice_out_of_range_returns_none() {
        let def = wired_def();
        assert!(cgp_backward_slice(&def, 99, 10).is_none());
    }

    // ── Forward slicing ─────────────────────────────────────────────────────

    #[test]
    fn forward_slice_from_leaf() {
        let def = wired_def();
        // CN1 is used by CN0's input → forward slice from CN1 should include CN0
        let gene = cgp_forward_slice(&def, 1, 10).unwrap();
        assert!(gene.indices.contains(&0));
        assert!(gene.indices.contains(&1));
    }

    #[test]
    fn forward_slice_isolated_node() {
        let def = wired_def();
        // CN2 is isolated — forward slice from it should only contain itself
        let gene = cgp_forward_slice(&def, 2, 10).unwrap();
        assert_eq!(gene.indices, vec![2]);
    }

    // ── Functional complexity ───────────────────────────────────────────────

    #[test]
    fn complexity_empty_graph_is_zero() {
        let def = empty_def();
        assert_eq!(cgp_functional_complexity(&def), 0);
    }

    #[test]
    fn complexity_unwired_fixed_outputs_is_zero() {
        let config = MutationConfig::default();
        let def = CgpGraphBackendDef::new_with_fixed_outputs(&config);
        assert_eq!(cgp_functional_complexity(&def), 0);
    }

    #[test]
    fn complexity_wired_graph() {
        let def = wired_def();
        let score = cgp_functional_complexity(&def);

        // Expected:
        // Wired sinks: 1 (CustomOutput(0))
        // Wired action slots: 1 (slot 0 gate wired)
        // Wired execute gate: 1
        // Live compute nodes: CN0 (2 edges) + CN1 (0 edges) = 2 nodes + 2 edges
        // Consumed refs: InputLeaf(ref_idx=0) = 1
        // Total: 1 + 1 + 1 + 2 + 2 + 1 = 8
        assert_eq!(score, 8);
    }
}
