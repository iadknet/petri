//! The graph genome's edge container (T25.F01).
//!
//! The only code that edits a graph's compute nodes and edges, and the only
//! code that keeps the birth payload ([`BirthSeeds`]) aligned with them. The
//! compute-node list, the sink list and both `inputs` lists are newtypes that
//! read as slices and cannot be written from outside this module: every edit
//! goes through a primitive below, which applies the seed rule of the write
//! it replaces. The container draws no random numbers.

use std::fmt;
use std::ops::Deref;

use super::edge::{GraphEdge, GraphSource};
use super::{
    ComputeNodeKind, DirectedVoteKind, OutputSinkKind, RingProjection, CUSTOM_OUTPUT_COUNT,
    FIXED_SINK_COUNT, SHARED_MEMORY_SLOTS,
};
use crate::contracts::MAX_GATE_SLOTS;
use crate::creature::genome::vote::{ActionParamField, VoteSink};
use crate::creature::genome::PlasticityConfig;

// ── Read-only list newtypes ─────────────────────────────────────────────────

macro_rules! read_only_list {
    ($(#[$meta:meta])* $name:ident, $item:ty) => {
        $(#[$meta])*
        #[derive(Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
        #[serde(transparent)]
        pub struct $name(Vec<$item>);

        impl Deref for $name {
            type Target = [$item];

            #[inline]
            fn deref(&self) -> &[$item] {
                &self.0
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        impl<'a> IntoIterator for &'a $name {
            type Item = &'a $item;
            type IntoIter = std::slice::Iter<'a, $item>;

            #[inline]
            fn into_iter(self) -> Self::IntoIter {
                self.0.iter()
            }
        }

        // Compares with plain lists the way the `Vec` it replaced did.
        impl PartialEq<Vec<$item>> for $name {
            fn eq(&self, other: &Vec<$item>) -> bool {
                self.0 == *other
            }
        }

        impl PartialEq<[$item]> for $name {
            fn eq(&self, other: &[$item]) -> bool {
                self.0 == other
            }
        }
    };
}

read_only_list!(
    /// The ordered edges a compute node or sink reads. Order is semantic.
    EdgeList,
    GraphEdge
);
read_only_list!(
    /// A graph's compute nodes, in evaluation order.
    ComputeNodes,
    ComputeNode
);
read_only_list!(
    /// A graph's output sinks, in catalog order.
    OutputSinks,
    OutputSink
);

// ── Node and sink ───────────────────────────────────────────────────────────

/// A single compute node in the graph backend.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ComputeNode {
    pub kind: ComputeNodeKind,
    pub inputs: EdgeList,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plasticity: Option<PlasticityConfig>,
}

impl ComputeNode {
    #[must_use]
    pub fn new(
        kind: ComputeNodeKind,
        inputs: Vec<GraphEdge>,
        plasticity: Option<PlasticityConfig>,
    ) -> Self {
        Self {
            kind,
            inputs: EdgeList(inputs),
            plasticity,
        }
    }
}

/// Fixed structural output — one per target slot.
/// Kind is NOT mutated; only inputs (edges) change via mutation.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OutputSink {
    pub kind: OutputSinkKind,
    pub inputs: EdgeList,
}

impl OutputSink {
    #[must_use]
    pub fn new(kind: OutputSinkKind, inputs: Vec<GraphEdge>) -> Self {
        Self {
            kind,
            inputs: EdgeList(inputs),
        }
    }
}

// ── Edge occurrences ────────────────────────────────────────────────────────

/// Which edge-bearing surface an edge belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeSurface {
    ComputeInput(usize),
    SinkInput(usize),
}

/// One edge: its surface and its index within that surface's inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EdgeOccurrence {
    pub surface: EdgeSurface,
    pub index: usize,
}

impl EdgeOccurrence {
    #[inline]
    #[must_use]
    pub const fn new(surface: EdgeSurface, index: usize) -> Self {
        Self { surface, index }
    }
}

// ── Birth seeds ─────────────────────────────────────────────────────────────

/// The birth payload a reproduction draft carries (T25.F01, design note D1):
/// the parent's learned value per compute-input edge, captured before
/// mutation and consumed by the newborn's constructor. Outside genome
/// identity, serialization and equality.
#[derive(Debug, Clone, PartialEq)]
pub enum BirthSeeds {
    /// A draft whose parent had nothing learned: no rows, no allocation.
    Empty,
    /// One row per compute node, each sized to that node's `inputs`; `None`
    /// where the edge has no inherited value.
    Rows(Vec<Vec<Option<f32>>>),
}

impl BirthSeeds {
    /// The rows, `None` for the empty form.
    #[inline]
    #[must_use]
    pub fn rows(&self) -> Option<&[Vec<Option<f32>>]> {
        match self {
            Self::Empty => None,
            Self::Rows(rows) => Some(rows),
        }
    }
}

// ── The definition ──────────────────────────────────────────────────────────

/// CGP-style graph backend definition.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct CgpGraphBackendDef {
    /// Birth-only payload; see [`BirthSeeds`].
    #[serde(skip)]
    birth_seeds: Option<BirthSeeds>,
    compute_nodes: ComputeNodes,
    /// Fixed set — structurally immutable. Only edges are evolvable.
    output_sinks: OutputSinks,
    /// Ring projections (T11.F31), in genome order; omitted when empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub projections: Vec<RingProjection>,
}

// Keep the birth payload out of diagnostic genome identity as well as
// serialization and equality. Existing state fingerprints use this format.
impl fmt::Debug for CgpGraphBackendDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = f.debug_struct("CgpGraphBackendDef");
        debug
            .field("compute_nodes", &self.compute_nodes)
            .field("output_sinks", &self.output_sinks);
        // Omitted when empty, so a genome without projections keeps the
        // fingerprint it had before T11.F31.
        if !self.projections.is_empty() {
            debug.field("projections", &self.projections);
        }
        debug.finish()
    }
}

impl PartialEq for CgpGraphBackendDef {
    fn eq(&self, other: &Self) -> bool {
        self.compute_nodes == other.compute_nodes
            && self.output_sinks == other.output_sinks
            && self.projections == other.projections
    }
}

/// A handle on one edge through which its source and weight change with the
/// seed rule of each write.
pub struct EdgeMut<'a> {
    def: &'a mut CgpGraphBackendDef,
    occurrence: EdgeOccurrence,
}

impl EdgeMut<'_> {
    /// The edge as it stands.
    #[must_use]
    pub fn edge(&self) -> GraphEdge {
        *self.def.edge_at(self.occurrence)
    }

    /// Point the edge at `source`; its seed is cleared only when the source
    /// changes.
    pub fn set_source(&mut self, source: GraphSource) {
        let edge = self.def.edge_slot(self.occurrence);
        let changed = edge.source != source;
        edge.source = source;
        if changed {
            self.def.clear_seed(self.occurrence);
        }
        self.def.debug_assert_seed_shape();
    }

    /// Set the edge's weight; its seed is cleared.
    pub fn set_weight(&mut self, weight: f32) {
        self.def.clear_seed(self.occurrence);
        self.def.edge_slot(self.occurrence).weight = weight;
        self.def.debug_assert_seed_shape();
    }
}

impl CgpGraphBackendDef {
    // ── Construction ────────────────────────────────────────────────────────

    /// Construct a new graph backend with the full fixed output catalog.
    /// All sinks start with empty edges (inert).
    #[must_use]
    pub fn new_with_fixed_outputs() -> Self {
        let mut output_sinks = Vec::with_capacity(FIXED_SINK_COUNT);
        let mut push = |kind| output_sinks.push(OutputSink::new(kind, Vec::new()));
        for slot in 0..CUSTOM_OUTPUT_COUNT {
            push(OutputSinkKind::CustomOutput(slot));
        }
        for slot in 0..MAX_GATE_SLOTS as u8 {
            push(OutputSinkKind::RouterGate(slot));
        }
        for slot in 0..SHARED_MEMORY_SLOTS {
            push(OutputSinkKind::WriteSlot(slot));
        }
        for slot in 0..SHARED_MEMORY_SLOTS {
            push(OutputSinkKind::ClearSlot(slot));
        }
        // ActionVote sinks in `VoteSink` index order (T19.F03).
        for sink in VoteSink::all() {
            push(OutputSinkKind::ActionVote(sink));
        }
        // ActionParam sinks in `ActionParamField::ALL` order (T11.F27, T02.F08).
        for field in ActionParamField::ALL {
            push(OutputSinkKind::ActionParam(field));
        }
        Self {
            birth_seeds: None,
            compute_nodes: ComputeNodes::default(),
            output_sinks: OutputSinks(output_sinks),
            projections: Vec::new(),
        }
    }

    /// Replace the compute nodes with `compute_nodes`, exactly as given. A
    /// seedless builder: any birth payload is dropped.
    #[must_use = "the builder returns the modified def"]
    pub fn with_compute_nodes(mut self, compute_nodes: Vec<ComputeNode>) -> Self {
        self.birth_seeds = None;
        self.compute_nodes = ComputeNodes(compute_nodes);
        self
    }

    /// Replace the sink catalog with `output_sinks`, exactly as given: no
    /// filling, sorting, deduplication or kind validation.
    #[must_use = "the builder returns the modified def"]
    pub fn with_output_sinks(mut self, output_sinks: Vec<OutputSink>) -> Self {
        self.output_sinks = OutputSinks(output_sinks);
        self
    }

    /// Replace the ring projections with `projections`, exactly as given.
    #[must_use = "the builder returns the modified def"]
    pub fn with_projections(mut self, projections: Vec<RingProjection>) -> Self {
        self.projections = projections;
        self
    }

    // ── Reads ───────────────────────────────────────────────────────────────

    /// The compute nodes, in evaluation order.
    #[inline]
    #[must_use]
    pub fn compute_nodes(&self) -> &ComputeNodes {
        &self.compute_nodes
    }

    /// The output sinks, in catalog order.
    #[inline]
    #[must_use]
    pub fn output_sinks(&self) -> &OutputSinks {
        &self.output_sinks
    }

    /// The output sink of `kind`, when the graph carries it.
    #[must_use]
    pub fn sink(&self, kind: OutputSinkKind) -> Option<&OutputSink> {
        self.output_sinks.iter().find(|sink| sink.kind == kind)
    }

    /// The surface of the first sink of `kind`, when the graph carries it.
    #[must_use]
    pub fn sink_surface(&self, kind: OutputSinkKind) -> Option<EdgeSurface> {
        self.output_sinks
            .iter()
            .position(|sink| sink.kind == kind)
            .map(EdgeSurface::SinkInput)
    }

    /// Every edge, in the canonical order of [`Self::edge_sites`].
    pub fn edges(&self) -> impl Iterator<Item = &GraphEdge> {
        self.compute_nodes
            .iter()
            .flat_map(|node| node.inputs.iter())
            .chain(self.output_sinks.iter().flat_map(|sink| sink.inputs.iter()))
    }

    /// Every edge occurrence in canonical order: compute inputs in node
    /// order, then sink inputs in catalog order. Random edge draws index
    /// this order, so it is part of every mutation draw stream.
    pub fn edge_sites(&self) -> impl Iterator<Item = EdgeOccurrence> + '_ {
        let compute = self.compute_nodes.iter().enumerate().flat_map(|(i, node)| {
            (0..node.inputs.len())
                .map(move |index| EdgeOccurrence::new(EdgeSurface::ComputeInput(i), index))
        });
        let sinks = self.output_sinks.iter().enumerate().flat_map(|(i, sink)| {
            (0..sink.inputs.len())
                .map(move |index| EdgeOccurrence::new(EdgeSurface::SinkInput(i), index))
        });
        compute.chain(sinks)
    }

    /// The edges of `surface`.
    #[must_use]
    pub fn surface_edges(&self, surface: EdgeSurface) -> &EdgeList {
        match surface {
            EdgeSurface::ComputeInput(i) => &self.compute_nodes[i].inputs,
            EdgeSurface::SinkInput(i) => &self.output_sinks[i].inputs,
        }
    }

    /// The edge at `occurrence`.
    #[must_use]
    pub fn edge_at(&self, occurrence: EdgeOccurrence) -> &GraphEdge {
        &self.surface_edges(occurrence.surface)[occurrence.index]
    }

    /// The birth payload, when this def belongs to a reproduction draft.
    #[inline]
    #[must_use]
    pub fn birth_seeds(&self) -> Option<&BirthSeeds> {
        self.birth_seeds.as_ref()
    }

    /// Whether a visit to this graph enters evaluation and the effects pass.
    ///
    /// True with at least one compute node, and true for a zero-compute graph
    /// with a wired output sink or a ring projection (T11.F31). A graph with
    /// none of these computes nothing and applies nothing, so its visit is
    /// free. Derived from the genome on every visit and stored nowhere.
    #[must_use]
    pub fn enters_visit(&self) -> bool {
        !self.compute_nodes.is_empty()
            || !self.projections.is_empty()
            || self.output_sinks.iter().any(|sink| !sink.inputs.is_empty())
    }

    /// Whether any projection votes for `kind`.
    #[must_use]
    pub fn projects_kind(&self, kind: DirectedVoteKind) -> bool {
        self.projections.iter().any(|p| p.kind == kind)
    }

    // ── Seeds ───────────────────────────────────────────────────────────────

    /// Attach a reproduction draft's payload. Rows must match the compute
    /// nodes and their inputs (I11).
    pub fn seed_birth(&mut self, seeds: BirthSeeds) {
        self.birth_seeds = Some(seeds);
        self.debug_assert_seed_shape();
    }

    /// Remove and return the payload, leaving the def seedless.
    pub fn take_birth_seeds(&mut self) -> Option<BirthSeeds> {
        self.birth_seeds.take()
    }

    // ── Compute-node primitives ─────────────────────────────────────────────

    /// Append `node` without repairing indices: existing dangling sources
    /// keep their behavior when their index becomes addressable. Its seed row
    /// is empty of values.
    pub fn append_compute_node(&mut self, node: ComputeNode) {
        if let Some(rows) = self.seed_rows_mut() {
            rows.push(vec![None; node.inputs.len()]);
        }
        self.compute_nodes.0.push(node);
        self.debug_assert_seed_shape();
    }

    /// Remove the compute node at `idx`. Remaps `GraphSource::ComputeNode`
    /// indices across both edge containers: edges pointing to the removed
    /// node get `ComputeNode(u16::MAX)` and lose their seed; edges pointing
    /// above it are decremented.
    pub fn remove_compute_node(&mut self, idx: usize) {
        debug_assert!(
            idx < self.compute_nodes.len(),
            "remove_compute_node: idx {} out of bounds (len {})",
            idx,
            self.compute_nodes.len()
        );
        self.compute_nodes.0.remove(idx);
        if let Some(BirthSeeds::Rows(rows)) = &mut self.birth_seeds {
            rows.remove(idx);
            for (node, values) in self.compute_nodes.iter().zip(rows) {
                for (edge, inherited) in node.inputs.iter().zip(values) {
                    if edge.source == GraphSource::ComputeNode(idx as u16) {
                        *inherited = None;
                    }
                }
            }
        }
        let removed = idx as u16;
        self.remap_compute_sources(|src_idx| {
            if src_idx == removed {
                u16::MAX
            } else if src_idx > removed {
                src_idx - 1
            } else {
                src_idx
            }
        });
        self.debug_assert_seed_shape();
    }

    /// Insert `node` at compute index `idx`, the inverse of
    /// [`Self::remove_compute_node`]. Every `GraphSource::ComputeNode(i)`
    /// reference at or above `idx`, across both edge-bearing surfaces
    /// (including edges belonging to `node` itself, resolved by the caller
    /// before insertion), is incremented by one so every surviving edge keeps
    /// pointing at the same logical node after the shift. The new row holds
    /// no values.
    pub fn insert_compute_node(&mut self, idx: usize, node: ComputeNode) {
        debug_assert!(
            idx <= self.compute_nodes.len(),
            "insert_compute_node: idx {} out of bounds (len {})",
            idx,
            self.compute_nodes.len()
        );
        let at = idx as u16;
        self.remap_compute_sources(|src_idx| {
            // `u16::MAX` is `remove_compute_node`'s dangling-reference
            // sentinel, never a real index; leave it alone rather than
            // overflow.
            if src_idx != u16::MAX && src_idx >= at {
                src_idx + 1
            } else {
                src_idx
            }
        });
        if let Some(rows) = self.seed_rows_mut() {
            rows.insert(idx, vec![None; node.inputs.len()]);
        }
        self.compute_nodes.0.insert(idx, node);
        self.debug_assert_seed_shape();
    }

    /// Duplicate the compute nodes at `sources`, inserting each copy directly
    /// after its original so the copy reads every source in the same
    /// evaluation phase its original reads: a source below the original stays
    /// below the copy and is read from this visit, a source at or above it
    /// stays above and is read from the last committed outputs
    /// (`runtime/cgp/sources.rs`, T11.F06). The `i`-th source ends at index
    /// `sources[i] + i` and its copy at `sources[i] + i + 1`; each copy's
    /// seed row is its original's.
    ///
    /// An edge from one duplicated node to another, including a self-edge,
    /// points at the corresponding copy, so the duplicated set's internal
    /// wiring and per-node state are its own. Every other reference, in the
    /// copies and across both edge-bearing surfaces, is remapped to its
    /// shifted index, so no surviving edge changes what it reads.
    ///
    /// `sources` must be strictly ascending and in range; callers must keep
    /// `compute_nodes().len() + sources.len()` within `u16::MAX`.
    pub fn duplicate_compute_nodes(&mut self, sources: &[usize]) {
        let old_len = self.compute_nodes.len();
        debug_assert!(
            sources.windows(2).all(|pair| pair[0] < pair[1])
                && sources.last().is_none_or(|&last| last < old_len),
            "duplicate_compute_nodes: sources must be strictly ascending and in range"
        );
        let originals: Vec<ComputeNode> = sources
            .iter()
            .map(|&idx| self.compute_nodes[idx].clone())
            .collect();
        // A surviving reference moves up by the number of duplicated nodes
        // below it. Out-of-range indices, including `remove_compute_node`'s
        // `u16::MAX` sentinel, are left alone.
        let shift = |src_idx: u16| {
            if (src_idx as usize) >= old_len {
                src_idx
            } else {
                src_idx + sources.partition_point(|&c| c < src_idx as usize) as u16
            }
        };
        if let Some(rows) = self.seed_rows_mut() {
            for &source in sources.iter().rev() {
                rows.insert(source + 1, rows[source].clone());
            }
        }
        self.remap_compute_sources(shift);
        for (i, mut copy) in originals.into_iter().enumerate().rev() {
            for edge in &mut copy.inputs.0 {
                if let GraphSource::ComputeNode(ref mut src_idx) = edge.source {
                    *src_idx = match sources.binary_search(&(*src_idx as usize)) {
                        Ok(position) => (sources[position] + position + 1) as u16,
                        Err(_) => shift(*src_idx),
                    };
                }
            }
            self.compute_nodes.0.insert(sources[i] + 1, copy);
        }
        self.debug_assert_seed_shape();
    }

    /// Set the kind of compute node `idx`. No seed change.
    pub fn set_compute_kind(&mut self, idx: usize, kind: ComputeNodeKind) {
        self.compute_nodes.0[idx].kind = kind;
    }

    /// Set or clear the plasticity of compute node `idx`. No seed change.
    pub fn set_node_plasticity(&mut self, idx: usize, plasticity: Option<PlasticityConfig>) {
        self.compute_nodes.0[idx].plasticity = plasticity;
    }

    /// The plasticity of compute node `idx`, mutably. No seed change.
    pub fn node_plasticity_mut(&mut self, idx: usize) -> Option<&mut PlasticityConfig> {
        self.compute_nodes.0[idx].plasticity.as_mut()
    }

    // ── Edge primitives ─────────────────────────────────────────────────────

    /// Append `edge` to `surface`; on a compute surface its seed is empty.
    pub fn push_edge(&mut self, surface: EdgeSurface, edge: GraphEdge) {
        match surface {
            EdgeSurface::ComputeInput(i) => {
                self.compute_nodes.0[i].inputs.0.push(edge);
                if let Some(rows) = self.seed_rows_mut() {
                    rows[i].push(None);
                }
            }
            EdgeSurface::SinkInput(i) => self.output_sinks.0[i].inputs.0.push(edge),
        }
        self.debug_assert_seed_shape();
    }

    /// Remove the edge at `occurrence` with its seed.
    pub fn remove_edge(&mut self, occurrence: EdgeOccurrence) {
        let EdgeOccurrence { surface, index } = occurrence;
        match surface {
            EdgeSurface::ComputeInput(i) => {
                self.compute_nodes.0[i].inputs.0.remove(index);
                if let Some(rows) = self.seed_rows_mut() {
                    rows[i].remove(index);
                }
            }
            EdgeSurface::SinkInput(i) => {
                self.output_sinks.0[i].inputs.0.remove(index);
            }
        }
        self.debug_assert_seed_shape();
    }

    /// Append copies of compute node `from`'s edges, with their seeds, to
    /// compute node `to`.
    pub fn copy_edges(&mut self, from: usize, to: usize) {
        let copied = self.compute_nodes[from].inputs.0.clone();
        self.compute_nodes.0[to].inputs.0.extend(copied);
        if let Some(rows) = self.seed_rows_mut() {
            let copied = rows[from].clone();
            rows[to].extend(copied);
        }
        self.debug_assert_seed_shape();
    }

    /// Split the edge at `occurrence` with `identity`, an identity node with
    /// no inputs: the identity node reads the edge's source at weight 1.0 and
    /// the edge reads the identity node, keeping its weight and seed. A
    /// compute consumer gets the identity node inserted at its own index
    /// ([`Self::insert_compute_node`]), so evaluation order is preserved; a
    /// sink consumer gets it appended. The identity node's edge has no seed.
    pub fn split_edge(&mut self, occurrence: EdgeOccurrence, mut identity: ComputeNode) {
        debug_assert!(
            identity.inputs.is_empty(),
            "split_edge: the identity node arrives without inputs"
        );
        let EdgeOccurrence { surface, index } = occurrence;
        match surface {
            EdgeSurface::ComputeInput(consumer) => {
                // Insert first so the index remap (which also shifts the
                // consumer's source in place when it targets a node at or
                // above `consumer`, a self-loop included) runs before the
                // identity node's input is read back.
                self.insert_compute_node(consumer, identity);
                let shifted = consumer + 1;
                let source = self.compute_nodes[shifted].inputs[index].source;
                self.push_edge(
                    EdgeSurface::ComputeInput(consumer),
                    GraphEdge::new(source, 1.0),
                );
                self.compute_nodes.0[shifted].inputs.0[index].source =
                    GraphSource::ComputeNode(consumer as u16);
            }
            EdgeSurface::SinkInput(sink) => {
                let new_idx = self.compute_nodes.len() as u16;
                let source = self.output_sinks[sink].inputs[index].source;
                identity.inputs.0.push(GraphEdge::new(source, 1.0));
                self.append_compute_node(identity);
                self.output_sinks.0[sink].inputs.0[index].source =
                    GraphSource::ComputeNode(new_idx);
            }
        }
        self.debug_assert_seed_shape();
    }

    /// A handle for changing the source or weight of the edge at `occurrence`.
    pub fn edge_mut(&mut self, occurrence: EdgeOccurrence) -> EdgeMut<'_> {
        EdgeMut {
            def: self,
            occurrence,
        }
    }

    // ── Input-reference primitives ──────────────────────────────────────────

    /// After the input reference at `removed_ref_idx` is pruned, decrement
    /// every higher `InputLeaf.ref_idx` and projection `ref_idx`. `Prune`
    /// removes only an index nothing addresses (`prunable_indices`), so no
    /// edge or projection is removed and the seeds are untouched.
    pub fn reindex_input_refs_after_removal(&mut self, removed_ref_idx: u16) {
        for projection in &mut self.projections {
            debug_assert_ne!(
                projection.ref_idx, removed_ref_idx,
                "a pruned input reference is addressed by no projection"
            );
            if projection.ref_idx > removed_ref_idx {
                projection.ref_idx -= 1;
            }
        }
        self.for_each_edge_mut(|edge| {
            if let GraphSource::InputLeaf { ref_idx, .. } = &mut edge.source {
                debug_assert_ne!(
                    *ref_idx, removed_ref_idx,
                    "a pruned input reference is addressed by no edge"
                );
                if *ref_idx > removed_ref_idx {
                    *ref_idx -= 1;
                }
            }
        });
    }

    /// Point input leaves elsewhere on an observation copy: every
    /// `InputLeaf { ref_idx, sub_idx }` for which `redirect` returns
    /// `Some(new)` reads `input_refs[new]` instead. No seed change and no
    /// check; such a copy never re-enters mutation or validation.
    pub fn redirect_leaf_reads(&mut self, mut redirect: impl FnMut(u16, u16) -> Option<u16>) {
        self.for_each_edge_mut(|edge| {
            if let GraphSource::InputLeaf { ref_idx, sub_idx } = &mut edge.source {
                if let Some(new) = redirect(*ref_idx, *sub_idx) {
                    *ref_idx = new;
                }
            }
        });
    }

    // ── Internal helpers ────────────────────────────────────────────────────

    fn seed_rows_mut(&mut self) -> Option<&mut Vec<Vec<Option<f32>>>> {
        match &mut self.birth_seeds {
            Some(BirthSeeds::Rows(rows)) => Some(rows),
            _ => None,
        }
    }

    fn clear_seed(&mut self, occurrence: EdgeOccurrence) {
        if let (EdgeSurface::ComputeInput(node), Some(rows)) =
            (occurrence.surface, self.seed_rows_mut())
        {
            rows[node][occurrence.index] = None;
        }
    }

    fn edge_slot(&mut self, occurrence: EdgeOccurrence) -> &mut GraphEdge {
        match occurrence.surface {
            EdgeSurface::ComputeInput(i) => &mut self.compute_nodes.0[i].inputs.0[occurrence.index],
            EdgeSurface::SinkInput(i) => &mut self.output_sinks.0[i].inputs.0[occurrence.index],
        }
    }

    /// I11: when rows are present there is one per compute node, each sized
    /// to that node's inputs.
    fn debug_assert_seed_shape(&self) {
        if let Some(BirthSeeds::Rows(rows)) = &self.birth_seeds {
            debug_assert_eq!(
                rows.len(),
                self.compute_nodes.len(),
                "one seed row per compute node"
            );
            debug_assert!(
                rows.iter()
                    .zip(self.compute_nodes.iter())
                    .all(|(row, node)| row.len() == node.inputs.len()),
                "each seed row sized to its node's inputs"
            );
        }
    }

    /// Apply a remapping function to all `GraphSource::ComputeNode(idx)` edges
    /// across every edge container.
    fn remap_compute_sources(&mut self, remap: impl Fn(u16) -> u16) {
        self.for_each_edge_mut(|edge| {
            if let GraphSource::ComputeNode(idx) = &mut edge.source {
                *idx = remap(*idx);
            }
        });
    }

    /// Apply a closure to every `GraphEdge` across all containers.
    fn for_each_edge_mut(&mut self, mut f: impl FnMut(&mut GraphEdge)) {
        for node in &mut self.compute_nodes.0 {
            node.inputs.0.iter_mut().for_each(&mut f);
        }
        for sink in &mut self.output_sinks.0 {
            sink.inputs.0.iter_mut().for_each(&mut f);
        }
    }
}

#[cfg(test)]
#[path = "container_tests.rs"]
mod tests;
