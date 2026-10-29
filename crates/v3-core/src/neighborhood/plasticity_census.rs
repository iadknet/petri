//! The plasticity census: how many creatures carry plasticity, of which kind,
//! and how far learning has moved those weights from the genome.
//!
//! Reachable Graph nodes only. A carrier has a learner on a reachable node
//! (T25.F12 invariant 11). An edge learner (a lone edge's weight or a
//! member's modifier, T25.F06, T25.F12) is keyed by learning pathway (pure
//! Hebbian or reward-modulated) and surface (compute or sink), and counted
//! separately by rule, by inheritance (its effective `lamarckian`) and by
//! source (`lone`, `own`, `member_default`). The census reads the learned
//! state's view: edge drift is `|learned - birth|` over initialized
//! learners (lifetime learning, T25.F09), and an initialized learner sits
//! at its clamp when it reaches the clamp, which every write saturates to
//! exactly. Each scale learner counts once per bundle, by its
//! `scale_learning`'s `lamarckian`, with its drift `|learned - birth|` and
//! whether it sits at its clamp. Edges and scales also count by retention
//! bin ([`RETENTION_KEYS`], T25.F09).
//!
//! Pure structure and state: it reads genomes, cached reachability and
//! learned state and executes no brain.

use crate::genome::cgp::EdgeSurface;
use crate::genome::learning::LearnerSource;
use crate::genome::HebbianRule;
use crate::runtime::plasticity::{LearnedState, RULES};

/// The `petri.plasticity_rule` key of `rule`.
#[must_use]
pub const fn rule_key(rule: HebbianRule) -> &'static str {
    match rule {
        HebbianRule::Classic => "classic",
        HebbianRule::Oja => "oja",
        HebbianRule::AntiHebb => "anti_hebb",
        HebbianRule::Covariance => "covariance",
    }
}

/// The `petri.plasticity_learning` keys, by pathway index (0 pure Hebbian,
/// 1 reward-modulated).
pub const LEARNING_KEYS: [&str; 2] = ["hebbian", "reward_modulated"];

/// The `petri.surface` keys, by surface index (0 compute, 1 sink).
pub const SURFACE_KEYS: [&str; 2] = ["compute", "sink"];

/// The `petri.plasticity_inheritance` keys, by inheritance index (0
/// Darwinian, 1 Lamarckian). A scale's inheritance is its
/// `scale_learning`'s `lamarckian`.
pub const INHERITANCE_KEYS: [&str; 2] = ["darwinian", "lamarckian"];

/// The `petri.plasticity_source` keys, by [`LearnerSource::index`].
pub const SOURCE_KEYS: [&str; 3] = [
    LearnerSource::Lone.as_key(),
    LearnerSource::Own.as_key(),
    LearnerSource::MemberDefault.as_key(),
];

/// The `petri.plasticity_retention` keys, by retention bin (T25.F09): `none`
/// for a retention of exactly 1, else the bin's upper bound on the deficit
/// `d = 1 − ρ`: `(0, 10⁻³)`, `[10⁻³, 10⁻²)`, `[10⁻², 10⁻¹)`, `[10⁻¹, 1]`.
pub const RETENTION_KEYS: [&str; 5] = ["none", "1e-3", "1e-2", "1e-1", "1"];

/// The retention bin of a retention `retention ∈ [0, 1]`
/// ([`RETENTION_KEYS`]). The deficit is exact in `f64`.
#[must_use]
pub fn retention_index(retention: f32) -> usize {
    // Exact: only a retention of 1 is the no-op bin.
    if retention == 1.0 {
        return 0;
    }
    let deficit = 1.0 - f64::from(retention);
    if deficit < 1e-3 {
        1
    } else if deficit < 1e-2 {
        2
    } else if deficit < 1e-1 {
        3
    } else {
        4
    }
}

/// One retention bin's edges or scale slots over a population (T25.F09).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct RetentionCell {
    /// Every entry of the bin.
    pub count: u64,
    /// Entries with a learned own quantity or an initialized scale.
    pub initialized: u64,
    /// Sum of `|learned - birth|` over the initialized entries.
    pub drift_sum: f64,
    /// Initialized entries at their clamp or bound.
    pub at_bound: u64,
}

impl RetentionCell {
    /// Count one entry, with its learned value when initialized.
    fn add(&mut self, learned: Option<f32>, birth: f32, bound: f32) {
        self.count += 1;
        let Some(value) = learned else {
            return;
        };
        self.initialized += 1;
        add_drift(&mut self.drift_sum, value, birth);
        self.at_bound += u64::from(value.abs() >= bound);
    }
}

/// Add `|value - birth|` to `sum`; a non-finite drift would poison the
/// population sum.
fn add_drift(sum: &mut f64, value: f32, birth: f32) {
    let drift = f64::from((value - birth).abs());
    if drift.is_finite() {
        *sum += drift;
    }
}

/// One inheritance cell's learned scales over a population.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct LearnedScales {
    pub scales: u64,
    /// Sum of `|learned - birth|` over the initialized scale slots.
    pub drift_sum: f64,
    /// Initialized scale slots whose learned scale sits at their bound.
    pub at_bound: u64,
}

/// One (pathway, surface) cell's plastic edges over a population.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PlasticEdges {
    pub edges: u64,
    /// Sum of `|learned - birth|` over the initialized edges.
    pub drift_sum: f64,
    /// Initialized edges whose learned weight sits at the edge's clamp.
    pub at_clamp: u64,
}

/// The census of one population.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PlasticityCensus {
    pub creatures: u64,
    /// Creatures with at least one plastic edge on a reachable node.
    pub carriers: u64,
    /// Indexed `[pathway][surface]` ([`LEARNING_KEYS`], [`SURFACE_KEYS`]).
    pub edges: [[PlasticEdges; 2]; 2],
    /// Edge counts per rule, in [`RULES`] order.
    pub edges_by_rule: [u64; 4],
    /// Edge counts per inheritance, in [`INHERITANCE_KEYS`] order.
    pub edges_by_inheritance: [u64; 2],
    /// Edge counts per source, in [`SOURCE_KEYS`] order.
    pub edges_by_source: [u64; 3],
    /// Scale slots per inheritance, in [`INHERITANCE_KEYS`] order.
    pub scales: [LearnedScales; 2],
    /// Reachable plastic edges per retention bin, in [`RETENTION_KEYS`]
    /// order (T25.F09).
    pub edges_by_retention: [RetentionCell; 5],
    /// Scale slots per their bundle's retention bin, in [`RETENTION_KEYS`]
    /// order (T25.F09).
    pub scales_by_retention: [RetentionCell; 5],
}

impl PlasticityCensus {
    /// Census `creatures`, each its reachable node indices and its learned
    /// state (whose view is empty on a VM node).
    pub fn observe<'a>(
        creatures: impl IntoIterator<Item = (&'a [usize], &'a LearnedState)>,
    ) -> Self {
        let mut census = Self::default();
        for (reachable, learned) in creatures {
            census.observe_one(reachable, learned);
        }
        census
    }

    /// Every (pathway, surface) cell with its keys, in [`LEARNING_KEYS`] then
    /// [`SURFACE_KEYS`] order.
    pub fn cells(&self) -> impl Iterator<Item = (&'static str, &'static str, PlasticEdges)> + '_ {
        LEARNING_KEYS
            .iter()
            .zip(&self.edges)
            .flat_map(|(&learning, row)| {
                SURFACE_KEYS
                    .iter()
                    .zip(row)
                    .map(move |(&surface, &cell)| (learning, surface, cell))
            })
    }

    /// Edge counts keyed by rule, in [`RULES`] order.
    pub fn rule_counts(&self) -> impl Iterator<Item = (&'static str, u64)> + '_ {
        RULES
            .iter()
            .map(|&rule| rule_key(rule))
            .zip(self.edges_by_rule)
    }

    /// Edge counts keyed by inheritance, in [`INHERITANCE_KEYS`] order.
    pub fn inheritance_counts(&self) -> impl Iterator<Item = (&'static str, u64)> + '_ {
        INHERITANCE_KEYS
            .iter()
            .copied()
            .zip(self.edges_by_inheritance)
    }

    /// Edge counts keyed by source, in [`SOURCE_KEYS`] order.
    pub fn source_counts(&self) -> impl Iterator<Item = (&'static str, u64)> + '_ {
        SOURCE_KEYS.iter().copied().zip(self.edges_by_source)
    }

    /// Scale cells keyed by inheritance, in [`INHERITANCE_KEYS`] order.
    pub fn scale_cells(&self) -> impl Iterator<Item = (&'static str, LearnedScales)> + '_ {
        INHERITANCE_KEYS.iter().copied().zip(self.scales)
    }

    /// Edge retention cells keyed by bin, in [`RETENTION_KEYS`] order.
    pub fn edge_retention_cells(&self) -> impl Iterator<Item = (&'static str, RetentionCell)> + '_ {
        RETENTION_KEYS.iter().copied().zip(self.edges_by_retention)
    }

    /// Scale retention cells keyed by bin, in [`RETENTION_KEYS`] order.
    pub fn scale_retention_cells(
        &self,
    ) -> impl Iterator<Item = (&'static str, RetentionCell)> + '_ {
        RETENTION_KEYS.iter().copied().zip(self.scales_by_retention)
    }

    /// Drift sum and at-clamp count per surface over both pathways, keyed
    /// in [`SURFACE_KEYS`] order.
    pub fn surface_totals(&self) -> impl Iterator<Item = (&'static str, f64, u64)> + '_ {
        SURFACE_KEYS.iter().enumerate().map(|(surface, &key)| {
            let (hebbian, reward) = (self.edges[0][surface], self.edges[1][surface]);
            (
                key,
                hebbian.drift_sum + reward.drift_sum,
                hebbian.at_clamp + reward.at_clamp,
            )
        })
    }

    fn observe_one(&mut self, reachable: &[usize], learned: &LearnedState) {
        self.creatures += 1;
        let mut carries = false;
        for &node_idx in reachable {
            for edge in learned.edges(node_idx) {
                carries = true;
                let kind = edge.kind;
                let surface = match edge.occurrence.surface {
                    EdgeSurface::ComputeInput(_) => 0,
                    EdgeSurface::SinkInput(_) => 1,
                };
                self.edges_by_rule[kind.rule_index()] += 1;
                self.edges_by_inheritance[usize::from(kind.lamarckian)] += 1;
                self.edges_by_source[edge.source.index()] += 1;
                self.edges_by_retention[retention_index(edge.retention)].add(
                    edge.learned,
                    edge.birth,
                    edge.bound,
                );
                let cell = &mut self.edges[usize::from(kind.reward_modulated)][surface];
                cell.edges += 1;
                let Some(weight) = edge.learned else {
                    continue;
                };
                add_drift(&mut cell.drift_sum, weight, edge.birth);
                cell.at_clamp += u64::from(weight.abs() >= edge.bound);
            }
            // A carrier has a learner, an edge's or a scale's, on a
            // reachable node (T25.F12 invariant 11).
            for scale in learned.scales(node_idx) {
                carries = true;
                self.scales_by_retention[retention_index(scale.retention)].add(
                    scale.learned,
                    scale.birth,
                    scale.bound,
                );
                let cell = &mut self.scales[usize::from(scale.lamarckian)];
                cell.scales += 1;
                let Some(value) = scale.learned else {
                    continue;
                };
                add_drift(&mut cell.drift_sum, value, scale.birth);
                cell.at_bound += u64::from(value.abs() >= scale.bound);
            }
        }
        self.carriers += u64::from(carries);
    }
}

#[cfg(test)]
#[path = "plasticity_census_tests.rs"]
mod tests;
