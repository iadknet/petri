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
use crate::creature::genome::analysis::{mesh_cycle_nodes, mesh_reachable_nodes};
use crate::creature::genome::CreatureGenome;
use crate::mutation::reachability::ParentExecuted;
use crate::mutation::MutationEngine;
use crate::neighborhood::battery::{Battery, Signature};
use crate::neighborhood::classify::{classify, Tally};
use crate::neighborhood::mutation_effects::genome_identity;
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
    /// `any_events` partitioned by whether the offspring's reachable mesh
    /// carries a cycle (T19.F02): `cycle_carrying` merged with `acyclic`
    /// equals `any_events`.
    pub cycle_carrying: Tally,
    pub acyclic: Tally,
}

/// One classified birth: requested and applied event counts, the tally of
/// its class (empty for a zero-event birth), and whether the offspring's
/// reachable mesh carries a cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BirthOutcome {
    pub requested_events: u32,
    pub applied_events: u32,
    pub tally: Tally,
    pub cycle_carrying: bool,
}

impl BirthResult {
    /// Pool complete per-birth integer accounting without changing denominators.
    #[must_use]
    pub fn merge(mut self, other: &Self) -> Self {
        self.births_total += other.births_total;
        self.zero_event_births += other.zero_event_births;
        self.any_events = self.any_events.merge(other.any_events);
        self.cycle_carrying = self.cycle_carrying.merge(other.cycle_carrying);
        self.acyclic = self.acyclic.merge(other.acyclic);
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

/// Distinct-queue buckets of the exposure histogram (T11.F26): 1, 2, 3–4,
/// 5–8, 9+.
pub const QUEUE_BUCKETS: [&str; 5] = ["1", "2", "3-4", "5-8", "9+"];

/// The [`QUEUE_BUCKETS`] position of a parent with `distinct` queues; `0`
/// for an empty signature.
#[must_use]
pub fn queue_bucket(distinct: usize) -> usize {
    match distinct {
        0..=1 => 0,
        2 => 1,
        3..=4 => 2,
        5..=8 => 3,
        _ => 4,
    }
}

/// One parent-capability side of the exposure split: that side's births by
/// outcome, zero-applied births included.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CapabilitySplit {
    pub silent: u32,
    pub changed: u32,
    pub dead: u32,
    pub zero_applied: u32,
}

impl CapabilitySplit {
    #[must_use]
    pub fn merge(self, other: Self) -> Self {
        Self {
            silent: self.silent + other.silent,
            changed: self.changed + other.changed,
            dead: self.dead + other.dead,
            zero_applied: self.zero_applied + other.zero_applied,
        }
    }
}

/// Exposure strata (T11.F26) folded from births a reading already runs:
/// parent capability, the partition of births by requested and applied
/// events, and event-bearing births whose child genome is identical to the
/// parent. Integer-only; [`BirthExposure::merge`] is field-wise addition.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BirthExposure {
    pub parents: u32,
    /// Parents whose every battery execution is `NoOp`.
    pub parents_all_noop: u32,
    /// Parents with exactly one distinct action queue across the battery.
    pub parents_one_queue: u32,
    /// Parents by distinct-queue count, bucketed by [`QUEUE_BUCKETS`].
    pub distinct_queues_histogram: [u32; 5],
    pub births_total: u32,
    /// Births that drew no event.
    pub zero_requested: u32,
    /// Births that drew events and applied none.
    pub requested_all_skipped: u32,
    /// Births that applied at least one event.
    pub event_bearing: u32,
    pub requested_events_total: u64,
    pub applied_events_total: u64,
    /// Event-bearing births whose child genome equals the parent under
    /// [`super::mutation_effects::genome_identity`].
    pub genome_identical: u32,
    pub from_actionless: CapabilitySplit,
    pub from_acting: CapabilitySplit,
}

impl BirthExposure {
    #[must_use]
    pub fn merge(mut self, other: &Self) -> Self {
        self.parents += other.parents;
        self.parents_all_noop += other.parents_all_noop;
        self.parents_one_queue += other.parents_one_queue;
        for (bucket, count) in self
            .distinct_queues_histogram
            .iter_mut()
            .zip(other.distinct_queues_histogram)
        {
            *bucket += count;
        }
        self.births_total += other.births_total;
        self.zero_requested += other.zero_requested;
        self.requested_all_skipped += other.requested_all_skipped;
        self.event_bearing += other.event_bearing;
        self.requested_events_total += other.requested_events_total;
        self.applied_events_total += other.applied_events_total;
        self.genome_identical += other.genome_identical;
        self.from_actionless = self.from_actionless.merge(other.from_actionless);
        self.from_acting = self.from_acting.merge(other.from_acting);
        self
    }
}

/// Fold one parent's births into exposure strata. `outcomes` pairs each
/// birth with whether its child genome is identical to the parent.
fn fold_exposure(base: &Signature, outcomes: &[(BirthOutcome, bool)]) -> BirthExposure {
    let actionless = base.all_noop();
    let distinct = base.distinct_queue_count();
    let mut exposure = BirthExposure {
        parents: 1,
        parents_all_noop: u32::from(actionless),
        parents_one_queue: u32::from(distinct == 1),
        births_total: outcomes.len() as u32,
        ..BirthExposure::default()
    };
    exposure.distinct_queues_histogram[queue_bucket(distinct)] = 1;
    let side = if actionless {
        &mut exposure.from_actionless
    } else {
        &mut exposure.from_acting
    };
    for (outcome, _) in outcomes {
        side.silent += outcome.tally.silent;
        side.changed += outcome.tally.changed;
        side.dead += outcome.tally.dead;
        if outcome.applied_events == 0 {
            side.zero_applied += 1;
        }
    }
    for (outcome, identical) in outcomes {
        exposure.requested_events_total += u64::from(outcome.requested_events);
        exposure.applied_events_total += u64::from(outcome.applied_events);
        if outcome.requested_events == 0 {
            exposure.zero_requested += 1;
        } else if outcome.applied_events == 0 {
            exposure.requested_all_skipped += 1;
        } else {
            exposure.event_bearing += 1;
            exposure.genome_identical += u32::from(*identical);
        }
    }
    exposure
}

/// Run `births` seeded production births from `subject`, seeded by
/// `seed_offset + BIRTH_SEED_BASE + birth_index`, and classify every
/// mutated (nonzero-event) offspring against `base`. Each birth draws its
/// supply on the subject's own `genome_size()`, as production does.
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
    per_birth_result_on_units(
        subject,
        subject.genome_size(),
        base,
        battery,
        mutation_config,
        context,
        births,
        seed_offset,
    )
}

/// [`per_birth_result`] and the exposure strata of the same births (T11.F26).
#[must_use]
pub fn per_birth_reading(
    subject: &CreatureGenome,
    base: &Signature,
    battery: &Battery,
    mutation_config: &MutationConfig,
    context: &EvalContext,
    births: u32,
    seed_offset: u64,
) -> (BirthResult, BirthExposure) {
    per_birth_reading_on_units(
        subject,
        subject.genome_size(),
        base,
        battery,
        mutation_config,
        context,
        births,
        seed_offset,
    )
}

/// [`per_birth_result`] with every birth's supply drawn on `units` instead of
/// the subject's own size: the drift walk's checkpoint births pass the
/// instrument constant `FOUNDER_GENOME_SIZE_UNITS` (T11.F20).
#[allow(
    clippy::too_many_arguments,
    reason = "per_birth_result's seven inputs plus the supply unit count"
)]
#[must_use]
pub fn per_birth_result_on_units(
    subject: &CreatureGenome,
    units: u32,
    base: &Signature,
    battery: &Battery,
    mutation_config: &MutationConfig,
    context: &EvalContext,
    births: u32,
    seed_offset: u64,
) -> BirthResult {
    per_birth_reading_on_units(
        subject,
        units,
        base,
        battery,
        mutation_config,
        context,
        births,
        seed_offset,
    )
    .0
}

/// [`per_birth_result_on_units`] and, from the same births, the exposure
/// strata of T11.F26. Draws exactly the same mutations: exposure only reads
/// each birth's summary and compares its child genome with the subject.
#[allow(
    clippy::too_many_arguments,
    reason = "per_birth_result_on_units's inputs, unchanged"
)]
#[must_use]
pub fn per_birth_reading_on_units(
    subject: &CreatureGenome,
    units: u32,
    base: &Signature,
    battery: &Battery,
    mutation_config: &MutationConfig,
    context: &EvalContext,
    births: u32,
    seed_offset: u64,
) -> (BirthResult, BirthExposure) {
    let reachable = mesh_reachable_nodes(subject);
    let identity = genome_identity(subject);
    // Observation stand-in for a live parent's dispatch record (T11.F17):
    // the nodes this subject dispatches anywhere in the battery.
    let executed =
        battery.executed_indices(subject, context.runtime, context.shared_memory_decay_rate);

    let outcomes: Vec<(BirthOutcome, bool)> = (0..births)
        .into_par_iter()
        .map(|birth_index| {
            let mut genome = subject.clone();
            let mut rng =
                SmallRng::seed_from_u64(seed_offset + BIRTH_SEED_BASE + u64::from(birth_index));
            let summary = MutationEngine::apply_mutations_on_units(
                &mut genome,
                units,
                mutation_config,
                &reachable,
                ParentExecuted::Indices(&executed),
                &mut rng,
                context.food_type_count,
            );
            let cycle_carrying = !mesh_cycle_nodes(&genome).is_empty();
            if summary.applied_events == 0 {
                let outcome = BirthOutcome {
                    requested_events: summary.attempted_events,
                    applied_events: 0,
                    tally: Tally::default(),
                    cycle_carrying,
                };
                return (outcome, false);
            }
            let signature =
                battery.signature(&genome, context.runtime, context.shared_memory_decay_rate);
            let outcome = BirthOutcome {
                requested_events: summary.attempted_events,
                applied_events: summary.applied_events,
                tally: Tally::default().record(classify(base, &signature)),
                cycle_carrying,
            };
            (outcome, genome_identity(&genome) == identity)
        })
        .collect();

    let exposure = fold_exposure(base, &outcomes);
    let result = fold_outcomes(
        births,
        outcomes.into_iter().map(|(outcome, _)| outcome).collect(),
    );
    (result, exposure)
}

/// Fold one [`BirthOutcome`] per birth into pure integer accounting.
/// Zero-applied births carry an empty tally and need no evaluation.
fn fold_outcomes(births_total: u32, outcomes: Vec<BirthOutcome>) -> BirthResult {
    let mut result = BirthResult {
        births_total,
        ..BirthResult::default()
    };
    for outcome in outcomes {
        *result
            .by_requested_events
            .entry(outcome.requested_events)
            .or_default() += 1;
        if outcome.applied_events == 0 {
            result.zero_event_births += 1;
        } else {
            let tally = outcome.tally;
            result.any_events = result.any_events.merge(tally);
            let bucket = result.by_events.entry(outcome.applied_events).or_default();
            *bucket = bucket.merge(tally);
            let partition = if outcome.cycle_carrying {
                &mut result.cycle_carrying
            } else {
                &mut result.acyclic
            };
            *partition = partition.merge(tally);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neighborhood::classify::{Class, Classification};
    use proptest::prelude::*;

    fn outcome(requested: u32, applied: u32, tally: Tally, cycle_carrying: bool) -> BirthOutcome {
        BirthOutcome {
            requested_events: requested,
            applied_events: applied,
            tally,
            cycle_carrying,
        }
    }

    fn tally_of_one(class: Class) -> Tally {
        Tally::default().record(Classification {
            class,
            changed_only_in_sequences: false,
            shape: crate::neighborhood::classify::ChangeShape::Other,
            differing_executions: u32::from(!matches!(class, Class::Silent)),
            total_executions: 80,
        })
    }

    #[test]
    fn zero_event_births_are_counted_but_not_folded_into_any_bucket() {
        let result = fold_outcomes(
            5,
            vec![
                outcome(0, 0, Tally::default(), false),
                outcome(0, 0, Tally::default(), true),
                outcome(3, 2, tally_of_one(Class::Changed), true),
                outcome(0, 0, Tally::default(), false),
                outcome(0, 0, Tally::default(), false),
            ],
        );
        assert_eq!(result.births_total, 5);
        assert_eq!(result.zero_event_births, 4);
        assert_eq!(result.any_events.trials, 1);
        assert_eq!(result.by_events.len(), 1);
        assert_eq!(result.by_events[&2].trials, 1);
        assert_eq!(result.by_requested_events, BTreeMap::from([(0, 4), (3, 1)]));
        assert_eq!(result.cycle_carrying, tally_of_one(Class::Changed));
        assert_eq!(
            result.acyclic,
            Tally::default(),
            "a zero-event birth is not classified, whatever its mesh shape"
        );
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
        let executed =
            battery.executed_indices(&subject, context.runtime, context.shared_memory_decay_rate);
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
                ParentExecuted::Indices(&executed),
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
            if mesh_cycle_nodes(&genome).is_empty() {
                expected.acyclic = expected.acyclic.merge(tally);
            } else {
                expected.cycle_carrying = expected.cycle_carrying.merge(tally);
            }
            let bucket = expected
                .by_events
                .entry(summary.applied_events)
                .or_default();
            *bucket = bucket.merge(tally);
        }

        assert_eq!(actual, expected);
    }

    /// The exposure strata come from the same births as the result: the
    /// wrapped result is unchanged, the births partition, the capability
    /// split reconciles with the pooled tally, and the parent's capability
    /// matches its signature.
    #[test]
    fn exposure_reconciles_with_the_birth_result_it_is_folded_beside() {
        use crate::config::SimulationConfig;
        use crate::creature::founder::founder_genome;

        let config = SimulationConfig::default();
        let subject = founder_genome(crate::config::FounderProfile::V3Alpha1);
        let battery = Battery::generate(config.world.food.types.len());
        let context = EvalContext::from_config(&config);
        let base = battery.signature(&subject, context.runtime, context.shared_memory_decay_rate);
        let (result, exposure) = per_birth_reading(
            &subject,
            &base,
            &battery,
            &config.mutation,
            &context,
            60,
            77,
        );

        assert_eq!(
            result,
            per_birth_result(
                &subject,
                &base,
                &battery,
                &config.mutation,
                &context,
                60,
                77
            )
        );
        assert_eq!(exposure.births_total, result.births_total);
        assert_eq!(
            exposure.zero_requested + exposure.requested_all_skipped + exposure.event_bearing,
            exposure.births_total
        );
        assert_eq!(
            exposure.zero_requested,
            result.by_requested_events.get(&0).copied().unwrap_or(0)
        );
        assert_eq!(exposure.event_bearing, result.any_events.trials);
        let split = exposure.from_actionless.merge(exposure.from_acting);
        assert_eq!(split.silent, result.any_events.silent);
        assert_eq!(split.changed, result.any_events.changed);
        assert_eq!(split.dead, result.any_events.dead);
        assert_eq!(split.zero_applied, result.zero_event_births);
        assert_eq!(exposure.parents, 1);
        assert_eq!(exposure.parents_all_noop, u32::from(base.all_noop()));
        assert_eq!(
            exposure.from_actionless,
            CapabilitySplit::default(),
            "the founder acts"
        );
        assert_eq!(
            exposure.distinct_queues_histogram[queue_bucket(base.distinct_queue_count())],
            1
        );
        assert!(exposure.applied_events_total <= exposure.requested_events_total);
    }

    #[test]
    fn queue_buckets_follow_the_predeclared_edges() {
        let buckets: Vec<usize> = [1, 2, 3, 4, 5, 8, 9, 80]
            .into_iter()
            .map(queue_bucket)
            .collect();
        assert_eq!(buckets, [0, 1, 2, 2, 3, 3, 4, 4]);
    }

    proptest! {
        /// Exposure folding partitions every birth exactly once, and its
        /// capability split sums to the birth fold's tallies, whatever the
        /// outcomes drawn.
        #[test]
        fn exposure_partitions_births_and_reconciles_with_the_fold(
            outcomes in prop::collection::vec(
                (0u32..3, prop::option::of((1u32..3, 0u8..3)), any::<bool>()),
                0..40,
            ),
            actionless in any::<bool>(),
        ) {
            let paired: Vec<(BirthOutcome, bool)> = outcomes
                .iter()
                .map(|&(skipped, applied, identical)| match applied {
                    Some((applied, class)) => (outcome(applied + skipped, applied, tally_of_one(match class { 0 => Class::Silent, 1 => Class::Changed, _ => Class::Dead }), false), identical),
                    None => (outcome(skipped, 0, Tally::default(), false), identical),
                })
                .collect();
            let action = if actionless {
                crate::contracts::WorldAction::NoOp
            } else {
                crate::contracts::WorldAction::Move(crate::contracts::Direction::ALL[0])
            };
            let base = Signature { snapshots: vec![vec![action]; 3], sequences: Vec::new() };
            let exposure = fold_exposure(&base, &paired);
            let result = fold_outcomes(paired.len() as u32, paired.iter().map(|(o, _)| *o).collect());
            prop_assert_eq!(exposure.zero_requested + exposure.requested_all_skipped + exposure.event_bearing, exposure.births_total);
            prop_assert_eq!(exposure.births_total, result.births_total);
            let split = exposure.from_actionless.merge(exposure.from_acting);
            prop_assert_eq!(split.silent, result.any_events.silent);
            prop_assert_eq!(split.changed, result.any_events.changed);
            prop_assert_eq!(split.dead, result.any_events.dead);
            prop_assert_eq!(split.zero_applied, result.zero_event_births);
            let empty = if actionless { exposure.from_acting } else { exposure.from_actionless };
            prop_assert_eq!(empty, CapabilitySplit::default());
            prop_assert!(exposure.genome_identical <= exposure.event_bearing);
            prop_assert_eq!(exposure.applied_events_total, paired.iter().map(|(o, _)| u64::from(o.applied_events)).sum::<u64>());
            let doubled = exposure.merge(&exposure);
            prop_assert_eq!(doubled.births_total, 2 * exposure.births_total);
            prop_assert_eq!(doubled.distinct_queues_histogram.iter().sum::<u32>(), 2);
        }

        #[test]
        fn pooling_preserves_all_counts_and_buckets_under_regrouping(
            outcomes in prop::collection::vec((0u32..4, 0u8..4, any::<bool>()), 0..50),
            split in 0usize..50,
        ) {
            let expanded: Vec<_> = outcomes.iter().map(|&(requested, class, cycle)| {
                if class == 0 { outcome(requested, 0, Tally::default(), cycle) }
                else { outcome(requested + 2, requested + 1, tally_of_one(match class { 1 => Class::Silent, 2 => Class::Changed, _ => Class::Dead }), cycle) }
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
        /// merge of every applied-event-count bucket and of the cycle
        /// partition, and every birth is accounted for exactly once (as a
        /// zero-event birth or in exactly one bucket) — independent of which
        /// applied-event counts, classes, and mesh shapes proptest draws.
        #[test]
        fn bucket_totals_equal_the_overall_any_events_tally(
            outcomes in prop::collection::vec(
                (0u32..4, prop::option::of((1u32..5, prop_oneof![
                    Just(Class::Silent), Just(Class::Changed), Just(Class::Dead),
                ])), any::<bool>()),
                0..40,
            ),
        ) {
            let births_total = outcomes.len() as u32;
            let expanded: Vec<BirthOutcome> = outcomes
                .into_iter()
                .map(|(skipped, entry, cycle)| match entry {
                    Some((applied, class)) => outcome(applied + skipped, applied, tally_of_one(class), cycle),
                    None => outcome(skipped, 0, Tally::default(), cycle),
                })
                .collect();
            let zero_events_expected = expanded.iter().filter(|o| o.applied_events == 0).count() as u32;
            let mutated_expected = births_total - zero_events_expected;
            let requested_total: u32 = expanded.iter().map(|o| o.requested_events).sum();
            let skipped_total: u32 = expanded.iter().map(|o| o.requested_events - o.applied_events).sum();
            let result = fold_outcomes(births_total, expanded);
            prop_assert_eq!(result.cycle_carrying.merge(result.acyclic), result.any_events);
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
