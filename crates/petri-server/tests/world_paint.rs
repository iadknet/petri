mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::json;
use tower::ServiceExt;

use petri_server::build_router;

#[tokio::test]
async fn world_paint_rejects_running_phase() {
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
        .expect("start request should succeed");

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/simulation/world/paint")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "action": "stroke",
                        "tool": "food",
                        "brush_half_extent": 0,
                        "points": [{"x": 2, "y": 2}]
                    })
                    .to_string(),
                ))
                .expect("request should build"),
        )
        .await
        .expect("paint request should complete");

    assert_eq!(response.status(), StatusCode::CONFLICT);
    let err_json = common::read_json(response).await;
    assert_eq!(err_json["code"], "paint_phase_not_editable");
}

#[tokio::test]
async fn paused_world_paint_stroke_mutates_active_world() {
    let state = common::fast_state();
    let app = build_router(state);

    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/simulation/startup-draft")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "initial_food_density": 0.0
                    })
                    .to_string(),
                ))
                .expect("request should build"),
        )
        .await
        .expect("startup patch should succeed");

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
        .expect("start request should succeed");

    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/config")
                .header("content-type", "application/json")
                .body(Body::from(json!({"paused": true}).to_string()))
                .expect("request should build"),
        )
        .await
        .expect("pause patch should succeed");

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/simulation/world/paint")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "action": "stroke",
                        "tool": "food",
                        "brush_half_extent": 0,
                        "points": [{"x": 1, "y": 1}]
                    })
                    .to_string(),
                ))
                .expect("request should build"),
        )
        .await
        .expect("paint request should complete");

    assert_eq!(response.status(), StatusCode::OK);
    let body = common::read_json(response).await;
    assert_eq!(body["phase"], "paused");
    assert_eq!(body["stats"]["food_set_cells"], 1);
    let width = body["frame"]["width"]
        .as_u64()
        .expect("frame width should be u64") as usize;
    let food = body["frame"]["food"]
        .as_array()
        .expect("frame food should be array");
    let idx = width + 1;
    assert!(food[idx].as_u64().expect("food value should be u64") > 0);
}

#[tokio::test]
async fn idle_paint_persists_across_start_and_restart() {
    let state = common::fast_state();
    let app = build_router(state);
    let x = 7_u32;
    let y = 8_u32;

    let paint_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/simulation/world/paint")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "action": "stroke",
                        "tool": "barrier",
                        "brush_half_extent": 0,
                        "points": [{"x": x, "y": y}],
                        "idle_preview_mode": "paint_layer"
                    })
                    .to_string(),
                ))
                .expect("request should build"),
        )
        .await
        .expect("paint request should complete");
    assert_eq!(paint_response.status(), StatusCode::OK);

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
        .expect("start request should succeed");

    let first_snapshot = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/simulation/snapshot")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("snapshot request should complete");
    assert_eq!(first_snapshot.status(), StatusCode::OK);
    let first_json = common::read_json(first_snapshot).await;
    let width = first_json["config"]["width"]
        .as_u64()
        .expect("config width should exist") as usize;
    let idx = y as usize * width + x as usize;
    let first_barriers = first_json["cells_barrier"]
        .as_array()
        .expect("cells_barrier should be array");
    assert_eq!(first_barriers[idx], true);

    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/simulation/restart")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("restart request should succeed");

    let second_snapshot = app
        .oneshot(
            Request::builder()
                .uri("/simulation/snapshot")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("snapshot request should complete");
    let second_json = common::read_json(second_snapshot).await;
    let second_barriers = second_json["cells_barrier"]
        .as_array()
        .expect("cells_barrier should be array");
    assert_eq!(second_barriers[idx], true);
}

#[tokio::test]
async fn idle_paint_layer_clips_when_dimensions_shrink() {
    let state = common::fast_state();
    let app = build_router(state);

    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/simulation/world/paint")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "action": "stroke",
                        "tool": "barrier",
                        "brush_half_extent": 0,
                        "points": [{"x": 80, "y": 80}],
                        "idle_preview_mode": "paint_layer"
                    })
                    .to_string(),
                ))
                .expect("request should build"),
        )
        .await
        .expect("paint request should complete");

    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/simulation/startup-draft")
                .header("content-type", "application/json")
                .body(Body::from(json!({"width": 60, "height": 60}).to_string()))
                .expect("request should build"),
        )
        .await
        .expect("startup patch should complete");

    let preview = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/simulation/world/paint")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "action": "preview",
                        "idle_preview_mode": "paint_layer"
                    })
                    .to_string(),
                ))
                .expect("request should build"),
        )
        .await
        .expect("preview request should complete");
    assert_eq!(preview.status(), StatusCode::OK);
    let body = common::read_json(preview).await;
    assert_eq!(body["frame"]["width"], 60);
    let barrier_bits = body["frame"]["barrier_bits"]
        .as_array()
        .expect("barrier_bits should be array")
        .iter()
        .map(|v| v.as_u64().expect("barrier byte should be u64") as u8)
        .collect::<Vec<_>>();
    assert!(barrier_bits.iter().all(|byte| *byte == 0));
}

#[tokio::test]
async fn paint_clear_all_works_in_idle_and_paused() {
    let state = common::fast_state();
    let app = build_router(state);

    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/simulation/world/paint")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "action": "stroke",
                        "tool": "food",
                        "brush_half_extent": 0,
                        "points": [{"x": 2, "y": 2}]
                    })
                    .to_string(),
                ))
                .expect("request should build"),
        )
        .await
        .expect("idle paint should complete");

    let idle_clear = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/simulation/world/paint")
                .header("content-type", "application/json")
                .body(Body::from(json!({"action": "clear_all"}).to_string()))
                .expect("request should build"),
        )
        .await
        .expect("idle clear should complete");
    assert_eq!(idle_clear.status(), StatusCode::OK);
    let idle_body = common::read_json(idle_clear).await;
    assert!(
        idle_body["stats"]["food_cleared_cells"]
            .as_u64()
            .expect("food_cleared should be u64")
            > 0
    );

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
        .expect("start should succeed");

    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/config")
                .header("content-type", "application/json")
                .body(Body::from(json!({"paused": true}).to_string()))
                .expect("request should build"),
        )
        .await
        .expect("pause should succeed");

    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/simulation/world/paint")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "action": "stroke",
                        "tool": "barrier",
                        "brush_half_extent": 0,
                        "points": [{"x": 3, "y": 3}]
                    })
                    .to_string(),
                ))
                .expect("request should build"),
        )
        .await
        .expect("paused paint should succeed");

    let paused_clear = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/simulation/world/paint")
                .header("content-type", "application/json")
                .body(Body::from(json!({"action": "clear_all"}).to_string()))
                .expect("request should build"),
        )
        .await
        .expect("paused clear should complete");
    assert_eq!(paused_clear.status(), StatusCode::OK);
    let paused_body = common::read_json(paused_clear).await;
    let barrier_bits = paused_body["frame"]["barrier_bits"]
        .as_array()
        .expect("barrier_bits should be array")
        .iter()
        .map(|v| v.as_u64().expect("barrier byte should be u64") as u8)
        .collect::<Vec<_>>();
    assert!(barrier_bits.iter().all(|byte| *byte == 0));
    let food = paused_body["frame"]["food"]
        .as_array()
        .expect("food should be array")
        .iter()
        .map(|v| v.as_u64().expect("food byte should be u64") as u8)
        .collect::<Vec<_>>();
    assert!(food.iter().all(|value| *value == 0));
}

#[tokio::test]
async fn idle_preview_mode_returns_paint_layer_and_full_startup() {
    let state = common::fast_state();
    let app = build_router(state);

    let paint_layer = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/simulation/world/paint")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "action": "preview",
                        "idle_preview_mode": "paint_layer"
                    })
                    .to_string(),
                ))
                .expect("request should build"),
        )
        .await
        .expect("preview request should complete");
    assert_eq!(paint_layer.status(), StatusCode::OK);
    let paint_layer_body = common::read_json(paint_layer).await;
    assert_eq!(paint_layer_body["phase"], "idle");
    assert_eq!(paint_layer_body["frame"]["population"], 0);
    assert!(paint_layer_body["frame"]["food"]
        .as_array()
        .expect("food should be array")
        .iter()
        .all(|v| v.as_u64().expect("food byte should be u64") == 0));

    let full_startup = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/simulation/world/paint")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "action": "preview",
                        "idle_preview_mode": "full_startup"
                    })
                    .to_string(),
                ))
                .expect("request should build"),
        )
        .await
        .expect("preview request should complete");
    let full_startup_body = common::read_json(full_startup).await;
    assert_eq!(full_startup_body["phase"], "idle");
    assert!(
        full_startup_body["frame"]["population"]
            .as_u64()
            .expect("population should be u64")
            > 0
    );
}

#[tokio::test]
async fn world_paint_returns_bad_request_for_invalid_brush() {
    let state = common::fast_state();
    let app = build_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/simulation/world/paint")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "action": "stroke",
                        "tool": "food",
                        "brush_half_extent": 4,
                        "points": [{"x": 2, "y": 2}]
                    })
                    .to_string(),
                ))
                .expect("request should build"),
        )
        .await
        .expect("request should complete");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let err_json = common::read_json(response).await;
    assert_eq!(err_json["code"], "invalid_paint_request");
}
