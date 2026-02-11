use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

use petri_core::WorldConfig;

use crate::AppState;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigPatch {
    pub paused: Option<bool>,
    pub ticks_per_second: Option<u32>,
}

pub fn build_router(_state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/config", get(get_config).patch(patch_config))
        .route("/ws", get(ws_handler))
        .with_state(_state)
}

async fn health() -> &'static str {
    "ok"
}

async fn get_config(State(_state): State<AppState>) -> Json<WorldConfig> {
    let cfg = {
        let world = _state.world.read().await;
        world.config.clone()
    };
    Json(cfg)
}

async fn patch_config(
    State(_state): State<AppState>,
    Json(patch): Json<ConfigPatch>,
) -> Json<WorldConfig> {
    let cfg = {
        let mut world = _state.world.write().await;
        if let Some(paused) = patch.paused {
            world.config.paused = paused;
        }
        if let Some(tps) = patch.ticks_per_second {
            world.config.ticks_per_second = tps.max(1);
        }
        world.config.clone()
    };
    Json(cfg)
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
