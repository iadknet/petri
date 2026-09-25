use super::*;
use crate::bench::indicators::structure_size_distribution;
use crate::bench::run::{build_report, deterministic_block_json, run_deterministic};
use crate::bench::tests::{small_profile, small_world_set_report};

/// Every level's `Display` string is the string it serializes to, which
/// is what `run_bench` prints and what the stored reports carry.
#[test]
fn comparison_level_displays_as_its_serialized_string() {
    for level in [
        ComparisonLevel::Ok,
        ComparisonLevel::Flag,
        ComparisonLevel::Severe,
        ComparisonLevel::New,
    ] {
        let serialized = serde_json::to_value(level).expect("a level always serializes");
        assert_eq!(
            level.to_string(),
            serialized.as_str().expect("levels serialize as strings")
        );
    }
    assert_eq!(ComparisonLevel::Severe.to_string(), "severe");
}

#[test]
fn historical_goal_indicators_default_temporal_memory_to_undefined() {
    let report: Report = serde_json::from_str(include_str!(
        "../../../tests/fixtures/synthetic-full-benchmark-v1.json"
    ))
    .unwrap();
    assert_eq!(
        report
            .deterministic
            .goal_indicators
            .temporal_memory_sensitivity,
        undefined_temporal_memory_sensitivity()
    );
    assert_eq!(
        report.deterministic.graph_work_definition,
        "graph_relax_iters: entered relaxation passes (before T11.F06)"
    );
}

#[test]
fn generated_report_with_requested_births_roundtrips_through_outer_indicator() {
    let report =
        build_report(&small_profile("gate"), "t11-f04-report-roundtrip").expect("a valid profile");
    let encoded = serde_json::to_string(&report).unwrap();
    let decoded: Report = serde_json::from_str(&encoded).unwrap();
    assert_eq!(
        deterministic_block_json(&decoded),
        deterministic_block_json(&report)
    );
}

#[test]
fn goal_indicators_defaults_mutational_neighborhood_to_undefined_when_the_field_is_absent() {
    let report = build_report(&small_profile("synthetic"), "t11-f01-serde-default-check")
        .expect("a valid profile");
    let mut value = serde_json::to_value(&report).expect("a report always serializes");
    value["deterministic"]["goal_indicators"]
        .as_object_mut()
        .expect("goal_indicators is an object")
        .remove("mutational_neighborhood");

    let reparsed: Report =
        serde_json::from_value(value).expect("a missing field falls back to the serde default");
    assert!(matches!(
        reparsed.deterministic.goal_indicators.mutational_neighborhood,
        Indicator::Undefined(ref value) if value == "Undefined"
    ));
}

#[test]
fn historical_mesh_fields_default_to_unmeasured() {
    let (det, _) = run_deterministic(&small_profile("goal")).expect("a valid profile");
    let Indicator::Defined(neighborhood) = det.goal_indicators.mutational_neighborhood else {
        panic!("goal reading");
    };
    let mut founder = serde_json::to_value(&neighborhood.founder).unwrap();
    founder.as_object_mut().unwrap().remove("mesh_execution");
    founder.as_object_mut().unwrap().remove("steering");
    founder.as_object_mut().unwrap().remove("generation");
    let founder: NeighborhoodFounderHalf = serde_json::from_value(founder).unwrap();
    assert!(matches!(founder.mesh_execution, Indicator::Undefined(_)));
    assert!(matches!(founder.steering, Indicator::Undefined(_)));
    assert_eq!(founder.generation, None);
    let Indicator::Defined(evolved) = neighborhood.evolved else {
        panic!("evolved reading");
    };
    let mut seed = serde_json::to_value(&evolved.per_seed[0]).unwrap();
    seed.as_object_mut()
        .unwrap()
        .remove("generation_distribution");
    seed.as_object_mut().unwrap().remove("steering_pooled");
    for sample in seed["sampled_genomes"].as_array_mut().unwrap() {
        sample.as_object_mut().unwrap().remove("mesh_execution");
        sample.as_object_mut().unwrap().remove("steering");
        sample.as_object_mut().unwrap().remove("generation");
    }
    let seed: NeighborhoodEvolvedSeed = serde_json::from_value(seed).unwrap();
    assert!(matches!(
        seed.generation_distribution,
        Indicator::Undefined(_)
    ));
    assert!(matches!(seed.steering_pooled, Indicator::Undefined(_)));
    for sample in seed.sampled_genomes {
        assert_eq!(sample.generation, None);
        assert!(matches!(sample.mesh_execution, Indicator::Undefined(_)));
        assert!(matches!(sample.steering, Indicator::Undefined(_)));
    }
}

/// A new report stamps every versioned indicator with its definition token,
/// pooled and per case.
#[test]
fn new_reports_carry_indicator_version_tokens() {
    let report = small_world_set_report();
    let indicators = &report.deterministic.goal_indicators;
    let Indicator::Defined(lineage) = &indicators.lineage_diversity else {
        panic!("world set defines lineage diversity");
    };
    assert_eq!(lineage.version.as_deref(), Some(LINEAGE_DIVERSITY_VERSION));
    assert_eq!(LINEAGE_DIVERSITY_VERSION, "lineage-diversity-v1");
    let Indicator::Defined(memory) = &indicators.memory_sensitivity else {
        panic!("world set defines memory sensitivity");
    };
    assert_eq!(memory.version.as_deref(), Some(MEMORY_SENSITIVITY_VERSION));
    assert_eq!(MEMORY_SENSITIVITY_VERSION, "memory-sensitivity-v1");
    assert_eq!(
        indicators
            .reachable_structure_size_distribution
            .version
            .as_deref(),
        Some(REACHABLE_STRUCTURE_VERSION)
    );
    assert_eq!(REACHABLE_STRUCTURE_VERSION, "reachable-structure-v1");
    assert!(!indicators.cases.is_empty());
    for case in &indicators.cases {
        assert_eq!(
            case.reachable_structure_size_distribution
                .as_ref()
                .expect("a new report measures every case")
                .version
                .as_deref(),
            Some(REACHABLE_STRUCTURE_VERSION)
        );
    }
    assert_eq!(
        structure_size_distribution(Vec::new()).version.as_deref(),
        Some(REACHABLE_STRUCTURE_VERSION),
        "an empty population still names the definition it was measured under"
    );
}

/// T12.F04 predates the version tokens. Its historical aggregates, copied
/// from its v1 summary into a fixture (summary v2 keeps only what reporting
/// reads), load and reserialize exactly as stored, with no `version` key.
#[test]
fn historical_goal_aggregates_load_without_versions_and_reserialize_them_absent() {
    let stored_indicators: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/t12-f04-historical-goal-aggregates.json"
    ))
    .unwrap();
    let stored_indicators = &stored_indicators;

    let lineage: LineageDiversity =
        serde_json::from_value(stored_indicators["lineage_diversity"].clone()).unwrap();
    assert_eq!(lineage.version, None);
    assert_eq!(
        serde_json::to_value(&lineage).unwrap(),
        stored_indicators["lineage_diversity"]
    );
    let memory: MemorySensitivity =
        serde_json::from_value(stored_indicators["memory_sensitivity"].clone()).unwrap();
    assert_eq!(memory.version, None);
    assert_eq!(
        serde_json::to_value(&memory).unwrap(),
        stored_indicators["memory_sensitivity"]
    );
    let structure: StructureSizeDistribution =
        serde_json::from_value(stored_indicators["reachable_structure_size_distribution"].clone())
            .unwrap();
    assert_eq!(structure.version, None);
    assert_eq!(
        serde_json::to_value(&structure).unwrap(),
        stored_indicators["reachable_structure_size_distribution"]
    );
    let cases = stored_indicators["cases"].as_array().unwrap();
    assert!(!cases.is_empty());
    for stored_case in cases {
        let distribution: StructureSizeDistribution =
            serde_json::from_value(stored_case["reachable_structure_size_distribution"].clone())
                .expect("the stored report measured every case");
        assert_eq!(distribution.version, None);
        assert_eq!(
            serde_json::to_value(&distribution).unwrap(),
            stored_case["reachable_structure_size_distribution"]
        );
    }
}

/// A value that is neither an undefined-indicator string nor a reading
/// object is rejected with an error naming both accepted shapes.
#[test]
fn indicator_rejects_a_non_string_non_object_value_naming_both_accepted_shapes() {
    let error = serde_json::from_str::<Indicator<LineageDiversity>>("5")
        .expect_err("a number is neither indicator shape");
    assert!(
        error
            .to_string()
            .contains("expected an undefined-indicator string or a measured reading object"),
        "unexpected error: {error}"
    );
}
