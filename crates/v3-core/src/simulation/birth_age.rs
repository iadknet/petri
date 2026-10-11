//! Parent age at each delivered birth, as a fixed-bucket histogram per birth order.

/// Inclusive upper bounds of the age buckets, in ticks; one overflow bucket
/// follows. Fine just above the default `min_reproduce_age` (20), where most
/// births fall, then roughly log-spaced past the default `age_cost.age_cap`
/// (200) to long-lived parents.
pub const BIRTH_PARENT_AGE_BOUNDS: [u64; 15] = [
    20, 22, 25, 30, 40, 50, 75, 100, 150, 200, 300, 500, 1_000, 2_000, 5_000,
];

/// Whether a birth is the parent's first delivered child.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BirthOrder {
    First,
    Later,
}

impl BirthOrder {
    pub const ALL: [Self; 2] = [Self::First, Self::Later];

    #[must_use]
    pub const fn of(offspring_before: u64) -> Self {
        if offspring_before == 0 {
            Self::First
        } else {
            Self::Later
        }
    }

    #[must_use]
    pub const fn as_key(self) -> &'static str {
        match self {
            Self::First => "first",
            Self::Later => "later",
        }
    }
}

/// Cumulative births by parent age: bucket `i` counts ages in
/// `(BOUNDS[i - 1], BOUNDS[i]]`, the last bucket ages above every bound.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ParentAgeHistogram {
    pub bucket_counts: [u64; BIRTH_PARENT_AGE_BOUNDS.len() + 1],
    pub sum: u64,
    pub count: u64,
}

impl ParentAgeHistogram {
    fn record(&mut self, age: u64) {
        let bucket = BIRTH_PARENT_AGE_BOUNDS.partition_point(|bound| *bound < age);
        self.bucket_counts[bucket] += 1;
        self.sum = self.sum.saturating_add(age);
        self.count += 1;
    }
}

/// One histogram per [`BirthOrder`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BirthParentAge {
    by_order: [ParentAgeHistogram; BirthOrder::ALL.len()],
}

impl BirthParentAge {
    /// Record one delivered birth from a parent of `age` ticks.
    pub fn record(&mut self, order: BirthOrder, age: u64) {
        self.by_order[order as usize].record(age);
    }

    #[must_use]
    pub fn totals(&self, order: BirthOrder) -> ParentAgeHistogram {
        self.by_order[order as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn bounds_are_inclusive_upper_edges_with_an_overflow_bucket() {
        let mut ages = BirthParentAge::default();
        for age in [0, 20, 21, 5_000, 5_001] {
            ages.record(BirthOrder::First, age);
        }
        let first = ages.totals(BirthOrder::First);
        assert_eq!(first.bucket_counts[0], 2, "0 and 20");
        assert_eq!(first.bucket_counts[1], 1, "21");
        assert_eq!(first.bucket_counts[14], 1, "5_000");
        assert_eq!(first.bucket_counts[15], 1, "5_001");
        assert_eq!(first.count, 5);
        assert_eq!(first.sum, 10_042);
        assert_eq!(
            ages.totals(BirthOrder::Later),
            ParentAgeHistogram::default()
        );
    }

    #[test]
    fn birth_order_keys_and_first_birth_rule() {
        assert_eq!(BirthOrder::ALL.map(BirthOrder::as_key), ["first", "later"]);
        assert_eq!(BirthOrder::of(0), BirthOrder::First);
        assert_eq!(BirthOrder::of(1), BirthOrder::Later);
    }

    proptest! {
        #[test]
        fn every_age_lands_in_the_one_bucket_containing_it(
            ages in proptest::collection::vec(0u64..20_000, 0..64),
        ) {
            let mut histogram = ParentAgeHistogram::default();
            for &age in &ages {
                let before = histogram;
                histogram.record(age);
                let bucket = (0..histogram.bucket_counts.len())
                    .find(|&i| histogram.bucket_counts[i] != before.bucket_counts[i])
                    .unwrap();
                let lower = bucket.checked_sub(1).map(|i| BIRTH_PARENT_AGE_BOUNDS[i]);
                let upper = BIRTH_PARENT_AGE_BOUNDS.get(bucket);
                prop_assert!(lower.is_none_or(|lower| age > lower));
                prop_assert!(upper.is_none_or(|upper| age <= *upper));
            }
            prop_assert_eq!(histogram.bucket_counts.iter().sum::<u64>(), histogram.count);
            prop_assert_eq!(histogram.count, ages.len() as u64);
            prop_assert_eq!(histogram.sum, ages.iter().sum::<u64>());
        }
    }
}
