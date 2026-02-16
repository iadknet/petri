use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::state::ServerState;

#[derive(Serialize)]
pub struct StatusResponse {
    pub tick: u64,
    pub running: bool,
    pub creature_count: usize,
}

pub async fn get_status(State(state): State<Arc<RwLock<ServerState>>>) -> Json<StatusResponse> {
    let state = state.read().await;
    Json(StatusResponse {
        tick: state.sim.tick_number,
        running: state.running,
        creature_count: state.sim.creatures.len(),
    })
}

pub async fn start_simulation(State(state): State<Arc<RwLock<ServerState>>>) -> StatusCode {
    let mut state = state.write().await;
    state.running = true;
    StatusCode::OK
}

pub async fn pause_simulation(State(state): State<Arc<RwLock<ServerState>>>) -> StatusCode {
    let mut state = state.write().await;
    state.running = false;
    StatusCode::OK
}
