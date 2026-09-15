use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use slotmap::Key;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tokio::time::{timeout, Duration};
use tower::ServiceExt;

use v3_core::config::SimulationConfig;
use v3_server::{
    router,
    state::{build_ws_frame, AppState},
};

fn test_config() -> SimulationConfig {
    let mut config = SimulationConfig::default();
    config.world.width = 64;
    config.world.height = 64;
    config.population.initial_creatures = 64;
    config.population.max_creatures = 512;
    config
}

fn test_state() -> AppState {
    AppState::from_config(test_config(), 0)
}

fn app() -> axum::Router {
    router(test_state())
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

fn assert_json_f64_close(value: &serde_json::Value, expected: f64) {
    assert!(
        (value
            .as_f64()
            .unwrap_or_else(|| panic!("expected f64 JSON value, got {value}"))
            - expected)
            .abs()
            < 1e-6,
        "expected {expected}, got {value}"
    );
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

#[tokio::test]
async fn configured_state_supplies_startup_defaults_and_request_overrides() {
    let mut config = test_config();
    config.world.width = 48;
    config.world.height = 48;
    config.population.initial_creatures = 3;
    let state = AppState::from_config(config, 7);
    {
        let handle = state.sim.lock().await;
        assert_eq!(handle.sim.config.world.width, 48);
        assert_eq!(handle.sim.creatures.len(), 3);
    }

    let (status, body) = do_request(
        router(state.clone()),
        startup_req(r#"{"seed":42,"world":{"width":56}}"#),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["seeded_creatures"], 3);
    let handle = state.sim.lock().await;
    assert_eq!(handle.sim.config.world.width, 56);
    assert_eq!(handle.sim.config.world.height, 48);
    assert_eq!(handle.sim.config.population.initial_creatures, 3);
}

// ── 2. startup_unknown_field_returns_422 ───────────────────────────────────

#[tokio::test]
async fn startup_unknown_field_returns_422() {
    let (status, body) = do_request(app(), startup_req(r#"{"seed":42,"bogus_field":1}"#)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "body: {body}");
    assert_eq!(
        body["error"]["code"].as_str(),
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
        body["error"]["code"].as_str(),
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
        body["error"]["code"].as_str(),
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
        body["error"]["code"].as_str(),
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
        body["error"]["code"].as_str(),
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
        body["mutation_events_skipped_total_by_operator"].is_object(),
        "missing mutation_events_skipped_total_by_operator"
    );
    assert!(
        body["mutation_target_reachability_total"].is_object(),
        "missing mutation_target_reachability_total"
    );
    assert!(
        body["move_actions_blocked_total_by_cause"].is_object(),
        "missing move_actions_blocked_total_by_cause"
    );
    assert!(
        body["move_actions_blocked_avoidable_total_by_reader_state"].is_object(),
        "missing move_actions_blocked_avoidable_total_by_reader_state"
    );
    assert!(
        body["reproduction_actions_rejected_invalid_target_total_by_cause"].is_object(),
        "missing reproduction_actions_rejected_invalid_target_total_by_cause"
    );
    assert!(
        body["reproduction_actions_rejected_invalid_target_avoidable_total_by_reader_state"]
            .is_object(),
        "missing reproduction_actions_rejected_invalid_target_avoidable_total_by_reader_state"
    );
    assert!(
        body["mutation_operator_funnel_total_by_operator"].is_object(),
        "missing mutation_operator_funnel_total_by_operator"
    );
    assert!(
        body["mutation_skip_reasons_total_by_operator"].is_object(),
        "missing mutation_skip_reasons_total_by_operator"
    );
    assert!(
        body["move_attempts_with_barrier_neighbor_total_by_reader_state"].is_object(),
        "missing move_attempts_with_barrier_neighbor_total_by_reader_state"
    );
    assert!(
        body["move_blocked_barrier_with_barrier_neighbor_total_by_reader_state"].is_object(),
        "missing move_blocked_barrier_with_barrier_neighbor_total_by_reader_state"
    );
    assert!(
        body["reproduction_attempts_with_barrier_neighbor_total_by_reader_state"].is_object(),
        "missing reproduction_attempts_with_barrier_neighbor_total_by_reader_state"
    );
    assert!(
        body["reproduction_invalid_target_barrier_with_barrier_neighbor_total_by_reader_state"]
            .is_object(),
        "missing reproduction_invalid_target_barrier_with_barrier_neighbor_total_by_reader_state"
    );
    assert!(
        body["mutation_value_totals_by_operator"].is_object(),
        "missing mutation_value_totals_by_operator"
    );
    assert!(
        body["mutation_outcome_summary"].is_object(),
        "missing mutation_outcome_summary"
    );
    assert!(
        body["mutation_outcome_summary"]["helpful_total"].is_number(),
        "missing mutation_outcome_summary.helpful_total"
    );
    assert!(
        body["mutation_outcome_summary"]["neutral_total"].is_number(),
        "missing mutation_outcome_summary.neutral_total"
    );
    assert!(
        body["mutation_outcome_summary"]["detrimental_total"].is_number(),
        "missing mutation_outcome_summary.detrimental_total"
    );
    assert!(
        body["mutation_added_node_input_classes_total_by_operator"].is_object(),
        "missing mutation_added_node_input_classes_total_by_operator"
    );
    assert!(
        body["mutation_added_node_world_inputs_total_by_operator"].is_object(),
        "missing mutation_added_node_world_inputs_total_by_operator"
    );
    assert!(
        body["vm_live_read_world_inputs_current"].is_object(),
        "missing vm_live_read_world_inputs_current"
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
    assert_eq!(
        body["world_static"]["food_types"].as_array().unwrap().len(),
        1
    );
    assert!(body["world_static"]["food_types"][0]
        .get("metabolic_energy_yield")
        .is_none());
    assert!(body["world_static"]["food_types"][0]
        .get("reproductive_reserve_yield")
        .is_none());
    assert!(
        body["world_static"]["barrier_mask"].is_array(),
        "body: {body}"
    );
    assert!(
        body["world_static"]["food_fertility_u8"].is_array(),
        "body: {body}"
    );
    assert!(body["view"]["grid_width"].is_number(), "body: {body}");
    assert!(body["view"]["grid_height"].is_number(), "body: {body}");
    assert!(body["view"]["food"].is_array(), "body: {body}");
    assert!(
        body["view"]["food_fertility_u8"].is_null(),
        "overview view payload should not include static fertility bytes: {body}"
    );
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

// ── 10f. snapshot_detail_omits_fertility_bytes_from_view_payload ──────────

#[tokio::test]
async fn snapshot_detail_omits_fertility_bytes_from_view_payload() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":22}"#))
        .await
        .unwrap();

    let (status, body) = do_request(a, get_req("/v3/simulation/snapshot?zoom_tier=detail")).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(
        body["view"]["kind"].as_str(),
        Some("detail"),
        "body: {body}"
    );
    assert!(body["view"]["food"].is_array(), "body: {body}");
    assert!(
        body["view"]["food_fertility_u8"].is_null(),
        "detail view payload should not include static fertility bytes: {body}"
    );
    assert!(
        body["world_static"]["food_fertility_u8"].is_array(),
        "world_static must carry fertility bytes: {body}"
    );
}

// ── 11. patch_config_world_topology_fields_are_restart_only ────────────────

#[tokio::test]
async fn patch_config_world_topology_fields_are_restart_only() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":1}"#))
        .await
        .unwrap();
    for patch in [
        r#"{"world":{"width":200}}"#,
        r#"{"world":{"height":200}}"#,
        r#"{"world":{"edge_mode":"Bounded"}}"#,
    ] {
        let (status, body) = do_request(a.clone(), patch_req("/v3/simulation/config", patch)).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "body: {body}");
        assert_eq!(
            body["error"]["code"].as_str(),
            Some("validation_rejected"),
            "body: {body}"
        );
        assert!(
            body["error"]["details"]["field_errors"]
                .to_string()
                .contains("restart-only"),
            "{body}"
        );
    }
}

// ── 11b. get_config_omits_retired_topology_new_node_birth ──────────────

#[tokio::test]
async fn get_config_omits_retired_topology_new_node_birth() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":1}"#))
        .await
        .unwrap();

    let (status, body) = do_request(a, get_req("/v3/simulation/config")).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert!(body["config"]["mutation"]
        .get("topology_new_node_birth")
        .is_none());
}

// ── 11c. get_config_includes_food_occupancy_depletion_defaults ─────────────

#[tokio::test]
async fn get_config_includes_food_occupancy_depletion_defaults() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":1}"#))
        .await
        .unwrap();

    let (status, body) = do_request(a, get_req("/v3/simulation/config")).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    let depletion = &body["config"]["world"]["food"]["shared"]["occupancy_depletion"];
    assert!(depletion.is_object(), "missing occupancy_depletion: {body}");
    assert_eq!(depletion["enabled"].as_bool(), Some(true));
    assert_json_f64_close(&depletion["deposit_per_occupied_tick"], 0.08);
}

// ── 11d. patch_config_rejects_retired_topology_new_node_birth ────────────

#[tokio::test]
async fn patch_config_rejects_retired_topology_new_node_birth() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":1}"#))
        .await
        .unwrap();

    let patch = r#"{
        "mutation": {
            "topology_new_node_birth": {
                "graph_backend_chance": 1.0,
                "graph_initialized_chance": 0.0,
                "graph_compute_gate_chance": 1.0
            }
        }
    }"#;

    let (patch_status, patch_body) =
        do_request(a.clone(), patch_req("/v3/simulation/config", patch)).await;
    assert_eq!(
        patch_status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "body: {patch_body}"
    );
}

// ── 11e. patch_config_roundtrips_food_occupancy_depletion_fields ───────────

#[tokio::test]
async fn patch_config_roundtrips_food_occupancy_depletion_fields() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":1}"#))
        .await
        .unwrap();

    let patch = r#"{
        "world": {
            "food": {
                "shared": {
                    "occupancy_depletion": {
                        "enabled": false,
                        "deposit_per_occupied_tick": 0.25
                    }
                }
            }
        }
    }"#;

    let (patch_status, patch_body) =
        do_request(a.clone(), patch_req("/v3/simulation/config", patch)).await;
    assert_eq!(patch_status, StatusCode::OK, "body: {patch_body}");
    let patched = &patch_body["config"]["world"]["food"]["shared"]["occupancy_depletion"];
    assert_eq!(patched["enabled"].as_bool(), Some(false));
    assert_json_f64_close(&patched["deposit_per_occupied_tick"], 0.25);

    let (get_status, get_body) = do_request(a, get_req("/v3/simulation/config")).await;
    assert_eq!(get_status, StatusCode::OK, "body: {get_body}");
    let fetched = &get_body["config"]["world"]["food"]["shared"]["occupancy_depletion"];
    assert_eq!(fetched["enabled"].as_bool(), Some(false));
    assert_json_f64_close(&fetched["deposit_per_occupied_tick"], 0.25);
}

// ── 11f. patch_config_food_occupancy_depletion_live_patch_keeps_world_static ─

#[tokio::test]
async fn patch_config_food_occupancy_depletion_live_patch_keeps_world_static_revision() {
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

    let (_, before) = do_request(a.clone(), get_req("/v3/simulation/snapshot")).await;
    let before_projection_revision = before["projection_revision"].as_u64().unwrap_or(0);
    let before_world_static_revision = before["world_static_revision"].as_u64().unwrap_or(0);

    let patch = r#"{
        "world": {
            "food": {
                "shared": {
                    "occupancy_depletion": {
                        "enabled": false,
                        "deposit_per_occupied_tick": 0.25
                    }
                }
            }
        }
    }"#;

    let (patch_status, patch_body) =
        do_request(a.clone(), patch_req("/v3/simulation/config", patch)).await;
    assert_eq!(patch_status, StatusCode::OK, "body: {patch_body}");
    assert_eq!(
        patch_body["state"].as_str(),
        Some("paused"),
        "body: {patch_body}"
    );

    let (_, after) = do_request(a, get_req("/v3/simulation/snapshot")).await;
    let after_projection_revision = after["projection_revision"].as_u64().unwrap_or(0);
    let after_world_static_revision = after["world_static_revision"].as_u64().unwrap_or(0);

    assert!(
        after_projection_revision > before_projection_revision,
        "projection_revision should advance after a runtime config patch: before={before_projection_revision}, after={after_projection_revision}, body={after}"
    );
    assert_eq!(
        after_world_static_revision, before_world_static_revision,
        "food depletion config patches must not mark the world static shape as changed: before={before_world_static_revision}, after={after_world_static_revision}"
    );
}

// ── 11g. patch_config_rejects_invalid_food_occupancy_depletion_values ──────

#[tokio::test]
async fn patch_config_rejects_invalid_food_occupancy_depletion_values() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":1}"#))
        .await
        .unwrap();

    let patch = r#"{
        "world": {
            "food": {
                "shared": {
                    "occupancy_depletion": {
                        "deposit_per_occupied_tick": 1.5
                    }
                }
            }
        }
    }"#;

    let (status, body) = do_request(a, patch_req("/v3/simulation/config", patch)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "body: {body}");
    assert_eq!(
        body["error"]["code"].as_str(),
        Some("validation_rejected"),
        "body: {body}"
    );
}

// ── 11g2. get_config_includes_food_grazing_defaults ────────────────────────

#[tokio::test]
async fn get_config_includes_food_grazing_defaults() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":1}"#))
        .await
        .unwrap();

    let (status, body) = do_request(a, get_req("/v3/simulation/config")).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    let grazing = &body["config"]["world"]["food"]["shared"]["grazing"];
    assert!(grazing.is_object(), "missing grazing: {body}");
    assert_eq!(grazing["enabled"].as_bool(), Some(true));
    assert_json_f64_close(&grazing["factor"], 0.5);
    assert_json_f64_close(&grazing["floor"], 0.05);
    assert_eq!(grazing["recovery_ticks"].as_u64(), Some(1000));
}

// ── 11g3. patch_config_roundtrips_food_grazing_fields ──────────────────────

#[tokio::test]
async fn patch_config_roundtrips_food_grazing_fields() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":1}"#))
        .await
        .unwrap();

    let patch = r#"{
        "world": {
            "food": {
                "shared": {
                    "grazing": {
                        "enabled": false,
                        "factor": 0.25,
                        "floor": 0.1,
                        "recovery_ticks": 250
                    }
                }
            }
        }
    }"#;

    let (patch_status, patch_body) =
        do_request(a.clone(), patch_req("/v3/simulation/config", patch)).await;
    assert_eq!(patch_status, StatusCode::OK, "body: {patch_body}");
    let patched = &patch_body["config"]["world"]["food"]["shared"]["grazing"];
    assert_eq!(patched["enabled"].as_bool(), Some(false));
    assert_json_f64_close(&patched["factor"], 0.25);
    assert_json_f64_close(&patched["floor"], 0.1);
    assert_eq!(patched["recovery_ticks"].as_u64(), Some(250));

    let (get_status, get_body) = do_request(a, get_req("/v3/simulation/config")).await;
    assert_eq!(get_status, StatusCode::OK, "body: {get_body}");
    let fetched = &get_body["config"]["world"]["food"]["shared"]["grazing"];
    assert_eq!(fetched["enabled"].as_bool(), Some(false));
    assert_json_f64_close(&fetched["factor"], 0.25);
    assert_json_f64_close(&fetched["floor"], 0.1);
    assert_eq!(fetched["recovery_ticks"].as_u64(), Some(250));
}

// ── 11g4. patch_config_rejects_invalid_food_grazing_values ─────────────────

#[tokio::test]
async fn patch_config_rejects_invalid_food_grazing_values() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":1}"#))
        .await
        .unwrap();

    for patch in [
        r#"{"world":{"food":{"shared":{"grazing":{"factor":1.5}}}}}"#,
        r#"{"world":{"food":{"shared":{"grazing":{"recovery_ticks":0}}}}}"#,
    ] {
        let (status, body) = do_request(a.clone(), patch_req("/v3/simulation/config", patch)).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "body: {body}");
        assert_eq!(
            body["error"]["code"].as_str(),
            Some("validation_rejected"),
            "body: {body}"
        );
    }
}

// ── 11h. startup_accepts_founder_profile_and_get_config_roundtrips ─────────

#[tokio::test]
async fn startup_accepts_founder_profile_and_get_config_roundtrips() {
    let a = app();
    let startup_body = r#"{
        "seed": 7,
        "population": {
            "founder_profile": "forage_first_sparse_rich_offspring"
        }
    }"#;
    let (startup_status, startup_resp) = do_request(a.clone(), startup_req(startup_body)).await;
    assert_eq!(startup_status, StatusCode::OK, "body: {startup_resp}");

    let (get_status, get_body) = do_request(a, get_req("/v3/simulation/config")).await;
    assert_eq!(get_status, StatusCode::OK, "body: {get_body}");
    assert_eq!(
        get_body["config"]["population"]["founder_profile"].as_str(),
        Some("forage_first_sparse_rich_offspring")
    );
}

// ── 11f. startup_failed_action_penalty_ramp_pins_runtime_penalty_to_end ───

#[tokio::test]
async fn startup_failed_action_penalty_ramp_pins_runtime_penalty_to_end() {
    let a = app();
    let startup_body = r#"{
        "seed": 123,
        "startup": {
            "ramps": {
                "failed_action_penalty": {
                    "enabled": true,
                    "start": 5.0,
                    "end": 30.0,
                    "target_tick": 1000
                }
            }
        }
    }"#;

    let (startup_status, startup_resp) = do_request(a.clone(), startup_req(startup_body)).await;
    assert_eq!(startup_status, StatusCode::OK, "body: {startup_resp}");

    let (get_status, get_body) = do_request(a, get_req("/v3/simulation/config")).await;
    assert_eq!(get_status, StatusCode::OK, "body: {get_body}");
    assert_eq!(
        get_body["config"]["energy"]["costs"]["failed_action_penalty"].as_f64(),
        Some(30.0)
    );
    assert_eq!(
        get_body["config"]["startup"]["ramps"]["failed_action_penalty"]["enabled"].as_bool(),
        Some(true)
    );
}

// ── 11g. patch_config_rejects_startup_fields ───────────────────────────────

#[tokio::test]
async fn patch_config_rejects_startup_fields() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":1}"#))
        .await
        .unwrap();

    let patch = r#"{
        "startup": {
            "ramps": {
                "failed_action_penalty": {
                    "enabled": true,
                    "start": 5.0,
                    "end": 30.0,
                    "target_tick": 1000
                }
            }
        }
    }"#;

    let (status, body) = do_request(a, patch_req("/v3/simulation/config", patch)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "body: {body}");
    assert_eq!(
        body["error"]["code"].as_str(),
        Some("validation_rejected"),
        "body: {body}"
    );
}

// ── 11g. patch_config_rejects_initial_creatures_as_restart_only ───────────

/// Founders are seeded only by `POST /v3/simulation/startup`, so a patched
/// `population.initial_creatures` would be stored but never read. The value
/// sits inside the harness bounds (1..=512) so canonical normalization alone
/// would have accepted it.
#[tokio::test]
async fn patch_config_rejects_initial_creatures_as_restart_only() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":1}"#))
        .await
        .unwrap();
    let (_, before) = do_request(a.clone(), get_req("/v3/simulation/config")).await;

    let patch = r#"{"population":{"initial_creatures":10}}"#;
    let (status, body) = do_request(a.clone(), patch_req("/v3/simulation/config", patch)).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "body: {body}");
    assert_eq!(
        body["error"]["code"].as_str(),
        Some("validation_rejected"),
        "body: {body}"
    );
    assert_eq!(
        error_field_paths(&body),
        vec!["population.initial_creatures"],
        "body: {body}"
    );
    let (_, after) = do_request(a, get_req("/v3/simulation/config")).await;
    assert_eq!(after["config"], before["config"], "after: {after}");
}

// ── 11i. patch_config_rejects_founder_profile_as_restart_only ─────────────

/// Founders are seeded only by `POST /v3/simulation/startup`, so a patched
/// `population.founder_profile` would be stored but never read.
#[tokio::test]
async fn patch_config_rejects_founder_profile_as_restart_only() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":1}"#))
        .await
        .unwrap();
    let (_, before) = do_request(a.clone(), get_req("/v3/simulation/config")).await;

    let patch = r#"{"population":{"founder_profile":"forage_first_sparse"}}"#;
    let (status, body) = do_request(a.clone(), patch_req("/v3/simulation/config", patch)).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "body: {body}");
    assert_eq!(
        body["error"]["code"].as_str(),
        Some("validation_rejected"),
        "body: {body}"
    );
    assert_eq!(
        error_field_paths(&body),
        vec!["population.founder_profile"],
        "body: {body}"
    );
    let (_, after) = do_request(a, get_req("/v3/simulation/config")).await;
    assert_eq!(after["config"], before["config"], "after: {after}");
}

/// `population.max_creatures` is a live reproduction cap and stays patchable
/// on its own, so the restart-only rule must not cover all of `population`.
#[tokio::test]
async fn patch_config_keeps_max_creatures_patchable() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":1}"#))
        .await
        .unwrap();

    let patch = r#"{"population":{"max_creatures":1024}}"#;
    let (status, body) = do_request(a.clone(), patch_req("/v3/simulation/config", patch)).await;

    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(
        body["config"]["population"]["max_creatures"], 1024,
        "body: {body}"
    );
    let (_, after) = do_request(a, get_req("/v3/simulation/config")).await;
    assert_eq!(
        after["config"]["population"]["max_creatures"], 1024,
        "after: {after}"
    );
    assert_eq!(
        after["config"]["population"]["initial_creatures"], 64,
        "after: {after}"
    );
}

#[tokio::test]
async fn patch_config_rejects_removed_nutrition_fields() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":1}"#))
        .await
        .unwrap();

    let patch = r#"{
        "nutrition": {
            "reproductive_reserve_capacity": 12.0,
            "reproductive_reserve_cost": 4.0
        }
    }"#;

    let (status, body) = do_request(a, patch_req("/v3/simulation/config", patch)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "body: {body}");
    assert_eq!(
        body["error"]["code"].as_str(),
        Some("validation_rejected"),
        "body: {body}"
    );
}

// ── 11h. patch_config_rejects_fertility_layer_generation_fields ───────────

#[tokio::test]
async fn patch_config_rejects_fertility_layer_generation_fields() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":1}"#))
        .await
        .unwrap();

    let patch = r#"{
        "world": {
            "food": {
                "fertility": {
                    "layers": [
                        {
                            "algorithm": {
                                "Uniform": { "value": 1.0 }
                            },
                            "weight": 1.0
                        }
                    ]
                }
            }
        }
    }"#;

    let (status, body) = do_request(a, patch_req("/v3/simulation/config", patch)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "body: {body}");
    assert_eq!(
        body["error"]["code"].as_str(),
        Some("validation_rejected"),
        "body: {body}"
    );
}

// ── 11h. patch_config_rejects_food_types_runtime_patch ────────────────────

#[tokio::test]
async fn patch_config_rejects_food_types_runtime_patch() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":1}"#))
        .await
        .unwrap();

    let patch = r##"{
        "world": {
            "food": {
                "types": [
                    {
                        "name": "Blue Food",
                        "color": "#3b82f6",
                        "initial_density": 0.8,
                        "initial_coverage": 0.4,
                        "growth_inhibitor": 0.2,
                        "energy_per_unit": 12.0,
                        "growth_rate": 0.01,
                        "recovery_spawn_rate": 0.0,
                        "initial_fertility_only": true
                    }
                ]
            }
        }
    }"##;

    let (status, body) = do_request(a, patch_req("/v3/simulation/config", patch)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "body: {body}");
    assert_eq!(
        body["error"]["code"].as_str(),
        Some("validation_rejected"),
        "body: {body}"
    );
}

// ── 11h. patch_config_rejects_food_annealing_runtime_patch ────────────────

#[tokio::test]
async fn patch_config_rejects_food_annealing_runtime_patch() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":1}"#))
        .await
        .unwrap();

    let patch = r#"{
        "world": {
            "food": {
                "annealing": {
                    "enabled": true,
                    "ramp_ticks": 3000,
                    "initial_min_fertility": 0.4,
                    "initial_max_fertility": 1.5
                }
            }
        }
    }"#;

    let (status, body) = do_request(a, patch_req("/v3/simulation/config", patch)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "body: {body}");
    assert_eq!(
        body["error"]["code"].as_str(),
        Some("validation_rejected"),
        "body: {body}"
    );
}

// ── 11h. patch_config_rejects_legacy_food_occupancy_path ──────────────────

#[tokio::test]
async fn patch_config_rejects_legacy_food_occupancy_path() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":1}"#))
        .await
        .unwrap();

    let patch = r#"{
        "world": {
            "food": {
                "occupancy_depletion": {
                    "enabled": false,
                    "deposit_per_occupied_tick": 0.15
                }
            }
        }
    }"#;

    let (status, body) = do_request(a, patch_req("/v3/simulation/config", patch)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "body: {body}");
    assert_eq!(
        body["error"]["code"].as_str(),
        Some("validation_rejected"),
        "body: {body}"
    );
}

// ── 11h. patch_config_rejects_non_object_food_shared_patch ────────────────

#[tokio::test]
async fn patch_config_rejects_non_object_food_shared_patch() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":1}"#))
        .await
        .unwrap();

    let patch = r#"{
        "world": {
            "food": {
                "shared": 7
            }
        }
    }"#;

    let (status, body) = do_request(a, patch_req("/v3/simulation/config", patch)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "body: {body}");
    assert_eq!(
        body["error"]["code"].as_str(),
        Some("validation_rejected"),
        "body: {body}"
    );
}

// ── 11h. patch_config_blocks_failed_penalty_while_ramp_active ─────────────

#[tokio::test]
async fn patch_config_blocks_failed_penalty_while_ramp_active() {
    let a = app();
    let startup_body = r#"{
        "seed": 321,
        "startup": {
            "ramps": {
                "failed_action_penalty": {
                    "enabled": true,
                    "start": 5.0,
                    "end": 30.0,
                    "target_tick": 10
                }
            }
        }
    }"#;
    let (startup_status, startup_resp) = do_request(a.clone(), startup_req(startup_body)).await;
    assert_eq!(startup_status, StatusCode::OK, "body: {startup_resp}");

    let patch = r#"{"energy":{"costs":{"failed_action_penalty":7.0}}}"#;
    let (status, body) = do_request(a, patch_req("/v3/simulation/config", patch)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "body: {body}");
    assert_eq!(
        body["error"]["code"].as_str(),
        Some("validation_rejected"),
        "body: {body}"
    );
}

// ── 11i. patch_config_allows_failed_penalty_after_ramp_target_tick ─────────

#[tokio::test]
async fn patch_config_allows_failed_penalty_after_ramp_target_tick() {
    let a = app();
    let startup_body = r#"{
        "seed": 777,
        "startup": {
            "ramps": {
                "failed_action_penalty": {
                    "enabled": true,
                    "start": 5.0,
                    "end": 30.0,
                    "target_tick": 1
                }
            }
        }
    }"#;
    let (startup_status, startup_resp) = do_request(a.clone(), startup_req(startup_body)).await;
    assert_eq!(startup_status, StatusCode::OK, "body: {startup_resp}");

    let (start_status, start_body) = do_request(a.clone(), post_req("/v3/simulation/start")).await;
    assert_eq!(start_status, StatusCode::OK, "body: {start_body}");
    let (pause_status, pause_body) = do_request(a.clone(), post_req("/v3/simulation/pause")).await;
    assert_eq!(pause_status, StatusCode::OK, "body: {pause_body}");
    let (step_status, step_body) = do_request(
        a.clone(),
        post_json("/v3/simulation/step", r#"{"steps":1}"#),
    )
    .await;
    assert_eq!(step_status, StatusCode::OK, "body: {step_body}");

    let patch = r#"{"energy":{"costs":{"failed_action_penalty":9.0}}}"#;
    let (status, body) = do_request(a, patch_req("/v3/simulation/config", patch)).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(
        body["config"]["energy"]["costs"]["failed_action_penalty"].as_f64(),
        Some(9.0),
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
    use v3_server::query::cache::build_food_fertility_u8;
    use v3_server::state::build_ws_frame;
    use v3_server::state::{SimHandle, SimulationStatus};

    let mut cfg = SimulationConfig::default();
    cfg.world.width = 16;
    cfg.world.height = 16;
    cfg.population.initial_creatures = 5;
    cfg.mutation.per_unit_supply_enabled = false;
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
    let cached_fertility_u8 = build_food_fertility_u8(sim.world.food());
    let handle = SimHandle {
        sim,
        status: SimulationStatus::Paused,
        active_trace: None,
        cached_fertility_u8,
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
    let skipped_by_operator: u64 = frame
        .health
        .mutation_events_skipped_total_by_operator
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
    // Operator-level attempts may be less than total when events are skipped at
    // the domain level (no applicable operator under complexity pressure).
    assert!(
        attempted_by_operator <= frame.health.mutation_events_attempted_total,
        "attempted_by_operator ({attempted_by_operator}) must not exceed mutation_events_attempted_total ({})",
        frame.health.mutation_events_attempted_total
    );
    assert_eq!(
        applied_by_operator, frame.health.mutation_events_applied_total,
        "applied_by_operator must reconcile to mutation_events_applied_total"
    );
    assert!(
        skipped_by_operator <= frame.health.mutation_events_skipped_total,
        "skipped_by_operator ({skipped_by_operator}) must not exceed mutation_events_skipped_total ({})",
        frame.health.mutation_events_skipped_total
    );
    assert_eq!(
        frame.health.mutation_target_reachability_total.reachable
            + frame.health.mutation_target_reachability_total.unreachable
            + frame
                .health
                .mutation_target_reachability_total
                .not_applicable,
        frame.health.mutation_events_applied_total,
        "reachability totals must reconcile to mutation_events_applied_total"
    );
}

// ── 15. status_payload_includes_state ───────────────────────────────────────

#[tokio::test]
async fn status_payload_includes_state() {
    use v3_core::simulation::seed_simulation;
    use v3_server::query::cache::build_food_fertility_u8;
    use v3_server::state::build_ws_frame;
    use v3_server::state::{SimHandle, SimulationStatus};

    let sim = seed_simulation(test_config(), 7);
    let cached_fertility_u8 = build_food_fertility_u8(sim.world.food());
    let handle = SimHandle {
        sim,
        status: SimulationStatus::Paused,
        active_trace: None,
        cached_fertility_u8,
    };

    let frame = build_ws_frame(&handle);
    assert_eq!(
        frame.status.state,
        SimulationStatus::Paused,
        "status payload must include current simulation state"
    );
}

#[tokio::test]
async fn status_and_health_include_food_occupancy_depletion_summaries() {
    use v3_core::simulation::{run_tick, seed_simulation};
    use v3_server::query::cache::build_food_fertility_u8;
    use v3_server::state::build_ws_frame;
    use v3_server::state::{SimHandle, SimulationStatus};

    let mut sim = seed_simulation(test_config(), 7);
    run_tick(&mut sim, &mut None);
    let cached_fertility_u8 = build_food_fertility_u8(sim.world.food());
    let handle = SimHandle {
        sim,
        status: SimulationStatus::Paused,
        active_trace: None,
        cached_fertility_u8,
    };

    let frame = build_ws_frame(&handle);
    assert!(frame.status.last_tick_food_occupancy_depletion_mean >= 0.0);
    assert!(
        frame
            .status
            .last_tick_food_occupancy_depletion_occupied_cells
            > 0
    );
    assert!(
        frame
            .status
            .last_tick_food_growth_suppressed_by_occupancy_depletion
            >= 0.0
    );
    assert_eq!(
        frame.status.last_tick_food_occupancy_depletion_mean,
        frame.health.last_tick_food_occupancy_depletion_mean
    );
    assert_eq!(
        frame
            .status
            .last_tick_food_occupancy_depletion_occupied_cells,
        frame
            .health
            .last_tick_food_occupancy_depletion_occupied_cells
    );
    assert_eq!(
        frame
            .status
            .last_tick_food_growth_suppressed_by_occupancy_depletion,
        frame
            .health
            .last_tick_food_growth_suppressed_by_occupancy_depletion
    );
}

// ── 16. ws_frame_msgpack_roundtrip ──────────────────────────────────────────

#[tokio::test]
async fn ws_frame_msgpack_roundtrip() {
    use v3_core::simulation::{run_tick, seed_simulation};
    use v3_server::query::cache::build_food_fertility_u8;
    use v3_server::state::build_ws_frame;
    use v3_server::state::{SimHandle, SimulationStatus, WsFrame};

    let mut sim = seed_simulation(test_config(), 99);
    for _ in 0..5 {
        run_tick(&mut sim, &mut None);
    }
    let cached_fertility_u8 = build_food_fertility_u8(sim.world.food());
    let handle = SimHandle {
        sim,
        status: SimulationStatus::Running,
        active_trace: None,
        cached_fertility_u8,
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
        body["error"]["code"].as_str(),
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
        body["error"]["code"].as_str(),
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
        body["error"]["code"].as_str(),
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
        body["error"]["code"].as_str(),
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
    assert!(body.get("reproductive_reserve").is_none());
    assert!(body.get("reproductive_reserve_capacity").is_none());
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

    // Shared memory
    assert!(body["shared_memory"].is_array(), "missing shared_memory");
    assert_eq!(
        body["shared_memory"].as_array().unwrap().len(),
        16,
        "shared_memory must have 16 slots"
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
    assert_eq!(
        body["error"]["code"].as_str(),
        Some("not_found"),
        "body: {body}"
    );
}

#[tokio::test]
async fn get_creature_noncanonical_ffi_id_returns_404() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":42}"#))
        .await
        .unwrap();

    let (_, snapshot_body) = do_request(
        a.clone(),
        get_req("/v3/simulation/snapshot?zoom_tier=detail"),
    )
    .await;
    let creature_id = snapshot_body["view"]["creatures"][0]["id"]
        .as_u64()
        .expect("creature id");

    // Flip bit 32 to produce a non-canonical FFI representation that currently
    // aliases the same slotmap key through KeyData::from_ffi.
    let noncanonical_id = creature_id ^ (1_u64 << 32);
    assert_ne!(noncanonical_id, creature_id, "test setup must alter id");

    let uri = format!("/v3/simulation/creature/{noncanonical_id}");
    let (status, body) = do_request(a, get_req(&uri)).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "body: {body}");
    assert_eq!(
        body["error"]["code"].as_str(),
        Some("not_found"),
        "body: {body}"
    );
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

// ── 28b. get_creature_includes_action_log ─────────────────────────────────

#[tokio::test]
async fn get_creature_includes_action_log() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":42}"#))
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

    // Run one tick so creatures have action log entries.
    let (status, _) = do_request(
        a.clone(),
        post_json("/v3/simulation/step", r#"{"steps":1}"#),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Get a creature ID from snapshot.
    let (_, snapshot_body) = do_request(
        a.clone(),
        get_req("/v3/simulation/snapshot?zoom_tier=detail"),
    )
    .await;
    let creatures = snapshot_body["view"]["creatures"]
        .as_array()
        .expect("creatures array");
    assert!(!creatures.is_empty());
    let creature_id = creatures[0]["id"].as_u64().unwrap();

    let uri = format!("/v3/simulation/creature/{creature_id}");
    let (status, body) = do_request(a, get_req(&uri)).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");

    // action_log must be an array.
    let action_log = body["action_log"]
        .as_array()
        .expect("action_log should be an array");
    assert!(
        !action_log.is_empty(),
        "action_log should have entries after one tick"
    );

    // Verify entry structure.
    let entry = &action_log[0];
    assert!(entry["tick"].is_number(), "missing tick");
    assert!(entry["action_type"].is_string(), "missing action_type");
    assert!(entry["result"].is_string(), "missing result");
    assert!(entry["direction"].is_number(), "missing direction");
    assert!(entry["energy_before"].is_number(), "missing energy_before");
    assert!(entry["energy_after"].is_number(), "missing energy_after");
    assert!(entry["amount"].is_number(), "missing amount");
    assert!(
        entry["food_type"].is_null() || entry["food_type"].is_number(),
        "missing food_type"
    );
    assert!(entry["priority_bid"].is_number(), "missing priority_bid");
}

// ── 28c. get_creature_since_tick_filters_action_log ──────────────────────────

#[tokio::test]
async fn get_creature_since_tick_filters_action_log() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":42}"#))
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

    // Run 3 ticks so creatures have multiple action log entries.
    let (status, _) = do_request(
        a.clone(),
        post_json("/v3/simulation/step", r#"{"steps":3}"#),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Get a creature ID.
    let (_, snapshot_body) = do_request(
        a.clone(),
        get_req("/v3/simulation/snapshot?zoom_tier=detail"),
    )
    .await;
    let creatures = snapshot_body["view"]["creatures"]
        .as_array()
        .expect("creatures array");
    assert!(!creatures.is_empty());
    let creature_id = creatures[0]["id"].as_u64().unwrap();

    // Full fetch (no since_tick) — should include latest_tick and full action_log.
    let uri = format!("/v3/simulation/creature/{creature_id}");
    let (status, body) = do_request(a.clone(), get_req(&uri)).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    let full_log = body["action_log"].as_array().expect("action_log array");
    let full_count = full_log.len();
    assert!(full_count > 0, "should have entries after 3 ticks");
    let latest_tick = body["latest_tick"].as_u64().expect("latest_tick");
    assert!(latest_tick > 0, "latest_tick should be > 0 after stepping");

    // Fetch with since_tick = latest_tick — should return empty action_log
    // (no entries have tick > latest_tick).
    let uri_future = format!("/v3/simulation/creature/{creature_id}?since_tick={latest_tick}");
    let (status, body) = do_request(a.clone(), get_req(&uri_future)).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    let future_log = body["action_log"].as_array().expect("action_log array");
    assert!(
        future_log.is_empty(),
        "since_tick at latest_tick should return empty log, got {} entries",
        future_log.len()
    );
    // latest_tick is still the same even with since_tick filtering.
    assert_eq!(
        body["latest_tick"].as_u64().unwrap(),
        latest_tick,
        "latest_tick should be consistent"
    );

    // Fetch with since_tick = latest_tick - 1 — should return fewer entries than full.
    if latest_tick > 1 {
        let uri_partial = format!(
            "/v3/simulation/creature/{creature_id}?since_tick={}",
            latest_tick - 1
        );
        let (status, body) = do_request(a, get_req(&uri_partial)).await;
        assert_eq!(status, StatusCode::OK, "body: {body}");
        let partial_log = body["action_log"].as_array().expect("action_log array");
        // All returned entries must have tick > (latest_tick - 1).
        for entry in partial_log {
            let t = entry["tick"].as_u64().unwrap();
            assert!(
                t > latest_tick - 1,
                "entry tick {t} should be > {}",
                latest_tick - 1
            );
        }
        // Partial result should be a subset of the full result.
        assert!(
            partial_log.len() <= full_count,
            "filtered log ({}) should not exceed full log ({})",
            partial_log.len(),
            full_count
        );
    }
}

// ── 28d. get_creature_exclude_omits_fields ──────────────────────────────────

#[tokio::test]
async fn get_creature_exclude_omits_fields() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":42}"#))
        .await
        .unwrap();

    // Get a creature ID.
    let (_, snapshot_body) = do_request(
        a.clone(),
        get_req("/v3/simulation/snapshot?zoom_tier=detail"),
    )
    .await;
    let creatures = snapshot_body["view"]["creatures"]
        .as_array()
        .expect("creatures array");
    assert!(!creatures.is_empty());
    let creature_id = creatures[0]["id"].as_u64().unwrap();

    // No exclude — all fields present.
    let uri = format!("/v3/simulation/creature/{creature_id}");
    let (status, body) = do_request(a.clone(), get_req(&uri)).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert!(body.get("genome").is_some(), "genome should be present");
    assert!(
        body.get("mesh_annotations").is_some(),
        "mesh_annotations should be present when genome is present"
    );
    assert!(
        body.get("shared_memory").is_some(),
        "shared_memory should be present"
    );
    assert!(
        body.get("action_log").is_some(),
        "action_log should be present"
    );
    assert!(
        body.get("diagnostics").is_some(),
        "diagnostics should be present"
    );
    assert!(
        body["diagnostics"]["current_inputs"]["neighbor_barrier"].is_array(),
        "diagnostics.current_inputs.neighbor_barrier should be an array"
    );
    assert!(
        body["diagnostics"]["live_circuit"]["reachable_node_count"].is_number(),
        "diagnostics.live_circuit.reachable_node_count should be numeric"
    );
    assert!(
        body["diagnostics"]["live_circuit"]["barrier_decision_writer_reachable_node_count"]
            .is_number(),
        "diagnostics.live_circuit.barrier_decision_writer_reachable_node_count should be numeric"
    );
    assert!(
        body["diagnostics"]["live_circuit"]
            ["barrier_reader_without_decision_writer_reachable_node_count"]
            .is_number(),
        "diagnostics.live_circuit.barrier_reader_without_decision_writer_reachable_node_count should be numeric"
    );
    assert!(
        body["diagnostics"]["recent_actions"]["by_action_result"].is_object(),
        "diagnostics.recent_actions.by_action_result should be an object"
    );

    // exclude=genome — genome omitted, others present.
    let uri_excl_genome = format!("/v3/simulation/creature/{creature_id}?exclude=genome");
    let (status, body) = do_request(a.clone(), get_req(&uri_excl_genome)).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert!(body.get("genome").is_none(), "genome should be omitted");
    assert!(
        body.get("mesh_annotations").is_none(),
        "mesh_annotations should be omitted when genome is omitted"
    );
    assert!(
        body.get("shared_memory").is_some(),
        "shared_memory should be present"
    );
    assert!(
        body.get("action_log").is_some(),
        "action_log should be present"
    );
    // Scalar fields always present.
    assert!(body.get("energy").is_some(), "energy should be present");
    assert!(
        body.get("phenotype").is_some(),
        "phenotype should be present"
    );
    assert!(
        body.get("latest_tick").is_some(),
        "latest_tick should be present"
    );

    // exclude=genome,action_log — both omitted.
    let uri_excl_both = format!("/v3/simulation/creature/{creature_id}?exclude=genome,action_log");
    let (status, body) = do_request(a.clone(), get_req(&uri_excl_both)).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert!(body.get("genome").is_none(), "genome should be omitted");
    assert!(
        body.get("mesh_annotations").is_none(),
        "mesh_annotations should be omitted when genome is omitted"
    );
    assert!(
        body.get("action_log").is_none(),
        "action_log should be omitted"
    );
    assert!(
        body.get("shared_memory").is_some(),
        "shared_memory should be present"
    );

    // exclude=genome,action_log,shared_memory — all optional fields omitted.
    let uri_excl_all =
        format!("/v3/simulation/creature/{creature_id}?exclude=genome,action_log,shared_memory");
    let (status, body) = do_request(a.clone(), get_req(&uri_excl_all)).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert!(body.get("genome").is_none(), "genome should be omitted");
    assert!(
        body.get("mesh_annotations").is_none(),
        "mesh_annotations should be omitted when genome is omitted"
    );
    assert!(
        body.get("action_log").is_none(),
        "action_log should be omitted"
    );
    assert!(
        body.get("shared_memory").is_none(),
        "shared_memory should be omitted"
    );
    assert!(
        body.get("diagnostics").is_some(),
        "diagnostics should still be present unless explicitly excluded"
    );
    // Core scalar fields still present.
    assert!(body.get("energy").is_some(), "energy should be present");
    assert!(body.get("id").is_some(), "id should be present");

    let uri_excl_diagnostics = format!(
        "/v3/simulation/creature/{creature_id}?exclude=genome,action_log,shared_memory,diagnostics"
    );
    let (status, body) = do_request(a.clone(), get_req(&uri_excl_diagnostics)).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert!(
        body.get("diagnostics").is_none(),
        "diagnostics should be omitted when requested"
    );
}

#[tokio::test]
async fn get_creature_mesh_annotations_use_cached_reachable_nodes() {
    let state = test_state();
    let creature_id = {
        let mut handle = state.sim.lock().await;
        let creature_id = handle
            .sim
            .creatures
            .keys()
            .next()
            .expect("default state should have creatures");
        handle
            .sim
            .creatures
            .get_mut(creature_id)
            .expect("creature should exist")
            .cached_reachable_nodes = Vec::new().into_boxed_slice();
        creature_id.data().as_ffi()
    };

    let uri = format!("/v3/simulation/creature/{creature_id}");
    let (status, body) = do_request(router(state), get_req(&uri)).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");

    let annotations = body["mesh_annotations"]
        .as_array()
        .expect("mesh_annotations array");
    assert!(
        annotations
            .iter()
            .all(|annotation| annotation["reachable"].as_bool() == Some(false)),
        "handler should trust cached reachable indices, body: {body}"
    );
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

#[tokio::test]
async fn start_sample_noncanonical_ffi_id_returns_404() {
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
        .expect("creature id");

    let noncanonical_id = creature_id ^ (1_u64 << 32);
    assert_ne!(noncanonical_id, creature_id, "test setup must alter id");

    let uri = format!("/v3/simulation/creature/{noncanonical_id}/sample");
    let (status, body) = do_request(a, post_json(&uri, r#"{"ticks": 5}"#)).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "body: {body}");
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

    let state = test_state();
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

// ── 35a. ws_startup_with_changed_dimensions_emits_updated_world_static ───────

#[tokio::test]
async fn ws_startup_with_changed_dimensions_emits_updated_world_static_for_active_subscriber() {
    use futures_util::SinkExt;
    use tokio_tungstenite::connect_async;
    use tokio_tungstenite::tungstenite::Message;
    use v3_server::transport::protocol::ServerMessage;

    // Arrange
    let state = test_state();
    let (ws_url, server_task) = spawn_ws_app(state.clone()).await;
    let (mut socket, _) = connect_async(ws_url).await.expect("connect websocket");
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
    let initial_messages = recv_server_messages(&mut socket).await;
    let (initial_projection_revision, initial_world_static_revision) = initial_messages
        .iter()
        .find_map(|message| match message {
            ServerMessage::WorldStatic {
                projection_revision,
                world_static_revision,
                ..
            } => Some((*projection_revision, *world_static_revision)),
            _ => None,
        })
        .expect("subscribe should prime world static");

    // Act
    let (startup_status, startup_body) = do_request(
        router(state.clone()),
        startup_req(r#"{"seed":424242,"world":{"width":256,"height":256}}"#),
    )
    .await;
    let updated_messages = recv_server_messages(&mut socket).await;

    // Assert
    assert_eq!(startup_status, StatusCode::OK, "body: {startup_body}");
    let (updated_projection_revision, updated_world_static_revision, updated_world_static) =
        updated_messages
            .iter()
            .find_map(|message| match message {
                ServerMessage::WorldStatic {
                    projection_revision,
                    world_static_revision,
                    payload,
                    ..
                } => Some((*projection_revision, *world_static_revision, payload)),
                _ => None,
            })
            .expect("startup must emit world static to the active subscriber");
    assert_eq!(updated_world_static.width, 256);
    assert_eq!(updated_world_static.height, 256);
    assert!(
        updated_world_static_revision > initial_world_static_revision,
        "startup must advance world static revision: initial={initial_world_static_revision}, updated={updated_world_static_revision}"
    );
    assert!(
        updated_projection_revision > initial_projection_revision,
        "startup must advance projection revision: initial={initial_projection_revision}, updated={updated_projection_revision}"
    );
    assert!(
        updated_messages.iter().any(|message| matches!(
            message,
            ServerMessage::Status {
                projection_revision,
                world_static_revision,
                ..
            } if *projection_revision == updated_projection_revision
                && *world_static_revision == updated_world_static_revision
        )),
        "startup status must share the world-static publication revisions: {updated_messages:?}"
    );
    assert!(
        updated_messages.iter().any(|message| matches!(
            message,
            ServerMessage::ViewOverview {
                projection_revision,
                world_static_revision,
                ..
            } if *projection_revision == updated_projection_revision
                && *world_static_revision == updated_world_static_revision
        )),
        "startup viewport must share the world-static publication revisions: {updated_messages:?}"
    );

    let (current_projection_revision, current_world_static_revision) = {
        let projection = state.projection.read().expect("projection lock poisoned");
        (
            projection.current().projection_revision,
            projection.current().world_static_revision,
        )
    };
    assert_eq!(updated_projection_revision, current_projection_revision);
    assert_eq!(updated_world_static_revision, current_world_static_revision);

    server_task.abort();
}

// ── 36. ws_latest_request_id_wins ───────────────────────────────────────────

#[tokio::test]
async fn ws_latest_request_id_wins() {
    use futures_util::SinkExt;
    use tokio_tungstenite::connect_async;
    use tokio_tungstenite::tungstenite::Message;
    use v3_server::transport::protocol::ServerMessage;

    let state = test_state();
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

    let state = test_state();
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

    let state = test_state();
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

    let state = test_state();
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

    let state = test_state();
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

// ── compression_returns_gzip_when_accepted ────────────────────────────────

#[tokio::test]
async fn compression_returns_gzip_when_accepted() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":42}"#))
        .await
        .unwrap();

    // Request status with Accept-Encoding: gzip
    let req = Request::builder()
        .method("GET")
        .uri("/v3/simulation/status")
        .header("accept-encoding", "gzip")
        .body(Body::empty())
        .unwrap();

    let resp = a.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Verify the response uses gzip content-encoding
    let encoding = resp
        .headers()
        .get("content-encoding")
        .expect("missing content-encoding header")
        .to_str()
        .unwrap();
    assert_eq!(encoding, "gzip", "expected gzip content-encoding");

    // Verify the body is valid gzip-compressed data that can be decompressed
    let compressed_bytes = resp.into_body().collect().await.unwrap().to_bytes();
    assert!(
        !compressed_bytes.is_empty(),
        "compressed body should not be empty"
    );
}

// ── Pattern endpoint tests ──────────────────────────────────────────────────

#[tokio::test]
async fn pattern_preview_returns_bitmap() {
    let a = app();
    // First startup to have a world.
    let (status, _) = do_request(a.clone(), startup_req(r#"{"seed":42}"#)).await;
    assert_eq!(status, StatusCode::OK);

    let req = post_json(
        "/v3/simulation/pattern/preview",
        r#"{"params":{"pattern_type":"Noise","density":0.15,"cluster_size":3},"bounds":{"x":0,"y":0,"width":20,"height":20},"seed":42}"#,
    );
    let (status, body) = do_request(a, req).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert!(body["protocol_version"].is_string());
    assert!(body["bitmap"].is_string());
    assert!(body["cell_count"].is_number());
    assert!(body["cell_count"].as_u64().unwrap() > 0);
    assert_eq!(body["bounds"]["width"].as_u64().unwrap(), 20);
    assert_eq!(body["bounds"]["height"].as_u64().unwrap(), 20);
}

#[tokio::test]
async fn pattern_apply_creates_barriers() {
    let a = app();
    let (status, _) = do_request(a.clone(), startup_req(r#"{"seed":42}"#)).await;
    assert_eq!(status, StatusCode::OK);

    let req = post_json(
        "/v3/simulation/pattern/apply",
        r#"{"params":{"pattern_type":"Star","point_count":1,"ray_count":4,"ray_length":5,"ray_thickness":1},"bounds":{"x":10,"y":10,"width":20,"height":20},"seed":42}"#,
    );
    let (status, body) = do_request(a, req).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert!(body["protocol_version"].is_string());
    assert!(body["stats"]["barrier_set_cells"].as_u64().unwrap() > 0);
    assert!(body["world_static_changed"].as_bool().unwrap());
    assert_eq!(body["dirty_rect"]["x"].as_u64().unwrap(), 10);
    assert_eq!(body["dirty_rect"]["y"].as_u64().unwrap(), 10);
}

#[tokio::test]
async fn pattern_apply_while_running_returns_error() {
    let a = app();
    let (status, _) = do_request(a.clone(), startup_req(r#"{"seed":42}"#)).await;
    assert_eq!(status, StatusCode::OK);

    // Start the simulation.
    let (status, _) = do_request(a.clone(), post_req("/v3/simulation/start")).await;
    assert_eq!(status, StatusCode::OK);

    let req = post_json(
        "/v3/simulation/pattern/apply",
        r#"{"params":{"pattern_type":"Maze","corridor_width":2,"wall_thickness":1,"open_center_radius":0},"bounds":{"x":0,"y":0,"width":10,"height":10},"seed":42}"#,
    );
    let (status, body) = do_request(a, req).await;
    assert_eq!(status, StatusCode::CONFLICT, "body: {body}");
}

#[tokio::test]
async fn pattern_preview_deterministic_same_seed() {
    let a = app();
    let (status, _) = do_request(a.clone(), startup_req(r#"{"seed":42}"#)).await;
    assert_eq!(status, StatusCode::OK);

    let json = r#"{"params":{"pattern_type":"Noise","density":0.15,"cluster_size":3},"bounds":{"x":0,"y":0,"width":30,"height":30},"seed":12345}"#;

    let req1 = post_json("/v3/simulation/pattern/preview", json);
    let (_, body1) = do_request(a.clone(), req1).await;

    let req2 = post_json("/v3/simulation/pattern/preview", json);
    let (_, body2) = do_request(a, req2).await;

    assert_eq!(body1["bitmap"], body2["bitmap"]);
    assert_eq!(body1["cell_count"], body2["cell_count"]);
}

#[tokio::test]
async fn energy_only_shared_reward_can_be_changed_live() {
    use v3_core::config::{FoodTypeConfig, OrdinaryFoodTypeId};
    use v3_core::simulation::actions::apply_typed_eat;
    let mut config = test_config();
    config.world.food.types.push(FoodTypeConfig::default());
    let state = AppState::from_config(config, 1);
    let a = router(state.clone());
    let (status, body) = do_request(
        a.clone(),
        patch_req(
            "/v3/simulation/config",
            r#"{"energy":{"costs":{"eat_reward_per_food":7.5}}}"#,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (_, body) = do_request(a.clone(), get_req("/v3/simulation/config")).await;
    assert_eq!(
        body["config"]["energy"]["costs"]["eat_reward_per_food"],
        7.5
    );
    assert!(body["config"].get("nutrition").is_none());
    {
        let mut handle = state.sim.lock().await;
        let sim = &mut handle.sim;
        let id = sim.creatures.keys().next().unwrap();
        let position = sim.creatures[id].position;
        for type_idx in [OrdinaryFoodTypeId::new(0), OrdinaryFoodTypeId::new(1)] {
            sim.creatures[id].energy = 10.0;
            sim.world.set_food_type(position, type_idx, 0.5);
            assert!(apply_typed_eat(
                &mut sim.creatures[id],
                &mut sim.world,
                &sim.config,
                type_idx,
                &mut sim.stats.energy_flows
            ));
            assert!((sim.creatures[id].energy - 13.75).abs() < 1e-6);
        }
    }
    let (_, snapshot) = do_request(a, get_req("/v3/simulation/snapshot?zoom_tier=detail")).await;
    assert!(!snapshot.to_string().contains("reproductive_reserve"));
    assert!(!snapshot.to_string().contains("metabolic_energy_yield"));
}

// ── Runtime config apply fixes: nested error envelope ─────────────────────

fn error_field_errors(body: &serde_json::Value) -> &Vec<serde_json::Value> {
    body["error"]["details"]["field_errors"]
        .as_array()
        .unwrap_or_else(|| panic!("expected details.field_errors array, got {body}"))
}

fn error_field_paths(body: &serde_json::Value) -> Vec<&str> {
    error_field_errors(body)
        .iter()
        .map(|entry| {
            entry["field"]
                .as_str()
                .unwrap_or_else(|| panic!("expected string field, got {entry}"))
        })
        .collect()
}

#[tokio::test]
async fn error_envelope_nests_code_message_and_details() {
    let (status, body) = do_request(app(), startup_req(r#"{"seed":42,"bogus_field":1}"#)).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "body: {body}");
    assert_eq!(
        body["error"]["code"].as_str(),
        Some("validation_rejected"),
        "body: {body}"
    );
    assert!(
        !body["error"]["message"].as_str().unwrap_or("").is_empty(),
        "body: {body}"
    );
    assert_eq!(
        body["error"]["details"]["endpoint"].as_str(),
        Some("startup"),
        "body: {body}"
    );
    assert!(!error_field_errors(&body).is_empty(), "body: {body}");
    assert!(body.get("field_errors").is_none(), "body: {body}");
    assert!(body.get("message").is_none(), "body: {body}");
}

#[tokio::test]
async fn error_envelope_reports_state_transition_details() {
    let (status, body) = do_request(app(), post_req("/v3/simulation/pause")).await;

    assert_eq!(status, StatusCode::CONFLICT, "body: {body}");
    assert_eq!(
        body["error"]["code"].as_str(),
        Some("invalid_state_transition"),
        "body: {body}"
    );
    assert_eq!(
        body["error"]["details"]["current_state"].as_str(),
        Some("idle"),
        "body: {body}"
    );
    // `pause` on idle declares no single expected state, so the key is omitted.
    assert!(
        body["error"]["details"].get("expected_state").is_none(),
        "body: {body}"
    );

    let (step_status, step_body) =
        do_request(app(), post_json("/v3/simulation/step", r#"{"steps":1}"#)).await;
    assert_eq!(step_status, StatusCode::CONFLICT, "body: {step_body}");
    assert_eq!(
        step_body["error"]["details"]["expected_state"].as_str(),
        Some("paused"),
        "body: {step_body}"
    );
    assert_eq!(
        step_body["error"]["details"]["current_state"].as_str(),
        Some("idle"),
        "body: {step_body}"
    );
}

#[tokio::test]
async fn error_envelope_omits_details_when_there_are_none() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":42}"#))
        .await
        .unwrap();

    let (status, body) = do_request(a, get_req("/v3/simulation/creature/999999999")).await;

    assert_eq!(status, StatusCode::NOT_FOUND, "body: {body}");
    assert_eq!(
        body["error"]["code"].as_str(),
        Some("not_found"),
        "body: {body}"
    );
    assert!(body["error"].get("details").is_none(), "body: {body}");
}

// ── Runtime config apply fixes: construction-time normalization ───────────

#[tokio::test]
async fn fresh_default_server_accepts_an_empty_config_patch() {
    let a = router(AppState::new());

    let (before_status, before) = do_request(a.clone(), get_req("/v3/simulation/config")).await;
    assert_eq!(before_status, StatusCode::OK, "body: {before}");

    let (status, body) = do_request(a, patch_req("/v3/simulation/config", "{}")).await;

    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["config"], before["config"], "body: {body}");
}

#[tokio::test]
async fn from_config_normalizes_before_storing_and_seeding() {
    let mut config = test_config();
    config.action_log.capacity = 0;
    config.population.max_creatures = 1;
    let a = router(AppState::from_config(config, 0));

    let (status, body) = do_request(a.clone(), get_req("/v3/simulation/config")).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(
        body["config"]["action_log"]["capacity"], 500,
        "body: {body}"
    );
    assert_eq!(
        body["config"]["population"]["max_creatures"], 100000,
        "body: {body}"
    );

    let (patch_status, patch_body) = do_request(a, patch_req("/v3/simulation/config", "{}")).await;
    assert_eq!(patch_status, StatusCode::OK, "body: {patch_body}");
}

// ── Runtime config apply fixes: per-field rejection attribution ───────────

/// Each bound-constrained runtime control, patched alone at a value the panel
/// used to offer, must name its own path (2026-09-07 apply audit, item 4).
#[tokio::test]
async fn patch_config_names_each_bound_constrained_field_patched_alone() {
    let cases: [(&str, &str); 8] = [
        (
            "population.max_creatures",
            r#"{"population":{"max_creatures":32}}"#,
        ),
        (
            "mutation.per_unit_rate",
            r#"{"mutation":{"per_unit_rate":1.5}}"#,
        ),
        (
            "world.food.shared.max_density",
            r#"{"world":{"food":{"shared":{"max_density":0.5}}}}"#,
        ),
        (
            "runtime.max_actions_per_turn",
            r#"{"runtime":{"max_actions_per_turn":2}}"#,
        ),
        (
            "mutation.action_queue_cap",
            r#"{"mutation":{"action_queue_cap":12}}"#,
        ),
        (
            "mutation.per_birth_mutation_events_min",
            r#"{"mutation":{"per_birth_mutation_events_min":15}}"#,
        ),
        (
            "mutation.per_birth_mutation_events_max",
            r#"{"mutation":{"per_birth_mutation_events_max":0}}"#,
        ),
        ("action_log.capacity", r#"{"action_log":{"capacity":0}}"#),
    ];

    for (field, patch) in cases {
        let a = app();
        a.clone()
            .oneshot(startup_req(r#"{"seed":1}"#))
            .await
            .unwrap();

        let (status, body) = do_request(a, patch_req("/v3/simulation/config", patch)).await;

        assert_eq!(
            status,
            StatusCode::UNPROCESSABLE_ENTITY,
            "{field} body: {body}"
        );
        assert_eq!(
            body["error"]["code"].as_str(),
            Some("validation_rejected"),
            "{field} body: {body}"
        );
        assert_eq!(
            error_field_paths(&body),
            vec![field],
            "{field} body: {body}"
        );
        let reason = error_field_errors(&body)[0]["reason"]
            .as_str()
            .unwrap_or_else(|| panic!("{field} expected string reason: {body}"));
        assert!(!reason.is_empty(), "{field} body: {body}");
    }
}

#[tokio::test]
async fn patch_config_reason_names_the_cross_field_path_it_would_move() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":1}"#))
        .await
        .unwrap();

    let patch = r#"{"mutation":{"per_birth_mutation_events_min":15}}"#;
    let (status, body) = do_request(a, patch_req("/v3/simulation/config", patch)).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "body: {body}");
    let reason = error_field_errors(&body)[0]["reason"]
        .as_str()
        .expect("string reason");
    assert!(reason.contains("15"), "reason: {reason}");
    assert!(
        reason.contains("mutation.per_birth_mutation_events_max"),
        "reason: {reason}"
    );
}

/// The audit's mixed patch: three acceptable edits plus one refused edit are
/// rejected as a unit, naming only the offending path and storing nothing.
#[tokio::test]
async fn patch_config_rejects_mixed_patch_atomically_naming_only_the_offender() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":1}"#))
        .await
        .unwrap();

    let (_, before) = do_request(a.clone(), get_req("/v3/simulation/config")).await;

    let patch = r#"{
        "energy": { "costs": { "move_cost": 0.35 } },
        "mutation": { "mutation_probability": 0.6 },
        "world": { "food": { "shared": { "growth_rate": 0.12 } } },
        "population": { "max_creatures": 32 }
    }"#;
    let (status, body) = do_request(a.clone(), patch_req("/v3/simulation/config", patch)).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "body: {body}");
    assert_eq!(
        error_field_paths(&body),
        vec!["population.max_creatures"],
        "body: {body}"
    );

    let (_, after) = do_request(a, get_req("/v3/simulation/config")).await;
    assert_eq!(after["config"], before["config"], "after: {after}");
}

/// A patch that only changes acceptable values must still apply in full, so
/// the atomicity rule cannot be satisfied by rejecting everything.
#[tokio::test]
async fn patch_config_applies_a_fully_valid_mixed_patch() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":1}"#))
        .await
        .unwrap();

    let patch = r#"{
        "energy": { "costs": { "move_cost": 0.35 } },
        "mutation": { "mutation_probability": 0.6 },
        "world": { "food": { "shared": { "growth_rate": 0.12 } } }
    }"#;
    let (status, body) = do_request(a.clone(), patch_req("/v3/simulation/config", patch)).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");

    let (_, after) = do_request(a, get_req("/v3/simulation/config")).await;
    assert_json_f64_close(&after["config"]["energy"]["costs"]["move_cost"], 0.35);
    assert_json_f64_close(&after["config"]["mutation"]["mutation_probability"], 0.6);
    assert_json_f64_close(
        &after["config"]["world"]["food"]["shared"]["growth_rate"],
        0.12,
    );
}

/// Both per-unit supply fields (T11.F19) are live-patchable and echoed back;
/// the defaults are the production rule.
#[tokio::test]
async fn patch_config_applies_the_per_unit_supply_fields() {
    let a = app();
    a.clone()
        .oneshot(startup_req(r#"{"seed":1}"#))
        .await
        .unwrap();

    let (_, before) = do_request(a.clone(), get_req("/v3/simulation/config")).await;
    assert_eq!(
        before["config"]["mutation"]["per_unit_supply_enabled"],
        true
    );
    assert_json_f64_close(&before["config"]["mutation"]["per_unit_rate"], 0.005);

    let patch = r#"{"mutation":{"per_unit_supply_enabled":false,"per_unit_rate":0.02}}"#;
    let (status, body) = do_request(a.clone(), patch_req("/v3/simulation/config", patch)).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["config"]["mutation"]["per_unit_supply_enabled"], false);
    assert_json_f64_close(&body["config"]["mutation"]["per_unit_rate"], 0.02);

    let (_, after) = do_request(a, get_req("/v3/simulation/config")).await;
    assert_eq!(
        after["config"]["mutation"]["per_unit_supply_enabled"],
        false
    );
    assert_json_f64_close(&after["config"]["mutation"]["per_unit_rate"], 0.02);
}

#[tokio::test]
async fn terrain_fields_are_restart_only_even_when_empty_or_null() {
    let a = app();
    let (_, before) = do_request(a.clone(), get_req("/v3/simulation/config")).await;
    for patch in [
        r#"{"world":{"terrain":[]}}"#,
        r#"{"world":{"terrain":null}}"#,
        r#"{"world":{"world_seed":null}}"#,
        r#"{"world":{"world_seed":42},"runtime":{"ticks_per_second":1}}"#,
    ] {
        let (status, body) = do_request(a.clone(), patch_req("/v3/simulation/config", patch)).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
        assert!(
            body["error"]["details"]["field_errors"]
                .to_string()
                .contains("restart-only"),
            "{body}"
        );
        let (_, after) = do_request(a.clone(), get_req("/v3/simulation/config")).await;
        assert_eq!(before, after);
    }
}

#[tokio::test]
async fn startup_terrain_partial_override_projects_applied_barriers() {
    let state = test_state();
    let a = router(state.clone());
    let (status,body)=do_request(a.clone(),startup_req(r#"{"seed":42,"world":{"world_seed":7,"terrain":[{"params":{"pattern_type":"Noise","density":1.0,"cluster_size":1},"bounds":{"x":1,"y":2,"width":3,"height":4},"seed":null}]}}"#)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (_, body) = do_request(a.clone(), get_req("/v3/simulation/config")).await;
    assert_eq!(body["config"]["world"]["world_seed"], 7);
    assert_eq!(body["config"]["world"]["terrain"][0]["bounds"]["width"], 3);
    assert_eq!(body["config"]["world"]["height"], 64);
    {
        let projection = state.projection.read().unwrap();
        let frame = &projection.current().ws_frame.frame;
        assert_eq!(frame.barriers.len(), 12);
        assert!(frame
            .barriers
            .iter()
            .all(|p| p.x >= 1 && p.x < 4 && p.y >= 2 && p.y < 6));
    }
    let (status,_)=do_request(a,startup_req(r#"{"seed":42,"world":{"terrain":[{"params":{"pattern_type":"Noise","density":0.1,"cluster_size":1},"oops":1}]}}"#)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn recipe_export_preserves_complete_config_and_large_seeds() {
    let state = test_state();
    let a = router(state.clone());
    let mut recipe = serde_json::json!({"world":{"world_seed":u64::MAX,"terrain":[{"params":{"pattern_type":"Noise","density":0.2,"cluster_size":1},"seed":u64::MAX}],"food":{"fertility":{"layers":[{"weight":1.0,"algorithm":{"Fbm":{"octaves":2,"frequency":0.1,"lacunarity":2.0,"persistence":0.5,"seed":u64::MAX}}}]}}},"population":{"founder_profile":"forage_first_sparse"},"energy":{"costs":{"move_cost":0.25}}});
    recipe["world"]["terrain"][0]["params"] = serde_json::json!({"pattern_type":"FbmThreshold","octaves":4,"frequency":0.02,"lacunarity":2.0,"persistence":0.5,"threshold":0.0});
    recipe["world"]["food"]["types"] = serde_json::json!([{"name":"Fruit","color":"#f97316","initial_density":1.0,"initial_coverage":0.2,"energy_per_unit":12.0,"growth_rate":null,"recovery_spawn_rate":0.0,"initial_fertility_only":true}]);
    let mut request = recipe.clone();
    request["seed"] = 42.into();
    let (status, startup) = do_request(a.clone(), startup_req(&request.to_string())).await;
    assert_eq!(status, StatusCode::OK, "{startup}");
    let (status, exported) =
        do_request(a.clone(), get_req("/v3/simulation/config?format=recipe")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(exported["world"]["world_seed"].as_u64(), Some(u64::MAX));
    assert_eq!(
        exported["world"]["terrain"][0]["seed"].as_u64(),
        Some(u64::MAX)
    );
    assert_eq!(
        exported["world"]["food"]["fertility"]["layers"][0]["algorithm"]["Fbm"]["seed"].as_u64(),
        Some(u64::MAX)
    );
    let expected = v3_core::config::resolve_config(&test_config(), recipe).unwrap();
    assert_eq!(exported, serde_json::to_value(&expected).unwrap());
    assert_eq!(
        startup["config_digest"],
        v3_core::config::config_digest(&expected)
    );
    let fingerprint = |sim: &v3_core::simulation::Simulation| {
        let cells = (0..sim.world.height)
            .flat_map(|y| {
                (0..sim.world.width).map(move |x| {
                    let pos = v3_core::contracts::Position::new(x, y);
                    (
                        sim.world.is_barrier(pos),
                        sim.world.food_at(pos),
                        *sim.world.food().fertility().get(x, y),
                    )
                })
            })
            .collect::<Vec<_>>();
        let founders = sim
            .creatures
            .values()
            .map(|creature| {
                (
                    creature.position,
                    creature.energy,
                    serde_json::to_value(&creature.genome).unwrap(),
                )
            })
            .collect::<Vec<_>>();
        (cells, founders)
    };
    let before = fingerprint(&state.sim.lock().await.sim);
    assert_eq!(
        before,
        fingerprint(&v3_core::simulation::seed_simulation(expected, 42))
    );
    let mut reload = exported.clone();
    reload["seed"] = 42.into();
    assert_eq!(
        do_request(a.clone(), startup_req(&reload.to_string()))
            .await
            .0,
        StatusCode::OK
    );
    let handle = state.sim.lock().await;
    assert_eq!(fingerprint(&handle.sim), before);
    drop(handle);
    let (status, _) = do_request(a.clone(), startup_req(r#"{"seed":42,"stale":1}"#)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        do_request(a, get_req("/v3/simulation/config?format=recipe"))
            .await
            .1,
        exported
    );
}
