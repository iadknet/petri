use std::time::{Duration, Instant};

use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use v3_core::config::SimulationConfig;
use v3_core::simulation::{run_tick, seed_simulation};

use crate::error::{AppError, FieldError};
use crate::state::{build_ws_frame, AppState, SimHandle, SimulationStatus};
use crate::types::{config_digest, deep_merge, StepRequest, PROTOCOL_VERSION};

const FRAME_INTERVAL: Duration = Duration::from_millis(100);

fn validate_startup_ramps(config: &SimulationConfig) -> Result<(), AppError> {
    let ramp = &config.startup.ramps.failed_action_penalty;
    if ramp.target_tick < 1 {
        return Err(AppError::ValidationRejected {
            field_errors: vec![FieldError {
                field: "startup.ramps.failed_action_penalty.target_tick".into(),
                reason: "must be >= 1".into(),
            }],
            endpoint: "startup",
        });
    }
    if !ramp.start.is_finite() || ramp.start < 0.0 {
        return Err(AppError::ValidationRejected {
            field_errors: vec![FieldError {
                field: "startup.ramps.failed_action_penalty.start".into(),
                reason: "must be finite and >= 0.0".into(),
            }],
            endpoint: "startup",
        });
    }
    if !ramp.end.is_finite() || ramp.end < 0.0 {
        return Err(AppError::ValidationRejected {
            field_errors: vec![FieldError {
                field: "startup.ramps.failed_action_penalty.end".into(),
                reason: "must be finite and >= 0.0".into(),
            }],
            endpoint: "startup",
        });
    }
    Ok(())
}

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

    // Merge the request over this server instance's startup baseline. Production
    // state uses SimulationConfig::default(); tests can use a smaller baseline.
    let mut base = serde_json::to_value(app.startup_defaults.as_ref())
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
    validate_startup_ramps(&config)?;
    config.normalize();
    config.apply_startup_overrides();

    // Re-seed the simulation.
    let new_sim = seed_simulation(config.clone(), seed);
    let seeded_creatures = new_sim.creatures.len();
    let digest = config_digest(&config);
    // Fertility is static after seeding; compute once and cache in the handle.
    let cached_fertility_u8 =
        crate::query::cache::build_primary_food_fertility_u8(new_sim.world.food());

    let mut handle = app.sim.lock().await;
    handle.status = SimulationStatus::Idle;
    handle.sim = new_sim;
    handle.active_trace = None;
    handle.cached_fertility_u8 = cached_fertility_u8;
    let frame = build_ws_frame(&handle);
    drop(handle);
    app.publish_startup_ws_frame(frame);

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
        let frame = build_ws_frame(&handle);
        let tick = handle.sim.tick;
        drop(handle);
        app.publish_ws_frame(frame);
        tick
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
    let frame = build_ws_frame(&handle);
    let tick = handle.sim.tick;
    drop(handle);
    app.publish_ws_frame(frame);
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

    {
        let SimHandle {
            sim, active_trace, ..
        } = &mut *handle;
        for _ in 0..req.steps {
            run_tick(sim, active_trace);
        }
    }
    let frame = build_ws_frame(&handle);
    let tick = handle.sim.tick;
    drop(handle);
    app.publish_ws_frame(frame);

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
        let h = &mut *handle;
        run_tick(&mut h.sim, &mut h.active_trace);
        let frame = if last_frame.elapsed() >= FRAME_INTERVAL {
            // Projection refresh may be slightly stale while the simulation runs,
            // avoiding a full frame rebuild on every tick.
            Some(build_ws_frame(&handle))
        } else {
            None
        };
        drop(handle);
        if let Some(frame) = frame {
            app.publish_ws_frame(frame);
            last_frame = Instant::now();
        }
        tokio::task::yield_now().await;
    }
}

#[cfg(test)]
mod tests {
    use v3_core::config::{FoodTypeConfig, OrdinaryFoodTypeId, SimulationConfig};
    use v3_core::contracts::Position;
    use v3_core::simulation::seed_simulation;

    use crate::state::{build_ws_frame, SimHandle, SimulationStatus};

    #[test]
    fn build_ws_frame_serializes_overlapping_food_types_per_cell() {
        let mut config = SimulationConfig::default();
        config.world.width = 4;
        config.world.height = 4;
        config.population.initial_creatures = 0;
        config.world.food.types = vec![
            FoodTypeConfig {
                name: "Primary Food".to_string(),
                color: "#22c55e".to_string(),
                initial_density: 0.0,
                initial_coverage: 0.0,
                growth_inhibitor: 0.2,
            },
            FoodTypeConfig {
                name: "Secondary Food".to_string(),
                color: "#0ea5e9".to_string(),
                initial_density: 0.0,
                initial_coverage: 0.0,
                growth_inhibitor: 0.2,
            },
        ];

        let mut sim = seed_simulation(config, 7);
        let pos = Position::new(1, 1);
        sim.world
            .set_food_type(pos, OrdinaryFoodTypeId::new(0), 0.75);
        sim.world
            .set_food_type(pos, OrdinaryFoodTypeId::new(1), 0.5);

        let handle = SimHandle {
            sim,
            status: SimulationStatus::Paused,
            active_trace: None,
            cached_fertility_u8: std::sync::Arc::from(vec![0u8; 16]),
        };

        let frame = build_ws_frame(&handle);

        assert_eq!(frame.frame.food.len(), 2);
        assert!(frame
            .frame
            .food
            .iter()
            .any(|cell| cell.x == 1 && cell.y == 1 && cell.type_idx == 0 && cell.density == 0.75));
        assert!(frame
            .frame
            .food
            .iter()
            .any(|cell| cell.x == 1 && cell.y == 1 && cell.type_idx == 1 && cell.density == 0.5));
    }
}
