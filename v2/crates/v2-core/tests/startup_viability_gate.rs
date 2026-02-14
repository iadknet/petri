use v2_core::ecology::EcologyConfig;
use v2_core::viability::{StartupViabilityGate, run_startup_viability_gate};

#[test]
fn standard_viability_gate_passes_noncollapse_baseline() {
    let config = EcologyConfig::default();
    let gate = StartupViabilityGate {
        probe_ticks: 2_000,
        min_final_window_ratio: 0.10,
        require_births: true,
    };

    let result = run_startup_viability_gate(42, 120, &config, &gate);
    assert!(result.viable);
    assert!(result.births_total > 0);
}

#[test]
fn standard_viability_gate_can_flag_birthless_seed_populations() {
    let config = EcologyConfig::default();
    let gate = StartupViabilityGate {
        probe_ticks: 200,
        min_final_window_ratio: 0.10,
        require_births: true,
    };

    let result = run_startup_viability_gate(42, 1, &config, &gate);
    assert!(!result.viable);
    assert_eq!(result.births_total, 0);
}
