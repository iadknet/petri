use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use slotmap::Key;
use v3_core::config::SimulationConfig;
use v3_core::mutation::phenotype::channels_to_rgb;
use v3_core::simulation::{run_tick, seed_simulation};

use crate::error::{AppError, FieldError};
use crate::state::{
    AppState, BarrierCell, CreatureSnapshot, FoodCell, FramePayload, HealthPayload,
    LastTickActions, MutationOperatorFunnelPayload, MutationOperatorValueTotalsPayload,
    MutationTargetReachabilityTotalPayload, PredationEventSnapshot, SimHandle, SimulationStatus,
    StatusPayload, WsFrame,
};
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
    validate_startup_ramps(&config)?;
    config.normalize();
    config.apply_startup_overrides();

    // Re-seed the simulation.
    let new_sim = seed_simulation(config.clone(), seed);
    let seeded_creatures = new_sim.creatures.len();
    let digest = config_digest(&config);
    // Fertility is static after seeding; compute once and cache in the handle.
    let cached_fertility_u8 = crate::query::cache::build_food_fertility_u8(new_sim.world.food());

    let mut handle = app.sim.lock().await;
    handle.status = SimulationStatus::Idle;
    handle.sim = new_sim;
    handle.active_trace = None;
    handle.cached_fertility_u8 = cached_fertility_u8;
    let frame = build_ws_frame(&handle);
    drop(handle);
    app.publish_ws_frame(frame);

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
            // Transitional projection refresh is allowed to be slightly stale while
            // the sim is running, but it should not force a full frame rebuild on
            // every tick.
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
            steal: stats.last_tick_steal,
            predation_kills: stats.last_tick_predation_kills,
        },
        reproduction_actions_attempted_total: stats.reproduction_actions_attempted_total,
        reproduction_actions_spawned_total: stats.reproduction_actions_spawned_total,
        reproduction_actions_rejected_total: stats.reproduction_actions_rejected_total,
        predation_actions_attempted_total: stats.predation_actions_attempted_total,
        predation_actions_transferred_total: stats.predation_actions_transferred_total,
        predation_actions_rejected_total: stats.predation_actions_rejected_total,
        predation_kills_total: stats.predation_kills_total,
        last_tick_compute_total_mean: stats.last_tick_compute_total_mean,
        last_tick_compute_total_min: stats.last_tick_compute_total_min,
        last_tick_compute_total_max: stats.last_tick_compute_total_max,
        last_tick_compute_vm_mean: stats.last_tick_compute_vm_mean,
        last_tick_compute_graph_mean: stats.last_tick_compute_graph_mean,
        last_tick_food_occupancy_depletion_mean: stats.last_tick_food_occupancy_depletion_mean,
        last_tick_food_occupancy_depletion_occupied_cells: stats
            .last_tick_food_occupancy_depletion_occupied_cells,
        last_tick_food_growth_suppressed_by_occupancy_depletion: stats
            .last_tick_food_growth_suppressed_by_occupancy_depletion,
    };

    let mut creatures = Vec::with_capacity(sim.creatures.len());
    let mut complexity_sum: u64 = 0;
    let mut complexity_min: u32 = u32::MAX;
    let mut complexity_max: u32 = 0;
    let mut vm_live_read_world_inputs_current = std::collections::HashMap::new();
    for (id, creature) in &sim.creatures {
        let c = creature.cached_complexity;
        complexity_sum += c as u64;
        complexity_min = complexity_min.min(c);
        complexity_max = complexity_max.max(c);
        for (key, count) in creature.cached_live_vm_world_inputs.iter().copied() {
            *vm_live_read_world_inputs_current.entry(key).or_insert(0u64) += u64::from(count);
        }
        creatures.push(CreatureSnapshot {
            id: id.data().as_ffi(),
            x: creature.position.x,
            y: creature.position.y,
            energy: creature.energy,
            generation: creature.generation,
            phenotype_rgb: channels_to_rgb(creature.phenotype_channels),
        });
    }
    let creature_count = creatures.len();
    let complexity_mean = if creature_count > 0 {
        complexity_sum as f32 / creature_count as f32
    } else {
        0.0
    };
    if creature_count == 0 {
        complexity_min = 0;
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

    let food_fertility_u8 = Arc::clone(&handle.cached_fertility_u8);

    let frame = FramePayload {
        width: sim.world.width,
        height: sim.world.height,
        creatures,
        food,
        barriers,
        food_fertility_u8,
    };

    let map_mutation_value_totals = |totals: &v3_core::simulation::stats::MutationValueTotals| {
        MutationOperatorValueTotalsPayload {
            carriers_observed_total: totals.carriers_observed_total,
            survival_ticks_sum: totals.survival_ticks_sum,
            offspring_spawned_sum: totals.offspring_spawned_sum,
            final_energy_sum: totals.final_energy_sum,
            helpful_total: totals.helpful_total,
            neutral_total: totals.neutral_total,
            detrimental_total: totals.detrimental_total,
            confidence_low_total: totals.confidence_low_total,
            confidence_medium_total: totals.confidence_medium_total,
            confidence_high_total: totals.confidence_high_total,
            viability_score_sum: totals.viability_score_sum,
            viability_score_delta_sum: totals.viability_score_delta_sum,
            survived_short_horizon_total: totals.survived_short_horizon_total,
            survived_long_horizon_total: totals.survived_long_horizon_total,
            reproduced_once_total: totals.reproduced_once_total,
            mean_lifetime_energy_sum: totals.mean_lifetime_energy_sum,
            action_attempted_total: totals.action_attempted_total,
            blocked_move_total: totals.blocked_move_total,
            invalid_reproduce_total: totals.invalid_reproduce_total,
            invalid_action_total: totals.invalid_action_total,
        }
    };

    let health = HealthPayload {
        population: sim.creatures.len(),
        mean_energy: sim.mean_energy(),
        last_tick_food_occupancy_depletion_mean: stats.last_tick_food_occupancy_depletion_mean,
        last_tick_food_occupancy_depletion_occupied_cells: stats
            .last_tick_food_occupancy_depletion_occupied_cells,
        last_tick_food_growth_suppressed_by_occupancy_depletion: stats
            .last_tick_food_growth_suppressed_by_occupancy_depletion,
        mutation_events_attempted_total: stats.mutation_events_attempted_total,
        mutation_events_applied_total: stats.mutation_events_applied_total,
        mutation_events_skipped_total: stats.mutation_events_skipped_total,
        mutation_events_attempted_total_by_domain: stats
            .mutation_events_attempted_total_by_domain
            .iter()
            .map(|(domain, count)| (domain.as_key().to_string(), *count))
            .collect(),
        mutation_events_applied_total_by_domain: stats
            .mutation_events_applied_total_by_domain
            .iter()
            .map(|(domain, count)| (domain.as_key().to_string(), *count))
            .collect(),
        mutation_events_attempted_total_by_operator: stats
            .mutation_events_attempted_total_by_operator
            .iter()
            .map(|(operator, count)| (operator.as_key().to_string(), *count))
            .collect(),
        mutation_events_applied_total_by_operator: stats
            .mutation_events_applied_total_by_operator
            .iter()
            .map(|(operator, count)| (operator.as_key().to_string(), *count))
            .collect(),
        mutation_events_skipped_total_by_operator: stats
            .mutation_events_skipped_total_by_operator
            .iter()
            .map(|(operator, count)| (operator.as_key().to_string(), *count))
            .collect(),
        mutation_operator_funnel_total_by_operator: stats
            .mutation_operator_funnel_total_by_operator
            .iter()
            .map(|(operator, funnel)| {
                (
                    operator.as_key().to_string(),
                    MutationOperatorFunnelPayload {
                        attempted: funnel.attempted,
                        applicable: funnel.applicable,
                        structurally_valid: funnel.structurally_valid,
                        applied: funnel.applied,
                        semantic_change: funnel.semantic_change,
                        skipped: funnel.skipped,
                    },
                )
            })
            .collect(),
        mutation_skip_reasons_total_by_operator: stats
            .mutation_skip_reasons_total_by_operator
            .iter()
            .map(|(operator, by_reason)| {
                (
                    operator.as_key().to_string(),
                    by_reason
                        .iter()
                        .map(|(reason, count)| (reason.as_key().to_string(), *count))
                        .collect(),
                )
            })
            .collect(),
        mutation_added_node_input_classes_total_by_operator: stats
            .mutation_added_node_input_classes_total_by_operator
            .iter()
            .map(|(operator, by_class)| {
                (
                    operator.as_key().to_string(),
                    by_class
                        .iter()
                        .map(|(class, count)| (class.as_key().to_string(), *count))
                        .collect(),
                )
            })
            .collect(),
        mutation_added_node_world_inputs_total_by_operator: stats
            .mutation_added_node_world_inputs_total_by_operator
            .iter()
            .map(|(operator, by_key)| {
                (
                    operator.as_key().to_string(),
                    by_key
                        .iter()
                        .map(|(key, count)| (key.as_key().to_string(), *count))
                        .collect(),
                )
            })
            .collect(),
        vm_live_read_world_inputs_current: vm_live_read_world_inputs_current
            .into_iter()
            .map(|(key, count)| (key.as_key().to_string(), count))
            .collect(),
        mutation_events_applied_total_semantic_noop: stats
            .mutation_events_applied_total_semantic_noop,
        mutation_events_applied_total_semantic_change: stats
            .mutation_events_applied_total_semantic_change,
        mutation_target_reachability_total: MutationTargetReachabilityTotalPayload {
            reachable: stats.mutation_reachable_target_total,
            unreachable: stats.mutation_unreachable_target_total,
            not_applicable: stats.mutation_not_applicable_target_total,
        },
        mutation_value_totals_by_operator: stats
            .mutation_value_totals_by_operator
            .iter()
            .map(|(operator, totals)| {
                (
                    operator.as_key().to_string(),
                    map_mutation_value_totals(totals),
                )
            })
            .collect(),
        mutation_outcome_summary: map_mutation_value_totals(&stats.mutation_outcome_summary),
        reproduction_actions_attempted_total: stats.reproduction_actions_attempted_total,
        reproduction_actions_spawned_total: stats.reproduction_actions_spawned_total,
        reproduction_actions_rejected_total: stats.reproduction_actions_rejected_total,
        reproduction_actions_rejected_total_by_reason: stats
            .reproduction_actions_rejected_by_reason
            .iter()
            .map(|(reason, count)| (reason.as_key().to_string(), *count))
            .collect(),
        reproduction_actions_rejected_invalid_target_total_by_cause: stats
            .reproduction_actions_rejected_invalid_target_total_by_cause
            .iter()
            .map(|(cause, count)| (cause.as_key().to_string(), *count))
            .collect(),
        reproduction_actions_rejected_invalid_target_avoidable_total_by_reader_state: stats
            .reproduction_actions_rejected_invalid_target_avoidable_total_by_reader_state
            .iter()
            .map(|(state, count)| (state.as_key().to_string(), *count))
            .collect(),
        mutation_events_skipped_total_by_reason: stats
            .mutation_events_skipped_by_reason
            .iter()
            .map(|(reason, count)| (reason.as_key().to_string(), *count))
            .collect(),
        move_actions_blocked_total_by_cause: stats
            .move_actions_blocked_total_by_cause
            .iter()
            .map(|(cause, count)| (cause.as_key().to_string(), *count))
            .collect(),
        move_actions_blocked_avoidable_total_by_reader_state: stats
            .move_actions_blocked_avoidable_total_by_reader_state
            .iter()
            .map(|(state, count)| (state.as_key().to_string(), *count))
            .collect(),
        move_attempts_with_barrier_neighbor_total_by_reader_state: stats
            .move_attempts_with_barrier_neighbor_total_by_reader_state
            .iter()
            .map(|(state, count)| (state.as_key().to_string(), *count))
            .collect(),
        move_blocked_barrier_with_barrier_neighbor_total_by_reader_state: stats
            .move_blocked_barrier_with_barrier_neighbor_total_by_reader_state
            .iter()
            .map(|(state, count)| (state.as_key().to_string(), *count))
            .collect(),
        reproduction_attempts_with_barrier_neighbor_total_by_reader_state: stats
            .reproduction_attempts_with_barrier_neighbor_total_by_reader_state
            .iter()
            .map(|(state, count)| (state.as_key().to_string(), *count))
            .collect(),
        reproduction_invalid_target_barrier_with_barrier_neighbor_total_by_reader_state: stats
            .reproduction_invalid_target_barrier_with_barrier_neighbor_total_by_reader_state
            .iter()
            .map(|(state, count)| (state.as_key().to_string(), *count))
            .collect(),
        predation_actions_attempted_total: stats.predation_actions_attempted_total,
        predation_actions_transferred_total: stats.predation_actions_transferred_total,
        predation_actions_rejected_total: stats.predation_actions_rejected_total,
        predation_kills_total: stats.predation_kills_total,
        predation_actions_by_result: stats
            .predation_actions_by_result
            .iter()
            .map(|(result, count)| (result.as_key().to_string(), *count))
            .collect(),
        genome_complexity_mean: complexity_mean,
        genome_complexity_min: complexity_min,
        genome_complexity_max: complexity_max,
    };

    let predation_events: Vec<PredationEventSnapshot> = stats
        .last_tick_predation_events
        .iter()
        .map(|e| PredationEventSnapshot {
            attacker_x: e.attacker_x,
            attacker_y: e.attacker_y,
            victim_x: e.victim_x,
            victim_y: e.victim_y,
            energy_stolen: e.energy_stolen,
            killed: e.killed,
        })
        .collect();

    WsFrame {
        tick: sim.tick,
        status,
        frame,
        health,
        predation_events,
    }
}
