//! Per-birth treatment (T11.F01 Battery, "Per-birth treatment"): run the
//! production mutation engine on fresh copies of the subject through
//! [`MutationEngine::apply_mutations_with_food_type_count`], bucketed by
//! `applied_events`. Zero-event births (the mutation gate did not fire, or
//! every drawn event skipped) are counted, not evaluated — they are
//! identical to the base by construction.

use std::collections::BTreeMap;

use rand::rngs::SmallRng;
use rand::SeedableRng;
use rayon::prelude::*;

use crate::config::MutationConfig;
use crate::creature::genome::analysis::mesh_reachable_nodes;
use crate::creature::genome::CreatureGenome;
use crate::mutation::MutationEngine;
use crate::neighborhood::battery::{Battery, Signature};
use crate::neighborhood::classify::{classify, Tally};
use crate::neighborhood::EvalContext;

/// Seed base for per-birth trials.
pub const BIRTH_SEED_BASE: u64 = 9_000;

/// The per-birth reading: total births attempted, how many drew zero applied
/// events, the pooled tally over every birth with at least one applied event
/// ("any events"), and the same trials bucketed by their exact
/// `applied_events` count.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BirthResult {
    pub births_total: u32,
    pub zero_event_births: u32,
    /// Birth counts by requested (attempted) events, including zero.
    pub by_requested_events: BTreeMap<u32, u32>,
    pub any_events: Tally,
    pub by_events: BTreeMap<u32, Tally>,
}

impl BirthResult {
    /// Pool complete per-birth integer accounting without changing denominators.
    #[must_use]
    pub fn merge(mut self, other: &Self) -> Self {
        self.births_total += other.births_total;
        self.zero_event_births += other.zero_event_births;
        self.any_events = self.any_events.merge(other.any_events);
        for (&events, &births) in &other.by_requested_events {
            *self.by_requested_events.entry(events).or_default() += births;
        }
        for (&applied_events, tally) in &other.by_events {
            let entry = self.by_events.entry(applied_events).or_default();
            *entry = entry.merge(*tally);
        }
        self
    }
}

/// Run `births` seeded production births from `subject`, seeded by
/// `seed_offset + BIRTH_SEED_BASE + birth_index`, and classify every
/// mutated (nonzero-event) offspring against `base`.
#[must_use]
pub fn per_birth_result(
    subject: &CreatureGenome,
    base: &Signature,
    battery: &Battery,
    mutation_config: &MutationConfig,
    context: &EvalContext,
    births: u32,
    seed_offset: u64,
) -> BirthResult {
    let reachable = mesh_reachable_nodes(subject);

    let outcomes: Vec<(u32, u32, Tally)> = (0..births)
        .into_par_iter()
        .map(|birth_index| {
            let mut genome = subject.clone();
            let mut rng =
                SmallRng::seed_from_u64(seed_offset + BIRTH_SEED_BASE + u64::from(birth_index));
            let summary = MutationEngine::apply_mutations_with_food_type_count(
                &mut genome,
                mutation_config,
                &reachable,
                &mut rng,
                context.food_type_count,
            );
            if summary.applied_events == 0 {
                return (summary.attempted_events, 0, Tally::default());
            }
            let signature =
                battery.signature(&genome, context.runtime, context.shared_memory_decay_rate);
            let tally = Tally::default().record(classify(base, &signature));
            (summary.attempted_events, summary.applied_events, tally)
        })
        .collect();

    fold_outcomes(births, outcomes)
}

/// Fold `(requested_events, applied_events, tally)` per birth into pure integer
/// accounting. Zero-applied births carry an empty tally and need no evaluation.
fn fold_outcomes(births_total: u32, outcomes: Vec<(u32, u32, Tally)>) -> BirthResult {
    let mut result = BirthResult {
        births_total,
        ..BirthResult::default()
    };
    for (requested_events, applied_events, tally) in outcomes {
        *result
            .by_requested_events
            .entry(requested_events)
            .or_default() += 1;
        if applied_events == 0 {
            result.zero_event_births += 1;
        } else {
            result.any_events = result.any_events.merge(tally);
            let bucket = result.by_events.entry(applied_events).or_default();
            *bucket = bucket.merge(tally);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neighborhood::classify::{Class, Classification};
    use proptest::prelude::*;

    fn tally_of_one(class: Class) -> Tally {
        Tally::default().record(Classification {
            class,
            changed_only_in_sequences: false,
            differing_executions: u32::from(!matches!(class, Class::Silent)),
            total_executions: 80,
        })
    }

    #[test]
    fn zero_event_births_are_counted_but_not_folded_into_any_bucket() {
        let result = fold_outcomes(
            5,
            vec![
                (0, 0, Tally::default()),
                (0, 0, Tally::default()),
                (3, 2, tally_of_one(Class::Changed)),
                (0, 0, Tally::default()),
                (0, 0, Tally::default()),
            ],
        );
        assert_eq!(result.births_total, 5);
        assert_eq!(result.zero_event_births, 4);
        assert_eq!(result.any_events.trials, 1);
        assert_eq!(result.by_events.len(), 1);
        assert_eq!(result.by_events[&2].trials, 1);
        assert_eq!(result.by_requested_events, BTreeMap::from([(0, 4), (3, 1)]));
    }

    /// `per_birth_result` seeds birth `i` by
    /// `seed_offset + BIRTH_SEED_BASE + i`, and buckets a birth by
    /// `applied_events == 0` versus otherwise. This reconstructs the same
    /// per-birth outcome independently (using the public seed base and the
    /// same production mutation call) and checks it against the function's
    /// own output byte-for-byte, which pins both the seed arithmetic (a `-`
    /// or `*` in place of either `+` would almost certainly draw different
    /// mutations) and the `== 0` bucketing test (an `!= 0` would swap which
    /// births count as zero-event).
    #[test]
    fn per_birth_result_seeds_each_birth_by_offset_plus_base_plus_index() {
        use crate::config::SimulationConfig;
        use crate::creature::founder::founder_genome;
        use crate::mutation::MutationEngine;

        let config = SimulationConfig::default();
        let subject = founder_genome(crate::config::FounderProfile::V3Alpha1);
        let battery = Battery::generate(config.world.food.types.len());
        let context = EvalContext::from_config(&config);
        let base = battery.signature(&subject, context.runtime, context.shared_memory_decay_rate);
        let reachable = mesh_reachable_nodes(&subject);
        // Keep the established 120-birth seed-arithmetic fixture; its request
        // histogram now also distinguishes triggered-but-skipped births.
        let births = 120u32;
        let seed_offset = 555u64;

        let actual = per_birth_result(
            &subject,
            &base,
            &battery,
            &config.mutation,
            &context,
            births,
            seed_offset,
        );

        let mut expected = BirthResult {
            births_total: births,
            ..BirthResult::default()
        };
        for birth_index in 0..births {
            let mut genome = subject.clone();
            let mut rng =
                SmallRng::seed_from_u64(seed_offset + BIRTH_SEED_BASE + u64::from(birth_index));
            let summary = MutationEngine::apply_mutations_with_food_type_count(
                &mut genome,
                &config.mutation,
                &reachable,
                &mut rng,
                context.food_type_count,
            );
            *expected
                .by_requested_events
                .entry(summary.attempted_events)
                .or_default() += 1;
            if summary.applied_events == 0 {
                expected.zero_event_births += 1;
                continue;
            }
            let signature =
                battery.signature(&genome, context.runtime, context.shared_memory_decay_rate);
            let tally = Tally::default().record(classify(&base, &signature));
            expected.any_events = expected.any_events.merge(tally);
            let bucket = expected
                .by_events
                .entry(summary.applied_events)
                .or_default();
            *bucket = bucket.merge(tally);
        }

        assert_eq!(actual, expected);
    }

    proptest! {
        #[test]
        fn pooling_preserves_all_counts_and_buckets_under_regrouping(
            outcomes in prop::collection::vec((0u32..4, 0u8..4), 0..50),
            split in 0usize..50,
        ) {
            let expanded: Vec<_> = outcomes.iter().map(|&(requested, class)| {
                if class == 0 { (requested, 0, Tally::default()) }
                else { (requested + 2, requested + 1, tally_of_one(match class { 1 => Class::Silent, 2 => Class::Changed, _ => Class::Dead })) }
            }).collect();
            let split = split.min(expanded.len());
            let a = fold_outcomes(split as u32, expanded[..split].to_vec());
            let b = fold_outcomes((expanded.len() - split) as u32, expanded[split..].to_vec());
            let whole = fold_outcomes(expanded.len() as u32, expanded);
            prop_assert_eq!(a.clone().merge(&b), whole.clone());
            prop_assert_eq!(b.merge(&a), whole.clone());
            prop_assert_eq!(whole.clone().merge(&BirthResult::default()), whole);
        }

        /// The pooled "any events" tally always equals the by-construction
        /// merge of every applied-event-count bucket, and every birth is
        /// accounted for exactly once (as a zero-event birth or in exactly
        /// one bucket) — independent of which applied-event counts and
        /// classes proptest draws.
        #[test]
        fn bucket_totals_equal_the_overall_any_events_tally(
            outcomes in prop::collection::vec(
                (0u32..4, prop::option::of((1u32..5, prop_oneof![
                    Just(Class::Silent), Just(Class::Changed), Just(Class::Dead),
                ]))),
                0..40,
            ),
        ) {
            let births_total = outcomes.len() as u32;
            let expanded: Vec<(u32, u32, Tally)> = outcomes
                .into_iter()
                .map(|(skipped, entry)| match entry {
                    Some((applied, class)) => (applied + skipped, applied, tally_of_one(class)),
                    None => (skipped, 0, Tally::default()),
                })
                .collect();
            let zero_events_expected = expanded.iter().filter(|o| o.1 == 0).count() as u32;
            let mutated_expected = births_total - zero_events_expected;
            let requested_total: u32 = expanded.iter().map(|o| o.0).sum();
            let skipped_total: u32 = expanded.iter().map(|o| o.0 - o.1).sum();
            let result = fold_outcomes(births_total, expanded);
            prop_assert_eq!(result.by_requested_events.values().sum::<u32>(), births_total);
            prop_assert_eq!(result.by_requested_events.iter().map(|(events, births)| events * births).sum::<u32>(), requested_total);
            prop_assert_eq!(result.by_events.iter().map(|(events, tally)| events * tally.trials).sum::<u32>() + skipped_total, requested_total);

            prop_assert_eq!(result.zero_event_births, zero_events_expected);
            prop_assert_eq!(result.any_events.trials, mutated_expected);

            let merged_buckets = result
                .by_events
                .values()
                .fold(Tally::default(), |acc, tally| acc.merge(*tally));
            prop_assert_eq!(merged_buckets, result.any_events);
            prop_assert_eq!(
                result.zero_event_births + result.by_events.values().map(|t| t.trials).sum::<u32>(),
                result.births_total
            );
        }
    }
}
