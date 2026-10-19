//! The plasticity census: how many creatures carry plasticity, of which kind,
//! and how far learning has moved those weights from the genome.
//!
//! Reachable Graph nodes only. A plastic edge is an edge on either surface
//! with learning attributes (T25.F06), keyed by learning pathway (pure
//! Hebbian or reward-modulated) and surface (compute or sink), and counted
//! separately by rule and by inheritance (Lamarckian or Darwinian). The
//! census reads the learned state's view: drift is `|learned - genomic|`
//! over initialized edges (lifetime learning on a Darwinian edge; on a
//! Lamarckian edge it also carries what ancestors learned), and an
//! initialized edge sits at its clamp when its learned weight reaches the
//! edge's clamp, which every write saturates to exactly.
//!
//! Pure structure and state: it reads genomes, cached reachability and
//! learned state and executes no brain.

use crate::creature::genome::cgp::EdgeSurface;
use crate::creature::genome::{BackendDef, CreatureGenome, HebbianRule, PlasticityConfig};
use crate::runtime::plasticity::LearnedState;

/// One plastic edge kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlasticityKind {
    pub rule: HebbianRule,
    pub reward_modulated: bool,
    pub lamarckian: bool,
}

/// The four rules, in census order.
pub const RULES: [HebbianRule; 4] = [
    HebbianRule::Classic,
    HebbianRule::Oja,
    HebbianRule::AntiHebb,
    HebbianRule::Covariance,
];

impl PlasticityKind {
    /// The kind of an edge with learning attributes `config`.
    #[must_use]
    pub fn of(config: &PlasticityConfig) -> Self {
        Self {
            rule: config.rule,
            reward_modulated: config.modulation.is_some(),
            lamarckian: config.lamarckian,
        }
    }

    /// Position of the rule in [`RULES`].
    #[must_use]
    pub const fn rule_index(self) -> usize {
        match self.rule {
            HebbianRule::Classic => 0,
            HebbianRule::Oja => 1,
            HebbianRule::AntiHebb => 2,
            HebbianRule::Covariance => 3,
        }
    }
}

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
/// Darwinian, 1 Lamarckian).
pub const INHERITANCE_KEYS: [&str; 2] = ["darwinian", "lamarckian"];

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
}

impl PlasticityCensus {
    /// Census `creatures`, each a genome, its reachable node indices and its
    /// learned state.
    pub fn observe<'a>(
        creatures: impl IntoIterator<Item = (&'a CreatureGenome, &'a [usize], &'a LearnedState)>,
    ) -> Self {
        let mut census = Self::default();
        for (genome, reachable, learned) in creatures {
            census.observe_one(genome, reachable, learned);
        }
        census
    }

    fn observe_one(
        &mut self,
        genome: &CreatureGenome,
        reachable: &[usize],
        learned: &LearnedState,
    ) {
        self.creatures += 1;
        let mut carries = false;
        for &node_idx in reachable {
            if !matches!(
                genome.nodes.get(node_idx).map(|n| &n.backend_def),
                Some(BackendDef::Graph(_))
            ) {
                continue;
            }
            for edge in learned.edges(node_idx) {
                carries = true;
                let kind = edge.kind;
                let surface = match edge.occurrence.surface {
                    EdgeSurface::ComputeInput(_) => 0,
                    EdgeSurface::SinkInput(_) => 1,
                };
                self.edges_by_rule[kind.rule_index()] += 1;
                self.edges_by_inheritance[usize::from(kind.lamarckian)] += 1;
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
        }
        self.carriers += u64::from(carries);
    }
}

#[cfg(test)]
#[path = "plasticity_census_tests.rs"]
mod tests;
