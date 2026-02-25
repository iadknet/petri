use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, Mutex};
use v3_core::config::SimulationConfig;
use v3_core::simulation::{seed_simulation, Simulation};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SimulationStatus {
    Idle,
    Running,
    Paused,
}

pub struct SimHandle {
    pub sim: Simulation,
    pub status: SimulationStatus,
}

impl SimHandle {
    pub fn new_default() -> Self {
        let config = SimulationConfig::default();
        let sim = seed_simulation(config, 0);
        Self {
            sim,
            status: SimulationStatus::Idle,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LastTickActions {
    #[serde(rename = "move")]
    pub move_count: u32,
    pub eat: u32,
    pub reproduce: u32,
    pub noop: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StatusPayload {
    pub state: SimulationStatus,
    pub population: usize,
    pub mean_energy: f32,
    pub last_tick_actions: LastTickActions,
    pub reproduction_actions_attempted_total: u64,
    pub reproduction_actions_spawned_total: u64,
    pub reproduction_actions_rejected_total: u64,
    pub last_tick_compute_total_mean: f32,
    pub last_tick_compute_total_min: f32,
    pub last_tick_compute_total_max: f32,
    pub last_tick_compute_vm_mean: f32,
    pub last_tick_compute_graph_mean: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreatureSnapshot {
    pub id: u64,
    pub x: u16,
    pub y: u16,
    pub energy: f32,
    pub generation: u64,
    pub phenotype_rgb: [u8; 3],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FoodCell {
    pub x: u16,
    pub y: u16,
    pub density: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BarrierCell {
    pub x: u16,
    pub y: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FramePayload {
    pub width: u16,
    pub height: u16,
    pub creatures: Vec<CreatureSnapshot>,
    pub food: Vec<FoodCell>,
    pub barriers: Vec<BarrierCell>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HealthPayload {
    pub population: usize,
    pub mean_energy: f32,
    pub mutation_events_attempted_total: u64,
    pub mutation_events_applied_total: u64,
    pub mutation_events_skipped_total: u64,
    pub reproduction_actions_attempted_total: u64,
    pub reproduction_actions_spawned_total: u64,
    pub reproduction_actions_rejected_total: u64,
    pub reproduction_actions_rejected_total_by_reason: HashMap<String, u64>,
    pub mutation_events_skipped_total_by_reason: HashMap<String, u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WsFrame {
    pub tick: u64,
    pub status: StatusPayload,
    pub frame: FramePayload,
    pub health: HealthPayload,
}

#[derive(Clone)]
pub struct AppState {
    pub sim: Arc<Mutex<SimHandle>>,
    pub ws_tx: broadcast::Sender<Vec<u8>>,
}

impl AppState {
    pub fn new() -> Self {
        let (ws_tx, _) = broadcast::channel(16);
        Self {
            sim: Arc::new(Mutex::new(SimHandle::new_default())),
            ws_tx,
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
