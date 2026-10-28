//! Distributions over the living population: a heritable trait's
//! nearest-rank percentiles and bound masses (T26.F02 capacity, T27.F03
//! exchange shares), the generation median and maximum, and the
//! rounded-index percentiles of reachable structure size.

use crate::simulation::Simulation;

/// A trait's distribution at one sampled tick. Percentiles are nearest-rank
/// over the sorted values: the `q` percentile is the value at rank
/// `ceil(q × n)` (1-based), so every reported value is one some creature
/// carries. The mean sums the sorted values in `f64`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TraitDistribution {
    pub min: f64,
    pub p10: f64,
    pub median: f64,
    pub p90: f64,
    pub max: f64,
    pub mean: f64,
    /// Share of values equal to the lower bound.
    pub at_min_fraction: f64,
    /// Share of values equal to the upper bound.
    pub at_max_fraction: f64,
}

impl TraitDistribution {
    /// The distribution of `values` in a `[lower, upper]` range, `None` when
    /// empty; the boundary fractions count exact equality, which the
    /// inheritance clamp produces.
    pub fn of(values: impl Iterator<Item = f32>, lower: f32, upper: f32) -> Option<Self> {
        let mut sorted: Vec<f32> = values.collect();
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
            sorted.iter().filter(|&&value| value == bound).count() as f64 / count as f64
        };
        // WORKAROUND(T30.F13): one definition of mean capacity — this
        // sorted-order mean (the benchmark `capacity` block) and the live
        // creature-order mean (`PopulationCensus::mean_capacity_live`) are
        // kept side by side, bit-identical to their consumers; unifying them
        // changes a live output's arithmetic, which needs its own decision.
        let mean = sorted.iter().copied().map(f64::from).sum::<f64>() / count as f64;
        Some(Self {
            min: f64::from(sorted[0]),
            p10: rank(0.1),
            median: rank(0.5),
            p90: rank(0.9),
            max: f64::from(sorted[count - 1]),
            mean,
            at_min_fraction: share(lower),
            at_max_fraction: share(upper),
        })
    }
}

/// The living population's heritable capacity over
/// `[energy.capacity.min, energy.lifecycle.max_energy]`, `None` at
/// extinction.
#[must_use]
pub fn capacity_distribution(sim: &Simulation) -> Option<TraitDistribution> {
    TraitDistribution::of(
        sim.creatures
            .values()
            .map(|creature| creature.body.capacity),
        sim.config.energy.capacity.min,
        sim.config.energy.lifecycle.max_energy,
    )
}

/// The two heritable exchange shares, each over `[exchange.share_min, 1]`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShareDistributions {
    pub steal_share: TraitDistribution,
    pub give_share: TraitDistribution,
}

/// The living population's exchange shares, `None` at extinction.
#[must_use]
pub fn share_distributions(sim: &Simulation) -> Option<ShareDistributions> {
    let share_min = sim.config.exchange.share_min;
    let of = |share: fn(&crate::creature::body::HeritableBody) -> f32| {
        TraitDistribution::of(
            sim.creatures.values().map(|creature| share(&creature.body)),
            share_min,
            1.0,
        )
    };
    Some(ShareDistributions {
        steal_share: of(|body| body.steal_share)?,
        give_share: of(|body| body.give_share)?,
    })
}

/// The generation median (the upper middle of an even count) and maximum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GenerationDistribution {
    pub median: u64,
    pub max: u64,
}

/// The distribution of `generations`, `None` when empty.
#[must_use]
pub fn generation_distribution(mut generations: Vec<u64>) -> Option<GenerationDistribution> {
    if generations.is_empty() {
        return None;
    }
    generations.sort_unstable();
    Some(GenerationDistribution {
        median: generations[generations.len() / 2],
        max: generations[generations.len() - 1],
    })
}

/// Reachable structure sizes: the minimum, the rounded-index 25th, 50th and
/// 75th percentiles, the maximum and the mean, all `0` when empty.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct StructureSizes {
    pub min: u32,
    pub p25: u32,
    pub median: u32,
    pub p75: u32,
    pub max: u32,
    pub mean: f64,
}

/// The value at index `round(p / 100 × (n − 1))` of `sorted`.
fn percentile(sorted: &[u32], p: f64) -> u32 {
    if sorted.is_empty() {
        return 0;
    }
    let idx = (p / 100.0 * (sorted.len() - 1) as f64).round();
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let idx = idx.clamp(0.0, (sorted.len() - 1) as f64) as usize;
    sorted[idx]
}

/// The distribution of the `pooled` sizes.
#[must_use]
pub fn structure_sizes(mut pooled: Vec<u32>) -> StructureSizes {
    if pooled.is_empty() {
        return StructureSizes::default();
    }
    pooled.sort_unstable();
    let sum: u64 = pooled.iter().map(|&v| u64::from(v)).sum();
    StructureSizes {
        min: pooled[0],
        p25: percentile(&pooled, 25.0),
        median: percentile(&pooled, 50.0),
        p75: percentile(&pooled, 75.0),
        max: pooled[pooled.len() - 1],
        mean: sum as f64 / pooled.len() as f64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn an_empty_population_has_no_trait_distribution() {
        assert_eq!(TraitDistribution::of(std::iter::empty(), 50.0, 800.0), None);
        assert_eq!(generation_distribution(Vec::new()), None);
        assert_eq!(structure_sizes(Vec::new()), StructureSizes::default());
    }

    #[test]
    fn the_distribution_reads_nearest_rank_percentiles_and_bound_shares() {
        let values = [
            50.0_f32, 50.0, 100.0, 200.0, 300.0, 400.0, 500.0, 600.0, 700.0, 800.0,
        ];
        let block = TraitDistribution::of(values.into_iter().rev(), 50.0, 800.0).unwrap();
        assert_eq!(
            (block.min, block.p10, block.median, block.p90, block.max),
            (50.0, 50.0, 300.0, 700.0, 800.0)
        );
        assert_eq!(block.mean, 3700.0 / 10.0);
        assert_eq!((block.at_min_fraction, block.at_max_fraction), (0.2, 0.1));
    }

    /// The sorted-order mean and a creature-order mean are two summation
    /// orders: on magnitudes about thirty binary orders apart they differ,
    /// which is why the live mean is kept beside this one.
    #[test]
    fn the_sorted_and_creature_order_capacity_means_are_two_definitions() {
        let values = [1.0e9_f32, 3.0e-1, -1.0e9, 3.0e-1];
        let sorted = TraitDistribution::of(values.into_iter(), 0.0, 1.0)
            .unwrap()
            .mean;
        let creature_order = values.iter().map(|&v| f64::from(v)).sum::<f64>() / 4.0;
        assert_ne!(sorted.to_bits(), creature_order.to_bits());
    }

    #[test]
    fn structure_sizes_round_their_percentile_index() {
        let sizes = structure_sizes(vec![9, 1, 5, 3]);
        assert_eq!(
            (sizes.min, sizes.p25, sizes.median, sizes.p75, sizes.max),
            (1, 3, 5, 5, 9)
        );
        assert_eq!(sizes.mean, 4.5);
        assert_eq!(
            generation_distribution(vec![4, 1, 3, 2]),
            Some(GenerationDistribution { median: 3, max: 4 })
        );
    }

    proptest! {
        #[test]
        fn the_order_statistics_are_ordered(values in prop::collection::vec(0.0f32..1000.0, 1..60)) {
            let block = TraitDistribution::of(values.iter().copied(), 0.0, 1000.0).unwrap();
            prop_assert!(block.min <= block.p10);
            prop_assert!(block.p10 <= block.median);
            prop_assert!(block.median <= block.p90);
            prop_assert!(block.p90 <= block.max);
            prop_assert!(block.min <= block.mean && block.mean <= block.max);
            let sizes = structure_sizes(values.iter().map(|&v| v as u32).collect());
            prop_assert!(sizes.min <= sizes.p25 && sizes.p25 <= sizes.median);
            prop_assert!(sizes.median <= sizes.p75 && sizes.p75 <= sizes.max);
        }
    }
}
