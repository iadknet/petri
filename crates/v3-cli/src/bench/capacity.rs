//! The per-sample `capacity` block (T26.F02): the distribution of the living
//! population's heritable energy capacity and the mass at each bound of its
//! range; and the `shares` block beside it (T27.F03), the same distribution
//! of the two heritable exchange shares over `[exchange.share_min, 1]`.

use serde::{Deserialize, Serialize};

use v3_core::neighborhood::distributions::{self, TraitDistribution};

use crate::six;

/// The living population's capacity distribution at one sampled tick.
/// Percentiles are nearest-rank over the sorted capacities: the `q`
/// percentile is the value at rank `ceil(q × n)` (1-based), so every
/// reported value is a capacity some creature carries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapacityDistribution {
    pub min: String,
    pub p10: String,
    pub median: String,
    pub p90: String,
    pub max: String,
    pub mean: String,
    /// Share of creatures whose capacity equals `energy.capacity.min`.
    pub at_min_fraction: String,
    /// Share of creatures whose capacity equals `energy.lifecycle.max_energy`.
    pub at_max_fraction: String,
}

impl CapacityDistribution {
    /// The distribution of the living population's capacities, `None` at
    /// extinction.
    pub(super) fn observe(sim: &v3_core::simulation::Simulation) -> Option<Self> {
        distributions::capacity_distribution(sim).map(Self::from)
    }

    /// The readers' distribution of `capacities` in a `[lower, upper]` range,
    /// formatted.
    #[cfg(test)]
    fn of(capacities: impl Iterator<Item = f32>, lower: f32, upper: f32) -> Option<Self> {
        TraitDistribution::of(capacities, lower, upper).map(Self::from)
    }
}

impl From<TraitDistribution> for CapacityDistribution {
    fn from(distribution: TraitDistribution) -> Self {
        Self {
            min: six(distribution.min),
            p10: six(distribution.p10),
            median: six(distribution.median),
            p90: six(distribution.p90),
            max: six(distribution.max),
            mean: six(distribution.mean),
            at_min_fraction: six(distribution.at_min_fraction),
            at_max_fraction: six(distribution.at_max_fraction),
        }
    }
}

/// The living population's two exchange shares at one sampled tick
/// (T27.F03), each over `[exchange.share_min, 1]`; the boundary fractions
/// count equality with the bounds in force.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShareDistributions {
    pub steal_share: CapacityDistribution,
    pub give_share: CapacityDistribution,
}

impl ShareDistributions {
    /// The two share distributions, `None` at extinction.
    pub(super) fn observe(sim: &v3_core::simulation::Simulation) -> Option<Self> {
        distributions::share_distributions(sim).map(|shares| Self {
            steal_share: shares.steal_share.into(),
            give_share: shares.give_share.into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn an_empty_population_has_no_distribution() {
        assert_eq!(
            CapacityDistribution::of(std::iter::empty(), 50.0, 800.0),
            None
        );
    }

    #[test]
    fn a_seeded_population_reports_its_founders_and_extinction_none() {
        let mut config = v3_core::config::SimulationConfig::default();
        config.population.initial_creatures = 3;
        let mut sim = v3_core::simulation::seed_simulation(config, 3);
        let block = CapacityDistribution::observe(&sim).expect("a living population");
        assert_eq!(block.median, six(200.0));
        assert_eq!(block.at_min_fraction, six(0.0));
        sim.creatures.clear();
        assert_eq!(CapacityDistribution::observe(&sim), None);
    }

    #[test]
    fn a_seeded_population_reports_its_founder_shares_and_extinction_none() {
        let mut config = v3_core::config::SimulationConfig::default();
        config.population.initial_creatures = 3;
        let mut sim = v3_core::simulation::seed_simulation(config, 3);
        let block = ShareDistributions::observe(&sim).expect("a living population");
        for share in [&block.steal_share, &block.give_share] {
            assert_eq!(share.median, six(f64::from(0.2f32)));
            assert_eq!(share.at_min_fraction, six(0.0));
            assert_eq!(share.at_max_fraction, six(0.0));
        }
        let first = sim.creatures.keys().next().unwrap();
        sim.creatures[first].body.steal_share = sim.config.exchange.share_min;
        sim.creatures[first].body.give_share = 1.0;
        let block = ShareDistributions::observe(&sim).unwrap();
        assert_eq!(block.steal_share.at_min_fraction, six(1.0 / 3.0));
        assert_eq!(block.give_share.at_max_fraction, six(1.0 / 3.0));
        sim.creatures.clear();
        assert_eq!(ShareDistributions::observe(&sim), None);
    }

    #[test]
    fn the_distribution_reads_nearest_rank_percentiles_and_bound_shares() {
        // Ten creatures: two at the min, one at the max.
        let capacities = [
            50.0, 50.0, 100.0, 150.0, 200.0, 200.0, 250.0, 300.0, 400.0, 800.0,
        ];
        let block = CapacityDistribution::of(capacities.into_iter().rev(), 50.0, 800.0).unwrap();
        assert_eq!(
            block,
            CapacityDistribution {
                min: six(50.0),
                p10: six(50.0),
                median: six(200.0),
                p90: six(400.0),
                max: six(800.0),
                mean: six(250.0),
                at_min_fraction: six(0.2),
                at_max_fraction: six(0.1),
            }
        );
    }

    #[test]
    fn a_founder_population_sits_at_the_reference_off_both_bounds() {
        let block = CapacityDistribution::of([200.0; 4].into_iter(), 50.0, 800.0).unwrap();
        assert_eq!(block.min, block.max);
        assert_eq!(block.median, six(200.0));
        assert_eq!(block.at_min_fraction, six(0.0));
        assert_eq!(block.at_max_fraction, six(0.0));
    }

    proptest! {
        /// The reported order statistics are ordered, the mean lies within
        /// them, and the boundary shares are fractions.
        #[test]
        fn the_order_statistics_are_ordered(
            capacities in proptest::collection::vec(50.0f32..=800.0, 1..64),
        ) {
            let block = CapacityDistribution::of(capacities.into_iter(), 50.0, 800.0).unwrap();
            let read = |s: &str| s.parse::<f64>().unwrap();
            let order = [&block.min, &block.p10, &block.median, &block.p90, &block.max].map(|s| read(s));
            prop_assert!(order.windows(2).all(|pair| pair[0] <= pair[1]));
            prop_assert!(read(&block.mean) >= order[0] - 1e-6 && read(&block.mean) <= order[4] + 1e-6);
            for share in [&block.at_min_fraction, &block.at_max_fraction] {
                prop_assert!((0.0..=1.0).contains(&read(share)));
            }
        }
    }
}
