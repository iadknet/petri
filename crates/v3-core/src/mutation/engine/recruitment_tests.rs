use super::*;
use crate::creature::founder::v3alpha1_founder_genome;
use rand::{rngs::SmallRng, SeedableRng};

#[test]
fn recruitment_off_preserves_pre_feature_genomes_events_and_rng() {
    let config = MutationConfig::default();
    let mut hash = 0xcbf29ce484222325u64;
    for seed in 0..256 {
        let mut genome = v3alpha1_founder_genome();
        let mut rng = SmallRng::seed_from_u64(seed);
        let summary = MutationEngine::apply_mutations_with_food_type_count(
            &mut genome,
            &config,
            &[0, 1, 2],
            ParentExecuted::NONE,
            &mut rng,
            2,
        );
        let bytes = format!(
            "{}{:?}{}",
            serde_json::to_string(&genome).unwrap(),
            summary.events,
            rng.gen::<u64>()
        );
        for byte in bytes.bytes() {
            hash = (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3);
        }
    }
    assert_eq!(
        hash, 3980442848510747838,
        "pin captured before recruitment implementation"
    );
}

#[test]
fn recruitment_configuration_accepts_opt_in_and_defaults_to_off() {
    let original = serde_json::to_value(MutationConfig::default()).unwrap();
    for arm in ["Off", "SingleChannel", "WholeFamily"] {
        let mut json = original.clone();
        json["neutral_input_recruitment"] = serde_json::json!(arm);
        let decoded: MutationConfig = serde_json::from_value(json).unwrap();
        assert_eq!(
            serde_json::to_value(decoded).unwrap()["neutral_input_recruitment"],
            arm
        );
    }
    let mut omitted = original;
    omitted
        .as_object_mut()
        .unwrap()
        .remove("neutral_input_recruitment");
    let decoded: MutationConfig = serde_json::from_value(omitted).unwrap();
    assert_eq!(
        serde_json::to_value(decoded).unwrap()["neutral_input_recruitment"],
        "Off"
    );
}

proptest::proptest! {
    #[test]
    fn recruitment_is_admitted_only_by_opt_in_and_never_under_restricted_pressure(seed in proptest::prelude::any::<u64>(), whole in proptest::prelude::any::<bool>()) {
        use crate::config::NeutralInputRecruitment as Arm;
        let mut config = MutationConfig { neutral_input_recruitment: if whole { Arm::WholeFamily } else { Arm::SingleChannel }, per_unit_rate: 1.0, mesh_layer_probability: 0.0, ..MutationConfig::default() };
        let mut genome = v3alpha1_founder_genome();
        let summary = MutationEngine::apply_mutations_on_units(&mut genome, 64, &config, &[0, 1, 2], ParentExecuted::NONE, &mut SmallRng::seed_from_u64(seed), 2);
        proptest::prop_assert_eq!(summary.attempted_events, 64);
        proptest::prop_assert_eq!(summary.applied_events + summary.skipped_events, 64);
        let key = MutationOperator::GraphRecruitNeutralInput;
        proptest::prop_assert_eq!(summary.applied_by_operator.get(&key).copied().unwrap_or(0) as usize, summary.recruitment_deltas.len());
        for event in &summary.events {
            if event.operator == Some(key) { proptest::prop_assert!(event.target.is_some()); }
        }
        config.genome_size_pressure_enabled = true;
        config.genome_size_cap = 1;
        let mut genome = v3alpha1_founder_genome();
        let summary = MutationEngine::apply_mutations_on_units(&mut genome, 1, &config, &[], ParentExecuted::NONE, &mut SmallRng::seed_from_u64(seed), 2);
        proptest::prop_assert!(!summary.operator_funnel_by_operator.contains_key(&key));
    }
}
