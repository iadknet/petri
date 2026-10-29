//! Learning (T25.F06, T25.F09, T25.F12): the one learning type every
//! learnable number runs, a member's policy, and the learning layout, the
//! one genome-owned analysis of what learns in a graph.

use super::cgp::{BundleId, CgpGraphBackendDef, EdgeOccurrence, EdgeRole};
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
