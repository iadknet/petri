mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::json;
use tower::ServiceExt;

use petri_server::{build_router, sim_loop::run_single_iteration};

#[tokio::test]
async fn snapshot_endpoints_round_trip_world_state() {
    let state = common::fast_state();
    let app = build_router(state.clone());

    let startup_patch = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/simulation/startup-draft")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "initial_creatures": 40,
                    })
                    .to_string(),
                ))
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");
    assert_eq!(startup_patch.status(), StatusCode::OK);

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

    let _ = run_single_iteration(&state).await;
    let _ = run_single_iteration(&state).await;

    let get_snapshot_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/simulation/snapshot")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");
    assert_eq!(get_snapshot_response.status(), StatusCode::OK);
    let snapshot_json = common::read_json(get_snapshot_response).await;
    assert!(snapshot_json["tick"].as_u64().expect("tick should be u64") >= 1);
    assert!(snapshot_json["creatures"].is_array());
    assert!(snapshot_json["creatures"][0]["cognition"].is_object());
    assert!(snapshot_json["creatures"][0]["cognition"]["think_steps"].is_u64());
    assert!(snapshot_json["creatures"][0]["cognition"]["halted"].is_boolean());
    assert!(snapshot_json["creatures"][0]["cognition"]["selected_action"].is_string());
    assert!(snapshot_json["creatures"][0]["cognition"]["selected_confidence"].is_number());

    let load_snapshot_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/simulation/snapshot")
                .header("content-type", "application/json")
                .body(Body::from(snapshot_json.to_string()))
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");
    assert_eq!(load_snapshot_response.status(), StatusCode::OK);
    let load_json = common::read_json(load_snapshot_response).await;
    assert_eq!(load_json["phase"], "running");
    assert_eq!(load_json["tick"], snapshot_json["tick"]);
}

#[tokio::test]
async fn creature_detail_endpoint_returns_last_inputs_outputs_cognition_and_events() {
    let state = common::fast_state();
    let app = build_router(state.clone());

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

    let _ = run_single_iteration(&state).await;

    let snapshot_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/simulation/snapshot")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");
    assert_eq!(snapshot_response.status(), StatusCode::OK);
    let snapshot_json = common::read_json(snapshot_response).await;
    let creature_id = snapshot_json["creatures"][0]["id"]
        .as_u64()
        .expect("snapshot creature should include id");

    let detail_response = app
        .oneshot(
            Request::builder()
                .uri(format!("/simulation/creature/{creature_id}"))
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");
    assert_eq!(detail_response.status(), StatusCode::OK);
    let detail_json = common::read_json(detail_response).await;

    assert_eq!(detail_json["id"], creature_id);
    assert!(detail_json["last_inputs"].is_object());
    assert!(detail_json["last_inputs"]["barrier_direction"].is_number());
    assert!(detail_json["last_inputs"]["barrier_distance"].is_number());
    assert!(detail_json["last_inputs"]["memory_address_norm"].is_number());
    assert!(detail_json["last_inputs"]["prev_action_confidence"].is_array());
    assert!(detail_json["last_inputs"]["max_action_confidence"].is_array());
    assert!(detail_json["last_inputs"]["energy_start_tick"].is_number());
    assert!(detail_json["last_inputs"]["energy_spent_tick"].is_number());
    assert!(detail_json["last_inputs"]["energy_remaining"].is_number());
    assert!(detail_json["last_outputs"].is_object());
    assert!(detail_json["last_outputs"]["memory_write_value"].is_number());
    assert!(detail_json["last_outputs"]["memory_write_enable"].is_number());
    assert!(detail_json["last_outputs"]["memory_address_select"].is_number());
    assert!(detail_json["last_outputs"]["no_op"].is_number());
    assert!(detail_json["last_outputs"]["halt"].is_number());
    assert!(detail_json["last_memory_head"].is_object());
    assert!(detail_json["last_memory_head"]["address_index"].is_u64());
    assert!(detail_json["last_memory_head"]["read_value"].is_u64());
    assert!(detail_json["last_memory_head"]["write_value"].is_u64());
    assert!(detail_json["last_memory_head"]["write_applied"].is_boolean());
    assert!(detail_json["cognition"].is_object());
    assert!(detail_json["cognition"]["think_steps"].is_u64());
    assert!(detail_json["cognition"]["halted"].is_boolean());
    assert!(detail_json["cognition"]["selected_action"].is_string());
    assert!(detail_json["cognition"]["selected_confidence"].is_number());
    assert!(detail_json["events"].is_array());
    assert!(detail_json["node_count"].as_u64().is_some());
    assert!(detail_json["phenotype_color"].is_array());
    assert_eq!(
        detail_json["phenotype_color"]
            .as_array()
            .expect("phenotype_color should be rgb array")
            .len(),
        3
    );
}

#[tokio::test]
async fn creature_detail_endpoint_returns_not_found_for_unknown_id() {
    let state = common::fast_state();
    let app = build_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/simulation/creature/999999")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let err_json = common::read_json(response).await;
    assert_eq!(err_json["code"], "creature_not_found");
}
