use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tokio::time::{timeout, Duration};
use tower::ServiceExt;

use v3_server::{handlers::lifecycle::build_ws_frame, router, state::AppState};

fn app() -> axum::Router {
    router(AppState::new())
}

async fn spawn_ws_app(state: AppState) -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind test listener");
    let addr = listener.local_addr().expect("listener address");
    let app = router(state);
    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve test app");
    });
    (format!("ws://{addr}/v3/ws"), handle)
}

async fn do_request(app: axum::Router, req: Request<Body>) -> (StatusCode, serde_json::Value) {
    let resp = app.oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
    (status, body)
}

async fn recv_server_messages(
    socket: &mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
) -> Vec<v3_server::transport::protocol::ServerMessage> {
    use futures_util::StreamExt;
    use tokio_tungstenite::tungstenite::Message;
    use v3_server::transport::protocol::ServerMessage;

    let mut messages = Vec::new();
    let first = timeout(Duration::from_secs(1), socket.next())
        .await
        .expect("expected websocket payload")
        .expect("socket stream item")
        .expect("websocket message");
    messages.push(match first {
        Message::Binary(bytes) => rmp_serde::from_slice(&bytes).expect("decode server message"),
        other => panic!("expected binary message, got {other:?}"),
    });

    loop {
        match timeout(Duration::from_millis(50), socket.next()).await {
            Ok(Some(Ok(Message::Binary(bytes)))) => {
                let decoded: ServerMessage =
                    rmp_serde::from_slice(&bytes).expect("decode server message");
                messages.push(decoded);
            }
            Ok(Some(Ok(other))) => panic!("expected binary message, got {other:?}"),
            Ok(Some(Err(error))) => panic!("websocket error: {error}"),
            Ok(None) | Err(_) => break,
        }
    }

    messages
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
    assert!(
        body["mutation_events_attempted_total_by_domain"].is_object(),
        "missing mutation_events_attempted_total_by_domain"
    );
    assert!(
        body["mutation_events_applied_total_by_domain"].is_object(),
        "missing mutation_events_applied_total_by_domain"
    );
    assert!(
        body["mutation_events_attempted_total_by_operator"].is_object(),
        "missing mutation_events_attempted_total_by_operator"
    );
    assert!(
        body["mutation_events_applied_total_by_operator"].is_object(),
        "missing mutation_events_applied_total_by_operator"
    );
    assert!(
        body["mutation_events_applied_total_semantic_noop"].is_number(),
        "missing mutation_events_applied_total_semantic_noop"
    );
    assert!(
        body["mutation_events_applied_total_semantic_change"].is_number(),
        "missing mutation_events_applied_total_semantic_change"
    );
}

// ── 9b. get_status_uses_energy_names_and_perf_block ────────────────────────

#[tokio::test]
async fn get_status_uses_energy_names_and_perf_block() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":42}"#))
        .await
        .unwrap();

    let (status, body) = do_request(a, get_req("/v3/simulation/status")).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert!(
        body.get("last_tick_compute_total_mean").is_none(),
        "legacy compute field must be removed: {body}"
    );
    assert!(
        body["last_tick_compute_energy_total_mean"].is_number(),
        "missing energy total mean: {body}"
    );
    assert!(
        body["last_tick_compute_energy_total_min"].is_number(),
        "missing energy total min: {body}"
    );
    assert!(
        body["last_tick_compute_energy_total_max"].is_number(),
        "missing energy total max: {body}"
    );
    assert!(
        body["last_tick_compute_energy_vm_mean"].is_number(),
        "missing energy vm mean: {body}"
    );
    assert!(
        body["last_tick_compute_energy_graph_mean"].is_number(),
        "missing energy graph mean: {body}"
    );
    assert!(body["perf"].is_object(), "missing perf block: {body}");
    assert!(
        body["perf"]["projection_publish_ms"].is_number(),
        "missing projection_publish_ms: {body}"
    );
    assert!(
        body["perf"]["ws_frame_publish_ms"].is_number(),
        "missing ws_frame_publish_ms: {body}"
    );
    assert!(
        body["perf"]["subscriber_count"].is_number(),
        "missing subscriber_count: {body}"
    );
}

// ── 10. get_frame_endpoint_is_removed ───────────────────────────────────────

#[tokio::test]
async fn get_frame_endpoint_is_removed() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":42}"#))
        .await
        .unwrap();

    let (status, body) = do_request(a, get_req("/v3/simulation/frame")).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "body: {body}");
}

// ── 10b. snapshot_bootstrap_returns_overview_and_revisions ─────────────────

#[tokio::test]
async fn snapshot_bootstrap_returns_overview_and_revisions() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":42}"#))
        .await
        .unwrap();

    let (status, body) = do_request(a, get_req("/v3/simulation/snapshot")).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert!(body["projection_revision"].is_number(), "body: {body}");
    assert!(body["world_static_revision"].is_number(), "body: {body}");
    assert_eq!(
        body["view"]["kind"].as_str(),
        Some("overview"),
        "body: {body}"
    );
    assert!(body["world_static"]["width"].is_number(), "body: {body}");
    assert!(body["world_static"]["height"].is_number(), "body: {body}");
    assert!(
        body["world_static"]["barrier_mask"].is_array(),
        "body: {body}"
    );
    assert!(body["view"]["grid_width"].is_number(), "body: {body}");
    assert!(body["view"]["grid_height"].is_number(), "body: {body}");
    assert!(body["view"]["food_density_u8"].is_array(), "body: {body}");
    assert!(
        body["view"]["creature_count_u16"].is_array(),
        "body: {body}"
    );
}

// ── 10c. projection_revision_increases_after_step ───────────────────────────

#[tokio::test]
async fn projection_revision_increases_after_step() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":7}"#))
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

    let (_, before) = do_request(a.clone(), get_req("/v3/simulation/status")).await;
    let before_revision = before["projection_revision"].as_u64().unwrap_or(0);
    let before_tick = before["tick"].as_u64().unwrap_or(0);

    let (step_status, _) = do_request(
        a.clone(),
        post_json("/v3/simulation/step", r#"{"steps":1}"#),
    )
    .await;
    assert_eq!(step_status, StatusCode::OK);

    let (_, after) = do_request(a, get_req("/v3/simulation/status")).await;
    let after_revision = after["projection_revision"].as_u64().unwrap_or(0);
    let after_tick = after["tick"].as_u64().unwrap_or(0);

    assert!(
        after_revision > before_revision,
        "projection_revision must increase after step: before={before_revision}, after={after_revision}, body={after}"
    );
    assert!(
        after_tick > before_tick,
        "tick must increase after step: before={before_tick}, after={after_tick}, body={after}"
    );
}

// ── 10d. world_static_revision_stays_stable_across_step ────────────────────

#[tokio::test]
async fn world_static_revision_stays_stable_across_step() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":13}"#))
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

    let (_, before) = do_request(a.clone(), get_req("/v3/simulation/snapshot")).await;
    let before_static_revision = before["world_static_revision"].as_u64().unwrap_or(0);

    let (step_status, _) = do_request(
        a.clone(),
        post_json("/v3/simulation/step", r#"{"steps":1}"#),
    )
    .await;
    assert_eq!(step_status, StatusCode::OK);

    let (_, after) = do_request(a, get_req("/v3/simulation/snapshot")).await;
    let after_static_revision = after["world_static_revision"].as_u64().unwrap_or(0);

    assert_eq!(
        after_static_revision, before_static_revision,
        "world_static_revision should not change on a pure tick without topology edits: before={before_static_revision}, after={after_static_revision}"
    );
}

// ── 10e. status_and_snapshot_share_projection_when_not_mutating ────────────

#[tokio::test]
async fn status_and_snapshot_share_projection_when_not_mutating() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":21}"#))
        .await
        .unwrap();

    let (_, status_body) = do_request(a.clone(), get_req("/v3/simulation/status")).await;
    let (_, snapshot_body) = do_request(a, get_req("/v3/simulation/snapshot")).await;

    assert_eq!(
        status_body["projection_revision"], snapshot_body["projection_revision"],
        "reads without an intervening mutation should come from the same published projection"
    );
    assert_eq!(status_body["tick"], snapshot_body["tick"]);
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
        run_tick(&mut sim, &mut None);
    }
    let handle = SimHandle {
        sim,
        status: SimulationStatus::Paused,
        active_trace: None,
    };
    let frame = build_ws_frame(&handle);
    // The field always exists as part of the typed struct; verify it's accessible.
    let _ = &frame.health.mutation_events_skipped_total_by_reason;
    let attempted_by_domain: u64 = frame
        .health
        .mutation_events_attempted_total_by_domain
        .values()
        .sum();
    let applied_by_domain: u64 = frame
        .health
        .mutation_events_applied_total_by_domain
        .values()
        .sum();
    let attempted_by_operator: u64 = frame
        .health
        .mutation_events_attempted_total_by_operator
        .values()
        .sum();
    let applied_by_operator: u64 = frame
        .health
        .mutation_events_applied_total_by_operator
        .values()
        .sum();
    assert_eq!(
        attempted_by_domain, frame.health.mutation_events_attempted_total,
        "attempted_by_domain must reconcile to mutation_events_attempted_total"
    );
    assert_eq!(
        applied_by_domain, frame.health.mutation_events_applied_total,
        "applied_by_domain must reconcile to mutation_events_applied_total"
    );
    assert_eq!(
        attempted_by_operator, frame.health.mutation_events_attempted_total,
        "attempted_by_operator must reconcile to mutation_events_attempted_total"
    );
    assert_eq!(
        applied_by_operator, frame.health.mutation_events_applied_total,
        "applied_by_operator must reconcile to mutation_events_applied_total"
    );
    assert_eq!(
        frame.health.mutation_events_applied_total_semantic_noop
            + frame.health.mutation_events_applied_total_semantic_change,
        frame.health.mutation_events_applied_total,
        "semantic categories must reconcile to mutation_events_applied_total"
    );
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
        active_trace: None,
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
        run_tick(&mut sim, &mut None);
    }
    let handle = SimHandle {
        sim,
        status: SimulationStatus::Running,
        active_trace: None,
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
    assert_eq!(
        decoded.health.mutation_events_attempted_total_by_domain,
        frame.health.mutation_events_attempted_total_by_domain
    );
    assert_eq!(
        decoded.health.mutation_events_applied_total_by_operator,
        frame.health.mutation_events_applied_total_by_operator
    );
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
    assert!(
        body.get("frame").is_none(),
        "legacy frame object must be absent: {body}"
    );
    assert!(body["dirty_rect"].is_object(), "missing dirty_rect: {body}");
    assert_eq!(
        body["world_static_changed"].as_bool(),
        Some(true),
        "barrier paint must mark static invalidation: {body}"
    );
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

    // Get snapshot to find a creature position
    let (_, snapshot_body) = do_request(
        a.clone(),
        get_req("/v3/simulation/snapshot?zoom_tier=detail"),
    )
    .await;
    let creatures = snapshot_body["view"]["creatures"]
        .as_array()
        .expect("creatures array");
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

// ── 25. paint_response_includes_invalidation_metadata ──────────────────────

#[tokio::test]
async fn paint_response_includes_invalidation_metadata() {
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
    assert!(
        body.get("frame").is_none(),
        "legacy embedded frame must be removed: {body}"
    );
    assert!(body["dirty_rect"].is_object(), "missing dirty_rect");
    assert!(
        body["world_static_changed"].is_boolean(),
        "missing world_static_changed"
    );
}

// ── 25b. paint_food_response_marks_only_dynamic_invalidation ────────────────

#[tokio::test]
async fn paint_food_response_marks_only_dynamic_invalidation() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":1}"#))
        .await
        .unwrap();

    let (status, body) = do_request(
        a,
        post_json(
            "/v3/simulation/paint",
            r#"{"tool":"food","brush_half_extent":1,"points":[{"x":1,"y":1}]}"#,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(
        body["world_static_changed"].as_bool(),
        Some(false),
        "body: {body}"
    );
    assert_eq!(body["dirty_rect"]["x"].as_u64(), Some(0));
    assert_eq!(body["dirty_rect"]["y"].as_u64(), Some(0));
    assert_eq!(body["dirty_rect"]["width"].as_u64(), Some(3));
    assert_eq!(body["dirty_rect"]["height"].as_u64(), Some(3));
}

// ── 25c. paint_barrier_response_marks_world_static_changed ──────────────────

#[tokio::test]
async fn paint_barrier_response_marks_world_static_changed() {
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
    assert_eq!(
        body["world_static_changed"].as_bool(),
        Some(true),
        "body: {body}"
    );
}

// ── 26. get_creature_returns_full_detail ──────────────────────────────────────

#[tokio::test]
async fn get_creature_returns_full_detail() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":42}"#))
        .await
        .unwrap();

    // Get snapshot to find a creature ID
    let (_, snapshot_body) = do_request(
        a.clone(),
        get_req("/v3/simulation/snapshot?zoom_tier=detail"),
    )
    .await;
    let creatures = snapshot_body["view"]["creatures"]
        .as_array()
        .expect("creatures array");
    assert!(!creatures.is_empty(), "need at least one creature");
    let creature_id = creatures[0]["id"].as_u64().unwrap();

    let uri = format!("/v3/simulation/creature/{creature_id}");
    let (status, body) = do_request(a, get_req(&uri)).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert!(
        body["protocol_version"].is_string(),
        "missing protocol_version"
    );
    assert_eq!(body["id"].as_u64(), Some(creature_id));
    assert!(body["position"]["x"].is_number(), "missing position.x");
    assert!(body["position"]["y"].is_number(), "missing position.y");
    assert!(body["energy"].is_number(), "missing energy");
    assert!(body["max_energy"].is_number(), "missing max_energy");
    assert!(body["age"].is_number(), "missing age");
    assert!(body["generation"].is_number(), "missing generation");
    assert!(body["complexity"].is_number(), "missing complexity");

    // Phenotype
    let pheno = &body["phenotype"];
    assert!(pheno["channels"].is_array(), "missing phenotype.channels");
    assert_eq!(
        pheno["channels"].as_array().unwrap().len(),
        6,
        "phenotype.channels must have 6 elements"
    );
    assert!(
        pheno["active_channel"].is_number(),
        "missing phenotype.active_channel"
    );
    assert!(pheno["polarity"].is_array(), "missing phenotype.polarity");
    assert!(pheno["rgb"].is_array(), "missing phenotype.rgb");
    assert_eq!(
        pheno["rgb"].as_array().unwrap().len(),
        3,
        "phenotype.rgb must have 3 elements"
    );

    // Genome
    assert!(body["genome"].is_object(), "missing genome");
    assert!(
        body["genome"]["entry_node_id"].is_number(),
        "missing genome.entry_node_id"
    );
    assert!(body["genome"]["nodes"].is_array(), "missing genome.nodes");

    // Memory
    assert!(body["memory"].is_array(), "missing memory");
    assert_eq!(
        body["memory"].as_array().unwrap().len(),
        1024,
        "memory must have 1024 bytes"
    );
}

// ── 27. get_creature_invalid_id_returns_404 ──────────────────────────────────

#[tokio::test]
async fn get_creature_invalid_id_returns_404() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":42}"#))
        .await
        .unwrap();

    // Use an ID that doesn't correspond to any creature
    let (status, body) = do_request(a, get_req("/v3/simulation/creature/999999999")).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "body: {body}");
    assert_eq!(body["error"].as_str(), Some("not_found"), "body: {body}");
}

// ── 28. get_creature_on_idle_state_works ─────────────────────────────────────

#[tokio::test]
async fn get_creature_on_idle_state_works() {
    // Fresh default state has creatures from default seed
    let a = app();

    // Get snapshot to find a creature (default state seeds creatures)
    let (_, snapshot_body) = do_request(
        a.clone(),
        get_req("/v3/simulation/snapshot?zoom_tier=detail"),
    )
    .await;
    let creatures = snapshot_body["view"]["creatures"]
        .as_array()
        .expect("creatures array");
    assert!(!creatures.is_empty(), "default state should have creatures");
    let creature_id = creatures[0]["id"].as_u64().unwrap();

    let uri = format!("/v3/simulation/creature/{creature_id}");
    let (status, body) = do_request(a, get_req(&uri)).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["id"].as_u64(), Some(creature_id));
}

// ── 29. paint_erase_barrier_clears_barrier ──────────────────────────────────

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

// ── 30. start_sample_returns_recording ────────────────────────────────────────

#[tokio::test]
async fn start_sample_returns_recording() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":42}"#))
        .await
        .unwrap();

    // Pause so we can step manually.
    let (_, _) = do_request(a.clone(), post_req("/v3/simulation/start")).await;
    let (_, _) = do_request(a.clone(), post_req("/v3/simulation/pause")).await;

    // Find a creature ID.
    let (_, snapshot_body) = do_request(
        a.clone(),
        get_req("/v3/simulation/snapshot?zoom_tier=detail"),
    )
    .await;
    let creature_id = snapshot_body["view"]["creatures"][0]["id"]
        .as_u64()
        .unwrap();

    // Start sample.
    let uri = format!("/v3/simulation/creature/{creature_id}/sample");
    let (status, body) = do_request(a.clone(), post_json(&uri, r#"{"ticks": 3}"#)).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["status"].as_str(), Some("recording"));
    assert_eq!(body["ticks_requested"].as_u64(), Some(3));

    // Step 3 ticks to complete the trace.
    let (_, _) = do_request(
        a.clone(),
        post_json("/v3/simulation/step", r#"{"steps": 3}"#),
    )
    .await;

    // Poll for completed sample.
    let (status, body) = do_request(a, get_req(&uri)).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["status"].as_str(), Some("complete"));
    assert!(body["sample"].is_object(), "missing sample");
    assert_eq!(body["sample"]["creature_id"].as_u64(), Some(creature_id));

    let ticks = body["sample"]["ticks"].as_array().expect("ticks array");
    assert_eq!(ticks.len(), 3, "expected 3 tick traces");

    // Each tick should have hops and action.
    for tick in ticks {
        assert!(tick["hops"].is_array(), "missing hops");
        assert!(tick["final_actions"].is_array(), "missing final_actions");
    }
}

// ── 31. start_sample_invalid_creature_returns_404 ─────────────────────────────

#[tokio::test]
async fn start_sample_invalid_creature_returns_404() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":42}"#))
        .await
        .unwrap();
    let (_, _) = do_request(a.clone(), post_req("/v3/simulation/start")).await;
    let (_, _) = do_request(a.clone(), post_req("/v3/simulation/pause")).await;

    let (status, _) = do_request(
        a,
        post_json(
            "/v3/simulation/creature/999999999/sample",
            r#"{"ticks": 5}"#,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ── 32. start_sample_while_idle_returns_409 ──────────────────────────────────

#[tokio::test]
async fn start_sample_while_idle_returns_409() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":42}"#))
        .await
        .unwrap();

    // Get a creature ID (available even while idle).
    let (_, snapshot_body) = do_request(
        a.clone(),
        get_req("/v3/simulation/snapshot?zoom_tier=detail"),
    )
    .await;
    let creature_id = snapshot_body["view"]["creatures"][0]["id"]
        .as_u64()
        .unwrap();

    let uri = format!("/v3/simulation/creature/{creature_id}/sample");
    let (status, _) = do_request(a, post_json(&uri, r#"{"ticks": 5}"#)).await;
    assert_eq!(status, StatusCode::CONFLICT);
}

// ── 33. get_sample_before_start_returns_idle ─────────────────────────────────

#[tokio::test]
async fn get_sample_before_start_returns_idle() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":42}"#))
        .await
        .unwrap();
    let (_, _) = do_request(a.clone(), post_req("/v3/simulation/start")).await;
    let (_, _) = do_request(a.clone(), post_req("/v3/simulation/pause")).await;

    let (_, snapshot_body) = do_request(
        a.clone(),
        get_req("/v3/simulation/snapshot?zoom_tier=detail"),
    )
    .await;
    let creature_id = snapshot_body["view"]["creatures"][0]["id"]
        .as_u64()
        .unwrap();

    let uri = format!("/v3/simulation/creature/{creature_id}/sample");
    let (status, body) = do_request(a, get_req(&uri)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"].as_str(), Some("idle"));
}

// ── 34. get_sample_while_recording_returns_progress ──────────────────────────

#[tokio::test]
async fn get_sample_while_recording_returns_progress() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":42}"#))
        .await
        .unwrap();
    let (_, _) = do_request(a.clone(), post_req("/v3/simulation/start")).await;
    let (_, _) = do_request(a.clone(), post_req("/v3/simulation/pause")).await;

    let (_, snapshot_body) = do_request(
        a.clone(),
        get_req("/v3/simulation/snapshot?zoom_tier=detail"),
    )
    .await;
    let creature_id = snapshot_body["view"]["creatures"][0]["id"]
        .as_u64()
        .unwrap();

    // Start sample with 5 ticks.
    let uri = format!("/v3/simulation/creature/{creature_id}/sample");
    let (_, _) = do_request(a.clone(), post_json(&uri, r#"{"ticks": 5}"#)).await;

    // Step only 2 ticks.
    let (_, _) = do_request(
        a.clone(),
        post_json("/v3/simulation/step", r#"{"steps": 2}"#),
    )
    .await;

    // Poll — should be "recording" with progress.
    let (status, body) = do_request(a, get_req(&uri)).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["status"].as_str(), Some("recording"));
    assert_eq!(body["ticks_completed"].as_u64(), Some(2));
    assert_eq!(body["ticks_remaining"].as_u64(), Some(3));
}

// ── 35. ws_updates_require_active_subscription ──────────────────────────────

#[tokio::test]
async fn ws_updates_require_active_subscription() {
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::connect_async;
    use tokio_tungstenite::tungstenite::Message;
    use v3_server::transport::protocol::ServerMessage;

    let state = AppState::new();
    let (ws_url, server_task) = spawn_ws_app(state.clone()).await;
    let (mut socket, _) = connect_async(ws_url).await.expect("connect websocket");

    {
        let handle = state.sim.lock().await;
        state.publish_ws_frame(build_ws_frame(&handle));
    }

    assert!(
        timeout(Duration::from_millis(150), socket.next())
            .await
            .is_err(),
        "server should not deliver view updates before subscribe_view"
    );

    socket
        .send(Message::Text(
            serde_json::json!({
                "type": "subscribe_view",
                "request_id": 1u64,
                "x": 0u16,
                "y": 0u16,
                "width": 8u16,
                "height": 8u16,
                "canvas_width": 320u16,
                "canvas_height": 240u16,
                "zoom_tier": "overview"
            })
            .to_string(),
        ))
        .await
        .expect("send subscribe message");

    {
        let handle = state.sim.lock().await;
        state.publish_ws_frame(build_ws_frame(&handle));
    }

    let messages = recv_server_messages(&mut socket).await;
    assert!(
        messages
            .iter()
            .any(|message| matches!(message, ServerMessage::WorldStatic { .. })),
        "subscribe should prime a world_static event: {messages:?}"
    );
    assert!(
        messages.iter().any(|message| matches!(
            message,
            ServerMessage::ViewOverview { request_id, .. } if *request_id == 1
        )),
        "subscribe should deliver an overview view event for request_id=1: {messages:?}"
    );

    server_task.abort();
}

// ── 36. ws_latest_request_id_wins ───────────────────────────────────────────

#[tokio::test]
async fn ws_latest_request_id_wins() {
    use futures_util::SinkExt;
    use tokio_tungstenite::connect_async;
    use tokio_tungstenite::tungstenite::Message;
    use v3_server::transport::protocol::ServerMessage;

    let state = AppState::new();
    let (ws_url, server_task) = spawn_ws_app(state.clone()).await;
    let (mut socket, _) = connect_async(ws_url).await.expect("connect websocket");

    socket
        .send(Message::Text(
            serde_json::json!({
                "type": "subscribe_view",
                "request_id": 2u64,
                "x": 0u16,
                "y": 0u16,
                "width": 8u16,
                "height": 8u16,
                "canvas_width": 320u16,
                "canvas_height": 240u16,
                "zoom_tier": "detail"
            })
            .to_string(),
        ))
        .await
        .expect("send latest subscribe");
    socket
        .send(Message::Text(
            serde_json::json!({
                "type": "subscribe_view",
                "request_id": 1u64,
                "x": 4u16,
                "y": 4u16,
                "width": 4u16,
                "height": 4u16,
                "canvas_width": 128u16,
                "canvas_height": 128u16,
                "zoom_tier": "overview"
            })
            .to_string(),
        ))
        .await
        .expect("send stale subscribe");

    {
        let handle = state.sim.lock().await;
        state.publish_ws_frame(build_ws_frame(&handle));
    }

    let messages = recv_server_messages(&mut socket).await;
    assert!(
        messages.iter().any(|message| matches!(
            message,
            ServerMessage::ViewDetail { request_id, .. } if *request_id == 2
        )),
        "latest subscription should own emitted detail views: {messages:?}"
    );
    assert!(
        !messages.iter().any(|message| matches!(
            message,
            ServerMessage::ViewOverview { request_id, .. } | ServerMessage::ViewDetail { request_id, .. } if *request_id == 1
        )),
        "stale request_id=1 must not produce view messages: {messages:?}"
    );

    server_task.abort();
}

// ── 37. ws_unsubscribe_stops_delivery ───────────────────────────────────────

#[tokio::test]
async fn ws_unsubscribe_stops_delivery() {
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::connect_async;
    use tokio_tungstenite::tungstenite::Message;
    use v3_server::transport::protocol::ServerMessage;

    let state = AppState::new();
    let (ws_url, server_task) = spawn_ws_app(state.clone()).await;
    let (mut socket, _) = connect_async(ws_url).await.expect("connect websocket");

    socket
        .send(Message::Text(
            serde_json::json!({
                "type": "subscribe_view",
                "request_id": 4u64,
                "x": 0u16,
                "y": 0u16,
                "width": 8u16,
                "height": 8u16,
                "canvas_width": 320u16,
                "canvas_height": 240u16,
                "zoom_tier": "detail"
            })
            .to_string(),
        ))
        .await
        .expect("send subscribe");

    let initial_messages = recv_server_messages(&mut socket).await;
    assert!(
        initial_messages.iter().any(|message| matches!(
            message,
            ServerMessage::ViewDetail { request_id, .. } if *request_id == 4
        )),
        "initial subscribe should deliver a detail view: {initial_messages:?}"
    );

    socket
        .send(Message::Text(
            serde_json::json!({
                "type": "unsubscribe_view"
            })
            .to_string(),
        ))
        .await
        .expect("send unsubscribe");
    tokio::time::sleep(Duration::from_millis(50)).await;

    {
        let handle = state.sim.lock().await;
        state.publish_ws_frame(build_ws_frame(&handle));
    }

    assert!(
        timeout(Duration::from_millis(150), socket.next())
            .await
            .is_err(),
        "server should stop delivering after unsubscribe_view"
    );

    server_task.abort();
}

// ── 37b. ws_disconnect_cleans_up_session ───────────────────────────────────

#[tokio::test]
async fn ws_disconnect_cleans_up_session() {
    use futures_util::SinkExt;
    use tokio_tungstenite::connect_async;
    use tokio_tungstenite::tungstenite::Message;

    let state = AppState::new();
    let (ws_url, server_task) = spawn_ws_app(state.clone()).await;
    let (mut socket, _) = connect_async(ws_url).await.expect("connect websocket");

    socket
        .send(Message::Text(
            serde_json::json!({
                "type": "subscribe_view",
                "request_id": 7u64,
                "x": 0u16,
                "y": 0u16,
                "width": 8u16,
                "height": 8u16,
                "canvas_width": 320u16,
                "canvas_height": 240u16,
                "zoom_tier": "detail"
            })
            .to_string(),
        ))
        .await
        .expect("send subscribe");

    let _ = recv_server_messages(&mut socket).await;
    assert_eq!(
        state
            .sessions
            .read()
            .expect("session registry lock poisoned")
            .len(),
        1,
        "connected websocket should register a transport session"
    );

    socket.close(None).await.expect("close websocket");
    drop(socket);

    timeout(Duration::from_secs(1), async {
        loop {
            if state
                .sessions
                .read()
                .expect("session registry lock poisoned")
                .is_empty()
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("server should remove the transport session after disconnect");

    server_task.abort();
}

// ── 38. ws_non_overlapping_paint_does_not_refresh_view ─────────────────────

#[tokio::test]
async fn ws_non_overlapping_paint_does_not_refresh_view() {
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::connect_async;
    use tokio_tungstenite::tungstenite::Message;
    use v3_server::transport::protocol::ServerMessage;

    let state = AppState::new();
    let (ws_url, server_task) = spawn_ws_app(state.clone()).await;
    let (mut socket, _) = connect_async(ws_url).await.expect("connect websocket");

    socket
        .send(Message::Text(
            serde_json::json!({
                "type": "subscribe_view",
                "request_id": 5u64,
                "x": 40u16,
                "y": 40u16,
                "width": 8u16,
                "height": 8u16,
                "canvas_width": 320u16,
                "canvas_height": 240u16,
                "zoom_tier": "detail"
            })
            .to_string(),
        ))
        .await
        .expect("send subscribe");

    let initial_messages = recv_server_messages(&mut socket).await;
    assert!(
        initial_messages.iter().any(|message| matches!(
            message,
            ServerMessage::ViewDetail { request_id, .. } if *request_id == 5
        )),
        "initial subscribe should deliver a detail view: {initial_messages:?}"
    );

    let (paint_status, _) = do_request(
        router(state),
        post_json(
            "/v3/simulation/paint",
            r#"{"tool":"food","brush_half_extent":0,"points":[{"x":1,"y":1}]}"#,
        ),
    )
    .await;
    assert_eq!(paint_status, StatusCode::OK);

    assert!(
        timeout(Duration::from_millis(150), socket.next())
            .await
            .is_err(),
        "non-overlapping paint should not refresh an unrelated subscribed view"
    );

    server_task.abort();
}

// ── 39. ws_non_overlapping_barrier_paint_rebroadcasts_world_static_only ────

#[tokio::test]
async fn ws_non_overlapping_barrier_paint_rebroadcasts_world_static_only() {
    use futures_util::SinkExt;
    use tokio_tungstenite::connect_async;
    use tokio_tungstenite::tungstenite::Message;
    use v3_server::transport::protocol::ServerMessage;

    let state = AppState::new();
    let (ws_url, server_task) = spawn_ws_app(state.clone()).await;
    let (mut socket, _) = connect_async(ws_url).await.expect("connect websocket");

    socket
        .send(Message::Text(
            serde_json::json!({
                "type": "subscribe_view",
                "request_id": 6u64,
                "x": 40u16,
                "y": 40u16,
                "width": 8u16,
                "height": 8u16,
                "canvas_width": 320u16,
                "canvas_height": 240u16,
                "zoom_tier": "detail"
            })
            .to_string(),
        ))
        .await
        .expect("send subscribe");

    let initial_messages = recv_server_messages(&mut socket).await;
    assert!(
        initial_messages.iter().any(|message| matches!(
            message,
            ServerMessage::ViewDetail { request_id, .. } if *request_id == 6
        )),
        "initial subscribe should deliver a detail view: {initial_messages:?}"
    );

    let (paint_status, _) = do_request(
        router(state),
        post_json(
            "/v3/simulation/paint",
            r#"{"tool":"barrier","brush_half_extent":0,"points":[{"x":1,"y":1}]}"#,
        ),
    )
    .await;
    assert_eq!(paint_status, StatusCode::OK);

    let messages = recv_server_messages(&mut socket).await;
    assert!(
        messages
            .iter()
            .any(|message| matches!(message, ServerMessage::WorldStatic { .. })),
        "barrier paint must rebroadcast world_static globally: {messages:?}"
    );
    assert!(
        !messages.iter().any(|message| matches!(
            message,
            ServerMessage::ViewOverview { .. } | ServerMessage::ViewDetail { .. }
        )),
        "non-overlapping barrier paint must not resend the current viewport: {messages:?}"
    );

    server_task.abort();
}
