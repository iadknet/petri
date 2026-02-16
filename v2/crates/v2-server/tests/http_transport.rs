use axum::body::Body;
use axum::body::to_bytes;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

#[tokio::test]
async fn startup_endpoint_is_available_over_http() {
    let app = v2_server::server::build_router_for_tests();

    let request_body = serde_json::json!({
        "seed": 7,
        "world": {
            "width": 12,
            "height": 8,
            "wrap": true,
            "sensor_radius": 3
        },
        "population": {
            "initial_creatures": 10,
            "max_creatures": 50
        },
        "runtime": {
            "ticks_per_second": 30,
            "max_tick_budget_ms": 16
        }
    })
    .to_string();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v2/simulation/startup")
                .header("content-type", "application/json")
                .body(Body::from(request_body))
                .expect("valid request"),
        )
        .await
        .expect("router call succeeds");

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn startup_post_with_origin_includes_cors_allow_origin() {
    let app = v2_server::server::build_router_for_tests();

    let request_body = serde_json::json!({
        "seed": 7,
        "world": {
            "width": 12,
            "height": 8,
            "wrap": true,
            "sensor_radius": 3
        },
        "population": {
            "initial_creatures": 10,
            "max_creatures": 50
        },
        "runtime": {
            "ticks_per_second": 30,
            "max_tick_budget_ms": 16
        }
    })
    .to_string();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v2/simulation/startup")
                .header("origin", "http://127.0.0.1:4173")
                .header("content-type", "application/json")
                .body(Body::from(request_body))
                .expect("valid request"),
        )
        .await
        .expect("router call succeeds");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get("access-control-allow-origin")
            .and_then(|value| value.to_str().ok()),
        Some("*")
    );
}

#[tokio::test]
async fn cors_preflight_allows_browser_clients() {
    let app = v2_server::server::build_router_for_tests();

    let response = app
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/v2/simulation/start")
                .header("origin", "http://127.0.0.1:4173")
                .header("access-control-request-method", "POST")
                .body(Body::empty())
                .expect("valid preflight request"),
        )
        .await
        .expect("router call succeeds");

    assert!(
        response.status() == StatusCode::NO_CONTENT || response.status() == StatusCode::OK,
        "unexpected preflight status: {}",
        response.status()
    );
    assert_eq!(
        response
            .headers()
            .get("access-control-allow-origin")
            .and_then(|value| value.to_str().ok()),
        Some("*")
    );
    let allow_methods = response
        .headers()
        .get("access-control-allow-methods")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    assert!(
        allow_methods.contains("POST"),
        "preflight should allow POST, got: {allow_methods}"
    );
}

#[tokio::test]
async fn startup_invalid_json_returns_protocol_error_envelope() {
    let app = v2_server::server::build_router_for_tests();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v2/simulation/startup")
                .header("content-type", "application/json")
                .body(Body::from("{not-json"))
                .expect("valid request"),
        )
        .await
        .expect("router call succeeds");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body");
    let payload: serde_json::Value = serde_json::from_slice(&bytes).expect("json envelope");
    assert_eq!(payload["protocol_version"], "v2alpha1");
    assert_eq!(payload["error"]["code"], "invalid_request");
}

#[tokio::test]
async fn startup_tuning_fields_influence_runtime_dynamics() {
    let app = v2_server::server::build_router_for_tests();

    let request_body = serde_json::json!({
        "seed": 7,
        "world": {
            "width": 12,
            "height": 8,
            "wrap": true,
            "sensor_radius": 3
        },
        "population": {
            "initial_creatures": 10,
            "max_creatures": 50
        },
        "runtime": {
            "ticks_per_second": 30,
            "max_tick_budget_ms": 16
        },
        "tuning": {
            "tick": {
                "initial_energy": 6.0,
                "energy_decay_per_tick": 5.0,
                "move_cost": 0.0,
                "food_energy_gain": 0.0,
                "energy_max": 6.0
            }
        }
    })
    .to_string();

    let startup = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v2/simulation/startup")
                .header("content-type", "application/json")
                .body(Body::from(request_body))
                .expect("startup request"),
        )
        .await
        .expect("startup call succeeds");
    assert_eq!(startup.status(), StatusCode::OK);

    let start = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v2/simulation/start")
                .body(Body::empty())
                .expect("start request"),
        )
        .await
        .expect("start call succeeds");
    assert_eq!(start.status(), StatusCode::OK);

    let pause = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v2/simulation/pause")
                .body(Body::empty())
                .expect("pause request"),
        )
        .await
        .expect("pause call succeeds");
    assert_eq!(pause.status(), StatusCode::OK);

    let step = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v2/simulation/step")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"steps":1}"#))
                .expect("step request"),
        )
        .await
        .expect("step call succeeds");
    assert_eq!(step.status(), StatusCode::OK);

    let status = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v2/simulation/status")
                .body(Body::empty())
                .expect("status request"),
        )
        .await
        .expect("status call succeeds");
    assert_eq!(status.status(), StatusCode::OK);

    let bytes = to_bytes(status.into_body(), usize::MAX)
        .await
        .expect("status body");
    let payload: serde_json::Value = serde_json::from_slice(&bytes).expect("json payload");
    let mean_energy = payload["mean_energy"]
        .as_f64()
        .expect("status.mean_energy should be numeric");
    assert!(
        mean_energy < 5.0,
        "tuning should materially lower post-step energy, got {mean_energy}"
    );
}
