/// Compute the probability of restricting mutations to non-increasing operators.
///
/// Uses a quadratic curve: `pressure = clamp(complexity / cap, 0, 1)^2`
/// This gives a smooth ramp — low-complexity creatures are unrestricted,
/// pressure escalates near the cap.
///
/// Returns 0.0 if `cap` is 0 (guard against division by zero).
#[must_use]
pub(crate) fn restriction_probability(complexity: u32, cap: u32) -> f64 {
    if cap == 0 {
        return 0.0;
    }
    let fill = (complexity as f64 / cap as f64).clamp(0.0, 1.0);
    fill * fill
}

/// Determine whether this birth's mutations should be restricted to non-increasing operators.
///
/// Rolls the RNG against `restriction_probability(complexity, cap)`.
pub(crate) fn is_restricted(complexity: u32, cap: u32, rng: &mut impl rand::Rng) -> bool {
    let prob = restriction_probability(complexity, cap);
    if prob <= 0.0 {
        return false;
    }
    if prob >= 1.0 {
        return true;
    }
    rng.gen_bool(prob)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_complexity_gives_zero_pressure() {
        let p = restriction_probability(0, 1200);
        assert!((p - 0.0).abs() < 1e-12);
    }

    #[test]
    fn full_cap_gives_pressure_one() {
        let p = restriction_probability(1200, 1200);
        assert!((p - 1.0).abs() < 1e-12);
    }

    #[test]
    fn half_cap_gives_quarter_pressure() {
        let p = restriction_probability(600, 1200);
        assert!((p - 0.25).abs() < 1e-12);
    }

    #[test]
    fn quarter_cap_gives_expected_pressure() {
        let p = restriction_probability(300, 1200);
        assert!((p - 0.0625).abs() < 1e-12);
    }

    #[test]
    fn above_cap_clamps_to_one() {
        let p = restriction_probability(2000, 1200);
        assert!((p - 1.0).abs() < 1e-12);
    }

    #[test]
    fn zero_cap_returns_zero() {
        let p = restriction_probability(100, 0);
        assert!((p - 0.0).abs() < 1e-12);
    }

    #[test]
    fn monotonically_increasing() {
        let cap = 1200u32;
        let mut prev = 0.0;
        for c in 0..=cap {
            let p = restriction_probability(c, cap);
            assert!(
                p >= prev,
                "pressure must be monotonically increasing: {} < {} at complexity {}",
                p,
                prev,
                c
            );
            prev = p;
        }
    }

    #[test]
    fn is_restricted_zero_complexity_never_restricted() {
        use rand::rngs::SmallRng;
        use rand::SeedableRng;
        for seed in 0u64..100 {
            let mut rng = SmallRng::seed_from_u64(seed);
            assert!(!is_restricted(0, 1200, &mut rng));
        }
    }

    #[test]
    fn is_restricted_at_cap_always_restricted() {
        use rand::rngs::SmallRng;
        use rand::SeedableRng;
        for seed in 0u64..100 {
            let mut rng = SmallRng::seed_from_u64(seed);
            assert!(is_restricted(1200, 1200, &mut rng));
        }
    }

    #[test]
    fn is_restricted_above_cap_always_restricted() {
        use rand::rngs::SmallRng;
        use rand::SeedableRng;
        for seed in 0u64..100 {
            let mut rng = SmallRng::seed_from_u64(seed);
            assert!(is_restricted(5000, 1200, &mut rng));
        }
    }

    #[test]
    fn is_restricted_half_cap_sometimes_restricted() {
        use rand::rngs::SmallRng;
        use rand::SeedableRng;
        let mut restricted_count = 0;
        for seed in 0u64..1000 {
            let mut rng = SmallRng::seed_from_u64(seed);
            if is_restricted(600, 1200, &mut rng) {
                restricted_count += 1;
            }
        }
        // At p=0.25, expect ~250 restricted out of 1000. Accept 150..350.
        assert!(
            restricted_count > 150 && restricted_count < 350,
            "expected ~25% restricted at half cap, got {}/1000",
            restricted_count
        );
    }

    #[test]
    fn is_restricted_zero_cap_never_restricted() {
        use rand::rngs::SmallRng;
        use rand::SeedableRng;
        for seed in 0u64..100 {
            let mut rng = SmallRng::seed_from_u64(seed);
            assert!(!is_restricted(500, 0, &mut rng));
        }
    }
}
