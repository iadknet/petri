use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::Json;
use slotmap::{Key, KeyData};
use v3_core::contracts::CreatureId;
use v3_core::mutation::phenotype::channels_to_rgb;
use v3_core::runtime::trace::ActiveTrace;

use crate::error::{AppError, FieldError};
use crate::state::{AppState, SimulationStatus};
use crate::types::PROTOCOL_VERSION;

pub async fn get_creature(
    State(app): State<AppState>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    let handle = app.sim.lock().await;
    let sim = &handle.sim;

    let creature_id: CreatureId = KeyData::from_ffi(id).into();
    let creature = sim
        .creatures
        .get(creature_id)
        .ok_or_else(|| AppError::NotFound(format!("creature {id} not found")))?;

    let rgb = channels_to_rgb(creature.phenotype_channels);
    let memory: &[u8] = &creature.memory;

    Ok(Json(serde_json::json!({
        "protocol_version": PROTOCOL_VERSION,
        "id": id,
        "position": { "x": creature.position.x, "y": creature.position.y },
        "energy": creature.energy,
        "max_energy": sim.config.energy.lifecycle.initial_energy,
        "age": creature.age,
        "generation": creature.generation,
        "complexity": creature.genome.complexity(),
        "phenotype": {
            "channels": creature.phenotype_channels,
            "active_channel": creature.phenotype_active_channel,
            "polarity": creature.phenotype_channel_polarity,
            "rgb": rgb,
        },
        "genome": creature.genome,
        "memory": memory,
    })))
}

// ── Execution Sampler endpoints ─────────────────────────────────────────────

#[derive(serde::Deserialize)]
struct StartSampleRequest {
    #[serde(default = "default_sample_ticks")]
    ticks: u32,
}

fn default_sample_ticks() -> u32 {
    5
}

/// POST /v3/simulation/creature/:id/sample — start recording execution trace.
pub async fn start_sample(
    State(app): State<AppState>,
    Path(id): Path<u64>,
    body: axum::body::Bytes,
) -> Result<impl IntoResponse, AppError> {
    let req: StartSampleRequest = if body.is_empty() {
        StartSampleRequest { ticks: 5 }
    } else {
        serde_json::from_slice(&body)
            .map_err(|e| AppError::InvalidRequest(format!("invalid JSON: {e}")))?
    };

    // Validate ticks range [1, 10].
    if req.ticks < 1 || req.ticks > 10 {
        return Err(AppError::ValidationRejected {
            field_errors: vec![FieldError {
                field: "ticks".into(),
                reason: "ticks must be between 1 and 10".into(),
            }],
            endpoint: "start_sample",
        });
    }

    let mut handle = app.sim.lock().await;

    // Require non-idle state.
    if handle.status == SimulationStatus::Idle {
        return Err(AppError::InvalidStateTransition {
            expected: Some("running or paused".into()),
            current: "idle".into(),
        });
    }

    // Validate creature exists.
    let creature_id: CreatureId = KeyData::from_ffi(id).into();
    if !handle.sim.creatures.contains_key(creature_id) {
        return Err(AppError::NotFound(format!("creature {id} not found")));
    }

    // Start recording (replaces any existing trace).
    handle.active_trace = Some(ActiveTrace::new(creature_id, req.ticks));

    Ok(Json(serde_json::json!({
        "protocol_version": PROTOCOL_VERSION,
        "status": "recording",
        "ticks_requested": req.ticks,
    })))
}

/// GET /v3/simulation/creature/:id/sample — poll for trace results.
pub async fn get_sample(
    State(app): State<AppState>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    let mut handle = app.sim.lock().await;

    let creature_id: CreatureId = KeyData::from_ffi(id).into();

    // Check if there's an active trace for this creature.
    let Some(active) = handle
        .active_trace
        .as_ref()
        .filter(|t| t.creature_id == creature_id)
    else {
        return Ok(Json(serde_json::json!({
            "protocol_version": PROTOCOL_VERSION,
            "status": "idle",
        })));
    };

    if active.is_complete() {
        // Take the completed trace and convert to sample.
        let ffi_id = active.creature_id.data().as_ffi();
        let trace = handle
            .active_trace
            .take()
            .expect("active_trace confirmed Some above");
        let sample = trace.into_sample(ffi_id);

        Ok(Json(serde_json::json!({
            "protocol_version": PROTOCOL_VERSION,
            "status": "complete",
            "sample": sample,
        })))
    } else {
        let ticks_completed = active.ticks.len();
        let ticks_remaining = active.ticks_remaining;

        Ok(Json(serde_json::json!({
            "protocol_version": PROTOCOL_VERSION,
            "status": "recording",
            "ticks_completed": ticks_completed,
            "ticks_remaining": ticks_remaining,
        })))
    }
}
