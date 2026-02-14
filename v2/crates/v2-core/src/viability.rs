use crate::ecology::{EcologyConfig, run_noncollapse_baseline};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StartupViabilityGate {
    pub probe_ticks: u32,
    pub min_final_window_ratio: f32,
    pub require_births: bool,
}

impl Default for StartupViabilityGate {
    fn default() -> Self {
        Self {
            probe_ticks: 100,
            min_final_window_ratio: 0.10,
            require_births: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StartupViabilityResult {
    pub viable: bool,
    pub baseline_population: u32,
    pub births_total: u32,
    pub final_window_mean_population: f32,
    pub threshold_population: f32,
}

#[must_use]
pub fn run_startup_viability_gate(
    seed: u64,
    initial_population: u32,
    config: &EcologyConfig,
    gate: &StartupViabilityGate,
) -> StartupViabilityResult {
    let ticks = gate.probe_ticks.max(1);
    let ratio = gate.min_final_window_ratio.max(0.0);

    let run = run_noncollapse_baseline(seed, ticks, initial_population.max(1), config);
    let threshold_population = (run.baseline_population as f32 * ratio).ceil();

    let stable = run.final_window_mean_population >= threshold_population;
    let births_ok = !gate.require_births || run.births_total > 0;

    StartupViabilityResult {
        viable: stable && births_ok,
        baseline_population: run.baseline_population,
        births_total: run.births_total,
        final_window_mean_population: run.final_window_mean_population,
        threshold_population,
    }
}
