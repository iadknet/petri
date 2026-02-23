use std::sync::Arc;

use tokio::sync::{broadcast, Mutex};
use v3_core::config::SimulationConfig;
use v3_core::simulation::{seed_simulation, Simulation};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
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

#[derive(Clone, Debug, serde::Serialize)]
pub struct WsEvent {
    pub tick: u64,
    pub status_payload: serde_json::Value,
    pub frame_payload: serde_json::Value,
    pub health_payload: serde_json::Value,
}

#[derive(Clone)]
pub struct AppState {
    pub sim: Arc<Mutex<SimHandle>>,
    pub ws_tx: broadcast::Sender<WsEvent>,
}

impl AppState {
    pub fn new() -> Self {
        let (ws_tx, _) = broadcast::channel(256);
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
