//! Classification of one mutated signature against its unmutated base, and
//! the integer-only tally that aggregates many classifications.
//!
//! Every accumulator here is an integer count summed in a fixed order, so a
//! tally built by folding trials in parallel (rayon) is byte-identical to one
//! built sequentially, regardless of thread count: integer addition is
//! associative and commutative, unlike a parallel floating-point sum.

use crate::contracts::WorldAction;
use crate::neighborhood::battery::Signature;

/// How a mutated signature compares with its unmutated base.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    /// Identical to the base on every execution.
    Silent,
    /// Differs somewhere, and at least one execution still emits a non-`NoOp`
    /// action.
    Changed,
    /// Differs somewhere, and every execution on the mutated genome is
    /// `NoOp` (including an empty action queue, which is vacuously all-`NoOp`).
    Dead,
}

fn is_all_noop(actions: &[WorldAction]) -> bool {
    actions
        .iter()
        .all(|action| matches!(action, WorldAction::NoOp))
}

/// One classification's full detail: the class, whether the two signatures
/// agree on every snapshot but differ in a sequence, and how many of the
/// signature's executions differ.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Classification {
    pub class: Class,
    pub changed_only_in_sequences: bool,
    pub differing_executions: u32,
    pub total_executions: u32,
}

/// Classify `candidate` against `base`. Pure and total: never touches a
/// genome, mutator, or RNG.
#[must_use]
pub fn classify(base: &Signature, candidate: &Signature) -> Classification {
    let snapshot_diff = base
        .snapshots
        .iter()
        .zip(candidate.snapshots.iter())
        .filter(|(a, b)| a != b)
        .count() as u32;
    let sequence_diff = base
        .sequences
        .iter()
        .zip(candidate.sequences.iter())
        .flat_map(|(a, b)| a.iter().zip(b.iter()))
        .filter(|(a, b)| a != b)
        .count() as u32;
    let differing_executions = snapshot_diff + sequence_diff;
    let total_executions = candidate.execution_count() as u32;

    let class = if differing_executions == 0 {
        Class::Silent
    } else {
        let all_noop = candidate
            .snapshots
            .iter()
            .all(|actions| is_all_noop(actions))
            && candidate
                .sequences
                .iter()
                .all(|sequence| sequence.iter().all(|actions| is_all_noop(actions)));
        if all_noop {
            Class::Dead
        } else {
            Class::Changed
        }
    };

    Classification {
        class,
        changed_only_in_sequences: snapshot_diff == 0 && sequence_diff > 0,
        differing_executions,
        total_executions,
    }
}

/// An integer-only tally over many trials of one treatment (one operator, or
/// one applied-event-count bucket of births). Every field is a plain count;
/// [`Tally::merge`] is the fold's combine step and is exactly integer
/// addition, so folding in any order or any chunking yields the same result.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Tally {
    pub trials: u32,
    pub skipped: u32,
    pub silent: u32,
    pub changed: u32,
    pub dead: u32,
    pub changed_only_in_sequences: u32,
    /// Sum of `differing_executions` over every changed-or-dead trial, so the
    /// mean fraction of executions that differ is computed once at format
    /// time from two integers rather than accumulated as a running float.
    pub differing_executions_total: u64,
    /// Sum of `total_executions` over the same changed-or-dead trials: the
    /// denominator [`Tally::mean_fraction_differing`] divides by, so the
    /// fraction never depends on a separately supplied battery size.
    pub total_executions_total: u64,
}

impl Tally {
    /// Record one attempted trial that could not apply (`MutationSkipReason`).
    #[must_use]
    pub fn skip(mut self) -> Self {
        self.trials += 1;
        self.skipped += 1;
        self
    }

    /// Record one applied trial's classification.
    #[must_use]
    pub fn record(mut self, classification: Classification) -> Self {
        self.trials += 1;
        match classification.class {
            Class::Silent => self.silent += 1,
            Class::Changed => self.changed += 1,
            Class::Dead => self.dead += 1,
        }
        if !matches!(classification.class, Class::Silent) {
            self.differing_executions_total += u64::from(classification.differing_executions);
            self.total_executions_total += u64::from(classification.total_executions);
        }
        if classification.changed_only_in_sequences {
            self.changed_only_in_sequences += 1;
        }
        self
    }

    /// Combine two tallies. Associative and commutative: the sole means by
    /// which parallel folds recombine, so it is the only place thread-count
    /// dependence could enter, and every field here is integer addition.
    #[must_use]
    pub fn merge(mut self, other: Self) -> Self {
        self.trials += other.trials;
        self.skipped += other.skipped;
        self.silent += other.silent;
        self.changed += other.changed;
        self.dead += other.dead;
        self.changed_only_in_sequences += other.changed_only_in_sequences;
        self.differing_executions_total += other.differing_executions_total;
        self.total_executions_total += other.total_executions_total;
        self
    }

    #[must_use]
    pub fn applied(&self) -> u32 {
        self.trials - self.skipped
    }

    /// Mean fraction of a signature's executions that differ, among changed
    /// and dead trials only. `0.0` when there are none (all silent, all
    /// skipped, or no trials), matching the empty-population convention used
    /// elsewhere in the report (a defined zero, not `Undefined`, since the
    /// denominator here is trial count, never population size). The
    /// denominator is this tally's own recorded `total_executions_total`, so
    /// the fraction never depends on a separately supplied battery size.
    #[must_use]
    pub fn mean_fraction_differing(&self) -> f64 {
        if self.total_executions_total == 0 {
            return 0.0;
        }
        self.differing_executions_total as f64 / self.total_executions_total as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::Direction;
    use proptest::prelude::*;

    fn action(differs: bool) -> Vec<WorldAction> {
        if differs {
            vec![WorldAction::Move(Direction::N)]
        } else {
            vec![WorldAction::NoOp]
        }
    }

    fn signature_from_bits(snapshot_bits: &[bool], sequence_bits: &[Vec<bool>]) -> Signature {
        Signature {
            snapshots: snapshot_bits.iter().map(|&b| action(b)).collect(),
            sequences: sequence_bits
                .iter()
                .map(|seq| seq.iter().map(|&b| action(b)).collect())
                .collect(),
        }
    }

    #[test]
    fn identical_signatures_classify_as_silent() {
        let sig = signature_from_bits(&[false, true], &[vec![false, true, true]]);
        let result = classify(&sig, &sig);
        assert_eq!(result.class, Class::Silent);
        assert_eq!(result.differing_executions, 0);
        assert!(!result.changed_only_in_sequences);
    }

    #[test]
    fn a_snapshot_difference_with_a_non_noop_action_is_changed() {
        let base = signature_from_bits(&[false, false], &[vec![false]]);
        let candidate = signature_from_bits(&[true, false], &[vec![false]]);
        let result = classify(&base, &candidate);
        assert_eq!(result.class, Class::Changed);
        assert_eq!(result.differing_executions, 1);
        assert!(!result.changed_only_in_sequences);
    }

    #[test]
    fn every_execution_noop_and_differing_is_dead_even_with_an_empty_action_queue() {
        let base = signature_from_bits(&[true], &[]);
        let candidate = Signature {
            snapshots: vec![Vec::new()],
            sequences: Vec::new(),
        };
        let result = classify(&base, &candidate);
        assert_eq!(
            result.class,
            Class::Dead,
            "an empty action queue is vacuously all-NoOp"
        );
    }

    #[test]
    fn changed_only_in_sequences_requires_identical_snapshots_and_a_differing_sequence() {
        let base = signature_from_bits(&[false, false], &[vec![false, false]]);
        let sequence_only_diff = signature_from_bits(&[false, false], &[vec![true, false]]);
        let result = classify(&base, &sequence_only_diff);
        assert!(result.changed_only_in_sequences);
        assert_eq!(result.class, Class::Changed);

        let snapshot_diff = signature_from_bits(&[true, false], &[vec![false, false]]);
        let result_with_snapshot_diff = classify(&base, &snapshot_diff);
        assert!(!result_with_snapshot_diff.changed_only_in_sequences);
    }

    #[test]
    fn tally_applied_is_trials_minus_skipped() {
        let tally = Tally::default().skip().skip().record(Classification {
            class: Class::Silent,
            changed_only_in_sequences: false,
            differing_executions: 0,
            total_executions: 80,
        });
        assert_eq!(tally.trials, 3);
        assert_eq!(tally.skipped, 2);
        assert_eq!(tally.applied(), 1);
    }

    #[test]
    fn mean_fraction_differing_is_zero_with_no_changed_or_dead_trials() {
        let tally = Tally::default().skip().record(Classification {
            class: Class::Silent,
            changed_only_in_sequences: false,
            differing_executions: 0,
            total_executions: 80,
        });
        assert_eq!(tally.mean_fraction_differing(), 0.0);
    }

    #[test]
    fn mean_fraction_differing_divides_the_recorded_totals() {
        let tally = Tally::default().record(Classification {
            class: Class::Changed,
            changed_only_in_sequences: false,
            differing_executions: 5,
            total_executions: 10,
        });
        assert_eq!(tally.mean_fraction_differing(), 0.5);
    }

    /// Every field of `record` starts at `0` (`Tally::default`), so `+=` and
    /// `*=` diverge on the very first call: `0 += 1 == 1` but `0 *= 1 == 0`.
    /// Two calls from different classes pin every field to a concrete count,
    /// including the `!matches!(.., Silent)` guard around the execution
    /// totals (a `Silent`-only counter-check would not distinguish it).
    #[test]
    fn record_accumulates_every_field_by_addition_not_multiplication() {
        let tally = Tally::default().record(Classification {
            class: Class::Changed,
            changed_only_in_sequences: true,
            differing_executions: 3,
            total_executions: 4,
        });
        assert_eq!(tally.silent, 0);
        assert_eq!(tally.changed, 1);
        assert_eq!(tally.dead, 0);
        assert_eq!(tally.changed_only_in_sequences, 1);
        assert_eq!(tally.differing_executions_total, 3);
        assert_eq!(tally.total_executions_total, 4);

        let tally = tally.record(Classification {
            class: Class::Dead,
            changed_only_in_sequences: false,
            differing_executions: 2,
            total_executions: 5,
        });
        assert_eq!(tally.changed, 1);
        assert_eq!(tally.dead, 1);
        assert_eq!(
            tally.changed_only_in_sequences, 1,
            "unchanged by the second, non-changed_only_in_sequences call"
        );
        assert_eq!(tally.differing_executions_total, 5);
        assert_eq!(tally.total_executions_total, 9);
    }

    /// `merge` is field-wise integer addition; commutativity and
    /// associativity alone (asserted above) do not distinguish it from
    /// multiplication, which satisfies both algebraic laws too. Concrete,
    /// non-0/1 field values pin it down: addition and multiplication give
    /// different results for them.
    #[test]
    fn merge_adds_every_field_rather_than_multiplying() {
        let a = Tally {
            trials: 2,
            skipped: 3,
            silent: 4,
            changed: 5,
            dead: 6,
            changed_only_in_sequences: 7,
            differing_executions_total: 8,
            total_executions_total: 9,
        };
        let b = Tally {
            trials: 10,
            skipped: 11,
            silent: 12,
            changed: 13,
            dead: 14,
            changed_only_in_sequences: 15,
            differing_executions_total: 16,
            total_executions_total: 17,
        };
        let merged = a.merge(b);
        assert_eq!(merged.trials, 12);
        assert_eq!(merged.skipped, 14);
        assert_eq!(merged.silent, 16);
        assert_eq!(merged.changed, 18);
        assert_eq!(merged.dead, 20);
        assert_eq!(merged.changed_only_in_sequences, 22);
        assert_eq!(merged.differing_executions_total, 24);
        assert_eq!(merged.total_executions_total, 26);
    }

    proptest! {
        #[test]
        fn a_signature_equal_to_its_base_is_always_silent(
            snapshot_bits in prop::collection::vec(any::<bool>(), 0..8),
            sequence_bits in prop::collection::vec(prop::collection::vec(any::<bool>(), 0..4), 0..3),
        ) {
            let sig = signature_from_bits(&snapshot_bits, &sequence_bits);
            let result = classify(&sig, &sig);
            prop_assert_eq!(result.class, Class::Silent);
            prop_assert_eq!(result.differing_executions, 0);
            prop_assert!(!result.changed_only_in_sequences);
        }

        #[test]
        fn an_all_noop_differing_signature_is_always_dead(
            base_snapshot_bits in prop::collection::vec(any::<bool>(), 1..8),
            base_sequence_bits in prop::collection::vec(prop::collection::vec(any::<bool>(), 1..4), 0..3),
        ) {
            let any_true = base_snapshot_bits.iter().any(|&b| b)
                || base_sequence_bits.iter().any(|seq| seq.iter().any(|&b| b));
            prop_assume!(any_true, "the base must differ somewhere from an all-NoOp candidate");

            let base = signature_from_bits(&base_snapshot_bits, &base_sequence_bits);
            let all_false_snapshots = vec![false; base_snapshot_bits.len()];
            let all_false_sequences: Vec<Vec<bool>> =
                base_sequence_bits.iter().map(|seq| vec![false; seq.len()]).collect();
            let candidate = signature_from_bits(&all_false_snapshots, &all_false_sequences);

            let result = classify(&base, &candidate);
            prop_assert_eq!(result.class, Class::Dead);
        }

        /// `Tally::merge` is exactly field-wise integer addition, so folding in
        /// any order — the property parallel evaluation with rayon relies on —
        /// yields the same tally regardless of grouping.
        #[test]
        fn tally_merge_is_commutative_and_associative(
            a in tally_strategy(), b in tally_strategy(), c in tally_strategy(),
        ) {
            prop_assert_eq!(a.merge(b), b.merge(a));
            prop_assert_eq!(a.merge(b).merge(c), a.merge(b.merge(c)));
        }
    }

    fn tally_strategy() -> impl Strategy<Value = Tally> {
        (
            0u32..20,
            0u32..20,
            0u32..20,
            0u32..20,
            0u32..20,
            0u64..500,
            0u64..2000,
        )
            .prop_map(
                |(
                    skipped,
                    silent,
                    changed,
                    dead,
                    changed_only_in_sequences,
                    differing_executions_total,
                    total_executions_total,
                )| Tally {
                    trials: skipped + silent + changed + dead,
                    skipped,
                    silent,
                    changed,
                    dead,
                    changed_only_in_sequences,
                    differing_executions_total,
                    total_executions_total,
                },
            )
    }
}
