#[derive(Clone, Debug, PartialEq)]
pub struct EcologyConfig {
    pub resource_gradient_bands: u8,
    pub base_food_spawn_rate: f32,
    pub scarcity_multiplier_min: f32,
    pub scarcity_multiplier_max: f32,
    pub crowding_radius: u16,
    pub crowding_penalty_per_neighbor: f32,
    pub season_length_ticks: u32,
    pub season_transition_ticks: u32,
    pub barrier_density: f32,
    pub health_window_ticks: u32,
}

impl Default for EcologyConfig {
    fn default() -> Self {
        Self {
            resource_gradient_bands: 4,
            base_food_spawn_rate: 0.010,
            scarcity_multiplier_min: 0.25,
            scarcity_multiplier_max: 1.75,
            crowding_radius: 3,
            crowding_penalty_per_neighbor: 0.005,
            season_length_ticks: 500,
            season_transition_ticks: 50,
            barrier_density: 0.06,
            health_window_ticks: 100,
        }
    }
}

impl EcologyConfig {
    pub fn validate(&self) -> Result<(), EcologyConfigError> {
        if self.resource_gradient_bands == 0 {
            return Err(EcologyConfigError::InvalidGradientBands);
        }

        if self.base_food_spawn_rate <= 0.0 || !self.base_food_spawn_rate.is_finite() {
            return Err(EcologyConfigError::InvalidFoodSpawnRate);
        }

        if !(0.0..=1.0).contains(&self.barrier_density) {
            return Err(EcologyConfigError::InvalidBarrierDensity);
        }

        if !(0.0..=1.0).contains(&self.crowding_penalty_per_neighbor) {
            return Err(EcologyConfigError::InvalidCrowdingPenalty);
        }

        if self.scarcity_multiplier_min <= 0.0
            || self.scarcity_multiplier_max < self.scarcity_multiplier_min
            || !self.scarcity_multiplier_min.is_finite()
            || !self.scarcity_multiplier_max.is_finite()
        {
            return Err(EcologyConfigError::InvalidScarcityBounds);
        }

        if self.season_length_ticks == 0 {
            return Err(EcologyConfigError::InvalidSeasonLength);
        }

        if self.season_transition_ticks > self.season_length_ticks {
            return Err(EcologyConfigError::InvalidSeasonTransition);
        }

        if self.health_window_ticks == 0 {
            return Err(EcologyConfigError::InvalidHealthWindow);
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum EcologyConfigError {
    InvalidGradientBands,
    InvalidFoodSpawnRate,
    InvalidScarcityBounds,
    InvalidCrowdingPenalty,
    InvalidSeasonLength,
    InvalidSeasonTransition,
    InvalidBarrierDensity,
    InvalidHealthWindow,
}
