#[cfg(test)]
use v3_core::config::SimulationConfig;
pub use v3_core::config::{config_digest, deep_merge, sort_json_keys_recursive};

pub const PROTOCOL_VERSION: &str = "v3alpha3";

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
