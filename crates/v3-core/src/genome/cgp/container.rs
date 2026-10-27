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

use super::bundle::{
    compose, pair_birth_members, plans, ring_birth_members, BirthCoordinate, Bundle, BundleBirth,
    BundleId, BundleSource, Shape, Site, Wiring,
};
use super::edge::{GraphEdge, GraphSource};
use super::edge::{Membership, Scope};
use super::{
    ComputeNodeKind, OutputSinkKind, CUSTOM_OUTPUT_COUNT, FIXED_SINK_COUNT, SHARED_MEMORY_SLOTS,
};
use crate::contracts::votes::{
    ActionParamField, DirectedVoteKind as Kind, VoteSink, VOTE_DIRECTION_COUNT,
};
use crate::contracts::MAX_GATE_SLOTS;
use crate::contracts::{DirectionPair, FieldSet, InputReference};
use crate::genome::{BackendDef, NodeEditError, NodeGenome, PlasticityConfig};

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

/// A single compute node in the graph backend. Learning attributes live on
/// its input edges (T25.F06).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComputeNode {
    pub kind: ComputeNodeKind,
    pub inputs: EdgeList,
}

impl ComputeNode {
    #[must_use]
    pub fn new(kind: ComputeNodeKind, inputs: Vec<GraphEdge>) -> Self {
        Self {
            kind,
            inputs: EdgeList(inputs),
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

/// The birth payload a reproduction draft carries (T25.F01, design note D1,
/// T25.F06, T25.F07): the parent's learned own quantity per initialized
/// ungrouped or edge-scope learner on either surface and its learned scale
/// per initialized scale slot, captured before mutation
/// (`LearnedState::capture`) and consumed by the newborn's constructor. A parent backend with nothing
/// initialized carries no payload at all. Outside genome identity,
/// serialization and equality.
#[derive(Debug, Clone, PartialEq)]
pub enum BirthSeeds {
    /// One row per compute node and one per sink, each sized to that
    /// container's `inputs`, and one bundle slot per bundle record (I11);
    /// `None` where the edge or the bundle has no inherited value.
    Rows {
        compute: Vec<Vec<Option<f32>>>,
        sinks: Vec<Vec<Option<f32>>>,
        bundles: Vec<Option<f32>>,
    },
}

impl BirthSeeds {
    /// The row of `surface`, empty when the payload has none.
    #[must_use]
    pub fn row(&self, surface: EdgeSurface) -> &[Option<f32>] {
        let Self::Rows { compute, sinks, .. } = self;
        let rows = match surface {
            EdgeSurface::ComputeInput(i) => compute.get(i),
            EdgeSurface::SinkInput(i) => sinks.get(i),
        };
        rows.map_or(&[], Vec::as_slice)
    }

    /// The inherited value of the edge at `occurrence`, if any.
    #[must_use]
    pub fn value(&self, occurrence: EdgeOccurrence) -> Option<f32> {
        self.row(occurrence.surface)
            .get(occurrence.index)
            .copied()
            .flatten()
    }

    /// The inherited scale of `bundle`, if any.
    #[must_use]
    pub fn bundle_value(&self, bundle: BundleId) -> Option<f32> {
        self.bundle_slots().get(bundle.index()).copied().flatten()
    }

    /// One slot per bundle record.
    #[must_use]
    pub fn bundle_slots(&self) -> &[Option<f32>] {
        let Self::Rows { bundles, .. } = self;
        bundles
    }

    /// One row per compute node.
    #[must_use]
    pub fn compute_rows(&self) -> &[Vec<Option<f32>>] {
        let Self::Rows { compute, .. } = self;
        compute
    }

    /// One row per sink.
    #[must_use]
    pub fn sink_rows(&self) -> &[Vec<Option<f32>>] {
        let Self::Rows { sinks, .. } = self;
        sinks
    }

    fn row_mut(&mut self, surface: EdgeSurface) -> &mut Vec<Option<f32>> {
        let Self::Rows { compute, sinks, .. } = self;
        match surface {
            EdgeSurface::ComputeInput(i) => &mut compute[i],
            EdgeSurface::SinkInput(i) => &mut sinks[i],
        }
    }

    fn bundles_mut(&mut self) -> &mut Vec<Option<f32>> {
        let Self::Rows { bundles, .. } = self;
        bundles
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
    /// Edge-bundle records (T25.F03), indexed by `BundleId`; omitted from JSON
    /// when empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    bundles: Vec<Bundle>,
}

// Keep the birth payload out of diagnostic genome identity as well as
// serialization and equality. Existing state fingerprints use this format.
impl fmt::Debug for CgpGraphBackendDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = f.debug_struct("CgpGraphBackendDef");
        debug
            .field("compute_nodes", &self.compute_nodes)
            .field("output_sinks", &self.output_sinks);
        debug.field("bundles", &self.bundles).finish()
    }
}

impl PartialEq for CgpGraphBackendDef {
    fn eq(&self, other: &Self) -> bool {
        self.compute_nodes == other.compute_nodes
            && self.output_sinks == other.output_sinks
            && self.bundles == other.bundles
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
            "a member's source changes only through its bundle or after a detach"
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

    /// Set or clear the edge's learning attributes (T25.F06), the only
    /// writer; no seed change. A config is in range (I1) and the edge may
    /// learn ([`CgpGraphBackendDef::can_learn`]).
    pub fn set_plasticity(&mut self, plasticity: Option<PlasticityConfig>) {
        debug_assert!(
            plasticity.is_none_or(|config| config.in_range()),
            "set_plasticity: a config is in range (I1)"
        );
        debug_assert!(
            plasticity.is_none() || self.def.can_learn(self.occurrence),
            "set_plasticity: a ClearSlot edge does not learn"
        );
        self.def.edge_slot(self.occurrence).plasticity = plasticity;
    }

    /// Set a member's learning scope (T25.F07), the only writer; no seed
    /// change.
    ///
    /// # Panics
    ///
    /// If the edge is not a member.
    pub fn set_scope(&mut self, scope: Scope) {
        self.def
            .edge_slot(self.occurrence)
            .membership
            .as_mut()
            .expect("set_scope: the edge is a member")
            .scope = scope;
    }
}

/// A handle on one bundle's record.
pub struct BundleMut<'a> {
    def: &'a mut CgpGraphBackendDef,
    bundle: BundleId,
}

impl BundleMut<'_> {
    /// The record as it stands.
    #[must_use]
    pub fn bundle(&self) -> Bundle {
        self.def.bundles[self.bundle.index()]
    }

    /// Set the bundle's shared scale; its seed is cleared.
    pub fn set_scale(&mut self, scale: f32) {
        if let Some(seeds) = &mut self.def.birth_seeds {
            seeds.bundles_mut()[self.bundle.index()] = None;
        }
        self.def.bundles[self.bundle.index()].scale = scale;
    }

    /// Set whether a parent's learned scale seeds the child's (T25.F07),
    /// the only writer; no seed change.
    pub fn set_inherit_scale(&mut self, inherit_scale: bool) {
        self.def.bundles[self.bundle.index()].inherit_scale = inherit_scale;
    }

    /// Set the learned scale's retention (T25.F09), the only writer; in
    /// `[0, 1]`; no seed change.
    pub fn set_retention(&mut self, retention: f32) {
        debug_assert!(
            (0.0..=1.0).contains(&retention),
            "set_retention: a retention is in [0, 1]"
        );
        self.def.bundles[self.bundle.index()].retention = retention;
    }
}

/// The graph invariant a genome breaks (T25.F03, design note §3, I2–I10).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphInvariantViolation {
    /// I2: a compute-input edge carries a membership.
    MemberOnComputeInput,
    /// I3: a membership names no bundle.
    BundleIdOutOfRange,
    /// I3: more than `u16::MAX` bundles.
    TooManyBundles,
    /// I4, I8: a member does not fit its bundle's shape.
    MemberShape,
    /// I5: two members of one bundle share a coordinate.
    DuplicateMember,
    /// I6: a bundle has no member.
    EmptyBundle,
    /// I7: a bundle's offset is not below 8.
    OffsetOutOfRange,
    /// I7: a bundle's shape reference is not a ring (`Ring`) or does not
    /// declare the named pair (`DirectionPair`).
    ShapeReference,
    /// I10: a sink's members are not after its independent edges in
    /// `(BundleId, sub_idx)` order.
    SinkOrder,
    /// I1 (T25.F06): a learning attribute is non-finite or out of range.
    PlasticityOutOfRange,
    /// T25.F06 invariant 2, T25.F07 invariant 1: a `ClearSlot` edge is
    /// plastic.
    IneligiblePlasticEdge,
    /// T25.F09 invariant 1: a bundle's retention is non-finite or outside
    /// `[0, 1]`.
    BundleRetentionOutOfRange,
}

/// The direction a `Ring` member of `bundle` on `channel` votes toward:
/// `(offset + channel) mod 8`.
fn member_direction(bundle: &Bundle, channel: u16) -> u8 {
    ((u16::from(bundle.offset) + channel) % u16::from(VOTE_DIRECTION_COUNT)) as u8
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
    for bundle in &def.bundles {
        let resolves = match bundle.shape {
            Shape::Ring { ring_ref } => node
                .input_refs
                .get(usize::from(ring_ref))
                .is_some_and(InputReference::is_ring),
            Shape::DirectionPair { .. } => bundle.shape.resolve_pair(&node.input_refs).is_some(),
        };
        if !resolves {
            return Err(GraphInvariantViolation::ShapeReference);
        }
    }
    // I4's pair roles (T25.F10): a direction member reads its pair's `x` or
    // `y` field, a fact member one of its pair's `facts`.
    for sink in def.output_sinks.iter() {
        for edge in sink.inputs.iter() {
            let Some(Membership { bundle, .. }) = edge.membership else {
                continue;
            };
            let record = &def.bundles[bundle.index()];
            let Some(pair) = record.shape.resolve_pair(&node.input_refs) else {
                continue;
            };
            let GraphSource::InputLeaf { sub_idx, .. } = edge.source else {
                return Err(GraphInvariantViolation::MemberShape);
            };
            let fits = match Site::of(sink.kind, record.kind) {
                Some(Site::Direction(_)) => sub_idx == pair.x || sub_idx == pair.y,
                Some(Site::Drive) => pair.facts.contains(&sub_idx),
                None => false,
            };
            if !fits {
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
            bundles: Vec::new(),
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
        self.birth_seeds = None;
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

    /// Every plastic edge with its learning attributes, in canonical order
    /// (T25.F06): the one definition of "carries learning".
    pub fn plastic_edges(&self) -> impl Iterator<Item = (EdgeOccurrence, PlasticityConfig)> + '_ {
        self.edge_sites().filter_map(|occurrence| {
            self.edge_at(occurrence)
                .plasticity
                .map(|config| (occurrence, config))
        })
    }

    /// Whether the edge at `occurrence` may carry learning attributes
    /// (T25.F06 invariant 2, T25.F07 invariant 1): every edge except one of
    /// a `ClearSlot` sink, whose sum is ignored. A member learns at its
    /// scope.
    #[must_use]
    pub fn can_learn(&self, occurrence: EdgeOccurrence) -> bool {
        match occurrence.surface {
            EdgeSurface::ComputeInput(_) => true,
            EdgeSurface::SinkInput(i) => {
                !matches!(self.output_sinks[i].kind, OutputSinkKind::ClearSlot(_))
            }
        }
    }

    /// The edge-bundle records, indexed by `BundleId`.
    #[inline]
    #[must_use]
    pub fn bundles(&self) -> &[Bundle] {
        &self.bundles
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
                self.bundles[membership.bundle.index()].scale,
                self.pattern_at(edge, sink, input_refs),
            ),
        }
    }

    /// The pattern of the edge at `occurrence`: its bundle's at the member's
    /// field and sink direction, 0 for an ungrouped edge (T25.F07).
    #[must_use]
    pub fn pattern(&self, occurrence: EdgeOccurrence, input_refs: &[InputReference]) -> f32 {
        if self.edge_at(occurrence).membership.is_none() {
            return 0.0;
        }
        self.member_pattern(occurrence, input_refs)
    }

    /// The pattern of the member at `occurrence` (its bundle's at the
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

    /// The pattern of member `edge` on a sink of kind `sink`: 0 for a fact
    /// member, by role, before any geometry (T25.F10); otherwise its bundle's
    /// at the member's field and the sink's direction (pattern 0 for a
    /// member that reads no input leaf or sits on no direction sink of its
    /// bundle's kind).
    fn pattern_at(
        &self,
        edge: &GraphEdge,
        sink: OutputSinkKind,
        input_refs: &[InputReference],
    ) -> f32 {
        let Membership { bundle, .. } = edge.membership.expect("pattern_at: a member");
        let record = &self.bundles[bundle.index()];
        let (GraphSource::InputLeaf { sub_idx, .. }, Some(Site::Direction(direction))) =
            (edge.source, Site::of(sink, record.kind))
        else {
            return 0.0;
        };
        record.pattern(input_refs, sub_idx, direction)
    }

    /// The site of the member at `occurrence` relative to its bundle's kind
    /// (T25.F10); `None` for an ungrouped edge or a member off its kind's
    /// sinks (a broken genome).
    #[must_use]
    pub fn site_of(&self, occurrence: EdgeOccurrence) -> Option<Site> {
        let EdgeSurface::SinkInput(sink) = occurrence.surface else {
            return None;
        };
        let Membership { bundle, .. } = self.edge_at(occurrence).membership?;
        Site::of(
            self.output_sinks[sink].kind,
            self.bundles[bundle.index()].kind,
        )
    }

    /// Whether the edge at `occurrence` is a fact member: a member on its
    /// bundle's kind's drive sink (T25.F10).
    #[must_use]
    pub fn is_fact_member(&self, occurrence: EdgeOccurrence) -> bool {
        self.site_of(occurrence) == Some(Site::Drive)
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
    /// with a wired output sink (a bundle's members wire their sinks). A graph
    /// with neither computes nothing and applies nothing, so its visit is
    /// free. Derived from the genome on every visit and stored nowhere.
    #[must_use]
    pub fn enters_visit(&self) -> bool {
        !self.compute_nodes.is_empty()
            || self.output_sinks.iter().any(|sink| !sink.inputs.is_empty())
    }

    // ── Seeds ───────────────────────────────────────────────────────────────

    /// Attach a reproduction draft's payload. Rows must match the compute
    /// nodes, the sinks and their inputs (I11).
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
        if let Some(rows) = self.compute_seed_rows_mut() {
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
        if let Some(BirthSeeds::Rows { compute, sinks, .. }) = &mut self.birth_seeds {
            compute.remove(idx);
            let read_removed = GraphSource::ComputeNode(idx as u16);
            let containers = self
                .compute_nodes
                .iter()
                .map(|node| &node.inputs)
                .chain(self.output_sinks.iter().map(|sink| &sink.inputs));
            for (inputs, values) in containers.zip(compute.iter_mut().chain(sinks.iter_mut())) {
                for (edge, inherited) in inputs.iter().zip(values) {
                    if edge.source == read_removed {
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
        if let Some(rows) = self.compute_seed_rows_mut() {
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
        if let Some(rows) = self.compute_seed_rows_mut() {
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

    // ── Edge primitives ─────────────────────────────────────────────────────

    /// Add `edge` to `surface` with an empty seed: appended on a compute
    /// surface, at the end of the independent prefix on a sink (I10).
    pub fn push_edge(&mut self, surface: EdgeSurface, edge: GraphEdge) {
        match surface {
            EdgeSurface::ComputeInput(i) => {
                self.compute_nodes.0[i].inputs.0.push(edge);
                if let Some(seeds) = &mut self.birth_seeds {
                    seeds.row_mut(surface).push(None);
                }
            }
            EdgeSurface::SinkInput(i) => {
                debug_assert!(
                    edge.membership.is_none(),
                    "push_edge adds an ungrouped edge"
                );
                debug_assert!(
                    edge.plasticity.is_none()
                        || !matches!(self.output_sinks[i].kind, OutputSinkKind::ClearSlot(_)),
                    "a ClearSlot edge never learns"
                );
                let at = self.first_member(i);
                self.output_sinks.0[i].inputs.0.insert(at, edge);
                if let Some(seeds) = &mut self.birth_seeds {
                    seeds.row_mut(surface).insert(at, None);
                }
            }
        }
        self.debug_assert_seed_shape();
        self.debug_assert_bundles();
    }

    /// Remove the edge at `occurrence` with its seed; a bundle's last member
    /// takes its record along. Returns whether a record was removed.
    pub fn remove_edge(&mut self, occurrence: EdgeOccurrence) -> bool {
        let EdgeOccurrence { surface, index } = occurrence;
        let mut record_removed = false;
        if let Some(seeds) = &mut self.birth_seeds {
            seeds.row_mut(surface).remove(index);
        }
        match surface {
            EdgeSurface::ComputeInput(i) => {
                self.compute_nodes.0[i].inputs.0.remove(index);
            }
            EdgeSurface::SinkInput(i) => {
                let removed = self.output_sinks.0[i].inputs.0.remove(index);
                if let Some(Membership { bundle, .. }) = removed.membership {
                    if self.members(bundle).next().is_none() {
                        self.drop_bundle_record(bundle);
                        record_removed = true;
                    }
                }
            }
        }
        self.debug_assert_seed_shape();
        self.debug_assert_bundles();
        record_removed
    }

    /// Append copies of compute node `from`'s edges, with their seeds, to
    /// compute node `to`.
    pub fn copy_edges(&mut self, from: usize, to: usize) {
        let copied = self.compute_nodes[from].inputs.0.clone();
        self.compute_nodes.0[to].inputs.0.extend(copied);
        if let Some(rows) = self.compute_seed_rows_mut() {
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
    /// every higher `InputLeaf.ref_idx` and bundle shape reference. `Prune`
    /// removes only an index nothing addresses (`prunable_indices`), so no
    /// edge or bundle is removed and the seeds are untouched.
    pub fn reindex_input_refs_after_removal(&mut self, removed_ref_idx: u16) {
        for bundle in &mut self.bundles {
            let (Shape::Ring {
                ring_ref: shape_ref,
            }
            | Shape::DirectionPair {
                ref_idx: shape_ref, ..
            }) = &mut bundle.shape;
            debug_assert_ne!(
                *shape_ref, removed_ref_idx,
                "a pruned input reference is addressed by no bundle"
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

    // ── Bundle primitives (T25.F03, T25.F04) ─────────────────────────────────

    /// Whether a bundle of either shape can be created: all 24 direction
    /// sinks are present and the bundle list is below capacity.
    #[must_use]
    pub fn can_host_bundle(&self) -> bool {
        self.bundles.len() < usize::from(u16::MAX)
            && Kind::ALL.into_iter().all(|kind| {
                (0..VOTE_DIRECTION_COUNT)
                    .all(|d| self.site_sink(kind, Site::Direction(d)).is_some())
            })
    }

    /// Whether a gene of kind `kind` can carry fact members: `kind`'s drive
    /// sink is present (T25.F10). Recruitment's pair preflight and
    /// `RetargetBundleKind` read it.
    #[must_use]
    pub fn can_host_facts(&self, kind: Kind) -> bool {
        self.site_sink(kind, Site::Drive).is_some()
    }

    /// The catalog index of the sink `site` names for a kind-`kind` bundle,
    /// when present.
    fn site_sink(&self, kind: Kind, site: Site) -> Option<usize> {
        let target = site.sink(kind);
        self.output_sinks
            .iter()
            .position(|sink| sink.kind == target)
    }

    /// Every member of `bundle`, in canonical edge order.
    pub fn members(&self, bundle: BundleId) -> impl Iterator<Item = EdgeOccurrence> + '_ {
        self.output_sinks
            .iter()
            .enumerate()
            .flat_map(move |(sink, entry)| {
                entry
                    .inputs
                    .iter()
                    .enumerate()
                    .filter(move |(_, edge)| edge.is_member_of(bundle))
                    .map(move |(index, _)| EdgeOccurrence::new(EdgeSurface::SinkInput(sink), index))
            })
    }

    /// The birth coordinates of `bundle` with presence (T30.F01): a `Ring`
    /// bundle's eight [`ring_birth_members`]; a `DirectionPair` bundle's
    /// twelve [`super::pair_birth_members`] then one drive coordinate per
    /// declared fact of its pair, none when the pair no longer resolves. A
    /// coordinate is present when a member of `bundle` reads its field of
    /// the shape's reference on its site.
    #[must_use]
    pub fn birth_coordinates(
        &self,
        bundle: BundleId,
        input_refs: &[InputReference],
    ) -> Vec<BirthCoordinate> {
        let record = &self.bundles[bundle.index()];
        let ref_idx = record.shape.ref_idx();
        let births: Vec<(u16, Site)> = match record.shape {
            Shape::Ring { .. } => ring_birth_members(record.offset)
                .map(|(field, d)| (field, Site::Direction(d)))
                .collect(),
            Shape::DirectionPair { .. } => record
                .shape
                .resolve_pair(input_refs)
                .map(|pair| {
                    pair_birth_members(pair, record.offset)
                        .map(|(field, d)| (field, Site::Direction(d)))
                        .chain(pair.facts.iter().map(|&field| (field, Site::Drive)))
                        .collect()
                })
                .unwrap_or_default(),
        };
        let members: Vec<(u16, Site)> = self
            .members(bundle)
            .filter_map(|member| match self.edge_at(member).source {
                GraphSource::InputLeaf {
                    ref_idx: at,
                    sub_idx,
                } if at == ref_idx => Some((sub_idx, self.site_of(member)?)),
                _ => None,
            })
            .collect();
        births
            .into_iter()
            .map(|(field, site)| BirthCoordinate {
                field,
                site,
                present: members.contains(&(field, site)),
            })
            .collect()
    }

    /// Create a `Ring` bundle reading `input_refs[ring_ref]` into `kind` at
    /// `offset`: one record at scale 0 and eight members at own weight 0,
    /// channel `c` on sink `kind[(c + offset) mod 8]`. A new bundle is silent.
    ///
    /// # Panics
    ///
    /// If `shape` is not a `Ring` (a pair bundle is created by
    /// [`Self::add_pair_bundle`]), [`Self::can_host_bundle`] is false or
    /// `offset >= 8`.
    pub fn add_ring_bundle(&mut self, shape: Shape, kind: Kind, offset: u8) -> BundleId {
        let Shape::Ring { ring_ref } = shape else {
            panic!("add_bundle: a DirectionPair bundle is created by add_pair_bundle");
        };
        let id = self.push_bundle_record(shape, kind, offset);
        for (channel, direction) in ring_birth_members(offset) {
            self.insert_member(
                Self::new_member(id, ring_ref, channel),
                Site::Direction(direction),
                None,
            );
        }
        self.debug_assert_bundles();
        id
    }

    /// Create a `DirectionPair` gene on pair `pair` (`layout`, the pair
    /// `input_refs[ref_idx]` declares there) into `kind` at `offset`: one
    /// record at scale 0, the twelve direction members whose pattern is
    /// non-zero ([`super::pair_birth_members`]), then one fact member on
    /// `kind`'s drive sink per field of `facts`, in `facts` order (T25.F10):
    /// every member born at own weight 0, static, scope `Edge`, no seed. A
    /// new gene is silent. `AddDirectionPairBundle` passes no facts;
    /// recruitment's pair case passes `layout.facts`.
    ///
    /// # Panics
    ///
    /// If [`Self::can_host_bundle`] is false, `offset >= 8`, a field of
    /// `facts` is not one of `layout.facts`, or `facts` is non-empty and
    /// [`Self::can_host_facts`] is false for `kind`.
    pub fn add_pair_bundle(
        &mut self,
        ref_idx: u16,
        pair: u8,
        layout: DirectionPair,
        kind: Kind,
        offset: u8,
        facts: &[u16],
    ) -> BundleId {
        assert!(
            facts.iter().all(|field| layout.facts.contains(field)),
            "add_pair_bundle: every fact member reads one of the pair's facts"
        );
        assert!(
            facts.is_empty() || self.can_host_facts(kind),
            "add_pair_bundle: fact members need the kind's drive sink"
        );
        let id = self.push_bundle_record(Shape::DirectionPair { ref_idx, pair }, kind, offset);
        for (field, direction) in pair_birth_members(layout, offset) {
            self.insert_member(
                Self::new_member(id, ref_idx, field),
                Site::Direction(direction),
                None,
            );
        }
        for &field in facts {
            self.insert_member(Self::new_member(id, ref_idx, field), Site::Drive, None);
        }
        self.debug_assert_bundles();
        id
    }

    /// Build one bundle reading `source`, whose field set the node resolved
    /// to `set`, with `birth`'s wiring and scale: every member of the set
    /// at own weight 0, static, scope `Edge`, no seed, in one edit.
    ///
    /// # Errors
    ///
    /// The wiring is not admissible for `set` ([`plans`]) or the graph holds
    /// `u16::MAX` records; nothing is written.
    // Commit 1 of T25.F11: a record holds a sensor's `Ring` or `Pair` set
    // only; any other source or set is `Inadmissible` until the record
    // carries its source and wiring.
    pub fn add_bundle(
        &mut self,
        source: BundleSource,
        set: FieldSet,
        birth: BundleBirth,
    ) -> Result<BundleId, NodeEditError> {
        if !plans(self, &set).contains(&birth.wiring) {
            return Err(NodeEditError::Inadmissible);
        }
        if self.bundles.len() >= usize::from(u16::MAX) {
            return Err(NodeEditError::BundleCapacity);
        }
        let BundleSource::Sensor { ref_idx, field_set } = source else {
            return Err(NodeEditError::Inadmissible);
        };
        let id = match (set, birth.wiring) {
            (FieldSet::Ring, Wiring::Ring { kind, offset }) => {
                self.add_ring_bundle(Shape::Ring { ring_ref: ref_idx }, kind, offset)
            }
            (FieldSet::Pair(layout), Wiring::DirectionPair { kind, offset }) => {
                self.add_pair_bundle(ref_idx, field_set, layout, kind, offset, layout.facts)
            }
            _ => return Err(NodeEditError::Inadmissible),
        };
        self.bundle_mut(id).set_scale(birth.scale);
        Ok(id)
    }

    fn push_bundle_record(&mut self, shape: Shape, kind: Kind, offset: u8) -> BundleId {
        assert!(
            self.can_host_bundle(),
            "add_bundle: the graph cannot host a bundle"
        );
        assert!(offset < VOTE_DIRECTION_COUNT, "add_bundle: offset below 8");
        let id = BundleId(u16::try_from(self.bundles.len()).expect("bundles below u16::MAX"));
        self.bundles.push(Bundle::new(shape, kind, offset));
        if let Some(seeds) = &mut self.birth_seeds {
            seeds.bundles_mut().push(None);
        }
        id
    }

    fn new_member(bundle: BundleId, ref_idx: u16, sub_idx: u16) -> GraphEdge {
        GraphEdge {
            source: GraphSource::InputLeaf { ref_idx, sub_idx },
            own_weight: 0.0,
            plasticity: None,
            membership: Some(Membership {
                bundle,
                scope: Scope::Edge,
            }),
        }
    }

    /// Remove `bundle`'s record and every member, renumbering every higher
    /// `BundleId`. Returns the number of members removed.
    pub fn remove_bundle(&mut self, bundle: BundleId) -> usize {
        let removed = self.take_members(bundle).len();
        self.drop_bundle_record(bundle);
        self.debug_assert_bundles();
        removed
    }

    /// Move `bundle` to `kind` at `offset` (`ShiftBundleOffset`,
    /// `RetargetBundleKind`): every member leaves its sink and is re-inserted
    /// at its new sink in the I10 position, keeping its own weight. A `Ring`
    /// member moves by its channel; a `DirectionPair` direction member on
    /// direction `d` moves to `(d + offset − old_offset) mod 8`, so every
    /// member's relative coordinate, holes included, is preserved; a fact
    /// member moves to the new kind's drive sink (T25.F10).
    ///
    /// # Panics
    ///
    /// If `offset >= 8`, or the bundle has a fact member and
    /// [`Self::can_host_facts`] is false for `kind`.
    pub fn relocate_bundle(&mut self, bundle: BundleId, kind: Kind, offset: u8) {
        assert!(
            offset < VOTE_DIRECTION_COUNT,
            "relocate_bundle: offset below 8"
        );
        let n = VOTE_DIRECTION_COUNT;
        let old = self.bundles[bundle.index()];
        let moved = self.take_members(bundle);
        let record = &mut self.bundles[bundle.index()];
        record.kind = kind;
        record.offset = offset;
        let record = *record;
        for (edge, site, seed) in moved {
            let site = match (record.shape, site) {
                (_, Site::Drive) => Site::Drive,
                (Shape::Ring { .. }, Site::Direction(_)) => {
                    let (_, channel) = Self::member_key(&edge).expect("a member");
                    Site::Direction(member_direction(&record, channel))
                }
                (Shape::DirectionPair { .. }, Site::Direction(d)) => {
                    Site::Direction((d + n - old.offset + offset) % n)
                }
            };
            self.insert_member(edge, site, seed);
        }
        self.debug_assert_bundles();
    }

    /// Detach the member at `occurrence` with its bundle's `pattern` there:
    /// its own weight becomes its genomic weight
    /// `compose(own, scale, pattern)`, bitwise, its membership is cleared and
    /// it moves to the end of its sink's independent prefix. A fact member
    /// (T25.F10) detaches own-only: its own weight bits and its seed entry
    /// stay exactly as stored and `pattern` is not read. Detaching a bundle's
    /// last member removes the bundle. Returns the edge's new occurrence and
    /// whether the bundle was removed.
    pub fn detach_at(
        &mut self,
        occurrence: EdgeOccurrence,
        pattern: f32,
    ) -> (EdgeOccurrence, bool) {
        let EdgeSurface::SinkInput(sink) = occurrence.surface else {
            panic!("detach_at: only a sink edge is a member");
        };
        let fact = self.is_fact_member(occurrence);
        let mut edge = self.output_sinks.0[sink].inputs.0.remove(occurrence.index);
        let mut seed = self
            .birth_seeds
            .as_mut()
            .and_then(|seeds| seeds.row_mut(occurrence.surface).remove(occurrence.index));
        let Membership { bundle, .. } = edge.membership.expect("detach_at: the edge is a member");
        if !fact {
            let scale = self.bundles[bundle.index()].scale;
            // The composed seed (T25.F07 invariant 6): what the detached
            // edge weighed at the parent, from the edge's and the bundle's
            // seeds, each falling back to its genomic value; no flag read and
            // no clamp (`from_seeds` clamps at construction).
            let bundle_seed = self
                .birth_seeds
                .as_ref()
                .and_then(|seeds| seeds.bundle_value(bundle));
            seed = (seed.is_some() || bundle_seed.is_some()).then(|| {
                compose(
                    seed.unwrap_or(edge.own_weight),
                    bundle_seed.unwrap_or(scale),
                    pattern,
                )
            });
            edge.own_weight = compose(edge.own_weight, scale, pattern);
        }
        edge.membership = None;
        let at = self.first_member(sink);
        self.output_sinks.0[sink].inputs.0.insert(at, edge);
        if let Some(seeds) = &mut self.birth_seeds {
            seeds.row_mut(occurrence.surface).insert(at, seed);
        }
        let removed = self.members(bundle).next().is_none();
        if removed {
            self.drop_bundle_record(bundle);
        }
        self.debug_assert_bundles();
        (
            EdgeOccurrence::new(EdgeSurface::SinkInput(sink), at),
            removed,
        )
    }

    /// A handle for changing `bundle`'s record.
    pub fn bundle_mut(&mut self, bundle: BundleId) -> BundleMut<'_> {
        BundleMut { def: self, bundle }
    }

    /// The index of `sink`'s first member, its input count when it has none.
    fn first_member(&self, sink: usize) -> usize {
        let inputs = &self.output_sinks[sink].inputs;
        inputs
            .iter()
            .position(|edge| edge.membership.is_some())
            .unwrap_or(inputs.len())
    }

    /// The I10 sort key of a member: `(BundleId, sub_idx)` (a `Ring`
    /// member's channel; a pair member's field, `x ≠ y`, so unique within a
    /// sink).
    fn member_key(edge: &GraphEdge) -> Option<(BundleId, u16)> {
        let Membership { bundle, .. } = edge.membership?;
        let channel = match edge.source {
            GraphSource::InputLeaf { sub_idx, .. } => sub_idx,
            _ => u16::MAX,
        };
        Some((bundle, channel))
    }

    /// Insert member `edge` with `seed` on the sink `site` names for its
    /// bundle's kind, in the I10 position.
    fn insert_member(&mut self, edge: GraphEdge, site: Site, seed: Option<f32>) {
        let key = Self::member_key(&edge);
        let (bundle, _) = key.expect("a member");
        let sink = self
            .site_sink(self.bundles[bundle.index()].kind, site)
            .expect("a bundle's graph carries every sink its members sit on");
        let inputs = &mut self.output_sinks.0[sink].inputs.0;
        let at = inputs
            .iter()
            .position(|other| other.membership.is_some() && Self::member_key(other) > key)
            .unwrap_or(inputs.len());
        inputs.insert(at, edge);
        if let Some(seeds) = &mut self.birth_seeds {
            seeds.row_mut(EdgeSurface::SinkInput(sink)).insert(at, seed);
        }
    }

    /// Remove every member of `bundle` from its sink, in canonical order,
    /// each with its site and its seed.
    fn take_members(&mut self, bundle: BundleId) -> Vec<(GraphEdge, Site, Option<f32>)> {
        let kind = self.bundles[bundle.index()].kind;
        let mut taken = Vec::with_capacity(usize::from(2 * VOTE_DIRECTION_COUNT));
        for (sink_idx, sink) in self.output_sinks.0.iter_mut().enumerate() {
            let Some(site) = Site::of(sink.kind, kind) else {
                debug_assert!(
                    !sink.inputs.iter().any(|edge| edge.is_member_of(bundle)),
                    "a member sits on a sink of its bundle's kind"
                );
                continue;
            };
            let mut seeds = self
                .birth_seeds
                .as_mut()
                .map(|seeds| seeds.row_mut(EdgeSurface::SinkInput(sink_idx)));
            let mut index = 0;
            while index < sink.inputs.len() {
                if sink.inputs[index].is_member_of(bundle) {
                    let edge = sink.inputs.0.remove(index);
                    let seed = seeds.as_mut().and_then(|row| row.remove(index));
                    taken.push((edge, site, seed));
                } else {
                    index += 1;
                }
            }
        }
        taken
    }

    /// Remove `bundle`'s record and decrement every higher `BundleId`.
    fn drop_bundle_record(&mut self, bundle: BundleId) {
        self.bundles.remove(bundle.index());
        if let Some(seeds) = &mut self.birth_seeds {
            seeds.bundles_mut().remove(bundle.index());
        }
        for sink in &mut self.output_sinks.0 {
            for edge in &mut sink.inputs.0 {
                if let Some(Membership { bundle: id, .. }) = &mut edge.membership {
                    debug_assert_ne!(*id, bundle, "the dropped bundle has no member");
                    if *id > bundle {
                        id.0 -= 1;
                    }
                }
            }
        }
    }

    /// The def-local bundle invariants (I2–I6, I10, `offset < 8`), checked
    /// in debug builds at every bundle-touching primitive's exit.
    fn debug_assert_bundles(&self) {
        if cfg!(debug_assertions) && !self.bundles.is_empty() {
            if let Err(violation) = self.check_local_invariants() {
                panic!("graph invariant violated: {violation:?}");
            }
        }
    }

    /// The invariants that need only the def (I2–I6, I10, `offset < 8`). A
    /// bundle's member set is keyed by channel for a `Ring` and by
    /// `(sub_idx, site)` for a `DirectionPair` (a drive site only on the
    /// record's kind, T25.F10); the pair's field roles need the input
    /// references ([`validate_graph`]).
    pub(crate) fn check_local_invariants(&self) -> Result<(), GraphInvariantViolation> {
        use GraphInvariantViolation as V;
        if self.bundles.len() > usize::from(u16::MAX) {
            return Err(V::TooManyBundles);
        }
        if self
            .compute_nodes
            .iter()
            .any(|node| node.inputs.iter().any(|e| e.membership.is_some()))
        {
            return Err(V::MemberOnComputeInput);
        }
        if self
            .bundles
            .iter()
            .any(|bundle| bundle.offset >= VOTE_DIRECTION_COUNT)
        {
            return Err(V::OffsetOutOfRange);
        }
        if self
            .bundles
            .iter()
            .any(|bundle| !(0.0..=1.0).contains(&bundle.retention))
        {
            return Err(V::BundleRetentionOutOfRange);
        }
        for (occurrence, config) in self.plastic_edges() {
            if !config.in_range() {
                return Err(V::PlasticityOutOfRange);
            }
            if !self.can_learn(occurrence) {
                return Err(V::IneligiblePlasticEdge);
            }
        }
        let mut members = vec![0u32; self.bundles.len()];
        let mut ring_channels = vec![0u8; self.bundles.len()];
        let mut pair_coordinates = std::collections::HashSet::new();
        for sink in self.output_sinks.iter() {
            let mut previous: Option<(BundleId, u16)> = None;
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
                let (bundle, sub_idx) = key;
                let Some(record) = self.bundles.get(bundle.index()) else {
                    return Err(V::BundleIdOutOfRange);
                };
                let reads_shape_ref = matches!(
                    edge.source,
                    GraphSource::InputLeaf { ref_idx, .. } if ref_idx == record.shape.ref_idx()
                );
                // A `Ring` member sits on the direction sink of its channel;
                // a `DirectionPair` member on a direction sink of its kind
                // or, as a fact member, on its kind's drive (T25.F10).
                let site = Site::of(sink.kind, record.kind).filter(|&site| {
                    reads_shape_ref
                        && match (record.shape, site) {
                            (Shape::Ring { .. }, Site::Direction(d)) => {
                                sub_idx < u16::from(VOTE_DIRECTION_COUNT)
                                    && member_direction(record, sub_idx) == d
                            }
                            (Shape::Ring { .. }, Site::Drive) => false,
                            (Shape::DirectionPair { .. }, _) => true,
                        }
                });
                let Some(site) = site else {
                    return Err(V::MemberShape);
                };
                let fresh = match record.shape {
                    Shape::Ring { .. } => {
                        let bit = 1u8 << sub_idx;
                        let fresh = ring_channels[bundle.index()] & bit == 0;
                        ring_channels[bundle.index()] |= bit;
                        fresh
                    }
                    Shape::DirectionPair { .. } => pair_coordinates.insert((bundle, sub_idx, site)),
                };
                if !fresh {
                    return Err(V::DuplicateMember);
                }
                members[bundle.index()] += 1;
            }
        }
        if members.contains(&0) {
            return Err(V::EmptyBundle);
        }
        Ok(())
    }

    // ── Internal helpers ────────────────────────────────────────────────────

    fn compute_seed_rows_mut(&mut self) -> Option<&mut Vec<Vec<Option<f32>>>> {
        self.birth_seeds
            .as_mut()
            .map(|BirthSeeds::Rows { compute, .. }| compute)
    }

    fn clear_seed(&mut self, occurrence: EdgeOccurrence) {
        if let Some(seeds) = &mut self.birth_seeds {
            seeds.row_mut(occurrence.surface)[occurrence.index] = None;
        }
    }

    fn edge_slot(&mut self, occurrence: EdgeOccurrence) -> &mut GraphEdge {
        match occurrence.surface {
            EdgeSurface::ComputeInput(i) => &mut self.compute_nodes.0[i].inputs.0[occurrence.index],
            EdgeSurface::SinkInput(i) => &mut self.output_sinks.0[i].inputs.0[occurrence.index],
        }
    }

    /// I11: when rows are present there is one per compute node and one per
    /// sink, each sized to that container's inputs, and one bundle slot per
    /// bundle record.
    fn debug_assert_seed_shape(&self) {
        if let Some(BirthSeeds::Rows {
            compute,
            sinks,
            bundles,
        }) = &self.birth_seeds
        {
            debug_assert_eq!(
                bundles.len(),
                self.bundles.len(),
                "one seed slot per bundle record"
            );
            debug_assert_eq!(
                compute.len(),
                self.compute_nodes.len(),
                "one seed row per compute node"
            );
            debug_assert_eq!(
                sinks.len(),
                self.output_sinks.len(),
                "one seed row per sink"
            );
            debug_assert!(
                compute
                    .iter()
                    .zip(self.compute_nodes.iter())
                    .all(|(row, node)| row.len() == node.inputs.len())
                    && sinks
                        .iter()
                        .zip(self.output_sinks.iter())
                        .all(|(row, sink)| row.len() == sink.inputs.len()),
                "each seed row sized to its container's inputs"
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
    /// Seed the compute rows `compute` and an empty row per sink, each
    /// sized to its sink's inputs (a fixture).
    pub(crate) fn seed_compute_rows(&mut self, compute: Vec<Vec<Option<f32>>>) {
        let sinks = self
            .output_sinks
            .iter()
            .map(|sink| vec![None; sink.inputs.len()])
            .collect();
        let bundles = vec![None; self.bundles.len()];
        self.seed_birth(BirthSeeds::Rows {
            compute,
            sinks,
            bundles,
        });
    }

    /// Fill the bundle table to its `u16::MAX`-record limit with memberless
    /// records, so [`Self::can_host_bundle`] is false on an otherwise full
    /// sink catalog (a preflight fixture; such a def fails `EmptyBundle`).
    pub(crate) fn exhaust_bundle_capacity(&mut self) {
        let record = Bundle::new(Shape::Ring { ring_ref: 0 }, Kind::Move, 0);
        self.bundles = vec![record; usize::from(u16::MAX)];
    }

    /// Drop the last bundle record (a capacity fixture, after
    /// [`Self::exhaust_bundle_capacity`]).
    pub(crate) fn drop_last_bundle_record(&mut self) {
        self.bundles.pop();
    }
}

#[cfg(test)]
#[path = "container_tests.rs"]
mod tests;
