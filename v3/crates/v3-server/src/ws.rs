use axum::extract::ws::{Message, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use tokio::sync::broadcast;

use crate::state::AppState;

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
            Ok(bytes) => {
                if socket.send(Message::Binary(bytes)).await.is_err() {
                    return;
                }
            }
            Err(broadcast::error::RecvError::Closed) => break,
            Err(broadcast::error::RecvError::Lagged(_)) => continue,
        }
    }
}
