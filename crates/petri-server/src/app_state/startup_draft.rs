use serde::{Deserialize, Serialize};

use petri_core::WorldConfig;

use super::SimulationError;

const MIN_ENERGY_PER_THINK_STEP: f32 = 0.0001;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StartupDraft {
    pub initial_creatures: usize,
    pub max_creatures: usize,
    pub width: u32,
    pub height: u32,
    pub sensor_radius: u32,
    pub initial_food_density: f32,
    pub energy_initial: f32,
    pub food_spawn_rate: f32,
    pub food_growth_rate: f32,
    pub food_spread_threshold: f32,
    pub food_spawn_floor_density: f32,
    pub energy_per_tick_decay: f32,
    pub energy_per_think_step: f32,
    pub energy_per_move: f32,
    pub energy_per_inventory_attempt: f32,
    pub illegal_action_energy_penalty: f32,
    pub world_wrap: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct StartupDraftPatch {
    pub initial_creatures: Option<usize>,
    pub max_creatures: Option<usize>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub sensor_radius: Option<u32>,
    pub initial_food_density: Option<f32>,
    pub energy_initial: Option<f32>,
    pub food_spawn_rate: Option<f32>,
    pub food_growth_rate: Option<f32>,
    pub food_spread_threshold: Option<f32>,
    pub food_spawn_floor_density: Option<f32>,
    pub energy_per_tick_decay: Option<f32>,
    pub energy_per_think_step: Option<f32>,
    pub energy_per_move: Option<f32>,
    pub energy_per_inventory_attempt: Option<f32>,
    pub illegal_action_energy_penalty: Option<f32>,
    pub world_wrap: Option<bool>,
}
impl StartupDraft {
    pub fn viable_default() -> Self {
        Self {
            initial_creatures: 300,
            max_creatures: 500_000,
            width: 400,
            height: 400,
            sensor_radius: 12,
            initial_food_density: 0.15,
            energy_initial: 0.7,
            food_spawn_rate: 0.05,
            food_growth_rate: 0.10,
            food_spread_threshold: 0.75,
            food_spawn_floor_density: 0.03,
            energy_per_tick_decay: 0.01,
            energy_per_think_step: 0.005,
            energy_per_move: 0.02,
            energy_per_inventory_attempt: 0.005,
            illegal_action_energy_penalty: 0.01,
            world_wrap: true,
        }
    }

    pub(super) fn apply_patch(&mut self, patch: StartupDraftPatch) -> bool {
        let mut changed = false;

        if let Some(v) = patch.initial_creatures {
            changed |= self.initial_creatures != v;
            self.initial_creatures = v;
        }
        if let Some(v) = patch.max_creatures {
            changed |= self.max_creatures != v;
            self.max_creatures = v;
        }
        if let Some(v) = patch.width {
            changed |= self.width != v;
            self.width = v;
        }
        if let Some(v) = patch.height {
            changed |= self.height != v;
            self.height = v;
        }
        if let Some(v) = patch.sensor_radius {
            changed |= self.sensor_radius != v;
            self.sensor_radius = v;
        }
        if let Some(v) = patch.initial_food_density {
            changed |= (self.initial_food_density - v).abs() > f32::EPSILON;
            self.initial_food_density = v;
        }
        if let Some(v) = patch.energy_initial {
            changed |= (self.energy_initial - v).abs() > f32::EPSILON;
            self.energy_initial = v;
        }
        if let Some(v) = patch.food_spawn_rate {
            changed |= (self.food_spawn_rate - v).abs() > f32::EPSILON;
            self.food_spawn_rate = v;
        }
        if let Some(v) = patch.food_growth_rate {
            changed |= (self.food_growth_rate - v).abs() > f32::EPSILON;
            self.food_growth_rate = v;
        }
        if let Some(v) = patch.food_spread_threshold {
            changed |= (self.food_spread_threshold - v).abs() > f32::EPSILON;
            self.food_spread_threshold = v;
        }
        if let Some(v) = patch.food_spawn_floor_density {
            changed |= (self.food_spawn_floor_density - v).abs() > f32::EPSILON;
            self.food_spawn_floor_density = v;
        }
        if let Some(v) = patch.energy_per_tick_decay {
            changed |= (self.energy_per_tick_decay - v).abs() > f32::EPSILON;
            self.energy_per_tick_decay = v;
        }
        if let Some(v) = patch.energy_per_think_step {
            changed |= (self.energy_per_think_step - v).abs() > f32::EPSILON;
            self.energy_per_think_step = v;
        }
        if let Some(v) = patch.energy_per_move {
            changed |= (self.energy_per_move - v).abs() > f32::EPSILON;
            self.energy_per_move = v;
        }
        if let Some(v) = patch.energy_per_inventory_attempt {
            changed |= (self.energy_per_inventory_attempt - v).abs() > f32::EPSILON;
            self.energy_per_inventory_attempt = v;
        }
        if let Some(v) = patch.illegal_action_energy_penalty {
            changed |= (self.illegal_action_energy_penalty - v).abs() > f32::EPSILON;
            self.illegal_action_energy_penalty = v;
        }
        if let Some(v) = patch.world_wrap {
            changed |= self.world_wrap != v;
            self.world_wrap = v;
        }

        changed
    }

    pub(super) fn validate(&self) -> Result<(), SimulationError> {
        validate_range(
            "initial_creatures",
            self.initial_creatures as f64,
            1.0,
            20_000.0,
        )?;
        validate_range("max_creatures", self.max_creatures as f64, 1.0, 500_000.0)?;
        validate_range("width", self.width as f64, 50.0, 2_000.0)?;
        validate_range("height", self.height as f64, 50.0, 2_000.0)?;
        validate_range("sensor_radius", self.sensor_radius as f64, 1.0, 64.0)?;
        validate_range("energy_initial", self.energy_initial as f64, 0.01, 20.0)?;
        validate_range(
            "initial_food_density",
            self.initial_food_density as f64,
            0.0,
            1.0,
        )?;
        validate_range("food_spawn_rate", self.food_spawn_rate as f64, 0.0, 1.0)?;
        validate_range("food_growth_rate", self.food_growth_rate as f64, 0.0, 1.0)?;
        validate_range(
            "food_spread_threshold",
            self.food_spread_threshold as f64,
            0.0,
            1.0,
        )?;
        validate_range(
            "food_spawn_floor_density",
            self.food_spawn_floor_density as f64,
            0.0,
            1.0,
        )?;
        validate_range(
            "energy_per_tick_decay",
            self.energy_per_tick_decay as f64,
            0.0,
            0.50,
        )?;
        validate_range(
            "energy_per_think_step",
            self.energy_per_think_step as f64,
            MIN_ENERGY_PER_THINK_STEP as f64,
            0.50,
        )?;
        validate_range("energy_per_move", self.energy_per_move as f64, 0.0, 0.50)?;
        validate_range(
            "energy_per_inventory_attempt",
            self.energy_per_inventory_attempt as f64,
            0.0,
            0.20,
        )?;
        validate_range(
            "illegal_action_energy_penalty",
            self.illegal_action_energy_penalty as f64,
            0.0,
            0.50,
        )?;
        Ok(())
    }
}

fn validate_range(
    field: &'static str,
    value: f64,
    min: f64,
    max: f64,
) -> Result<(), SimulationError> {
    if value < min || value > max {
        return Err(SimulationError::InvalidStartupRange {
            field,
            min,
            max,
            actual: value,
        });
    }
    Ok(())
}
pub(super) fn build_world_config(base: &WorldConfig, draft: &StartupDraft) -> WorldConfig {
    let mut cfg = base.clone();
    cfg.width = draft.width;
    cfg.height = draft.height;
    cfg.sensor_radius = draft.sensor_radius;
    cfg.world_wrap = draft.world_wrap;
    cfg.initial_creatures = draft.initial_creatures;
    cfg.max_creatures = draft.max_creatures;
    cfg.energy_initial = draft.energy_initial;
    cfg.food_spawn_rate = draft.food_spawn_rate;
    cfg.food_growth_rate = draft.food_growth_rate;
    cfg.food_spread_threshold = draft.food_spread_threshold;
    cfg.food_spawn_floor_density = draft.food_spawn_floor_density;
    cfg.energy_per_tick_decay = draft.energy_per_tick_decay;
    cfg.energy_per_think_step = draft.energy_per_think_step;
    cfg.energy_per_move = draft.energy_per_move;
    cfg.energy_per_inventory_attempt = draft.energy_per_inventory_attempt;
    cfg.illegal_action_energy_penalty = draft.illegal_action_energy_penalty;
    cfg.paused = false;
    cfg
}

pub(super) fn startup_draft_from_config(
    config: &WorldConfig,
    initial_food_density: f32,
) -> StartupDraft {
    StartupDraft {
        initial_creatures: config.initial_creatures,
        max_creatures: config.max_creatures,
        width: config.width,
        height: config.height,
        sensor_radius: config.sensor_radius,
        initial_food_density,
        energy_initial: config.energy_initial,
        food_spawn_rate: config.food_spawn_rate,
        food_growth_rate: config.food_growth_rate,
        food_spread_threshold: config.food_spread_threshold,
        food_spawn_floor_density: config.food_spawn_floor_density,
        energy_per_tick_decay: config.energy_per_tick_decay,
        energy_per_think_step: config.energy_per_think_step,
        energy_per_move: config.energy_per_move,
        energy_per_inventory_attempt: config.energy_per_inventory_attempt,
        illegal_action_energy_penalty: config.illegal_action_energy_penalty,
        world_wrap: config.world_wrap,
    }
}
