//! Learning (T25.F06, T25.F09, T25.F12): the learning configuration every
//! learnable number runs, and the learning layout, the one genome-owned
//! analysis of what learns in a graph.

use super::cgp::{BundleId, CgpGraphBackendDef, EdgeOccurrence, Scope};
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

/// One edge's learning attributes (T25.F06), stored on the `GraphEdge`.
///
/// Governs both pure Hebbian and reward-modulated (three-factor) learning.
/// When `modulation` is `None`, this is pure Hebbian learning. When `Some`,
/// activity without the learning-rate factor replaces the tick contribution to an eligibility trace; weight
/// updates are deferred to the reward-modulated learning pass.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PlasticityConfig {
    pub rule: HebbianRule,
    /// Learning rate, applied once per weight update, in [0.0, 1.0] (I1).
    pub learning_rate: f32,
    /// Symmetric weight clamp magnitude, in [0.01, 10.0] (I1).
    pub weight_clamp: f32,
    /// If true, offspring inherit learned weights (Lamarckian); otherwise
    /// offspring start from genome birth weights (Darwinian).
    pub lamarckian: bool,
    /// Reward modulation config. `None` = pure Hebbian (default).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modulation: Option<RewardModulationConfig>,
    /// Learned-weight retention per world tick, in [0.0, 1.0] (T25.F09): the
    /// learned own quantity relaxes `b + ρ·(x − b)` toward its birth value;
    /// 1 is an exact no-op.
    pub retention: f32,
}

impl PlasticityConfig {
    /// I1 for plasticity (T25.F06, T25.F09): every value finite, the learning
    /// rate, trace decay and retention in `[0, 1]` and the weight clamp in
    /// `[0.01, 10]`. The
    /// enable draws and perturbation laws keep it by construction; the load
    /// boundary checks it ([`cgp::validate_graph`]).
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

/// Whether a learned-weight retention (a [`PlasticityConfig`]'s or a
/// bundle's) decays: below 1, compared exactly so that 1 is an exact no-op
/// (T25.F09 invariant 3).
#[inline]
#[must_use]
pub(crate) fn retention_decays(retention: f32) -> bool {
    retention < 1.0
}

// ── Learning layout ─────────────────────────────────────────────────────────

/// Where an edge learner's configuration is stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LearnerSource {
    /// A lone edge's own configuration.
    Lone,
    /// A member's own configuration.
    Own,
}

/// A member's place in its bundle: the bundle and the member's pattern.
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
    /// The effective configuration the learner runs.
    pub learning: PlasticityConfig,
    pub source: LearnerSource,
    /// The member's bundle and pattern, `None` for a lone edge.
    pub member: Option<MemberTerm>,
}

/// One member whose activity feeds its bundle's scale, with the
/// configuration its contribution runs (its own, until T25.F12 commit 2).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Contributor {
    pub term: MemberTerm,
    pub learning: PlasticityConfig,
}

/// One bundle's learnable scale.
#[derive(Debug, Clone, PartialEq)]
pub struct ScaleLearner {
    pub bundle: BundleId,
    /// The scale's own configuration: its clamp, retention and inheritance
    /// (`lamarckian`) bound, decay and seed the scale.
    pub learning: PlasticityConfig,
    /// The members feeding the scale, in canonical order (possibly none).
    pub contributors: Vec<Contributor>,
    /// Every member whose weight reads the learned scale, in canonical
    /// order.
    pub members: Vec<MemberTerm>,
}

/// One graph's learning layout (T25.F12 invariant 4): every edge learner
/// in canonical order and every scale learner in `BundleId` order. The one
/// derivation of what learns; its emptiness is the one definition of
/// "carries learning".
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LearningLayout {
    pub edges: Vec<EdgeLearner>,
    pub scales: Vec<ScaleLearner>,
}

impl LearningLayout {
    /// The layout of `def` on a node declaring `input_refs`.
    ///
    /// Today's model: a plastic lone edge or `Edge`-scope member learns its
    /// own weight; a `Group`-scope plastic member contributes to its
    /// bundle's scale with its own configuration and learns no own weight;
    /// a bundle has a scale learner only with such a contributor, bounded
    /// by the smallest contributor clamp and taking the record's retention
    /// and `inherit_scale`.
    #[must_use]
    pub fn of(def: &CgpGraphBackendDef, input_refs: &[InputReference]) -> Self {
        let mut edges = Vec::new();
        let mut contributors: Vec<Vec<Contributor>> = vec![Vec::new(); def.bundles().len()];
        for occurrence in def.edge_sites() {
            let edge = def.edge_at(occurrence);
            let Some(learning) = edge.plasticity() else {
                continue;
            };
            let member = edge.membership().map(|membership| {
                (
                    membership.scope,
                    MemberTerm {
                        occurrence,
                        bundle: membership.bundle,
                        pattern: def.pattern(occurrence, input_refs),
                    },
                )
            });
            match member {
                Some((Scope::Group, term)) => {
                    contributors[term.bundle.index()].push(Contributor { term, learning });
                }
                Some((Scope::Edge, term)) => edges.push(EdgeLearner {
                    occurrence,
                    learning,
                    source: LearnerSource::Own,
                    member: Some(term),
                }),
                None => edges.push(EdgeLearner {
                    occurrence,
                    learning,
                    source: LearnerSource::Lone,
                    member: None,
                }),
            }
        }
        let scales = def
            .indexed_bundles()
            .zip(contributors)
            .filter_map(|((bundle, record), contributors)| {
                let first = contributors.first()?.learning;
                let bound = contributors
                    .iter()
                    .map(|contributor| contributor.learning.weight_clamp)
                    .fold(first.weight_clamp, f32::min);
                Some(ScaleLearner {
                    bundle,
                    learning: PlasticityConfig {
                        weight_clamp: bound,
                        retention: record.retention(),
                        lamarckian: record.inherit_scale(),
                        ..first
                    },
                    contributors,
                    members: def
                        .members(bundle)
                        .map(|occurrence| MemberTerm {
                            occurrence,
                            bundle,
                            pattern: def.pattern(occurrence, input_refs),
                        })
                        .collect(),
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
