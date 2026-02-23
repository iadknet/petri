use axum::extract::ws::{Message, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use tokio::sync::broadcast;

use crate::state::AppState;
use crate::types::PROTOCOL_VERSION;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(app): State<AppState>,
) -> impl axum::response::IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, app))
}

async fn handle_socket(mut socket: WebSocket, app: AppState) {
    let mut rx = app.ws_tx.subscribe();
    loop {
        match rx.recv().await {
            Ok(event) => {
                let payloads = [
                    ("status", event.status_payload),
                    ("frame", event.frame_payload),
                    ("health", event.health_payload),
                ];
                for (evt_name, payload) in payloads {
                    let msg = serde_json::to_string(&serde_json::json!({
                        "protocol_version": PROTOCOL_VERSION,
                        "event": evt_name,
                        "tick": event.tick,
                        "payload": payload,
                    }))
                    .unwrap_or_default();
                    if socket.send(Message::Text(msg)).await.is_err() {
                        return;
                    }
                }
            }
            Err(broadcast::error::RecvError::Closed) => break,
            Err(broadcast::error::RecvError::Lagged(_)) => continue,
        }
    }
}
