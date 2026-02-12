use serde::{Deserialize, Serialize};

const fn default_sensor_radius() -> u32 {
    12
}

const fn default_illegal_action_energy_penalty() -> f32 {
    0.01
}

const fn default_energy_per_inventory_attempt() -> f32 {
    0.005
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorldConfig {
    pub width: u32,
    pub height: u32,
    #[serde(default = "default_sensor_radius")]
    pub sensor_radius: u32,
    pub world_wrap: bool,
    pub initial_creatures: usize,
    pub max_creatures: usize,
    pub food_spawn_rate: f32,
    pub food_growth_rate: f32,
    pub food_spread_threshold: f32,
    pub food_spawn_floor_density: f32,
    pub food_max_density: f32,
    pub food_energy_value: f32,
    pub energy_per_tick_decay: f32,
    pub energy_per_move: f32,
    #[serde(default = "default_energy_per_inventory_attempt")]
    pub energy_per_inventory_attempt: f32,
    pub energy_per_compute_node: f32,
    pub energy_per_reproduce: f32,
    #[serde(default = "default_illegal_action_energy_penalty")]
    pub illegal_action_energy_penalty: f32,
    pub energy_initial: f32,
    pub energy_max: f32,
    pub min_reproduce_energy: f32,
    pub offspring_energy_fraction: f32,
    pub weight_mutation_rate: f32,
    pub weight_mutation_magnitude: f32,
    pub logic_node_mutation_rate: f32,
    pub structural_mutation_rate: f32,
    pub ticks_per_second: u32,
    pub paused: bool,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            width: 400,
            height: 400,
            sensor_radius: default_sensor_radius(),
            world_wrap: true,
            initial_creatures: 200,
            max_creatures: 500_000,
            food_spawn_rate: 0.02,
            food_growth_rate: 0.05,
            food_spread_threshold: 0.75,
            food_spawn_floor_density: 0.03,
            food_max_density: 1.0,
            food_energy_value: 0.35,
            energy_per_tick_decay: 0.01,
            energy_per_move: 0.02,
            energy_per_inventory_attempt: default_energy_per_inventory_attempt(),
            energy_per_compute_node: 0.005,
            energy_per_reproduce: 0.12,
            illegal_action_energy_penalty: default_illegal_action_energy_penalty(),
            energy_initial: 0.7,
            energy_max: 1.5,
            min_reproduce_energy: 1.0,
            offspring_energy_fraction: 0.45,
            weight_mutation_rate: 0.08,
            weight_mutation_magnitude: 0.18,
            logic_node_mutation_rate: 0.01,
            structural_mutation_rate: 0.02,
            ticks_per_second: 30,
            paused: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::WorldConfig;

    #[test]
    fn default_mutation_rates_are_lowered() {
        let cfg = WorldConfig::default();
        assert_eq!(cfg.sensor_radius, 12);
        assert_eq!(cfg.max_creatures, 500_000);
        assert!((cfg.weight_mutation_rate - 0.08).abs() < f32::EPSILON);
        assert!((cfg.logic_node_mutation_rate - 0.01).abs() < f32::EPSILON);
        assert!((cfg.structural_mutation_rate - 0.02).abs() < f32::EPSILON);
        assert!((cfg.illegal_action_energy_penalty - 0.01).abs() < f32::EPSILON);
        assert!((cfg.energy_per_inventory_attempt - 0.005).abs() < f32::EPSILON);
    }
}
