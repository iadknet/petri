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

use super::edge::Membership;
use super::edge::{GraphEdge, GraphSource};
use super::group::{compose, pair_birth_members, Group, GroupId, Shape};
use super::{
    ComputeNodeKind, OutputSinkKind, CUSTOM_OUTPUT_COUNT, FIXED_SINK_COUNT, SHARED_MEMORY_SLOTS,
};
use crate::contracts::MAX_GATE_SLOTS;
use crate::contracts::{DirectionPair, InputReference};
use crate::creature::genome::vote::{
    ActionParamField, DirectedVoteKind as Kind, VoteSink, VOTE_DIRECTION_COUNT,
};
use crate::creature::genome::{BackendDef, NodeGenome, PlasticityConfig};

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
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
pub struct CgpGraphBackendDef {
    /// Birth-only payload; see [`BirthSeeds`].
    #[serde(skip)]
    birth_seeds: Option<BirthSeeds>,
    compute_nodes: ComputeNodes,
    /// Fixed set — structurally immutable. Only edges are evolvable.
    output_sinks: OutputSinks,
    /// Edge-group records (T25.F03), indexed by `GroupId`; omitted from JSON
    /// when empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    groups: Vec<Group>,
}

// Keep the birth payload out of diagnostic genome identity as well as
// serialization and equality. Existing state fingerprints use this format.
impl fmt::Debug for CgpGraphBackendDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = f.debug_struct("CgpGraphBackendDef");
        debug
            .field("compute_nodes", &self.compute_nodes)
            .field("output_sinks", &self.output_sinks);
        debug.field("groups", &self.groups).finish()
    }
}

impl PartialEq for CgpGraphBackendDef {
    fn eq(&self, other: &Self) -> bool {
        self.compute_nodes == other.compute_nodes
            && self.output_sinks == other.output_sinks
            && self.groups == other.groups
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
        debug_assert!(
            !changed || edge.membership.is_none(),
            "a member's source changes only through its group or after a detach"
        );
        edge.source = source;
        if changed {
            self.def.clear_seed(self.occurrence);
        }
        self.def.debug_assert_seed_shape();
    }

    /// Set the edge's weight; its seed is cleared.
    pub fn set_weight(&mut self, weight: f32) {
        self.def.clear_seed(self.occurrence);
        self.def.edge_slot(self.occurrence).own_weight = weight;
        self.def.debug_assert_seed_shape();
    }
}

/// A handle on one group's record.
pub struct GroupMut<'a> {
    def: &'a mut CgpGraphBackendDef,
    group: GroupId,
}

impl GroupMut<'_> {
    /// The record as it stands.
    #[must_use]
    pub fn group(&self) -> Group {
        self.def.groups[self.group.index()]
    }

    /// Set the group's shared scale.
    pub fn set_scale(&mut self, scale: f32) {
        self.def.groups[self.group.index()].scale = scale;
    }
}

/// The graph invariant a genome breaks (T25.F03, design note §3, I2–I10).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphInvariantViolation {
    /// I2: a compute-input edge carries a membership.
    MemberOnComputeInput,
    /// I3: a membership names no group.
    GroupIdOutOfRange,
    /// I3: more than `u16::MAX` groups.
    TooManyGroups,
    /// I4, I8: a member does not fit its group's shape.
    MemberShape,
    /// I5: two members of one group share a coordinate.
    DuplicateMember,
    /// I6: a group has no member.
    EmptyGroup,
    /// I7: a group's offset is not below 8.
    OffsetOutOfRange,
    /// I7: a group's shape reference is not a ring (`Ring`) or does not
    /// declare the named pair (`DirectionPair`).
    ShapeReference,
    /// I10: a sink's members are not after its independent edges in
    /// `(GroupId, sub_idx)` order.
    SinkOrder,
}

/// The direction a `Ring` member of `group` on `channel` votes toward:
/// `(offset + channel) mod 8`.
fn member_direction(group: &Group, channel: u16) -> u8 {
    ((u16::from(group.offset) + channel) % u16::from(VOTE_DIRECTION_COUNT)) as u8
}

/// Check every graph invariant (I2–I10) of `node`'s graph against its input
/// references: at `GenomeFile::load` and in the mutation gate. A VM node
/// passes.
///
/// # Errors
///
/// The first invariant the graph breaks.
pub fn validate_graph(node: &NodeGenome) -> Result<(), GraphInvariantViolation> {
    let BackendDef::Graph(def) = &node.backend_def else {
        return Ok(());
    };
    def.check_local_invariants()?;
    for group in &def.groups {
        let resolves = match group.shape {
            Shape::Ring { ring_ref } => node
                .input_refs
                .get(usize::from(ring_ref))
                .is_some_and(InputReference::is_ring),
            Shape::DirectionPair { .. } => group.shape.resolve_pair(&node.input_refs).is_some(),
        };
        if !resolves {
            return Err(GraphInvariantViolation::ShapeReference);
        }
    }
    // I4's pair role: every `DirectionPair` member reads its pair's `x` or
    // `y` field.
    for sink in def.output_sinks.iter() {
        for edge in sink.inputs.iter() {
            let Some(Membership { group }) = edge.membership else {
                continue;
            };
            let Some(pair) = def.groups[group.index()]
                .shape
                .resolve_pair(&node.input_refs)
            else {
                continue;
            };
            let GraphSource::InputLeaf { sub_idx, .. } = edge.source else {
                return Err(GraphInvariantViolation::MemberShape);
            };
            if sub_idx != pair.x && sub_idx != pair.y {
                return Err(GraphInvariantViolation::MemberShape);
            }
        }
    }
    Ok(())
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
            groups: Vec::new(),
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

    /// The edge-group records, indexed by `GroupId`.
    #[inline]
    #[must_use]
    pub fn groups(&self) -> &[Group] {
        &self.groups
    }

    /// The genomic weight of the edge at `occurrence`: its own weight when
    /// ungrouped, [`compose`]`(own_weight, scale, pattern)` for a member.
    #[must_use]
    pub fn genomic_weight(&self, occurrence: EdgeOccurrence, input_refs: &[InputReference]) -> f32 {
        let edge = self.edge_at(occurrence);
        match occurrence.surface {
            EdgeSurface::SinkInput(i) => {
                self.edge_weight(edge, self.output_sinks[i].kind, input_refs)
            }
            // A compute-input edge is never a member (I2).
            EdgeSurface::ComputeInput(_) => edge.own_weight,
        }
    }

    /// The genomic weight of `edge`, an edge of this graph on a sink of kind
    /// `sink`: its own weight when ungrouped, [`compose`]`(own_weight, scale,
    /// pattern)` for a member, the pattern taken at the member's field and
    /// the sink's direction (T25.F04: identical-source pair members on
    /// different sinks carry different patterns).
    #[must_use]
    pub fn edge_weight(
        &self,
        edge: &GraphEdge,
        sink: OutputSinkKind,
        input_refs: &[InputReference],
    ) -> f32 {
        match edge.membership {
            None => edge.own_weight,
            Some(membership) => compose(
                edge.own_weight,
                self.groups[membership.group.index()].scale,
                self.pattern_at(edge, sink, input_refs),
            ),
        }
    }

    /// The pattern of the member at `occurrence` (its group's at the
    /// member's field and sink direction).
    pub(super) fn member_pattern(
        &self,
        occurrence: EdgeOccurrence,
        input_refs: &[InputReference],
    ) -> f32 {
        let EdgeSurface::SinkInput(sink) = occurrence.surface else {
            panic!("member_pattern: only a sink edge is a member");
        };
        self.pattern_at(
            self.edge_at(occurrence),
            self.output_sinks[sink].kind,
            input_refs,
        )
    }

    /// The pattern of member `edge` on a sink of kind `sink`: its group's at
    /// the member's field and the sink's direction (`u16::MAX` / `u8::MAX`,
    /// pattern 0 for a pair group, when either is absent).
    fn pattern_at(
        &self,
        edge: &GraphEdge,
        sink: OutputSinkKind,
        input_refs: &[InputReference],
    ) -> f32 {
        let Membership { group } = edge.membership.expect("pattern_at: a member");
        let sub_idx = match edge.source {
            GraphSource::InputLeaf { sub_idx, .. } => sub_idx,
            _ => u16::MAX,
        };
        let direction = sink.direction().map_or(u8::MAX, |(_, d)| d);
        self.groups[group.index()].pattern(input_refs, sub_idx, direction)
    }

    /// One row per sink, one genomic weight per sink edge in order: the
    /// runtime weight table of this graph (T25.F03).
    #[must_use]
    pub fn sink_weight_rows(&self, input_refs: &[InputReference]) -> Vec<Box<[f32]>> {
        self.output_sinks
            .iter()
            .map(|entry| {
                entry
                    .inputs
                    .iter()
                    .map(|edge| self.edge_weight(edge, entry.kind, input_refs))
                    .collect()
            })
            .collect()
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
    /// with a wired output sink (a group's members wire their sinks). A graph
    /// with neither computes nothing and applies nothing, so its visit is
    /// free. Derived from the genome on every visit and stored nowhere.
    #[must_use]
    pub fn enters_visit(&self) -> bool {
        !self.compute_nodes.is_empty()
            || self.output_sinks.iter().any(|sink| !sink.inputs.is_empty())
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
            EdgeSurface::SinkInput(i) => {
                debug_assert!(
                    edge.membership.is_none(),
                    "push_edge adds an ungrouped edge"
                );
                let at = self.first_member(i);
                self.output_sinks.0[i].inputs.0.insert(at, edge);
            }
        }
        self.debug_assert_seed_shape();
        self.debug_assert_groups();
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
                let removed = self.output_sinks.0[i].inputs.0.remove(index);
                if let Some(Membership { group }) = removed.membership {
                    if self.members(group).next().is_none() {
                        self.drop_group_record(group);
                    }
                }
            }
        }
        self.debug_assert_seed_shape();
        self.debug_assert_groups();
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
                debug_assert!(
                    self.output_sinks[sink].inputs[index].membership.is_none(),
                    "split_edge: a member is detached before its split"
                );
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
    /// every higher `InputLeaf.ref_idx` and group shape reference. `Prune`
    /// removes only an index nothing addresses (`prunable_indices`), so no
    /// edge or group is removed and the seeds are untouched.
    pub fn reindex_input_refs_after_removal(&mut self, removed_ref_idx: u16) {
        for group in &mut self.groups {
            let (Shape::Ring {
                ring_ref: shape_ref,
            }
            | Shape::DirectionPair {
                ref_idx: shape_ref, ..
            }) = &mut group.shape;
            debug_assert_ne!(
                *shape_ref, removed_ref_idx,
                "a pruned input reference is addressed by no group"
            );
            if *shape_ref > removed_ref_idx {
                *shape_ref -= 1;
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

    // ── Group primitives (T25.F03, T25.F04) ─────────────────────────────────

    /// Whether a group of either shape can be created: all 24 direction
    /// sinks are present and the group list is below capacity.
    #[must_use]
    pub fn can_host_group(&self) -> bool {
        self.groups.len() < usize::from(u16::MAX)
            && Kind::ALL.into_iter().all(|kind| {
                (0..VOTE_DIRECTION_COUNT).all(|d| self.direction_sink(kind, d).is_some())
            })
    }

    /// The catalog index of `kind`'s direction-`d` sink, when present.
    fn direction_sink(&self, kind: Kind, d: u8) -> Option<usize> {
        let target = OutputSinkKind::ActionVote(kind.sink(d));
        self.output_sinks
            .iter()
            .position(|sink| sink.kind == target)
    }

    /// Every member of `group`, in canonical edge order.
    pub fn members(&self, group: GroupId) -> impl Iterator<Item = EdgeOccurrence> + '_ {
        self.output_sinks
            .iter()
            .enumerate()
            .flat_map(move |(sink, entry)| {
                entry
                    .inputs
                    .iter()
                    .enumerate()
                    .filter(move |(_, edge)| edge.is_member_of(group))
                    .map(move |(index, _)| EdgeOccurrence::new(EdgeSurface::SinkInput(sink), index))
            })
    }

    /// Create a `Ring` group reading `input_refs[ring_ref]` into `kind` at
    /// `offset`: one record at scale 0 and eight members at own weight 0,
    /// channel `c` on sink `kind[(c + offset) mod 8]`. A new group is silent.
    ///
    /// # Panics
    ///
    /// If `shape` is not a `Ring` (a pair group is created by
    /// [`Self::add_pair_group`]), [`Self::can_host_group`] is false or
    /// `offset >= 8`.
    pub fn add_group(&mut self, shape: Shape, kind: Kind, offset: u8) -> GroupId {
        let Shape::Ring { ring_ref } = shape else {
            panic!("add_group: a DirectionPair group is created by add_pair_group");
        };
        let id = self.push_group_record(shape, kind, offset);
        for channel in 0..VOTE_DIRECTION_COUNT {
            let direction = (channel + offset) % VOTE_DIRECTION_COUNT;
            self.insert_member(
                Self::new_member(id, ring_ref, u16::from(channel)),
                direction,
            );
        }
        self.debug_assert_groups();
        id
    }

    /// Create a `DirectionPair` group on pair `pair` (`layout`, the pair
    /// `input_refs[ref_idx]` declares there) into `kind` at `offset`: one
    /// record at scale 0 and the twelve members whose pattern is non-zero
    /// ([`super::pair_birth_members`]) at own weight 0. A new group is silent.
    ///
    /// # Panics
    ///
    /// If [`Self::can_host_group`] is false or `offset >= 8`.
    pub fn add_pair_group(
        &mut self,
        ref_idx: u16,
        pair: u8,
        layout: DirectionPair,
        kind: Kind,
        offset: u8,
    ) -> GroupId {
        let id = self.push_group_record(Shape::DirectionPair { ref_idx, pair }, kind, offset);
        for (field, direction) in pair_birth_members(layout, offset) {
            self.insert_member(Self::new_member(id, ref_idx, field), direction);
        }
        self.debug_assert_groups();
        id
    }

    fn push_group_record(&mut self, shape: Shape, kind: Kind, offset: u8) -> GroupId {
        assert!(
            self.can_host_group(),
            "add_group: the graph cannot host a group"
        );
        assert!(offset < VOTE_DIRECTION_COUNT, "add_group: offset below 8");
        let id = GroupId(u16::try_from(self.groups.len()).expect("groups below u16::MAX"));
        self.groups.push(Group {
            shape,
            kind,
            offset,
            scale: 0.0,
        });
        id
    }

    fn new_member(group: GroupId, ref_idx: u16, sub_idx: u16) -> GraphEdge {
        GraphEdge {
            source: GraphSource::InputLeaf { ref_idx, sub_idx },
            own_weight: 0.0,
            plasticity: None,
            membership: Some(Membership { group }),
        }
    }

    /// Remove `group`'s record and every member, renumbering every higher
    /// `GroupId`. Returns the number of members removed.
    pub fn remove_group(&mut self, group: GroupId) -> usize {
        let mut removed = 0;
        for sink in &mut self.output_sinks.0 {
            let before = sink.inputs.len();
            sink.inputs.0.retain(|edge| !edge.is_member_of(group));
            removed += before - sink.inputs.len();
        }
        self.drop_group_record(group);
        self.debug_assert_groups();
        removed
    }

    /// Move `group` to `kind` at `offset` (`ShiftGroupOffset`,
    /// `RetargetGroupKind`): every member leaves its sink and is re-inserted
    /// at its new sink in the I10 position, keeping its own weight. A `Ring`
    /// member moves by its channel; a `DirectionPair` member on direction `d`
    /// moves to `(d + offset − old_offset) mod 8`, so every member's
    /// relative coordinate, holes included, is preserved.
    ///
    /// # Panics
    ///
    /// If `offset >= 8`.
    pub fn relocate_group(&mut self, group: GroupId, kind: Kind, offset: u8) {
        assert!(
            offset < VOTE_DIRECTION_COUNT,
            "relocate_group: offset below 8"
        );
        let n = VOTE_DIRECTION_COUNT;
        let old = self.groups[group.index()];
        let mut moved = Vec::with_capacity(usize::from(2 * n));
        for sink in &mut self.output_sinks.0 {
            let direction = sink.kind.direction().map_or(0, |(_, d)| d);
            sink.inputs.0.retain(|edge| {
                let member = edge.is_member_of(group);
                if member {
                    moved.push((*edge, direction));
                }
                !member
            });
        }
        let record = &mut self.groups[group.index()];
        record.kind = kind;
        record.offset = offset;
        let record = *record;
        for (edge, direction) in moved {
            let direction = match record.shape {
                Shape::Ring { .. } => {
                    let (_, channel) = Self::member_key(&edge).expect("a member");
                    member_direction(&record, channel)
                }
                Shape::DirectionPair { .. } => (direction + n - old.offset + offset) % n,
            };
            self.insert_member(edge, direction);
        }
        self.debug_assert_groups();
    }

    /// Detach the member at `occurrence` with its group's `pattern` there:
    /// its own weight becomes its genomic weight
    /// `compose(own, scale, pattern)`, bitwise, its membership is cleared and
    /// it moves to the end of its sink's independent prefix. Detaching a
    /// group's last member removes the group. Returns the edge's new
    /// occurrence and whether the group was removed.
    pub fn detach_at(
        &mut self,
        occurrence: EdgeOccurrence,
        pattern: f32,
    ) -> (EdgeOccurrence, bool) {
        let EdgeSurface::SinkInput(sink) = occurrence.surface else {
            panic!("detach_at: only a sink edge is a member");
        };
        let mut edge = self.output_sinks.0[sink].inputs.0.remove(occurrence.index);
        let Membership { group } = edge.membership.expect("detach_at: the edge is a member");
        edge.own_weight = compose(edge.own_weight, self.groups[group.index()].scale, pattern);
        edge.membership = None;
        let at = self.first_member(sink);
        self.output_sinks.0[sink].inputs.0.insert(at, edge);
        let removed = self.members(group).next().is_none();
        if removed {
            self.drop_group_record(group);
        }
        self.debug_assert_groups();
        (
            EdgeOccurrence::new(EdgeSurface::SinkInput(sink), at),
            removed,
        )
    }

    /// A handle for changing `group`'s record.
    pub fn group_mut(&mut self, group: GroupId) -> GroupMut<'_> {
        GroupMut { def: self, group }
    }

    /// The index of `sink`'s first member, its input count when it has none.
    fn first_member(&self, sink: usize) -> usize {
        let inputs = &self.output_sinks[sink].inputs;
        inputs
            .iter()
            .position(|edge| edge.membership.is_some())
            .unwrap_or(inputs.len())
    }

    /// The I10 sort key of a member: `(GroupId, sub_idx)` (a `Ring`
    /// member's channel; a pair member's field, `x ≠ y`, so unique within a
    /// sink).
    fn member_key(edge: &GraphEdge) -> Option<(GroupId, u16)> {
        let Membership { group } = edge.membership?;
        let channel = match edge.source {
            GraphSource::InputLeaf { sub_idx, .. } => sub_idx,
            _ => u16::MAX,
        };
        Some((group, channel))
    }

    /// Insert member `edge` on its group's direction-`direction` sink in the
    /// I10 position.
    fn insert_member(&mut self, edge: GraphEdge, direction: u8) {
        let key = Self::member_key(&edge);
        let (group, _) = key.expect("a member");
        let sink = self
            .direction_sink(self.groups[group.index()].kind, direction)
            .expect("a group's graph carries every direction sink");
        let inputs = &mut self.output_sinks.0[sink].inputs.0;
        let at = inputs
            .iter()
            .position(|other| other.membership.is_some() && Self::member_key(other) > key)
            .unwrap_or(inputs.len());
        inputs.insert(at, edge);
    }

    /// Remove `group`'s record and decrement every higher `GroupId`.
    fn drop_group_record(&mut self, group: GroupId) {
        self.groups.remove(group.index());
        for sink in &mut self.output_sinks.0 {
            for edge in &mut sink.inputs.0 {
                if let Some(Membership { group: id }) = &mut edge.membership {
                    debug_assert_ne!(*id, group, "the dropped group has no member");
                    if *id > group {
                        id.0 -= 1;
                    }
                }
            }
        }
    }

    /// The def-local group invariants (I2–I6, I10, `offset < 8`), checked
    /// in debug builds at every group-touching primitive's exit.
    fn debug_assert_groups(&self) {
        if cfg!(debug_assertions) && !self.groups.is_empty() {
            if let Err(violation) = self.check_local_invariants() {
                panic!("graph invariant violated: {violation:?}");
            }
        }
    }

    /// The invariants that need only the def (I2–I6, I10, `offset < 8`). A
    /// group's member set is keyed by channel for a `Ring` and by
    /// `(sub_idx, direction)` for a `DirectionPair`; the pair's field roles
    /// need the input references ([`validate_graph`]).
    pub(crate) fn check_local_invariants(&self) -> Result<(), GraphInvariantViolation> {
        use GraphInvariantViolation as V;
        if self.groups.len() > usize::from(u16::MAX) {
            return Err(V::TooManyGroups);
        }
        if self
            .compute_nodes
            .iter()
            .any(|node| node.inputs.iter().any(|e| e.membership.is_some()))
        {
            return Err(V::MemberOnComputeInput);
        }
        if self
            .groups
            .iter()
            .any(|group| group.offset >= VOTE_DIRECTION_COUNT)
        {
            return Err(V::OffsetOutOfRange);
        }
        let mut members = vec![0u32; self.groups.len()];
        let mut ring_channels = vec![0u8; self.groups.len()];
        let mut pair_coordinates = std::collections::HashSet::new();
        for (sink_idx, sink) in self.output_sinks.iter().enumerate() {
            let mut previous: Option<(GroupId, u16)> = None;
            let mut in_members = false;
            for edge in sink.inputs.iter() {
                let Some(key) = Self::member_key(edge) else {
                    if in_members {
                        return Err(V::SinkOrder);
                    }
                    continue;
                };
                in_members = true;
                match previous.map(|previous| previous.cmp(&key)) {
                    Some(std::cmp::Ordering::Greater) => return Err(V::SinkOrder),
                    Some(std::cmp::Ordering::Equal) => return Err(V::DuplicateMember),
                    _ => {}
                }
                previous = Some(key);
                let (group, sub_idx) = key;
                let Some(record) = self.groups.get(group.index()) else {
                    return Err(V::GroupIdOutOfRange);
                };
                let on_kind = sink
                    .kind
                    .direction()
                    .filter(|&(kind, _)| kind == record.kind)
                    .map(|(_, d)| d);
                let reads_shape_ref = matches!(
                    edge.source,
                    GraphSource::InputLeaf { ref_idx, .. } if ref_idx == record.shape.ref_idx()
                );
                let fits = match record.shape {
                    Shape::Ring { .. } => {
                        sub_idx < u16::from(VOTE_DIRECTION_COUNT)
                            && reads_shape_ref
                            && self.direction_sink(record.kind, member_direction(record, sub_idx))
                                == Some(sink_idx)
                    }
                    Shape::DirectionPair { .. } => reads_shape_ref && on_kind.is_some(),
                };
                if !fits {
                    return Err(V::MemberShape);
                }
                let fresh = match record.shape {
                    Shape::Ring { .. } => {
                        let bit = 1u8 << sub_idx;
                        let fresh = ring_channels[group.index()] & bit == 0;
                        ring_channels[group.index()] |= bit;
                        fresh
                    }
                    Shape::DirectionPair { .. } => {
                        pair_coordinates.insert((group, sub_idx, on_kind.expect("fits")))
                    }
                };
                if !fresh {
                    return Err(V::DuplicateMember);
                }
                members[group.index()] += 1;
            }
        }
        if members.contains(&0) {
            return Err(V::EmptyGroup);
        }
        Ok(())
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
impl CgpGraphBackendDef {
    /// Fill the group table to its `u16::MAX`-record limit with memberless
    /// records, so [`Self::can_host_group`] is false on an otherwise full
    /// sink catalog (a preflight fixture; such a def fails `EmptyGroup`).
    pub(crate) fn exhaust_group_capacity(&mut self) {
        let record = Group {
            shape: Shape::Ring { ring_ref: 0 },
            kind: Kind::Move,
            offset: 0,
            scale: 0.0,
        };
        self.groups = vec![record; usize::from(u16::MAX)];
    }
}

#[cfg(test)]
#[path = "container_tests.rs"]
mod tests;
