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
