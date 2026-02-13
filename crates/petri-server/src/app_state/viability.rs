use petri_core::{World, WorldConfig};

use super::startup_draft::{build_world_config, StartupDraft};
use super::types::StartupViability;
use super::SimulationError;

pub(super) fn ensure_viable_start(
    config: &WorldConfig,
    initial_food_density: f32,
    seed: u64,
) -> Result<usize, SimulationError> {
    let mut probe = World::new(config.clone(), seed);
    probe.seed_food_density(initial_food_density);
    for _ in 0..100 {
        probe.tick();
        if probe.creature_count() == 0 {
            break;
        }
    }

    let survivors = probe.creature_count();
    if survivors == 0 {
        return Err(SimulationError::NonViableStartupConfig);
    }
    Ok(survivors)
}
pub(super) fn evaluate_startup_viability(
    base: &WorldConfig,
    draft: &StartupDraft,
    viability_probe_enabled: bool,
) -> StartupViability {
    if let Err(err) = draft.validate() {
        return StartupViability::from_error(err);
    }
    if !viability_probe_enabled {
        return StartupViability::viable();
    }

    let config = build_world_config(base, draft);
    match ensure_viable_start(
        &config,
        draft.initial_food_density,
        startup_probe_seed(draft),
    ) {
        Ok(_) => StartupViability::viable(),
        Err(err) => StartupViability::from_error(err),
    }
}

pub(super) fn startup_probe_seed(draft: &StartupDraft) -> u64 {
    let mut seed = 0xA11C_E5EED_u64;
    seed = seed
        .wrapping_mul(1_099_511_628_211)
        .wrapping_add(draft.initial_creatures as u64);
    seed = seed
        .wrapping_mul(1_099_511_628_211)
        .wrapping_add(draft.max_creatures as u64);
    seed = seed
        .wrapping_mul(1_099_511_628_211)
        .wrapping_add(draft.width as u64);
    seed = seed
        .wrapping_mul(1_099_511_628_211)
        .wrapping_add(draft.height as u64);
    seed = seed
        .wrapping_mul(1_099_511_628_211)
        .wrapping_add(draft.sensor_radius as u64);
    seed = seed
        .wrapping_mul(1_099_511_628_211)
        .wrapping_add(draft.initial_food_density.to_bits() as u64);
    seed = seed
        .wrapping_mul(1_099_511_628_211)
        .wrapping_add(draft.energy_initial.to_bits() as u64);
    seed = seed
        .wrapping_mul(1_099_511_628_211)
        .wrapping_add(draft.food_spawn_rate.to_bits() as u64);
    seed = seed
        .wrapping_mul(1_099_511_628_211)
        .wrapping_add(draft.food_growth_rate.to_bits() as u64);
    seed = seed
        .wrapping_mul(1_099_511_628_211)
        .wrapping_add(draft.food_spread_threshold.to_bits() as u64);
    seed = seed
        .wrapping_mul(1_099_511_628_211)
        .wrapping_add(draft.food_spawn_floor_density.to_bits() as u64);
    seed = seed
        .wrapping_mul(1_099_511_628_211)
        .wrapping_add(draft.energy_per_tick_decay.to_bits() as u64);
    seed = seed
        .wrapping_mul(1_099_511_628_211)
        .wrapping_add(draft.energy_per_think_step.to_bits() as u64);
    seed = seed
        .wrapping_mul(1_099_511_628_211)
        .wrapping_add(draft.energy_per_move.to_bits() as u64);
    seed = seed
        .wrapping_mul(1_099_511_628_211)
        .wrapping_add(u64::from(draft.world_wrap));
    seed
}
