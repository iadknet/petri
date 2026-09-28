//! Small summary statistics. Sums are sequential folds in index order, so
//! results never depend on thread count.

/// z for a two-sided 95% interval.
const Z95: f64 = 1.959_963_984_540_054;

/// Arithmetic mean; `None` for an empty slice.
#[must_use]
pub fn mean(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        None
    } else {
        Some(values.iter().sum::<f64>() / values.len() as f64)
    }
}

/// Median (mean of the two middle values for an even count).
#[must_use]
pub fn median(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let mid = sorted.len() / 2;
    Some(if sorted.len().is_multiple_of(2) {
        (sorted[mid - 1] + sorted[mid]) / 2.0
    } else {
        sorted[mid]
    })
}

/// Wilson score 95% interval for `successes` of `trials`; `None` when
/// `trials` is zero.
#[must_use]
pub fn wilson_95(successes: u32, trials: u32) -> Option<[f64; 2]> {
    if trials == 0 || successes > trials {
        return None;
    }
    let n = f64::from(trials);
    let p = f64::from(successes) / n;
    let z2 = Z95 * Z95;
    let denominator = 1.0 + z2 / n;
    let centre = (p + z2 / (2.0 * n)) / denominator;
    let half = Z95 * (p * (1.0 - p) / n + z2 / (4.0 * n * n)).sqrt() / denominator;
    Some([(centre - half).max(0.0), (centre + half).min(1.0)])
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 5e-4
    }

    #[test]
    fn wilson_matches_known_counts() {
        // Reference values (Wilson 1927 score interval, z = 1.96).
        let [lo, hi] = wilson_95(0, 8).unwrap();
        assert!(close(lo, 0.0) && close(hi, 0.3244), "{lo} {hi}");
        let [lo, hi] = wilson_95(4, 8).unwrap();
        assert!(close(lo, 0.2152) && close(hi, 0.7848), "{lo} {hi}");
        let [lo, hi] = wilson_95(8, 8).unwrap();
        assert!(close(lo, 0.6756) && close(hi, 1.0), "{lo} {hi}");
        let [lo, hi] = wilson_95(1, 10).unwrap();
        assert!(close(lo, 0.0179) && close(hi, 0.4042), "{lo} {hi}");
        assert_eq!(wilson_95(0, 0), None);
    }

    #[test]
    fn median_and_mean_of_small_sets() {
        assert_eq!(median(&[3.0, 1.0, 2.0]), Some(2.0));
        assert_eq!(median(&[4.0, 1.0, 2.0, 3.0]), Some(2.5));
        assert_eq!(mean(&[1.0, 2.0]), Some(1.5));
        assert_eq!(mean(&[]), None);
    }

    proptest! {
        #[test]
        fn wilson_interval_is_ordered_in_unit_range_and_contains_the_rate(
            trials in 1u32..500,
            share in 0.0f64..=1.0,
        ) {
            let successes = (share * f64::from(trials)).round() as u32;
            let [lo, hi] = wilson_95(successes, trials).unwrap();
            let p = f64::from(successes) / f64::from(trials);
            prop_assert!((0.0..=1.0).contains(&lo) && (0.0..=1.0).contains(&hi));
            prop_assert!(lo <= p + 1e-12 && p <= hi + 1e-12);
        }
    }
}
