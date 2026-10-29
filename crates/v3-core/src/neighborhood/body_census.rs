//! The body census (T29.F02 invariant 12): how the living population's
//! colour and kin-tag genes have spread, read from the genome's body
//! chromosome and each creature's lineage. The kin-tag distances are mean
//! Hamming distances over unordered pairs of distinct living creatures,
//! computed exactly from per-bit counts; a pair's `KIN_AFFINITY` is
//! `1 − h/32`.

use std::collections::{BTreeMap, BTreeSet};

use crate::genome::body::Phenotype;
use crate::simulation::Simulation;

/// The living population's body census at one sampled tick.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BodyCensus {
    /// Distinct colour channel arrays.
    pub distinct_colours: u64,
    /// Share of creatures whose colour channels are the founder's.
    pub founder_colour_fraction: f64,
    /// Distinct kin tags.
    pub distinct_kin_tags: u64,
    /// Mean kin-tag Hamming distance over pairs of equal `lineage_id`,
    /// `None` without such a pair.
    pub kin_hamming_within_clade: Option<f64>,
    /// Mean kin-tag Hamming distance over pairs of different `lineage_id`,
    /// `None` without such a pair.
    pub kin_hamming_across_clades: Option<f64>,
}

/// One creature as the census reads it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BodyRecord {
    pub lineage_id: u32,
    pub channels: [u8; 6],
    pub kin_tag: u32,
}

/// Per-bit set counts of a group of kin tags, and its size.
#[derive(Debug, Clone, Copy, Default)]
struct BitCounts {
    size: u64,
    set: [u64; 32],
}

impl BitCounts {
    fn add(&mut self, kin_tag: u32) {
        self.size += 1;
        for (bit, count) in self.set.iter_mut().enumerate() {
            *count += u64::from(kin_tag >> bit & 1);
        }
    }

    /// The summed Hamming distance over the group's unordered pairs: each bit
    /// differs in `set × (size − set)` of them.
    fn pair_distance(&self) -> u64 {
        self.set.iter().map(|&set| set * (self.size - set)).sum()
    }

    fn pairs(&self) -> u64 {
        self.size * self.size.saturating_sub(1) / 2
    }
}

impl BodyCensus {
    /// The census of `sim`'s living population, `None` at extinction.
    #[must_use]
    pub fn of(sim: &Simulation) -> Option<Self> {
        Self::of_records(sim.creatures.values().map(|creature| {
            let body = creature.genome().body();
            BodyRecord {
                lineage_id: creature.lineage_id,
                channels: body.colour().channels(),
                kin_tag: body.kin_tag(),
            }
        }))
    }

    /// The census of `records`, `None` when empty.
    #[must_use]
    pub fn of_records(records: impl IntoIterator<Item = BodyRecord>) -> Option<Self> {
        let founder = Phenotype::FOUNDER.channels();
        let mut colours = BTreeSet::new();
        let mut kin_tags = BTreeSet::new();
        let mut founder_coloured = 0u64;
        let mut all = BitCounts::default();
        let mut clades: BTreeMap<u32, BitCounts> = BTreeMap::new();
        for record in records {
            colours.insert(record.channels);
            kin_tags.insert(record.kin_tag);
            founder_coloured += u64::from(record.channels == founder);
            all.add(record.kin_tag);
            clades
                .entry(record.lineage_id)
                .or_default()
                .add(record.kin_tag);
        }
        if all.size == 0 {
            return None;
        }
        let within_distance: u64 = clades.values().map(BitCounts::pair_distance).sum();
        let within_pairs: u64 = clades.values().map(BitCounts::pairs).sum();
        let across_distance = all.pair_distance() - within_distance;
        let across_pairs = all.pairs() - within_pairs;
        let mean = |distance: u64, pairs: u64| (pairs > 0).then(|| distance as f64 / pairs as f64);
        Some(Self {
            distinct_colours: colours.len() as u64,
            founder_colour_fraction: founder_coloured as f64 / all.size as f64,
            distinct_kin_tags: kin_tags.len() as u64,
            kin_hamming_within_clade: mean(within_distance, within_pairs),
            kin_hamming_across_clades: mean(across_distance, across_pairs),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn record(lineage_id: u32, channels: [u8; 6], kin_tag: u32) -> BodyRecord {
        BodyRecord {
            lineage_id,
            channels,
            kin_tag,
        }
    }

    #[test]
    fn an_empty_population_has_no_census() {
        assert_eq!(BodyCensus::of_records([]), None);
    }

    #[test]
    fn a_lone_creature_has_no_pair_and_counts_its_founder_colour() {
        let founder = Phenotype::FOUNDER.channels();
        let census = BodyCensus::of_records([record(0, founder, 7)]).unwrap();
        assert_eq!(census.distinct_colours, 1);
        assert_eq!(census.founder_colour_fraction, 1.0);
        assert_eq!(census.distinct_kin_tags, 1);
        assert_eq!(census.kin_hamming_within_clade, None);
        assert_eq!(census.kin_hamming_across_clades, None);
    }

    #[test]
    fn the_seeded_founders_are_one_colour_with_a_tag_each() {
        let mut config = crate::config::SimulationConfig::default();
        config.population.initial_creatures = 4;
        let sim = crate::simulation::seed_simulation(config, 3);
        let census = BodyCensus::of(&sim).unwrap();
        assert_eq!(census.distinct_colours, 1);
        assert_eq!(census.founder_colour_fraction, 1.0);
        assert_eq!(census.distinct_kin_tags, 4);
        assert_eq!(
            census.kin_hamming_within_clade, None,
            "one founder per clade"
        );
        assert!(census.kin_hamming_across_clades.is_some());
    }

    /// Brute force over unordered pairs of distinct records.
    fn brute_force(records: &[BodyRecord], same_clade: bool) -> Option<f64> {
        let mut distance = 0u64;
        let mut pairs = 0u64;
        for (i, a) in records.iter().enumerate() {
            for b in &records[i + 1..] {
                if (a.lineage_id == b.lineage_id) == same_clade {
                    distance += u64::from((a.kin_tag ^ b.kin_tag).count_ones());
                    pairs += 1;
                }
            }
        }
        (pairs > 0).then(|| distance as f64 / pairs as f64)
    }

    proptest! {
        /// The per-bit counts give the brute-force pair means exactly, and
        /// the distinct counts are the distinct values.
        #[test]
        fn the_census_equals_brute_force_over_pairs(
            records in proptest::collection::vec(
                (0u32..4, prop::sample::select(vec![[0u8; 6], [1u8; 6], Phenotype::FOUNDER.channels()]), any::<u32>()),
                1..40,
            ),
        ) {
            let records: Vec<BodyRecord> = records
                .into_iter()
                .map(|(lineage_id, channels, kin_tag)| record(lineage_id, channels, kin_tag))
                .collect();
            let census = BodyCensus::of_records(records.iter().copied()).unwrap();
            prop_assert_eq!(census.kin_hamming_within_clade, brute_force(&records, true));
            prop_assert_eq!(census.kin_hamming_across_clades, brute_force(&records, false));
            let colours: BTreeSet<_> = records.iter().map(|r| r.channels).collect();
            let tags: BTreeSet<_> = records.iter().map(|r| r.kin_tag).collect();
            prop_assert_eq!(census.distinct_colours, colours.len() as u64);
            prop_assert_eq!(census.distinct_kin_tags, tags.len() as u64);
            let founder = records.iter().filter(|r| r.channels == Phenotype::FOUNDER.channels()).count();
            prop_assert_eq!(census.founder_colour_fraction, founder as f64 / records.len() as f64);
        }
    }
}
