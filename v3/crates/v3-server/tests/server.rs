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
    use v3_server::handlers::lifecycle::build_ws_event;
    use v3_server::state::{SimHandle, SimulationStatus};

    let mut cfg = SimulationConfig::default();
    cfg.world.width = 16;
    cfg.world.height = 16;
    cfg.population.initial_creatures = 5;
    cfg.mutation.mutation_probability = 1.0;
    cfg.mutation.per_birth_mutation_events_min = 3;
    cfg.mutation.per_birth_mutation_events_max = 3;
    cfg.world.food.initial_coverage = 0.8;
    cfg.world.food.initial_density = 120;
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
    let event = build_ws_event(&handle);
    assert!(
        event
            .health_payload
            .get("mutation_events_skipped_total_by_reason")
            .is_some(),
        "health_payload must include mutation_events_skipped_total_by_reason"
    );
}
