use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::Json;
use slotmap::{Key, KeyData};
use v3_core::contracts::CreatureId;
use v3_core::mutation::phenotype::channels_to_rgb;
use v3_core::runtime::trace::ActiveTrace;

use crate::error::{AppError, FieldError};
use crate::state::{AppState, SimulationStatus};
use crate::types::PROTOCOL_VERSION;

/// Query parameters for `GET /v3/simulation/creature/:id`.
#[derive(Debug, Default, serde::Deserialize)]
pub struct CreatureQuery {
    /// When present, only return action_log entries with `tick > since_tick`.
    pub since_tick: Option<u64>,
    /// Comma-separated field names to exclude from the response
    /// (valid values: `genome`, `action_log`, `memory`).
    pub exclude: Option<String>,
}

pub async fn get_creature(
    State(app): State<AppState>,
    Path(id): Path<u64>,
    Query(query): Query<CreatureQuery>,
) -> Result<impl IntoResponse, AppError> {
    let handle = app.sim.lock().await;
    let sim = &handle.sim;

    let creature_id: CreatureId = KeyData::from_ffi(id).into();
    let creature = sim
        .creatures
        .get(creature_id)
        .ok_or_else(|| AppError::NotFound(format!("creature {id} not found")))?;

    let rgb = channels_to_rgb(creature.phenotype_channels);

    // Parse exclude set.
    let exclude: std::collections::HashSet<&str> = query
        .exclude
        .as_deref()
        .map(|s| s.split(',').map(str::trim).collect())
        .unwrap_or_default();

    // Build action_log with optional since_tick filtering.
    let action_log_entries: Vec<_> = if exclude.contains("action_log") {
        Vec::new()
    } else {
        sim.action_logs
            .get(creature_id)
            .map(|log| {
                if let Some(since) = query.since_tick {
                    log.entries().iter().filter(|e| e.tick > since).collect()
                } else {
                    log.entries().iter().collect()
                }
            })
            .unwrap_or_default()
    };

    // Compute latest_tick from all entries (unfiltered) for cursor tracking.
    let latest_tick: u64 = sim
        .action_logs
        .get(creature_id)
        .and_then(|log| log.entries().back().map(|e| e.tick))
        .unwrap_or(0);

    // Build response with conditional field inclusion.
    let mut map = serde_json::Map::new();
    map.insert(
        "protocol_version".into(),
        serde_json::Value::String(PROTOCOL_VERSION.into()),
    );
    map.insert("id".into(), serde_json::json!(id));
    map.insert(
        "position".into(),
        serde_json::json!({ "x": creature.position.x, "y": creature.position.y }),
    );
    map.insert("energy".into(), serde_json::json!(creature.energy));
    map.insert(
        "max_energy".into(),
        serde_json::json!(sim.config.energy.lifecycle.initial_energy),
    );
    map.insert("age".into(), serde_json::json!(creature.age));
    map.insert("generation".into(), serde_json::json!(creature.generation));
    map.insert(
        "complexity".into(),
        serde_json::json!(creature.genome.complexity()),
    );
    map.insert(
        "phenotype".into(),
        serde_json::json!({
            "channels": creature.phenotype_channels,
            "active_channel": creature.phenotype_active_channel,
            "polarity": creature.phenotype_channel_polarity,
            "rgb": rgb,
        }),
    );
    map.insert("latest_tick".into(), serde_json::json!(latest_tick));

    if !exclude.contains("genome") {
        map.insert("genome".into(), serde_json::json!(creature.genome));
    }
    if !exclude.contains("memory") {
        let memory: &[u8] = &creature.memory;
        map.insert("memory".into(), serde_json::json!(memory));
    }
    if !exclude.contains("action_log") {
        map.insert("action_log".into(), serde_json::json!(action_log_entries));
    }

    Ok(Json(serde_json::Value::Object(map)))
}

// ── Execution Sampler endpoints ─────────────────────────────────────────────

#[derive(serde::Deserialize)]
struct StartSampleRequest {
    #[serde(default = "default_sample_ticks")]
    ticks: u32,
    #[serde(default)]
    include_perception_debug: bool,
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
        StartSampleRequest {
            ticks: 5,
            include_perception_debug: false,
        }
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
    let mut active = ActiveTrace::new(creature_id, req.ticks);
    active.include_perception_debug = req.include_perception_debug;
    handle.active_trace = Some(active);

    Ok(Json(serde_json::json!({
        "protocol_version": PROTOCOL_VERSION,
        "status": "recording",
        "ticks_requested": req.ticks,
        "include_perception_debug": req.include_perception_debug,
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
