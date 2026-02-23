use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use slotmap::Key;
use v3_core::config::SimulationConfig;

use crate::error::{AppError, FieldError};
use crate::state::{AppState, SimulationStatus};
use crate::types::{deep_merge, PROTOCOL_VERSION};

pub async fn get_status(State(app): State<AppState>) -> impl IntoResponse {
    let handle = app.sim.lock().await;
    let sim = &handle.sim;
    let stats = &sim.stats;

    Json(serde_json::json!({
        "protocol_version": PROTOCOL_VERSION,
        "state": handle.status,
        "tick": sim.tick,
        "population": sim.creatures.len(),
        "mean_energy": sim.mean_energy(),
        "last_tick_actions": {
            "move": stats.last_tick_move,
            "eat": stats.last_tick_eat,
            "reproduce": stats.last_tick_reproduce,
            "noop": stats.last_tick_noop,
        },
        "reproduction_actions_attempted_total": stats.reproduction_actions_attempted_total,
        "reproduction_actions_spawned_total": stats.reproduction_actions_spawned_total,
        "reproduction_actions_rejected_total": stats.reproduction_actions_rejected_total,
        "mutation_events_attempted_total": stats.mutation_events_attempted_total,
        "mutation_events_applied_total": stats.mutation_events_applied_total,
        "mutation_events_skipped_total": stats.mutation_events_skipped_total,
    }))
}

pub async fn get_frame(State(app): State<AppState>) -> impl IntoResponse {
    let handle = app.sim.lock().await;
    let sim = &handle.sim;

    let mut creatures = Vec::new();
    for (id, creature) in &sim.creatures {
        let numeric_id = id.data().as_ffi();
        creatures.push(serde_json::json!({
            "id": numeric_id,
            "x": creature.position.x,
            "y": creature.position.y,
            "energy": creature.energy,
            "generation": creature.generation,
            "phenotype_rgb": creature.phenotype_rgb,
        }));
    }

    let mut food_cells = Vec::new();
    let mut barrier_cells = Vec::new();
    for y in 0..sim.world.height {
        for x in 0..sim.world.width {
            let pos = v3_core::contracts::Position::new(x, y);
            let density = sim.world.food_at(pos);
            if density > 0 {
                food_cells.push(serde_json::json!({"x": x, "y": y, "density": density}));
            }
            if sim.world.is_barrier(pos) {
                barrier_cells.push(serde_json::json!({"x": x, "y": y}));
            }
        }
    }

    Json(serde_json::json!({
        "protocol_version": PROTOCOL_VERSION,
        "state": handle.status,
        "tick": sim.tick,
        "width": sim.world.width,
        "height": sim.world.height,
        "creatures": creatures,
        "food": food_cells,
        "barriers": barrier_cells,
    }))
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

    let mut handle = app.sim.lock().await;

    // Check if patch touches world-topology fields (require Idle state).
    let topology_touched = patch.get("world").is_some_and(|w| {
        w.get("width").is_some() || w.get("height").is_some() || w.get("edge_mode").is_some()
    });
    if topology_touched && handle.status != SimulationStatus::Idle {
        return Err(AppError::InvalidStateTransition {
            expected: Some("idle".into()),
            current: format!("{:?}", handle.status).to_lowercase(),
        });
    }

    // Merge patch into current config.
    let mut base = serde_json::to_value(&handle.sim.config)
        .map_err(|e| AppError::Internal(format!("config serialization error: {e}")))?;
    deep_merge(&mut base, patch);

    let mut merged_config: SimulationConfig =
        serde_json::from_value(base).map_err(|e| AppError::ValidationRejected {
            field_errors: vec![FieldError {
                field: "config".into(),
                reason: e.to_string(),
            }],
            endpoint: "patch_config",
        })?;
    merged_config.normalize();

    handle.sim.config = merged_config.clone();

    Ok(Json(serde_json::json!({
        "protocol_version": PROTOCOL_VERSION,
        "state": handle.status,
        "config": merged_config,
    })))
}
