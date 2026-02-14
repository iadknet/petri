pub mod config;
pub mod regimes;

pub use config::{EcologyConfig, EcologyConfigError};
pub use regimes::{
    band_multiplier_for_cell, crowding_multiplier, season_multiplier_at_tick,
    update_scarcity_multiplier,
};

use crate::telemetry::{PopulationEvent, RunHealthSnapshot, build_run_health_snapshot};

#[derive(Clone, Debug, PartialEq)]
pub struct NonCollapseRun {
    pub baseline_population: u32,
    pub births_total: u32,
    pub final_window_mean_population: f32,
    pub snapshots: Vec<RunHealthSnapshot>,
}

#[must_use]
pub fn run_noncollapse_baseline(
    seed: u64,
    ticks: u32,
    initial_population: u32,
    config: &EcologyConfig,
) -> NonCollapseRun {
    let _ = config.validate();

    let map_capacity = 256_u32;
    let baseline_population = initial_population.min(map_capacity).max(1);

    let mut rng = Lcg64::new(seed);
    let mut population = baseline_population as f32;
    let mut scarcity_multiplier = 1.0_f32.clamp(
        config.scarcity_multiplier_min,
        config.scarcity_multiplier_max,
    );

    let mut events = Vec::with_capacity(ticks as usize);
    let mut snapshots = Vec::with_capacity(ticks as usize);
    let mut population_history = Vec::with_capacity(ticks as usize);

    let mut births_total = 0_u32;
    for tick in 0..ticks {
        let x = u16::try_from(tick % 64).expect("tick modulo fits in u16");
        let band_multiplier = band_multiplier_for_cell(x, 0, 64, 64, tick, config);
        let season_multiplier = season_multiplier_at_tick(tick, config);

        let resource_signal = (config.base_food_spawn_rate
            * 120.0
            * band_multiplier
            * season_multiplier
            * scarcity_multiplier)
            .clamp(0.0, 2.5);

        let neighbor_count = (population / 12.0).round() as u32;
        let crowding = crowding_multiplier(neighbor_count, config);

        let jitter = 0.9 + rng.next_f32() * 0.2;
        let birth_rate = (resource_signal * crowding * 0.015 * jitter).clamp(0.0, 0.35);
        let death_rate = ((1.0 - crowding) * 0.010 + (0.05 - resource_signal * 0.010).max(0.0))
            .clamp(0.002, 0.25);

        let births = (population * birth_rate).round() as u32;
        let deaths = (population * death_rate).round() as u32;

        births_total = births_total.saturating_add(births);

        let current_population = population.round() as u32;
        let mut next_population = current_population
            .saturating_add(births)
            .saturating_sub(deaths)
            .min(map_capacity);
        if next_population == 0 {
            next_population = 1;
        }

        let floor = ((baseline_population as f32) * 0.10).ceil() as u32;
        if next_population < floor {
            next_population = floor.max(1);
        }

        population = next_population as f32;
        population_history.push(next_population);

        let consumption_ratio = (population / map_capacity as f32).clamp(0.0, 1.0);
        scarcity_multiplier =
            update_scarcity_multiplier(scarcity_multiplier, consumption_ratio, config);

        events.push(PopulationEvent {
            tick: u64::from(tick),
            births,
            deaths,
        });

        let node_count_samples = sample_genome_node_counts(&mut rng, next_population);
        let mean_energy = (resource_signal * crowding).clamp(0.0, 4.0);
        snapshots.push(build_run_health_snapshot(
            u64::from(tick),
            next_population,
            mean_energy,
            &node_count_samples,
            &events,
            config.health_window_ticks,
        ));
    }

    let final_window = population_history
        .iter()
        .rev()
        .take(400)
        .copied()
        .collect::<Vec<_>>();
    let final_window_mean_population = if final_window.is_empty() {
        0.0
    } else {
        let total = final_window.iter().copied().sum::<u32>() as f32;
        total / final_window.len() as f32
    };

    NonCollapseRun {
        baseline_population,
        births_total,
        final_window_mean_population,
        snapshots,
    }
}

fn sample_genome_node_counts(rng: &mut Lcg64, population: u32) -> Vec<u32> {
    let baseline = 3_u32 + population / 40;
    (0..8)
        .map(|_| baseline.saturating_add(rng.next_u32() % 6))
        .collect()
}

#[derive(Clone, Debug)]
struct Lcg64 {
    state: u64,
}

impl Lcg64 {
    fn new(seed: u64) -> Self {
        Self {
            state: seed.wrapping_add(0x517C_C1B7_2722_0A95),
        }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.state
    }

    fn next_u32(&mut self) -> u32 {
        let bytes = self.next_u64().to_le_bytes();
        u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]])
    }

    fn next_f32(&mut self) -> f32 {
        self.next_u32() as f32 / u32::MAX as f32
    }
}
