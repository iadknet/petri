#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PopulationEvent {
    pub tick: u64,
    pub births: u32,
    pub deaths: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RunHealthSnapshot {
    pub tick: u64,
    pub population: u32,
    pub births_last_window: u32,
    pub deaths_last_window: u32,
    pub mean_energy: f32,
    pub genome_node_count_p50: u32,
    pub genome_node_count_p90: u32,
}

#[must_use]
pub fn build_run_health_snapshot(
    tick: u64,
    population: u32,
    mean_energy: f32,
    genome_node_counts: &[u32],
    events: &[PopulationEvent],
    health_window_ticks: u32,
) -> RunHealthSnapshot {
    let (births_last_window, deaths_last_window) =
        births_and_deaths_in_window(tick, events, u64::from(health_window_ticks));

    RunHealthSnapshot {
        tick,
        population,
        births_last_window,
        deaths_last_window,
        mean_energy,
        genome_node_count_p50: percentile(genome_node_counts, 0.50),
        genome_node_count_p90: percentile(genome_node_counts, 0.90),
    }
}

#[must_use]
pub fn births_and_deaths_in_window(
    tick: u64,
    events: &[PopulationEvent],
    health_window_ticks: u64,
) -> (u32, u32) {
    let window = health_window_ticks.max(1);
    let start_tick = tick.saturating_sub(window.saturating_sub(1));

    let mut births = 0_u32;
    let mut deaths = 0_u32;
    for event in events {
        if (start_tick..=tick).contains(&event.tick) {
            births = births.saturating_add(event.births);
            deaths = deaths.saturating_add(event.deaths);
        }
    }

    (births, deaths)
}

fn percentile(values: &[u32], percentile: f32) -> u32 {
    if values.is_empty() {
        return 0;
    }

    let mut sorted = values.to_vec();
    sorted.sort_unstable();

    let rank = ((sorted.len() - 1) as f32 * percentile.clamp(0.0, 1.0)).round() as usize;
    sorted[rank]
}
