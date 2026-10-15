//! The plasticity census: how many creatures carry plasticity, of which kind,
//! and how far learning has moved those weights from the genome.
//!
//! Reachable Graph nodes only. A plastic edge is an input edge of a compute
//! node with `plasticity: Some(_)`, keyed by rule, learning pathway (pure
//! Hebbian or reward-modulated) and inheritance (Lamarckian or Darwinian).
//! Its learned weight is the creature's `plasticity_weights` entry when the
//! runtime has initialized one, else the genome weight, as the runtime reads
//! it. Drift is `|learned - genome|`: lifetime learning on a Darwinian edge;
//! on a Lamarckian edge it also carries what ancestors learned. An edge sits
//! at its clamp when an initialized learned weight reaches the node's runtime
//! clamp, which every write saturates to exactly.
//!
//! Pure structure and state: it reads genomes, cached reachability and learned
//! weights and executes no brain.

use crate::creature::genome::{BackendDef, CreatureGenome, HebbianRule, PlasticityConfig};

/// One plastic edge kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlasticityKind {
    pub rule: HebbianRule,
    pub reward_modulated: bool,
    pub lamarckian: bool,
}

const RULES: [HebbianRule; 4] = [
    HebbianRule::Classic,
    HebbianRule::Oja,
    HebbianRule::AntiHebb,
    HebbianRule::Covariance,
];

const KINDS: usize = RULES.len() * 4;

impl PlasticityKind {
    /// Every kind, in [`Self::index`] order.
    pub const ALL: [Self; KINDS] = {
        let mut all = [Self {
            rule: HebbianRule::Classic,
            reward_modulated: false,
            lamarckian: false,
        }; KINDS];
        let mut i = 0;
        while i < KINDS {
            all[i] = Self {
                rule: RULES[i / 4],
                reward_modulated: i % 4 >= 2,
                lamarckian: i % 2 == 1,
            };
            i += 1;
        }
        all
    };

    fn of(config: &PlasticityConfig) -> Self {
        Self {
            rule: config.rule,
            reward_modulated: config.modulation.is_some(),
            lamarckian: config.lamarckian,
        }
    }

    /// Position in [`Self::ALL`].
    #[must_use]
    pub const fn index(self) -> usize {
        let rule = match self.rule {
            HebbianRule::Classic => 0,
            HebbianRule::Oja => 1,
            HebbianRule::AntiHebb => 2,
            HebbianRule::Covariance => 3,
        };
        rule * 4 + (self.reward_modulated as usize) * 2 + self.lamarckian as usize
    }

    #[must_use]
    pub const fn rule_key(self) -> &'static str {
        match self.rule {
            HebbianRule::Classic => "classic",
            HebbianRule::Oja => "oja",
            HebbianRule::AntiHebb => "anti_hebb",
            HebbianRule::Covariance => "covariance",
        }
    }

    #[must_use]
    pub const fn learning_key(self) -> &'static str {
        if self.reward_modulated {
            "reward_modulated"
        } else {
            "hebbian"
        }
    }

    #[must_use]
    pub const fn inheritance_key(self) -> &'static str {
        if self.lamarckian {
            "lamarckian"
        } else {
            "darwinian"
        }
    }
}

/// One kind's plastic edges over a population.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PlasticEdges {
    pub edges: u64,
    /// Sum of `|learned - genome|` over the edges.
    pub drift_sum: f64,
    /// Edges whose initialized learned weight sits at the node's clamp.
    pub at_clamp: u64,
}

/// The census of one population, per kind in [`PlasticityKind::ALL`] order.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PlasticityCensus {
    pub creatures: u64,
    /// Creatures with at least one reachable plastic edge.
    pub carriers: u64,
    pub kinds: [PlasticEdges; KINDS],
}

/// A creature's learned weights, `[mesh node][compute node][edge]`.
pub type LearnedWeights = [Vec<Box<[f32]>>];

impl PlasticityCensus {
    /// Census `creatures`, each a genome, its reachable node indices and its
    /// learned weights.
    pub fn observe<'a>(
        creatures: impl IntoIterator<Item = (&'a CreatureGenome, &'a [usize], &'a LearnedWeights)>,
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
        learned: &LearnedWeights,
    ) {
        self.creatures += 1;
        let mut carries = false;
        for &node_idx in reachable {
            let Some(BackendDef::Graph(def)) = genome.nodes.get(node_idx).map(|n| &n.backend_def)
            else {
                continue;
            };
            for (compute_idx, compute) in def.compute_nodes().iter().enumerate() {
                let Some(config) = &compute.plasticity else {
                    continue;
                };
                carries |= !compute.inputs.is_empty();
                // The runtime's bounds (runtime/plasticity), which every write saturates to.
                let clamp = config.weight_clamp.clamp(0.01, 10.0);
                let weights = learned
                    .get(node_idx)
                    .and_then(|node| node.get(compute_idx))
                    .map_or(&[][..], |weights| &weights[..]);
                let row = &mut self.kinds[PlasticityKind::of(config).index()];
                for (edge, genomic) in compute.inputs.iter().enumerate() {
                    row.edges += 1;
                    let Some(&weight) = weights.get(edge) else {
                        continue;
                    };
                    let drift = f64::from((weight - genomic.own_weight()).abs());
                    // A non-finite weight would poison the population sum.
                    if drift.is_finite() {
                        row.drift_sum += drift;
                    }
                    row.at_clamp += u64::from(weight.abs() >= clamp);
                }
            }
        }
        self.carriers += u64::from(carries);
    }
}

#[cfg(test)]
#[path = "plasticity_census_tests.rs"]
mod tests;
