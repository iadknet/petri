//! CGP-style graph mutation operators.
//!
//! Topology mutations operate on `compute_nodes` only. Edge mutations work
//! across both edge-bearing surfaces: compute inputs and sink inputs, the
//! action-vote and parameter sinks included (T19.F04).

use rand::Rng;

use crate::contracts::{DynamicIntrospectionKey, InputReference};
use crate::creature::genome::cgp::{
    CgpGraphBackendDef, ComputeNode, ComputeNodeKind, GraphEdge, GraphSource,
};
use crate::creature::genome::{BackendDef, CreatureGenome};
use crate::mutation::compound::sub_value_count;
use crate::mutation::types::MutationSkipReason;

#[inline]
fn graph_def_mut(
    genome: &mut CreatureGenome,
    node_idx: usize,
) -> Result<&mut CgpGraphBackendDef, MutationSkipReason> {
    let node = genome
        .nodes
        .get_mut(node_idx)
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
    match &mut node.backend_def {
        BackendDef::Graph(def) => Ok(def),
        _ => Err(MutationSkipReason::NoApplicableTarget),
    }
}

/// The target node's graph backend def alongside a clone of its
/// `input_refs`, for operators (`random_graph_source` callers) that need
/// both: the clone is taken before the mutable borrow of `backend_def` so
/// both are available together despite living on the same `NodeGenome`.
#[inline]
fn graph_def_mut_with_input_refs(
    genome: &mut CreatureGenome,
    node_idx: usize,
) -> Result<(&mut CgpGraphBackendDef, Vec<InputReference>), MutationSkipReason> {
    let input_refs = genome
        .nodes
        .get(node_idx)
        .map(|n| n.input_refs.clone())
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
    let def = graph_def_mut(genome, node_idx)?;
    Ok((def, input_refs))
}

pub(super) fn alter_edge_weight(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let def = graph_def_mut(genome, node_idx)?;
    alter_edge_weight_in_def(def, rng)
}

pub(super) fn swap_operator(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let def = graph_def_mut(genome, node_idx)?;
    swap_compute_kind(def, rng)
}

pub(super) fn mutate_operator_param(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let def = graph_def_mut(genome, node_idx)?;
    mutate_compute_param(def, rng)
}

pub(super) fn add_internal_node(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let (def, input_refs) = graph_def_mut_with_input_refs(genome, node_idx)?;
    add_compute_node(def, &input_refs, rng)
}

pub(super) fn add_graph_edge(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let (def, input_refs) = graph_def_mut_with_input_refs(genome, node_idx)?;
    add_edge(def, &input_refs, rng)
}

pub(super) fn remove_internal_node(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let def = graph_def_mut(genome, node_idx)?;
    remove_compute_node(def, rng)
}

pub(super) fn retarget_graph_edge(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let (def, input_refs) = graph_def_mut_with_input_refs(genome, node_idx)?;
    retarget_edge(def, &input_refs, rng)
}

pub(super) fn remove_graph_edge(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let def = graph_def_mut(genome, node_idx)?;
    remove_edge(def, rng)
}

pub(super) fn apply_graph_raw_field_mutation(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let (def, input_refs) = graph_def_mut_with_input_refs(genome, node_idx)?;
    raw_field_mutation(def, &input_refs, rng)
}

pub(super) fn apply_copy_internal_node(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let def = graph_def_mut(genome, node_idx)?;
    copy_compute_node(def, rng)
}

pub(super) fn apply_copy_subgraph(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let def = graph_def_mut(genome, node_idx)?;
    copy_cgp_subgraph(def, rng)
}

pub(super) fn apply_copy_edge_bundle(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let def = graph_def_mut(genome, node_idx)?;
    copy_edge_bundle(def, rng)
}

// ─── Random source generation ───────────────────────────────────────────────

/// Generate a random `GraphSource` for edge creation/retargeting.
///
/// Distribution: 50% ComputeNode, 30% InputLeaf, 20% SharedMemory.
/// Falls back to SharedMemory if compute_count and input_refs are both empty.
/// `sub_idx` is drawn uniformly across the sampled reference's full width
/// (`mutation::compound::sub_value_count`), so a new edge can reach every
/// sub-value of a compound input, not just index 0.
pub(crate) fn random_graph_source(
    compute_count: u16,
    input_refs: &[InputReference],
    rng: &mut impl Rng,
) -> GraphSource {
    let roll: f32 = rng.gen();
    if roll < 0.5 && compute_count > 0 {
        GraphSource::ComputeNode(rng.gen_range(0..compute_count))
    } else if roll < 0.8 && !input_refs.is_empty() {
        let ref_idx = rng.gen_range(0..input_refs.len() as u16);
        let width = sub_value_count(&input_refs[ref_idx as usize]).max(1);
        GraphSource::InputLeaf {
            ref_idx,
            sub_idx: rng.gen_range(0..width),
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

// ─── Edge surface helpers ───────────────────────────────────────────────────

/// Identifies which edge-bearing surface an edge belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EdgeSurface {
    ComputeInput(usize),
    SinkInput(usize),
}

/// Every edge site on the def, defining the canonical surface order:
/// compute inputs, then sink inputs. The single enumeration [`total_edge_count`],
/// [`pick_random_edge`], the edge-site applicability predicates and the
/// operators that draw from filtered edge sets all read.
fn edge_sites(def: &CgpGraphBackendDef) -> impl Iterator<Item = (EdgeSurface, usize)> + '_ {
    let compute = def.compute_nodes.iter().enumerate().flat_map(|(i, node)| {
        (0..node.inputs.len()).map(move |edge| (EdgeSurface::ComputeInput(i), edge))
    });
    let sinks = def.output_sinks.iter().enumerate().flat_map(|(i, sink)| {
        (0..sink.inputs.len()).map(move |edge| (EdgeSurface::SinkInput(i), edge))
    });
    compute.chain(sinks)
}

/// The edge at `idx` on `surface`; read-only twin of [`edge_at_mut`].
fn edge_at(def: &CgpGraphBackendDef, surface: EdgeSurface, idx: usize) -> &GraphEdge {
    match surface {
        EdgeSurface::ComputeInput(i) => &def.compute_nodes[i].inputs[idx],
        EdgeSurface::SinkInput(i) => &def.output_sinks[i].inputs[idx],
    }
}

/// Count total edges across both surfaces: the length of [`edge_sites`].
fn total_edge_count(def: &CgpGraphBackendDef) -> usize {
    edge_sites(def).count()
}

/// Pick a random edge uniformly among [`edge_sites`].
/// Returns the surface and the index of the edge within that surface.
fn pick_random_edge(def: &CgpGraphBackendDef, rng: &mut impl Rng) -> Option<(EdgeSurface, usize)> {
    let total = total_edge_count(def);
    if total == 0 {
        return None;
    }
    edge_sites(def).nth(rng.gen_range(0..total))
}

/// Pick a random edge container (surface) to add an edge to, uniform over
/// every compute node and every catalog sink, the action-vote and parameter
/// sinks included (T19.F04).
pub(crate) fn pick_random_surface(
    def: &CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Option<EdgeSurface> {
    let compute = def.compute_nodes.len();
    let total = compute + def.output_sinks.len();
    if total == 0 {
        return None;
    }
    let pick = rng.gen_range(0..total);
    Some(if pick < compute {
        EdgeSurface::ComputeInput(pick)
    } else {
        EdgeSurface::SinkInput(pick - compute)
    })
}

/// Mutable access to the edge at `idx` on `surface`.
pub(crate) fn edge_at_mut(
    def: &mut CgpGraphBackendDef,
    surface: EdgeSurface,
    idx: usize,
) -> &mut GraphEdge {
    match surface {
        EdgeSurface::ComputeInput(i) => &mut def.compute_nodes[i].inputs[idx],
        EdgeSurface::SinkInput(i) => &mut def.output_sinks[i].inputs[idx],
    }
}

/// Append `edge` to `surface`.
fn push_edge(def: &mut CgpGraphBackendDef, surface: EdgeSurface, edge: GraphEdge) {
    match surface {
        EdgeSurface::ComputeInput(i) => def.compute_nodes[i].inputs.push(edge),
        EdgeSurface::SinkInput(i) => def.output_sinks[i].inputs.push(edge),
    }
}

/// Remove the edge at `idx` from `surface`.
fn remove_edge_at(def: &mut CgpGraphBackendDef, surface: EdgeSurface, idx: usize) {
    match surface {
        EdgeSurface::ComputeInput(i) => {
            def.compute_nodes[i].inputs.remove(idx);
        }
        EdgeSurface::SinkInput(i) => {
            def.output_sinks[i].inputs.remove(idx);
        }
    }
}

// ─── Topology operators ────────────────────────────────────────────────────

/// Add a new compute node by one of three function-preserving forms, drawn
/// uniformly: disconnected (a random kind with no inputs), bootstrap (a
/// random kind with one input edge, read by no surface), or split (a
/// NEAT-style insertion into an existing edge that reproduces the edge's
/// prior behavior exactly). None of the three forms sprays edges into the
/// output surfaces; only the split form retargets the one edge it splits.
///
/// The form is drawn among the forms this def admits: disconnected and
/// bootstrap append to any def (so a blank module keeps its growth reach),
/// while split is offered only when the def has a splittable edge and room
/// for the identity node.
pub(crate) fn add_compute_node(
    def: &mut CgpGraphBackendDef,
    input_refs: &[InputReference],
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let can_split = check_compute_node_capacity(def, 1).is_ok()
        && splittable_edge_sites(def, input_refs).next().is_some();
    let forms = if can_split { 3u8 } else { 2u8 };
    match rng.gen_range(0u8..forms) {
        0 => add_disconnected_node(def, rng),
        1 => add_bootstrap_node(def, input_refs, rng),
        _ => split_existing_edge(def, input_refs, rng),
    }
}

/// Append without repairing indices: existing dangling sources retain the
/// mutation operator's original behavior when their index becomes addressable.
fn append_compute_node(def: &mut CgpGraphBackendDef, node: ComputeNode) {
    if let Some(weights) = &mut def.birth_weights {
        weights.push(vec![None; node.inputs.len()]);
    }
    def.compute_nodes.push(node);
}

/// Form (a): a random kind joins the graph with no inputs and no consumer.
pub(crate) fn add_disconnected_node(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    append_compute_node(
        def,
        ComputeNode {
            kind: random_compute_node_kind(rng),
            inputs: Vec::new(),
            plasticity: None,
        },
    );
    Ok(())
}

/// Form (b): a random kind joins the graph reading one bootstrap edge; no
/// surface reads the new node back, so it has no effect until a later
/// connection operator wires it in.
pub(crate) fn add_bootstrap_node(
    def: &mut CgpGraphBackendDef,
    input_refs: &[InputReference],
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let compute_count = def.compute_nodes.len() as u16;
    let source = random_graph_source(compute_count, input_refs, rng);
    append_compute_node(
        def,
        ComputeNode {
            kind: random_compute_node_kind(rng),
            inputs: vec![GraphEdge {
                source,
                weight: rng.gen_range(-1.0f32..=1.0),
            }],
            plasticity: None,
        },
    );
    Ok(())
}

/// Form (c): pick one existing edge on any surface and insert an identity
/// `Add` node between its source and consumer. The new node's single input
/// (weight 1.0) reproduces the old source exactly in f32; the old edge is
/// retargeted to read the new node, keeping its old weight. When the
/// consumer is a compute node, the new node is inserted at the consumer's
/// index and every `ComputeNode(i >= consumer)` reference is remapped to
/// `i + 1` (`CgpGraphBackendDef::insert_compute_node_at`), so Gauss-Seidel
/// pass order is preserved. When the consumer is a sink, the new node is
/// appended instead. The one edge shape a
/// split cannot preserve is skipped rather than split: see
/// [`is_excluded_introspection_split`].
pub(crate) fn split_existing_edge(
    def: &mut CgpGraphBackendDef,
    input_refs: &[InputReference],
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    check_compute_node_capacity(def, 1)?;
    let sites: Vec<(EdgeSurface, usize)> = splittable_edge_sites(def, input_refs).collect();
    if sites.is_empty() {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    let (surface, edge_idx) = sites[rng.gen_range(0..sites.len())];
    let old_source = edge_at(def, surface, edge_idx).source;

    if let EdgeSurface::ComputeInput(consumer_idx) = surface {
        // Insert an empty placeholder first so the global index remap (which
        // also shifts `old_source` in place if it targets a ComputeNode at or
        // above `consumer_idx`, including a self-loop) runs before the new
        // node's own input is read back.
        def.insert_compute_node_at(consumer_idx, identity_add_node(Vec::new()));
        let new_idx = consumer_idx as u16;
        let shifted_consumer_idx = consumer_idx + 1; // shifted by the insert above
        let remapped_source = def.compute_nodes[shifted_consumer_idx].inputs[edge_idx].source;
        if let Some(weights) = &mut def.birth_weights {
            weights[consumer_idx].push(None);
        }
        def.compute_nodes[consumer_idx].inputs.push(GraphEdge {
            source: remapped_source,
            weight: 1.0,
        });
        def.compute_nodes[shifted_consumer_idx].inputs[edge_idx].source =
            GraphSource::ComputeNode(new_idx);
    } else {
        let new_idx = def.compute_nodes.len() as u16;
        append_compute_node(
            def,
            identity_add_node(vec![GraphEdge {
                source: old_source,
                weight: 1.0,
            }]),
        );
        edge_at_mut(def, surface, edge_idx).source = GraphSource::ComputeNode(new_idx);
    }
    Ok(())
}

/// The edges `split_existing_edge` can split: every edge except one whose
/// `ComputeNode` source dangles (a prior removal's sentinel) and one whose
/// shape a split cannot preserve
/// ([`is_excluded_introspection_split`], which only applies off the compute
/// surfaces).
fn splittable_edge_sites<'a>(
    def: &'a CgpGraphBackendDef,
    input_refs: &'a [InputReference],
) -> impl Iterator<Item = (EdgeSurface, usize)> + 'a {
    edge_sites(def).filter(move |&(surface, edge_idx)| {
        let source = edge_at(def, surface, edge_idx).source;
        if let GraphSource::ComputeNode(idx) = source {
            if idx as usize >= def.compute_nodes.len() {
                return false;
            }
        }
        matches!(surface, EdgeSurface::ComputeInput(_))
            || !is_excluded_introspection_split(def, input_refs, source)
    })
}

/// True for the one edge shape a split cannot preserve (T11.F08, replacing
/// T11.F03's documented exception): on a graph carrying plasticity, a sink
/// reading a
/// `DynamicIntrospection(EnergyCurrent)` reference directly. An identity node
/// between them caches the value during evaluation, while the direct edge
/// resolves it in the post-convergence effects context after the
/// plasticity-cost deduction, so the two can differ.
/// `split_existing_edge` skips such an edge instead of splitting it.
///
/// `EnergyCurrent` is the only excluded key. In `runtime/cgp/execute.rs` the
/// effects `ResolveCtx` differs from the evaluation `ResolveCtx` only in
/// `energy`: `energy_consumed` is unchanged, so `EnergyConsumedThisTick`
/// resolves identically either side of the deduction and its edges split normally.
fn is_excluded_introspection_split(
    def: &CgpGraphBackendDef,
    input_refs: &[InputReference],
    source: GraphSource,
) -> bool {
    let GraphSource::InputLeaf { ref_idx, .. } = source else {
        return false;
    };
    matches!(
        input_refs.get(ref_idx as usize),
        Some(InputReference::DynamicIntrospection(
            DynamicIntrospectionKey::EnergyCurrent
        ))
    ) && def
        .compute_nodes
        .iter()
        .any(|node| node.plasticity.is_some())
}

/// An identity `Add` node (weighted sum reproduces a single input exactly)
/// with the given inputs: the node the split-edge form inserts or appends.
fn identity_add_node(inputs: Vec<GraphEdge>) -> ComputeNode {
    ComputeNode {
        kind: ComputeNodeKind::Add,
        inputs,
        plasticity: None,
    }
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

/// Push a faithful copy (kind, inputs, plasticity) of one random compute
/// node. The copy is not wired to anything else in the graph: it reads
/// whatever its source node read, and nothing reads it back. A copy that
/// should start disconnected is the `AddInternalGraphNode` disconnected
/// form, not a coin-flipped clear here.
pub(crate) fn copy_compute_node(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if def.compute_nodes.is_empty() {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    check_compute_node_capacity(def, 1)?;
    let source_idx = rng.gen_range(0..def.compute_nodes.len());
    def.duplicate_compute_nodes_in_place(&[source_idx]);
    Ok(())
}

/// Added nodes are only addressable while every resulting index stays below
/// the `u16::MAX` dangling-reference sentinel `remove_compute_node_at` uses.
fn check_compute_node_capacity(
    def: &CgpGraphBackendDef,
    added: usize,
) -> Result<(), MutationSkipReason> {
    if def.compute_nodes.len() + added > u16::MAX as usize {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    Ok(())
}

/// The largest random-walk cluster `copy_cgp_subgraph` can build, and so the
/// capacity `can_copy_subgraph` must reserve.
const MAX_SUBGRAPH_CLUSTER: usize = 4;

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
    let target_size = rng
        .gen_range(2..=MAX_SUBGRAPH_CLUSTER)
        .min(def.compute_nodes.len());

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
    check_compute_node_capacity(def, cluster.len())?;
    def.duplicate_compute_nodes_in_place(&cluster);
    Ok(())
}

// ─── Edge operators (across all 5 surfaces) ─────────────────────────────────

/// Add a random edge to a random edge-bearing surface.
pub(crate) fn add_edge(
    def: &mut CgpGraphBackendDef,
    input_refs: &[InputReference],
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let surface = pick_random_surface(def, rng).ok_or(MutationSkipReason::NoApplicableTarget)?;
    let compute_count = def.compute_nodes.len() as u16;
    let source = random_graph_source(compute_count, input_refs, rng);
    let weight = rng.gen_range(-1.0f32..=1.0);
    push_edge(def, surface, GraphEdge { source, weight });
    if let (Some(weights), EdgeSurface::ComputeInput(node)) = (&mut def.birth_weights, surface) {
        weights[node].push(None);
    }
    Ok(())
}

/// Remove a random edge from any surface.
pub(crate) fn remove_edge(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let (surface, edge_idx) =
        pick_random_edge(def, rng).ok_or(MutationSkipReason::NoApplicableTarget)?;
    remove_edge_at(def, surface, edge_idx);
    if let (Some(weights), EdgeSurface::ComputeInput(node)) = (&mut def.birth_weights, surface) {
        weights[node].remove(edge_idx);
    }
    Ok(())
}

/// Change the source of a random edge.
pub(crate) fn retarget_edge(
    def: &mut CgpGraphBackendDef,
    input_refs: &[InputReference],
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let (surface, edge_idx) =
        pick_random_edge(def, rng).ok_or(MutationSkipReason::NoApplicableTarget)?;
    let compute_count = def.compute_nodes.len() as u16;
    let new_source = random_graph_source(compute_count, input_refs, rng);
    let edge = edge_at_mut(def, surface, edge_idx);
    let changed = edge.source != new_source;
    edge.source = new_source;
    if changed {
        reset_inherited_edge(def, surface, edge_idx);
    }
    Ok(())
}

fn reset_inherited_edge(def: &mut CgpGraphBackendDef, surface: EdgeSurface, edge: usize) {
    if let (Some(weights), EdgeSurface::ComputeInput(node)) = (&mut def.birth_weights, surface) {
        weights[node][edge] = None;
    }
}

/// Alter the weight of a random edge.
pub(crate) fn alter_edge_weight_in_def(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let (surface, edge_idx) =
        pick_random_edge(def, rng).ok_or(MutationSkipReason::NoApplicableTarget)?;
    reset_inherited_edge(def, surface, edge_idx);
    let w = &mut edge_at_mut(def, surface, edge_idx).weight;
    if w.abs() > 0.01 {
        *w *= 1.0 + rng.gen_range(-0.2f32..=0.2);
    } else {
        *w += rng.gen_range(-0.1f32..=0.1);
    }
    Ok(())
}

/// The compute nodes `copy_edge_bundle` can copy a bundle from: those that
/// have at least one edge to copy.
fn bundle_source_indices(def: &CgpGraphBackendDef) -> impl Iterator<Item = usize> + '_ {
    def.compute_nodes
        .iter()
        .enumerate()
        .filter(|(_, node)| !node.inputs.is_empty())
        .map(|(i, _)| i)
}

/// Copy all edges from one compute node to another. The source is drawn
/// from [`bundle_source_indices`], so a def with an edge-bearing node never
/// skips here.
pub(crate) fn copy_edge_bundle(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if def.compute_nodes.len() < 2 {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    let sources: Vec<usize> = bundle_source_indices(def).collect();
    if sources.is_empty() {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    let source = sources[rng.gen_range(0..sources.len())];
    let mut target = rng.gen_range(0..def.compute_nodes.len() - 1);
    if target >= source {
        target += 1;
    }
    let copied = def.compute_nodes[source].inputs.clone();
    def.compute_nodes[target].inputs.extend(copied);
    if let Some(weights) = &mut def.birth_weights {
        let copied = weights[source].clone();
        weights[target].extend(copied);
    }
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

// ─── Raw field mutation ─────────────────────────────────────────────────────

/// One field-level move available on a picked edge's `GraphSource`. Each
/// variant changes exactly one field by one unit; the source variant itself
/// is never replaced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EdgeFieldMove {
    ComputeIdx(i32),
    RefIdx(i32),
    SubIdx(i32),
    SharedSlot(i32),
    FlipPrevious,
}

/// Every field-level move valid for the edge at `edge_idx` on `surface`.
fn valid_edge_moves(
    def: &CgpGraphBackendDef,
    surface: EdgeSurface,
    edge_idx: usize,
    input_refs: &[InputReference],
) -> Vec<EdgeFieldMove> {
    valid_edge_field_moves(
        edge_at(def, surface, edge_idx).source,
        def.compute_nodes.len() as u16,
        input_refs,
    )
}

/// Enumerate every field-level move that is a valid single-step change for
/// `source`, given the current graph size and input references. "Inward"
/// moves are excluded at either bound; an `InputLeaf.ref_idx` move is only
/// offered when the current `sub_idx` stays within the candidate
/// reference's width.
fn valid_edge_field_moves(
    source: GraphSource,
    compute_count: u16,
    input_refs: &[InputReference],
) -> Vec<EdgeFieldMove> {
    let mut moves = Vec::new();
    match source {
        GraphSource::ComputeNode(idx) => {
            if idx < compute_count {
                if idx > 0 {
                    moves.push(EdgeFieldMove::ComputeIdx(-1));
                }
                if idx + 1 < compute_count {
                    moves.push(EdgeFieldMove::ComputeIdx(1));
                }
            }
        }
        GraphSource::InputLeaf { ref_idx, sub_idx } => {
            let ref_count = input_refs.len() as u16;
            if ref_idx > 0 {
                let candidate = (ref_idx - 1) as usize;
                if let Some(reference) = input_refs.get(candidate) {
                    let width = sub_value_count(reference).max(1);
                    if sub_idx < width {
                        moves.push(EdgeFieldMove::RefIdx(-1));
                    }
                }
            }
            if ref_idx + 1 < ref_count {
                let candidate = (ref_idx + 1) as usize;
                if let Some(reference) = input_refs.get(candidate) {
                    let width = sub_value_count(reference).max(1);
                    if sub_idx < width {
                        moves.push(EdgeFieldMove::RefIdx(1));
                    }
                }
            }
            if (ref_idx as usize) < input_refs.len() {
                let width = sub_value_count(&input_refs[ref_idx as usize]).max(1);
                if sub_idx > 0 {
                    moves.push(EdgeFieldMove::SubIdx(-1));
                }
                if sub_idx + 1 < width {
                    moves.push(EdgeFieldMove::SubIdx(1));
                }
            }
        }
        GraphSource::SharedMemory { .. } => {
            // Both slot directions wrap modulo 16 and the previous-tick flag
            // always has a flip, so this variant always has a valid move.
            moves.push(EdgeFieldMove::SharedSlot(-1));
            moves.push(EdgeFieldMove::SharedSlot(1));
            moves.push(EdgeFieldMove::FlipPrevious);
        }
    }
    moves
}

fn apply_edge_field_move(source: &mut GraphSource, mv: EdgeFieldMove) {
    match (source, mv) {
        (GraphSource::ComputeNode(idx), EdgeFieldMove::ComputeIdx(delta)) => {
            *idx = (i32::from(*idx) + delta) as u16;
        }
        (GraphSource::InputLeaf { ref_idx, .. }, EdgeFieldMove::RefIdx(delta)) => {
            *ref_idx = (i32::from(*ref_idx) + delta) as u16;
        }
        (GraphSource::InputLeaf { sub_idx, .. }, EdgeFieldMove::SubIdx(delta)) => {
            *sub_idx = (i32::from(*sub_idx) + delta) as u16;
        }
        (GraphSource::SharedMemory { slot, .. }, EdgeFieldMove::SharedSlot(delta)) => {
            *slot = (i32::from(*slot) + delta).rem_euclid(16) as u8;
        }
        (GraphSource::SharedMemory { previous, .. }, EdgeFieldMove::FlipPrevious) => {
            *previous = !*previous;
        }
        (source, mv) => unreachable!(
            "valid_edge_field_moves only returns moves matching the source variant and \
             direction moves are applied to the bank edge, not its source; got {source:?} with {mv:?}"
        ),
    }
}

/// The edge sites `raw_field_mutation` can move a field on: those whose
/// `GraphSource` has at least one valid unit move.
fn raw_field_edge_sites<'a>(
    def: &'a CgpGraphBackendDef,
    input_refs: &'a [InputReference],
) -> impl Iterator<Item = (EdgeSurface, usize)> + 'a {
    edge_sites(def).filter(move |&(surface, edge_idx)| {
        !valid_edge_moves(def, surface, edge_idx, input_refs).is_empty()
    })
}

/// Change exactly one field of one target by one unit: a parameterized
/// compute node's parameter (the existing `MutateGraphOperatorParam` step),
/// or one field of one edge's `GraphSource` (a `ComputeNode` index, an
/// `InputLeaf` ref or sub index, a `SharedMemory` slot, or its previous-tick
/// flag). The target and, for edges, the field are both selected uniformly
/// among the moves that are valid for the picked target; the source variant
/// is never replaced. Edges with no valid unit move are excluded from the
/// draw by [`raw_field_edge_sites`], so a def [`has_raw_field_site`] accepts
/// never skips here.
pub(crate) fn raw_field_mutation(
    def: &mut CgpGraphBackendDef,
    input_refs: &[InputReference],
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let param_count = def
        .compute_nodes
        .iter()
        .filter(|n| is_compute_parameterized(&n.kind))
        .count();
    let edges: Vec<(EdgeSurface, usize)> = raw_field_edge_sites(def, input_refs).collect();
    let total = param_count + edges.len();

    if total == 0 {
        return Err(MutationSkipReason::NoApplicableTarget);
    }

    let pick = rng.gen_range(0..total);
    if pick < param_count {
        return mutate_compute_param(def, rng);
    }

    let (surface, edge_idx) = edges[pick - param_count];
    let moves = valid_edge_moves(def, surface, edge_idx, input_refs);
    let chosen = moves[rng.gen_range(0..moves.len())];
    let source = edge_at(def, surface, edge_idx).source;
    apply_edge_field_move(&mut edge_at_mut(def, surface, edge_idx).source, chosen);
    if edge_at(def, surface, edge_idx).source != source {
        reset_inherited_edge(def, surface, edge_idx);
    }
    Ok(())
}

// ─── Applicability predicates ───────────────────────────────────────────────
//
// One predicate per Graph operator, over a single node's backend def (and
// its `input_refs` where the operator reads them). `GraphMutator::apply`
// filters the Graph-backend node indices by the operator's predicate before
// the biased draw, so an operator is never selected onto a node it cannot
// apply to. Each predicate tests exactly the site enumeration its operator
// draws from, so a predicate that accepts a node guarantees application
// succeeds on it.

/// `AlterGraphEdgeWeight`, `RetargetGraphEdge`, `RemoveGraphEdge`: any edge
/// on any surface is a site, which is exactly what `pick_random_edge` draws.
pub(super) fn has_edge_site(def: &CgpGraphBackendDef) -> bool {
    total_edge_count(def) > 0
}

/// `SwapGraphOperator`, `RemoveInternalGraphNode`: any compute node.
pub(super) fn has_compute_node(def: &CgpGraphBackendDef) -> bool {
    !def.compute_nodes.is_empty()
}

/// `MutateGraphOperatorParam`: a compute node carrying a float parameter,
/// the set `mutate_compute_param` draws from.
pub(super) fn has_parameterized_compute_node(def: &CgpGraphBackendDef) -> bool {
    def.compute_nodes
        .iter()
        .any(|node| is_compute_parameterized(&node.kind))
}

/// `CopyInternalNode`: a compute node to copy, with room for the copy.
pub(super) fn can_copy_compute_node(def: &CgpGraphBackendDef) -> bool {
    !def.compute_nodes.is_empty() && check_compute_node_capacity(def, 1).is_ok()
}

/// `CopySubgraph`: at least two compute nodes to walk, with room for the
/// largest cluster the walk can build (`target_size` is capped at 4 and at
/// the node count).
pub(super) fn can_copy_subgraph(def: &CgpGraphBackendDef) -> bool {
    def.compute_nodes.len() >= 2
        && check_compute_node_capacity(def, MAX_SUBGRAPH_CLUSTER.min(def.compute_nodes.len()))
            .is_ok()
}

/// `CopyEdgeBundle`: a compute node with edges to copy, plus a second node
/// to receive them.
pub(super) fn can_copy_edge_bundle(def: &CgpGraphBackendDef) -> bool {
    def.compute_nodes.len() >= 2 && bundle_source_indices(def).next().is_some()
}

/// `GraphRawFieldMutation`: a parameterized compute node, or an edge with at
/// least one valid unit move. The parameter check comes first so the common
/// case never walks the edges.
pub(super) fn has_raw_field_site(def: &CgpGraphBackendDef, input_refs: &[InputReference]) -> bool {
    has_parameterized_compute_node(def) || raw_field_edge_sites(def, input_refs).next().is_some()
}

/// `AddInternalGraphNode`: the disconnected form appends to any def, so
/// every Graph node is a target and blank modules keep their growth reach.
pub(super) const fn can_add_compute_node(_def: &CgpGraphBackendDef) -> bool {
    true
}

/// `AddGraphEdge`: any compute node or catalog sink is a surface, the set
/// `pick_random_surface` draws from. Every graph built on the fixed catalog
/// qualifies.
pub(super) fn can_add_edge(def: &CgpGraphBackendDef) -> bool {
    !def.compute_nodes.is_empty() || !def.output_sinks.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::OrdinaryFoodTypeId;
    use crate::contracts::{InputReference, WorldInputKey};
    use crate::creature::genome::cgp::{OutputSink, OutputSinkKind};
    use crate::creature::genome::vote::VoteSink;
    use rand::SeedableRng;

    fn test_rng() -> rand::rngs::SmallRng {
        rand::rngs::SmallRng::seed_from_u64(42)
    }

    fn minimal_def() -> CgpGraphBackendDef {
        CgpGraphBackendDef {
            birth_weights: None,
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
                        source: GraphSource::ComputeNode(0),
                        weight: 1.0,
                    }],
                },
            ],
        }
    }

    // ── Topology tests ──────────────────────────────────────────────────────

    fn sample_input_refs() -> Vec<InputReference> {
        vec![
            InputReference::World(WorldInputKey::food_here(OrdinaryFoodTypeId::default())),
            InputReference::World(WorldInputKey::NeighborBarrierRing),
            InputReference::ActionQueue,
        ]
    }

    #[test]
    fn add_compute_node_increases_count() {
        let mut def = minimal_def();
        let mut rng = test_rng();
        let before = def.compute_nodes.len();
        let input_refs = sample_input_refs();
        add_compute_node(&mut def, &input_refs, &mut rng).unwrap();
        assert_eq!(def.compute_nodes.len(), before + 1);
    }

    /// None of the three add-node forms sprays edges into unrelated surfaces:
    /// the disconnected form adds zero edges, the bootstrap form adds exactly
    /// one (the new node's own input), and the split form adds exactly one
    /// (also the new node's own input; the split edge is retargeted, not
    /// duplicated). Every existing edge not touched by a split keeps its
    /// exact prior source and weight.
    #[test]
    fn add_compute_node_never_sprays_edges() {
        let input_refs = sample_input_refs();
        for seed in 0u64..500 {
            let mut def = minimal_def();
            let before_nodes = def.compute_nodes.len();
            let before_edges = total_edge_count(&def);
            let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
            add_compute_node(&mut def, &input_refs, &mut rng).unwrap();
            assert_eq!(def.compute_nodes.len(), before_nodes + 1, "seed {seed}");
            let after_edges = total_edge_count(&def);
            assert!(
                after_edges <= before_edges + 1,
                "seed {seed}: edges grew by more than one ({before_edges} -> {after_edges})"
            );
        }
    }

    #[test]
    fn add_disconnected_node_form_has_no_inputs() {
        let mut def = minimal_def();
        let before = total_edge_count(&def);
        let mut rng = test_rng();
        add_disconnected_node(&mut def, &mut rng).unwrap();
        assert!(def.compute_nodes.last().unwrap().inputs.is_empty());
        assert_eq!(total_edge_count(&def), before);
    }

    #[test]
    fn add_bootstrap_node_form_has_exactly_one_unwired_input() {
        let mut def = minimal_def();
        let input_refs = sample_input_refs();
        let before = total_edge_count(&def);
        let mut rng = test_rng();
        add_bootstrap_node(&mut def, &input_refs, &mut rng).unwrap();
        assert_eq!(def.compute_nodes.last().unwrap().inputs.len(), 1);
        // No surface reads the new node back.
        let new_idx = (def.compute_nodes.len() - 1) as u16;
        for node in &def.compute_nodes[..def.compute_nodes.len() - 1] {
            assert!(node
                .inputs
                .iter()
                .all(|e| e.source != GraphSource::ComputeNode(new_idx)));
        }
        assert_eq!(total_edge_count(&def), before + 1);
    }

    #[test]
    fn split_existing_edge_no_edges_is_skip() {
        let mut def = CgpGraphBackendDef {
            birth_weights: None,
            compute_nodes: vec![ComputeNode {
                kind: ComputeNodeKind::Constant(1.0),
                inputs: Vec::new(),
                plasticity: None,
            }],
            output_sinks: Vec::new(),
        };
        let mut rng = test_rng();
        assert_eq!(
            split_existing_edge(&mut def, &[], &mut rng),
            Err(MutationSkipReason::NoApplicableTarget)
        );
    }

    #[test]
    fn split_existing_edge_dangling_compute_source_is_skip() {
        let mut def = CgpGraphBackendDef {
            birth_weights: None,
            compute_nodes: vec![ComputeNode {
                kind: ComputeNodeKind::Constant(1.0),
                inputs: Vec::new(),
                plasticity: None,
            }],
            output_sinks: vec![OutputSink {
                kind: OutputSinkKind::CustomOutput(0),
                inputs: vec![GraphEdge {
                    source: GraphSource::ComputeNode(u16::MAX),
                    weight: 1.0,
                }],
            }],
        };
        let mut rng = test_rng();
        assert_eq!(
            split_existing_edge(&mut def, &[], &mut rng),
            Err(MutationSkipReason::NoApplicableTarget)
        );
    }

    /// Splitting an edge whose consumer is a compute node inserts the new
    /// `Add` node at the consumer's index, remaps every `ComputeNode(i >=
    /// consumer)` reference, and retargets the split edge to the new node
    /// while keeping its old weight. This is the example the property tests
    /// in `mutation/graph/tests/operators.rs` generalize.
    #[test]
    fn split_existing_edge_compute_consumer_inserts_and_remaps() {
        let mut def = CgpGraphBackendDef {
            birth_weights: None,
            compute_nodes: vec![
                ComputeNode {
                    kind: ComputeNodeKind::Constant(0.25),
                    inputs: Vec::new(),
                    plasticity: None,
                },
                ComputeNode {
                    kind: ComputeNodeKind::Sigmoid,
                    inputs: vec![GraphEdge {
                        source: GraphSource::ComputeNode(0),
                        weight: 0.75,
                    }],
                    plasticity: None,
                },
            ],
            output_sinks: vec![OutputSink {
                kind: OutputSinkKind::ActionVote(VoteSink::Terminate),
                inputs: vec![GraphEdge {
                    source: GraphSource::ComputeNode(1),
                    weight: 1.0,
                }],
            }],
        };
        // Force selection of the only edge (ComputeInput(1), edge 0).
        let mut rng = test_rng();
        split_existing_edge(&mut def, &[], &mut rng).unwrap();

        assert_eq!(def.compute_nodes.len(), 3);
        // New Add node inserted at index 1, reading the old source (CN(0)).
        assert_eq!(def.compute_nodes[1].kind, ComputeNodeKind::Add);
        assert_eq!(def.compute_nodes[1].inputs.len(), 1);
        assert_eq!(
            def.compute_nodes[1].inputs[0].source,
            GraphSource::ComputeNode(0)
        );
        assert_eq!(def.compute_nodes[1].inputs[0].weight, 1.0);
        // Sigmoid (the consumer) shifted to index 2, now reads the new node,
        // keeping its old weight.
        assert_eq!(def.compute_nodes[2].kind, ComputeNodeKind::Sigmoid);
        assert_eq!(
            def.compute_nodes[2].inputs[0].source,
            GraphSource::ComputeNode(1)
        );
        assert_eq!(def.compute_nodes[2].inputs[0].weight, 0.75);
        // The vote sink's unrelated reference to the shifted Sigmoid is
        // remapped too.
        assert_eq!(
            def.output_sinks[0].inputs[0].source,
            GraphSource::ComputeNode(2)
        );
    }

    /// Splitting a self-loop shifts the new node's input to the consumer's
    /// own new (shifted) index, so the new node reads the consumer's prior
    /// value one pass later rather than reproducing a stale reference.
    #[test]
    fn split_existing_edge_self_loop_shifts_consistently() {
        let mut def = CgpGraphBackendDef {
            birth_weights: None,
            compute_nodes: vec![ComputeNode {
                kind: ComputeNodeKind::DecayIntegrator(0.5),
                inputs: vec![GraphEdge {
                    source: GraphSource::ComputeNode(0),
                    weight: 1.0,
                }],
                plasticity: None,
            }],
            output_sinks: Vec::new(),
        };
        let mut rng = test_rng();
        split_existing_edge(&mut def, &[], &mut rng).unwrap();

        assert_eq!(def.compute_nodes.len(), 2);
        assert_eq!(def.compute_nodes[0].kind, ComputeNodeKind::Add);
        // The new node reads the shifted consumer's own new index (1).
        assert_eq!(
            def.compute_nodes[0].inputs[0].source,
            GraphSource::ComputeNode(1)
        );
        // The consumer's own edge now reads the new node.
        assert_eq!(
            def.compute_nodes[1].inputs[0].source,
            GraphSource::ComputeNode(0)
        );
    }

    /// Splitting an edge whose consumer is a sink appends the new node
    /// instead of inserting at an index.
    #[test]
    fn split_existing_edge_sink_consumer_appends() {
        let mut def = CgpGraphBackendDef {
            birth_weights: None,
            compute_nodes: vec![ComputeNode {
                kind: ComputeNodeKind::Constant(0.5),
                inputs: Vec::new(),
                plasticity: None,
            }],
            output_sinks: vec![OutputSink {
                kind: OutputSinkKind::CustomOutput(0),
                inputs: vec![GraphEdge {
                    source: GraphSource::ComputeNode(0),
                    weight: 2.0,
                }],
            }],
        };
        let mut rng = test_rng();
        split_existing_edge(&mut def, &[], &mut rng).unwrap();

        assert_eq!(def.compute_nodes.len(), 2);
        assert_eq!(def.compute_nodes[1].kind, ComputeNodeKind::Add);
        assert_eq!(
            def.compute_nodes[1].inputs[0].source,
            GraphSource::ComputeNode(0)
        );
        assert_eq!(def.compute_nodes[1].inputs[0].weight, 1.0);
        assert_eq!(
            def.output_sinks[0].inputs[0].source,
            GraphSource::ComputeNode(1)
        );
        assert_eq!(def.output_sinks[0].inputs[0].weight, 2.0);
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
            birth_weights: None,
            compute_nodes: Vec::new(),
            output_sinks: Vec::new(),
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
            birth_weights: None,
            compute_nodes: vec![ComputeNode {
                kind: ComputeNodeKind::Add,
                inputs: Vec::new(),
                plasticity: None,
            }],
            output_sinks: Vec::new(),
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
        let input_refs = sample_input_refs();
        add_edge(&mut def, &input_refs, &mut rng).unwrap();
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
            birth_weights: None,
            compute_nodes: vec![ComputeNode {
                kind: ComputeNodeKind::Add,
                inputs: vec![GraphEdge {
                    source: GraphSource::ComputeNode(0),
                    weight: 1.0,
                }],
                plasticity: None,
            }],
            output_sinks: Vec::new(),
        };
        let original = def.compute_nodes[0].inputs[0].source;
        let input_refs = sample_input_refs();
        let mut changed = false;
        for seed in 0u64..20 {
            let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
            retarget_edge(&mut def, &input_refs, &mut rng).unwrap();
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
            birth_weights: None,
            compute_nodes: vec![ComputeNode {
                kind: ComputeNodeKind::Add,
                inputs: vec![GraphEdge {
                    source: GraphSource::ComputeNode(0),
                    weight: 1.0,
                }],
                plasticity: None,
            }],
            output_sinks: Vec::new(),
        };
        let original_weight = 1.0f32;
        let mut rng = test_rng();
        for _ in 0..10 {
            alter_edge_weight_in_def(&mut def, &mut rng).unwrap();
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
            birth_weights: None,
            compute_nodes: vec![ComputeNode {
                kind: ComputeNodeKind::Add,
                inputs: Vec::new(),
                plasticity: None,
            }],
            output_sinks: Vec::new(),
        };
        let mut rng = test_rng();
        assert!(mutate_compute_param(&mut def, &mut rng).is_err());
    }

    // ── Random generation tests ─────────────────────────────────────────────

    #[test]
    fn random_graph_source_covers_variants() {
        let mut rng = test_rng();
        let input_refs = sample_input_refs();
        let mut seen_compute = false;
        let mut seen_input = false;
        let mut seen_shared = false;
        for _ in 0..100 {
            match random_graph_source(5, &input_refs, &mut rng) {
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
            match random_graph_source(0, &[], &mut rng) {
                GraphSource::SharedMemory { slot, .. } => assert!(slot < 16),
                other => panic!("expected SharedMemory, got {:?}", other),
            }
        }
    }

    /// Over enough draws, `random_graph_source` reaches every sub-value of a
    /// compound input reference, not just index 0.
    #[test]
    fn random_graph_source_reaches_every_sub_value_of_a_compound_reference() {
        let input_refs = vec![InputReference::World(WorldInputKey::NeighborBarrierRing)]; // width 8
        let mut seen: std::collections::HashSet<u16> = std::collections::HashSet::new();
        let mut rng = test_rng();
        for _ in 0..2000 {
            if let GraphSource::InputLeaf {
                ref_idx: 0,
                sub_idx,
            } = random_graph_source(0, &input_refs, &mut rng)
            {
                seen.insert(sub_idx);
            }
        }
        for sub_idx in 0..8u16 {
            assert!(
                seen.contains(&sub_idx),
                "sub_idx={sub_idx} should be reachable; saw {seen:?}"
            );
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
    fn add_edge_reaches_the_vote_and_parameter_sinks() {
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
        def.compute_nodes.push(ComputeNode {
            kind: ComputeNodeKind::Constant(1.0),
            inputs: Vec::new(),
            plasticity: None,
        });

        let mut rng = test_rng();
        for _ in 0..200 {
            let _ = add_edge(&mut def, &[], &mut rng);
        }
        let wired = |pick: fn(OutputSinkKind) -> bool| {
            def.output_sinks
                .iter()
                .any(|sink| pick(sink.kind) && !sink.inputs.is_empty())
        };
        assert!(wired(|kind| matches!(kind, OutputSinkKind::ActionVote(_))));
        assert!(wired(|kind| matches!(
            kind,
            OutputSinkKind::ActionParam(_, _)
        )));
    }

    // ── Raw-field mutation tests ────────────────────────────────────────────

    #[test]
    fn raw_field_mutation_compute_node_index_moves_by_one_unit_inward() {
        // Non-parameterized kinds throughout, so `param_count == 0` and the
        // only eligible target is the Add node's one edge — every trial
        // mutates that edge's ComputeNode index.
        let def = CgpGraphBackendDef {
            birth_weights: None,
            compute_nodes: vec![
                ComputeNode {
                    kind: ComputeNodeKind::WeightedSum,
                    inputs: Vec::new(),
                    plasticity: None,
                },
                ComputeNode {
                    kind: ComputeNodeKind::Negate,
                    inputs: Vec::new(),
                    plasticity: None,
                },
                ComputeNode {
                    kind: ComputeNodeKind::Add,
                    inputs: vec![GraphEdge {
                        source: GraphSource::ComputeNode(1),
                        weight: 1.0,
                    }],
                    plasticity: None,
                },
            ],
            output_sinks: Vec::new(),
        };
        for seed in 0u64..64 {
            let mut d = def.clone();
            let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
            raw_field_mutation(&mut d, &[], &mut rng).unwrap();
            let GraphSource::ComputeNode(idx) = d.compute_nodes[2].inputs[0].source else {
                panic!("variant must not change");
            };
            assert!(idx == 0 || idx == 2, "moved by one unit inward, got {idx}");
        }
    }

    #[test]
    fn raw_field_mutation_shared_memory_slot_wraps_modulo_16() {
        let def = CgpGraphBackendDef {
            birth_weights: None,
            compute_nodes: vec![ComputeNode {
                kind: ComputeNodeKind::Add,
                inputs: vec![GraphEdge {
                    source: GraphSource::SharedMemory {
                        slot: 0,
                        previous: false,
                    },
                    weight: 1.0,
                }],
                plasticity: None,
            }],
            output_sinks: Vec::new(),
        };
        for seed in 0u64..64 {
            let mut d = def.clone();
            let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
            raw_field_mutation(&mut d, &[], &mut rng).unwrap();
            match d.compute_nodes[0].inputs[0].source {
                GraphSource::SharedMemory { slot, previous } => {
                    // Either the slot moved by one unit (wrapping to 15
                    // going down from 0) with `previous` untouched, or the
                    // slot stayed put and `previous` flipped to true.
                    assert!(if previous {
                        slot == 0
                    } else {
                        slot == 1 || slot == 15
                    });
                }
                other => panic!("variant must not change, got {other:?}"),
            }
        }
    }

    #[test]
    fn raw_field_mutation_never_changes_more_than_one_field() {
        let base_refs = vec![
            InputReference::World(WorldInputKey::food_here(OrdinaryFoodTypeId::default())),
            InputReference::World(WorldInputKey::NeighborBarrierRing),
        ];
        let def = CgpGraphBackendDef {
            birth_weights: None,
            compute_nodes: vec![
                ComputeNode {
                    kind: ComputeNodeKind::Constant(0.5),
                    inputs: Vec::new(),
                    plasticity: None,
                },
                ComputeNode {
                    kind: ComputeNodeKind::Add,
                    inputs: vec![
                        GraphEdge {
                            source: GraphSource::ComputeNode(0),
                            weight: 1.0,
                        },
                        GraphEdge {
                            source: GraphSource::InputLeaf {
                                ref_idx: 1,
                                sub_idx: 3,
                            },
                            weight: 1.0,
                        },
                        GraphEdge {
                            source: GraphSource::SharedMemory {
                                slot: 5,
                                previous: false,
                            },
                            weight: 1.0,
                        },
                    ],
                    plasticity: None,
                },
            ],
            output_sinks: Vec::new(),
        };
        for seed in 0u64..500 {
            let mut d = def.clone();
            let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
            if raw_field_mutation(&mut d, &base_refs, &mut rng).is_err() {
                continue;
            }
            let before_edges: Vec<GraphEdge> = def.compute_nodes[1].inputs.clone();
            let after_edges: Vec<GraphEdge> = d.compute_nodes[1].inputs.clone();
            let param_changed = d.compute_nodes[0].kind != def.compute_nodes[0].kind;
            let edges_changed = before_edges != after_edges;
            assert!(
                param_changed ^ edges_changed || (!param_changed && !edges_changed),
                "seed {seed}: exactly the param or the edges may change, not both"
            );
            if edges_changed {
                let diffs = before_edges
                    .iter()
                    .zip(after_edges.iter())
                    .filter(|(a, b)| a != b)
                    .count();
                assert_eq!(diffs, 1, "seed {seed}: exactly one edge should change");
            }
        }
    }

    #[test]
    fn raw_field_mutation_skips_single_compute_node_index_move() {
        // A single compute node's self-referencing edge has no valid
        // ComputeNode index move (0 is the only valid index).
        let mut def = CgpGraphBackendDef {
            birth_weights: None,
            compute_nodes: vec![ComputeNode {
                kind: ComputeNodeKind::Add,
                inputs: vec![GraphEdge {
                    source: GraphSource::ComputeNode(0),
                    weight: 1.0,
                }],
                plasticity: None,
            }],
            output_sinks: Vec::new(),
        };
        let mut rng = test_rng();
        assert_eq!(
            raw_field_mutation(&mut def, &[], &mut rng),
            Err(MutationSkipReason::NoApplicableTarget)
        );
    }

    // ── Mutation-survivor coverage ──────────────────────────────────────────
    //
    // The tests below were added to close `cargo-mutants` survivors found in
    // the fresh run recorded in the T11.F03 spec's Verification section. Each
    // pins an exact structural outcome so the named operator mutation (an
    // inverted comparison, a flipped arithmetic sign, or a deleted match arm)
    // changes the asserted value.

    /// `add_graph_edge`'s wrapper must actually add an edge, not just report
    /// success (kills the `-> Ok(())` stub mutant at its definition).
    #[test]
    fn add_graph_edge_wrapper_increases_edge_count() {
        use crate::contracts::NodeId;
        use crate::creature::genome::{BackendDef, CreatureGenome, NodeGenome};

        let def = minimal_def();
        let before = total_edge_count(&def);
        let mut genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(0),
                input_refs: sample_input_refs(),
                backend_def: BackendDef::Graph(def),
                targets: Vec::new(),
            }],
        };
        let mut rng = test_rng();
        add_graph_edge(&mut genome, 0, &mut rng).expect("edge add should succeed");
        let BackendDef::Graph(after_def) = &genome.nodes[0].backend_def else {
            panic!("expected a Cgp graph node");
        };
        assert_eq!(total_edge_count(after_def), before + 1);
    }

    /// With `compute_count == 0`, `random_graph_source`'s first branch is
    /// unreachable, so the `roll < 0.8` comparison alone decides InputLeaf
    /// (80%) vs SharedMemory (20%). Flipping it to `>` swaps the majority to
    /// SharedMemory; a wide margin over many fixed draws catches that
    /// inversion without depending on which cases were drawn. The `<=`
    /// variant is deferred; see the spec's mutation record.
    #[test]
    fn random_graph_source_input_leaf_is_the_majority_when_compute_is_empty() {
        let input_refs = vec![InputReference::World(WorldInputKey::NeighborBarrierRing)];
        let mut rng = test_rng();
        let mut input_leaf_count = 0u32;
        let mut shared_memory_count = 0u32;
        for _ in 0..2000 {
            match random_graph_source(0, &input_refs, &mut rng) {
                GraphSource::InputLeaf { .. } => input_leaf_count += 1,
                GraphSource::SharedMemory { .. } => shared_memory_count += 1,
                GraphSource::ComputeNode(_) => {
                    panic!("compute_count == 0 must never yield a ComputeNode source")
                }
            }
        }
        assert!(
            input_leaf_count > shared_memory_count * 2,
            "expected InputLeaf (~80%) to dominate SharedMemory (~20%); saw {input_leaf_count} vs {shared_memory_count}"
        );
    }

    /// `add_compute_node`'s three-way dispatch must reach every form. The
    /// fixture's only edge lives on a vote sink (not a compute-node
    /// consumer), so `split_existing_edge` always appends its new node and
    /// retargets that one edge — giving each form an exact, non-overlapping
    /// structural fingerprint (kills both match-arm-deletion mutants).
    #[test]
    fn add_compute_node_reaches_all_three_forms_with_distinct_signatures() {
        fn fixture() -> CgpGraphBackendDef {
            CgpGraphBackendDef {
                birth_weights: None,
                compute_nodes: vec![ComputeNode {
                    kind: ComputeNodeKind::Negate,
                    inputs: Vec::new(),
                    plasticity: None,
                }],
                output_sinks: vec![OutputSink {
                    kind: OutputSinkKind::ActionVote(VoteSink::Terminate),
                    inputs: vec![GraphEdge {
                        source: GraphSource::ComputeNode(0),
                        weight: 0.5,
                    }],
                }],
            }
        }

        let input_refs = sample_input_refs();
        let mut rng = test_rng();
        let mut saw_disconnected = false;
        let mut saw_bootstrap = false;
        let mut saw_split = false;
        for _ in 0..300 {
            let mut def = fixture();
            add_compute_node(&mut def, &input_refs, &mut rng).unwrap();
            assert_eq!(
                def.compute_nodes.len(),
                2,
                "every form appends exactly one node"
            );
            let new_node = &def.compute_nodes[1];
            let gate_source = def.output_sinks[0].inputs[0].source;
            if gate_source == GraphSource::ComputeNode(1) {
                // Split: the pre-existing edge now sources the new identity
                // node, which reproduces the old source exactly.
                assert_eq!(
                    def.output_sinks[0].inputs[0].weight, 0.5,
                    "split preserves the old weight"
                );
                assert_eq!(new_node.kind, ComputeNodeKind::Add);
                assert_eq!(
                    new_node.inputs,
                    vec![GraphEdge {
                        source: GraphSource::ComputeNode(0),
                        weight: 1.0,
                    }]
                );
                saw_split = true;
            } else {
                assert_eq!(
                    gate_source,
                    GraphSource::ComputeNode(0),
                    "only split retargets the gate edge"
                );
                match new_node.inputs.len() {
                    0 => saw_disconnected = true,
                    1 => saw_bootstrap = true,
                    n => panic!("unexpected input count {n} for a non-split form"),
                }
            }
        }
        assert!(saw_disconnected, "disconnected form never observed");
        assert!(saw_bootstrap, "bootstrap form never observed");
        assert!(saw_split, "split form never observed");
    }

    /// Exhaustive boundary cases for `valid_edge_field_moves`'s `ComputeNode`
    /// branch: no valid move at either end of a length-1 range, one move
    /// each at the two ends of a longer range, both moves in the interior,
    /// and no moves for a dangling out-of-range index (including exactly
    /// `idx == compute_count`, distinct from `idx > compute_count`).
    #[test]
    fn valid_edge_field_moves_compute_node_bounds() {
        assert_eq!(
            valid_edge_field_moves(GraphSource::ComputeNode(0), 1, &[]),
            vec![]
        );
        assert_eq!(
            valid_edge_field_moves(GraphSource::ComputeNode(0), 3, &[]),
            vec![EdgeFieldMove::ComputeIdx(1)]
        );
        assert_eq!(
            valid_edge_field_moves(GraphSource::ComputeNode(2), 3, &[]),
            vec![EdgeFieldMove::ComputeIdx(-1)]
        );
        assert_eq!(
            valid_edge_field_moves(GraphSource::ComputeNode(1), 3, &[]),
            vec![EdgeFieldMove::ComputeIdx(-1), EdgeFieldMove::ComputeIdx(1)]
        );
        assert_eq!(
            valid_edge_field_moves(GraphSource::ComputeNode(3), 3, &[]),
            vec![],
            "idx == compute_count is out of range, not the last valid index"
        );
        assert_eq!(
            valid_edge_field_moves(GraphSource::ComputeNode(5), 3, &[]),
            vec![]
        );
    }

    /// Exhaustive boundary cases for `valid_edge_field_moves`'s `InputLeaf`
    /// branch, using references wide enough (8 sub-values each) that a
    /// middling `ref_idx`/`sub_idx` exercises `RefIdx` and `SubIdx` moves in
    /// both directions simultaneously.
    #[test]
    fn valid_edge_field_moves_input_leaf_bounds() {
        let wide_refs = vec![
            InputReference::World(WorldInputKey::NeighborBarrierRing),
            InputReference::World(WorldInputKey::NeighborBarrierRing),
            InputReference::World(WorldInputKey::NeighborBarrierRing),
        ];

        // Interior ref_idx and interior sub_idx: all four moves, in
        // RefIdx(-1), RefIdx(1), SubIdx(-1), SubIdx(1) order.
        assert_eq!(
            valid_edge_field_moves(
                GraphSource::InputLeaf {
                    ref_idx: 1,
                    sub_idx: 3
                },
                0,
                &wide_refs
            ),
            vec![
                EdgeFieldMove::RefIdx(-1),
                EdgeFieldMove::RefIdx(1),
                EdgeFieldMove::SubIdx(-1),
                EdgeFieldMove::SubIdx(1),
            ]
        );
        // sub_idx at the reference's first slot: no SubIdx(-1).
        assert_eq!(
            valid_edge_field_moves(
                GraphSource::InputLeaf {
                    ref_idx: 1,
                    sub_idx: 0
                },
                0,
                &wide_refs
            ),
            vec![
                EdgeFieldMove::RefIdx(-1),
                EdgeFieldMove::RefIdx(1),
                EdgeFieldMove::SubIdx(1)
            ]
        );
        // sub_idx at the reference's last slot: no SubIdx(1).
        assert_eq!(
            valid_edge_field_moves(
                GraphSource::InputLeaf {
                    ref_idx: 1,
                    sub_idx: 7
                },
                0,
                &wide_refs
            ),
            vec![
                EdgeFieldMove::RefIdx(-1),
                EdgeFieldMove::RefIdx(1),
                EdgeFieldMove::SubIdx(-1)
            ]
        );
        // First ref_idx: no RefIdx(-1); last ref_idx: no RefIdx(1).
        assert_eq!(
            valid_edge_field_moves(
                GraphSource::InputLeaf {
                    ref_idx: 0,
                    sub_idx: 3
                },
                0,
                &wide_refs
            ),
            vec![
                EdgeFieldMove::RefIdx(1),
                EdgeFieldMove::SubIdx(-1),
                EdgeFieldMove::SubIdx(1)
            ]
        );
        assert_eq!(
            valid_edge_field_moves(
                GraphSource::InputLeaf {
                    ref_idx: 2,
                    sub_idx: 3
                },
                0,
                &wide_refs
            ),
            vec![
                EdgeFieldMove::RefIdx(-1),
                EdgeFieldMove::SubIdx(-1),
                EdgeFieldMove::SubIdx(1)
            ]
        );
        // A candidate neighbor too narrow to hold the current sub_idx
        // excludes the RefIdx move toward it, even though the neighbor
        // exists.
        let mixed_refs = vec![
            InputReference::World(WorldInputKey::food_here(OrdinaryFoodTypeId::default())), // width 1
            InputReference::World(WorldInputKey::NeighborBarrierRing), // width 8
            InputReference::World(WorldInputKey::food_here(OrdinaryFoodTypeId::default())), // width 1
        ];
        assert_eq!(
            valid_edge_field_moves(
                GraphSource::InputLeaf {
                    ref_idx: 1,
                    sub_idx: 5
                },
                0,
                &mixed_refs
            ),
            vec![EdgeFieldMove::SubIdx(-1), EdgeFieldMove::SubIdx(1)],
            "neither width-1 neighbor can hold sub_idx 5"
        );
        // Dangling ref_idx out of range: no moves at all.
        assert_eq!(
            valid_edge_field_moves(
                GraphSource::InputLeaf {
                    ref_idx: 5,
                    sub_idx: 0
                },
                0,
                &mixed_refs
            ),
            vec![]
        );
        // Exact boundary: sub_idx equals a narrower neighbor's width on both
        // sides at once (sub_idx == 1 == the width-1 neighbors' width).
        // `sub_idx < width` must reject this, not `sub_idx <= width`.
        assert_eq!(
            valid_edge_field_moves(
                GraphSource::InputLeaf {
                    ref_idx: 1,
                    sub_idx: 1
                },
                0,
                &mixed_refs
            ),
            vec![EdgeFieldMove::SubIdx(-1), EdgeFieldMove::SubIdx(1)],
            "sub_idx == width exactly must exclude both RefIdx moves"
        );
    }

    /// `SharedMemory` always offers both slot directions (they wrap modulo
    /// 16) plus the previous-tick flip, in that fixed order.
    #[test]
    fn valid_edge_field_moves_shared_memory_always_offers_three_moves() {
        assert_eq!(
            valid_edge_field_moves(
                GraphSource::SharedMemory {
                    slot: 5,
                    previous: false
                },
                0,
                &[]
            ),
            vec![
                EdgeFieldMove::SharedSlot(-1),
                EdgeFieldMove::SharedSlot(1),
                EdgeFieldMove::FlipPrevious,
            ]
        );
    }

    /// `apply_edge_field_move` must move each numeric field by exactly the
    /// signed delta, not its negation (kills both `+` → `-` sign-flip
    /// mutants: a flipped sign on `ComputeIdx(1)` from index 3 would produce
    /// 2, not 4; on `SharedSlot(-1)` from slot 0 it would produce 1 before
    /// wrapping, not 15).
    #[test]
    fn apply_edge_field_move_moves_by_the_exact_signed_delta() {
        let mut source = GraphSource::ComputeNode(3);
        apply_edge_field_move(&mut source, EdgeFieldMove::ComputeIdx(1));
        assert_eq!(source, GraphSource::ComputeNode(4));
        apply_edge_field_move(&mut source, EdgeFieldMove::ComputeIdx(-1));
        assert_eq!(source, GraphSource::ComputeNode(3));

        let mut source = GraphSource::SharedMemory {
            slot: 5,
            previous: false,
        };
        apply_edge_field_move(&mut source, EdgeFieldMove::SharedSlot(1));
        assert_eq!(
            source,
            GraphSource::SharedMemory {
                slot: 6,
                previous: false
            }
        );
        apply_edge_field_move(&mut source, EdgeFieldMove::SharedSlot(-1));
        assert_eq!(
            source,
            GraphSource::SharedMemory {
                slot: 5,
                previous: false
            }
        );

        let mut at_zero = GraphSource::SharedMemory {
            slot: 0,
            previous: false,
        };
        apply_edge_field_move(&mut at_zero, EdgeFieldMove::SharedSlot(-1));
        assert_eq!(
            at_zero,
            GraphSource::SharedMemory {
                slot: 15,
                previous: false
            }
        );
    }

    /// `has_raw_field_site` is a disjunction: either arm alone makes a def a
    /// raw-field target. Each case below satisfies exactly one arm, so a def
    /// that only has a parameterized compute node and a def that only has a
    /// movable edge must both be accepted, and `raw_field_mutation` must then
    /// apply without skipping.
    #[test]
    fn has_raw_field_site_accepts_either_arm_alone() {
        let input_refs = sample_input_refs();

        // Arm 1 only: a parameterized compute node, and no edge anywhere.
        let mut params_only = CgpGraphBackendDef {
            birth_weights: None,
            compute_nodes: vec![ComputeNode {
                kind: ComputeNodeKind::Constant(0.5),
                inputs: Vec::new(),
                plasticity: None,
            }],
            output_sinks: Vec::new(),
        };
        assert_eq!(total_edge_count(&params_only), 0);
        assert!(has_parameterized_compute_node(&params_only));
        assert!(
            has_raw_field_site(&params_only, &input_refs),
            "a parameterized compute node alone is a raw-field site"
        );
        assert!(raw_field_mutation(&mut params_only, &input_refs, &mut test_rng()).is_ok());

        // Arm 2 only: no parameterized compute node, but a movable edge.
        let mut edges_only = CgpGraphBackendDef {
            birth_weights: None,
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
                    kind: ComputeNodeKind::Add,
                    inputs: Vec::new(),
                    plasticity: None,
                },
            ],
            output_sinks: Vec::new(),
        };
        assert!(!has_parameterized_compute_node(&edges_only));
        assert!(raw_field_edge_sites(&edges_only, &input_refs)
            .next()
            .is_some());
        assert!(
            has_raw_field_site(&edges_only, &input_refs),
            "a movable edge alone is a raw-field site"
        );
        assert!(raw_field_mutation(&mut edges_only, &input_refs, &mut test_rng()).is_ok());
    }

    // ── The edge-surface draw (T19.F04) ─────────────────────────────────────

    fn founder_graph_def() -> CgpGraphBackendDef {
        crate::creature::cgp_founder::build_cgp_founder_graph_with_thresholds(0.5, 100, 500)
    }

    /// The draw is uniform over every compute node, then every catalog sink
    /// in order, the vote and parameter sinks included: one `gen_range` over
    /// the concatenated list.
    #[test]
    fn pick_random_surface_draws_uniformly_over_compute_nodes_and_every_sink() {
        let def = founder_graph_def();
        let reference: Vec<EdgeSurface> = (0..def.compute_nodes.len())
            .map(EdgeSurface::ComputeInput)
            .chain((0..def.output_sinks.len()).map(EdgeSurface::SinkInput))
            .collect();
        let mut drew_vote_surface = false;
        for seed in 0u64..512 {
            let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
            let surface = pick_random_surface(&def, &mut rng).expect("founder graph has surfaces");
            let mut replay = rand::rngs::SmallRng::seed_from_u64(seed);
            assert_eq!(surface, reference[replay.gen_range(0..reference.len())]);
            if let EdgeSurface::SinkInput(i) = surface {
                drew_vote_surface |= matches!(
                    def.output_sinks[i].kind,
                    OutputSinkKind::ActionVote(_) | OutputSinkKind::ActionParam(_, _)
                );
            }
        }
        assert!(drew_vote_surface);
        assert!(pick_random_surface(
            &CgpGraphBackendDef {
                birth_weights: None,
                compute_nodes: Vec::new(),
                output_sinks: Vec::new(),
            },
            &mut test_rng()
        )
        .is_none());
    }
}
