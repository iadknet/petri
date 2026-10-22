//! The per-sample `capacity` block (T26.F02): the distribution of the living
//! population's heritable energy capacity and the mass at each bound of its
//! range.

use serde::{Deserialize, Serialize};

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
        Self::of(
            sim.creatures
                .values()
                .map(|creature| creature.body.capacity),
            sim.config.energy.capacity.min,
            sim.config.energy.lifecycle.max_energy,
        )
    }

    /// The distribution of `capacities` in a `[lower, upper]` range; the
    /// boundary fractions count exact equality, which the inheritance clamp
    /// produces.
    fn of(capacities: impl Iterator<Item = f32>, lower: f32, upper: f32) -> Option<Self> {
        let mut sorted: Vec<f32> = capacities.collect();
        if sorted.is_empty() {
            return None;
        }
        sorted.sort_by(f32::total_cmp);
        let count = sorted.len();
        let rank = |q: f64| {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let rank = (q * count as f64).ceil() as usize;
            f64::from(sorted[rank.clamp(1, count) - 1])
        };
        let share = |bound: f32| {
            sorted.iter().filter(|&&capacity| capacity == bound).count() as f64 / count as f64
        };
        let mean = sorted.iter().copied().map(f64::from).sum::<f64>() / count as f64;
        Some(Self {
            min: six(f64::from(sorted[0])),
            p10: six(rank(0.1)),
            median: six(rank(0.5)),
            p90: six(rank(0.9)),
            max: six(f64::from(sorted[count - 1])),
            mean: six(mean),
            at_min_fraction: six(share(lower)),
            at_max_fraction: six(share(upper)),
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
