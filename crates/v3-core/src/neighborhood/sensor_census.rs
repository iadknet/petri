//! The live sensor-use census: per catalog key, how many creatures declare,
//! wire and read it, from the birth caches the genome's sensor analysis
//! computes (`genome::sensor_analysis`) and each creature's most
//! recent mesh reads.

use std::collections::{BTreeMap, BTreeSet};

use crate::contracts::WorldInputKey;
use crate::genome::sensor_analysis::{sensor_census, DecisionInputKey};
use crate::genome::CreatureGenome;
use crate::sensors::reads::WorldKeySet;
use crate::simulation::Simulation;

/// The world input keys of `sim`'s food catalog: the row set of every
/// population sensor census.
#[must_use]
pub fn world_input_catalog(sim: &Simulation) -> BTreeSet<WorldInputKey> {
    WorldInputKey::universe(
        sim.world
            .food()
            .food_types()
            .iter()
            .map(|food_type| food_type.id),
    )
}

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
    /// [`sensor_census`]: crate::genome::sensor_analysis::sensor_census
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

/// The structural sensor census of one population (T14.F08, T19.F05): per
/// world input key of the run's universe and per decision-state input, the
/// creatures holding a live reference to it, and the three stateful-reach
/// counts, from each creature's `genome::sensor_analysis::sensor_census`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceCensus {
    /// Every universe key, in key order, with its creature count.
    pub world_inputs: BTreeMap<WorldInputKey, u64>,
    /// Indexed like `DecisionInputKey::ALL`.
    pub decision_inputs: [u64; DecisionInputKey::ALL.len()],
    pub reading_shared_memory: u64,
    pub with_stateful_node: u64,
    pub with_any_stateful_read: u64,
}

impl ReferenceCensus {
    /// Census `creatures`, each a genome and its reachable node indices,
    /// over the keys of `universe`. Pure structure: no brain executes.
    pub fn observe<'a>(
        universe: impl IntoIterator<Item = WorldInputKey>,
        creatures: impl IntoIterator<Item = (&'a CreatureGenome, &'a [usize])>,
    ) -> Self {
        let mut census = Self {
            world_inputs: universe.into_iter().map(|key| (key, 0)).collect(),
            decision_inputs: [0; DecisionInputKey::ALL.len()],
            reading_shared_memory: 0,
            with_stateful_node: 0,
            with_any_stateful_read: 0,
        };
        for (genome, reachable) in creatures {
            let reading = sensor_census(genome, reachable);
            census.reading_shared_memory += u64::from(reading.reads_shared_memory);
            census.with_stateful_node += u64::from(reading.holds_stateful_node);
            census.with_any_stateful_read += u64::from(reading.reads_any_stateful());
            for key in reading.decision_inputs {
                census.decision_inputs[key as usize] += 1;
            }
            for key in reading.world_inputs {
                // The row set is the same at every checkpoint: mutation draws
                // food type ids from `config.world.food.types.len()`, clamped
                // to at least one (`mutation::sampling::sample_food_type_id`),
                // and the universe is the runtime catalog — that same list, or
                // the one synthesized default when the config carries none. So
                // no key can land outside the declared universe.
                *census.world_inputs.entry(key).or_insert(0) += 1;
            }
        }
        census
    }

    /// The census of `sim`'s living population over its world's catalog.
    #[must_use]
    pub fn of(sim: &Simulation) -> Self {
        Self::observe(
            world_input_catalog(sim),
            sim.creatures
                .values()
                .map(|creature| (creature.genome(), creature.reachable_nodes())),
        )
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
    }
}
