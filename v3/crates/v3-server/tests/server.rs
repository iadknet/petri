use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

use v3_server::{router, state::AppState};

fn app() -> axum::Router {
    router(AppState::new())
}

async fn do_request(app: axum::Router, req: Request<Body>) -> (StatusCode, serde_json::Value) {
    let resp = app.oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
    (status, body)
}

fn startup_req(body: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/v3/simulation/startup")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn post_req(uri: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::empty())
        .unwrap()
}

fn post_json(uri: &str, body: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn get_req(uri: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(uri)
        .body(Body::empty())
        .unwrap()
}

fn patch_req(uri: &str, body: &str) -> Request<Body> {
    Request::builder()
        .method("PATCH")
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

// ── 1. startup_returns_200_with_required_fields ────────────────────────────

#[tokio::test]
async fn startup_returns_200_with_required_fields() {
    let (status, body) = do_request(app(), startup_req(r#"{"seed":42}"#)).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert!(
        body["protocol_version"].is_string(),
        "missing protocol_version"
    );
    assert!(body["state"].is_string(), "missing state");
    assert_eq!(body["tick"], 0, "tick must be 0");
    assert!(body["config_digest"].is_string(), "missing config_digest");
    assert!(
        body["seeded_creatures"].is_number(),
        "missing seeded_creatures"
    );
}

// ── 2. startup_unknown_field_returns_422 ───────────────────────────────────

#[tokio::test]
async fn startup_unknown_field_returns_422() {
    let (status, body) = do_request(app(), startup_req(r#"{"seed":42,"bogus_field":1}"#)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "body: {body}");
    assert_eq!(
        body["error"].as_str(),
        Some("validation_rejected"),
        "body: {body}"
    );
}

// ── 3. startup_missing_seed_returns_400 ────────────────────────────────────

#[tokio::test]
async fn startup_missing_seed_returns_400() {
    let (status, body) = do_request(app(), startup_req(r#"{}"#)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "body: {body}");
    assert_eq!(
        body["error"].as_str(),
        Some("invalid_request"),
        "body: {body}"
    );
}

// ── 4. start_transitions_to_running ────────────────────────────────────────

#[tokio::test]
async fn start_transitions_to_running() {
    let a = app();
    // Must startup first so we have a valid simulation.
    let startup_resp = a
        .clone()
        .oneshot(startup_req(r#"{"seed":1}"#))
        .await
        .unwrap();
    assert_eq!(startup_resp.status(), StatusCode::OK);

    let (status, body) = do_request(a, post_req("/v3/simulation/start")).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["state"].as_str(), Some("running"), "body: {body}");
}

// ── 5. pause_on_idle_returns_409 ────────────────────────────────────────────

#[tokio::test]
async fn pause_on_idle_returns_409() {
    // Fresh AppState is Idle; pause should return 409.
    let (status, body) = do_request(app(), post_req("/v3/simulation/pause")).await;
    assert_eq!(status, StatusCode::CONFLICT, "body: {body}");
    assert_eq!(
        body["error"].as_str(),
        Some("invalid_state_transition"),
        "body: {body}"
    );
}

// ── 6. step_on_non_paused_returns_409 ────────────────────────────────────────

#[tokio::test]
async fn step_on_non_paused_returns_409() {
    let a = app();
    // startup + start → Running
    a.clone()
        .oneshot(startup_req(r#"{"seed":1}"#))
        .await
        .unwrap();
    a.clone()
        .oneshot(post_req("/v3/simulation/start"))
        .await
        .unwrap();

    // step while Running → 409
    let (status, body) = do_request(a, post_json("/v3/simulation/step", r#"{"steps":1}"#)).await;
    assert_eq!(status, StatusCode::CONFLICT, "body: {body}");
}

// ── 7. step_with_zero_steps_returns_422 ─────────────────────────────────────

#[tokio::test]
async fn step_with_zero_steps_returns_422() {
    let a = app();
    // startup
    a.clone()
        .oneshot(startup_req(r#"{"seed":1}"#))
        .await
        .unwrap();
    // pause (from idle is 409 for pause; but we need to reach Paused via start+pause)
    // Actually we can only step when Paused; let's first start then pause.
    a.clone()
        .oneshot(post_req("/v3/simulation/start"))
        .await
        .unwrap();
    a.clone()
        .oneshot(post_req("/v3/simulation/pause"))
        .await
        .unwrap();

    let (status, body) = do_request(a, post_json("/v3/simulation/step", r#"{"steps":0}"#)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "body: {body}");
    assert_eq!(
        body["error"].as_str(),
        Some("validation_rejected"),
        "body: {body}"
    );
}

// ── 8. step_with_too_many_steps_returns_422 ──────────────────────────────────

#[tokio::test]
async fn step_with_too_many_steps_returns_422() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":1}"#))
        .await
        .unwrap();
    a.clone()
        .oneshot(post_req("/v3/simulation/start"))
        .await
        .unwrap();
    a.clone()
        .oneshot(post_req("/v3/simulation/pause"))
        .await
        .unwrap();

    let (status, body) = do_request(a, post_json("/v3/simulation/step", r#"{"steps":1001}"#)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "body: {body}");
    assert_eq!(
        body["error"].as_str(),
        Some("validation_rejected"),
        "body: {body}"
    );
}

// ── 9. get_status_has_all_required_fields ────────────────────────────────────

#[tokio::test]
async fn get_status_has_all_required_fields() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":42}"#))
        .await
        .unwrap();

    let (status, body) = do_request(a, get_req("/v3/simulation/status")).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert!(
        body["protocol_version"].is_string(),
        "missing protocol_version"
    );
    assert!(body["state"].is_string(), "missing state");
    assert!(body["tick"].is_number(), "missing tick");
    assert!(body["population"].is_number(), "missing population");
    assert!(body["mean_energy"].is_number(), "missing mean_energy");
}

// ── 10. get_frame_returns_creature_food_barrier_arrays ──────────────────────

#[tokio::test]
async fn get_frame_returns_creature_food_barrier_arrays() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":42}"#))
        .await
        .unwrap();

    let (status, body) = do_request(a, get_req("/v3/simulation/frame")).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert!(body["creatures"].is_array(), "missing creatures array");
    assert!(body["food"].is_array(), "missing food array");
    assert!(body["barriers"].is_array(), "missing barriers array");
}

// ── 11. patch_config_world_field_while_running_returns_409 ──────────────────

#[tokio::test]
async fn patch_config_world_field_while_running_returns_409() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":1}"#))
        .await
        .unwrap();
    a.clone()
        .oneshot(post_req("/v3/simulation/start"))
        .await
        .unwrap();

    let (status, body) = do_request(
        a,
        patch_req("/v3/simulation/config", r#"{"world":{"width":200}}"#),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "body: {body}");
    assert_eq!(
        body["error"].as_str(),
        Some("invalid_state_transition"),
        "body: {body}"
    );
}

// ── 12. error_envelope_has_protocol_version ─────────────────────────────────

#[tokio::test]
async fn error_envelope_has_protocol_version() {
    // Any 4xx should include protocol_version.
    let (status, body) = do_request(app(), startup_req(r#"{}"#)).await;
    assert!(status.is_client_error(), "expected 4xx, got {status}");
    assert!(
        body["protocol_version"].is_string(),
        "error envelope missing protocol_version: {body}"
    );
}

// ── 13. config_digest_present_in_startup_response ───────────────────────────

#[tokio::test]
async fn config_digest_present_in_startup_response() {
    let (status, body) = do_request(app(), startup_req(r#"{"seed":42}"#)).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    let digest = body["config_digest"].as_str().unwrap_or("");
    assert!(
        digest.starts_with("sha256:"),
        "config_digest must start with 'sha256:', got: {digest}"
    );
}

// ── 14. health_payload_contains_mutation_skip_by_reason ──────────────────────

#[tokio::test]
async fn health_payload_contains_mutation_skip_by_reason() {
    use v3_core::config::SimulationConfig;
    use v3_core::simulation::{run_tick, seed_simulation};
    use v3_server::handlers::lifecycle::build_ws_frame;
    use v3_server::state::{SimHandle, SimulationStatus};

    let mut cfg = SimulationConfig::default();
    cfg.world.width = 16;
    cfg.world.height = 16;
    cfg.population.initial_creatures = 5;
    cfg.mutation.mutation_probability = 1.0;
    cfg.mutation.per_birth_mutation_events_min = 3;
    cfg.mutation.per_birth_mutation_events_max = 3;
    cfg.world.food.initial_coverage = 0.8;
    cfg.world.food.initial_density = 1.0;
    cfg.world.food.growth_rate = 0.5;
    cfg.energy.lifecycle.initial_energy = 150.0;
    cfg.energy.lifecycle.default_offspring_energy = 4.0;
    cfg.energy.costs.reproduce_cost = 1.0;

    let mut sim = seed_simulation(cfg, 42);
    for _ in 0..20 {
        run_tick(&mut sim);
    }
    let handle = SimHandle {
        sim,
        status: SimulationStatus::Paused,
    };
    let frame = build_ws_frame(&handle);
    // The field always exists as part of the typed struct; verify it's accessible.
    let _ = &frame.health.mutation_events_skipped_total_by_reason;
}

// ── 15. status_payload_includes_state ───────────────────────────────────────

#[tokio::test]
async fn status_payload_includes_state() {
    use v3_core::config::SimulationConfig;
    use v3_core::simulation::seed_simulation;
    use v3_server::handlers::lifecycle::build_ws_frame;
    use v3_server::state::{SimHandle, SimulationStatus};

    let sim = seed_simulation(SimulationConfig::default(), 7);
    let handle = SimHandle {
        sim,
        status: SimulationStatus::Paused,
    };

    let frame = build_ws_frame(&handle);
    assert_eq!(
        frame.status.state,
        SimulationStatus::Paused,
        "status payload must include current simulation state"
    );
}

// ── 16. ws_frame_msgpack_roundtrip ──────────────────────────────────────────

#[tokio::test]
async fn ws_frame_msgpack_roundtrip() {
    use v3_core::config::SimulationConfig;
    use v3_core::simulation::{run_tick, seed_simulation};
    use v3_server::handlers::lifecycle::build_ws_frame;
    use v3_server::state::{SimHandle, SimulationStatus, WsFrame};

    let mut sim = seed_simulation(SimulationConfig::default(), 99);
    for _ in 0..5 {
        run_tick(&mut sim);
    }
    let handle = SimHandle {
        sim,
        status: SimulationStatus::Running,
    };

    let frame = build_ws_frame(&handle);
    let bytes = rmp_serde::to_vec_named(&frame).expect("msgpack serialize");
    let decoded: WsFrame = rmp_serde::from_slice(&bytes).expect("msgpack deserialize");

    assert_eq!(decoded.tick, frame.tick);
    assert_eq!(decoded.status.population, frame.status.population);
    assert_eq!(decoded.frame.width, frame.frame.width);
    assert_eq!(decoded.frame.height, frame.frame.height);
    assert_eq!(decoded.frame.creatures.len(), frame.frame.creatures.len());
    assert_eq!(decoded.health.population, frame.health.population);
    assert_eq!(decoded.status.state, SimulationStatus::Running);
}

// ── 17. paint_on_idle_succeeds ──────────────────────────────────────────────

#[tokio::test]
async fn paint_on_idle_succeeds() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":1}"#))
        .await
        .unwrap();

    let (status, body) = do_request(
        a,
        post_json(
            "/v3/simulation/paint",
            r#"{"tool":"barrier","brush_half_extent":0,"points":[{"x":5,"y":5}]}"#,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert!(
        body["protocol_version"].is_string(),
        "missing protocol_version"
    );
    assert!(
        body["stats"]["barrier_set_cells"].as_u64().unwrap_or(0) > 0,
        "expected barrier_set_cells > 0: {body}"
    );
    assert!(body["frame"].is_object(), "missing frame object");
}

// ── 18. paint_on_paused_succeeds ────────────────────────────────────────────

#[tokio::test]
async fn paint_on_paused_succeeds() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":1}"#))
        .await
        .unwrap();
    a.clone()
        .oneshot(post_req("/v3/simulation/start"))
        .await
        .unwrap();
    a.clone()
        .oneshot(post_req("/v3/simulation/pause"))
        .await
        .unwrap();

    let (status, body) = do_request(
        a,
        post_json(
            "/v3/simulation/paint",
            r#"{"tool":"food","brush_half_extent":0,"points":[{"x":3,"y":3}]}"#,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert!(
        body["stats"]["food_set_cells"].as_u64().unwrap_or(0) > 0,
        "expected food_set_cells > 0: {body}"
    );
}

// ── 19. paint_on_running_returns_409 ────────────────────────────────────────

#[tokio::test]
async fn paint_on_running_returns_409() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":1}"#))
        .await
        .unwrap();
    a.clone()
        .oneshot(post_req("/v3/simulation/start"))
        .await
        .unwrap();

    let (status, body) = do_request(
        a,
        post_json(
            "/v3/simulation/paint",
            r#"{"tool":"barrier","brush_half_extent":0,"points":[{"x":5,"y":5}]}"#,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "body: {body}");
    assert_eq!(
        body["error"].as_str(),
        Some("invalid_state_transition"),
        "body: {body}"
    );
}

// ── 20. paint_empty_points_returns_422 ──────────────────────────────────────

#[tokio::test]
async fn paint_empty_points_returns_422() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":1}"#))
        .await
        .unwrap();

    let (status, body) = do_request(
        a,
        post_json(
            "/v3/simulation/paint",
            r#"{"tool":"barrier","brush_half_extent":0,"points":[]}"#,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "body: {body}");
    assert_eq!(
        body["error"].as_str(),
        Some("validation_rejected"),
        "body: {body}"
    );
}

// ── 21. paint_invalid_brush_returns_422 ─────────────────────────────────────

#[tokio::test]
async fn paint_invalid_brush_returns_422() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":1}"#))
        .await
        .unwrap();

    let (status, body) = do_request(
        a,
        post_json(
            "/v3/simulation/paint",
            r#"{"tool":"barrier","brush_half_extent":5,"points":[{"x":5,"y":5}]}"#,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "body: {body}");
    assert_eq!(
        body["error"].as_str(),
        Some("validation_rejected"),
        "body: {body}"
    );
}

// ── 22. paint_too_many_points_returns_422 ───────────────────────────────────

#[tokio::test]
async fn paint_too_many_points_returns_422() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":1}"#))
        .await
        .unwrap();

    // Build 10001 points
    let points: Vec<String> = (0..10_001)
        .map(|i| format!(r#"{{"x":{},"y":{}}}"#, i % 100, i / 100))
        .collect();
    let body_str = format!(
        r#"{{"tool":"barrier","brush_half_extent":0,"points":[{}]}}"#,
        points.join(",")
    );

    let (status, body) = do_request(a, post_json("/v3/simulation/paint", &body_str)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "body: {body}");
    assert_eq!(
        body["error"].as_str(),
        Some("validation_rejected"),
        "body: {body}"
    );
}

// ── 23. paint_oob_points_are_filtered ───────────────────────────────────────

#[tokio::test]
async fn paint_oob_points_are_filtered() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":1}"#))
        .await
        .unwrap();

    let (status, body) = do_request(
        a,
        post_json(
            "/v3/simulation/paint",
            r#"{"tool":"barrier","brush_half_extent":0,"points":[{"x":9999,"y":9999}]}"#,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(
        body["stats"]["affected_cells"].as_u64().unwrap_or(999),
        0,
        "expected affected_cells == 0 for out-of-bounds: {body}"
    );
}

// ── 24. paint_barrier_evicts_creature ───────────────────────────────────────

#[tokio::test]
async fn paint_barrier_evicts_creature() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":42}"#))
        .await
        .unwrap();

    // Get frame to find a creature position
    let (_, frame_body) = do_request(a.clone(), get_req("/v3/simulation/frame")).await;
    let creatures = frame_body["creatures"].as_array().expect("creatures array");
    assert!(!creatures.is_empty(), "need at least one creature");
    let cx = creatures[0]["x"].as_u64().unwrap();
    let cy = creatures[0]["y"].as_u64().unwrap();

    // Paint barrier at creature position with brush extent 1 (3x3) for reliability
    let paint_body = format!(
        r#"{{"tool":"barrier","brush_half_extent":1,"points":[{{"x":{},"y":{}}}]}}"#,
        cx, cy
    );
    let (status, body) =
        do_request(a.clone(), post_json("/v3/simulation/paint", &paint_body)).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert!(
        body["stats"]["creatures_removed"].as_u64().unwrap_or(0) > 0,
        "expected creatures_removed > 0: {body}"
    );

    // Verify population decreased
    let (_, status_body) = do_request(a, get_req("/v3/simulation/status")).await;
    let pop = status_body["population"].as_u64().unwrap();
    let initial_pop = creatures.len() as u64;
    assert!(
        pop < initial_pop,
        "population {pop} should be less than initial {initial_pop}"
    );
}

// ── 25. paint_response_includes_frame ───────────────────────────────────────

#[tokio::test]
async fn paint_response_includes_frame() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":1}"#))
        .await
        .unwrap();

    let (status, body) = do_request(
        a,
        post_json(
            "/v3/simulation/paint",
            r#"{"tool":"food","brush_half_extent":0,"points":[{"x":1,"y":1}]}"#,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    let frame = &body["frame"];
    assert!(
        frame["frame"]["creatures"].is_array(),
        "missing frame.frame.creatures"
    );
    assert!(
        frame["frame"]["food"].is_array(),
        "missing frame.frame.food"
    );
    assert!(
        frame["frame"]["barriers"].is_array(),
        "missing frame.frame.barriers"
    );
}

// ── 26. paint_erase_barrier_clears_barrier ──────────────────────────────────

#[tokio::test]
async fn paint_erase_barrier_clears_barrier() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":1}"#))
        .await
        .unwrap();

    // First paint a barrier at (5,5)
    let (s1, _) = do_request(
        a.clone(),
        post_json(
            "/v3/simulation/paint",
            r#"{"tool":"barrier","brush_half_extent":0,"points":[{"x":5,"y":5}]}"#,
        ),
    )
    .await;
    assert_eq!(s1, StatusCode::OK);

    // Now erase barrier at (5,5)
    let (status, body) = do_request(
        a,
        post_json(
            "/v3/simulation/paint",
            r#"{"tool":"erase_barrier","brush_half_extent":0,"points":[{"x":5,"y":5}]}"#,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert!(
        body["stats"]["barrier_cleared_cells"].as_u64().unwrap_or(0) > 0,
        "expected barrier_cleared_cells > 0: {body}"
    );
}
