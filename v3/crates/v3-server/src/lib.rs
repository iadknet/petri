//! v3-server: HTTP/WS transport over v3-core.

pub mod app_state;
pub mod command;
pub mod error;
pub mod handlers;
pub mod http;
pub mod query;
pub mod state;
pub mod transport;
pub mod types;
pub mod ws;

use axum::routing::{get, patch, post};

pub fn router(state: app_state::AppState) -> axum::Router {
    axum::Router::new()
        .route("/v3/simulation/startup", post(http::lifecycle::startup))
        .route("/v3/simulation/start", post(http::lifecycle::start))
        .route("/v3/simulation/pause", post(http::lifecycle::pause_sim))
        .route("/v3/simulation/step", post(http::lifecycle::step))
        .route("/v3/simulation/status", get(http::status::get_status))
        .route("/v3/simulation/snapshot", get(http::snapshot::get_snapshot))
        .route("/v3/simulation/config", get(http::status::get_config))
        .route("/v3/simulation/config", patch(http::status::patch_config))
        .route("/v3/simulation/paint", post(http::paint::paint))
        .route(
            "/v3/simulation/creature/:id",
            get(http::creature::get_creature),
        )
        .route(
            "/v3/simulation/creature/:id/sample",
            post(http::creature::start_sample),
        )
        .route(
            "/v3/simulation/creature/:id/sample",
            get(http::creature::get_sample),
        )
        .route("/v3/ws", get(ws::ws_handler))
        .with_state(state)
}
