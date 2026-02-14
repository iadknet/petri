use super::config::EcologyConfig;

#[must_use]
pub fn crowding_multiplier(neighbor_count: u32, config: &EcologyConfig) -> f32 {
    let raw = 1.0 - config.crowding_penalty_per_neighbor * neighbor_count as f32;
    raw.clamp(0.0, 1.0)
}

#[must_use]
pub fn season_multiplier_at_tick(tick: u32, config: &EcologyConfig) -> f32 {
    let season = u64::from(config.season_length_ticks.max(1));
    let transition = u64::from(config.season_transition_ticks);

    let tick = u64::from(tick);
    let phase = tick / season;
    let offset_in_phase = tick % season;

    let current = phase_factor(phase);
    if transition == 0 || offset_in_phase >= transition {
        return current;
    }

    let previous = if phase == 0 {
        current
    } else {
        phase_factor(phase - 1)
    };
    let alpha = offset_in_phase as f32 / transition as f32;
    lerp(previous, current, alpha)
}

#[must_use]
pub fn band_multiplier_for_cell(
    x: u16,
    _y: u16,
    width: u16,
    _height: u16,
    tick: u32,
    config: &EcologyConfig,
) -> f32 {
    let bands = usize::from(config.resource_gradient_bands.max(1));
    let width = usize::from(width.max(1));
    let x = usize::from(x);

    let band_index = ((x * bands) / width).min(bands - 1);
    let normalized = if bands <= 1 {
        0.0
    } else {
        band_index as f32 / (bands - 1) as f32
    };

    let gradient = lerp(
        config.scarcity_multiplier_min,
        config.scarcity_multiplier_max,
        normalized,
    );

    gradient * season_multiplier_at_tick(tick, config)
}

#[must_use]
pub fn update_scarcity_multiplier(
    previous_multiplier: f32,
    consumption_ratio: f32,
    config: &EcologyConfig,
) -> f32 {
    let consumption = consumption_ratio.clamp(0.0, 1.0);
    let delta = (0.5 - consumption) * 0.10;
    (previous_multiplier + delta).clamp(
        config.scarcity_multiplier_min,
        config.scarcity_multiplier_max,
    )
}

fn phase_factor(phase: u64) -> f32 {
    let table = [0.85_f32, 1.00_f32, 1.15_f32, 0.95_f32];
    table[(phase as usize) % table.len()]
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}
