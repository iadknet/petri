use axum::extract::ws::{Message, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::broadcast;
use tracing::warn;

use crate::app_state::AppState;
use crate::query::cache::view_intersects_dirty_rect;
use crate::transport::protocol::{
    build_status_event_payload, build_world_static_payload, decode_client_message_binary,
    decode_client_message_text, encode_server_message, ServerMessage, PROTOCOL_VERSION,
};
use crate::transport::session::ProjectionNotice;
use crate::transport::session_registry::SessionId;
use crate::transport::view_assembler::{assemble_detail_payload, assemble_overview_payload};

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(app): State<AppState>,
) -> impl axum::response::IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, app))
}

async fn handle_socket(socket: WebSocket, app: AppState) {
    let session_id = {
        app.sessions
            .write()
            .expect("session registry lock poisoned")
            .insert()
    };
    let mut rx = app.ws_tx.subscribe();
    let (mut sender, mut receiver) = socket.split();

    loop {
        tokio::select! {
            biased;
            maybe_message = receiver.next() => {
                match maybe_message {
                    Some(Ok(message)) => {
                        match handle_client_message(&app, session_id, message) {
                            ClientMessageOutcome::Continue => {}
                            ClientMessageOutcome::Close => break,
                            ClientMessageOutcome::Send(batch) => {
                                for bytes in batch {
                                    if sender.send(Message::Binary(bytes)).await.is_err() {
                                        return;
                                    }
                                }
                            }
                        }
                    }
                    Some(Err(_)) | None => break,
                }
            }
            notice = rx.recv() => {
                match notice {
                    Ok(notice) => {
                        let Some(batch) = build_server_message_batch(&app, session_id, notice) else {
                            continue;
                        };

                        let mut failed = false;
                        for bytes in batch {
                            if sender.send(Message::Binary(bytes)).await.is_err() {
                                failed = true;
                                break;
                            }
                        }
                        if failed {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                }
            }
        }
    }

    let _ = app
        .sessions
        .write()
        .expect("session registry lock poisoned")
        .remove(session_id);
}

enum ClientMessageOutcome {
    Continue,
    Send(Vec<Vec<u8>>),
    Close,
}

fn handle_client_message(
    app: &AppState,
    session_id: SessionId,
    message: Message,
) -> ClientMessageOutcome {
    let decoded = match message {
        Message::Text(text) => decode_client_message_text(&text).ok(),
        Message::Binary(bytes) => decode_client_message_binary(&bytes).ok(),
        Message::Close(_) => return ClientMessageOutcome::Close,
        Message::Ping(_) | Message::Pong(_) => return ClientMessageOutcome::Continue,
    };

    let Some(decoded) = decoded else {
        return ClientMessageOutcome::Continue;
    };
    let should_prime = matches!(
        decoded,
        crate::transport::protocol::ClientMessage::SubscribeView { .. }
    );

    let _ = app
        .sessions
        .write()
        .expect("session registry lock poisoned")
        .apply_client_message(session_id, decoded);
    if should_prime {
        if let Some(batch) = build_server_message_batch(
            app,
            session_id,
            ProjectionNotice {
                projection_revision: 0,
                dirty_rect: None,
                world_static_changed: true,
            },
        ) {
            return ClientMessageOutcome::Send(batch);
        }
    }

    ClientMessageOutcome::Continue
}

fn build_server_message_batch(
    app: &AppState,
    session_id: SessionId,
    notice: ProjectionNotice,
) -> Option<Vec<Vec<u8>>> {
    let snapshot = {
        app.projection
            .read()
            .expect("projection lock poisoned")
            .current()
            .clone()
    };
    let perf = app.perf.read().expect("perf lock poisoned").clone();
    let delivery = {
        let mut sessions = app
            .sessions
            .write()
            .expect("session registry lock poisoned");
        let subscription = sessions.active_subscription(session_id)?;
        let view_needed = notice
            .dirty_rect
            .is_none_or(|dirty_rect| view_intersects_dirty_rect(&subscription, dirty_rect));
        if !view_needed && !notice.world_static_changed {
            return None;
        }
        if !view_needed {
            None
        } else {
            sessions.prepare_frame_delivery(
                session_id,
                ProjectionNotice {
                    projection_revision: snapshot
                        .projection_revision
                        .max(notice.projection_revision),
                    dirty_rect: notice.dirty_rect,
                    world_static_changed: notice.world_static_changed,
                },
            )
        }
    };
    let mut messages = Vec::new();
    if let Some(delivery) = delivery {
        messages.push(ServerMessage::Status {
            protocol_version: PROTOCOL_VERSION.to_string(),
            projection_revision: snapshot.projection_revision,
            world_static_revision: snapshot.world_static_revision,
            tick: snapshot.ws_frame.tick,
            payload: build_status_event_payload(&snapshot, &perf, app.ws_tx.receiver_count()),
        });
        messages.push(ServerMessage::Health {
            protocol_version: PROTOCOL_VERSION.to_string(),
            projection_revision: snapshot.projection_revision,
            world_static_revision: snapshot.world_static_revision,
            tick: snapshot.ws_frame.tick,
            payload: snapshot.ws_frame.health.clone(),
        });
        messages.push(match delivery.subscription.zoom_tier {
            crate::transport::protocol::ZoomTier::Overview => ServerMessage::ViewOverview {
                protocol_version: PROTOCOL_VERSION.to_string(),
                request_id: delivery.request_id,
                projection_revision: snapshot.projection_revision,
                world_static_revision: snapshot.world_static_revision,
                tick: snapshot.ws_frame.tick,
                payload: assemble_overview_payload(&snapshot, &delivery.subscription),
            },
            crate::transport::protocol::ZoomTier::Detail
            | crate::transport::protocol::ZoomTier::Inspect => ServerMessage::ViewDetail {
                protocol_version: PROTOCOL_VERSION.to_string(),
                request_id: delivery.request_id,
                projection_revision: snapshot.projection_revision,
                world_static_revision: snapshot.world_static_revision,
                tick: snapshot.ws_frame.tick,
                payload: assemble_detail_payload(&snapshot, &delivery.subscription),
            },
        });
    }
    if notice.world_static_changed || notice.projection_revision == 0 {
        messages.push(ServerMessage::WorldStatic {
            protocol_version: PROTOCOL_VERSION.to_string(),
            projection_revision: snapshot.projection_revision,
            world_static_revision: snapshot.world_static_revision,
            tick: snapshot.ws_frame.tick,
            payload: build_world_static_payload(&snapshot),
        });
    }
    if messages.is_empty() {
        return None;
    }

    let mut encoded = Vec::with_capacity(messages.len());
    for message in messages {
        match encode_server_message(&message) {
            Ok(bytes) => encoded.push(bytes),
            Err(error) => {
                warn!(
                    ?error,
                    session_id,
                    projection_revision = snapshot.projection_revision,
                    "failed to encode websocket message batch"
                );
                return None;
            }
        }
    }

    Some(encoded)
}
