//! Evolved-sample rank selection (T11.F01 Battery, "Predeclared sizes and
//! seeds"): 12 genomes from the final living, id-sorted population, at ranks
//! `floor(i * n / 12)` for `i` in `0..12`; all of them when `n < 12` (the
//! rank formula duplicates indices below 12 so it is not used there); none
//! when the population is empty.

/// Target sample size per seed.
pub const SAMPLE_SIZE: usize = 12;

/// The 0-based ranks, in ascending order, of the id-sorted final population
/// to sample. Pure over the population size alone.
#[must_use]
pub fn evolved_sample_ranks(population_size: usize) -> Vec<usize> {
    if population_size == 0 {
        return Vec::new();
    }
    if population_size < SAMPLE_SIZE {
        return (0..population_size).collect();
    }
    (0..SAMPLE_SIZE)
        .map(|i| i * population_size / SAMPLE_SIZE)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_population_samples_nothing() {
        assert_eq!(evolved_sample_ranks(0), Vec::<usize>::new());
    }

    #[test]
    fn population_below_sample_size_takes_every_rank() {
        assert_eq!(evolved_sample_ranks(1), vec![0]);
        assert_eq!(evolved_sample_ranks(5), vec![0, 1, 2, 3, 4]);
        assert_eq!(evolved_sample_ranks(11), (0..11).collect::<Vec<_>>());
    }

    #[test]
    fn population_at_sample_size_takes_every_rank_once() {
        assert_eq!(evolved_sample_ranks(12), (0..12).collect::<Vec<_>>());
    }

    #[test]
    fn population_above_sample_size_spreads_ranks_and_stays_distinct() {
        let ranks = evolved_sample_ranks(100);
        assert_eq!(ranks.len(), 12);
        assert!(
            ranks.windows(2).all(|w| w[0] < w[1]),
            "ranks must be strictly increasing: {ranks:?}"
        );
        assert!(*ranks.last().unwrap() < 100);
    }

    #[test]
    fn population_far_above_sample_size_stays_within_bounds_and_sorted() {
        for n in [13, 24, 1000, 24_418] {
            let ranks = evolved_sample_ranks(n);
            assert_eq!(ranks.len(), 12, "n={n}");
            assert!(
                ranks.windows(2).all(|w| w[0] < w[1]),
                "n={n} ranks={ranks:?}"
            );
            assert!(*ranks.last().unwrap() < n, "n={n} ranks={ranks:?}");
        }
    }
}

/// Seed base for the neighborhood read (T14.F12): the sample draw seeds
/// `SmallRng` with `READ_SEED_BASE.wrapping_add(world_seed)`, and sampled genome `i`
/// births with offset `READ_SEED_BASE + READ_GENOME_MULTIPLIER * (i + 1)`,
/// disjoint from the evolved half's `100_000 * (i + 1)` and the drift walk's
/// `7_000_000 + …` offsets.
pub const READ_SEED_BASE: u64 = 8_000_000;
pub const READ_GENOME_MULTIPLIER: u64 = 1_000;

/// The 0-based ranks, ascending, of the id-sorted living population the
/// neighborhood read samples: a uniform draw without replacement of
/// `min(sample_size, population_size)` ranks from
/// `SmallRng::seed_from_u64(READ_SEED_BASE.wrapping_add(world_seed))`, total
/// over `u64` by wrapping. Pure over its arguments; the draw algorithm is
/// pinned by a unit test and a change to it bumps the read's version.
#[must_use]
pub fn read_sample_ranks(
    population_size: usize,
    sample_size: usize,
    world_seed: u64,
) -> Vec<usize> {
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    let amount = sample_size.min(population_size);
    if amount == 0 {
        return Vec::new();
    }
    let mut rng = SmallRng::seed_from_u64(READ_SEED_BASE.wrapping_add(world_seed));
    let mut ranks = rand::seq::index::sample(&mut rng, population_size, amount).into_vec();
    ranks.sort_unstable();
    ranks
}

#[cfg(test)]
mod read_tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn empty_population_or_zero_sample_reads_nothing() {
        assert_eq!(read_sample_ranks(0, 50, 11), Vec::<usize>::new());
        assert_eq!(read_sample_ranks(10, 0, 11), Vec::<usize>::new());
    }

    #[test]
    fn population_at_or_below_sample_size_takes_every_rank() {
        assert_eq!(read_sample_ranks(1, 50, 11), vec![0]);
        assert_eq!(read_sample_ranks(7, 50, 11), (0..7).collect::<Vec<_>>());
        assert_eq!(read_sample_ranks(50, 50, 11), (0..50).collect::<Vec<_>>());
    }

    /// Pins the draw algorithm (`rand::seq::index::sample` from
    /// `SmallRng::seed_from_u64(8_000_000.wrapping_add(world_seed))`, sorted ascending)
    /// for one `(n, sample, seed)` triple; a change here must bump the read
    /// version.
    #[test]
    fn draw_is_pinned_for_one_population_and_seed() {
        assert_eq!(
            read_sample_ranks(200, 10, 11),
            vec![6, 8, 25, 101, 121, 127, 156, 158, 161, 167]
        );
    }

    proptest! {
        #[test]
        fn ranks_are_ascending_distinct_in_bounds_and_seed_fixed(
            population_size in 0usize..3_000,
            sample_size in 0usize..80,
            world_seed in any::<u64>(),
        ) {
            let ranks = read_sample_ranks(population_size, sample_size, world_seed);
            prop_assert_eq!(ranks.len(), sample_size.min(population_size));
            prop_assert!(ranks.windows(2).all(|w| w[0] < w[1]), "{:?}", ranks);
            prop_assert!(ranks.iter().all(|&rank| rank < population_size));
            prop_assert_eq!(
                read_sample_ranks(population_size, sample_size, world_seed),
                ranks
            );
        }
    }
}
