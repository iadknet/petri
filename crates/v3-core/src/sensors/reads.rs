//! Which world input keys one creature's mesh run actually resolved.
//!
//! Observation only: [`WorldReads`] rides on the frozen [`SensorSnapshot`]
//! and is marked by `resolve_input` on every world read (a VM `ReadInput`
//! that executes, a graph leaf that is evaluated, a group member included).
//! The tick copies it onto the creature as a [`WorldKeySet`]; no
//! simulation value depends on it.
//!
//! [`SensorSnapshot`]: crate::sensors::perception::SensorSnapshot

use std::collections::BTreeSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use crate::contracts::WorldInputKey;

/// Keys without a food type, one bit each.
const UNTYPED_BITS: u32 = 7;
/// Bits per food type: `FoodHere`, `NeighborFoodRing`, `AreaFoodSummary`.
const BITS_PER_FOOD_TYPE: u32 = 3;
/// Inline words: the untyped keys and the first 40 food types. Higher food
/// types spill into a sorted set, so every valid key has a bit.
const WORDS: usize = 2;
const INLINE_BITS: u32 = WORDS as u32 * 64;

/// The bit a key occupies: untyped keys first, then three per food type.
fn bit(key: WorldInputKey) -> u32 {
    let typed = |type_idx: u16, family: u32| {
        UNTYPED_BITS + u32::from(type_idx) * BITS_PER_FOOD_TYPE + family
    };
    match key {
        WorldInputKey::NeighborBarrierRing => 0,
        WorldInputKey::NeighborOccupiedRing => 1,
        WorldInputKey::AreaBarrierSummary => 2,
        WorldInputKey::AreaOccupancySummary => 3,
        WorldInputKey::NearbyCreatureCore => 4,
        WorldInputKey::NearbyCreatureVitals => 5,
        WorldInputKey::NearbyCreatureIdentity => 6,
        WorldInputKey::FoodHere { type_idx } => typed(type_idx.get(), 0),
        WorldInputKey::NeighborFoodRing { type_idx } => typed(type_idx.get(), 1),
        WorldInputKey::AreaFoodSummary { type_idx } => typed(type_idx.get(), 2),
    }
}

/// The keys one mesh run has resolved so far. Marked through a shared
/// reference during execution; each snapshot belongs to one creature's run,
/// so relaxed load-then-store is enough for the inline words (`Sync` only so
/// snapshots can be shared across the parallel cognition phase). Spilled
/// food types go through a mutex.
#[derive(Debug, Default)]
pub struct WorldReads {
    words: [AtomicU64; WORDS],
    spill: Mutex<BTreeSet<u32>>,
}

impl WorldReads {
    /// Record a read of `key`.
    #[inline]
    pub fn mark(&self, key: WorldInputKey) {
        let bit = bit(key);
        if bit >= INLINE_BITS {
            self.spill
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .insert(bit);
            return;
        }
        let word = &self.words[(bit / 64) as usize];
        let mask = 1u64 << (bit % 64);
        let current = word.load(Ordering::Relaxed);
        if current & mask == 0 {
            word.store(current | mask, Ordering::Relaxed);
        }
    }

    /// The keys read so far.
    #[must_use]
    pub fn set(&self) -> WorldKeySet {
        let spill = self
            .spill
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        WorldKeySet {
            words: self
                .words
                .each_ref()
                .map(|word| word.load(Ordering::Relaxed)),
            spill: spill.clone(),
        }
    }
}

impl Clone for WorldReads {
    fn clone(&self) -> Self {
        let set = self.set();
        Self {
            words: set.words.map(AtomicU64::new),
            spill: Mutex::new(set.spill),
        }
    }
}

impl PartialEq for WorldReads {
    fn eq(&self, other: &Self) -> bool {
        self.set() == other.set()
    }
}

/// A compact set of world input keys: the keys a mesh run resolved, or the
/// keys a genome declares or wires (cached at birth). Inline for the first
/// 40 food types; higher ones spill to the heap.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorldKeySet {
    words: [u64; WORDS],
    spill: BTreeSet<u32>,
}

impl WorldKeySet {
    fn insert_bit(&mut self, bit: u32) {
        if bit < INLINE_BITS {
            self.words[(bit / 64) as usize] |= 1u64 << (bit % 64);
        } else {
            self.spill.insert(bit);
        }
    }

    /// Whether the set holds `key`.
    #[must_use]
    pub fn contains(&self, key: WorldInputKey) -> bool {
        let bit = bit(key);
        if bit < INLINE_BITS {
            self.words[(bit / 64) as usize] & (1u64 << (bit % 64)) != 0
        } else {
            self.spill.contains(&bit)
        }
    }

    /// The set is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.words.iter().all(|&word| word == 0) && self.spill.is_empty()
    }

    /// Number of keys in the set.
    #[must_use]
    pub fn len(&self) -> usize {
        let inline: u32 = self.words.iter().map(|word| word.count_ones()).sum();
        inline as usize + self.spill.len()
    }
}

impl FromIterator<WorldInputKey> for WorldKeySet {
    fn from_iter<I: IntoIterator<Item = WorldInputKey>>(keys: I) -> Self {
        let mut set = Self::default();
        for key in keys {
            set.insert_bit(bit(key));
        }
        set
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::OrdinaryFoodTypeId;
    use crate::creature::sensor_census::world_input_key_universe;
    use proptest::prelude::*;

    #[test]
    fn a_marked_key_is_contained_and_no_other() {
        let reads = WorldReads::default();
        assert!(reads.set().is_empty());

        reads.mark(WorldInputKey::NearbyCreatureCore);
        reads.mark(WorldInputKey::NearbyCreatureCore);

        let set = reads.set();
        assert!(set.contains(WorldInputKey::NearbyCreatureCore));
        assert!(!set.contains(WorldInputKey::NearbyCreatureVitals));
        assert!(!set.is_empty());
    }

    #[test]
    fn food_types_past_the_inline_words_spill_and_are_still_recorded() {
        for type_idx in [39u16, 40, 41, 1_000, u16::MAX] {
            let key = WorldInputKey::area_food_summary(OrdinaryFoodTypeId::new(type_idx));
            let other = WorldInputKey::food_here(OrdinaryFoodTypeId::new(type_idx));
            let reads = WorldReads::default();
            reads.mark(key);

            let set = reads.set();
            assert!(set.contains(key), "type {type_idx}");
            assert!(!set.contains(other), "type {type_idx}");
            assert!(!set.is_empty());
            assert_eq!(set, [key].into_iter().collect::<WorldKeySet>());
            assert_eq!(reads.clone().set(), set);
        }
    }

    proptest! {
        /// Every key of a world within the recorded range has its own bit:
        /// marking any subset of the universe reads back exactly that subset.
        #[test]
        fn marks_read_back_exactly_over_the_key_universe(
            food_types in 1u16..=60,
            picks in proptest::collection::vec(any::<bool>(), 7 + 3 * 60),
        ) {
            let universe: Vec<_> = world_input_key_universe(
                (0..food_types).map(OrdinaryFoodTypeId::new),
            )
            .into_iter()
            .collect();
            let reads = WorldReads::default();
            for (key, &pick) in universe.iter().zip(&picks) {
                if pick {
                    reads.mark(*key);
                }
            }
            let set = reads.clone().set();
            for (key, &pick) in universe.iter().zip(&picks) {
                prop_assert_eq!(set.contains(*key), pick, "{:?}", key);
            }
            let collected: WorldKeySet = universe
                .iter()
                .zip(&picks)
                .filter(|(_, &pick)| pick)
                .map(|(key, _)| *key)
                .collect();
            prop_assert_eq!(&collected, &set);
            prop_assert_eq!(set.len(), picks.iter().take(universe.len()).filter(|&&p| p).count());
        }
    }
}
