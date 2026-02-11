use std::sync::Arc;

use rand::rngs::SmallRng;
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, RwLock};

use petri_core::{World, WorldConfig};

use super::startup_draft::StartupDraft;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SimulationPhase {
    Idle,
    Starting,
    Running,
    Paused,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuntimeConfigPatch {
    pub paused: Option<bool>,
    pub ticks_per_second: Option<u32>,
    pub food_spawn_rate: Option<f32>,
    pub food_growth_rate: Option<f32>,
    pub food_spread_threshold: Option<f32>,
    pub food_spawn_floor_density: Option<f32>,
    pub food_max_density: Option<f32>,
    pub food_energy_value: Option<f32>,
    pub energy_per_tick_decay: Option<f32>,
    pub energy_per_move: Option<f32>,
    pub energy_per_compute_node: Option<f32>,
    pub energy_per_reproduce: Option<f32>,
    pub energy_max: Option<f32>,
    pub min_reproduce_energy: Option<f32>,
    pub offspring_energy_fraction: Option<f32>,
    pub max_creatures: Option<usize>,
    pub weight_mutation_rate: Option<f32>,
    pub weight_mutation_magnitude: Option<f32>,
    pub logic_node_mutation_rate: Option<f32>,
    pub structural_mutation_rate: Option<f32>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SimulationStatus {
    pub phase: SimulationPhase,
    pub initialization_stage: Option<&'static str>,
    pub viability_probe_enabled: bool,
    pub run_id: Option<u64>,
    pub seed: Option<u64>,
    pub tick: u64,
    pub population: usize,
    pub average_energy: f32,
    pub pending_restart: bool,
    pub startup_viable: bool,
    pub startup_viability_code: Option<&'static str>,
    pub startup_viability_message: Option<String>,
    pub startup_draft: StartupDraft,
}

#[derive(Debug)]
pub enum SimulationError {
    InvalidStartupRange {
        field: &'static str,
        min: f64,
        max: f64,
        actual: f64,
    },
    AlreadyRunning,
    NoActiveRun,
    NonViableStartupConfig,
}

impl SimulationError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidStartupRange { .. } => "startup_value_out_of_range",
            Self::AlreadyRunning => "simulation_already_running",
            Self::NoActiveRun => "no_active_simulation",
            Self::NonViableStartupConfig => "non_viable_startup_config",
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::InvalidStartupRange {
                field,
                min,
                max,
                actual,
            } => format!("{field} must be in range [{min}, {max}] but received {actual}",),
            Self::AlreadyRunning => "simulation is already active; use restart instead".into(),
            Self::NoActiveRun => "cannot restart because no simulation has been started yet".into(),
            Self::NonViableStartupConfig => {
                "startup configuration failed viability probe (100 ticks ended with extinction)"
                    .into()
            }
        }
    }
}

pub(super) struct SimulationRun {
    pub(super) run_id: u64,
    pub(super) seed: u64,
    pub(super) world: World,
}

#[derive(Clone, Debug)]
pub(super) struct StartupViability {
    pub(super) is_viable: bool,
    pub(super) code: Option<&'static str>,
    pub(super) message: Option<String>,
}

impl StartupViability {
    pub(super) fn viable() -> Self {
        Self {
            is_viable: true,
            code: None,
            message: None,
        }
    }

    pub(super) fn from_error(err: SimulationError) -> Self {
        Self {
            is_viable: false,
            code: Some(err.code()),
            message: Some(err.message()),
        }
    }
}

pub struct SimulationState {
    pub(super) phase: SimulationPhase,
    pub(super) initialization_stage: Option<&'static str>,
    pub(super) viability_probe_enabled: bool,
    pub(super) run: Option<SimulationRun>,
    pub(super) startup_draft: StartupDraft,
    pub(super) startup_viability: StartupViability,
    pub(super) pending_restart: bool,
    pub(super) runtime_config: WorldConfig,
    pub(super) rng: SmallRng,
    pub(super) next_run_id: u64,
}

#[derive(Clone)]
pub struct AppState {
    pub simulation: Arc<RwLock<SimulationState>>,
    pub frames_tx: broadcast::Sender<Vec<u8>>,
}

#[derive(Clone, Copy, Debug)]
pub struct AppStateOptions {
    pub viability_probe_enabled: bool,
}

impl Default for AppStateOptions {
    fn default() -> Self {
        Self {
            viability_probe_enabled: true,
        }
    }
}
