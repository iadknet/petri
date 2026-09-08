use super::SimulationConfig;
use sha2::{Digest, Sha256};

/// Deep-merge `patch` into `base` (recursive object merge; non-object values replace).
pub fn deep_merge(base: &mut serde_json::Value, patch: serde_json::Value) {
    match (base, patch) {
        (serde_json::Value::Object(base_map), serde_json::Value::Object(patch_map)) => {
            for (key, patch_val) in patch_map {
                let base_val = base_map.entry(key).or_insert(serde_json::Value::Null);
                deep_merge(base_val, patch_val);
            }
        }
        (base, patch) => {
            *base = patch;
        }
    }
}

/// Sort all JSON object keys recursively (alphabetical order).
pub fn sort_json_keys_recursive(val: serde_json::Value) -> serde_json::Value {
    match val {
        serde_json::Value::Object(map) => {
            let mut sorted: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
            let mut keys: Vec<String> = map.keys().cloned().collect();
            keys.sort();
            for key in keys {
                let v = map[&key].clone();
                sorted.insert(key, sort_json_keys_recursive(v));
            }
            serde_json::Value::Object(sorted)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.into_iter().map(sort_json_keys_recursive).collect())
        }
        other => other,
    }
}

/// Compute a deterministic digest of the config as `"sha256:<lowercase_hex>"`.
pub fn config_digest(config: &SimulationConfig) -> String {
    let val = serde_json::to_value(config).expect("config must be serializable");
    let sorted = sort_json_keys_recursive(val);
    let canonical = serde_json::to_string(&sorted).expect("sorted value must be serializable");
    let hash = Sha256::digest(canonical.as_bytes());
    format!("sha256:{}", hex::encode(hash))
}

/// A recipe field rejected before normalization or simulation replacement.
#[derive(Debug)]
pub struct ConfigError {
    pub field: &'static str,
    pub reason: String,
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.reason)
    }
}
impl std::error::Error for ConfigError {}

/// Resolve a partial recipe against a startup baseline into its applied config.
pub fn resolve_config(
    baseline: &SimulationConfig,
    patch: serde_json::Value,
) -> Result<SimulationConfig, ConfigError> {
    if !patch.is_object() {
        return Err(ConfigError {
            field: "config",
            reason: "recipe must be a JSON object".into(),
        });
    }
    let mut value = serde_json::to_value(baseline).expect("config must be serializable");
    deep_merge(&mut value, patch);
    let mut config: SimulationConfig =
        serde_json::from_value(value).map_err(|error| ConfigError {
            field: "config",
            reason: error.to_string(),
        })?;
    let ramp = &config.startup.ramps.failed_action_penalty;
    for (field, invalid, reason) in [
        (
            "startup.ramps.failed_action_penalty.target_tick",
            ramp.target_tick < 1,
            "must be >= 1",
        ),
        (
            "startup.ramps.failed_action_penalty.start",
            !ramp.start.is_finite() || ramp.start < 0.0,
            "must be finite and >= 0.0",
        ),
        (
            "startup.ramps.failed_action_penalty.end",
            !ramp.end.is_finite() || ramp.end < 0.0,
            "must be finite and >= 0.0",
        ),
    ] {
        if invalid {
            return Err(ConfigError {
                field,
                reason: reason.into(),
            });
        }
    }
    config.normalize();
    config.apply_startup_overrides();
    Ok(config)
}
