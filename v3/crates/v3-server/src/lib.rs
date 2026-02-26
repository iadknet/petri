//! v3-server: HTTP/WS transport over v3-core.

pub mod error;
pub mod handlers;
pub mod state;
pub mod types;
pub mod ws;

use axum::routing::{get, patch, post};

pub fn router(state: state::AppState) -> axum::Router {
    axum::Router::new()
        .route("/v3/simulation/startup", post(handlers::lifecycle::startup))
        .route("/v3/simulation/start", post(handlers::lifecycle::start))
        .route("/v3/simulation/pause", post(handlers::lifecycle::pause_sim))
        .route("/v3/simulation/step", post(handlers::lifecycle::step))
        .route("/v3/simulation/status", get(handlers::status::get_status))
        .route("/v3/simulation/frame", get(handlers::status::get_frame))
        .route("/v3/simulation/config", get(handlers::status::get_config))
        .route(
            "/v3/simulation/config",
            patch(handlers::status::patch_config),
        )
        .route("/v3/simulation/paint", post(handlers::paint::paint))
        .route("/v3/ws", get(ws::ws_handler))
        .with_state(state)
}
