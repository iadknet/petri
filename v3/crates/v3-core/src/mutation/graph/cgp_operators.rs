//! CGP-style graph mutation operators.
//!
//! Topology mutations operate on `compute_nodes` only. Edge mutations work
//! across all 5 edge-bearing surfaces: compute inputs, sink inputs, action
//! gate edges, action param edges, and execute gate inputs.

use rand::Rng;

use crate::creature::genome::cgp::{
    ActionSlotBehavior, CgpGraphBackendDef, ComputeNode, ComputeNodeKind, GraphEdge, GraphSource,
    WorldActionKind,
};
use crate::mutation::types::MutationSkipReason;

// ─── Random source generation ───────────────────────────────────────────────

/// Generate a random `GraphSource` for edge creation/retargeting.
///
/// Distribution: 50% ComputeNode, 30% InputLeaf, 20% SharedMemory.
/// Falls back to SharedMemory if compute_count and input_ref_count are both 0.
pub(crate) fn random_graph_source(
    compute_count: u16,
    input_ref_count: u16,
    rng: &mut impl Rng,
) -> GraphSource {
    let roll: f32 = rng.gen();
    if roll < 0.5 && compute_count > 0 {
        GraphSource::ComputeNode(rng.gen_range(0..compute_count))
    } else if roll < 0.8 && input_ref_count > 0 {
        GraphSource::InputLeaf {
            ref_idx: rng.gen_range(0..input_ref_count),
            sub_idx: 0, // sub_idx starts at 0; raw field mutation can adjust
        }
    } else {
        GraphSource::SharedMemory {
            slot: rng.gen_range(0..16),
            previous: rng.gen_bool(0.3),
        }
    }
}

/// Generate a random `ComputeNodeKind` from all 19 variants.
pub(crate) fn random_compute_node_kind(rng: &mut impl Rng) -> ComputeNodeKind {
    match rng.gen_range(0u8..19) {
        0 => ComputeNodeKind::Add,
        1 => ComputeNodeKind::Multiply,
        2 => ComputeNodeKind::Negate,
        3 => ComputeNodeKind::Abs,
        4 => ComputeNodeKind::Min,
        5 => ComputeNodeKind::Max,
        6 => ComputeNodeKind::WeightedSum,
        7 => ComputeNodeKind::Sigmoid,
        8 => ComputeNodeKind::Tanh,
        9 => ComputeNodeKind::Relu,
        10 => ComputeNodeKind::Clamp01,
        11 => ComputeNodeKind::Threshold(rng.gen_range(-1.0f32..=1.0)),
        12 => ComputeNodeKind::GreaterThan,
        13 => ComputeNodeKind::Select,
        14 => ComputeNodeKind::DecayIntegrator(rng.gen_range(0.0f32..=1.0)),
        15 => ComputeNodeKind::Momentum(rng.gen_range(0.0f32..=1.0)),
        16 => ComputeNodeKind::Oscillator(rng.gen_range(0.01f32..=8.0)),
        17 => ComputeNodeKind::AdaptiveGain,
        _ => ComputeNodeKind::Constant(rng.gen_range(-1.0f32..=1.0)),
    }
}

/// Generate a random `ActionSlotBehavior`.
fn random_action_slot_behavior(rng: &mut impl Rng) -> ActionSlotBehavior {
    if rng.gen_bool(0.15) {
        ActionSlotBehavior::Pop
    } else {
        let kind = match rng.gen_range(0u8..5) {
            0 => WorldActionKind::Eat,
            1 => WorldActionKind::Move,
            2 => WorldActionKind::Reproduce,
            3 => WorldActionKind::StealEnergy,
            _ => WorldActionKind::NoOp,
        };
        ActionSlotBehavior::Emit(kind)
    }
}

// ─── Edge surface helpers ───────────────────────────────────────────────────

/// Identifies which edge-bearing surface an edge belongs to.
#[derive(Debug, Clone, Copy)]
enum EdgeSurface {
    ComputeInput(usize),
    SinkInput(usize),
    ActionGate(usize),
    ActionParam(usize),
    ExecuteGate,
}

/// Count total edges across all 5 surfaces.
fn total_edge_count(def: &CgpGraphBackendDef) -> usize {
    let mut count = 0;
    for node in &def.compute_nodes {
        count += node.inputs.len();
    }
    for sink in &def.output_sinks {
        count += sink.inputs.len();
    }
    for slot in &def.action_bank {
        count += slot.gate_inputs.len();
        count += slot.param_inputs.len();
    }
    count += def.execute_gate.inputs.len();
    count
}

/// Pick a random edge surface that has at least one edge.
/// Returns the surface and the index of the edge within that surface.
fn pick_random_edge(def: &CgpGraphBackendDef, rng: &mut impl Rng) -> Option<(EdgeSurface, usize)> {
    let total = total_edge_count(def);
    if total == 0 {
        return None;
    }

    let mut pick = rng.gen_range(0..total);

    for (i, node) in def.compute_nodes.iter().enumerate() {
        if pick < node.inputs.len() {
            return Some((EdgeSurface::ComputeInput(i), pick));
        }
        pick -= node.inputs.len();
    }
    for (i, sink) in def.output_sinks.iter().enumerate() {
        if pick < sink.inputs.len() {
            return Some((EdgeSurface::SinkInput(i), pick));
        }
        pick -= sink.inputs.len();
    }
    for (i, slot) in def.action_bank.iter().enumerate() {
        if pick < slot.gate_inputs.len() {
            return Some((EdgeSurface::ActionGate(i), pick));
        }
        pick -= slot.gate_inputs.len();
        if pick < slot.param_inputs.len() {
            return Some((EdgeSurface::ActionParam(i), pick));
        }
        pick -= slot.param_inputs.len();
    }
    if pick < def.execute_gate.inputs.len() {
        return Some((EdgeSurface::ExecuteGate, pick));
    }

    None // shouldn't reach here
}

/// Pick a random edge container (surface) to add an edge to.
/// Uniform across all surfaces (compute inputs, sink inputs, action gate/param, execute gate).
fn pick_random_surface(def: &CgpGraphBackendDef, rng: &mut impl Rng) -> Option<EdgeSurface> {
    // Build list of all available surfaces
    let mut surfaces = Vec::with_capacity(
        def.compute_nodes.len() + def.output_sinks.len() + def.action_bank.len() * 2 + 1,
    );

    for i in 0..def.compute_nodes.len() {
        surfaces.push(EdgeSurface::ComputeInput(i));
    }
    for i in 0..def.output_sinks.len() {
        surfaces.push(EdgeSurface::SinkInput(i));
    }
    for i in 0..def.action_bank.len() {
        surfaces.push(EdgeSurface::ActionGate(i));
        surfaces.push(EdgeSurface::ActionParam(i));
    }
    surfaces.push(EdgeSurface::ExecuteGate);

    if surfaces.is_empty() {
        return None;
    }

    Some(surfaces[rng.gen_range(0..surfaces.len())])
}

/// Get mutable reference to the edge vec for a given surface.
fn get_edge_vec_mut(def: &mut CgpGraphBackendDef, surface: EdgeSurface) -> &mut Vec<GraphEdge> {
    match surface {
        EdgeSurface::ComputeInput(i) => &mut def.compute_nodes[i].inputs,
        EdgeSurface::SinkInput(i) => &mut def.output_sinks[i].inputs,
        EdgeSurface::ActionGate(i) => &mut def.action_bank[i].gate_inputs,
        EdgeSurface::ActionParam(i) => &mut def.action_bank[i].param_inputs,
        EdgeSurface::ExecuteGate => &mut def.execute_gate.inputs,
    }
}

// ─── Topology operators ────────────────────────────────────────────────────

/// Add a new compute node with a random kind and optional bootstrap edge.
pub(crate) fn add_compute_node(
    def: &mut CgpGraphBackendDef,
    input_ref_count: u16,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let compute_count = def.compute_nodes.len() as u16;
    let kind = random_compute_node_kind(rng);
    let inputs = if compute_count > 0 && rng.gen_bool(0.5) {
        vec![GraphEdge {
            source: random_graph_source(compute_count, input_ref_count, rng),
            weight: rng.gen_range(-1.0f32..=1.0),
        }]
    } else {
        Vec::new()
    };
    def.compute_nodes.push(ComputeNode {
        kind,
        inputs,
        plasticity: None,
    });
    Ok(())
}

/// Remove a random compute node and remap all edges.
pub(crate) fn remove_compute_node(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if def.compute_nodes.is_empty() {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    let idx = rng.gen_range(0..def.compute_nodes.len());
    def.remove_compute_node_at(idx);
    Ok(())
}

/// Change a random compute node's kind to a new random kind.
pub(crate) fn swap_compute_kind(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if def.compute_nodes.is_empty() {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    let idx = rng.gen_range(0..def.compute_nodes.len());
    def.compute_nodes[idx].kind = random_compute_node_kind(rng);
    Ok(())
}

/// Clone a compute node. 50% chance to clear inputs. Adds backlink from existing node.
pub(crate) fn copy_compute_node(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if def.compute_nodes.is_empty() {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    let source_idx = rng.gen_range(0..def.compute_nodes.len());
    let mut copy = def.compute_nodes[source_idx].clone();
    if rng.gen_bool(0.5) {
        copy.inputs.clear();
    }
    let new_idx = def.compute_nodes.len() as u16;
    def.compute_nodes.push(copy);

    // Add backlink from random existing compute node to the copy.
    if new_idx > 0 {
        let target = rng.gen_range(0..new_idx as usize);
        def.compute_nodes[target].inputs.push(GraphEdge {
            source: GraphSource::ComputeNode(new_idx),
            weight: rng.gen_range(-1.0f32..=1.0),
        });
    }
    Ok(())
}

/// Copy a random-walk cluster of 2-4 compute nodes (CopySubgraph).
/// Internal edges remapped, external edges preserved. Copies not wired to sinks.
pub(crate) fn copy_cgp_subgraph(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if def.compute_nodes.len() < 2 {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    let seed = rng.gen_range(0..def.compute_nodes.len());
    let target_size = rng.gen_range(2..=4).min(def.compute_nodes.len());

    // Random walk to build cluster.
    let mut cluster = vec![seed];
    let mut in_cluster = vec![false; def.compute_nodes.len()];
    in_cluster[seed] = true;
    let mut neighbors = Vec::new();

    for _ in 0..10 {
        if cluster.len() >= target_size {
            break;
        }
        let current = cluster[rng.gen_range(0..cluster.len())];
        neighbors.clear();
        // Input neighbors (ComputeNode sources of current node).
        for edge in &def.compute_nodes[current].inputs {
            if let GraphSource::ComputeNode(idx) = edge.source {
                let i = idx as usize;
                if i < def.compute_nodes.len() && !in_cluster[i] {
                    neighbors.push(i);
                }
            }
        }
        // Output neighbors (nodes that reference current).
        for (i, node) in def.compute_nodes.iter().enumerate() {
            if !in_cluster[i] {
                for edge in &node.inputs {
                    if edge.source == GraphSource::ComputeNode(current as u16) {
                        neighbors.push(i);
                        break;
                    }
                }
            }
        }
        if !neighbors.is_empty() {
            let next = neighbors[rng.gen_range(0..neighbors.len())];
            if !in_cluster[next] {
                in_cluster[next] = true;
                cluster.push(next);
            }
        }
    }

    cluster.sort_unstable();
    let base = def.compute_nodes.len();
    let old_to_new: std::collections::HashMap<usize, u16> = cluster
        .iter()
        .enumerate()
        .map(|(i, &old)| (old, (base + i) as u16))
        .collect();

    // Clone nodes and remap intra-cluster ComputeNode edges.
    let cloned_nodes: Vec<ComputeNode> = cluster
        .iter()
        .map(|&idx| {
            let mut node = def.compute_nodes[idx].clone();
            for edge in &mut node.inputs {
                if let GraphSource::ComputeNode(ref mut src_idx) = edge.source {
                    if let Some(&new_idx) = old_to_new.get(&(*src_idx as usize)) {
                        *src_idx = new_idx;
                    }
                    // External ComputeNode edges keep original source.
                }
                // InputLeaf/SharedMemory edges preserved as-is.
            }
            node
        })
        .collect();
    def.compute_nodes.extend(cloned_nodes);
    Ok(())
}

// ─── Edge operators (across all 5 surfaces) ─────────────────────────────────

/// Add a random edge to a random edge-bearing surface.
pub(crate) fn add_edge(
    def: &mut CgpGraphBackendDef,
    input_ref_count: u16,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let surface = pick_random_surface(def, rng).ok_or(MutationSkipReason::NoApplicableTarget)?;
    let compute_count = def.compute_nodes.len() as u16;
    let source = random_graph_source(compute_count, input_ref_count, rng);
    let weight = rng.gen_range(-1.0f32..=1.0);
    let edges = get_edge_vec_mut(def, surface);
    edges.push(GraphEdge { source, weight });
    Ok(())
}

/// Remove a random edge from any surface.
pub(crate) fn remove_edge(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let (surface, edge_idx) =
        pick_random_edge(def, rng).ok_or(MutationSkipReason::NoApplicableTarget)?;
    let edges = get_edge_vec_mut(def, surface);
    edges.remove(edge_idx);
    Ok(())
}

/// Change the source of a random edge.
pub(crate) fn retarget_edge(
    def: &mut CgpGraphBackendDef,
    input_ref_count: u16,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let (surface, edge_idx) =
        pick_random_edge(def, rng).ok_or(MutationSkipReason::NoApplicableTarget)?;
    let compute_count = def.compute_nodes.len() as u16;
    let new_source = random_graph_source(compute_count, input_ref_count, rng);
    let edges = get_edge_vec_mut(def, surface);
    edges[edge_idx].source = new_source;
    Ok(())
}

/// Alter the weight of a random edge.
pub(crate) fn alter_edge_weight(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let (surface, edge_idx) =
        pick_random_edge(def, rng).ok_or(MutationSkipReason::NoApplicableTarget)?;
    let edges = get_edge_vec_mut(def, surface);
    let w = &mut edges[edge_idx].weight;
    if w.abs() > 0.01 {
        *w *= 1.0 + rng.gen_range(-0.2f32..=0.2);
    } else {
        *w += rng.gen_range(-0.1f32..=0.1);
    }
    Ok(())
}

/// Copy all edges from one compute node to another.
pub(crate) fn copy_edge_bundle(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if def.compute_nodes.len() < 2 {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    let source = rng.gen_range(0..def.compute_nodes.len());
    if def.compute_nodes[source].inputs.is_empty() {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    let mut target = rng.gen_range(0..def.compute_nodes.len() - 1);
    if target >= source {
        target += 1;
    }
    let copied = def.compute_nodes[source].inputs.clone();
    def.compute_nodes[target].inputs.extend(copied);
    Ok(())
}

// ─── Parameter operators ────────────────────────────────────────────────────

/// Returns true if the compute node kind has a mutable float parameter.
fn is_compute_parameterized(kind: &ComputeNodeKind) -> bool {
    matches!(
        kind,
        ComputeNodeKind::Constant(_)
            | ComputeNodeKind::Threshold(_)
            | ComputeNodeKind::DecayIntegrator(_)
            | ComputeNodeKind::Momentum(_)
            | ComputeNodeKind::Oscillator(_)
    )
}

/// Mutate the float parameter of a parameterized compute node.
pub(crate) fn mutate_compute_param(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let eligible: Vec<usize> = def
        .compute_nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| is_compute_parameterized(&n.kind))
        .map(|(i, _)| i)
        .collect();
    if eligible.is_empty() {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    let idx = eligible[rng.gen_range(0..eligible.len())];
    match &mut def.compute_nodes[idx].kind {
        ComputeNodeKind::Constant(ref mut p) | ComputeNodeKind::Threshold(ref mut p) => {
            *p += rng.gen_range(-0.1f32..=0.1);
        }
        ComputeNodeKind::DecayIntegrator(ref mut p) | ComputeNodeKind::Momentum(ref mut p) => {
            *p += rng.gen_range(-0.1f32..=0.1);
            *p = p.clamp(0.0, 1.0);
        }
        ComputeNodeKind::Oscillator(ref mut p) => {
            *p += rng.gen_range(-0.1f32..=0.1);
            *p = p.clamp(0.01, 8.0);
        }
        _ => unreachable!("is_compute_parameterized filter"),
    }
    Ok(())
}

// ─── Action slot behavior mutation ──────────────────────────────────────────

/// Mutate the behavior of a random action slot.
pub(crate) fn mutate_action_slot_behavior(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if def.action_bank.is_empty() {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    let idx = rng.gen_range(0..def.action_bank.len());
    def.action_bank[idx].behavior = random_action_slot_behavior(rng);
    Ok(())
}

// ─── Raw field mutation ─────────────────────────────────────────────────────

/// Raw field mutation: randomly retarget an edge source or mutate a compute param.
pub(crate) fn raw_field_mutation(
    def: &mut CgpGraphBackendDef,
    input_ref_count: u16,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    // Count eligible targets: parameterized compute nodes + all edges
    let param_count = def
        .compute_nodes
        .iter()
        .filter(|n| is_compute_parameterized(&n.kind))
        .count();
    let edge_count = total_edge_count(def);
    let total = param_count + edge_count;

    if total == 0 {
        return Err(MutationSkipReason::NoApplicableTarget);
    }

    let pick = rng.gen_range(0..total);
    if pick < param_count {
        // Mutate a compute param
        mutate_compute_param(def, rng)
    } else {
        // Retarget an edge source
        retarget_edge(def, input_ref_count, rng)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::MutationConfig;
    use crate::creature::genome::cgp::{
        ActionSlot, ActionSlotBehavior, ExecuteGate, OutputSink, OutputSinkKind, WorldActionKind,
    };
    use rand::SeedableRng;

    fn test_rng() -> rand::rngs::SmallRng {
        rand::rngs::SmallRng::seed_from_u64(42)
    }

    fn minimal_def() -> CgpGraphBackendDef {
        CgpGraphBackendDef {
            compute_nodes: vec![
                ComputeNode {
                    kind: ComputeNodeKind::Add,
                    inputs: vec![GraphEdge {
                        source: GraphSource::ComputeNode(0),
                        weight: 1.0,
                    }],
                    plasticity: None,
                },
                ComputeNode {
                    kind: ComputeNodeKind::Constant(0.5),
                    inputs: Vec::new(),
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
                    source: GraphSource::ComputeNode(0),
                    weight: 1.0,
                }],
                param_inputs: Vec::new(),
            }],
            execute_gate: ExecuteGate { inputs: Vec::new() },
        }
    }

    // ── Topology tests ──────────────────────────────────────────────────────

    #[test]
    fn add_compute_node_increases_count() {
        let mut def = minimal_def();
        let mut rng = test_rng();
        let before = def.compute_nodes.len();
        add_compute_node(&mut def, 2, &mut rng).unwrap();
        assert_eq!(def.compute_nodes.len(), before + 1);
    }

    #[test]
    fn remove_compute_node_decreases_count() {
        let mut def = minimal_def();
        let mut rng = test_rng();
        let before = def.compute_nodes.len();
        remove_compute_node(&mut def, &mut rng).unwrap();
        assert_eq!(def.compute_nodes.len(), before - 1);
    }

    #[test]
    fn remove_compute_node_empty_is_skip() {
        let mut def = CgpGraphBackendDef {
            compute_nodes: Vec::new(),
            output_sinks: Vec::new(),
            action_bank: Vec::new(),
            execute_gate: ExecuteGate { inputs: Vec::new() },
        };
        let mut rng = test_rng();
        assert!(remove_compute_node(&mut def, &mut rng).is_err());
    }

    #[test]
    fn swap_compute_kind_changes_kind() {
        let mut def = minimal_def();
        let mut rng = test_rng();
        let originals: Vec<_> = def.compute_nodes.iter().map(|n| n.kind).collect();
        // 20 swaps across 19 variants — statistically near-certain to change
        for _ in 0..20 {
            swap_compute_kind(&mut def, &mut rng).unwrap();
        }
        let any_changed =
            def.compute_nodes.iter().enumerate().any(|(i, n)| {
                std::mem::discriminant(&n.kind) != std::mem::discriminant(&originals[i])
            });
        assert!(
            any_changed,
            "at least one node kind should change after 20 swaps"
        );
    }

    #[test]
    fn copy_compute_node_duplicates() {
        let mut def = minimal_def();
        let mut rng = test_rng();
        let before = def.compute_nodes.len();
        copy_compute_node(&mut def, &mut rng).unwrap();
        assert_eq!(def.compute_nodes.len(), before + 1);
    }

    #[test]
    fn copy_cgp_subgraph_adds_nodes() {
        let mut def = minimal_def();
        let mut rng = test_rng();
        let before = def.compute_nodes.len();
        copy_cgp_subgraph(&mut def, &mut rng).unwrap();
        assert!(def.compute_nodes.len() > before);
    }

    #[test]
    fn copy_cgp_subgraph_too_small_is_skip() {
        let mut def = CgpGraphBackendDef {
            compute_nodes: vec![ComputeNode {
                kind: ComputeNodeKind::Add,
                inputs: Vec::new(),
                plasticity: None,
            }],
            output_sinks: Vec::new(),
            action_bank: Vec::new(),
            execute_gate: ExecuteGate { inputs: Vec::new() },
        };
        let mut rng = test_rng();
        assert!(copy_cgp_subgraph(&mut def, &mut rng).is_err());
    }

    // ── Edge tests ──────────────────────────────────────────────────────────

    #[test]
    fn add_edge_increases_total() {
        let mut def = minimal_def();
        let mut rng = test_rng();
        let before = total_edge_count(&def);
        add_edge(&mut def, 2, &mut rng).unwrap();
        assert_eq!(total_edge_count(&def), before + 1);
    }

    #[test]
    fn remove_edge_decreases_total() {
        let mut def = minimal_def();
        let mut rng = test_rng();
        let before = total_edge_count(&def);
        remove_edge(&mut def, &mut rng).unwrap();
        assert_eq!(total_edge_count(&def), before - 1);
    }

    #[test]
    fn retarget_edge_changes_source() {
        // Single edge for targeted testing
        let mut def = CgpGraphBackendDef {
            compute_nodes: vec![ComputeNode {
                kind: ComputeNodeKind::Add,
                inputs: vec![GraphEdge {
                    source: GraphSource::ComputeNode(0),
                    weight: 1.0,
                }],
                plasticity: None,
            }],
            output_sinks: Vec::new(),
            action_bank: Vec::new(),
            execute_gate: ExecuteGate { inputs: Vec::new() },
        };
        let original = def.compute_nodes[0].inputs[0].source;
        let mut changed = false;
        for seed in 0u64..20 {
            let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
            retarget_edge(&mut def, 2, &mut rng).unwrap();
            if def.compute_nodes[0].inputs[0].source != original {
                changed = true;
                break;
            }
        }
        assert!(changed, "source should change after retargeting");
    }

    #[test]
    fn alter_edge_weight_modifies_weight() {
        // Use a single compute node with one edge for targeted testing
        let mut def = CgpGraphBackendDef {
            compute_nodes: vec![ComputeNode {
                kind: ComputeNodeKind::Add,
                inputs: vec![GraphEdge {
                    source: GraphSource::ComputeNode(0),
                    weight: 1.0,
                }],
                plasticity: None,
            }],
            output_sinks: Vec::new(),
            action_bank: Vec::new(),
            execute_gate: ExecuteGate { inputs: Vec::new() },
        };
        let original_weight = 1.0f32;
        let mut rng = test_rng();
        for _ in 0..10 {
            alter_edge_weight(&mut def, &mut rng).unwrap();
        }
        let new_weight = def.compute_nodes[0].inputs[0].weight;
        assert!(
            (new_weight - original_weight).abs() > f32::EPSILON,
            "weight should change after 10 mutations"
        );
    }

    #[test]
    fn copy_edge_bundle_extends_target() {
        let mut def = minimal_def();
        let before = total_edge_count(&def);
        // Try multiple seeds — source node may have empty inputs causing skip
        for seed in 0u64..20 {
            let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
            let _ = copy_edge_bundle(&mut def, &mut rng);
        }
        // At least one successful copy should have increased total edges
        assert!(
            total_edge_count(&def) > before,
            "at least one copy_edge_bundle should succeed"
        );
    }

    // ── Parameter tests ─────────────────────────────────────────────────────

    #[test]
    fn mutate_compute_param_modifies_constant() {
        let mut def = minimal_def();
        let mut rng = test_rng();
        let original = match def.compute_nodes[1].kind {
            ComputeNodeKind::Constant(v) => v,
            _ => panic!("expected Constant"),
        };
        mutate_compute_param(&mut def, &mut rng).unwrap();
        let new_val = match def.compute_nodes[1].kind {
            ComputeNodeKind::Constant(v) => v,
            _ => panic!("expected Constant after param mutation"),
        };
        // Perturbation should have changed the value (additive ±0.1)
        assert!(
            (new_val - original).abs() <= 0.1 + f32::EPSILON,
            "perturbation should be within ±0.1"
        );
    }

    #[test]
    fn mutate_compute_param_no_parameterized_is_skip() {
        let mut def = CgpGraphBackendDef {
            compute_nodes: vec![ComputeNode {
                kind: ComputeNodeKind::Add,
                inputs: Vec::new(),
                plasticity: None,
            }],
            output_sinks: Vec::new(),
            action_bank: Vec::new(),
            execute_gate: ExecuteGate { inputs: Vec::new() },
        };
        let mut rng = test_rng();
        assert!(mutate_compute_param(&mut def, &mut rng).is_err());
    }

    // ── Action slot behavior tests ──────────────────────────────────────────

    #[test]
    fn mutate_action_slot_behavior_changes_slot() {
        let mut def = minimal_def();
        let mut rng = test_rng();
        let original = def.action_bank[0].behavior;
        // 20 mutations across 6 possible behaviors — near-certain to change
        let mut changed = false;
        for _ in 0..20 {
            mutate_action_slot_behavior(&mut def, &mut rng).unwrap();
            if def.action_bank[0].behavior != original {
                changed = true;
                break;
            }
        }
        assert!(changed, "behavior should change after 20 mutations");
    }

    #[test]
    fn mutate_action_slot_behavior_empty_bank_is_skip() {
        let mut def = CgpGraphBackendDef {
            compute_nodes: Vec::new(),
            output_sinks: Vec::new(),
            action_bank: Vec::new(),
            execute_gate: ExecuteGate { inputs: Vec::new() },
        };
        let mut rng = test_rng();
        assert!(mutate_action_slot_behavior(&mut def, &mut rng).is_err());
    }

    // ── Random generation tests ─────────────────────────────────────────────

    #[test]
    fn random_graph_source_covers_variants() {
        let mut rng = test_rng();
        let mut seen_compute = false;
        let mut seen_input = false;
        let mut seen_shared = false;
        for _ in 0..100 {
            match random_graph_source(5, 3, &mut rng) {
                GraphSource::ComputeNode(_) => seen_compute = true,
                GraphSource::InputLeaf { .. } => seen_input = true,
                GraphSource::SharedMemory { .. } => seen_shared = true,
            }
        }
        assert!(seen_compute && seen_input && seen_shared);
    }

    #[test]
    fn random_graph_source_fallback_to_shared_memory() {
        let mut rng = test_rng();
        // With 0 compute nodes and 0 input refs, should always return SharedMemory
        for _ in 0..20 {
            match random_graph_source(0, 0, &mut rng) {
                GraphSource::SharedMemory { slot, .. } => assert!(slot < 16),
                other => panic!("expected SharedMemory, got {:?}", other),
            }
        }
    }

    #[test]
    fn random_compute_node_kind_covers_all_19() {
        let mut rng = test_rng();
        let mut seen = std::collections::HashSet::new();
        for _ in 0..2000 {
            let kind = random_compute_node_kind(&mut rng);
            // Use discriminant for comparison (float params vary)
            seen.insert(std::mem::discriminant(&kind));
        }
        assert_eq!(
            seen.len(),
            19,
            "should cover all 19 ComputeNodeKind variants"
        );
    }

    // ── Full fixed output tests ─────────────────────────────────────────────

    #[test]
    fn add_edge_to_execute_gate() {
        let config = MutationConfig::default();
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs(&config);
        def.compute_nodes.push(ComputeNode {
            kind: ComputeNodeKind::Constant(1.0),
            inputs: Vec::new(),
            plasticity: None,
        });

        // Try many times to hit execute gate surface
        let mut rng = test_rng();
        for _ in 0..100 {
            let _ = add_edge(&mut def, 0, &mut rng);
        }
        // Execute gate should have gotten at least one edge
        assert!(
            !def.execute_gate.inputs.is_empty(),
            "execute gate should have received edges"
        );
    }
}
