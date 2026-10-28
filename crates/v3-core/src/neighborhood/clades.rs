//! The clade census: the living population counted by founder clade
//! (`lineage_id`), the diversity of that distribution, and the per-clade
//! profile of how each surviving clade made its living (T14.F07).
//!
//! Counting is keyed in a `BTreeMap`, so the counts never depend on how the
//! caller iterated the population. Clade entropy has one definition, summed
//! in lineage-id order, and the effective lineage count is its `exp`.

use std::collections::BTreeMap;

use crate::creature::action_log::ACTION_TYPE_COUNT;
use crate::creature::state::CreatureState;

/// Living creatures per founder clade, in lineage-id order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CladeCensus {
    pub counts: BTreeMap<u32, u64>,
}

impl CladeCensus {
    /// Count `lineage_ids`.
    pub fn of(lineage_ids: impl IntoIterator<Item = u32>) -> Self {
        let mut counts = BTreeMap::<u32, u64>::new();
        for lineage_id in lineage_ids {
            *counts.entry(lineage_id).or_default() += 1;
        }
        Self { counts }
    }

    /// Founder clades with at least one living creature.
    #[must_use]
    pub fn surviving(&self) -> u64 {
        self.counts.len() as u64
    }

    /// Shannon entropy of the clade distribution in nats, summed in
    /// lineage-id order: the one definition the benchmark sample,
    /// `lineage_diversity` and the effective lineage count read. `None` for
    /// an empty population.
    #[must_use]
    pub fn entropy_nats(&self) -> Option<f64> {
        let total: u64 = self.counts.values().sum();
        if total == 0 {
            return None;
        }
        let total = total as f64;
        Some(self.counts.values().fold(0.0, |acc, &count| {
            let probability = count as f64 / total;
            acc - probability * probability.ln()
        }))
    }

    /// The effective lineage count, `exp` of [`Self::entropy_nats`]: the
    /// number of equally common lineages with the same diversity (the
    /// server's `lineages_effective`). `None` for an empty population.
    #[must_use]
    pub fn effective_lineages(&self) -> Option<f64> {
        self.entropy_nats().map(f64::exp)
    }
}

/// One surviving clade's profile: its size, the means of its members'
/// energy, age, generation and genome size, and the sums of their lifetime
/// counters.
#[derive(Debug, Clone, PartialEq)]
pub struct CladeProfile {
    pub lineage_id: u32,
    pub size: u64,
    pub mean_energy: f64,
    pub mean_age: f64,
    pub mean_generation: f64,
    pub mean_genome_size: f64,
    /// Applied eats indexed by food type, exactly `food_type_count` long.
    pub eats_by_type: Vec<u64>,
    /// Attempts indexed like `ActionType::ALL`.
    pub actions_by_type: [u64; ACTION_TYPE_COUNT as usize],
    pub predation_kills: u64,
    pub predation_hits_taken: u64,
}

/// One living creature's contribution to its clade's profile: its lineage
/// and the lifetime counters `CreatureState` carries.
#[derive(Debug, Clone, PartialEq)]
pub struct CladeMemberReading {
    pub lineage_id: u32,
    pub energy: f32,
    pub age: u64,
    pub generation: u64,
    pub genome_size: u32,
    pub eats_by_type: Vec<u64>,
    pub actions_by_type: [u64; ACTION_TYPE_COUNT as usize],
    pub predation_kills: u64,
    pub predation_hits_taken: u64,
}

impl CladeMemberReading {
    #[must_use]
    pub fn observe(creature: &CreatureState) -> Self {
        Self {
            lineage_id: creature.body.identity.lineage_id,
            energy: creature.energy,
            age: creature.age,
            generation: creature.generation,
            genome_size: creature.genome_size(),
            eats_by_type: creature.lifetime_eats_applied_by_type.clone(),
            actions_by_type: creature.lifetime_actions_attempted_by_type,
            predation_kills: creature.lifetime_predation_kills_count,
            predation_hits_taken: creature.lifetime_predation_hits_taken_count,
        }
    }
}

#[derive(Default)]
struct CladeAccumulator {
    size: u64,
    energy_sum: f64,
    age_sum: u64,
    generation_sum: u64,
    genome_size_sum: u64,
    eats_by_type: Vec<u64>,
    actions_by_type: [u64; ACTION_TYPE_COUNT as usize],
    predation_kills: u64,
    predation_hits_taken: u64,
}

/// Bucket living creatures by `lineage_id` into profiles ascending by
/// lineage. Pure over its input: sizes sum to the member count, every
/// integer column is the sum over the profile's members, and `eats_by_type`
/// is exactly `food_type_count` long (a member's lazily grown vector may be
/// shorter, and a type index past the configured types is not a food type of
/// this run). Energy is summed in `f64` in input order, so feeding
/// `creatures.values()` keeps the order the sample `mean_energy` uses.
pub fn surviving_clade_profiles(
    food_type_count: usize,
    members: impl IntoIterator<Item = CladeMemberReading>,
) -> Vec<CladeProfile> {
    let mut clades: BTreeMap<u32, CladeAccumulator> = BTreeMap::new();
    for member in members {
        let clade = clades
            .entry(member.lineage_id)
            .or_insert_with(|| CladeAccumulator {
                eats_by_type: vec![0; food_type_count],
                ..CladeAccumulator::default()
            });
        clade.size += 1;
        clade.energy_sum += f64::from(member.energy);
        clade.age_sum += member.age;
        clade.generation_sum += member.generation;
        clade.genome_size_sum += u64::from(member.genome_size);
        for (total, eaten) in clade.eats_by_type.iter_mut().zip(&member.eats_by_type) {
            *total += eaten;
        }
        for (total, attempted) in clade.actions_by_type.iter_mut().zip(member.actions_by_type) {
            *total += attempted;
        }
        clade.predation_kills += member.predation_kills;
        clade.predation_hits_taken += member.predation_hits_taken;
    }
    clades
        .into_iter()
        .map(|(lineage_id, clade)| {
            let size = clade.size as f64;
            CladeProfile {
                lineage_id,
                size: clade.size,
                mean_energy: clade.energy_sum / size,
                mean_age: clade.age_sum as f64 / size,
                mean_generation: clade.generation_sum as f64 / size,
                mean_genome_size: clade.genome_size_sum as f64 / size,
                eats_by_type: clade.eats_by_type,
                actions_by_type: clade.actions_by_type,
                predation_kills: clade.predation_kills,
                predation_hits_taken: clade.predation_hits_taken,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn an_empty_population_has_no_clade_no_entropy_and_no_effective_lineages() {
        let census = CladeCensus::of([]);
        assert_eq!(census.surviving(), 0);
        assert_eq!(census.entropy_nats(), None);
        assert_eq!(census.effective_lineages(), None);
    }

    #[test]
    fn equally_common_clades_count_as_that_many_effective_lineages() {
        let census = CladeCensus::of([3, 3, 7, 7, 9, 9, 11, 11]);
        let effective = census.effective_lineages().expect("a living population");
        assert!((effective - 4.0).abs() < 1e-12);
    }

    proptest! {
        /// The effective lineage count is exactly `exp` of the entropy, and
        /// both are absent exactly when no creature lives.
        #[test]
        fn effective_lineages_is_the_exp_of_the_entropy(ids in prop::collection::vec(0u32..6, 0..40)) {
            let census = CladeCensus::of(ids.iter().copied());
            prop_assert_eq!(
                census.effective_lineages().map(f64::to_bits),
                census.entropy_nats().map(|entropy| entropy.exp().to_bits())
            );
            prop_assert_eq!(census.effective_lineages().is_none(), ids.is_empty());
        }

        /// Counts never depend on input order, sizes sum to the input, and
        /// profile sizes match the counts.
        #[test]
        fn clade_counts_are_order_free_and_complete(ids in prop::collection::vec(0u32..6, 0..40)) {
            let census = CladeCensus::of(ids.iter().copied());
            let reversed = CladeCensus::of(ids.iter().rev().copied());
            prop_assert_eq!(&census, &reversed);
            prop_assert_eq!(census.counts.values().sum::<u64>(), ids.len() as u64);
            let profiles = surviving_clade_profiles(1, ids.iter().map(|&lineage_id| CladeMemberReading {
                lineage_id,
                energy: 1.0,
                age: 2,
                generation: 3,
                genome_size: 4,
                eats_by_type: vec![1, 9],
                actions_by_type: [1; ACTION_TYPE_COUNT as usize],
                predation_kills: 0,
                predation_hits_taken: 1,
            }));
            prop_assert_eq!(profiles.len() as u64, census.surviving());
            for profile in &profiles {
                prop_assert_eq!(Some(&profile.size), census.counts.get(&profile.lineage_id));
                prop_assert_eq!(profile.eats_by_type.clone(), vec![profile.size]);
                prop_assert_eq!(profile.mean_generation, 3.0);
            }
        }
    }
}
