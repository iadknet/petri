//! T20.F09 disposal: unqualified experimental operators cannot be configured.
use proptest::prelude::*;
use v3_core::config::MutationConfig;
use v3_core::mutation::{graph::GraphOperator, MutationOperator};

#[test]
fn production_schema_and_registry_omit_unqualified_operators() {
    let config = serde_json::to_value(MutationConfig::default()).unwrap();
    assert!(config.get("neutral_input_recruitment").is_none());
    assert!(config.get("structured_heritable_refinement").is_none());
    for name in GraphOperator::ALL
        .map(|operator| format!("{operator:?}"))
        .into_iter()
        .chain(MutationOperator::all().map(|operator| format!("{operator:?}")))
    {
        assert!(!name.contains("RecruitNeutralInput"));
        assert!(!name.contains("RefineHeritableStructure"));
    }
}

proptest! {
    #[test]
    fn retired_configuration_is_rejected_instead_of_silently_ignored(rate in 0.0..=1.0f64) {
        for (key, value) in [
            ("neutral_input_recruitment", serde_json::json!("Off")),
            ("neutral_input_recruitment", serde_json::json!("SingleChannel")),
            ("neutral_input_recruitment", serde_json::json!("WholeFamily")),
            ("structured_heritable_refinement", serde_json::json!(false)),
            ("structured_heritable_refinement", serde_json::json!(true)),
        ] {
            let config = MutationConfig { per_unit_rate: rate, ..MutationConfig::default() };
            let mut encoded = serde_json::to_value(config).unwrap();
            encoded[key] = value;
            let decoded = serde_json::from_value::<MutationConfig>(encoded);
            prop_assert!(decoded.is_err(), "retired key {key} must fail explicitly");
        }
    }
}
