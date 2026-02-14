use crate::ablation::{best_preset, config_for_preset, deterministic_score};
use crate::output::{
    AblationCompletedEvent, AblationResultEvent, AblationStartedEvent, ActionCounts,
    RunCompletedEvent, RunStartedEvent, TickSampleEvent, as_ndjson_line,
};
use v2_core::ecology::{EcologyConfig, run_noncollapse_baseline};

#[must_use]
pub fn run_simulation(ticks: u64, sample_every: u16, seed: u64) -> Vec<String> {
    let sample_every = sample_every.max(1);
    let mut lines = vec![as_ndjson_line(&RunStartedEvent::new(
        seed,
        ticks,
        sample_every,
    ))];
    if ticks == 0 {
        lines.push(as_ndjson_line(&RunCompletedEvent::new(0, 0, 0.0)));
        return lines;
    }

    let ticks_u32 = u32::try_from(ticks).unwrap_or(u32::MAX);
    let run = run_noncollapse_baseline(seed, ticks_u32, 120, &EcologyConfig::default());
    let snapshots = run.snapshots;

    let mut final_population = 0;
    let mut final_mean_energy = 0.0;

    for tick in 1..=ticks {
        let index = usize::try_from(tick.saturating_sub(1)).unwrap_or(usize::MAX);
        let snapshot = snapshots
            .get(index)
            .or_else(|| snapshots.last())
            .expect("noncollapse baseline yields snapshots");

        final_population = snapshot.population;
        final_mean_energy = snapshot.mean_energy;
        let should_emit = tick % u64::from(sample_every) == 0 || tick == ticks;
        if should_emit {
            let sample = TickSampleEvent::new(
                tick,
                final_population,
                final_mean_energy,
                snapshot.births_last_window,
                snapshot.deaths_last_window,
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
        let preset_seed = seed ^ (deterministic_score(seed, ticks, &preset) as u64);
        let config = config_for_preset(&preset);
        let ticks_u32 = u32::try_from(ticks.max(1)).unwrap_or(u32::MAX);
        let run = run_noncollapse_baseline(preset_seed, ticks_u32, 120, &config);
        let latest = run.snapshots.last().expect("ablation run has snapshots");
        let final_population = latest.population;
        let final_mean_energy = latest.mean_energy;
        let score = run.final_window_mean_population + run.births_total as f32 * 0.05;

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
