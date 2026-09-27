mod recipe;
pub use recipe::{
    config_digest, deep_merge, resolve_config, sort_json_keys_recursive, ConfigError,
};
mod simulation;

pub use crate::contracts::OrdinaryFoodTypeId;
#[cfg(test)]
pub(crate) use simulation::MAX_GRAZING_RECOVERY_TICKS;
pub use simulation::{
    AnnealingConfig, EnergyConfig, EnergyCostsConfig, EnergyLifecycleConfig, FertilityAlgorithm,
    FertilityConfig, FertilityLayer, FertilityLayerTarget, FoodConfig, FoodResourceConfig,
    FoodTypeConfig, FounderProfile, GrazingConfig, MutationConfig, OccupancyDepletionConfig,
    PhenotypeConfig, PopulationConfig, PredationConfig, ReachableBiasConfig, RuntimeConfig,
    SimulationConfig, TerrainLayer, VmRuntimeConfig, WorldConfig, WorldEdgeMode,
};

#[cfg(test)]
mod recipe_tests {
    use super::*;
    use proptest::prelude::*;
    use serde_json::{json, Value};

    #[test]
    fn recipe_rejects_shapes_unknown_fields_and_invalid_ramps() {
        for patch in [
            json!(null),
            json!([]),
            json!(1),
            json!({"seed": 1}),
            json!({"world":{"stale":1}}),
            json!({"world":null}),
            json!({"startup":{"ramps":{"failed_action_penalty":{"target_tick":0}}}}),
            json!({"startup":{"ramps":{"failed_action_penalty":{"start":-1}}}}),
            json!({"startup":{"ramps":{"failed_action_penalty":{"end":-1}}}}),
        ] {
            assert!(
                resolve_config(&SimulationConfig::default(), patch.clone()).is_err(),
                "accepted {patch}"
            );
        }
    }

    /// T11.F20: a recipe carrying any retired per-birth supply key fails to
    /// resolve, with no alias, and names the key.
    #[test]
    fn recipe_rejects_the_retired_per_birth_supply_keys() {
        for (key, value) in [
            ("mutation_probability", json!(0.44)),
            ("per_birth_mutation_events_min", json!(1)),
            ("per_birth_mutation_events_max", json!(10)),
            (
                "per_birth_mutation_event_continuation_probability",
                json!(0.2),
            ),
            ("per_unit_supply_enabled", json!(true)),
        ] {
            let error = resolve_config(
                &SimulationConfig::default(),
                json!({"mutation": {key: value}}),
            )
            .unwrap_err();
            assert!(error.to_string().contains(key), "{key}: {error}");
        }
    }

    /// The seven tracked root recipes resolve once the retired keys are gone.
    #[test]
    fn root_world_recipes_resolve() {
        for (name, source) in [
            (
                "extended-age-drain-age10-baseline",
                include_str!("../../../../world-recipe-extended-age-drain-age10-baseline.json"),
            ),
            (
                "extended-age-drain-age10-mutation-half",
                include_str!(
                    "../../../../world-recipe-extended-age-drain-age10-mutation-half.json"
                ),
            ),
            (
                "extended-age-drain-age10-mutation-quarter",
                include_str!(
                    "../../../../world-recipe-extended-age-drain-age10-mutation-quarter.json"
                ),
            ),
            (
                "extended-age-drain-reduced-copy-growth",
                include_str!(
                    "../../../../world-recipe-extended-age-drain-reduced-copy-growth.json"
                ),
            ),
            (
                "extended-age-drain-relaxed-genome-costs",
                include_str!(
                    "../../../../world-recipe-extended-age-drain-relaxed-genome-costs.json"
                ),
            ),
            (
                "extended-age-drain",
                include_str!("../../../../world-recipe-extended-age-drain.json"),
            ),
            (
                "fertility-zones",
                include_str!("../../../../world-recipe-fertility-zones.json"),
            ),
        ] {
            let recipe: Value = serde_json::from_str(source).unwrap();
            if let Err(error) = resolve_config(&SimulationConfig::default(), recipe) {
                panic!("{name}: {error}");
            }
        }
    }

    #[test]
    fn recipe_validation_reports_fields_and_accepts_first_ramp_tick() {
        let error = resolve_config(&SimulationConfig::default(), json!(null)).unwrap_err();
        assert_eq!(error.to_string(), "config: recipe must be a JSON object");
        let config = resolve_config(&SimulationConfig::default(), json!({"startup":{"ramps":{"failed_action_penalty":{"target_tick":1,"start":0,"end":0}}}})).unwrap();
        assert_eq!(config.startup.ramps.failed_action_penalty.target_tick, 1);
    }
    proptest! {
        #[test]
        fn recipe_merge_preserves_objects_replaces_arrays_and_null(seed in any::<u64>(), values in prop::collection::vec(any::<u16>(), 0..8)) {
            let mut base = json!({"nested":{"keep":true,"replace":0},"array":[1,2],"nullable":1});
            deep_merge(&mut base, json!({"nested":{"replace":seed},"array":values,"nullable":null}));
            prop_assert_eq!(base, json!({"nested":{"keep":true,"replace":seed},"array":values,"nullable":null}));
        }
        #[test]
        fn recipe_effective_config_roundtrips_and_hashes(seed in any::<u64>(), width in 1u16..100, penalty in 0u16..100) {
            let patch = json!({"world":{"width":width,"world_seed":seed},"runtime":{"max_actions_per_turn":0},"startup":{"ramps":{"failed_action_penalty":{"enabled":true,"end":penalty}}}});
            let config = resolve_config(&SimulationConfig::default(), patch.clone()).unwrap();
            prop_assert_eq!(config.world.world_seed, Some(seed));
            prop_assert_eq!(config.energy.costs.failed_action_penalty, f32::from(penalty));
            let full = serde_json::to_value(&config).unwrap();
            let reloaded = resolve_config(&SimulationConfig::default(), serde_json::from_str(&serde_json::to_string_pretty(&full).unwrap()).unwrap()).unwrap();
            prop_assert_eq!(serde_json::to_value(&reloaded).unwrap(), full);
            prop_assert_eq!(config_digest(&config), config_digest(&reloaded));
            let mut manual: Value = serde_json::to_value(SimulationConfig::default()).unwrap();
            deep_merge(&mut manual, patch);
            let mut manual: SimulationConfig = serde_json::from_value(manual).unwrap();
            manual.normalize();
            manual.apply_startup_overrides();
            prop_assert_eq!(config_digest(&config), config_digest(&manual));
            let digest = config_digest(&config);
            prop_assert_eq!(digest.len(), 71);
            prop_assert!(digest.starts_with("sha256:"));
            let mut changed = config.clone();
            changed.world.world_seed = Some(seed.wrapping_add(1));
            prop_assert_ne!(config_digest(&config), config_digest(&changed));
            let layer = |layer_seed| json!({"params":{"pattern_type":"Noise","density":0.2,"cluster_size":1},"seed":layer_seed});
            let ordered = resolve_config(&config, json!({"world":{"terrain":[layer(seed),layer(seed.wrapping_add(1))]}})).unwrap();
            let reversed = resolve_config(&config, json!({"world":{"terrain":[layer(seed.wrapping_add(1)),layer(seed)]}})).unwrap();
            prop_assert_ne!(config_digest(&ordered), config_digest(&reversed));

        }
    }
}
