mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::json;
use tower::ServiceExt;

use petri_core::WorldConfig;
use petri_server::{build_router, sim_loop::run_single_iteration, AppState, AppStateOptions};

#[tokio::test]
async fn simulation_starts_idle_and_requires_explicit_start() {
    let state = common::fast_state();
    let app = build_router(state.clone());

    let status_before = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/simulation/status")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");
    assert_eq!(status_before.status(), StatusCode::OK);
    let status_before_json = common::read_json(status_before).await;
    assert_eq!(status_before_json["phase"], "idle");
    assert!(status_before_json["run_id"].is_null());
    assert_eq!(status_before_json["startup_viable"], true);

    let (frame, _delay_ms) = run_single_iteration(&state).await;
    assert!(frame.is_none());

    let start_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/simulation/start")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");
    assert_eq!(start_response.status(), StatusCode::OK);
    let start_json = common::read_json(start_response).await;
    assert_eq!(start_json["phase"], "running");
    assert!(start_json["run_id"].as_u64().is_some());

    let (frame, _delay_ms) = run_single_iteration(&state).await;
    assert!(frame.is_some());
}

#[tokio::test]
async fn default_startup_draft_uses_lower_food_settings() {
    let state = common::fast_state();
    let app = build_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/simulation/status")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");
    assert_eq!(response.status(), StatusCode::OK);
    let status_json = common::read_json(response).await;

    assert_eq!(status_json["startup_draft"]["initial_food_density"], 0.15);
    assert_eq!(status_json["startup_draft"]["max_creatures"], 500_000);
    assert_eq!(status_json["startup_draft"]["food_spawn_rate"], 0.05);
    assert_eq!(status_json["startup_draft"]["food_growth_rate"], 0.10);
    assert_eq!(status_json["startup_draft"]["food_spread_threshold"], 0.75);
    assert_eq!(
        status_json["startup_draft"]["food_spawn_floor_density"],
        0.03
    );
    assert_eq!(status_json["startup_draft"]["sensor_radius"], 12);
    assert_eq!(status_json["startup_draft"]["width"], 400);
    assert_eq!(status_json["startup_draft"]["height"], 400);
}

#[tokio::test]
async fn patch_startup_draft_while_running_sets_pending_restart() {
    let state = common::fast_state();
    let app = build_router(state);

    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/simulation/start")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");

    let patch_payload = json!({
        "initial_creatures": 350
    });

    let patch_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/simulation/startup-draft")
                .header("content-type", "application/json")
                .body(Body::from(patch_payload.to_string()))
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");
    assert_eq!(patch_response.status(), StatusCode::OK);
    let patch_json = common::read_json(patch_response).await;
    assert_eq!(patch_json["initial_creatures"], 350);

    let status_response = app
        .oneshot(
            Request::builder()
                .uri("/simulation/status")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");
    let status_json = common::read_json(status_response).await;
    assert_eq!(status_json["pending_restart"], true);
}

#[tokio::test]
async fn startup_draft_patch_updates_world_wrap() {
    let state = common::fast_state();
    let app = build_router(state);

    let patch_payload = json!({
        "world_wrap": false
    });

    let patch_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/simulation/startup-draft")
                .header("content-type", "application/json")
                .body(Body::from(patch_payload.to_string()))
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");
    assert_eq!(patch_response.status(), StatusCode::OK);
    let patch_json = common::read_json(patch_response).await;
    assert_eq!(patch_json["world_wrap"], false);

    let status_response = app
        .oneshot(
            Request::builder()
                .uri("/simulation/status")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");
    assert_eq!(status_response.status(), StatusCode::OK);
    let status_json = common::read_json(status_response).await;
    assert_eq!(status_json["startup_draft"]["world_wrap"], false);
}

#[tokio::test]
async fn startup_draft_patch_updates_restart_required_stage1_knobs() {
    let state = common::fast_state();
    let app = build_router(state);

    let patch_payload = json!({
        "max_creatures": 7200,
        "energy_initial": 0.82,
        "food_spread_threshold": 0.62,
        "food_spawn_floor_density": 0.08,
        "sensor_radius": 18
    });

    let patch_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/simulation/startup-draft")
                .header("content-type", "application/json")
                .body(Body::from(patch_payload.to_string()))
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");
    assert_eq!(patch_response.status(), StatusCode::OK);
    let patch_json = common::read_json(patch_response).await;
    assert_eq!(patch_json["max_creatures"], 7200);
    assert_eq!(patch_json["energy_initial"], 0.82);
    assert_eq!(patch_json["food_spread_threshold"], 0.62);
    assert_eq!(patch_json["food_spawn_floor_density"], 0.08);
    assert_eq!(patch_json["sensor_radius"], 18);

    let status_response = app
        .oneshot(
            Request::builder()
                .uri("/simulation/status")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");
    assert_eq!(status_response.status(), StatusCode::OK);
    let status_json = common::read_json(status_response).await;
    assert_eq!(status_json["startup_draft"]["max_creatures"], 7200);
    assert_eq!(status_json["startup_draft"]["energy_initial"], 0.82);
    assert_eq!(status_json["startup_draft"]["food_spread_threshold"], 0.62);
    assert_eq!(
        status_json["startup_draft"]["food_spawn_floor_density"],
        0.08
    );
    assert_eq!(status_json["startup_draft"]["sensor_radius"], 18);
}

#[tokio::test]
async fn startup_draft_patch_updates_world_dimensions() {
    let state = common::fast_state();
    let app = build_router(state);

    let patch_payload = json!({
        "width": 420,
        "height": 360
    });

    let patch_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/simulation/startup-draft")
                .header("content-type", "application/json")
                .body(Body::from(patch_payload.to_string()))
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");
    assert_eq!(patch_response.status(), StatusCode::OK);
    let patch_json = common::read_json(patch_response).await;
    assert_eq!(patch_json["width"], 420);
    assert_eq!(patch_json["height"], 360);

    let status_response = app
        .oneshot(
            Request::builder()
                .uri("/simulation/status")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");
    assert_eq!(status_response.status(), StatusCode::OK);
    let status_json = common::read_json(status_response).await;
    assert_eq!(status_json["startup_draft"]["width"], 420);
    assert_eq!(status_json["startup_draft"]["height"], 360);
}

#[tokio::test]
async fn patch_config_updates_runtime_values() {
    let state = common::fast_state();
    let app = build_router(state);

    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/simulation/start")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");

    let patch_payload = json!({
        "paused": true,
        "ticks_per_second": 12,
        "food_spawn_rate": 0.18,
        "food_growth_rate": 0.22,
        "food_spread_threshold": 0.66,
        "food_spawn_floor_density": 0.04,
        "sensor_radius": 24,
        "food_max_density": 1.2,
        "food_energy_value": 0.48,
        "energy_per_tick_decay": 0.015,
        "energy_per_move": 0.025,
        "energy_per_compute_node": 0.009,
        "energy_per_reproduce": 0.18,
        "energy_max": 1.8,
        "min_reproduce_energy": 1.2,
        "offspring_energy_fraction": 0.38,
        "max_creatures": 4200,
        "weight_mutation_rate": 0.31,
        "weight_mutation_magnitude": 0.27,
        "logic_node_mutation_rate": 0.06,
        "structural_mutation_rate": 0.14
    });

    let patch_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/config")
                .header("content-type", "application/json")
                .body(Body::from(patch_payload.to_string()))
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");
    assert_eq!(patch_response.status(), StatusCode::OK);

    let get_response = app
        .oneshot(
            Request::builder()
                .uri("/config")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");
    assert_eq!(get_response.status(), StatusCode::OK);

    let cfg_json = common::read_json(get_response).await;
    assert_eq!(cfg_json["paused"], true);
    assert_eq!(cfg_json["ticks_per_second"], 12);
    assert_eq!(cfg_json["food_spawn_rate"], 0.18);
    assert_eq!(cfg_json["food_growth_rate"], 0.22);
    assert_eq!(cfg_json["food_spread_threshold"], 0.66);
    assert_eq!(cfg_json["food_spawn_floor_density"], 0.04);
    assert_eq!(cfg_json["sensor_radius"], 24);
    assert_eq!(cfg_json["food_max_density"], 1.2);
    assert_eq!(cfg_json["food_energy_value"], 0.48);
    assert_eq!(cfg_json["energy_per_tick_decay"], 0.015);
    assert_eq!(cfg_json["energy_per_move"], 0.025);
    assert_eq!(cfg_json["energy_per_compute_node"], 0.009);
    assert_eq!(cfg_json["energy_per_reproduce"], 0.18);
    assert_eq!(cfg_json["energy_max"], 1.8);
    assert_eq!(cfg_json["min_reproduce_energy"], 1.2);
    assert_eq!(cfg_json["offspring_energy_fraction"], 0.38);
    assert_eq!(cfg_json["max_creatures"], 4200);
    assert_eq!(cfg_json["weight_mutation_rate"], 0.31);
    assert_eq!(cfg_json["weight_mutation_magnitude"], 0.27);
    assert_eq!(cfg_json["logic_node_mutation_rate"], 0.06);
    assert_eq!(cfg_json["structural_mutation_rate"], 0.14);
}

#[tokio::test]
async fn patch_config_clamps_sensor_radius_to_minimum_one() {
    let state = common::fast_state();
    let app = build_router(state);

    let patch_payload = json!({
        "sensor_radius": 0
    });

    let patch_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/config")
                .header("content-type", "application/json")
                .body(Body::from(patch_payload.to_string()))
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");
    assert_eq!(patch_response.status(), StatusCode::OK);

    let get_response = app
        .oneshot(
            Request::builder()
                .uri("/config")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");
    assert_eq!(get_response.status(), StatusCode::OK);

    let cfg_json = common::read_json(get_response).await;
    assert_eq!(cfg_json["sensor_radius"], 1);
}

#[tokio::test]
async fn restart_clears_pending_restart_and_rotates_seed() {
    let state = common::fast_state();
    let app = build_router(state);

    let start_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/simulation/start")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");
    let start_json = common::read_json(start_response).await;
    let seed_before = start_json["seed"]
        .as_u64()
        .expect("seed should be present after start");
    let run_id_before = start_json["run_id"]
        .as_u64()
        .expect("run_id should be present after start");

    let patch_payload = json!({
        "initial_food_density": 0.35
    });
    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/simulation/startup-draft")
                .header("content-type", "application/json")
                .body(Body::from(patch_payload.to_string()))
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");

    let restart_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/simulation/restart")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");
    assert_eq!(restart_response.status(), StatusCode::OK);
    let restart_json = common::read_json(restart_response).await;

    assert_eq!(restart_json["pending_restart"], false);
    assert_eq!(restart_json["phase"], "running");
    assert_ne!(
        restart_json["seed"]
            .as_u64()
            .expect("seed should be present after restart"),
        seed_before
    );
    assert!(
        restart_json["run_id"]
            .as_u64()
            .expect("run_id should be present after restart")
            > run_id_before
    );
}

#[tokio::test]
async fn non_viable_startup_config_is_rejected() {
    let state = AppState::new_for_tests();
    let app = build_router(state);

    let patch_payload = json!({
        "initial_creatures": 1,
        "initial_food_density": 0.0,
        "food_spawn_rate": 0.0,
        "food_growth_rate": 0.0,
        "energy_per_tick_decay": 0.03,
        "energy_per_move": 0.05
    });

    let patch_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/simulation/startup-draft")
                .header("content-type", "application/json")
                .body(Body::from(patch_payload.to_string()))
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");
    assert_eq!(patch_response.status(), StatusCode::OK);

    let start_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/simulation/start")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");
    assert_eq!(start_response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let err_json = common::read_json(start_response).await;
    assert_eq!(err_json["code"], "non_viable_startup_config");
}

#[tokio::test]
async fn status_reports_non_viable_startup_draft() {
    let state = AppState::new_for_tests();
    let app = build_router(state);

    let patch_payload = json!({
        "initial_creatures": 1,
        "initial_food_density": 0.0,
        "food_spawn_rate": 0.0,
        "food_growth_rate": 0.0,
        "energy_per_tick_decay": 0.03,
        "energy_per_move": 0.05
    });

    let patch_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/simulation/startup-draft")
                .header("content-type", "application/json")
                .body(Body::from(patch_payload.to_string()))
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");
    assert_eq!(patch_response.status(), StatusCode::OK);

    let status_response = app
        .oneshot(
            Request::builder()
                .uri("/simulation/status")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");
    assert_eq!(status_response.status(), StatusCode::OK);
    let status_json = common::read_json(status_response).await;
    assert_eq!(status_json["startup_viable"], false);
    assert_eq!(
        status_json["startup_viability_code"],
        "non_viable_startup_config"
    );
}

#[tokio::test]
async fn non_viable_startup_can_start_when_viability_probe_is_disabled() {
    let state = AppState::new_with_options(
        1,
        WorldConfig::default(),
        AppStateOptions {
            viability_probe_enabled: false,
        },
    );
    let app = build_router(state);

    let patch_payload = json!({
        "initial_creatures": 1,
        "initial_food_density": 0.0,
        "food_spawn_rate": 0.0,
        "food_growth_rate": 0.0,
        "energy_per_tick_decay": 0.03,
        "energy_per_move": 0.05
    });

    let patch_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/simulation/startup-draft")
                .header("content-type", "application/json")
                .body(Body::from(patch_payload.to_string()))
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");
    assert_eq!(patch_response.status(), StatusCode::OK);

    let start_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/simulation/start")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");
    assert_eq!(start_response.status(), StatusCode::OK);
    let start_json = common::read_json(start_response).await;
    assert_eq!(start_json["phase"], "running");
    assert_eq!(start_json["viability_probe_enabled"], false);
    assert_eq!(start_json["startup_viable"], true);
}
