use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::body::to_bytes;
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
