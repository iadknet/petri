//! The live sensor-use census: per catalog key, how many creatures declare,
//! wire and read it, from the birth caches the genome's sensor analysis
//! computes (`creature::genome::sensor_analysis`) and each creature's most
//! recent mesh reads; and the report label of a world input key.

use std::collections::BTreeSet;

use crate::contracts::WorldInputKey;
use crate::sensors::reads::WorldKeySet;

/// Per world input key of a world's catalog, how many creatures carry it,
/// wire it and read it (the live sensor census telemetry exports).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SensorUseRow {
    pub key: WorldInputKey,
    /// A reachable node's `input_refs` holds the key.
    pub declared: u64,
    /// The genome's [`sensor_census`] reaches the key: structurally wired,
    /// which a zero-weight or never-routed wire still is.
    ///
    /// [`sensor_census`]: crate::creature::genome::sensor_analysis::sensor_census
    pub wired: u64,
    /// The creature's most recent mesh run resolved the key.
    pub read: u64,
}

/// Count each catalog key over the creatures' declared, wired and read sets
/// (in that order per creature), every catalog key present (zeros included)
/// in `WorldInputKey` order. A key outside the catalog (a food type the world
/// does not have) is not counted anywhere.
#[must_use]
pub fn sensor_use_census<'a>(
    catalog: &BTreeSet<WorldInputKey>,
    creatures: impl IntoIterator<Item = [&'a WorldKeySet; 3]>,
) -> Vec<SensorUseRow> {
    let keys: Vec<WorldInputKey> = catalog.iter().copied().collect();
    let mut counts = vec![[0u64; 3]; keys.len()];
    for sets in creatures {
        for (key, count) in keys.iter().zip(&mut counts) {
            for (set, slot) in sets.iter().zip(count.iter_mut()) {
                *slot += u64::from(set.contains(*key));
            }
        }
    }
    keys.into_iter()
        .zip(counts)
        .map(|(key, [declared, wired, read])| SensorUseRow {
            key,
            declared,
            wired,
            read,
        })
        .collect()
}

/// Report label for a world input key: the transport key, with the food type
/// appended for the food-parameterized families so they stay distinct.
#[must_use]
pub fn world_input_key_label(key: WorldInputKey) -> String {
    match key.food_type_idx() {
        Some(type_idx) => format!("{}:{}", key.as_key(), type_idx.get()),
        None => key.as_key().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::world_key_strategy;
    use crate::contracts::OrdinaryFoodTypeId;
    use crate::sensors::reads::WorldReads;
    use proptest::prelude::*;

    const FOOD_0: OrdinaryFoodTypeId = OrdinaryFoodTypeId::new(0);
    const FOOD_1: OrdinaryFoodTypeId = OrdinaryFoodTypeId::new(1);

    #[test]
    fn the_key_universe_is_five_plus_three_per_food_type_with_labelled_families() {
        let one = WorldInputKey::universe([FOOD_0]);
        assert_eq!(one.len(), 8);

        let two = WorldInputKey::universe([FOOD_0, FOOD_1]);
        assert_eq!(two.len(), 11);
        assert!(two.is_superset(&one));

        assert_eq!(
            world_input_key_label(WorldInputKey::food_here(FOOD_1)),
            "FoodHere:1"
        );
        assert_eq!(
            world_input_key_label(WorldInputKey::AreaBarrierSummary),
            "AreaBarrierSummary"
        );
        let labels: BTreeSet<String> = two.iter().copied().map(world_input_key_label).collect();
        assert_eq!(labels.len(), two.len(), "no two keys share a label");
    }

    fn key_set(keys: &[WorldInputKey]) -> WorldKeySet {
        let reads = WorldReads::default();
        for key in keys {
            reads.mark(*key);
        }
        reads.set()
    }

    proptest! {
        /// Each row counts exactly the creatures whose declared, wired and
        /// read sets hold its key; keys outside the catalog are dropped and
        /// every catalog key has a row.
        #[test]
        fn the_sensor_use_census_counts_each_catalog_key_per_creature(
            creatures in proptest::collection::vec(
                (
                    proptest::collection::vec(world_key_strategy(), 0..5usize),
                    proptest::collection::vec(world_key_strategy(), 0..5usize),
                    proptest::collection::vec(world_key_strategy(), 0..4usize),
                ),
                0..6usize,
            ),
        ) {
            let catalog = WorldInputKey::universe([FOOD_0, FOOD_1]);
            let sets: Vec<[WorldKeySet; 3]> = creatures
                .iter()
                .map(|(declared, wired, read)| [key_set(declared), key_set(wired), key_set(read)])
                .collect();
            let rows = sensor_use_census(
                &catalog,
                sets.iter().map(|[declared, wired, read]| [declared, wired, read]),
            );

            prop_assert_eq!(
                rows.iter().map(|row| row.key).collect::<BTreeSet<_>>(),
                catalog.clone()
            );
            for row in &rows {
                let count = |column: usize| {
                    sets.iter().filter(|set| set[column].contains(row.key)).count() as u64
                };
                prop_assert_eq!(row.declared, count(0));
                prop_assert_eq!(row.wired, count(1));
                prop_assert_eq!(row.read, count(2));
            }
        }

        /// The key universe holds every key the label map distinguishes, and
        /// contains every food-parameterized family exactly once per type.
        #[test]
        fn the_key_universe_covers_every_food_type_once(count in 1usize..6) {
            let types: Vec<OrdinaryFoodTypeId> =
                (0..count).map(|i| OrdinaryFoodTypeId::new(i as u16)).collect();
            let universe = WorldInputKey::universe(types.iter().copied());

            prop_assert_eq!(universe.len(), 5 + 3 * count);
            for type_idx in types {
                prop_assert!(universe.contains(&WorldInputKey::food_here(type_idx)));
                prop_assert!(universe.contains(&WorldInputKey::neighbor_food_ring(type_idx)));
                prop_assert!(universe.contains(&WorldInputKey::area_food_summary(type_idx)));
            }
            let labels: BTreeSet<String> =
                universe.iter().copied().map(world_input_key_label).collect();
            prop_assert_eq!(labels.len(), universe.len());
        }
    }
}
