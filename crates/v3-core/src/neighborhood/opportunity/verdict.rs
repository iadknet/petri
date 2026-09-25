//! The opportunity verdict rules: an exposure gate, informative replicates,
//! and a paired sign rule on `a / z`, the `A_k` and `Z_k` cumulative births
//! of one replicate. Pure functions over integer tallies.

/// Pooled share of sampled `A_k` creatures that must read a nonzero authored
/// channel, and whose actions ablating those channels must change.
pub const MIN_EXPOSED: f64 = 0.05;
pub const MIN_APPLIED: f64 = 0.01;
/// A replicate is informative with at least this many `a + z` births.
pub const MIN_INFORMATIVE_BIRTHS: u64 = 20;
/// Informative replicates required, and replicates whose ratio must agree.
pub const MIN_INFORMATIVE: u32 = 7;
pub const MIN_AGREEING: u32 = 7;
/// The pooled-ratio line both rules use.
pub const RATIO_LINE: f64 = 1.05;

/// `a / z`: `+∞` when `z = 0 < a`, undefined when both are zero.
#[must_use]
pub fn ratio(a: u64, z: u64) -> Option<f64> {
    match (a, z) {
        (0, 0) => None,
        (_, 0) => Some(f64::INFINITY),
        _ => Some(a as f64 / z as f64),
    }
}

/// The exact one-sided sign-test p-value of `k` or more successes in `n`
/// fair trials: `sum_{j >= k} C(n, j) / 2^n`.
#[must_use]
pub fn sign_test_p(k: u32, n: u32) -> f64 {
    let mut coefficient = 1.0f64; // C(n, 0)
    let mut tail = 0.0;
    for j in 0..=n {
        if j >= k {
            tail += coefficient;
        }
        coefficient = coefficient * f64::from(n - j) / f64::from(j + 1);
    }
    tail / 2f64.powi(n as i32)
}

/// A family's verdict in one world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Positive,
    Negative,
    Inconclusive,
    /// The exposure gate is not met, or nothing was sampled.
    InconclusiveExposure,
    /// A predeclared null-reference pair: nothing to sense in this world.
    NotApplicable,
}

/// Exposure tallies of sampled `A_k` creatures, pooled over replicates and
/// samples.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Exposure {
    pub sampled: u64,
    pub exposed: u64,
    pub applied: u64,
}

impl Exposure {
    #[must_use]
    pub fn merge(self, other: Self) -> Self {
        Self {
            sampled: self.sampled + other.sampled,
            exposed: self.exposed + other.exposed,
            applied: self.applied + other.applied,
        }
    }

    /// `exposed ≥ 5%` and `applied ≥ 1%` of a nonzero sample.
    #[must_use]
    pub fn gate_met(self) -> bool {
        self.sampled > 0
            && self.exposed as f64 >= MIN_EXPOSED * self.sampled as f64
            && self.applied as f64 >= MIN_APPLIED * self.sampled as f64
    }
}

/// One family's reading in one world.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WorldVerdict {
    pub verdict: Verdict,
    pub exposure: Exposure,
    pub informative: u32,
    /// Replicates with `a / z > 1`, and with `a / z < 1.05`.
    pub above_one: u32,
    pub below_line: u32,
    /// Per-replicate ratios, `None` when both counts are zero.
    pub ratios: Vec<Option<f64>>,
    pub pooled: Option<f64>,
    /// Sign-test p-values of `above_one` and `below_line` over every replicate.
    pub p_above_one: f64,
    pub p_below_line: f64,
}

/// Apply the verdict rules to one family in one world. `pairs` holds each
/// replicate's `(a, z)`; the "7 of 8" counts run over every replicate, with
/// informativeness a separate requirement.
#[must_use]
pub fn world_verdict(applicable: bool, exposure: Exposure, pairs: &[(u64, u64)]) -> WorldVerdict {
    let ratios: Vec<Option<f64>> = pairs.iter().map(|&(a, z)| ratio(a, z)).collect();
    let informative = pairs
        .iter()
        .filter(|&&(a, z)| a + z >= MIN_INFORMATIVE_BIRTHS)
        .count() as u32;
    let above_one = ratios.iter().filter(|r| r.is_some_and(|r| r > 1.0)).count() as u32;
    let below_line = ratios
        .iter()
        .filter(|r| r.is_some_and(|r| r < RATIO_LINE))
        .count() as u32;
    let (sum_a, sum_z) = pairs
        .iter()
        .fold((0, 0), |(sa, sz), &(a, z)| (sa + a, sz + z));
    let pooled = ratio(sum_a, sum_z);
    let replicates = pairs.len() as u32;
    let enough = informative >= MIN_INFORMATIVE;
    let verdict = if !applicable {
        Verdict::NotApplicable
    } else if !exposure.gate_met() {
        Verdict::InconclusiveExposure
    } else if enough && above_one >= MIN_AGREEING && pooled.is_some_and(|p| p >= RATIO_LINE) {
        Verdict::Positive
    } else if enough && below_line >= MIN_AGREEING && pooled.is_some_and(|p| p < RATIO_LINE) {
        Verdict::Negative
    } else {
        Verdict::Inconclusive
    };
    WorldVerdict {
        verdict,
        exposure,
        informative,
        above_one,
        below_line,
        ratios,
        pooled,
        p_above_one: sign_test_p(above_one, replicates),
        p_below_line: sign_test_p(below_line, replicates),
    }
}

/// A family's verdict over its applicable worlds: positive when any is
/// positive, negative when all are negative, otherwise inconclusive.
#[must_use]
pub fn family_verdict(worlds: &[Verdict]) -> Verdict {
    let applicable: Vec<Verdict> = worlds
        .iter()
        .copied()
        .filter(|&v| v != Verdict::NotApplicable)
        .collect();
    if applicable.contains(&Verdict::Positive) {
        Verdict::Positive
    } else if !applicable.is_empty() && applicable.iter().all(|&v| v == Verdict::Negative) {
        Verdict::Negative
    } else {
        Verdict::Inconclusive
    }
}

/// The F02–F05 gate: at least two positive families, one of them non-ring,
/// each with a Graph-feasible controller. `families` holds
/// `(is_ring, verdict, graph_feasible)`.
#[must_use]
pub fn gate_favorable(families: &[(bool, Verdict, bool)]) -> bool {
    let positive: Vec<bool> = families
        .iter()
        .filter(|(_, verdict, feasible)| *verdict == Verdict::Positive && *feasible)
        .map(|(ring, _, _)| *ring)
        .collect();
    positive.len() >= 2 && positive.iter().any(|ring| !ring)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    const GATE: Exposure = Exposure {
        sampled: 100,
        exposed: 5,
        applied: 1,
    };

    #[test]
    fn seven_of_eight_is_the_documented_tail() {
        assert!((sign_test_p(7, 8) - 9.0 / 256.0).abs() < 1e-12);
        assert!((sign_test_p(0, 8) - 1.0).abs() < 1e-12);
        assert_eq!(sign_test_p(9, 8), 0.0);
    }

    #[test]
    fn ratios_follow_the_zero_rules() {
        assert_eq!(ratio(0, 0), None);
        assert_eq!(ratio(3, 0), Some(f64::INFINITY));
        assert_eq!(ratio(3, 2), Some(1.5));
    }

    #[test]
    fn the_exposure_gate_needs_five_and_one_percent_of_a_sample() {
        assert!(GATE.gate_met());
        assert!(!Exposure { exposed: 4, ..GATE }.gate_met());
        assert!(!Exposure { applied: 0, ..GATE }.gate_met());
        assert!(!Exposure::default().gate_met());
        let blocked = world_verdict(true, Exposure::default(), &[(30, 10); 8]);
        assert_eq!(blocked.verdict, Verdict::InconclusiveExposure);
    }

    #[test]
    fn a_consistent_advantage_is_positive_and_a_null_is_negative() {
        let positive = world_verdict(true, GATE, &[(30, 20); 8]);
        assert_eq!(positive.verdict, Verdict::Positive);
        assert_eq!(positive.above_one, 8);
        let negative = world_verdict(true, GATE, &[(20, 20); 8]);
        assert_eq!(negative.verdict, Verdict::Negative);
        assert_eq!(negative.below_line, 8);
    }

    #[test]
    fn two_disagreeing_replicates_or_too_few_births_are_inconclusive() {
        let mut pairs = [(30, 20); 8];
        pairs[0] = (10, 20);
        pairs[1] = (10, 20);
        assert_eq!(
            world_verdict(true, GATE, &pairs).verdict,
            Verdict::Inconclusive
        );
        let thin = world_verdict(true, GATE, &[(9, 5); 8]);
        assert_eq!(thin.informative, 0);
        assert_eq!(thin.verdict, Verdict::Inconclusive);
        // Above one everywhere but pooled below the line.
        assert_eq!(
            world_verdict(true, GATE, &[(101, 100); 8]).verdict,
            Verdict::Negative
        );
    }

    #[test]
    fn not_applicable_pairs_keep_their_ratio_but_no_verdict() {
        let reading = world_verdict(false, GATE, &[(30, 20); 8]);
        assert_eq!(reading.verdict, Verdict::NotApplicable);
        assert_eq!(reading.pooled, Some(1.5));
    }

    #[test]
    fn family_and_gate_rules() {
        use Verdict::*;
        assert_eq!(
            family_verdict(&[NotApplicable, Negative, Positive]),
            Positive
        );
        assert_eq!(
            family_verdict(&[NotApplicable, Negative, Negative]),
            Negative
        );
        assert_eq!(
            family_verdict(&[Negative, Inconclusive, Negative]),
            Inconclusive
        );
        assert_eq!(family_verdict(&[NotApplicable]), Inconclusive);
        assert!(gate_favorable(&[
            (true, Positive, true),
            (false, Positive, true),
            (false, Negative, true)
        ]));
        assert!(!gate_favorable(&[
            (true, Positive, true),
            (false, Positive, false)
        ]));
        assert!(!gate_favorable(&[(false, Positive, true)]));
    }

    proptest! {
        /// The tail is a probability, decreasing in `k`.
        #[test]
        fn the_sign_tail_is_monotone(n in 0u32..20, k in 0u32..20) {
            let p = sign_test_p(k, n);
            prop_assert!((0.0..=1.0 + 1e-12).contains(&p));
            prop_assert!(sign_test_p(k + 1, n) <= p + 1e-12);
        }

        /// Positive and negative never hold together, and a verdict never
        /// changes when replicates are reordered.
        #[test]
        fn verdicts_are_exclusive_and_order_free(
            pairs in proptest::collection::vec((0u64..60, 0u64..60), 8),
        ) {
            let reading = world_verdict(true, GATE, &pairs);
            let mut reversed = pairs.clone();
            reversed.reverse();
            prop_assert_eq!(world_verdict(true, GATE, &reversed).verdict, reading.verdict);
            let positive_rule = reading.informative >= MIN_INFORMATIVE
                && reading.above_one >= MIN_AGREEING
                && reading.pooled.is_some_and(|p| p >= RATIO_LINE);
            prop_assert_eq!(reading.verdict == Verdict::Positive, positive_rule);
        }
    }
}
