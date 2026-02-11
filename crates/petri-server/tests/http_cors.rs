mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use petri_server::build_router;

#[tokio::test]
async fn cors_preflight_allows_vite_dev_origin_for_patch_config() {
    let state = common::fast_state();
    let app = build_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/config")
                .header("origin", "http://127.0.0.1:5173")
                .header("access-control-request-method", "PATCH")
                .header("access-control-request-headers", "content-type")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("access-control-allow-origin"),
        Some(
            &"http://127.0.0.1:5173"
                .parse()
                .expect("header value should parse")
        )
    );
    assert!(response
        .headers()
        .contains_key("access-control-allow-methods"));
    assert!(response
        .headers()
        .contains_key("access-control-allow-headers"));
}

#[tokio::test]
async fn cors_preflight_allows_alternate_vite_dev_origin_for_patch_config() {
    let state = common::fast_state();
    let app = build_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/config")
                .header("origin", "http://127.0.0.1:5174")
                .header("access-control-request-method", "PATCH")
                .header("access-control-request-headers", "content-type")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("access-control-allow-origin"),
        Some(
            &"http://127.0.0.1:5174"
                .parse()
                .expect("header value should parse")
        )
    );
    assert!(response
        .headers()
        .contains_key("access-control-allow-methods"));
    assert!(response
        .headers()
        .contains_key("access-control-allow-headers"));
}
