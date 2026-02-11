use petri_core::WorldConfig;

use super::RuntimeConfigPatch;

pub(super) fn apply_runtime_patch(config: &mut WorldConfig, patch: &RuntimeConfigPatch) {
    if let Some(paused) = patch.paused {
        config.paused = paused;
    }
    if let Some(tps) = patch.ticks_per_second {
        config.ticks_per_second = tps.max(1);
    }
    if let Some(rate) = patch.food_spawn_rate {
        config.food_spawn_rate = rate.clamp(0.0, 1.0);
    }
    if let Some(rate) = patch.food_growth_rate {
        config.food_growth_rate = rate.clamp(0.0, 1.0);
    }
    if let Some(threshold) = patch.food_spread_threshold {
        config.food_spread_threshold = threshold.clamp(0.0, 1.0);
    }
    if let Some(floor) = patch.food_spawn_floor_density {
        config.food_spawn_floor_density = floor.clamp(0.0, 1.0);
    }
    if let Some(density) = patch.food_max_density {
        config.food_max_density = density.max(0.01);
    }
    if let Some(value) = patch.food_energy_value {
        config.food_energy_value = value.max(0.0);
    }
    if let Some(value) = patch.energy_per_tick_decay {
        config.energy_per_tick_decay = value.max(0.0);
    }
    if let Some(value) = patch.energy_per_move {
        config.energy_per_move = value.max(0.0);
    }
    if let Some(value) = patch.energy_per_compute_node {
        config.energy_per_compute_node = value.max(0.0);
    }
    if let Some(value) = patch.energy_per_reproduce {
        config.energy_per_reproduce = value.max(0.0);
    }
    if let Some(value) = patch.energy_max {
        config.energy_max = value.max(0.01);
    }
    if let Some(value) = patch.min_reproduce_energy {
        config.min_reproduce_energy = value.max(0.0);
    }
    if let Some(value) = patch.offspring_energy_fraction {
        config.offspring_energy_fraction = value.clamp(0.0, 1.0);
    }
    if let Some(max_creatures) = patch.max_creatures {
        config.max_creatures = max_creatures.max(1);
    }
    if let Some(rate) = patch.weight_mutation_rate {
        config.weight_mutation_rate = rate.clamp(0.0, 1.0);
    }
    if let Some(magnitude) = patch.weight_mutation_magnitude {
        config.weight_mutation_magnitude = magnitude.max(0.0);
    }
    if let Some(rate) = patch.logic_node_mutation_rate {
        config.logic_node_mutation_rate = rate.clamp(0.0, 1.0);
    }
    if let Some(rate) = patch.structural_mutation_rate {
        config.structural_mutation_rate = rate.clamp(0.0, 1.0);
    }
}
