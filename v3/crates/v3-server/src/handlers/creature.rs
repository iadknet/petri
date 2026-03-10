use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::Json;
use slotmap::{Key, KeyData};
use v3_core::contracts::CreatureId;
use v3_core::creature::action_log::ActionLogEntry;
use v3_core::creature::genome::mesh_annotations::{
    derive_mesh_annotations_with_reachable_indices, MeshNodeAnnotation,
};
use v3_core::mutation::phenotype::channels_to_rgb;
use v3_core::runtime::trace::{ActiveTrace, ExecutionSample};

use crate::error::{AppError, FieldError};
use crate::state::{AppState, SimulationStatus};
use crate::types::PROTOCOL_VERSION;

#[derive(Debug, Default, serde::Deserialize)]
pub struct CreatureQuery {
    pub since_tick: Option<u64>,
    pub exclude: Option<String>,
}

#[derive(serde::Serialize)]
struct CreaturePositionResponse {
    x: u16,
    y: u16,
}

#[derive(serde::Serialize)]
struct CreaturePhenotypeResponse {
    channels: [u8; 6],
    active_channel: usize,
    polarity: [bool; 6],
    rgb: [u8; 3],
}

#[derive(serde::Serialize)]
struct CreatureDetailResponse<'a> {
    protocol_version: &'static str,
    id: u64,
    position: CreaturePositionResponse,
    energy: f32,
    max_energy: f32,
    age: u64,
    generation: u64,
    complexity: u32,
    genome_size: u32,
    phenotype: CreaturePhenotypeResponse,
    latest_tick: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    genome: Option<&'a v3_core::creature::genome::CreatureGenome>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mesh_annotations: Option<Vec<MeshNodeAnnotation>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    shared_memory: Option<&'a [f32]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    action_log: Option<Vec<&'a ActionLogEntry>>,
}

#[derive(serde::Serialize)]
struct StartSampleResponse {
    protocol_version: &'static str,
    status: &'static str,
    ticks_requested: u32,
    include_perception_debug: bool,
}

#[derive(serde::Serialize)]
struct SampleIdleResponse {
    protocol_version: &'static str,
    status: &'static str,
}

#[derive(serde::Serialize)]
struct SampleRecordingResponse {
    protocol_version: &'static str,
    status: &'static str,
    ticks_completed: usize,
    ticks_remaining: u32,
}

#[derive(serde::Serialize)]
struct SampleCompleteResponse {
    protocol_version: &'static str,
    status: &'static str,
    sample: ExecutionSample,
}

fn to_json_value<T: serde::Serialize>(value: T) -> Result<Json<serde_json::Value>, AppError> {
    let body = serde_json::to_value(value)
        .map_err(|error| AppError::Internal(format!("failed to serialize response: {error}")))?;
    Ok(Json(body))
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
    let exclude: std::collections::HashSet<&str> = query
        .exclude
        .as_deref()
        .map(|s| s.split(',').map(str::trim).collect())
        .unwrap_or_default();

    let action_log_entries: Vec<_> = if exclude.contains("action_log") {
        Vec::new()
    } else {
        sim.action_logs
            .get(creature_id)
            .map(|log| {
                if let Some(since) = query.since_tick {
                    log.entries()
                        .iter()
                        .filter(|entry| entry.tick > since)
                        .collect()
                } else {
                    log.entries().iter().collect()
                }
            })
            .unwrap_or_default()
    };

    let latest_tick = sim
        .action_logs
        .get(creature_id)
        .and_then(|log| log.entries().back().map(|entry| entry.tick))
        .unwrap_or(0);

    let response = CreatureDetailResponse {
        protocol_version: PROTOCOL_VERSION,
        id,
        position: CreaturePositionResponse {
            x: creature.position.x,
            y: creature.position.y,
        },
        energy: creature.energy,
        max_energy: sim.config.energy.lifecycle.max_energy,
        age: creature.age,
        generation: creature.generation,
        complexity: creature.cached_complexity,
        genome_size: creature.genome.genome_size(),
        phenotype: CreaturePhenotypeResponse {
            channels: creature.phenotype_channels,
            active_channel: creature.phenotype_active_channel,
            polarity: creature.phenotype_channel_polarity,
            rgb,
        },
        latest_tick,
        genome: (!exclude.contains("genome")).then_some(&creature.genome),
        mesh_annotations: (!exclude.contains("genome")).then(|| {
            derive_mesh_annotations_with_reachable_indices(
                &creature.genome,
                &creature.cached_reachable_nodes,
            )
        }),
        shared_memory: (!exclude.contains("shared_memory")).then(|| &creature.shared_memory[..]),
        action_log: (!exclude.contains("action_log")).then_some(action_log_entries),
    };

    to_json_value(response)
}

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
            .map_err(|error| AppError::InvalidRequest(format!("invalid JSON: {error}")))?
    };

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
    if handle.status == SimulationStatus::Idle {
        return Err(AppError::InvalidStateTransition {
            expected: Some("running or paused".into()),
            current: "idle".into(),
        });
    }

    let creature_id: CreatureId = KeyData::from_ffi(id).into();
    if !handle.sim.creatures.contains_key(creature_id) {
        return Err(AppError::NotFound(format!("creature {id} not found")));
    }

    let mut active = ActiveTrace::new(creature_id, req.ticks);
    active.include_perception_debug = req.include_perception_debug;
    handle.active_trace = Some(active);

    to_json_value(StartSampleResponse {
        protocol_version: PROTOCOL_VERSION,
        status: "recording",
        ticks_requested: req.ticks,
        include_perception_debug: req.include_perception_debug,
    })
}

pub async fn get_sample(
    State(app): State<AppState>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    let mut handle = app.sim.lock().await;

    let creature_id: CreatureId = KeyData::from_ffi(id).into();
    let Some(active) = handle
        .active_trace
        .as_ref()
        .filter(|trace| trace.creature_id == creature_id)
    else {
        return to_json_value(SampleIdleResponse {
            protocol_version: PROTOCOL_VERSION,
            status: "idle",
        });
    };

    if active.is_complete() {
        let ffi_id = active.creature_id.data().as_ffi();
        let trace = handle
            .active_trace
            .take()
            .expect("active_trace confirmed Some above");
        let sample = trace.into_sample(ffi_id);

        return to_json_value(SampleCompleteResponse {
            protocol_version: PROTOCOL_VERSION,
            status: "complete",
            sample,
        });
    }

    to_json_value(SampleRecordingResponse {
        protocol_version: PROTOCOL_VERSION,
        status: "recording",
        ticks_completed: active.ticks.len(),
        ticks_remaining: active.ticks_remaining,
    })
}
