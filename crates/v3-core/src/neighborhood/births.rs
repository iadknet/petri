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
    pub any_events: Tally,
    pub by_events: BTreeMap<u32, Tally>,
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

    let outcomes: Vec<Option<(u32, Tally)>> = (0..births)
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
                return None;
            }
            let signature =
                battery.signature(&genome, context.runtime, context.shared_memory_decay_rate);
            let tally = Tally::default().record(classify(base, &signature));
            Some((summary.applied_events, tally))
        })
        .collect();

    fold_outcomes(births, outcomes)
}

/// Fold one birth per entry (`None` for a zero-applied-event birth,
/// `Some((applied_events, tally))` otherwise) into a [`BirthResult`]: pure
/// integer accounting, independent of how the outcomes were produced, so it
/// is directly proptestable without a genome, battery, or mutation engine.
fn fold_outcomes(births_total: u32, outcomes: Vec<Option<(u32, Tally)>>) -> BirthResult {
    let mut result = BirthResult {
        births_total,
        ..BirthResult::default()
    };
    for outcome in outcomes {
        match outcome {
            None => result.zero_event_births += 1,
            Some((applied_events, tally)) => {
                result.any_events = result.any_events.merge(tally);
                let bucket = result.by_events.entry(applied_events).or_default();
                *bucket = bucket.merge(tally);
            }
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
                None,
                None,
                Some((2, tally_of_one(Class::Changed))),
                None,
                None,
            ],
        );
        assert_eq!(result.births_total, 5);
        assert_eq!(result.zero_event_births, 4);
        assert_eq!(result.any_events.trials, 1);
        assert_eq!(result.by_events.len(), 1);
        assert_eq!(result.by_events[&2].trials, 1);
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
        // At the production ~8.8% mutated rate, a handful of births leaves a
        // real chance every birth in the sample is zero-event under both the
        // correct and a mutated seed formula (the two `BirthResult`s would
        // then coincidentally match). 120 births drives that chance to
        // effectively zero while staying well under a second.
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
        /// The pooled "any events" tally always equals the by-construction
        /// merge of every applied-event-count bucket, and every birth is
        /// accounted for exactly once (as a zero-event birth or in exactly
        /// one bucket) — independent of which applied-event counts and
        /// classes proptest draws.
        #[test]
        fn bucket_totals_equal_the_overall_any_events_tally(
            outcomes in prop::collection::vec(
                prop::option::of((1u32..5, prop_oneof![
                    Just(Class::Silent), Just(Class::Changed), Just(Class::Dead),
                ])),
                0..40,
            ),
        ) {
            let births_total = outcomes.len() as u32;
            let expanded: Vec<Option<(u32, Tally)>> = outcomes
                .into_iter()
                .map(|entry| entry.map(|(applied_events, class)| (applied_events, tally_of_one(class))))
                .collect();
            let zero_events_expected = expanded.iter().filter(|o| o.is_none()).count() as u32;
            let mutated_expected = expanded.iter().filter(|o| o.is_some()).count() as u32;

            let result = fold_outcomes(births_total, expanded);

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
