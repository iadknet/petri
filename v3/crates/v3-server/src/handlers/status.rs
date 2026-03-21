use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use v3_core::config::SimulationConfig;

use crate::error::{AppError, FieldError};
use crate::state::TransportPerfSnapshot;
use crate::transport::protocol::build_status_event_payload;
use crate::types::{deep_merge, PROTOCOL_VERSION};
use crate::{app_state::AppState, query::projection::ProjectionSnapshot};

fn patch_touches_startup(patch: &serde_json::Value) -> bool {
    patch.get("startup").is_some()
}

fn patch_touches_failed_action_penalty(patch: &serde_json::Value) -> bool {
    patch
        .get("energy")
        .and_then(|e| e.get("costs"))
        .and_then(|c| c.get("failed_action_penalty"))
        .is_some()
}

fn patch_touches_fertility_generation_layers(patch: &serde_json::Value) -> bool {
    patch
        .get("world")
        .and_then(|w| w.get("food"))
        .and_then(|f| f.get("fertility"))
        .and_then(|fertility| fertility.get("layers"))
        .is_some()
}

fn patch_touches_world_topology(patch: &serde_json::Value) -> bool {
    patch.get("world").is_some_and(|w| {
        w.get("width").is_some() || w.get("height").is_some() || w.get("edge_mode").is_some()
    })
}

pub async fn get_status(State(app): State<AppState>) -> impl IntoResponse {
    let snapshot = {
        app.projection
            .read()
            .expect("projection lock poisoned")
            .current()
            .clone()
    };
    let perf = app.perf.read().expect("perf lock poisoned").clone();
    Json(status_payload(&snapshot, &perf, app.ws_tx.receiver_count()))
}

pub async fn get_config(State(app): State<AppState>) -> impl IntoResponse {
    let handle = app.sim.lock().await;
    Json(serde_json::json!({
        "protocol_version": PROTOCOL_VERSION,
        "state": handle.status,
        "config": handle.sim.config,
    }))
}

pub async fn patch_config(
    State(app): State<AppState>,
    body: axum::body::Bytes,
) -> Result<impl IntoResponse, AppError> {
    let patch: serde_json::Value = serde_json::from_slice(&body)
        .map_err(|e| AppError::InvalidRequest(format!("invalid JSON: {e}")))?;

    if patch_touches_startup(&patch) {
        return Err(AppError::ValidationRejected {
            field_errors: vec![FieldError {
                field: "startup".into(),
                reason: "startup config is restart-only and cannot be patched".into(),
            }],
            endpoint: "patch_config",
        });
    }
    if patch_touches_fertility_generation_layers(&patch) {
        return Err(AppError::ValidationRejected {
            field_errors: vec![FieldError {
                field: "world.food.fertility.layers".into(),
                reason: "fertility layer generation is restart-only and cannot be patched".into(),
            }],
            endpoint: "patch_config",
        });
    }
    if patch_touches_world_topology(&patch) {
        return Err(AppError::ValidationRejected {
            field_errors: vec![FieldError {
                field: "world".into(),
                reason: "world topology is restart-only and cannot be patched".into(),
            }],
            endpoint: "patch_config",
        });
    }

    let mut handle = app.sim.lock().await;

    if patch_touches_failed_action_penalty(&patch)
        && handle
            .sim
            .config
            .failed_action_penalty_ramp_active(handle.sim.tick)
    {
        return Err(AppError::ValidationRejected {
            field_errors: vec![FieldError {
                field: "energy.costs.failed_action_penalty".into(),
                reason: "startup failed-action-penalty ramp is still active".into(),
            }],
            endpoint: "patch_config",
        });
    }

    // Merge patch into current config.
    let mut base = serde_json::to_value(&handle.sim.config)
        .map_err(|e| AppError::Internal(format!("config serialization error: {e}")))?;
    deep_merge(&mut base, patch);

    let merged_config: SimulationConfig =
        serde_json::from_value(base).map_err(|e| AppError::ValidationRejected {
            field_errors: vec![FieldError {
                field: "config".into(),
                reason: e.to_string(),
            }],
            endpoint: "patch_config",
        })?;
    let mut normalized_config = merged_config.clone();
    normalized_config.normalize();
    let merged_value = serde_json::to_value(&merged_config)
        .map_err(|e| AppError::Internal(format!("config serialization error: {e}")))?;
    let normalized_value = serde_json::to_value(&normalized_config)
        .map_err(|e| AppError::Internal(format!("config serialization error: {e}")))?;
    if normalized_value != merged_value {
        return Err(AppError::ValidationRejected {
            field_errors: vec![FieldError {
                field: "config".into(),
                reason: "runtime config patch contains values outside canonical constraints".into(),
            }],
            endpoint: "patch_config",
        });
    }

    handle.sim.config = normalized_config.clone();
    handle
        .sim
        .world
        .reconfigure_food(normalized_config.world.food.clone());
    let state = handle.status;
    let frame = crate::handlers::lifecycle::build_ws_frame(&handle);
    drop(handle);
    app.publish_ws_frame(frame);

    Ok(Json(serde_json::json!({
        "protocol_version": PROTOCOL_VERSION,
        "state": state,
        "config": normalized_config,
    })))
}

fn status_payload(
    snapshot: &ProjectionSnapshot,
    perf: &TransportPerfSnapshot,
    subscriber_count: usize,
) -> serde_json::Value {
    let mut body =
        serde_json::to_value(build_status_event_payload(snapshot, perf, subscriber_count))
            .expect("status payload must serialize");
    let object = body
        .as_object_mut()
        .expect("status payload must serialize to an object");
    object.insert(
        "protocol_version".into(),
        serde_json::Value::String(PROTOCOL_VERSION.to_string()),
    );
    object.insert(
        "projection_revision".into(),
        serde_json::Value::from(snapshot.projection_revision),
    );
    object.insert(
        "world_static_revision".into(),
        serde_json::Value::from(snapshot.world_static_revision),
    );
    object.insert(
        "tick".into(),
        serde_json::Value::from(snapshot.ws_frame.tick),
    );
    body
}
