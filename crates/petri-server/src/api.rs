use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::{header, HeaderValue, Method, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use futures_util::StreamExt;
use serde::Serialize;
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;

use petri_core::{WorldConfig, WorldSnapshot};

use crate::app_state::{RuntimeConfigPatch, SimulationError, StartupDraft, StartupDraftPatch};
use crate::AppState;

#[derive(Debug, Serialize)]
struct ApiErrorResponse {
    code: &'static str,
    message: String,
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/config", get(get_config).patch(patch_config))
        .route("/simulation/status", get(get_simulation_status))
        .route(
            "/simulation/startup-draft",
            get(get_startup_draft).patch(patch_startup_draft),
        )
        .route("/simulation/start", post(start_simulation))
        .route("/simulation/restart", post(restart_simulation))
        .route(
            "/simulation/snapshot",
            get(get_snapshot).post(load_snapshot),
        )
        .route("/ws", get(ws_handler))
        .layer(build_cors_layer())
        .with_state(state)
}

fn build_cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin([
            HeaderValue::from_static("http://127.0.0.1:5173"),
            HeaderValue::from_static("http://localhost:5173"),
        ])
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE])
}

async fn health() -> &'static str {
    "ok"
}

async fn get_config(State(state): State<AppState>) -> Json<WorldConfig> {
    Json(state.current_runtime_config().await)
}

async fn patch_config(
    State(state): State<AppState>,
    Json(patch): Json<RuntimeConfigPatch>,
) -> Json<WorldConfig> {
    Json(state.patch_runtime_config(patch).await)
}

async fn get_simulation_status(State(state): State<AppState>) -> impl IntoResponse {
    Json(state.simulation_status().await)
}

async fn get_startup_draft(State(state): State<AppState>) -> Json<StartupDraft> {
    Json(state.startup_draft().await)
}

async fn patch_startup_draft(
    State(state): State<AppState>,
    Json(patch): Json<StartupDraftPatch>,
) -> Result<Json<StartupDraft>, (StatusCode, Json<ApiErrorResponse>)> {
    state
        .patch_startup_draft(patch)
        .await
        .map(Json)
        .map_err(map_simulation_error)
}

async fn start_simulation(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiErrorResponse>)> {
    state
        .start_simulation()
        .await
        .map(Json)
        .map_err(map_simulation_error)
}

async fn restart_simulation(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiErrorResponse>)> {
    state
        .restart_simulation()
        .await
        .map(Json)
        .map_err(map_simulation_error)
}

async fn get_snapshot(State(state): State<AppState>) -> Json<WorldSnapshot> {
    Json(state.simulation_snapshot().await)
}

async fn load_snapshot(
    State(state): State<AppState>,
    Json(snapshot): Json<WorldSnapshot>,
) -> Json<crate::app_state::SimulationStatus> {
    Json(state.load_simulation_snapshot(snapshot).await)
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| websocket_session(socket, state))
}

async fn websocket_session(mut socket: WebSocket, state: AppState) {
    let mut rx = state.frames_tx.subscribe();
    loop {
        tokio::select! {
            incoming = rx.recv() => {
                match incoming {
                    Ok(bytes) => {
                        if socket.send(Message::Binary(bytes.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {}
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            msg = socket.next() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None | Some(Err(_)) => break,
                    _ => {}
                }
            }
        }
    }
}

fn map_simulation_error(err: SimulationError) -> (StatusCode, Json<ApiErrorResponse>) {
    let status = match err {
        SimulationError::InvalidStartupRange { .. } => StatusCode::BAD_REQUEST,
        SimulationError::AlreadyRunning | SimulationError::NoActiveRun => StatusCode::CONFLICT,
        SimulationError::NonViableStartupConfig => StatusCode::UNPROCESSABLE_ENTITY,
    };

    let body = ApiErrorResponse {
        code: err.code(),
        message: err.message(),
    };

    (status, Json(body))
}
