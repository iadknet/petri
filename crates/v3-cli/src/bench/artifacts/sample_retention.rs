//! The summary keep-lists derived from the persistence sample's retention
//! declarations (T30.F13, `bench::retention`): the keys the v1 stage copies,
//! the keys the v2 summary keeps on every checkpoint, and each projected
//! block's projection.

use crate::bench::retention::{CheckpointRetention, Retention};
use crate::bench::tracking::{PersistenceSample, SampleBlocks, WorldTracking};

/// Every declared sample key that is not a flattened record, with its
/// retention: the sample's own keys, then its blocks'.
fn declared() -> impl Iterator<Item = (&'static str, Retention)> {
    PersistenceSample::RETENTION
        .iter()
        .chain(SampleBlocks::RETENTION)
        .copied()
        .filter(|(_, retention)| *retention != Retention::Flattened)
}

/// The keys the v1 stage copies from a full report's sample beside its
/// `WorldTracking` aggregates: every declared key.
pub(super) fn sample_fields() -> Vec<&'static str> {
    declared().map(|(key, _)| key).collect()
}

/// The keys the v2 summary keeps on every checkpoint: the kept and the
/// projected keys, whose projection then narrows them, and the
/// `WorldTracking` fields a sample summary keeps.
pub(super) fn kept_sample_fields() -> Vec<&'static str> {
    declared()
        .filter(|(_, retention)| matches!(retention, Retention::Kept | Retention::Projected(_)))
        .map(|(key, _)| key)
        .chain(WorldTracking::SAMPLE_SUMMARY_FIELDS)
        .collect()
}

/// A projected key, its fields kept on every checkpoint, and its fields
/// kept on each world's last checkpoint only.
pub(super) type Projection = (&'static str, Vec<&'static str>, Vec<&'static str>);

/// Every projected key's projection.
pub(super) fn projected_sample_fields() -> Vec<Projection> {
    declared()
        .filter_map(|(key, retention)| match retention {
            Retention::Projected(fields) => {
                let at = |when: CheckpointRetention| {
                    fields
                        .iter()
                        .filter(|(_, kept)| *kept == when)
                        .map(|(field, _)| *field)
                        .collect()
                };
                Some((
                    key,
                    at(CheckpointRetention::Every),
                    at(CheckpointRetention::Last),
                ))
            }
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn keys(value: &serde_json::Value) -> BTreeSet<String> {
        value
            .as_object()
            .expect("an object")
            .keys()
            .cloned()
            .collect()
    }

    /// A living sample with every optional block present, and an extinct
    /// one, from real post-tick state.
    fn samples() -> [serde_json::Value; 2] {
        let mut config = v3_core::config::SimulationConfig::default();
        config.world.width = 24;
        config.world.height = 24;
        config.population.initial_creatures = 6;
        let living = v3_core::simulation::seed_simulation(config.clone(), 3);
        config.population.initial_creatures = 0;
        let extinct = v3_core::simulation::seed_simulation(config, 3);
        [living, extinct].map(|sim| {
            serde_json::to_value(PersistenceSample {
                tick: 1,
                population: sim.creatures.len() as u64,
                births_total: 0,
                blocks: SampleBlocks::observe(&sim),
                tracking: WorldTracking::observe(&sim),
            })
            .expect("a sample serializes")
        })
    }

    /// Every serialized sample key is declared, by the sample or its blocks
    /// or as a `WorldTracking` field; and every declared key is a serialized
    /// one, so no declaration names a field serde renamed.
    #[test]
    fn every_serialized_sample_key_has_one_declaration() {
        let tracking = keys(&serde_json::to_value(WorldTracking::default()).unwrap());
        let declared: BTreeSet<String> = sample_fields().into_iter().map(str::to_owned).collect();
        assert!(declared.is_disjoint(&tracking));
        let [living, extinct] = samples();
        for sample in [&living, &extinct] {
            let undeclared: Vec<_> = keys(sample)
                .into_iter()
                .filter(|key| !declared.contains(key) && !tracking.contains(key))
                .collect();
            assert!(
                undeclared.is_empty(),
                "undeclared sample keys: {undeclared:?}"
            );
        }
        assert!(declared.is_subset(&keys(&living)));
        for field in WorldTracking::SAMPLE_SUMMARY_FIELDS {
            assert!(tracking.contains(field), "{field} is a WorldTracking field");
        }
        let bundle_fields: BTreeSet<String> = crate::bench::bundles::Bundles::RETENTION
            .iter()
            .map(|(field, _)| (*field).to_owned())
            .collect();
        assert_eq!(keys(&living["bundles"]), bundle_fields);
    }

    /// The derived keep-lists are the readings' retention table (T30.F13);
    /// the minimum summary changes only by an edit to this pin.
    #[test]
    fn the_derived_keep_lists_are_the_pinned_retention_table() {
        let set = |keys: Vec<&str>| keys.into_iter().map(str::to_owned).collect::<BTreeSet<_>>();
        assert_eq!(
            set(sample_fields()),
            set(vec![
                "tick",
                "population",
                "mean_energy",
                "births_total",
                "mean_genome_size",
                "mean_mesh_nodes",
                "mean_generation",
                "mean_wired_sinks",
                "surviving_founder_clade_count",
                "shannon_entropy_nats",
                "sensor_census",
                "occupancy_grid",
                "bundles",
                "plasticity_census",
                "capacity",
                "shares",
            ])
        );
        assert_eq!(
            set(kept_sample_fields()),
            set(vec![
                "tick",
                "population",
                "mean_energy",
                "mean_genome_size",
                "mean_mesh_nodes",
                "mean_wired_sinks",
                "shannon_entropy_nats",
                "surviving_founder_clade_count",
                "food_density_total",
                "grazed_cell_share",
                "grazing_modifier_mean",
                "occupancy_grid",
                "sensor_census",
                "bundles",
                "plasticity_census",
                "capacity",
                "shares",
            ])
        );
        assert_eq!(
            projected_sample_fields(),
            vec![(
                "bundles",
                vec!["version", "creatures", "carriers", "wirings", "created"],
                vec!["rows", "learning"],
            )]
        );
    }
}
