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
    let food = patch.get("world").and_then(|w| w.get("food"));
    let direct_layers = food
        .and_then(|f| f.get("fertility"))
        .and_then(|fertility| fertility.get("layers"));
    let shared_layers = food
        .and_then(|f| f.get("shared"))
        .and_then(|shared| shared.get("fertility"))
        .and_then(|fertility| fertility.get("layers"));
    direct_layers.is_some() || shared_layers.is_some()
}

fn patch_touches_disallowed_food_runtime_paths(patch: &serde_json::Value) -> bool {
    let Some(food_patch) = patch.get("world").and_then(|w| w.get("food")) else {
        return false;
    };
    let Some(food_obj) = food_patch.as_object() else {
        return true;
    };
    food_obj.keys().any(|key| key != "shared")
}

fn patch_has_invalid_food_shared_shape(patch: &serde_json::Value) -> bool {
    patch
        .get("world")
        .and_then(|w| w.get("food"))
        .and_then(|food| food.get("shared"))
        .is_some_and(|shared| !shared.is_object())
}

fn patch_touches_world_topology(patch: &serde_json::Value) -> bool {
    patch.get("world").is_some_and(|w| {
        w.get("width").is_some() || w.get("height").is_some() || w.get("edge_mode").is_some()
    })
}

pub async fn get_status(State(app): State<AppState>) -> impl IntoResponse {
    let perf = app.perf.read().expect("perf lock poisoned").clone();
    let subscriber_count = app.ws_tx.receiver_count();
    let payload = {
        let projection = app.projection.read().expect("projection lock poisoned");
        status_payload(projection.current(), &perf, subscriber_count)
    };
    Json(payload)
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
    if patch_touches_disallowed_food_runtime_paths(&patch) {
        return Err(AppError::ValidationRejected {
            field_errors: vec![FieldError {
                field: "world.food".into(),
                reason: "runtime food patching is restricted to world.food.shared.*".into(),
            }],
            endpoint: "patch_config",
        });
    }
    if patch_has_invalid_food_shared_shape(&patch) {
        return Err(AppError::ValidationRejected {
            field_errors: vec![FieldError {
                field: "world.food.shared".into(),
                reason: "runtime food patch payload must be an object".into(),
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
    handle.cached_fertility_u8 =
        crate::query::cache::build_primary_food_fertility_u8(handle.sim.world.food());
    let state = handle.status;
    let frame = crate::state::build_ws_frame(&handle);
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
