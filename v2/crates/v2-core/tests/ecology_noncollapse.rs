use v2_core::ecology::{EcologyConfig, run_noncollapse_baseline};

#[test]
fn ecology_baseline_noncollapse_contract_holds() {
    let config = EcologyConfig::default();

    let result = run_noncollapse_baseline(42, 2_000, 120, &config);

    assert!(
        result.births_total > 0,
        "expected at least one successful birth"
    );

    let threshold = (result.baseline_population as f32 * 0.10).ceil();
    assert!(
        result.final_window_mean_population >= threshold,
        "mean final-window population {} below threshold {}",
        result.final_window_mean_population,
        threshold
    );

    let latest = result
        .snapshots
        .last()
        .expect("noncollapse run should emit snapshots");
    assert_eq!(latest.tick, 1_999);
    assert!(latest.genome_node_count_p90 >= latest.genome_node_count_p50);
}
