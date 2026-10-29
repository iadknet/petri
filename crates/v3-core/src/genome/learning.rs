//! Learning (T25.F06, T25.F09, T25.F12): the one learning type every
//! learnable number runs, a member's policy, and the learning layout, the
//! one genome-owned analysis of what learns in a graph.

use super::cgp::{BundleId, CgpGraphBackendDef, EdgeOccurrence, EdgeRole, EdgeSurface};
use crate::contracts::{InputReference, OutcomeChannel};

/// Hebbian learning rule variant controlling how edge weights adapt at runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum HebbianRule {
    /// dw = eta * pre * post
    Classic,
    /// dw = eta * post * (pre - w * post)  (normalizing)
    Oja,
    /// dw = -eta * pre * post
    AntiHebb,
    /// dw = eta * (pre - 0.5) * (post - 0.5)
    Covariance,
}

impl HebbianRule {
    /// Every rule, in declaration order.
    pub const ALL: [Self; 4] = [Self::Classic, Self::Oja, Self::AntiHebb, Self::Covariance];
}

/// Reward modulation config for three-factor plasticity.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RewardModulationConfig {
    pub reward_source: OutcomeChannel,
    /// Eligibility retention per world tick, in [0.0, 1.0] (I1).
    pub trace_decay: f32,
}

/// One learnable number's learning (T25.F06, T25.F12): stored on a lone
/// edge, on a member as its override ([`MemberPolicy::Own`]) and in a
/// bundle's two slots (`scale_learning`, `member_default`).
///
/// Governs both pure Hebbian and reward-modulated (three-factor) learning.
/// When `modulation` is `None`, this is pure Hebbian learning. When `Some`,
/// activity without the learning-rate factor replaces the tick contribution
/// to an eligibility trace; updates are deferred to the reward-modulated
/// learning pass.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Learning {
    pub rule: HebbianRule,
    /// Learning rate, applied once per update, in [0.0, 1.0] (I1).
    pub learning_rate: f32,
    /// Symmetric clamp magnitude of the learned value, in [0.01, 10.0] (I1).
    pub weight_clamp: f32,
    /// If true, offspring inherit the learned value (Lamarckian); otherwise
    /// offspring start from the genomic birth value (Darwinian).
    pub lamarckian: bool,
    /// Reward modulation config. `None` = pure Hebbian (default).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modulation: Option<RewardModulationConfig>,
    /// Learned-value retention per world tick, in [0.0, 1.0] (T25.F09): the
    /// learned value relaxes `b + ρ·(x − b)` toward its birth value; 1 is an
    /// exact no-op.
    pub retention: f32,
}

impl Learning {
    /// I1 for learning (T25.F06, T25.F09): every value finite, the learning
    /// rate, trace decay and retention in `[0, 1]` and the clamp in
    /// `[0.01, 10]`. The enable draws and perturbation laws keep it by
    /// construction; the load boundary checks it
    /// ([`super::cgp::validate_graph`]).
    #[must_use]
    pub fn in_range(&self) -> bool {
        (0.0..=1.0).contains(&self.learning_rate)
            && (0.01..=10.0).contains(&self.weight_clamp)
            && (0.0..=1.0).contains(&self.retention)
            && self
                .modulation
                .is_none_or(|modulation| (0.0..=1.0).contains(&modulation.trace_decay))
    }
}

/// Whether a learned-value retention decays: below 1, compared exactly so
/// that 1 is an exact no-op (T25.F09 invariant 3).
#[inline]
#[must_use]
pub(crate) fn retention_decays(retention: f32) -> bool {
    retention < 1.0
}

// ── Member policy ───────────────────────────────────────────────────────────

/// How a member's modifier learns (T25.F12 invariant 2): through its
/// bundle's `member_default` (`Inherit`), not at all (`Static`), or by its
/// own override (`Own`).
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum MemberPolicy {
    Inherit,
    Static,
    Own(Learning),
}

impl MemberPolicy {
    /// The member's effective learning under a bundle whose
    /// `member_default` is `member_default`: the override for `Own`, the
    /// default for `Inherit`, none for `Static`. Also the lone learning a
    /// detached member keeps (invariant 8).
    #[must_use]
    pub const fn effective(self, member_default: Option<Learning>) -> Option<Learning> {
        match self {
            Self::Own(learning) => Some(learning),
            Self::Inherit => member_default,
            Self::Static => None,
        }
    }

    /// The override an `Own` member stores; `None` otherwise.
    #[must_use]
    pub const fn stored(self) -> Option<Learning> {
        match self {
            Self::Own(learning) => Some(learning),
            Self::Inherit | Self::Static => None,
        }
    }

    /// Stable string key used by transport and bench boundaries.
    #[must_use]
    pub const fn as_key(self) -> &'static str {
        match self {
            Self::Inherit => "inherit",
            Self::Static => "static",
            Self::Own(_) => "own",
        }
    }
}

// ── Learning layout ─────────────────────────────────────────────────────────

/// Where an edge learner's effective learning is stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LearnerSource {
    /// A lone edge's learning.
    Lone,
    /// A member's override.
    Own,
    /// An `Inherit` member's bundle's `member_default`.
    MemberDefault,
}

impl LearnerSource {
    /// Every source, in index order.
    pub const ALL: [Self; 3] = [Self::Lone, Self::Own, Self::MemberDefault];

    /// This source's position in [`Self::ALL`].
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Lone => 0,
            Self::Own => 1,
            Self::MemberDefault => 2,
        }
    }

    /// Stable string key used by transport, telemetry and bench boundaries.
    #[must_use]
    pub const fn as_key(self) -> &'static str {
        match self {
            Self::Lone => "lone",
            Self::Own => "own",
            Self::MemberDefault => "member_default",
        }
    }
}

/// A member's place in its bundle: its occurrence, its bundle and its
/// pattern.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MemberTerm {
    pub occurrence: EdgeOccurrence,
    pub bundle: BundleId,
    pub pattern: f32,
}

/// One learnable edge number: a lone edge's weight or a member's modifier.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EdgeLearner {
    pub occurrence: EdgeOccurrence,
    /// The effective learning the learner runs.
    pub learning: Learning,
    pub source: LearnerSource,
    /// The member's bundle and pattern, `None` for a lone edge.
    pub member: Option<MemberTerm>,
}

/// One bundle's learnable scale.
#[derive(Debug, Clone, PartialEq)]
pub struct ScaleLearner {
    pub bundle: BundleId,
    /// The bundle's `scale_learning`.
    pub learning: Learning,
    /// The members whose activity feeds the scale (`credits`), in canonical
    /// order; possibly none.
    pub contributors: Vec<MemberTerm>,
    /// Every member, each weighing through the learned scale, in canonical
    /// order.
    pub members: Vec<MemberTerm>,
}

/// One graph's learning layout (T25.F12 invariant 4): every edge learner
/// in canonical order and every scale learner in `BundleId` order. The only
/// code that resolves `Inherit` or reads a policy or `credits` to decide
/// what learns; its emptiness is the one definition of "carries learning".
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LearningLayout {
    pub edges: Vec<EdgeLearner>,
    pub scales: Vec<ScaleLearner>,
}

impl LearningLayout {
    /// The layout of `def` on a node declaring `input_refs`: an edge
    /// learner for every lone edge with learning and every member with an
    /// effective learning; a scale learner for every bundle with
    /// `scale_learning`, its contributors the members with `credits`.
    #[must_use]
    pub fn of(def: &CgpGraphBackendDef, input_refs: &[InputReference]) -> Self {
        let bundles = def.bundles();
        let mut edges = Vec::new();
        let mut members: Vec<Vec<MemberTerm>> = vec![Vec::new(); bundles.len()];
        let mut contributors: Vec<Vec<MemberTerm>> = vec![Vec::new(); bundles.len()];
        for occurrence in def.edge_sites() {
            match def.edge_at(occurrence).role() {
                EdgeRole::Lone(Some(learning)) => edges.push(EdgeLearner {
                    occurrence,
                    learning,
                    source: LearnerSource::Lone,
                    member: None,
                }),
                EdgeRole::Lone(None) => {}
                EdgeRole::Member(membership) => {
                    let record = &bundles[membership.bundle.index()];
                    let effective = membership.policy.effective(record.member_default());
                    let learns_scale = record.scale_learning().is_some();
                    if effective.is_none() && !learns_scale {
                        continue;
                    }
                    let term = MemberTerm {
                        occurrence,
                        bundle: membership.bundle,
                        pattern: def.pattern(occurrence, input_refs),
                    };
                    if let Some(learning) = effective {
                        let source = match membership.policy {
                            MemberPolicy::Own(_) => LearnerSource::Own,
                            MemberPolicy::Inherit | MemberPolicy::Static => {
                                LearnerSource::MemberDefault
                            }
                        };
                        edges.push(EdgeLearner {
                            occurrence,
                            learning,
                            source,
                            member: Some(term),
                        });
                    }
                    if learns_scale {
                        members[membership.bundle.index()].push(term);
                        if membership.credits {
                            contributors[membership.bundle.index()].push(term);
                        }
                    }
                }
            }
        }
        let scales = def
            .indexed_bundles()
            .zip(members.into_iter().zip(contributors))
            .filter_map(|((bundle, record), (members, contributors))| {
                Some(ScaleLearner {
                    bundle,
                    learning: record.scale_learning()?,
                    contributors,
                    members,
                })
            })
            .collect();
        Self { edges, scales }
    }

    /// Whether nothing learns: the one definition of "carries no learning".
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.edges.is_empty() && self.scales.is_empty()
    }

    /// The scale learner of `bundle`, by position in [`Self::scales`].
    #[must_use]
    pub fn scale_of(&self, bundle: BundleId) -> Option<usize> {
        self.scales
            .binary_search_by_key(&bundle, |scale| scale.bundle)
            .ok()
    }
}

// ── Inheritance (T29.F01) ───────────────────────────────────────────────────

/// One learnable number of a Graph node revision: an edge's own quantity
/// (a lone edge's weight or a member's modifier) or a bundle's scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LearnableKey {
    Edge(EdgeOccurrence),
    Scale(BundleId),
}

/// A learnable identity: a node index and a key in the genome as it stood
/// when its values were captured.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LearnableId {
    pub node: usize,
    pub key: LearnableKey,
}

/// Where a post-edit learnable's inherited value comes from, naming keys of
/// the node revision before the edit.
#[derive(Debug, Clone, PartialEq)]
pub enum Origin {
    /// The same learnable, possibly moved.
    Keep(LearnableKey),
    /// A new learnable copying its source's value.
    Copy(LearnableKey),
    /// A detached member's own quantity: what it weighed at the parent.
    Compose(Box<Composed>),
    /// Nothing inherited.
    Fresh,
}

/// A detach's composition (T25.F07 invariant 6, T25.F11 invariant 4): the
/// member's modifier and its bundle's scale, each falling back to its
/// genomic value, composed at the member's pattern.
#[derive(Debug, Clone, PartialEq)]
pub struct Composed {
    pub modifier: Origin,
    pub scale: Origin,
    pub modifier_genomic: f32,
    pub scale_genomic: f32,
    pub pattern: f32,
}

impl Origin {
    /// This origin as a copy: what a duplicated learnable inherits.
    #[must_use]
    pub fn copied(self) -> Self {
        match self {
            Self::Keep(key) | Self::Copy(key) => Self::Copy(key),
            other => other,
        }
    }

    /// This origin, which names keys of an intermediate revision, through
    /// `earlier`, the mapping into that revision.
    #[must_use]
    pub fn through(&self, earlier: &GraphMapping) -> Self {
        match self {
            Self::Keep(key) => earlier.origin(*key),
            Self::Copy(key) => earlier.origin(*key).copied(),
            Self::Compose(composed) => {
                let modifier = composed.modifier.through(earlier);
                let scale = composed.scale.through(earlier);
                if modifier == Self::Fresh && scale == Self::Fresh {
                    return Self::Fresh;
                }
                Self::Compose(Box::new(Composed {
                    modifier,
                    scale,
                    ..**composed
                }))
            }
            Self::Fresh => Self::Fresh,
        }
    }

    /// The inherited value under `seeds`, the source revision's values:
    /// `Keep` and `Copy` the source's seed, `Compose` the composition when
    /// either seed is present, `Fresh` none.
    #[must_use]
    pub fn value(&self, seeds: &NodeSeeds) -> Option<f32> {
        match self {
            Self::Keep(key) | Self::Copy(key) => seeds.get(key).copied(),
            Self::Compose(composed) => {
                let modifier = composed.modifier.value(seeds);
                let scale = composed.scale.value(seeds);
                (modifier.is_some() || scale.is_some()).then(|| {
                    super::cgp::compose(
                        modifier.unwrap_or(composed.modifier_genomic),
                        scale.unwrap_or(composed.scale_genomic),
                        composed.pattern,
                    )
                })
            }
            Self::Fresh => None,
        }
    }
}

/// One origin per learnable of a Graph node after an edit: per compute node
/// and per sink one per input edge, and one per bundle record.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OriginRows {
    pub(crate) compute: Vec<Vec<Origin>>,
    pub(crate) sinks: Vec<Vec<Origin>>,
    pub(crate) bundles: Vec<Origin>,
}

impl OriginRows {
    /// Every learnable of `def` keeping its identity.
    #[must_use]
    pub fn identity(def: &CgpGraphBackendDef) -> Self {
        let keep = |surface: fn(usize) -> EdgeSurface, i: usize, len: usize| {
            (0..len)
                .map(|index| {
                    Origin::Keep(LearnableKey::Edge(EdgeOccurrence::new(surface(i), index)))
                })
                .collect()
        };
        Self {
            compute: def
                .compute_nodes()
                .iter()
                .enumerate()
                .map(|(i, node)| keep(EdgeSurface::ComputeInput, i, node.inputs.len()))
                .collect(),
            sinks: def
                .output_sinks()
                .iter()
                .enumerate()
                .map(|(i, sink)| keep(EdgeSurface::SinkInput, i, sink.inputs.len()))
                .collect(),
            bundles: def
                .indexed_bundles()
                .map(|(id, _)| Origin::Keep(LearnableKey::Scale(id)))
                .collect(),
        }
    }

    /// The row of `surface`.
    pub(crate) fn row_mut(&mut self, surface: EdgeSurface) -> &mut Vec<Origin> {
        match surface {
            EdgeSurface::ComputeInput(i) => &mut self.compute[i],
            EdgeSurface::SinkInput(i) => &mut self.sinks[i],
        }
    }

    /// The origin of `key`; `Fresh` past the rows.
    fn origin(&self, key: LearnableKey) -> Origin {
        let found = match key {
            LearnableKey::Edge(at) => match at.surface {
                EdgeSurface::ComputeInput(i) => self.compute.get(i),
                EdgeSurface::SinkInput(i) => self.sinks.get(i),
            }
            .and_then(|row| row.get(at.index)),
            LearnableKey::Scale(bundle) => self.bundles.get(bundle.index()),
        };
        debug_assert!(found.is_some(), "an origin names a key of its revision");
        found.cloned().unwrap_or(Origin::Fresh)
    }

    /// Every post-edit key with its origin.
    fn entries(&self) -> impl Iterator<Item = (LearnableKey, &Origin)> {
        fn edges(
            rows: &[Vec<Origin>],
            surface: fn(usize) -> EdgeSurface,
        ) -> impl Iterator<Item = (LearnableKey, &Origin)> {
            rows.iter().enumerate().flat_map(move |(i, row)| {
                row.iter().enumerate().map(move |(index, origin)| {
                    (
                        LearnableKey::Edge(EdgeOccurrence::new(surface(i), index)),
                        origin,
                    )
                })
            })
        }
        edges(&self.compute, EdgeSurface::ComputeInput)
            .chain(edges(&self.sinks, EdgeSurface::SinkInput))
            .chain(self.bundles.iter().enumerate().map(|(index, origin)| {
                let id = BundleId::new(u16::try_from(index).expect("bundles below u16::MAX"));
                (LearnableKey::Scale(id), origin)
            }))
    }

    fn map_origins(&mut self, mut f: impl FnMut(&Origin) -> Origin) {
        for origin in self
            .compute
            .iter_mut()
            .chain(self.sinks.iter_mut())
            .flatten()
            .chain(self.bundles.iter_mut())
        {
            *origin = f(origin);
        }
    }
}

/// One Graph node's inheritance mapping across an edit: where every
/// learnable of the node after the edit inherits from, among the learnables
/// of the node before it.
#[derive(Debug, Clone, PartialEq)]
pub enum GraphMapping {
    /// Every learnable keeps its identity: the edit moved none.
    Identity,
    /// Every learnable copies the same key of its source (a whole-node copy).
    Copied,
    /// An explicit origin per learnable.
    Rows(OriginRows),
}

impl Default for GraphMapping {
    fn default() -> Self {
        Self::Identity
    }
}

impl GraphMapping {
    /// The origin of post-edit `key`.
    #[must_use]
    pub fn origin(&self, key: LearnableKey) -> Origin {
        match self {
            Self::Identity => Origin::Keep(key),
            Self::Copied => Origin::Copy(key),
            Self::Rows(rows) => rows.origin(key),
        }
    }

    /// `self ∘ earlier`: the mapping across `earlier`'s edit and then this
    /// one, substituting `earlier`'s origins into this one's.
    #[must_use]
    pub fn after(self, earlier: &Self) -> Self {
        match (self, earlier) {
            (later, Self::Identity) => later,
            (Self::Identity, earlier) => earlier.clone(),
            (Self::Copied, Self::Copied) => Self::Copied,
            (Self::Copied, Self::Rows(rows)) => {
                let mut rows = rows.clone();
                rows.map_origins(|origin| origin.clone().copied());
                Self::Rows(rows)
            }
            (Self::Rows(mut rows), earlier) => {
                rows.map_origins(|origin| origin.through(earlier));
                Self::Rows(rows)
            }
        }
    }

    /// The node's inherited values after the edit, from `seeds`, its values
    /// before it.
    #[must_use]
    pub fn apply(&self, seeds: &NodeSeeds) -> NodeSeeds {
        match self {
            Self::Identity | Self::Copied => seeds.clone(),
            Self::Rows(rows) => rows
                .entries()
                .filter_map(|(key, origin)| origin.value(seeds).map(|value| (key, value)))
                .collect(),
        }
    }
}

/// One Graph node's inherited values, keyed by learnable key.
pub type NodeSeeds = std::collections::BTreeMap<LearnableKey, f32>;

/// The inherited learned values a birth carries (T29.F01 invariant 2):
/// each initialized learner's value keyed by learnable identity.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Seeds {
    nodes: std::collections::BTreeMap<usize, NodeSeeds>,
}

impl Seeds {
    /// Record `value` at `id`.
    pub fn insert(&mut self, id: LearnableId, value: f32) {
        self.nodes.entry(id.node).or_default().insert(id.key, value);
    }

    /// The value at `id`, if any.
    #[must_use]
    pub fn get(&self, id: LearnableId) -> Option<f32> {
        self.nodes.get(&id.node)?.get(&id.key).copied()
    }

    /// Node `node`'s values, if any.
    #[must_use]
    pub fn node(&self, node: usize) -> Option<&NodeSeeds> {
        self.nodes.get(&node)
    }

    /// Whether no value is carried.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.values().all(NodeSeeds::is_empty)
    }

    /// Every carried value with its identity, in identity order.
    pub fn iter(&self) -> impl Iterator<Item = (LearnableId, f32)> + '_ {
        self.nodes.iter().flat_map(|(&node, values)| {
            values
                .iter()
                .map(move |(&key, &value)| (LearnableId { node, key }, value))
        })
    }
}

impl FromIterator<(LearnableId, f32)> for Seeds {
    fn from_iter<I: IntoIterator<Item = (LearnableId, f32)>>(iter: I) -> Self {
        let mut seeds = Self::default();
        for (id, value) in iter {
            seeds.insert(id, value);
        }
        seeds
    }
}

/// Where one node of the genome after an edit inherits from.
#[derive(Debug, Clone, PartialEq)]
pub enum NodeInheritance {
    /// From node `node` of the genome before the edit, through `graph`.
    From { node: usize, graph: GraphMapping },
    /// Nothing inherited: a new node or a new backend.
    Fresh,
}

/// A genome edit's inheritance mapping (T29.F01 invariant 4): for every
/// node of the genome after the edit, the node it inherits from and that
/// node's mapping. `Identity` keeps every node and every learnable: an edit
/// that changes no node count and moves no learnable.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum InheritanceMapping {
    #[default]
    Identity,
    Nodes(Vec<NodeInheritance>),
}

impl InheritanceMapping {
    /// An edit inside node `node` of a genome of `node_count` nodes, moving
    /// that node's learnables by `graph`.
    #[must_use]
    pub fn within(node: usize, node_count: usize, graph: GraphMapping) -> Self {
        if graph == GraphMapping::Identity {
            return Self::Identity;
        }
        let mut nodes = Self::kept(node_count);
        nodes[node] = NodeInheritance::From { node, graph };
        Self::Nodes(nodes)
    }

    /// Every node of a genome of `node_count` nodes kept.
    #[must_use]
    pub fn kept(node_count: usize) -> Vec<NodeInheritance> {
        (0..node_count)
            .map(|node| NodeInheritance::From {
                node,
                graph: GraphMapping::Identity,
            })
            .collect()
    }

    /// Where post-edit node `node` inherits from.
    #[must_use]
    pub fn node(&self, node: usize) -> NodeInheritance {
        match self {
            Self::Identity => NodeInheritance::From {
                node,
                graph: GraphMapping::Identity,
            },
            Self::Nodes(nodes) => {
                debug_assert!(node < nodes.len(), "a mapping covers every node");
                nodes.get(node).cloned().unwrap_or(NodeInheritance::Fresh)
            }
        }
    }

    /// `self ∘ earlier`: the mapping across `earlier`'s edit and then this
    /// one.
    #[must_use]
    pub fn after(self, earlier: &Self) -> Self {
        match (self, earlier) {
            (later, Self::Identity) => later,
            (Self::Identity, earlier) => earlier.clone(),
            (Self::Nodes(nodes), earlier) => Self::Nodes(
                nodes
                    .into_iter()
                    .map(|later| match later {
                        NodeInheritance::Fresh => NodeInheritance::Fresh,
                        NodeInheritance::From { node, graph } => match earlier.node(node) {
                            NodeInheritance::Fresh => NodeInheritance::Fresh,
                            NodeInheritance::From {
                                node,
                                graph: earlier,
                            } => NodeInheritance::From {
                                node,
                                graph: graph.after(&earlier),
                            },
                        },
                    })
                    .collect(),
            ),
        }
    }

    /// The inherited values after the edit, from `seeds`, the values before
    /// it: today's values, bitwise.
    #[must_use]
    pub fn apply(&self, seeds: &Seeds) -> Seeds {
        match self {
            Self::Identity => seeds.clone(),
            Self::Nodes(nodes) => {
                let mut applied = Seeds::default();
                for (post, inheritance) in nodes.iter().enumerate() {
                    let NodeInheritance::From { node, graph } = inheritance else {
                        continue;
                    };
                    let Some(values) = seeds.node(*node) else {
                        continue;
                    };
                    let values = graph.apply(values);
                    if !values.is_empty() {
                        applied.nodes.insert(post, values);
                    }
                }
                applied
            }
        }
    }
}
