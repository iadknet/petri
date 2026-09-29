//! Arena `sparse-food-v1` configs, arm overlays and the lab invariants.
//!
//! Resolution order: production defaults → arena → overlay (an RFC 7396
//! merge patch over the arena config's JSON) → deserialize (unknown keys are
//! errors) → normalize → startup overrides → lab invariants.

use serde_json::Value;
use v3_core::config::{resolve_config, SimulationConfig, WorldEdgeMode};

use crate::LabError;

/// Overlay key paths the lab owns; an overlay naming one is refused.
const REFUSED_PATHS: &[&[&str]] = &[
    &["world", "width"],
    &["world", "height"],
    &["world", "terrain"],
    &["world", "food", "shared", "growth_rate"],
    &["world", "food", "shared", "recovery_spawn_rate"],
    &["population", "initial_creatures"],
    &["energy", "lifecycle", "min_reproduce_energy"],
];
/// Per-food-type overrides the lab owns.
const REFUSED_TYPE_KEYS: &[&str] = &["growth_rate", "recovery_spawn_rate"];

/// Arm label: whether the resolved `mutation` block is the reference's.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Policy {
    Native,
    PolicyDeviation,
}

/// Arm label: what the arm is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Role {
    Reference,
    Control,
    Instrument,
    User,
}

/// Production defaults as `v3-cli` resolves them: normalized, with startup
/// overrides applied.
///
/// # Panics
///
/// If the default config does not resolve (a `v3-core` bug).
#[must_use]
pub fn production_defaults() -> SimulationConfig {
    resolve_config(&SimulationConfig::default(), serde_json::json!({}))
        .expect("production defaults resolve")
}

/// The arena config at `size`: production defaults plus the lab invariants.
#[must_use]
pub fn arena_config(size: u16) -> SimulationConfig {
    let mut config = production_defaults();
    apply_lab_invariants(&mut config, size);
    config
}

/// Apply the arena overrides the lab owns. Idempotent.
pub fn apply_lab_invariants(config: &mut SimulationConfig, size: u16) {
    config.world.width = size;
    config.world.height = size;
    config.world.terrain.clear();
    config.world.food.shared.growth_rate = 0.0;
    config.world.food.shared.recovery_spawn_rate = 0.0;
    for food_type in &mut config.world.food.types {
        food_type.growth_rate = None;
        food_type.recovery_spawn_rate = None;
    }
    config.population.initial_creatures = 1;
    config.energy.lifecycle.min_reproduce_energy = config.energy.lifecycle.max_energy + 1.0;
}

/// Check a resolved arm config: reproduction suppressed in `f32`, start
/// energy within capacity, toroidal geometry.
///
/// # Errors
///
/// [`LabError::Config`] naming the violated invariant.
pub fn validate(config: &SimulationConfig, start_energy: f32) -> Result<(), LabError> {
    let lifecycle = &config.energy.lifecycle;
    if !(lifecycle.min_reproduce_energy.is_finite()
        && lifecycle.min_reproduce_energy > lifecycle.max_energy)
    {
        return Err(LabError::Config(format!(
            "reproduction threshold {} is not strictly above max_energy {} in f32",
            lifecycle.min_reproduce_energy, lifecycle.max_energy
        )));
    }
    if !(start_energy.is_finite() && start_energy > 0.0 && start_energy <= lifecycle.max_energy) {
        return Err(LabError::Config(format!(
            "start energy {start_energy} must be in (0, max_energy {}]",
            lifecycle.max_energy
        )));
    }
    if config.world.edge_mode != WorldEdgeMode::Wrap {
        return Err(LabError::Config(
            "arena geometry is toroidal: world.edge_mode must be Wrap".into(),
        ));
    }
    Ok(())
}

/// RFC 7396 JSON merge patch: `null` deletes, objects merge, anything else
/// replaces.
pub fn merge_patch(target: &mut Value, patch: &Value) {
    let Value::Object(patch_map) = patch else {
        *target = patch.clone();
        return;
    };
    if !target.is_object() {
        *target = Value::Object(serde_json::Map::new());
    }
    let Value::Object(target_map) = target else {
        unreachable!("target was just made an object");
    };
    for (key, value) in patch_map {
        if value.is_null() {
            target_map.remove(key);
        } else {
            merge_patch(target_map.entry(key.clone()).or_insert(Value::Null), value);
        }
    }
}

fn names_path(patch: &Value, path: &[&str]) -> bool {
    let mut node = patch;
    for key in path {
        match node.get(key) {
            Some(next) => node = next,
            None => return false,
        }
    }
    true
}

/// The first lab-owned key an overlay names, if any.
#[must_use]
pub fn refused_key(overlay: &Value) -> Option<String> {
    if let Some(path) = REFUSED_PATHS.iter().find(|path| names_path(overlay, path)) {
        return Some(path.join("."));
    }
    let types = overlay
        .get("world")
        .and_then(|world| world.get("food"))
        .and_then(|food| food.get("types"))
        .and_then(Value::as_array)?;
    types.iter().enumerate().find_map(|(index, entry)| {
        REFUSED_TYPE_KEYS
            .iter()
            .find(|key| entry.get(**key).is_some())
            .map(|key| format!("world.food.types[{index}].{key}"))
    })
}

/// Resolve an arm config: `overlay` merged over the arena config at `size`,
/// deserialized, normalized, startup overrides applied, then the lab
/// invariants re-applied and validated.
///
/// # Errors
///
/// [`LabError::Config`] for a refused key, an unknown key, a type error or a
/// violated invariant.
pub fn resolve_arm(
    overlay: Option<&Value>,
    size: u16,
    start_energy: f32,
) -> Result<SimulationConfig, LabError> {
    let arena = arena_config(size);
    let mut config = match overlay {
        None => arena,
        Some(overlay) => {
            if !overlay.is_object() {
                return Err(LabError::Config("an overlay must be a JSON object".into()));
            }
            if let Some(key) = refused_key(overlay) {
                return Err(LabError::Config(format!(
                    "overlay names lab-owned key `{key}`"
                )));
            }
            let mut value = serde_json::to_value(&arena).expect("config serializes");
            merge_patch(&mut value, overlay);
            let mut config: SimulationConfig = serde_json::from_value(value)
                .map_err(|error| LabError::Config(format!("overlay: {error}")))?;
            config.normalize();
            config.apply_startup_overrides();
            config
        }
    };
    apply_lab_invariants(&mut config, size);
    validate(&config, start_energy)?;
    Ok(config)
}

/// `policy-deviation` whenever the resolved `mutation` block differs from
/// the reference's.
#[must_use]
pub fn classify(config: &SimulationConfig, reference: &SimulationConfig) -> Policy {
    let block = |c: &SimulationConfig| serde_json::to_value(&c.mutation).expect("serializes");
    if block(config) == block(reference) {
        Policy::Native
    } else {
        Policy::PolicyDeviation
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use serde_json::json;

    #[test]
    fn arena_suppresses_reproduction_and_regrowth() {
        let config = arena_config(64);
        assert_eq!(config.world.width, 64);
        assert!(config.world.terrain.is_empty());
        assert_eq!(config.world.food.shared.growth_rate, 0.0);
        assert_eq!(config.population.initial_creatures, 1);
        assert!(config.energy.lifecycle.min_reproduce_energy > config.energy.lifecycle.max_energy);
        assert!(validate(&config, 100.0).is_ok());
    }

    #[test]
    fn a_reproduction_threshold_equal_to_max_energy_is_refused() {
        let mut config = arena_config(48);
        config.energy.lifecycle.min_reproduce_energy = config.energy.lifecycle.max_energy;
        let error = validate(&config, 100.0).unwrap_err();
        assert!(error.to_string().contains("strictly above"), "{error}");
    }

    #[test]
    fn start_energy_must_be_positive() {
        let config = arena_config(48);
        assert!(validate(&config, f32::MIN_POSITIVE).is_ok());
        assert!(validate(&config, 0.0).is_err());
    }

    #[test]
    fn overlay_naming_a_lab_key_is_refused() {
        for overlay in [
            json!({"population": {"initial_creatures": 4}}),
            json!({"energy": {"lifecycle": {"min_reproduce_energy": 1.0}}}),
            json!({"world": {"width": 32}}),
            json!({"world": {"food": {"types": [{"growth_rate": 0.1}]}}}),
        ] {
            let error = resolve_arm(Some(&overlay), 48, 100.0).unwrap_err();
            assert!(error.to_string().contains("lab-owned"), "{error}");
        }
    }

    #[test]
    fn unknown_keys_and_unsuppressible_energies_are_errors() {
        let unknown = json!({"mutation": {"no_such_knob": 1}});
        assert!(resolve_arm(Some(&unknown), 48, 100.0).is_err());
        // At 2^24 the +1.0 threshold rounds back to max_energy in f32.
        let huge = json!({"energy": {"lifecycle": {"max_energy": 16_777_216.0}}});
        let error = resolve_arm(Some(&huge), 48, 100.0).unwrap_err();
        assert!(error.to_string().contains("strictly above"), "{error}");
        assert!(resolve_arm(None, 48, 1_000.0).is_err());
    }

    #[test]
    fn overlay_raising_max_energy_keeps_births_suppressed() {
        let overlay = json!({"energy": {"lifecycle": {"max_energy": 500.0}}});
        let config = resolve_arm(Some(&overlay), 48, 100.0).unwrap();
        assert_eq!(config.energy.lifecycle.min_reproduce_energy, 501.0);
    }

    #[test]
    fn mutation_overlays_are_policy_deviations_and_others_native() {
        let reference = resolve_arm(None, 48, 100.0).unwrap();
        let off = resolve_arm(
            Some(&json!({"mutation": {"per_unit_rate": 0.0}})),
            48,
            100.0,
        )
        .unwrap();
        assert_eq!(off.mutation.per_unit_rate, 0.0, "normalize keeps rate 0");
        assert_eq!(classify(&off, &reference), Policy::PolicyDeviation);
        let costs = resolve_arm(
            Some(&json!({"energy": {"costs": {"move_cost": 0.3}}})),
            48,
            100.0,
        )
        .unwrap();
        assert_eq!(classify(&costs, &reference), Policy::Native);
    }

    #[test]
    fn merge_patch_follows_rfc_7396_examples() {
        let mut target = json!({"a": "b", "c": {"d": "e", "f": "g"}});
        merge_patch(&mut target, &json!({"a": "z", "c": {"f": null}}));
        assert_eq!(target, json!({"a": "z", "c": {"d": "e"}}));
        let mut target = json!({"a": [1]});
        merge_patch(&mut target, &json!({"a": {"b": "c"}}));
        assert_eq!(target, json!({"a": {"b": "c"}}));
    }

    fn small_json() -> impl Strategy<Value = Value> {
        let leaf = prop_oneof![
            Just(Value::Null),
            any::<bool>().prop_map(Value::Bool),
            (0i64..5).prop_map(|n| json!(n)),
        ];
        leaf.prop_recursive(3, 16, 3, |inner| {
            prop::collection::btree_map("[a-c]", inner, 0..3)
                .prop_map(|map| Value::Object(map.into_iter().collect()))
        })
    }

    proptest! {
        /// Applying the same patch twice is the same as applying it once.
        #[test]
        fn merge_patch_is_idempotent(target in small_json(), patch in small_json()) {
            let mut once = target.clone();
            merge_patch(&mut once, &patch);
            let mut twice = once.clone();
            merge_patch(&mut twice, &patch);
            prop_assert_eq!(once, twice);
        }
    }
}
