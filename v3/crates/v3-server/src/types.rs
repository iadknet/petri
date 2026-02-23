use sha2::{Digest, Sha256};
use v3_core::config::SimulationConfig;

pub const PROTOCOL_VERSION: &str = "v3alpha1";

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

/// Request body for the `POST /v3/simulation/step` endpoint.
#[derive(serde::Deserialize, Default)]
pub struct StepRequest {
    #[serde(default = "default_one")]
    pub steps: u32,
}

fn default_one() -> u32 {
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deep_merge_overwrites_leaf_values() {
        let mut base = serde_json::json!({"a": 1, "b": 2});
        let patch = serde_json::json!({"b": 99});
        deep_merge(&mut base, patch);
        assert_eq!(base["a"], 1);
        assert_eq!(base["b"], 99);
    }

    #[test]
    fn deep_merge_preserves_unpatched_keys() {
        let mut base = serde_json::json!({"a": {"x": 1, "y": 2}});
        let patch = serde_json::json!({"a": {"x": 10}});
        deep_merge(&mut base, patch);
        assert_eq!(base["a"]["x"], 10);
        assert_eq!(base["a"]["y"], 2);
    }

    #[test]
    fn sort_json_keys_recursive_sorts_all_levels() {
        let val = serde_json::json!({"z": 1, "a": {"m": 1, "b": 2}});
        let sorted = sort_json_keys_recursive(val);
        if let serde_json::Value::Object(ref map) = sorted {
            let keys: Vec<&str> = map.keys().map(|s| s.as_str()).collect();
            assert_eq!(keys, vec!["a", "z"]);
        } else {
            panic!("expected object");
        }
        if let serde_json::Value::Object(ref inner) = sorted["a"] {
            let keys: Vec<&str> = inner.keys().map(|s| s.as_str()).collect();
            assert_eq!(keys, vec!["b", "m"]);
        } else {
            panic!("expected nested object");
        }
    }

    #[test]
    fn config_digest_is_deterministic() {
        let cfg = SimulationConfig::default();
        let d1 = config_digest(&cfg);
        let d2 = config_digest(&cfg);
        assert_eq!(d1, d2);
        assert!(d1.starts_with("sha256:"));
    }

    #[test]
    fn config_digest_changes_on_config_change() {
        let cfg1 = SimulationConfig::default();
        let mut cfg2 = SimulationConfig::default();
        cfg2.world.width = 999;
        let d1 = config_digest(&cfg1);
        let d2 = config_digest(&cfg2);
        assert_ne!(d1, d2);
    }
}
