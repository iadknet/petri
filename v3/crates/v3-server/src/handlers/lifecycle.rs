use std::time::{Duration, Instant};

use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use slotmap::Key;
use v3_core::config::SimulationConfig;
use v3_core::simulation::{run_tick, seed_simulation};

use crate::error::{AppError, FieldError};
use crate::state::{
    AppState, BarrierCell, CreatureSnapshot, FoodCell, FramePayload, HealthPayload,
    LastTickActions, SimHandle, SimulationStatus, StatusPayload, WsFrame,
};
use crate::types::{config_digest, deep_merge, StepRequest, PROTOCOL_VERSION};

const FRAME_INTERVAL: Duration = Duration::from_millis(100);

pub async fn startup(
    State(app): State<AppState>,
    body: axum::body::Bytes,
) -> Result<impl IntoResponse, AppError> {
    // Parse body as JSON Value.
    let mut patch: serde_json::Value = serde_json::from_slice(&body)
        .map_err(|e| AppError::InvalidRequest(format!("invalid JSON: {e}")))?;

    // Extract seed (required field).
    let seed = match patch.as_object_mut().and_then(|m| m.remove("seed")) {
        Some(serde_json::Value::Number(n)) => n
            .as_u64()
            .ok_or_else(|| AppError::InvalidRequest("seed must be an unsigned integer".into()))?,
        Some(_) => return Err(AppError::InvalidRequest("seed must be a number".into())),
        None => {
            return Err(AppError::InvalidRequest(
                "missing required field: seed".into(),
            ))
        }
    };

    // Merge patch into default config value.
    let mut base = serde_json::to_value(SimulationConfig::default())
        .map_err(|e| AppError::Internal(format!("config serialization error: {e}")))?;
    deep_merge(&mut base, patch);

    // Deserialize merged value — deny_unknown_fields handles validation.
    let mut config: SimulationConfig =
        serde_json::from_value(base).map_err(|e| AppError::ValidationRejected {
            field_errors: vec![FieldError {
                field: "config".into(),
                reason: e.to_string(),
            }],
            endpoint: "startup",
        })?;
    config.normalize();

    // Re-seed the simulation.
    let new_sim = seed_simulation(config.clone(), seed);
    let seeded_creatures = new_sim.creatures.len();
    let digest = config_digest(&config);

    let mut handle = app.sim.lock().await;
    handle.status = SimulationStatus::Idle;
    handle.sim = new_sim;
    drop(handle);

    Ok(Json(serde_json::json!({
        "protocol_version": PROTOCOL_VERSION,
        "state": "idle",
        "tick": 0,
        "config_digest": digest,
        "seeded_creatures": seeded_creatures,
    })))
}

pub async fn start(State(app): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let tick = {
        let mut handle = app.sim.lock().await;
        if handle.status == SimulationStatus::Running {
            // Idempotent return.
            let tick = handle.sim.tick;
            return Ok(Json(serde_json::json!({
                "protocol_version": PROTOCOL_VERSION,
                "state": "running",
                "tick": tick,
            })));
        }
        handle.status = SimulationStatus::Running;
        handle.sim.tick
    };

    // Spawn run loop.
    tokio::spawn(run_loop(app));

    Ok(Json(serde_json::json!({
        "protocol_version": PROTOCOL_VERSION,
        "state": "running",
        "tick": tick,
    })))
}

pub async fn pause_sim(State(app): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let mut handle = app.sim.lock().await;
    if handle.status == SimulationStatus::Idle {
        return Err(AppError::InvalidStateTransition {
            expected: None,
            current: "idle".into(),
        });
    }
    handle.status = SimulationStatus::Paused;
    let tick = handle.sim.tick;
    Ok(Json(serde_json::json!({
        "protocol_version": PROTOCOL_VERSION,
        "state": "paused",
        "tick": tick,
    })))
}

pub async fn step(
    State(app): State<AppState>,
    body: axum::body::Bytes,
) -> Result<impl IntoResponse, AppError> {
    let req: StepRequest = if body.is_empty() {
        StepRequest::default()
    } else {
        serde_json::from_slice(&body)
            .map_err(|e| AppError::InvalidRequest(format!("invalid JSON: {e}")))?
    };

    if req.steps < 1 || req.steps > 1000 {
        return Err(AppError::ValidationRejected {
            field_errors: vec![FieldError {
                field: "steps".into(),
                reason: "steps must be between 1 and 1000".into(),
            }],
            endpoint: "step",
        });
    }

    let mut handle = app.sim.lock().await;
    if handle.status != SimulationStatus::Paused {
        return Err(AppError::InvalidStateTransition {
            expected: Some("paused".into()),
            current: format!("{:?}", handle.status).to_lowercase(),
        });
    }

    for _ in 0..req.steps {
        run_tick(&mut handle.sim);
    }
    let tick = handle.sim.tick;

    Ok(Json(serde_json::json!({
        "protocol_version": PROTOCOL_VERSION,
        "state": "paused",
        "tick": tick,
        "steps_applied": req.steps,
    })))
}

pub(crate) async fn run_loop(app: AppState) {
    let mut last_frame = Instant::now() - FRAME_INTERVAL;
    loop {
        let mut handle = app.sim.lock().await;
        if handle.status != SimulationStatus::Running {
            break;
        }
        run_tick(&mut handle.sim);
        if last_frame.elapsed() >= FRAME_INTERVAL {
            let frame = build_ws_frame(&handle);
            let bytes = rmp_serde::to_vec_named(&frame).unwrap_or_default();
            let _ = app.ws_tx.send(bytes);
            last_frame = Instant::now();
        }
        drop(handle);
        tokio::task::yield_now().await;
    }
}

pub fn build_ws_frame(handle: &SimHandle) -> WsFrame {
    let sim = &handle.sim;
    let stats = &sim.stats;

    let status = StatusPayload {
        state: handle.status,
        population: sim.creatures.len(),
        mean_energy: sim.mean_energy(),
        last_tick_actions: LastTickActions {
            move_count: stats.last_tick_move,
            eat: stats.last_tick_eat,
            reproduce: stats.last_tick_reproduce,
            noop: stats.last_tick_noop,
        },
        reproduction_actions_attempted_total: stats.reproduction_actions_attempted_total,
        reproduction_actions_spawned_total: stats.reproduction_actions_spawned_total,
        reproduction_actions_rejected_total: stats.reproduction_actions_rejected_total,
        last_tick_compute_total_mean: stats.last_tick_compute_total_mean,
        last_tick_compute_total_min: stats.last_tick_compute_total_min,
        last_tick_compute_total_max: stats.last_tick_compute_total_max,
        last_tick_compute_vm_mean: stats.last_tick_compute_vm_mean,
        last_tick_compute_graph_mean: stats.last_tick_compute_graph_mean,
    };

    let mut creatures = Vec::with_capacity(sim.creatures.len());
    for (id, creature) in &sim.creatures {
        creatures.push(CreatureSnapshot {
            id: id.data().as_ffi(),
            x: creature.position.x,
            y: creature.position.y,
            energy: creature.energy,
            generation: creature.generation,
            phenotype_rgb: creature.phenotype_rgb,
        });
    }

    let mut food = Vec::new();
    let mut barriers = Vec::new();
    for y in 0..sim.world.height {
        for x in 0..sim.world.width {
            let pos = v3_core::contracts::Position::new(x, y);
            let density = sim.world.food_at(pos);
            if density > 0.0 {
                food.push(FoodCell { x, y, density });
            }
            if sim.world.is_barrier(pos) {
                barriers.push(BarrierCell { x, y });
            }
        }
    }

    let frame = FramePayload {
        width: sim.world.width,
        height: sim.world.height,
        creatures,
        food,
        barriers,
    };

    let health = HealthPayload {
        population: sim.creatures.len(),
        mean_energy: sim.mean_energy(),
        mutation_events_attempted_total: stats.mutation_events_attempted_total,
        mutation_events_applied_total: stats.mutation_events_applied_total,
        mutation_events_skipped_total: stats.mutation_events_skipped_total,
        reproduction_actions_attempted_total: stats.reproduction_actions_attempted_total,
        reproduction_actions_spawned_total: stats.reproduction_actions_spawned_total,
        reproduction_actions_rejected_total: stats.reproduction_actions_rejected_total,
        reproduction_actions_rejected_total_by_reason: stats
            .reproduction_actions_rejected_by_reason
            .clone(),
        mutation_events_skipped_total_by_reason: stats.mutation_events_skipped_by_reason.clone(),
    };

    WsFrame {
        tick: sim.tick,
        status,
        frame,
        health,
    }
}
