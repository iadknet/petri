//! The plasticity census: how many creatures carry plasticity, of which kind,
//! and how far learning has moved those weights from the genome.
//!
//! Reachable Graph nodes only. A plastic edge is an edge on either surface
//! with learning attributes (T25.F06), keyed by learning pathway (pure
//! Hebbian or reward-modulated) and surface (compute or sink), and counted
//! separately by rule and by inheritance (Lamarckian or Darwinian). The
//! census reads the learned state's view: edge drift is `|learned -
//! genomic|` over initialized own quantities (an ungrouped edge's weight, an
//! edge-scope member's correction; lifetime learning on a Darwinian edge,
//! on a Lamarckian edge also what ancestors learned), and an initialized
//! own quantity sits at its clamp when it reaches the edge's clamp, which
//! every write saturates to exactly. A group learner has no own quantity
//! and adds neither (T25.F07). Every learner counts by scope; each scale
//! slot counts once per group, by its `inherit_scale`, with its drift
//! `|learned - genomic|` and whether it sits at its bound.
//!
//! Pure structure and state: it reads genomes, cached reachability and
//! learned state and executes no brain.

use crate::creature::genome::cgp::{EdgeSurface, Scope};
use crate::creature::genome::HebbianRule;
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
/// Darwinian, 1 Lamarckian). A scale slot's inheritance is its group's
/// `inherit_scale`.
pub const INHERITANCE_KEYS: [&str; 2] = ["darwinian", "lamarckian"];

/// The `petri.plasticity_scope` keys, by scope index (0 ungrouped, 1 edge,
/// 2 group).
pub const SCOPE_KEYS: [&str; 3] = ["ungrouped", "edge", "group"];

/// The scope index of a learner of scope `scope` ([`SCOPE_KEYS`]).
#[must_use]
pub const fn scope_index(scope: Option<Scope>) -> usize {
    match scope {
        None => 0,
        Some(Scope::Edge) => 1,
        Some(Scope::Group) => 2,
    }
}

/// One inheritance cell's learned scales over a population.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct LearnedScales {
    pub scales: u64,
    /// Sum of `|learned - genomic|` over the initialized scale slots.
    pub drift_sum: f64,
    /// Initialized scale slots whose learned scale sits at their bound.
    pub at_bound: u64,
}

/// One (pathway, surface) cell's plastic edges over a population.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PlasticEdges {
    pub edges: u64,
    /// Sum of `|learned - genomic|` over the initialized edges.
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
    /// Edge counts per scope, in [`SCOPE_KEYS`] order.
    pub edges_by_scope: [u64; 3],
    /// Scale slots per inheritance, in [`INHERITANCE_KEYS`] order.
    pub scales: [LearnedScales; 2],
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

    /// Edge counts keyed by scope, in [`SCOPE_KEYS`] order.
    pub fn scope_counts(&self) -> impl Iterator<Item = (&'static str, u64)> + '_ {
        SCOPE_KEYS.iter().copied().zip(self.edges_by_scope)
    }

    /// Scale cells keyed by inheritance, in [`INHERITANCE_KEYS`] order.
    pub fn scale_cells(&self) -> impl Iterator<Item = (&'static str, LearnedScales)> + '_ {
        INHERITANCE_KEYS.iter().copied().zip(self.scales)
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
                self.edges_by_scope[scope_index(edge.scope)] += 1;
                let cell = &mut self.edges[usize::from(kind.reward_modulated)][surface];
                cell.edges += 1;
                let Some(weight) = edge.learned else {
                    continue;
                };
                let drift = f64::from((weight - edge.genomic).abs());
                // A non-finite weight would poison the population sum.
                if drift.is_finite() {
                    cell.drift_sum += drift;
                }
                cell.at_clamp += u64::from(weight.abs() >= edge.bound);
            }
            for scale in learned.scales(node_idx) {
                let cell = &mut self.scales[usize::from(scale.inherit_scale)];
                cell.scales += 1;
                let Some(value) = scale.learned else {
                    continue;
                };
                let drift = f64::from((value - scale.genomic).abs());
                if drift.is_finite() {
                    cell.drift_sum += drift;
                }
                cell.at_bound += u64::from(value.abs() >= scale.bound);
            }
        }
        self.carriers += u64::from(carries);
    }
}

#[cfg(test)]
#[path = "plasticity_census_tests.rs"]
mod tests;
