use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use axum::Json;
use axum::Router;
use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::http::{Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use futures_util::StreamExt;
use tokio::sync::{Mutex, broadcast};
use tower_http::cors::{Any, CorsLayer};

use crate::api::{
    ErrorDetails, HealthPayload, PaintRequest, PaintResponse, ProtocolErrorCode, SimulationApi,
    SimulationError, StartupRequest,
};
use crate::ws::{WsEventEnvelope, ws_events_for_tick};

#[derive(Clone)]
pub struct ServerState {
    api: Arc<Mutex<SimulationApi>>,
    ws_tx: broadcast::Sender<WsEventEnvelope>,
}

impl ServerState {
    #[must_use]
    pub fn new() -> Self {
        let (ws_tx, _) = broadcast::channel(512);
        Self {
            api: Arc::new(Mutex::new(SimulationApi::new())),
            ws_tx,
        }
    }
}

impl Default for ServerState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn build_router_for_tests() -> Router {
    build_router(ServerState::new())
}

pub fn build_router(state: ServerState) -> Router {
    Router::new()
        .route("/v2/simulation/startup", post(startup_handler))
        .route("/v2/simulation/start", post(start_handler))
        .route("/v2/simulation/pause", post(pause_handler))
        .route("/v2/simulation/step", post(step_handler))
        .route("/v2/simulation/world/paint", post(paint_handler))
        .route("/v2/simulation/status", get(status_handler))
        .route("/v2/simulation/frame", get(frame_handler))
        .route("/v2/ws", get(ws_handler))
        .layer(cors_layer())
        .with_state(state)
}

fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any)
}

pub async fn run(bind: SocketAddr) -> Result<(), std::io::Error> {
    let state = ServerState::new();
    let tick_state = state.clone();
    tokio::spawn(async move {
        tick_loop(tick_state).await;
    });

    let listener = tokio::net::TcpListener::bind(bind).await?;
    println!("listening on http://{bind}");
    axum::serve(listener, build_router(state))
        .with_graceful_shutdown(shutdown_signal())
        .await
}

async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{SignalKind, signal};
        match signal(SignalKind::terminate()) {
            Ok(mut stream) => {
                let _ = stream.recv().await;
            }
            Err(_) => {
                std::future::pending::<()>().await;
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

async fn tick_loop(state: ServerState) {
    let mut interval = tokio::time::interval(Duration::from_millis(100));
    loop {
        interval.tick().await;
        let mut api = state.api.lock().await;
        if api.tick_running() {
            publish_snapshot(&api, &state.ws_tx);
        }
    }
}

async fn startup_handler(
    State(state): State<ServerState>,
    payload: Result<Option<Json<StartupRequest>>, axum::extract::rejection::JsonRejection>,
) -> Result<Json<crate::api::StartupResponse>, SimulationError> {
    let request = match payload {
        Ok(Some(json)) => json.0,
        Ok(None) => StartupRequest::default(),
        Err(rejection) => return Err(SimulationError::from(rejection)),
    };
    let mut api = state.api.lock().await;
    let response = api.startup(request);
    publish_snapshot(&api, &state.ws_tx);
    Ok(Json(response))
}

async fn start_handler(
    State(state): State<ServerState>,
) -> Result<Json<crate::api::LifecycleResponse>, SimulationError> {
    let mut api = state.api.lock().await;
    let response = api.start()?;
    publish_snapshot(&api, &state.ws_tx);
    Ok(Json(response))
}

async fn pause_handler(
    State(state): State<ServerState>,
) -> Result<Json<crate::api::LifecycleResponse>, SimulationError> {
    let mut api = state.api.lock().await;
    let response = api.pause()?;
    publish_snapshot(&api, &state.ws_tx);
    Ok(Json(response))
}

#[derive(Debug, Clone, serde::Deserialize)]
struct StepRequest {
    steps: Option<u16>,
}

async fn step_handler(
    State(state): State<ServerState>,
    payload: Result<Option<Json<StepRequest>>, axum::extract::rejection::JsonRejection>,
) -> Result<Json<crate::api::LifecycleResponse>, SimulationError> {
    let mut api = state.api.lock().await;
    let steps = match payload {
        Ok(payload) => payload.and_then(|json| json.steps),
        Err(rejection) => return Err(SimulationError::from(rejection)),
    };
    let response = api.step(steps)?;
    publish_snapshot(&api, &state.ws_tx);
    Ok(Json(response))
}

async fn paint_handler(
    State(state): State<ServerState>,
    payload: Result<Json<PaintRequest>, axum::extract::rejection::JsonRejection>,
) -> Result<Json<PaintResponse>, SimulationError> {
    let request = payload.map_err(SimulationError::from)?.0;
    let mut api = state.api.lock().await;
    let response = api.paint(request)?;
    publish_snapshot(&api, &state.ws_tx);
    Ok(Json(response))
}

async fn status_handler(
    State(state): State<ServerState>,
) -> Result<Json<crate::api::StatusResponse>, SimulationError> {
    let api = state.api.lock().await;
    Ok(Json(api.status()))
}

async fn frame_handler(
    State(state): State<ServerState>,
) -> Result<Json<crate::api::FrameResponse>, SimulationError> {
    let api = state.api.lock().await;
    Ok(Json(api.frame()))
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<ServerState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| ws_connection(socket, state))
}

async fn ws_connection(mut socket: WebSocket, state: ServerState) {
    {
        let api = state.api.lock().await;
        let initial = snapshot_events(&api);
        drop(api);

        for event in initial {
            if send_ws_event(&mut socket, &event).await.is_err() {
                return;
            }
        }
    }

    let mut rx = state.ws_tx.subscribe();
    loop {
        tokio::select! {
            inbound = socket.next() => {
                match inbound {
                    Some(Ok(message)) => {
                        if matches!(message, Message::Close(_)) {
                            return;
                        }
                    }
                    Some(Err(_)) | None => return,
                }
            }
            event = rx.recv() => {
                match event {
                    Ok(envelope) => {
                        if send_ws_event(&mut socket, &envelope).await.is_err() {
                            return;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => return,
                }
            }
        }
    }
}

async fn send_ws_event(socket: &mut WebSocket, envelope: &WsEventEnvelope) -> Result<(), ()> {
    let payload = serde_json::to_string(envelope).map_err(|_| ())?;
    socket
        .send(Message::Text(payload.into()))
        .await
        .map_err(|_| ())
}

fn publish_snapshot(api: &SimulationApi, ws_tx: &broadcast::Sender<WsEventEnvelope>) {
    for event in snapshot_events(api) {
        let _ = ws_tx.send(event);
    }
}

fn snapshot_events(api: &SimulationApi) -> Vec<WsEventEnvelope> {
    let status = api.status();
    let frame = api.frame();
    let health = api.health();
    ws_events_for_tick(
        &status,
        &frame,
        Some(&HealthPayload {
            population: health.population,
            genome_node_count_p50: health.genome_node_count_p50,
            genome_node_count_p90: health.genome_node_count_p90,
            mean_energy: health.mean_energy,
        }),
    )
}

impl IntoResponse for SimulationError {
    fn into_response(self) -> Response {
        let status =
            StatusCode::from_u16(self.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        (status, Json(self.into_envelope())).into_response()
    }
}

impl From<axum::extract::rejection::JsonRejection> for SimulationError {
    fn from(rejection: axum::extract::rejection::JsonRejection) -> Self {
        SimulationError::protocol(
            ProtocolErrorCode::InvalidRequest,
            format!("invalid request payload: {rejection}"),
            ErrorDetails {
                endpoint: None,
                field_errors: Vec::new(),
                expected_state: None,
                current_state: None,
            },
        )
    }
}
