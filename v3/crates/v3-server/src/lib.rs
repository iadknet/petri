pub mod api;
pub mod state;

use axum::routing::{get, post};
use axum::Router;
use std::sync::Arc;
use tokio::sync::RwLock;

use state::ServerState;

pub fn build_router(state: Arc<RwLock<ServerState>>) -> Router {
    Router::new()
        .route("/v3/simulation/status", get(api::get_status))
        .route("/v3/simulation/start", post(api::start_simulation))
        .route("/v3/simulation/pause", post(api::pause_simulation))
        .with_state(state)
}
