use crate::ablation::{best_preset, deterministic_score};
use crate::output::{
    AblationCompletedEvent, AblationResultEvent, AblationStartedEvent, ActionCounts,
    RunCompletedEvent, RunStartedEvent, TickSampleEvent, as_ndjson_line,
};

#[must_use]
pub fn run_simulation(ticks: u64, sample_every: u16, seed: u64) -> Vec<String> {
    let sample_every = sample_every.max(1);
    let mut lines = vec![as_ndjson_line(&RunStartedEvent::new(
        seed,
        ticks,
        sample_every,
    ))];
    let mut final_population = baseline_population(seed);
    let mut final_mean_energy = baseline_energy(seed);

    for tick in 1..=ticks {
        final_population = population_at_tick(seed, tick);
        final_mean_energy = mean_energy_at_tick(seed, tick);
        let should_emit = tick % u64::from(sample_every) == 0 || tick == ticks;
        if should_emit {
            let sample = TickSampleEvent::new(
                tick,
                final_population,
                final_mean_energy,
                ((tick / 5) % 11) as u32,
                ((tick / 9) % 7) as u32,
                action_counts_for_tick(tick),
            );
            lines.push(as_ndjson_line(&sample));
        }
    }

    lines.push(as_ndjson_line(&RunCompletedEvent::new(
        ticks,
        final_population,
        final_mean_energy,
    )));
    lines
}

#[must_use]
pub fn run_ablation(ticks: u64, seed: u64, presets: Vec<String>) -> Vec<String> {
    let presets = if presets.is_empty() {
        vec!["default".to_string()]
    } else {
        presets
    };

    let mut lines = vec![as_ndjson_line(&AblationStartedEvent::new(
        seed,
        ticks,
        presets.clone(),
    ))];

    let mut scored = Vec::new();
    for preset in presets {
        let score = deterministic_score(seed, ticks, &preset);
        let final_population = 20 + ((score as u32) % 80);
        let final_mean_energy = 5.0 + (score / 10.0);
        lines.push(as_ndjson_line(&AblationResultEvent::new(
            preset.clone(),
            score,
            final_population,
            final_mean_energy,
        )));
        scored.push((preset, score));
    }

    lines.push(as_ndjson_line(&AblationCompletedEvent::new(
        scored.len() as u16,
        best_preset(&scored),
    )));
    lines
}

fn baseline_population(seed: u64) -> u32 {
    50 + (seed % 25) as u32
}

fn baseline_energy(seed: u64) -> f32 {
    10.0 + (seed % 11) as f32
}

fn population_at_tick(seed: u64, tick: u64) -> u32 {
    let baseline = baseline_population(seed);
    let growth = (tick % 17) as u32;
    baseline.saturating_add(growth)
}

fn mean_energy_at_tick(seed: u64, tick: u64) -> f32 {
    let baseline = baseline_energy(seed);
    let drift = (tick % 10) as f32 * 0.1;
    baseline + drift
}

fn action_counts_for_tick(tick: u64) -> ActionCounts {
    ActionCounts {
        r#move: ((tick + 1) % 9) as u32,
        eat: ((tick + 2) % 7) as u32,
        reproduce: ((tick + 3) % 5) as u32,
        inventory_pickup: ((tick + 4) % 4) as u32,
        inventory_put: ((tick + 5) % 4) as u32,
        noop: ((tick + 6) % 6) as u32,
    }
}
