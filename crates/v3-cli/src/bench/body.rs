//! The per-sample `body` block (T29.F02): the body census of the living
//! population, how far its colour and kin-tag genes have spread.

use serde::{Deserialize, Serialize};

use v3_core::neighborhood::body_census::BodyCensus;

use crate::six;

/// The living population's body census at one sampled tick, formatted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BodyBlock {
    /// Distinct colour channel arrays.
    pub distinct_colours: u64,
    /// Share of creatures whose colour channels are the founder's.
    pub founder_colour_fraction: String,
    /// Distinct kin tags.
    pub distinct_kin_tags: u64,
    /// Mean kin-tag Hamming distance over pairs of equal `lineage_id`;
    /// `null` without such a pair. A pair's `KIN_AFFINITY` is `1 − h/32`.
    pub kin_hamming_within_clade: Option<String>,
    /// Mean kin-tag Hamming distance over pairs of different `lineage_id`;
    /// `null` without such a pair.
    pub kin_hamming_across_clades: Option<String>,
}

impl BodyBlock {
    /// The census of the living population, `None` at extinction.
    pub(super) fn observe(sim: &v3_core::simulation::Simulation) -> Option<Self> {
        BodyCensus::of(sim).map(Self::from)
    }
}

impl From<BodyCensus> for BodyBlock {
    fn from(census: BodyCensus) -> Self {
        Self {
            distinct_colours: census.distinct_colours,
            founder_colour_fraction: six(census.founder_colour_fraction),
            distinct_kin_tags: census.distinct_kin_tags,
            kin_hamming_within_clade: census.kin_hamming_within_clade.map(six),
            kin_hamming_across_clades: census.kin_hamming_across_clades.map(six),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_seeded_population_reports_its_founder_bodies_and_extinction_none() {
        let mut config = v3_core::config::SimulationConfig::default();
        config.population.initial_creatures = 3;
        let mut sim = v3_core::simulation::seed_simulation(config, 3);
        let block = BodyBlock::observe(&sim).expect("a living population");
        assert_eq!(block.distinct_colours, 1);
        assert_eq!(block.founder_colour_fraction, six(1.0));
        assert_eq!(block.distinct_kin_tags, 3);
        assert_eq!(block.kin_hamming_within_clade, None);
        assert!(block.kin_hamming_across_clades.is_some());
        let json = serde_json::to_value(&block).unwrap();
        assert!(json["kin_hamming_within_clade"].is_null());
        sim.creatures.clear();
        assert_eq!(BodyBlock::observe(&sim), None);
    }
}
